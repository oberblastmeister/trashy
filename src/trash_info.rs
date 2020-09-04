use std::fmt;

use std::path::{Path, PathBuf};
use chrono::NaiveDateTime;
use log::{debug, error, info, warn};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use snafu::{OptionExt, ResultExt, Snafu};

use super::parser::TRASH_DATETIME_FORMAT;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not convert path {:#?} to utf-8 str to do percent encoding", path))]
    Utf8Percent { path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub struct TrashInfo {
    path: String,
    deletion_date: NaiveDateTime,
}

impl TrashInfo {
    pub fn new(path: impl AsRef<Path>, deletion_date: NaiveDateTime) -> Result<Self> {
        let path = path.as_ref();
        let path = path.to_str().context(Utf8Percent { path })?;
        let path = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();

        Ok(TrashInfo {
            path,
            deletion_date,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn deletion_date(&self) -> NaiveDateTime {
        self.deletion_date
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
