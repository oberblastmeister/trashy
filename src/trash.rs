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
use crate::trash_info::{self, TrashInfo};
use crate::utils::{
    self, convert_paths, convert_to_str, convert_to_string, find_names_multiple, find_name, read_dir_path,
};

lazy_static! {
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
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
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

pub struct Trash {
    trash_dir: PathBuf,
    info_dir: PathBuf,
    file_dir: PathBuf,
}

impl Trash {
    /// Creates a new trash struct. When created this function takes a snapshot of the current
    /// state of the system and finds the trash directory based on that. Will fail if the trash
    /// directory cannot be determined based on the current state of the system.
    pub fn new() -> Result<Trash> {
        // let project_dirs = ProjectDirs::from("rs", "", "trash").context(ProjectDirsDetermine {})?;
        // let data_dir = project_dirs.data_dir();
        let user_dirs = UserDirs::new().context(UserDirsDetermine)?;
        let home_dir = user_dirs.home_dir();
        let trash_dir = PathBuf::from(home_dir.join(".local/share/Trash"));
        let file_dir = trash_dir.join("files");
        let info_dir = trash_dir.join("info");

        Ok(Trash {
            trash_dir,
            info_dir,
            file_dir,
        })
    }

    /// Returns a vector of all the parsed TrashInfo files
    pub fn list_all(&self, sorted: bool) -> Result<Vec<TrashInfo>> {
        let mut trash_infos: Vec<_> = self
            .read_dir_info()?
            // map paths to trash infos
            .map(|path| {
                let read_to_string_res = fs::read_to_string(&path).context(ReadToString {
                    path: &self.info_dir,
                });
                read_to_string_res
                    .and_then(|s| s.parse::<TrashInfo>().context(ParseTrashInfo { path }))
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
    pub fn read_dir_info(&self) -> Result<impl Iterator<Item = PathBuf> + '_> {
        read_dir_path(&self.info_dir).context(ReadDirPath {
            path: &self.info_dir,
        })
    }

    /// Returns something that iterates over the paths of the file dir in the trash dir.
    pub fn read_dir_files(&self) -> Result<impl Iterator<Item = PathBuf> + '_> {
        read_dir_path(&self.file_dir).context(ReadDirPath {
            path: &self.info_dir,
        })
    }

    /// Get existing paths that are similar to comparison path
    fn get_existing_paths(&self) -> Result<Vec<String>> {
        let existing = self
            .read_dir_info()?
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
    pub fn put_multiple(&self, paths: &[impl AsRef<Path>]) -> Result<()> {
        let existing = self.get_existing_paths()?;

        let from: Vec<&Path> = paths.into_iter().map(|p| p.as_ref()).collect();
        println!("from: {:#?}", from);

        let to = find_names_multiple(&convert_paths(&from), existing)
            .into_iter()
            .map(|s| s.into_owned())
            .collect::<Vec<String>>();

        let to = self.multiple_to_trash_files_dir(&to);
        let to_info = self.multiple_to_trash_info_dir(&to);
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

    fn put_single(&self, from: impl AsRef<Path>, existing: &[impl AsRef<str>]) -> Result<()> {
        let from = from.as_ref();
        let to = find_name(convert_to_str(from)?, existing).into_owned();
        if from.is_dir() {
            move_dir(from, to, &DIR_COPY_OPT).context(MoveDir { from, to })
        } else if from.is_file() {
            move_file(from, to, &FILE_COPY_OPT).context(MoveFile { from, to })
        } else {
            panic!("BUG: must be file or directory");
        }?;
        Ok()
    }

    // /// returns the path of the file if it were trashed
    fn to_trash_files_dir(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut trash_dir = self.file_dir.clone();
        trash_dir.push(path.as_ref().file_name().unwrap());
        trash_dir
    }

    fn multiple_to_trash_files_dir(&self, path: &[impl AsRef<Path>]) -> Vec<PathBuf> {
        path.iter().map(|p| self.to_trash_files_dir(p)).collect()
    }

    fn to_trash_info_dir(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut trash_dir = self.info_dir.clone();
        println!("info_dir: {:?}", trash_dir);
        trash_dir.push(path.as_ref().file_name().unwrap());
        trash_dir
    }

    fn multiple_to_trash_info_dir(&self, path: &[impl AsRef<Path>]) -> Vec<PathBuf> {
        path.iter().map(|p| self.to_trash_info_dir(p)).collect()
    }
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
