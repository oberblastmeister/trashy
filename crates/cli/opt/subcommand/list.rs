use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use chrono::naive::NaiveDateTime;
use eyre::{eyre, Result, WrapErr};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use lscolors::{LsColors, Style};
use prettytable::{cell, row, Cell, Row, Table};
use structopt::StructOpt;

use trash_lib::trash_entry::{self, read_dir_trash_entries, TrashEntry};
use trash_lib::trash_info::TrashInfo;
use trash_lib::{ok_log, TRASH_FILE_DIR, TRASH_INFO_DIR};

lazy_static! {
    static ref LS_COLORS: LsColors = LsColors::from_env().unwrap_or_default();
}

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {}

pub fn list(_opt: Opt) -> Result<()> {
    let res = read_dir_trash_entries();
    let iter = match res {
        Err(ref e) => match e {
            trash_entry::Error::NotFound { .. } => return Err(eyre!("should repeat this process")),
            _ => res?,
        },
        Ok(iter) => iter,
    };
    let mut table = Table::new();
    table.add_row(header_row());

    iter.map(map_trash_entry)
        .filter_map(|res| ok_log!(res => error!))
        .sorted_by(custom_cmp)
        .map(map_to_meta_data)
        .filter_map(|res| ok_log!(res => error!))
        .map(|(metadata, trash_info)| row_from_trash_info(trash_info, &metadata))
        .filter_map(|res| ok_log!(res => error!))
        .for_each(|row| {
            table.add_row(row);
        });

    table.printstd();
    Ok(())
}

fn map_trash_entry(trash_entry: TrashEntry) -> Result<(TrashEntry, TrashInfo)> {
    let trash_info = trash_entry.parse_trash_info()?;
    Ok((trash_entry, trash_info))
}

fn map_to_meta_data(stuff: (TrashEntry, TrashInfo)) -> Result<(fs::Metadata, TrashInfo)> {
    let metadata = fs::symlink_metadata(stuff.0.file_path())?;
    Ok((metadata, stuff.1))
}

fn custom_cmp<T, U>(t1: &(T, U), t2: &(T, U)) -> Ordering
where
    U: Ord,
{
    t1.1.cmp(&t2.1)
}

fn row_from_trash_info(trash_info: TrashInfo, metadata: &fs::Metadata) -> Result<Row> {
    let path = trash_info.percent_path().decoded()?;
    let colorized_path = colorize_path(path.as_ref(), &metadata);
    let mut res = format_date(trash_info.deletion_date());
    res.push(Cell::new(&colorized_path));
    Ok(Row::new(res))
}

fn colorize_path(path: &str, metadata: &fs::Metadata) -> String {
    let style = LS_COLORS.style_for_path_with_metadata(path, Some(metadata));
    let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
    format!("{}", ansi_style.paint(path))
}

fn header_row() -> Row {
    row!["Year", "Month", "Day", "Time", "Path"]
}

fn format_date(date: NaiveDateTime) -> Vec<Cell> {
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
