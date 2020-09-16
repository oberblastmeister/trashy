use std::fs::{self, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

use crate::read_dir_files;
use crate::TRASH_FILE_DIR;
use crate::{TrashEntry, TrashInfo};

use log::{debug, error, info, warn};
use snafu::{ResultExt, Snafu};

// #[derive(Debug, Snafu)]
// pub enum Error {
//     #[snafu(display("Failed to read paths inside {} into paths: {}", path.display(), source))]
//     ReadDir { source: io::Error, path: PathBuf },

//     #[snafu(display("Failed to read an entry from path {}: {}", path.display(), source))]
//     ReadDirEntry { source: io::Error, path: PathBuf },
// }

// type Result<T, E = Error> = ::std::result::Result<T, E>;

pub struct TrashEntryIter<T: Iterator<Item = TrashEntry>>(T);

impl<T: Iterator<Item = TrashEntry>> Iterator for TrashEntryIter<T> {
    type Item = TrashEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T: Iterator<Item = TrashEntry>> TrashEntryIter<T> {
    fn new() -> Self {
        let trash_entries = read_dir_path(&TRASH_FILE_DIR).unwrap()
            // .context(ReadDir {
            //     path: &TRASH_FILE_DIR,
            // })
            // map paths to trash entries
            .map(|path| TrashEntry::new(path).unwrap());
            // log parse erros
            // .inspect(|res| match res {
            //     Err(e) => warn!("{}", e),
            //     _ => (),
            // })
            // then remove parse errors
            // .filter_map(Result::ok);

        TrashEntryIter(trash_entries)
    }
}

impl<T: Iterator<Item = TrashEntry>> TrashEntryIter<T> {
    fn info_iter<U: Iterator<Item = (TrashEntry, TrashInfo)>>(self) -> TrashInfoIter<U> {
        let new_iter = self.0.map(|entry| entry.parse_trash_info());
        TrashInfoIter(new_iter)
    }
}

pub struct TrashInfoIter<T: Iterator<Item = (TrashEntry, TrashInfo)>>(T);

impl<T: Iterator<Item = (TrashEntry, TrashInfo)>> Iterator for TrashInfoIter<T> {
    type Item = (TrashEntry, TrashInfo);

    fn next(&mut self) -> Self::Item {
        self.0.next()
    }
}

pub fn read_dir_path<'a>(path: &'a Path) -> Result<impl Iterator<Item = PathBuf> + 'a, ()> {
    let paths = fs::read_dir(path)
        .unwrap()
        // .context(ReadDir { path })
        // context of dir_entry errors
        // .map(move |dent_res| dent_res.context(ReadDirEntry { path }))
        .map(move |dent_res| dent_res.unwrap())
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
