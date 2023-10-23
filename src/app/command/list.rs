use std::borrow::Cow;
use std::io::Write;
use std::num::NonZeroU32;
use std::{
    cmp, fs, io,
    path::{Path, PathBuf},
};

use chrono::{Local, TimeZone};
use clap::Parser;
use tabled::{width::Truncate, Table, Tabled};

use anyhow::{bail, Context, Result};
use trash::TrashItem;

use crate::app::TimeDisplayMode;
use crate::filter::Filters;
use crate::{
    app,
    filter::FilterArgs,
    range_set::RangeSet,
    utils::{self, swap},
};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: QueryArgs,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        display_items(&self.query_args.list(false)?, config_args)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct QueryArgs {
    #[clap(flatten)]
    pub filter_args: FilterArgs,

    /// Reverse the sorting of trash items
    ///
    /// Normally when 'list' is run, the newest trash items are at the bottom.
    /// This option puts the oldest trash item at the bottom.
    /// This will also affect 'empty' or 'restore' if used in either command.
    /// Examples:
    /// 'trashy empty --rev -n=10' will delete 10 oldest trash items are deleted.
    #[arg(long, verbatim_doc_comment)]
    pub rev: bool,

    /// Show 'n' maximum trash items
    ///
    /// This will also affect 'empty' or 'restore' if used in either command.
    /// Examples:
    /// 'trashy list -n=10' will list the ten newest trashed items.
    /// 'trashy restore -n=10' will list restore the ten newest trashed items.
    #[arg(short = 'n', long = "max", verbatim_doc_comment)]
    pub max: Option<NonZeroU32>,
}

impl QueryArgs {
    pub const CONFLICTS: &'static [&'static str] = &[
        "before",
        "within",
        "glob",
        "regex",
        "exact",
        "substring",
        "patterns",
        "match",
        "rev",
        "max",
        "directories",
    ];

    pub fn list(&self, non_empty: bool) -> Result<Vec<TrashItem>> {
        let filters = self.filter_args.to_filters()?;
        if non_empty && filters.is_empty() {
            bail!("Must match something");
        }
        list(self.rev, self.max, filters)
    }

    pub fn list_ranged(&self, non_empty: bool, ranges: RangeSet) -> Result<Vec<(u32, TrashItem)>> {
        let filters = self.filter_args.to_filters()?;
        if non_empty && filters.is_empty() {
            bail!("Must match something");
        }
        list_ranged(self.rev, self.max, filters, ranges)
    }
}

pub fn list_only() -> Result<Vec<TrashItem>> {
    let mut items = trash::os_limited::list()?;
    items.sort_by_key(|item| cmp::Reverse(item.time_deleted));
    Ok(items)
}

pub fn list(rev: bool, max: Option<NonZeroU32>, filters: Filters) -> Result<Vec<TrashItem>> {
    Ok(process_items(rev, max, filters, trash::os_limited::list()?))
}

pub fn process_items(
    rev: bool,
    max: Option<NonZeroU32>,
    filters: Filters,
    items: Vec<TrashItem>,
) -> Vec<TrashItem> {
    let mut items = if !filters.is_empty() {
        items.into_iter().filter(|item| filters.is_match(item)).collect()
    } else {
        items
    };
    if rev {
        items.sort_by_key(|item| item.time_deleted);
    } else {
        items.sort_by_key(|item| cmp::Reverse(item.time_deleted));
    }
    match max {
        Some(n) => items.into_iter().take(n.get() as usize).collect(),
        None => items,
    }
}

pub fn list_ranged(
    rev: bool,
    max: Option<NonZeroU32>,
    filters: Filters,
    ranges: RangeSet,
) -> Result<Vec<(u32, TrashItem)>> {
    let items = list(rev, max, filters)?;
    filter_by_ranges(&items, ranges)
}

pub fn filter_by_ranges(items: &[TrashItem], ranges: RangeSet) -> Result<Vec<(u32, TrashItem)>> {
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

pub fn display_items<'a>(items: &[TrashItem], config_args: &app::ConfigArgs) -> Result<()> {
    display_indexed_items(items.iter().zip(0..items.len() as u32).map(swap), config_args)
}

