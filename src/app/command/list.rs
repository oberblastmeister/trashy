use std::{
    cmp, fs,
    path::{Path, PathBuf},
};

use chrono::{Local, TimeZone};
use clap::Parser;

use anyhow::{bail, Result};
use tabled::{object::Segment, Alignment, Table, Tabled};
use trash::TrashItem;

use crate::{app, filter::FilterArgs, utils};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: QueryArgs,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        let is_atty = atty::is(atty::Stream::Stdout);
        let items = self.query_args.list(false)?;
        display_items(
            items.into_iter(),
            config_args.color_status.merge(is_atty),
            config_args.table_status.merge(is_atty),
            Path::new(""),
        )?;
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
    pub fn list(&self, nonempty: bool) -> Result<Vec<TrashItem>> {
        let filters = self.filter_args.to_filters()?;
        if nonempty && filters.is_empty() {
            bail!("Must match something");
        }
        let items = {
            let mut items = trash::os_limited::list()?;
            if !filters.is_empty() {
                items = items
                    .into_iter()
                    .filter(|item| filters.is_match(item))
                    .collect()
            };
            if self.rev {
                items.sort_by_key(|item| cmp::Reverse(item.time_deleted));
            } else {
                items.sort_by_key(|item| item.time_deleted);
            }
            items
        };
        Ok(match self.n {
            Some(n) => items.into_iter().take(n as usize).collect(),
            None => items,
        })
    }
}

pub fn display_items(
    items: impl ExactSizeIterator<Item = TrashItem>,
    use_color: bool,
    use_table: bool,
    base: &Path,
) -> Result<()> {
    let table = items_to_table(items, use_color, use_table, base)?;
    println!("{table}");
    Ok(())
}

pub fn items_to_table(
    items: impl ExactSizeIterator<Item = TrashItem>,
    use_color: bool,
    use_table: bool,
    base: &Path,
) -> Result<Table> {
    let mut failed = 0;
    let iter = {
        let vec: Vec<_> = items
            .filter_map(|item| match display_item(&item, use_color, base) {
                Ok(s) => Some(s),
                Err(_) => {
                    failed += 1;
                    None
                }
            })
            .collect();
        let len = vec.len();
        vec.into_iter()
            .zip((0..len as u32).rev())
            .map(|(iter, i)| TrashItemDisplay {
                i,
                time: iter.0,
                path: iter.1,
            })
    };
    let mut table = Table::builder(iter);
    if !use_table {
        table.remove_columns();
    };
    let table = table
        .build()
        .with(tabled::Modify::new(Segment::all()).with(Alignment::left()));
    let table = if use_table {
        table.with(tabled::Style::rounded())
    } else {
        table
            // .with(tabled::Header(""))
            .with(tabled::Style::empty())
    };
    Ok(table)
}

pub fn display_item(item: &TrashItem, color: bool, base: &Path) -> Result<(String, String)> {
    let mut displayed_path =
        utils::path::display(&item.original_path().strip_prefix(base).unwrap());
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
        vec![
            String::from("i"),
            String::from("Time"),
            String::from("Path"),
        ]
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
    // format!(
    //     "{} {}, {}",
    //     datetime.format("%B"),
    //     datetime.format("%d"),
    //     datetime.format("%H:%M")
    // )
}

pub fn files_path_from_info_path(info_path: &Path) -> PathBuf {
    let file_name = Path::new(info_path.file_name().unwrap()).with_extension("");
    let mut files_path =
        (|| Some(info_path.parent()?.parent()?.join("files")))().expect("Invalid info_path");
    files_path.push(file_name);
    files_path
}
