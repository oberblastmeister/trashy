use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::percent_path::PercentPath;
use chrono::{Local, NaiveDateTime};
use snafu::{ResultExt, Snafu};

use super::parser::{self, parse_trash_info, TRASH_DATETIME_FORMAT};
use crate::utils::to_trash_info_dir;
use crate::TRASH_INFO_EXT;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to open file with path {}: {}", path.display(), source))]
    FileOpen {
        source: io::Error,
        path: PathBuf,
    },

    #[snafu(display("Failed to write to trash info file: {}", source))]
    TrashInfoWrite {
        source: io::Error,
    },

    #[snafu(display("Failed to read path {} to a string: {}", path.display(), source))]
    ReadToStr {
        path: PathBuf,
        source: io::Error,
    },

    #[snafu(context(false))]
    ParseTrashInfo {
        source: parser::Error,
    },

    #[snafu(display("Wrong extension for path {}", path.display()))]
    WrongExtension {
        path: PathBuf,
    },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub struct TrashInfo {
    percent_path: PercentPath,
    deletion_date: NaiveDateTime,
}

impl TrashInfo {
    pub(super) fn new(
        percent_path: PercentPath,
        deletion_date: Option<NaiveDateTime>,
    ) -> Result<Self> {
        let deletion_date = deletion_date.unwrap_or(Local::now().naive_local());

        Ok(TrashInfo {
            percent_path,
            deletion_date,
        })
    }

    /// saves the name with the extension .trashinfo
    pub(super) fn save(self, name: &str) -> Result<()> {
        let mut name = PathBuf::from(name);
        name.set_extension(TRASH_INFO_EXT);
        let path = to_trash_info_dir(name);

        let mut trash_info_file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .create_new(true)
            .append(false)
            .truncate(false)
            .open(&path)
            .context(FileOpen { path })?;

        trash_info_file
            .write_all(self.to_string().as_bytes())
            .context(TrashInfoWrite)?;

        Ok(())
    }

    pub(crate) fn parse_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        check_extension(path)?;
        let trash_info = fs::read_to_string(path)
            .context(ReadToStr { path })?
            .parse::<TrashInfo>()?;
        Ok(trash_info)
    }

    /// Returns the path as a percent encoded string
    pub fn percent_path(&self) -> &PercentPath {
        &self.percent_path
    }

    /// Gets the deletion date
    pub fn deletion_date(&self) -> NaiveDateTime {
        self.deletion_date
    }

    /// Gets the deletions date as a string formated using the trash_info_format
    pub fn deletion_date_string_format(&self) -> String {
        format!("{}", self.deletion_date.format(TRASH_DATETIME_FORMAT))
    }
}

/// Checks if the extension is correct or no extension
fn check_extension(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Some(ext) = path.extension() {
        if ext != TRASH_INFO_EXT {
            WrongExtension { path }.fail()?;
        }
    } else {
        WrongExtension { path }.fail()?;
    }
    Ok(())
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
            self.percent_path,
            self.deletion_date_string_format(),
        )
    }
}

impl Ord for TrashInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deletion_date.cmp(&other.deletion_date)
    }
}

impl PartialOrd for TrashInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

