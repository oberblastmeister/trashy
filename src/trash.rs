use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use rayon::prelude::*;
use snafu::{OptionExt, ResultExt, Snafu};

use crate::parser;
use crate::trash_info::{self, TrashEntry, TrashInfo, TRASH_INFO_EXT};
use crate::utils::{
    self, convert_paths, convert_to_str, convert_to_string, find_name, find_names_multiple,
    read_dir_path,
};

lazy_static! {
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
    static ref USER_DIRS: UserDirs = UserDirs::new().expect("Failed to determine user directories");
    static ref HOME_DIR: &'static Path = USER_DIRS.home_dir();
    pub static ref TRASH_DIR: PathBuf = HOME_DIR.join(".local/share/Trash");
    pub static ref TRASH_INFO_DIR: PathBuf = TRASH_DIR.join("info");
    pub static ref TRASH_FILE_DIR: PathBuf = TRASH_DIR.join("files");
}

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
pub fn list_all(sorted: bool) -> Result<Vec<TrashInfo>> {
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

pub fn remove_all() {
    todo!()
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
    let existing = read_dir_info()?
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

    let from: Vec<&Path> = paths.into_iter().map(|p| p.as_ref()).collect();
    println!("from: {:#?}", from);

    let to = find_names_multiple(&convert_paths(&from), existing)
        .into_iter()
        .map(|s| s.into_owned())
        .collect::<Vec<String>>();

    let to = multiple_to_trash_files_dir(&to);
    let to_info = multiple_to_trash_info_dir(&to);
    println!("to: {:#?}", to);
    println!("to_info: {:#?}", to_info);

    assert_eq!(paths.len(), to.len());

    from.iter().zip(to.iter()).for_each(|(from, to)| {
        // let to = to.as_ref();
        let res = if from.is_dir() {
            move_dir(from, to, &DIR_COPY_OPT).context(MoveDir { from, to })
        } else if from.is_file() {
            move_file(from, to, &FILE_COPY_OPT).context(MoveFile { from, to })
        } else {
            panic!("BUG: must be file or directory");
        }
        .and_then(|_n| {
            TrashInfo::new(from, None)
                .context(TrashInfoNew)
                .and_then(|trash_info| {
                    let mut file_name =
                        PathBuf::from(to.file_name().expect("BUG: has to have filename"));
                    file_name.set_extension("trashinfo");
                    trash_info
                        .save(file_name)
                        .context(TrashInfoSave { path: to })
                })
        });

        if let Some(e) = res.as_ref().err() {
            warn!("{}", e);
        }
    });

    Ok(())
}

fn put_single(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    let from = from.as_ref();
    let to = to.as_ref();
    if from.is_dir() {
        move_dir(from, to, &DIR_COPY_OPT).context(MoveDir { from, to })
    } else if from.is_file() {
        move_file(from, to, &FILE_COPY_OPT).context(MoveFile { from, to })
    } else {
        panic!("BUG: must be file or directory");
    }?;
    let trash_entry = TrashEntry::new(
        to.to_owned(),
        convert_to_string(from).context(ConvertPath { path: from })?,
        None,
    )
    .context(TrashEntryNew)?;

    Ok(())
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
    fn put() -> Result<()> {
        let trash = Trash::new()?;
        // let mut file = NamedTempFile::new()?;
        let file_path = Path::new("/tmp/test_trash");
        let file = File::create(file_path)?;
        println!("File path: {}", file_path.display());

        let file_name = file_path.file_name().unwrap();
        trash.put_multiple(&[file_path])?;

        assert!(trash.read_dir_files()?.any(|p| p == file_name));
        assert!(trash.read_dir_info()?.any(|p| p == file_name));

        Ok(())
    }
}
