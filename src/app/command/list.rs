use std::{
    cmp, fs,
    path::{Path, PathBuf},
};

use chrono::{Local, TimeZone};
use clap::Parser;

use anyhow::{bail, Result};
use tabled::{object::Segment, Alignment, Table, Tabled};
use trash::TrashItem;

use crate::{
    app,
    filter::FilterArgs,
    range_syntax,
    utils::{self, swap},
};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: QueryArgs,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        let items = self.query_args.list(false)?;
        display_items(items.iter(), config_args)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct QueryArgs {
    #[clap(flatten)]
    filter_args: FilterArgs,

    /// Reverse the sorting of trash items
    ///
    /// Normally when 'list' is run, the newest trash items are at the bottom.
    /// This option puts the oldest trash item at the bottom.
    /// This will also affect 'empty' or 'restore' if used in either command.
    /// Examples:
    /// `trash empty --rev -n=10` will delete 10 oldest trash items are deleted.
    #[clap(long, verbatim_doc_comment)]
    rev: bool,

    /// Show 'n' maximum trash items
    ///
    /// This will also affect 'empty' or 'restore' if used in either command.
    /// Examples:
    /// `trash list -n=10` will list the ten newest trashed items.
    /// `trash restore -n=10` will list restore the ten newest trashed items.
    #[clap(short, verbatim_doc_comment)]
    n: Option<u32>,
}

impl QueryArgs {
    pub const CONFLICTS: [&'static str; 6] = ["before", "within", "patterns", "match", "rev", "n"];

    pub fn list(&self, nonempty: bool) -> Result<Vec<TrashItem>> {
        let filters = self.filter_args.to_filters()?;
        if nonempty && filters.is_empty() {
            bail!("Must match something");
        }
        let items = {
            let mut items = trash::os_limited::list()?;
            if !filters.is_empty() {
                items = items.into_iter().filter(|item| filters.is_match(item)).collect()
            };
            if self.rev {
                items.sort_by_key(|item| item.time_deleted);
            } else {
                items.sort_by_key(|item| cmp::Reverse(item.time_deleted));
            }
            items
        };
        Ok(match self.n {
            Some(n) => items.into_iter().take(n as usize).collect(),
            None => items,
        })
    }

    pub fn list_ranged(&self, nonempty: bool, ranges: &str) -> Result<Vec<(u32, TrashItem)>> {
        let ranges = range_syntax::parse_range_set(ranges)?;
        let items = self.list(if !ranges.is_empty() { false } else { nonempty })?;
        let mut new_items = Vec::new();
        for range in ranges {
            if range.start() as usize > items.len() || range.end() as usize > items.len() {
                bail!("Range is out of bounds");
            }
            new_items.extend(
                items[range.to_std()]
                    .iter()
                    .map(utils::clone_trash_item)
                    .zip(range.into_iter())
                    .map(swap),
            );
        }
        Ok(new_items)
    }
}

pub fn display_items<'a>(
    items: impl Iterator<Item = &'a TrashItem>,
    config_args: &app::ConfigArgs,
) -> Result<()> {
    display_indexed_items(items.zip(0..).map(swap), config_args)
}

pub fn display_indexed_items<'a>(
    items: impl Iterator<Item = (u32, &'a TrashItem)>,
    config_args: &app::ConfigArgs,
) -> Result<()> {
    let is_atty = atty::is(atty::Stream::Stdout);
    display_indexed_items_with(
        items,
        config_args.color_status.merge(is_atty),
        config_args.table_status.merge(is_atty),
        Path::new(""),
    )
}

fn display_indexed_items_with<'a>(
    items: impl Iterator<Item = (u32, &'a TrashItem)>,
    use_color: bool,
    use_table: bool,
    base: &Path,
) -> Result<()> {
    let table = indexed_items_to_table(items, use_color, use_table, base)?;
    println!("{table}");
    Ok(())
}

pub fn items_to_table<'a>(
    items: impl Iterator<Item = &'a TrashItem>,
    use_color: bool,
    use_table: bool,
    base: &Path,
) -> Result<Table> {
    indexed_items_to_table(items.zip(0..).map(swap), use_color, use_table, base)
}

pub fn indexed_items_to_table<'a>(
    items: impl Iterator<Item = (u32, &'a TrashItem)>,
    use_color: bool,
    use_table: bool,
    base: &Path,
) -> Result<Table> {
    let mut failed = 0;
    // this isn't actually needless since we need to reverse the items, which can't be done with a single-ended iterator
    #[allow(clippy::needless_collect)]
    let items: Vec<_> = items
        .filter_map(|(i, item)| match display_item(item, use_color, base) {
            Ok(s) => Some((i, s)),
            Err(_) => {
                failed += 1;
                None
            }
        })
        .map(|(i, t)| TrashItemDisplay { i, time: t.0, path: t.1 })
        .collect();
    let mut table = Table::builder(items.into_iter().rev());
    if !use_table {
        table.remove_columns();
    };
    let table = table.build().with(tabled::Modify::new(Segment::all()).with(Alignment::left()));
    let table = if use_table {
        table.with(tabled::Style::modern())
    } else {
        table.with(tabled::Style::empty())
    };
    Ok(table)
}

pub fn display_item(item: &TrashItem, color: bool, base: &Path) -> Result<(String, String)> {
    let mut displayed_path = utils::path::display(item.original_path().strip_prefix(base).unwrap());
    if cfg!(target_os = "linux") && color {
        if let Some(style) = item_lscolors(item)? {
            let ansi_style = style.to_ansi_term_style();
            displayed_path = ansi_style.paint(displayed_path).to_string();
        }
    }
    Ok((display_item_date(item), displayed_path))
}

pub struct TrashItemDisplay {
    i: u32,
    time: String,
    path: String,
}

impl Tabled for TrashItemDisplay {
    const LENGTH: usize = 3;

    fn fields(&self) -> Vec<String> {
        vec![self.i.to_string(), self.time.clone(), self.path.clone()]
    }

    fn headers() -> Vec<String> {
        vec![String::from("i"), String::from("Time"), String::from("Path")]
    }
}

pub fn item_lscolors(item: &TrashItem) -> Result<Option<lscolors::Style>> {
    if cfg!(target_os = "linux") {
        let files_path = files_path_from_info_path(Path::new(&item.id));
        if !files_path.exists() {
            Ok(None)
        } else {
            let meta = fs::metadata(&files_path)?;
            Ok(utils::path::style_for(&item.original_path(), &meta).cloned())
        }
    } else {
        Ok(None)
    }
}

pub fn display_item_date(item: &TrashItem) -> String {
    let datetime = Local.timestamp(item.time_deleted, 0);
    let humantime = chrono_humanize::HumanTime::from(datetime);
    format!("{humantime}")
}

pub fn files_path_from_info_path(info_path: &Path) -> PathBuf {
    let file_name = Path::new(info_path.file_name().unwrap()).with_extension("");
    let mut files_path =
        (|| Some(info_path.parent()?.parent()?.join("files")))().expect("Invalid info_path");
    files_path.push(file_name);
    files_path
}
