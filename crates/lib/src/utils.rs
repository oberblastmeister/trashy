use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use snafu::{OptionExt, ResultExt, Snafu};
use log::{debug, error, info, warn};

use crate::{TRASH_DIR, TRASH_FILE_DIR, TRASH_INFO_DIR};
// use crate::utils::{self, *};
use crate::{DIR_COPY_OPT, FILE_COPY_OPT};

#[derive(Debug, Snafu)]
pub enum Error {
    Utf8 { path: PathBuf },

    ReadDir { source: io::Error, path: PathBuf },

    ReadDirEntry { source: io::Error, path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

pub fn find_name<'a>(path: &'a str, existing: &[impl AsRef<str>]) -> Cow<'a, str> {
    let existing: Vec<&str> = existing.into_iter().map(|s| s.as_ref()).collect();
    (0..)
        .map(|n| {
            if n == 0 {
                Cow::Borrowed(path)
            } else {
                Cow::Owned(format!("{}_{}", path, n))
            }
        })
        .find(|new_path| !existing.contains(&&**new_path))
        .expect("BUG: path must be found, iterator is infinite")
}

pub fn find_name_trash(path: &str, existing: &[impl AsRef<str>]) -> PathBuf {
    let name = find_name(path, existing);
    to_trash_file_dir(&*name)
}

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

pub fn convert_to_string(path: &Path) -> Result<String> {
    Ok(convert_to_str(path)?.to_string())
}

pub fn convert_to_str(path: &Path) -> Result<&str> {
    let s = path.to_str().context(Utf8 { path })?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! string_vec {
        ($($x:expr),*) => {
            vec![$(String::from($x)),*]
        }
    }

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
    fn find_names_multiple_test() {
        assert_eq!(
            find_names_multiple(
                &["vim.log", "dude.txt", "vim.log"],
                string_vec!["vim.log", "vim.log_1"]
            ),
            vec!["vim.log_2", "dude.txt", "vim.log_3"]
        );
    }

    #[test]
    fn find_names_multiple2_test() {
        assert_eq!(
            find_names_multiple(
                &["vim.log", "vim.log", "vim.log_2", "vim.log"],
                string_vec!["vim.log", "vim.log_1", "vim.log_3"]
            ),
            vec!["vim.log_2", "vim.log_4", "vim.log_2_1", "vim.log_5"]
        );
    }

    #[test]
    fn find_names_multiple_same_naming_test() {
        assert_eq!(
            find_names_multiple(
                &["vim.log_1", "vim.log_2", "vim.log_3"],
                string_vec!["vim.log_1", "vim.log_2", "vim.log_3"]
            ),
            vec!["vim.log_1_1", "vim.log_2_1", "vim.log_3_1"]
        );
    }
}
