use std::borrow::Cow;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};

use crate::trash::{TRASH_FILE_DIR, TRASH_INFO_DIR};
use crate::utils::{self, convert_to_str, find_name, find_names_multiple};
use chrono::{DateTime, Local, NaiveDateTime};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use snafu::{OptionExt, ResultExt, Snafu};

use super::parser::TRASH_DATETIME_FORMAT;

lazy_static! {
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
}

pub const TRASH_INFO_EXT: &'_ str = "trashinfo";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "Could not convert path {:#?} to utf-8 str to do percent encoding",
        path
    ))]
    Utf8PercentEncode { path: PathBuf },

    #[snafu(display(
        "Percent-decoded bytes of {} are not well-formed in UTF-8: {}",
        string,
        source
    ))]
    Utf8PercentDecode {
        string: String,
        source: core::str::Utf8Error,
    },

    #[snafu(display("Failed to open file with path {}: {}", path.display(), source))]
    FileOpen { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to write to trash info file: {}", source))]
    TrashInfoWrite { source: io::Error },

    #[snafu(display("Failed to convert path {} to string {}", path.display(), source))]
    ConvertToStr { source: utils::Error, path: PathBuf },

    #[snafu(display("Failed to move directory from {} to {}: {}", from.display(), to.display(), source))]
    MoveDir {
        source: fs_extra::error::Error,
        from: PathBuf,
        to: PathBuf,
    },

    #[snafu(display("Failed to move file from {} to {}: {}", from.display(), to.display(), source))]
    MoveFile {
        source: fs_extra::error::Error,
        from: PathBuf,
        to: PathBuf,
    },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub struct TrashInfo {
    percent_path: String,
    deletion_date: NaiveDateTime,
}

impl TrashInfo {
    pub(crate) fn new(
        real_path: impl AsRef<Path>,
        deletion_date: Option<NaiveDateTime>,
    ) -> Result<Self> {
        let path = real_path.as_ref();
        let path = path.to_str().context(Utf8PercentEncode { path })?;
        let path = utf8_percent_encode(path, NON_ALPHANUMERIC).to_string();
        let deletion_date = deletion_date.unwrap_or(Local::now().naive_local());

        Ok(TrashInfo {
            percent_path: path,
            deletion_date,
        })
    }

    /// saves the path with the extension .trashinfo
    pub(crate) fn save(self, name: impl AsRef<Path>) -> Result<()> {
        let mut name = name.as_ref().to_path_buf();
        name.set_extension(TRASH_INFO_EXT);

        let mut trash_info_file = OpenOptions::new()
            .read(false)
            .write(true)
            .create(false)
            .create_new(true)
            .append(false)
            .truncate(false)
            .open(&name)
            .context(FileOpen { path: name })?;

        trash_info_file
            .write_all(self.to_string().as_bytes())
            .context(TrashInfoWrite)?;

        Ok(())
    }

    /// Returns the path as a percent encoded string
    pub fn path(&self) -> &str {
        &self.percent_path
    }

    /// Returns the path as a percent decoded string
    pub fn path_decoded(&self) -> Result<Cow<'_, str>> {
        let decoded_str = percent_decode_str(&self.percent_path)
            .decode_utf8()
            .context(Utf8PercentDecode {
                string: &self.percent_path,
            })?;

        Ok(decoded_str)
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

/// Represents the file that is trashed. Includes the file path and the trashinfo file that is also
/// created.
pub struct TrashEntry<'a> {
    path: &'a Path,
    trash_info: TrashInfo,
}

impl<'a> TrashEntry<'a> {
    fn new(
        from: &'a Path,
        trash_info: TrashInfo,
        existing: &[impl AsRef<str>],
    ) -> Result<TrashEntry<'a>> {
        Ok(TrashEntry {
            path: from,
            trash_info,
        })
    }

    /// Saves the trash entry by moving the file path to the trash and saving the trash info that
    /// was created.
    fn save(self, existing: &[impl AsRef<str>]) -> Result<()> {
        let path = self.path;
        let path_str = convert_to_str(path).context(ConvertToStr { path })?;
        let new_name = &*find_name(path_str, existing);

        if path.is_dir() {
            move_dir(path, new_name, &DIR_COPY_OPT).context(MoveDir {
                from: path,
                to: new_name,
            })
        } else if path.is_file() {
            move_file(path, new_name, &FILE_COPY_OPT).context(MoveFile {
                from: path,
                to: new_name,
            })
        } else {
            panic!("BUG: must be file or directory");
        }?;
        self.trash_info.save(new_name)?;

        Ok(())
    }
}
