use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use log::{debug, error, info, warn};
use snafu::{OptionExt, ResultExt, Snafu};
use std::borrow::Cow;

use crate::parser;
use crate::trash_info::{self, TrashInfo};
use crate::utils::{
    self, convert_paths, convert_to_str, convert_to_string, find_name, find_name_trash,
    find_names_multiple, read_dir_path,
};
use crate::{DIR_COPY_OPT, FILE_COPY_OPT};
pub use crate::{TRASH_FILE_DIR, TRASH_INFO_DIR};

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

    #[snafu(display("Failed to read paths inside {} into paths: {}", path.display(), source))]
    ReadDirPath { source: utils::Error, path: PathBuf },

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

    RemoveItems {
        dir: PathBuf,
        source: fs_extra::error::Error,
    },

    #[snafu(display("Failed to save trash info file to {}: {}", path.display(), source))]
    TrashInfoSave {
        source: trash_info::Error,
        path: PathBuf,
    },

    #[snafu(display("Failed to create new trash info struct: {}", source))]
    TrashInfoNew { source: trash_info::Error },

    #[snafu(display("Failed to create new trash entry struct: {}", source))]
    TrashEntryNew { source: trash_info::Error },

    #[snafu(display("Failed to convert path {} to string: {}", path.display(), source))]
    ConvertPath { source: utils::Error, path: PathBuf },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

/// Returns a vector of all the parsed TrashInfo files
pub fn list_trash_info(sorted: bool) -> Result<Vec<TrashInfo>> {
    let mut trash_infos: Vec<_> = read_dir_info()?
        // map paths to trash infos
        .map(|path| {
            let read_to_string_res = fs::read_to_string(&path).context(ReadToString {
                path: &*TRASH_INFO_DIR,
            });
            read_to_string_res.and_then(|s| s.parse::<TrashInfo>().context(ParseTrashInfo { path }))
        })
        // log parse erros
        .inspect(|res| match res {
            Err(e) => warn!("{}", e),
            _ => (),
        })
        // then remove parse errors
        .filter_map(Result::ok)
        .collect();

    if sorted {
        trash_infos.sort_unstable_by(|a, b| a.deletion_date().cmp(&b.deletion_date()));
    }

    Ok(trash_infos)
}

pub fn restore(path: impl AsRef<Path>) {
    todo!()
}

pub fn remove(entry_name: &str) {
    todo!()
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

/// Get existing paths that are similar to comparison path
fn get_existing_paths() -> Result<Vec<String>> {
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
pub fn put_multiple(paths: &[impl AsRef<Path>]) -> Result<()> {
    let existing = get_existing_paths()?;
    let mut existing: Vec<_> = existing.into_iter().map(|s| Cow::from(s)).collect();

    for path in paths {
        let new_path = put_single(path, &existing)?;
        let new_path_str = convert_to_string(&new_path).context(ConvertPath { path: &new_path })?;
        existing.push(Cow::from(new_path_str))
    }

    Ok(())
}

fn put_single(path: impl AsRef<Path>, existing: &[impl AsRef<str>]) -> Result<PathBuf> {
    let path = path.as_ref();
    let path_str = convert_to_str(path).context(ConvertPath { path })?;

    let new_name = &*find_name(path_str, existing);
    let to_file_dir = to_trash_files_dir(new_name);
    dbg!(&to_file_dir);

    // move directory or file
    if path.is_dir() {
        move_dir(path, &to_file_dir, &DIR_COPY_OPT).context(MoveDir {
            from: path,
            to: &to_file_dir,
        })
    } else if path.is_file() {
        move_file(path, &to_file_dir, &FILE_COPY_OPT).context(MoveFile {
            from: path,
            to: &to_file_dir,
        })
    } else {
        panic!("BUG: must be file or directory");
    }?;

    // create trash info file and save it
    let trash_info = TrashInfo::new(path, None).context(TrashInfoNew)?;
    trash_info
        .save(new_name)
        .context(TrashInfoSave { path: &to_file_dir })?;

    Ok(to_file_dir)
}

// /// returns the path of the file if it were trashed
fn to_trash_files_dir(path: impl AsRef<Path>) -> PathBuf {
    let mut trash_dir = TRASH_FILE_DIR.clone();
    trash_dir.push(path.as_ref().file_name().unwrap());
    trash_dir
}

fn multiple_to_trash_files_dir(path: &[impl AsRef<Path>]) -> Vec<PathBuf> {
    path.iter().map(|p| to_trash_files_dir(p)).collect()
}

fn to_trash_info_dir(path: impl AsRef<Path>) -> PathBuf {
    let mut trash_dir = TRASH_INFO_DIR.clone();
    println!("info_dir: {:?}", trash_dir);
    trash_dir.push(path.as_ref().file_name().unwrap());
    trash_dir
}

fn multiple_to_trash_info_dir(path: &[impl AsRef<Path>]) -> Vec<PathBuf> {
    path.iter().map(|p| to_trash_info_dir(p)).collect()
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, Write};
    use tempfile::NamedTempFile;

    use super::*;
    use anyhow::{Context, Result};

    #[test]
    fn put_single_test() -> Result<()> {
        // let mut file = NamedTempFile::new()?;
        let file_path = Path::new("/tmp/test_trash");
        let file = File::create(file_path)?;

        let existing = get_existing_paths()?;

        put_single(file_path, &existing)?;

        let file_name = file_path.file_name().unwrap();
        assert!(
            read_dir_files()?.any(|p| p.to_str().unwrap().contains(file_name.to_str().unwrap()))
        );
        assert!(read_dir_info()?.any(|p| p.to_str().unwrap().contains(file_name.to_str().unwrap())));

        remove_all()?;

        Ok(())
    }
}
