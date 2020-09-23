use std::cmp::Ordering;

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
    // iter.map(|trash_entry| {
    //     let trash_info = trash_entry.parse_trash_info();
    //     trash_info
    // })
    .filter_map(|res| ok_log!(res => error!))
    // .sorted()
    .sorted_by(|t1, t2| custom_cmp(t1, t2))
    .map(|trash_info| row_from_trash_info(trash_info))
    .filter_map(|res| ok_log!(res => error!))
    .for_each(|row| {
        table.add_row(row);
    });

    table.printstd();
    Ok(())
}

fn map_trash_entry(trash_entry: TrashEntry) -> Result<(TrashEntry, TrashInfo)> {
    let trash_info = trash_entry.parse_trash_info();
    trash_info
        .map(|trash_info| (trash_entry, trash_info))
        .map_err(Into::into)
}

fn custom_cmp(t1: &(TrashEntry, TrashInfo), t2: &(TrashEntry, TrashInfo)) -> Ordering {
    t1.1.cmp(&t2.1)
}

fn row_from_trash_info(trash_info: TrashInfo) -> Result<Row> {
    let path = trash_info.percent_path().decoded()?;
    let path = path.as_ref();
    let style = LS_COLORS.style_for_path(&path);
    let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
    let colored = format!("{}", ansi_style.paint(path));
    let mut res = format_date(trash_info.deletion_date());
    res.push(Cell::new(&colored));
    Ok(Row::new(res))
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
