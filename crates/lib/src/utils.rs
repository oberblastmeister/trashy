use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use log::{debug, error, info, warn};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{DIR_COPY_OPT, FILE_COPY_OPT};
use crate::{TRASH_DIR, TRASH_FILE_DIR, TRASH_INFO_DIR};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The path {} was not valid utf-8", path.display()))]
    Utf8 { path: PathBuf },

    #[snafu(display("Failed to read path {}: {}", path.display(), source))]
    ReadDir { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to read directory entry from path {}: {}", path.display(), source))]
    ReadDirEntry { source: io::Error, path: PathBuf },

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

pub fn convert_to_str(path: &Path) -> Result<&str> {
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

#[cfg(test)]
use tempfile::NamedTempFile;

#[cfg(test)]
use std::ops::Deref;

#[cfg(test)]
pub fn temp_file_iter<'a, T>(
    dir: &'a T,
    amount: usize,
) -> impl Iterator<Item = NamedTempFile> + 'a
where
    T: AsRef<Path> + ?Sized,
{

    let dir = dir.as_ref();
    (0..amount).map(move |_| NamedTempFile::new_in(dir).expect("Failed to create temp file"))
}

#[cfg(test)]
pub fn contains_all_elements<T, U>(v1: Vec<T>, v2: Vec<&U>)
where T: PartialEq + Deref<Target = U>,
      U: PartialEq + ?Sized
{
    assert_eq!(v1.len(), v2.len());
    assert!(v1.into_iter().all(|e| v2.contains(&&*e)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
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
}
