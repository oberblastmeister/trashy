use eyre::{eyre, Result, WrapErr};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::*;
use lscolors::{LsColors, Style};
use prettytable::{Cell, Row, Table};
use structopt::StructOpt;

use trash_lib::trash_entry::{self, read_dir_trash_entries};
use trash_lib::trash_info::TrashInfo;
use trash_lib::{TRASH_FILE_DIR, TRASH_INFO_DIR};

lazy_static! {
    static ref LS_COLORS: LsColors = LsColors::from_env().unwrap_or_default(); 
}

#[derive(StructOpt, Debug, PartialEq)]
pub struct ListOpt {}

pub fn trash_list(_opt: ListOpt) -> Result<()> {
    let res = read_dir_trash_entries();
    let iter = match res {
        Err(ref e) => match e {
            trash_entry::Error::NotFound { .. } => return Err(eyre!("should repeat this process")),
            _ => res?,
        },
        Ok(iter) => iter,
    };
    let mut table = Table::new();

    iter
    // .inspect(|t| println!("{:?}", t))
    .map(|trash_entry| {
        let trash_info = trash_entry.parse_trash_info();
        trash_info
    })
    .inspect(|t| println!("{:?}", t))
    .filter_map(Result::ok)
    // .inspect(|t| println!("{}", t))
    .sorted()
    .map(|trash_info| -> Result<Row> {
        row_from_trash_info(trash_info)
    })
    .filter_map(|res| res.ok())
    .for_each(|row| {
        table.add_row(row);
    });

    table.printstd();
    Ok(())
}

fn row_from_trash_info(trash_info: TrashInfo) -> Result<Row> {
    let path = trash_info.percent_path().decoded()?;
    let path = path.as_ref();
    let style = LS_COLORS.style_for_path(&path);
    let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
    let colored = format!("{}", ansi_style.paint(path));
    Ok(Row::new(vec![Cell::new(&trash_info.deletion_date_string_format()), Cell::new(&colored)]))
}
