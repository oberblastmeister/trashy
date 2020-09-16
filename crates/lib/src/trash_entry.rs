use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use log::{debug, error, info, warn};
use snafu::{ensure, OptionExt, ResultExt, Snafu};

use crate::percent_path::{self, PercentPath};
use crate::trash_info::{self, TrashInfo};
use crate::utils::{self, convert_to_str, move_path, read_dir_path, remove_path};
use crate::{DIR_COPY_OPT, FILE_COPY_OPT};
use crate::{TRASH_FILE_DIR, TRASH_INFO_DIR};

/// Represents an entry in the trash directory. Includes the file path and the trash info path.
pub struct TrashEntry {
    info_path: PathBuf,
    file_path: PathBuf,
}

#[derive(Snafu, Debug)]
pub enum Error {
    NotInTrash {
        path: PathBuf,
    },

    ExistsInfoPath {
        path: PathBuf,
    },

    ExistsFilePath {
        path: PathBuf,
    },

    NoFileName {
        path: PathBuf,
    },

    #[snafu(context(false))]
    DecodePercentPath {
        source: percent_path::Error,
    },

    #[snafu(context(false))]
    ParseTrashInfo {
        source: trash_info::Error,
    },

    #[snafu(context(false))]
    #[snafu(display("Utils error: {}", source))]
    Utils {
        source: utils::Error,
    },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

/// Represents an entry in the trash. Holds the paths of the file and the .trashinfo file that make
/// up one TrashEntry
impl TrashEntry {
    /// Create a new TrashEntry from a name or path
    pub fn new(name: impl AsRef<Path>) -> Result<TrashEntry> {
        let name = name.as_ref();
        let name = name.file_name().context(NoFileName { path: name })?;
        let info_path = TRASH_INFO_DIR.join(name);
        let file_path = TRASH_FILE_DIR.join(name);
        let trash_entry = TrashEntry {
            info_path,
            file_path,
        };
        trash_entry.is_valid()?;
        Ok(trash_entry)
    }

    pub(crate) fn create(path: impl AsRef<Path>, existing: &[TrashEntry]) -> Result<TrashEntry> {
        let path = path.as_ref();
        let percent_path = PercentPath::from_path(path)?;
        let name = find_name(path, existing)?;
        let name = name.as_ref();
        TrashInfo::new(percent_path, None)?.save(name)?;
        move_path(path, TRASH_FILE_DIR.join(name))?;
        Ok(TrashEntry::new(name)?)
    }

    /// Check if paths in TrashEntry is valid.
    pub fn is_valid(&self) -> Result<()> {
        ensure!(
            self.info_path.exists(),
            ExistsInfoPath {
                path: &self.info_path
            }
        );
        ensure!(
            self.file_path.exists(),
            ExistsFilePath {
                path: &self.info_path
            }
        );
        Ok(())
    }

    /// Restores the trash entry
    pub fn restore(self) -> Result<()> {
        self.is_valid()?;
        let trash_info = TrashInfo::parse_from_path(self.info_path)?;
        let original_path = trash_info.percent_path().decoded()?;

        move_path(&self.file_path, original_path.as_ref())?;
        remove_path(&self.file_path)?;
        Ok(())
    }

    /// Removes the trash_entry
    pub fn remove(self) -> Result<()> {
        self.is_valid()?;
        remove_path(self.info_path)?;
        remove_path(self.file_path)?;
        Ok(())
    }

    pub fn parse_trash_info(&self) -> Result<TrashInfo, trash_info::Error> {
        TrashInfo::parse_from_path(&self.info_path)
    }
}

pub fn read_dir_trash_entries() -> Result<impl Iterator<Item = TrashEntry>> {
    let iter = read_dir_path(&TRASH_FILE_DIR)?
        .map(|path| TrashEntry::new(path))
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        })
        .filter_map(Result::ok);
    Ok(iter)
}

pub fn find_name<'a, T>(path: &'a T, existing: &[TrashEntry]) -> Result<Cow<'a, str>>
where
    T: AsRef<Path> + ?Sized,
{
    let path = convert_to_str(path.as_ref())?;
    let existing_names: Vec<_> = existing
        .into_iter()
        .map(|trash_entry| {
            trash_entry
                .file_path
                .file_name()
                .expect("Need to have file name")
        })
        .map(|p| convert_to_str(p.as_ref()).unwrap())
        .collect();

    let res = (0..)
        .map(|n| {
            if n == 0 {
                Cow::Borrowed(path)
            } else {
                Cow::Owned(format!("{}_{}", path, n))
            }
        })
        .find(|new_path| !existing_names.contains(&&**new_path))
        .expect("BUG: path must be found, iterator is infinite");

    Ok(res)
}
