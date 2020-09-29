use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use log::{info, warn};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::ok_log;
use crate::{DIR_COPY_OPT, FILE_COPY_OPT, TRASH_INFO_EXT};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to convert path `{}` to a str because it was not valid utf-8", path.display()))]
    Utf8 { path: PathBuf },

    #[snafu(display("Failed to read directory entry from path `{}`", path.display()))]
    ReadDirEntry { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to move path `{}` to `{}`", from.display(), to.display()))]
    MovePath {
        source: fs_extra::error::Error,
        from: PathBuf,
        to: PathBuf,
    },

    #[snafu(display("Failed to remove path `{}`", path.display()))]
    RemovePath {
        path: PathBuf,
        source: fs_extra::error::Error,
    },

    #[snafu(display("The path `{}` did not have a file name", path.display()))]
    NoFileName { path: PathBuf },

    #[snafu(display("The path `{}` has no parent", path.display()))]
    NoParent { path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

/// makes the paths parent directory a new directory
pub fn to_directory(path: impl AsRef<Path>, dir: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    let mut dir = dir.as_ref().to_path_buf();
    let file_name = path.file_name().context(NoFileName { path })?;
    dir.push(file_name);
    Ok(dir)
}

pub fn convert_to_str(path: &Path) -> Result<&str> {
    let s = path.to_str().context(Utf8 { path })?;
    Ok(s)
}

pub(crate) fn move_path(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    let from = from.as_ref();
    let to = to.as_ref();

    if from.is_dir() {
        // first rename the dir because move_dir does not rename the directory while moving unlike
        // move_file, so we have to rename it first manually. Also move dir does not work like
        // move_file. Move dir moves the directory from a path to the inside of another directory.
        info!("Path {} is a dir", from.display());
        let (from, to) = match (
            from.file_name(),
            to.file_name(),
            from.parent().context(NoParent { path: from })?,
        ) {
            // only rename if the old name and new name are different
            (Some(name1), Some(name2), parent) if name1 != name2 => {
                info!("The parent is {}", parent.display());
                let new_from = parent.join(name2);
                info!("Renaming from {} to {}", from.display(), new_from.display());
                fs::rename(from, &new_from).unwrap();
                (Cow::from(new_from), Cow::from(to.parent().unwrap()))
            }
            (.., parent) => (Cow::from(from), Cow::from(to.parent().unwrap())),
        };

        info!("Moving from {} to inside {}", from.display(), to.display());
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

pub(crate) fn read_dir_path<'a>(path: &'a Path) -> io::Result<impl Iterator<Item = PathBuf> + 'a> {
    let paths = fs::read_dir(path)?
        .map(move |dent_res| dent_res.context(ReadDirEntry { path }))
        .filter_map(|res| ok_log!(res => warn!))
        .map(|d| d.path());

    Ok(paths)
}

pub fn add_trash_info_ext(path: PathBuf) -> PathBuf {
    let mut s = path.into_os_string();
    s.push(".");
    s.push(TRASH_INFO_EXT);
    PathBuf::from(s)
}

#[cfg(test)]
use tempfile::NamedTempFile;

#[cfg(test)]
use std::ops::Deref;

#[cfg(test)]
pub fn temp_file_iter<'a, T>(dir: &'a T, amount: usize) -> impl Iterator<Item = NamedTempFile> + 'a
where
    T: AsRef<Path> + ?Sized,
{
    let dir = dir.as_ref();
    (0..amount).map(move |_| NamedTempFile::new_in(dir).expect("Failed to create temp file"))
}

#[cfg(test)]
pub fn contains_all_elements<T, U>(v1: Vec<T>, v2: Vec<&U>)
where
    T: PartialEq + Deref<Target = U>,
    U: PartialEq + ?Sized,
{
    assert_eq!(v1.len(), v2.len());
    assert!(v1.into_iter().all(|e| v2.contains(&&*e)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};
    use log::error;
    use std::fmt::Display;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn read_dir_path_test() -> Result<()> {
        const TEMP_FILE_AMOUNT: usize = 20;

        let tempdir = tempdir()?;
        let tempdir_path = tempdir.path();

        let temp_files: Vec<_> = temp_file_iter(tempdir_path, TEMP_FILE_AMOUNT).collect();
        let temp_file_paths: Vec<_> = temp_files.iter().map(|file| file.path()).collect();

        let paths: Vec<_> = read_dir_path(tempdir_path)?.collect();

        contains_all_elements(paths, temp_file_paths);

        Ok(())
    }

    #[test]
    fn remove_path_dir_test() -> Result<()> {
        let tempdir = tempdir()?;
        let path = tempdir.path();
        remove_path(path)?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn remove_path_file_test() -> Result<()> {
        let tempfile = NamedTempFile::new()?;
        let path = tempfile.path();
        remove_path(path)?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn to_dir_simple_test() -> Result<()> {
        assert_eq!(
            to_directory("a_file", "a_dir")?,
            PathBuf::from("a_dir/a_file")
        );
        Ok(())
    }

    #[test]
    fn to_dir_already_dir_test() -> Result<()> {
        assert_eq!(
            to_directory("/tmp/hello/a_file", "another_directory")?,
            PathBuf::from("another_directory/a_file")
        );
        Ok(())
    }

    fn print_error(s: impl Display) {
        eprintln!("{}", s);
    }

    #[test]
    fn test_macro_test() {
        let res: Result<u8> = Ok(5);
        let op = ok_log!(res => error!);
        assert_eq!(op, Some(5));

        let res2: Result<()> = Err(anyhow!("this is an error"));
        let op2 = ok_log!(res2 => error!);
        assert_eq!(op2, None);

        let res3: Result<()> = Err(anyhow!("this is another error"));
        let op3 = ok_log!(res3 => print_error);
    }
}
