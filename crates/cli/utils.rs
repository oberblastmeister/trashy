use std::result::Result as StdResult;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use trash_lib::trash_entry::{self, TrashEntry};
use chrono::naive::NaiveDateTime;
use lscolors::{LsColors, Style};
use lazy_static::lazy_static;
use trash_lib::{ok_log, TRASH_FILE_DIR, TRASH_INFO_DIR};
use prettytable::{cell, row, Cell, Row, Table};
use trash_lib::trash_info::TrashInfo;
use eyre::{WrapErr, Result};

lazy_static! {
    static ref LS_COLORS: LsColors = LsColors::from_env().unwrap_or_default();
}

pub fn trash_entry_error_context(res: StdResult<TrashEntry, trash_entry::Error>) -> Result<TrashEntry> {
    res.wrap_err("Failed to create trash entry")
}

pub fn map_trash_entry_keep(trash_entry: TrashEntry) -> Result<(TrashEntry, TrashInfo)> {
    let trash_info = trash_entry.parse_trash_info()?;
    Ok((trash_entry, trash_info))
}

pub fn get_metadata(trash_entry: &TrashEntry) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(trash_entry.file_path())?;
    Ok(metadata)
}

pub fn custom_cmp<T, R, U>(t1: &(T, U), t2: &(R, U)) -> Ordering
where
    U: Ord,
{
    t1.1.cmp(&t2.1)
}

pub fn map_to_row(pair: (TrashEntry, TrashInfo)) -> Result<Row> {
    let (trash_entry, trash_info) = pair;
    let metadata = get_metadata(&trash_entry)?;
    let path = trash_info.percent_path().decoded()?;
    let colorized_path = colorize_path(path.as_ref(), &metadata);
    let mut res = format_date(trash_info.deletion_date());
    res.push(Cell::new(&colorized_path));
    Ok(Row::new(res))
}

pub fn colorize_path(path: &str, metadata: &fs::Metadata) -> String {
    let style = LS_COLORS.style_for_path_with_metadata(path, Some(metadata));
    let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
    format!("{}", ansi_style.paint(path))
}

pub fn title_row() -> Row {
    row!["Year", "Month", "Day", "Time", "Path"]
}

pub fn format_date(date: NaiveDateTime) -> Vec<Cell> {
    let year = format!("{}", date.format("%y"));
    let month = format!("{}", date.format("%b"));
    let day = format!("{}", date.format("%d"));
    let time = format!("{}", date.format("%H:%M:%S"));
    vec![
        Cell::new(&year),
        Cell::new(&month),
        Cell::new(&day),
        Cell::new(&time),
    ]
}
