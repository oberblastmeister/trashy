mod parser;
pub mod percent_path;
pub mod trash_entry;
pub mod trash_info;
mod utils;

use std::io;
use std::path::{Path, PathBuf};

use directories::UserDirs;
use fs_extra::dir;
use fs_extra::file;
use lazy_static::lazy_static;
use log::{error, warn};
use snafu::{ResultExt, Snafu};

use trash_entry::{read_dir_trash_entries, TrashEntry};
use utils::{read_dir_path, remove_path};

lazy_static! {
    static ref USER_DIRS: UserDirs = UserDirs::new().expect("Failed to determine user directories");
    pub(crate) static ref HOME_DIR: &'static Path = USER_DIRS.home_dir();
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
    pub static ref TRASH_DIR: PathBuf = HOME_DIR.join(".local/share/Trash");
    pub static ref TRASH_INFO_DIR: PathBuf = TRASH_DIR.join("info");
    pub static ref TRASH_FILE_DIR: PathBuf = TRASH_DIR.join("files");
}

pub const TRASH_INFO_EXT: &'_ str = "trashinfo";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create new trash entry struct"))]
    TrashEntryNew { source: trash_entry::Error },

    #[snafu(display(
        "Failed to create a new trash entry by moving a file and creating a trash info file"
    ))]
    TrashEntryCreation { source: trash_entry::Error },

    #[snafu(display("Failed to read an iterator of trash entries: {}", source))]
    ReadDirTrashEntry { source: trash_entry::Error },

    #[snafu(display("Failed to restore trash entry {}", source))]
    TrashEntryRestore { source: trash_entry::Error },

    #[snafu(display("Failed to remove trash entry {}", source))]
    TrashEntryRemove { source: trash_entry::Error },

    #[snafu(context(false))]
    #[snafu(display("Utils error: {}", source))]
    Utils { source: utils::Error },

    #[snafu(context(false))]
    ReadDirPath { source: io::Error },

    #[snafu(display("The stray path {} was found that could not be made into a trash entry", path.display()))]
    StrayPath { path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[macro_export]
macro_rules! ok_log {
    ($res:expr => $log_macro:ident!) => {
        match $res {
            Ok(t) => Some(t),
            Err(e) => {
                $log_macro!("{}", e);
                None
            }
        }
    };
    ($res:expr => $print_func:ident) => {
        match $res {
            Ok(t) => Some(t),
            Err(e) => {
                $print_func(format!("{}", e));
                None
            }
        }
    };
}

/// Helper function
pub fn restore(name: impl AsRef<Path>) -> Result<()> {
    Ok(TrashEntry::new(name)
        .context(TrashEntryNew)?
        .restore()
        .context(TrashEntryRestore)?)
}

/// Helper function
pub fn remove(name: impl AsRef<Path>) -> Result<()> {
    Ok(TrashEntry::new(name)
        .context(TrashEntryNew)?
        .remove()
        .context(TrashEntryRemove)?)
}

/// Removes all trash_entries and optionally removes stray files.
pub fn empty(keep_stray: bool) -> Result<()> {
    // remove the the correct trash_entries
    read_dir_trash_entries()
        .context(ReadDirTrashEntry)?
        .map(|trash_entry| trash_entry.remove().context(TrashEntryRemove))
        .filter_map(|res| ok_log!(res => error!))
        .for_each(|_| ());

    // remove the stray files taht does not have a pair
    if !keep_stray {
        read_dir_path(&TRASH_INFO_DIR)?
            .chain(read_dir_path(&TRASH_FILE_DIR)?)
            .inspect(|path| warn!("{}", StrayPath { path }.build()))
            .map(|path| remove_path(path))
            .filter_map(|res| ok_log!(res => error!))
            .for_each(|_| ());
    }

    Ok(())
}

/// Put a list of paths into the trash and returns the newly created trash_entries. Will panic if the
/// paths are empty!
pub fn put(paths: &[impl AsRef<Path>]) -> Result<Vec<TrashEntry>> {
    if paths.is_empty() {
        panic!("Attempting to put empty paths");
    }

    let mut existing: Vec<_> = read_dir_trash_entries()
        .context(ReadDirTrashEntry)?
        .collect();
    let old_trash_entries_end = existing.len() - 1;

    for path in paths {
        let trash_entry = TrashEntry::create(path, &existing).context(TrashEntryCreation)?;
        existing.push(trash_entry)
    }

    existing.drain(..old_trash_entries_end);

    Ok(existing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[should_panic]
    #[test]
    fn put_test_nothing_test() {
        let nothing: [&str; 0] = [];
        let _ = put(&nothing);
    }

    #[ignore]
    #[test]
    fn put_test_single() -> Result<()> {
        let mut tempfile = NamedTempFile::new()?;
        tempfile.write_all(b"this is for the put_test_single")?;
        put(&[tempfile.path()])?;
        // tempfile.close()?; // TODO: fix this failure

        Ok(())
    }
}
