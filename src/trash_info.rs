use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, NaiveDateTime};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use snafu::{OptionExt, ResultExt, Snafu};

use super::parser::TRASH_DATETIME_FORMAT;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "Could not convert path {:#?} to utf-8 str to do percent encoding",
        path
    ))]
    Utf8Percent { path: PathBuf },

    #[snafu(display("Failed to open file with path {}: {}", path.display(), source))]
    FileOpen { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to write to trash info file: {}", source))]
    TrashInfoWrite { source: io::Error },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub struct TrashInfo {
    path: String,
    deletion_date: NaiveDateTime,
}

impl TrashInfo {
    pub fn new(real_path: impl AsRef<Path>, deletion_date: Option<NaiveDateTime>) -> Result<Self> {
        let path = real_path.as_ref();
        let path = path.to_str().context(Utf8Percent { path })?;
        let path = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
        let deletion_date = deletion_date.unwrap_or(Local::now().naive_local());

        Ok(TrashInfo {
            path,
            deletion_date,
        })
    }

    pub fn save(self, outside_path: impl AsRef<Path>) -> Result<()> {
        let mut trash_info_file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .create_new(true)
            .append(false)
            .truncate(false)
            .open(&outside_path)
            .context(FileOpen {
                path: outside_path.as_ref(),
            })?;

        trash_info_file
            .write_all(self.to_string().as_bytes())
            .context(TrashInfoWrite)?;

        Ok(())
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
