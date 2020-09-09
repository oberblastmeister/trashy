use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::trash_info::{self, TrashInfo};
use crate::utils::{move_file_or_dir, remove_file_or_dir};
use crate::{TRASH_DIR, TRASH_FILE_DIR, TRASH_INFO_DIR, TRASH_INFO_EXT};
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
            .path_decoded()?
            .as_ref();
        move_file_or_dir(self.file_path, original_path)?;
        fs::remove_file(self.info_path)?;
        Ok(())
    }

    /// Removes the trash_entry
    pub fn remove(self) -> Result<()> {
        self.is_valid()?;
        fs::remove_file(self.info_path)?;
        remove_file_or_dir(self.file_path)?;
        Ok(())
    }

    pub fn parse_trash_info(&self) -> Result<TrashInfo, trash_info::Error> {
        TrashInfo::parse_from_path(self.info_path)
    }
}
