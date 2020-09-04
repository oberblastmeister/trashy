use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use fs_extra::dir::{self, move_dir};
use fs_extra::file::{self, move_file};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use snafu::{OptionExt, ResultExt, Snafu};

use crate::parser;
use crate::trash_info::{self, TrashInfo};
use crate::utils::{
    self, convert_paths, convert_to_str, convert_to_string, find_names_multiple, read_dir_path,
};

lazy_static! {
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to read {:#?} to string", path))]
    ReadToString { source: io::Error, path: PathBuf },

    #[snafu(display("Failed to parse {:#?} to string: {}", path, source))]
    ParseTrashInfo {
        source: parser::Error,
        path: PathBuf,
    },

    #[snafu(display("Project directories could not be determined"))]
    ProjectDirsDetermine,

    #[snafu(display("{}", source))]
    #[snafu(context(false))]
    Utils { source: utils::Error },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

pub struct Trash {
    trash_dir: PathBuf,
    info_dir: PathBuf,
    file_dir: PathBuf,
}

impl Trash {
    pub fn new() -> Result<Trash> {
        let project_dirs = ProjectDirs::from("rs", "", "trash").context(ProjectDirsDetermine {})?;
        let data_dir = project_dirs.data_dir();
        let trash_dir = data_dir.join("Trash");
        let file_dir = trash_dir.join("files");
        let info_dir = trash_dir.join("info");

        Ok(Trash {
            trash_dir,
            info_dir,
            file_dir,
        })
    }

    pub fn list_trash_infos(&self, sorted: bool) -> Result<Vec<TrashInfo>> {
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

    pub fn read_dir_info(&self) -> Result<impl Iterator<Item = PathBuf> + '_> {
        read_dir_path(&self.info_dir).map_err(Into::into)
    }

    pub fn read_dir_files(&self) -> Result<impl Iterator<Item = PathBuf> + '_> {
        read_dir_path(&self.file_dir).map_err(Into::into)
    }

    /// Get existing paths that are similar to comparison path
    fn get_existing_paths(&self) -> Result<Vec<String>> {
        let existing = self
            .read_dir_info()?
            // .map(|path| convert_to_string(&path))
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

    pub fn put(&self, paths: &[impl AsRef<Path>]) -> Result<()> {
        let existing = self.get_existing_paths()?;
        let existing = existing.iter().map(|s| &**s).collect();

        let from: Vec<&Path> = paths.into_iter().map(|p| p.as_ref()).collect();
        let to = find_names_multiple(&convert_paths(&from), existing);

        assert_eq!(paths.len(), to.len());

        from.par_iter().zip(to.par_iter()).for_each(|(from, to)| {
            let res = if from.is_dir() {
                move_dir(from, to, &DIR_COPY_OPT)
            } else if from.is_file() {
                move_file(from, to, &FILE_COPY_OPT)
            } else {
                panic!("BUG: must be file or directory");
            };

            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        });

        Ok(())
    }
}
