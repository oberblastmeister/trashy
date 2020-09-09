use std::fs::{self, DirEntry};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use prettytable::{Table, Row, Cell};
use lscolors::{LsColors, Style};
use log::*;
use rayon::prelude::*;
use snafu::{ResultExt, Snafu};
use structopt::StructOpt;

use crate::parser::{self, parse_trash_info};

const INFO_DIR: &'static str = "/home/brian/.local/share/Trash/info";

#[derive(StructOpt, Debug)]
pub struct ListOpts {

}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to create all the directories of path {}: {}", path, source))]
    CreateDirAll { source: io::Error, path: String },

    #[snafu(display("Failed to read directory {}: {}", path, source))]
    ReadDir { source: io::Error, path: String },

    #[snafu(display("Failed to read directory {} a second time: {}", path, source))]
    ReadDirSecond { source: io::Error, path: String },

    #[snafu(display("Failed to read {} to string: {}", path, source))]
    ReadToString { source: io::Error, path: String },

    #[snafu(display("Failed to parse {}:\n\n{}", path, source))]
    ParseTrashInfo { source: parser::Error, path: String },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

pub fn trash_list() -> Result<()> {
    let read_dir = match fs::read_dir(INFO_DIR) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all("~/.local/share/Trash/info").context(CreateDirAll {
                path: INFO_DIR.to_string(),
            })?;

            fs::read_dir(INFO_DIR).context(ReadDirSecond {
                path: INFO_DIR.to_string(),
            })?
        }
        Ok(readir) => readir,
        e => e.context(ReadDir {
            path: INFO_DIR.to_string(),
        })?,
    };

    let dir_entries: Vec<_> = read_dir
        .into_iter()
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("Failed to read an entry from {}: {}", INFO_DIR, e)
            }
        })
        .filter_map(Result::ok)
        .collect();

    let lscolors = LsColors::from_env().unwrap_or_default();

    let mut trash_infos: Vec<_> = dir_entries
        .into_par_iter()
        .map(|dir_entry| {
            let path = dir_entry.path();
            // TODO: fix unwrap
            let read_to_string_res = fs::read_to_string(&path).context(ReadToString {
                path: path.to_str().unwrap().to_string(),
            });
            read_to_string_res.and_then(|s| {
                parse_trash_info(&s).context(ParseTrashInfo {
                    path: path.to_str().unwrap().to_string(),
                })
            })
        })
        .inspect(|res| match res {
            Err(e) => warn!("{}", e),
            _ => (),
        })
        .filter_map(Result::ok)
        .map(|mut trash_info| {
            let path = trash_info.path;
            let style = lscolors.style_for_path(&path);
            let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
            trash_info.path = format!("{}", ansi_style.paint(&path));
            trash_info
        })
        .map(|trash_info| trash_info.create_row().map(|row| (trash_info, row)))
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        })
        .filter_map(Result::ok)
        .collect();

    trash_infos.sort_unstable_by(|a, b| a.0.deletion_date.cmp(&b.0.deletion_date));

    let mut table = Table::new();
    for trashinfo in trash_infos {
        table.add_row(trashinfo.1);
    }
    table.printstd();

    Ok(())
}
