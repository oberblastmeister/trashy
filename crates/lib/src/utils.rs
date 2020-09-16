use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use log::{debug, error, info, warn};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{TRASH_FILE_DIR, TRASH_INFO_DIR};
// use crate::utils::{self, *};
use crate::{DIR_COPY_OPT, FILE_COPY_OPT};

#[derive(Debug, Snafu)]
pub enum Error {
    Utf8 {
        path: PathBuf,
    },

    ReadDir {
        source: io::Error,
        path: PathBuf,
    },

    ReadDirEntry {
        source: io::Error,
        path: PathBuf,
    },

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

pub fn to_trash_file_dir(path: impl AsRef<Path>) -> PathBuf {
    to_directory(path, &*TRASH_FILE_DIR)
}

fn to_directory<T: AsRef<Path>>(path: T, dir: &Path) -> PathBuf {
    let path = path.as_ref();
    let mut dir = dir.to_path_buf();
    let file_name = path.file_name().expect("BUG: must have filename");
    dir.push(file_name);
    dir
}

pub fn to_trash_info_dir(path: impl AsRef<Path>) -> PathBuf {
    to_directory(path, &TRASH_INFO_DIR)
}

pub fn convert_to_string(path: impl AsRef<Path>) -> Result<String> {
    Ok(convert_to_str(path.as_ref())?.to_string())
}

pub fn convert_to_str<'a>(path: &'a Path) -> Result<&'a str> {
    let s = path.to_str().context(Utf8 { path })?;
    Ok(s)
}

pub(crate) fn move_path(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    if from.is_dir() {
        move_dir(from, to, &DIR_COPY_OPT)
    } else if from.is_file() {
        move_file(from, to, &FILE_COPY_OPT)
    } else {
        panic!("BUG: must be file or directory");
    }
    .context(MovePath { from, to })
}

pub(crate) fn remove_path(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        dir::remove(path)
    } else if path.is_file() {
        file::remove(path)
    } else {
        panic!("BUG: must be file or directory");
    }
    .context(RemovePath { path })
}

pub(crate) fn read_dir_path<'a>(path: &'a Path) -> Result<impl Iterator<Item = PathBuf> + 'a> {
    let paths = fs::read_dir(path)
        .context(ReadDir { path })?
        // context of dir_entry errors
        .map(move |dent_res| dent_res.context(ReadDirEntry { path }))
        // log dir_entry errors
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        })
        // filter out errors
        .filter_map(Result::ok)
        // convert dir_entry to string
        .map(|d| d.path());

    Ok(paths)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     macro_rules! string_vec {
//         ($($x:expr),*) => {
//             vec![$(String::from($x)),*]
//         }
//     }

//     #[test]
//     fn find_names_test() {
//         assert_eq!(find_name("vim.log", &["vim.log", "vim.log2"]), "vim.log_1");
//     }

//     #[test]
//     fn find_names_test_2_test() {
//         assert_eq!(find_name("vim.log", &["vim.log", "vim.log_1"]), "vim.log_2");
//     }

//     #[test]
//     fn find_names_test_none_test() {
//         assert_eq!(find_name("vim.log", &[""]), "vim.log");
//     }

//     #[test]
//     fn find_names_multiple_test() {
//         assert_eq!(
//             find_names_multiple(
//                 &["vim.log", "dude.txt", "vim.log"],
//                 string_vec!["vim.log", "vim.log_1"]
//             ),
//             vec!["vim.log_2", "dude.txt", "vim.log_3"]
//         );
//     }

//     #[test]
//     fn find_names_multiple2_test() {
//         assert_eq!(
//             find_names_multiple(
//                 &["vim.log", "vim.log", "vim.log_2", "vim.log"],
//                 string_vec!["vim.log", "vim.log_1", "vim.log_3"]
//             ),
//             vec!["vim.log_2", "vim.log_4", "vim.log_2_1", "vim.log_5"]
//         );
//     }

//     #[test]
//     fn find_names_multiple_same_naming_test() {
//         assert_eq!(
//             find_names_multiple(
//                 &["vim.log_1", "vim.log_2", "vim.log_3"],
//                 string_vec!["vim.log_1", "vim.log_2", "vim.log_3"]
//             ),
//             vec!["vim.log_1_1", "vim.log_2_1", "vim.log_3_1"]
//         );
//     }

// }
