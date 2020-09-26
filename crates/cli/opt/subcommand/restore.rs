use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

use eyre::{bail, Result, WrapErr};
use log::error;
use prettytable::Table;
use structopt::StructOpt;
use trash_lib::ok_log;
use trash_lib::trash_entry::{read_dir_trash_entries, TrashEntry};
use trash_lib::trash_info::TrashInfo;

use crate::border::Border;
use crate::utils::{map_to_row, map_trash_entry_keep};

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short = "p", long = "path")]
    #[structopt(parse(from_os_str))]
    path: Option<PathBuf>,

    #[structopt(short = "d", long = "directory")]
    #[structopt(parse(from_os_str))]
    directory: Option<PathBuf>,

    #[structopt(short = "s", long = "style", default_value = "Sharp", possible_values = &Border::variants(), case_insensitive = true)]
    border: Border,
}

pub fn restore(opt: Opt) -> Result<()> {
    match opt {
        Opt {
            path: Some(_),
            directory: Some(_),
            border: _,
        } => bail!("Cannot restore both path and in directory"),
        Opt {
            path: Some(path),
            ..
        } => {
            restore_file(&path)?;
        }
        Opt {
            directory: Some(directory),
            border,
            ..
        } => restore_in_directory(&directory, border)?,
        Opt {
            path: None,
            directory: None,
            border,
        } => restore_in_directory(
            &env::current_dir().wrap_err("Failed to find current working directory")?,
            border
        )?,
    }

    Ok(())
}

fn restore_file(path: &Path) -> Result<()> {
    trash_lib::restore(path).map_err(Into::into)
}

fn restore_in_directory(dir: &Path, border: Border) -> Result<()> {
    let mut table = Table::new();
    table.set_format(border.into());
    // table.set_titles(title_row());

    read_dir_trash_entries()?
        .map(map_trash_entry_keep)
        .filter_map(|res| ok_log!(res => error!))
        .filter(|keep| filter_by_in_dir(keep, dir))
        .map(|(trash_entry, _trash_info)| trash_entry)
        .map(|trash_entry| {
            trash_entry
                .restore()
                .wrap_err("Failed to restore trash_entry")
        })
        .filter_map(|res| ok_log!(res => error!))
            // .map(|trash_entry| {
            //     row_form
            // })
        .for_each(|_| ());

    Ok(())
}

fn in_dir(dir: &Path, path: &Path) -> bool {
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };
    dir == parent
}

fn filter_by_in_dir(keep: &(TrashEntry, TrashInfo), dir: &Path) -> bool {
    let decoded_res = keep.1.percent_path().decoded();
    if let Ok(decoded) = decoded_res {
        let decoded_path: &Path = decoded.as_ref().as_ref();
        in_dir(dir, decoded_path)
    } else {
        false
    }
}
