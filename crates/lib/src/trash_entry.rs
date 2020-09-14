use std::path::{Path, PathBuf};

use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};

use crate::{DIR_COPY_OPT, FILE_COPY_OPT};
use crate::percent_path;
use crate::trash_info::{self, TrashInfo};
use crate::{TRASH_FILE_DIR, TRASH_INFO_DIR};
use snafu::{ensure, OptionExt, ResultExt, Snafu};

/// Represents an entry in the trash directory. Includes the file path and the trash info path.
pub struct TrashEntry {
    info_path: PathBuf,
    file_path: PathBuf,
}

#[derive(Snafu, Debug)]
pub enum Error {
    NotInTrash { path: PathBuf },

    ExistsInfoPath { path: PathBuf },

    ExistsFilePath { path: PathBuf },

    NoFileName { path: PathBuf },

    #[snafu(context(false))]
    DecodePercentPath { source: percent_path::Error },

    #[snafu(context(false))]
    ParseTrashInfo { source: trash_info::Error },

    #[snafu(display("Failed to move path {} to {}: {}", from.display(), to.display(), source))]
    MovePath {
        source: fs_extra::error::Error,
        from: PathBuf,
        to: PathBuf,
    },

    #[snafu(display("Failed to remove path {}: {}", path.display(), source))]
    RemovePath {
        path: PathBuf,
        source: fs_extra::error::Error,
    },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

/// Represents an entry in the trash. Holds the paths of the file and the .trashinfo file that make
/// up one TrashEntry
impl TrashEntry {
    /// Create a new TrashEntry from path
    pub fn new(path: impl AsRef<Path>) -> Result<TrashEntry> {
        let path = path.as_ref();
        // get file name
        let name = path.file_name().context(NoFileName { path })?;
        // get paths
        let info_path = TRASH_INFO_DIR.join(name);
        let file_path = TRASH_FILE_DIR.join(name);
        let trash_entry = TrashEntry {
            info_path,
            file_path,
        };
        trash_entry.is_valid()?;
        Ok(trash_entry)
    }

    /// Check if paths in TrashEntry is valid.
    fn is_valid(self) -> Result<()> {
        ensure!(
            self.info_path.exists(),
            ExistsInfoPath {
                path: self.info_path
            }
        );
        ensure!(
            self.file_path.exists(),
            ExistsFilePath {
                path: self.info_path
            }
        );
        Ok(())
    }

    /// Restores the trash entry
    pub fn restore(self) -> Result<()> {
        self.is_valid()?;
        let original_path = TrashInfo::parse_from_path(self.info_path)?
            .percent_path()
            .decoded()?
            .as_ref();
        move_file_or_dir(self.file_path, original_path)?;
        remove_file_or_dir(self.file_path)?;
        Ok(())
    }

    /// Removes the trash_entry
    pub fn remove(self) -> Result<()> {
        self.is_valid()?;
        remove_file_or_dir(self.info_path)?;
        remove_file_or_dir(self.file_path)?;
        Ok(())
    }

    pub fn parse_trash_info(&self) -> Result<TrashInfo, trash_info::Error> {
        TrashInfo::parse_from_path(self.info_path)
    }
}

pub fn move_file_or_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    if from.is_dir() {
        move_dir(from, to, &DIR_COPY_OPT)
    } else if from.is_file() {
        move_file(from, to, &FILE_COPY_OPT)
    } else {
        panic!("BUG: must be file or directory");
    }.context(MovePath { from, to })
}

pub fn remove_file_or_dir(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        dir::remove(path)
    } else if path.is_file() {
        file::remove(path)
    } else {
        panic!("BUG: must be file or directory");
    }.context(RemovePath { path })
}

