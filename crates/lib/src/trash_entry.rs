use std::borrow::Cow;
use std::path::{Path, PathBuf};

use log::{debug, error, info, warn};
use snafu::{ensure, OptionExt, Snafu};

use crate::percent_path::{self, PercentPath};
use crate::trash_info::{self, TrashInfo};
use crate::utils::{self, convert_to_str, move_path, read_dir_path, remove_path};
use crate::{TRASH_FILE_DIR, TRASH_INFO_DIR, TRASH_DIR};

/// Represents an entry in the trash directory. Includes the file path and the trash info path.
pub struct TrashEntry {
    info_path: PathBuf,
    file_path: PathBuf,
}

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("The trash info path {} does not exist", path.display()))]
    ExistsInfoPath {
        path: PathBuf,
    },

    #[snafu(display("The trash file path {} does not exist", path.display()))]
    ExistsFilePath {
        path: PathBuf,
    },

    #[snafu(display("There is not a file name for path {}", path.display()))]
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

    #[snafu(display("Moving path {} into the trash directory when it is already there", path.display()))]
    CreationInTrash { path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

/// Represents an entry in the trash. Holds the paths of the file and the .trashinfo file that make
/// up one TrashEntry
impl TrashEntry {
    /// Constructor for TrashEntry struct. Fails if the name or path is not a valid TrashEntry
    /// name.
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

    /// Create a trash entry from a path not in the trash directory. Will move the path to the
    /// trash files directory. Can fail if attempting to move something already in the trash
    /// directory
    pub(crate) fn create(path: impl AsRef<Path>, existing: &[TrashEntry]) -> Result<TrashEntry> {
        let path = path.as_ref();

        if in_trash_dir(path) {
            CreationInTrash { path }.fail()?;
        }
        let percent_path = PercentPath::from_path(path)?;
        let name = find_name_trash_entry(path, existing)?;
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
                path: &self.file_path
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

pub fn find_name_trash_entry<'a, T>(path: &'a T, existing: &[TrashEntry]) -> Result<Cow<'a, str>>
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

    let res = find_name(path, &existing_names);
    Ok(res)
}

fn find_name<'a>(s: &'a str, existing: &[&str]) -> Cow<'a, str> {
    (0..)
        .map(|n| {
            if n == 0 {
                Cow::Borrowed(s)
            } else {
                Cow::Owned(format!("{}_{}", s, n))
            }
        })
        .find(|new_path| !existing.contains(&&**new_path))
        .expect("BUG: path must be found, iterator is infinite")
}

pub fn in_trash_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().parent()
        .and_then(|p| p.parent())
        .map(|p| p == *TRASH_DIR)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_names_test() {
        assert_eq!(find_name("vim.log", &["vim.log", "vim.log2"]), "vim.log_1");
    }

    #[test]
    fn find_names_test_2_test() {
        assert_eq!(find_name("vim.log", &["vim.log", "vim.log_1"]), "vim.log_2");
    }

    #[test]
    fn find_names_test_none_test() {
        assert_eq!(find_name("vim.log", &[""]), "vim.log");
    }

    #[test]
    fn in_trash_dir_test() {
        assert_eq!(in_trash_dir("/home/brian/.local/share/Trash"), false);
    }

    #[test]
    fn in_trash_dir_files() {
        assert_eq!(in_trash_dir("/home/brian/.local/share/Trash/files/a_file"), true);
    }

    #[test]
    fn ins_trash_dir_info_test() {
        assert_eq!(in_trash_dir("/home/brian/.local/share/Trash/info/another_file"), true);
    }

    #[test]
    fn in_trash_dir3_test() {
        assert_eq!(in_trash_dir("/home/brian/.local/share/Trash/info"), false);
    }

    #[test]
    fn in_trash_dir4_test() {
        assert_eq!(in_trash_dir("/home/brian/.local/share/Trash/files"), false);
    }
}
