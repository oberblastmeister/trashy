mod parser;
mod percent_path;
mod trash_entry;
pub mod trash_info;
mod utils;

use std::io;
use std::path::{Path, PathBuf};

use directories::UserDirs;
use fs_extra::dir;
use fs_extra::file;
use lazy_static::lazy_static;
// use log::{debug, error, info, warn};
use snafu::{ResultExt, Snafu};

use trash_entry::{read_dir_trash_entries, TrashEntry};

lazy_static! {
    static ref USER_DIRS: UserDirs = UserDirs::new().expect("Failed to determine user directories");
    static ref HOME_DIR: &'static Path = USER_DIRS.home_dir();
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
    pub static ref TRASH_DIR: PathBuf = HOME_DIR.join(".local/share/Trash");
    pub static ref TRASH_INFO_DIR: PathBuf = TRASH_DIR.join("info");
    pub static ref TRASH_FILE_DIR: PathBuf = TRASH_DIR.join("files");
}

pub const TRASH_INFO_EXT: &'_ str = "trashinfo";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create new trash entry struct: {}", source))]
    TrashEntryNew {
        source: trash_entry::Error,
    },

    #[snafu(display("Failed to restore trash entry {}", source))]
    TrashEntryRestore {
        source: trash_entry::Error,
    },

    #[snafu(display("Failed to remove trash entry {}", source))]
    TrashEntryRemove {
        source: trash_entry::Error,
    },

    #[snafu(context(false))]
    #[snafu(display("Utils error: {}", source))]
    Utils { source: utils::Error }
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

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

pub fn remove_all() -> Result<()> {
    for trash_entry in read_dir_trash_entries().unwrap() {
        trash_entry.remove().context(TrashEntryRemove)?
    }

    Ok(())
}

/// Put a list of paths into the trash
pub fn put(paths: &[impl AsRef<Path>]) -> Result<()> {
    let mut existing: Vec<_> = read_dir_trash_entries().unwrap().collect();
    for path in paths {
        let trash_entry = TrashEntry::create(path, &existing).unwrap();
        existing.push(trash_entry)
    }

    Ok(())
}
