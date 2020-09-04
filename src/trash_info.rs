use std::fmt;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::{Local, NaiveDateTime};
use log::{debug, error, info, warn};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use snafu::{OptionExt, ResultExt, Snafu};

use super::parser::{self, parse_trash_info, TRASH_DATETIME_FORMAT};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    #[snafu(display("{}", source))]
    ParseTrash { source: parser::Error },

    #[snafu(display("Path {:#?} is not valid utf-8", path))]
    Utf8Percent { path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub struct TrashInfo {
    pub path: String,
    pub deletion_date: NaiveDateTime,
}

impl TrashInfo {
    fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path = path.to_str().context(Utf8Percent { path })?;
        let path = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
        let deletion_date = Local::now().naive_local();

        Ok(TrashInfo {
            path,
            deletion_date,
        })
    }
}

impl FromStr for TrashInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<TrashInfo> {
        let trash_info = parse_trash_info(s)?;
        Ok(trash_info)
    }
}

impl fmt::Display for TrashInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Trash Info]\nPath={}\nDeletionDate={}",
            self.path,
            self.deletion_date.format(TRASH_DATETIME_FORMAT),
        )
    }
}

pub struct TrashEntry {
    name: PathBuf,
    trash_info: TrashInfo,
}