pub fn display_indexed_items<'a>(
    items: impl DoubleEndedIterator<Item = (u32, &'a TrashItem)> + ExactSizeIterator,
    config_args: &app::ConfigArgs,
) -> Result<()> {
    let is_atty = atty::is(atty::Stream::Stdout);
    display_indexed_items_with(
        items,
        config_args.color_status.merge(is_atty),
        config_args.table_status.merge(is_atty),
        config_args.time_display_mode,
        Path::new(""),
    )
}

fn display_indexed_items_with<'a>(
    items: impl DoubleEndedIterator<Item = (u32, &'a TrashItem)> + ExactSizeIterator,
    use_color: bool,
    use_table: bool,
    time_display_mode: TimeDisplayMode,
    base: &Path,
) -> Result<()> {
    if items.len() == 0 {
        return Ok(());
    }
    let table = indexed_items_to_table(items, use_color, use_table, time_display_mode, base)?;
    writeln!(io::stdout(), "{table}").context("Printing table")?;
    Ok(())
}

pub fn indexed_items_to_table<'a>(
    items: impl DoubleEndedIterator<Item = (u32, &'a TrashItem)>,
    use_color: bool,
    use_table: bool,
    time_display_mode: TimeDisplayMode,
    base: &Path,
) -> Result<Table> {
    let mut failed = 0; // 'failed' does not seem to be read anywhere except 197 line

    // this isn't actually needless since we need to reverse the items, which can't be done with a single-ended iterator
    let items = items
        .filter_map(|(i, item)| match display_item(item, use_color, time_display_mode, base) {
            Ok(s) => Some(TrashItemDisplay { i, time: s.0, path: s.1 }),
            Err(_) => {
                failed += 1;
                None
            }
        })
        .rev();
    let mut table = Table::builder(items);
    if !use_table {
        table.remove_columns();
    };
    use tabled::{object::Segment, Alignment, Modify};
    let mut table = table.build();
    table.with(Modify::new(Segment::all()).with(Alignment::left()));
    if let Some((terminal_size::Width(width), _)) = terminal_size::terminal_size() {
        let width = width as usize;
        table
            .with(Modify::new(Segment::new(.., 2..)).with(Truncate::new(width - 30).suffix("...")));
    }
    if use_table {
        table.with(tabled::Style::rounded());
    } else {
        table.with(tabled::Style::empty());
    }
    Ok(table)
}

pub fn display_item(
    item: &TrashItem,
    color: bool,
    time_display_mode: TimeDisplayMode,
    base: &Path,
) -> Result<(String, String)> {
    let mut displayed_path = utils::path::display(item.original_path().strip_prefix(base).unwrap());
    if cfg!(target_os = "linux") && color {
        if let Some(style) = item_lscolors(item)? {
            let ansi_style = style.to_ansi_term_style();
            displayed_path = ansi_style.paint(displayed_path).to_string();
        }
    }
    Ok((display_item_date(item, time_display_mode), displayed_path))
}

pub struct TrashItemDisplay {
    i: u32,
    time: String,
    path: String,
}

impl Tabled for TrashItemDisplay {
    const LENGTH: usize = 3;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![self.i.to_string().into(), self.time.clone().into(), self.path.clone().into()]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["i".into(), "Time".into(), "Path".into()]
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

pub fn display_item_date(item: &TrashItem, time_display_mode: TimeDisplayMode) -> String {
    let datetime = Local.timestamp(item.time_deleted, 0);
    match time_display_mode {
        TimeDisplayMode::Precise => {
            format!("{}", Local.timestamp(item.time_deleted, 0).format("%d/%m/%Y %H:%M"))
        }
        TimeDisplayMode::Imprecise => {
            let humantime = chrono_humanize::HumanTime::from(datetime);
            format!("{humantime}")
        }
    }
}

pub fn files_path_from_info_path(info_path: &Path) -> PathBuf {
    let file_name = Path::new(info_path.file_name().unwrap()).with_extension("");
    let mut files_path =
        (|| Some(info_path.parent()?.parent()?.join("files")))().expect("Invalid info_path");
    files_path.push(file_name);
    files_path
}
