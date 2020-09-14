mod utils;
mod trash_entry_iter;
pub mod trash_info;
mod trash_entry;
mod parser;
mod percent_path;

use lazy_static::lazy_static;
use directories::UserDirs;
use std::path::{Path, PathBuf};

use std::fs;
use std::io;

use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use itertools::Itertools;
use log::{debug, error, info, warn};
use snafu::{OptionExt, ResultExt, Snafu};
use std::borrow::Cow;

use crate::trash_entry::TrashEntry;
use crate::trash_info::TrashInfo;
use crate::utils::{
    convert_to_str, convert_to_string, find_name, read_dir_path,
    to_trash_file_dir, move_file_or_dir
};

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
    #[snafu(display("Failed to read {} to string", path.display()))]
    ReadToString { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to parse {} to string: {}", path.display(), source))]
    ParseTrashInfo {
        source: parser::Error,
        path: PathBuf,
    },

    #[snafu(display("Project directories could not be determined"))]
    ProjectDirsDetermine,

    #[snafu(display("User directories could not be determined"))]
    UserDirsDetermine,

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

    #[snafu(display("Failed to save trash info file to {}: {}", path.display(), source))]
    TrashInfoSave {
        source: trash_info::Error,
        path: PathBuf,
    },

    #[snafu(display("Failed to create new trash info struct: {}", source))]
    TrashInfoNew { source: trash_info::Error },

    #[snafu(display("Failed to create new trash entry struct: {}", source))]
    TrashEntryNew { source: trash_entry::Error },

    #[snafu(display("Failed to convert path {} to string: {}", path.display(), source))]
    ConvertPath { source: utils::Error, path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

/// Returns a iterator of all the parsed TrashInfo files
pub fn read_trash_entries() -> Result<impl Iterator<Item = TrashEntry>> {
    let trash_infos = read_dir_files()?
        // map paths to trash entries
        .map(|path| {
            TrashEntry::new(path).context(TrashEntryNew)
        })
        // log parse erros
        .inspect(|res| match res {
            Err(e) => warn!("{}", e),
            _ => (),
        })
        // then remove parse errors
        .filter_map(Result::ok);
    Ok(trash_infos)
}

pub fn read_trash_entries_sorted() -> Result<impl Iterator<Item = TrashInfo>> {
    Ok(read_trash_entries()?.sorted())
}

/// Helper function
pub fn restore(name: impl AsRef<Path>) {
    TrashEntry::new(name).restore()
}

/// Helper function
pub fn remove(name: impl AsRef<Path>) {
    TrashEntry::new(name).remove()
}

pub fn remove_all() -> Result<()> {
    let infos = read_dir_info()?.collect::<Vec<_>>();
    let files = read_dir_files()?.collect::<Vec<_>>();

    fs_extra::remove_items(&infos).context(RemoveItems {
        dir: &*TRASH_INFO_DIR,
    })?;
    fs_extra::remove_items(&files).context(RemoveItems {
        dir: &*TRASH_FILE_DIR,
    })?;

    Ok(())
}

/// Returns something that iterates over the paths of the info dir in the trash dir.
pub fn read_dir_info() -> Result<impl Iterator<Item = PathBuf>> {
    read_dir_path(&TRASH_INFO_DIR).context(ReadDirPath {
        path: &*TRASH_INFO_DIR,
    })
}

/// Returns something that iterates over the paths of the file dir in the trash dir.
pub fn read_dir_files() -> Result<impl Iterator<Item = PathBuf>> {
    read_dir_path(&TRASH_INFO_DIR).context(ReadDirPath {
        path: &*TRASH_INFO_DIR,
    })
}

pub fn read_names() -> Result<impl Iterator<Item = String>> {
    let iter = read_dir_files()?.map(|p| {
        let file_name = p.file_name().expect("Must have filename");
        file_name.to_os_string().into_string().unwrap()
    });
    Ok(iter)
}

/// Get existing paths that are similar to comparison path
fn get_names() -> Result<Vec<String>> {
    let existing = read_dir_files()?
        // convert pathbuf to string
        .map(|path| convert_to_string(&path))
        // log conversion errors
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        })
        // filter out conversion errors
        .filter_map(Result::ok)
        .collect();
    Ok(existing)
}

/// Put a list of paths into the trash
pub fn put(paths: &[impl AsRef<Path>]) -> Result<()> {
    let mut existing: Vec<_> = read_names()?.map(|s| Cow::from(s)).collect();

    for path in paths {
        let new_name = put_single(path, &existing)?;
        existing.push(Cow::from(new_name))
    }

    Ok(())
}

fn put_single<'a>(path: impl AsRef<Path> + 'a, existing: &[impl AsRef<str>]) -> Result<&'a str> {
    let path = path.as_ref();
    let path_str = convert_to_str(path).context(ConvertPath { path })?;

    let new_name = &*find_name(path_str, existing);
    let to_file_dir = to_trash_file_dir(new_name);
    dbg!(&to_file_dir);

    // move directory or file
    move_file_or_dir(path, &to_file_dir).unwrap();

    // create trash info file and save it
    let trash_info = TrashInfo::new(path, None).context(TrashInfoNew)?;
    trash_info
        .save(new_name)
        .context(TrashInfoSave { path: &to_file_dir })?;

    Ok(new_name)
}
