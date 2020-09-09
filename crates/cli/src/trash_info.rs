use std::fs;
use std::path::Path;

use chrono::NaiveDateTime;
use prettytable::{Cell, Row, Table};
use snafu::Snafu;

use super::parser::{parse_trash_info, TRASH_DATETIME_FORMAT};

#[derive(Debug, Snafu)]
pub enum Error {
    ParseNaiveDate { date: NaiveDateTime },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub struct TrashInfo {
    pub path: String,
    pub deletion_date: NaiveDateTime,
}

impl TrashInfo {
    fn new(s: &str) -> TrashInfo {
        let trashinfo = parse_trash_info(s).unwrap();
        trashinfo
    }

    pub fn create_row(&self) -> Result<Row> {
        let date_string = format!("{}", self.deletion_date.format(TRASH_DATETIME_FORMAT));
        Ok(Row::new(vec![Cell::new(&date_string), Cell::new(&self.path)]))
    }
}
