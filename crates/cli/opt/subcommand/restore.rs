use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

use eyre::{Result, WrapErr};
use log::error;
use structopt::StructOpt;
use trash_lib::ok_log;
use trash_lib::trash_entry::{read_dir_trash_entries, TrashEntry};
use trash_lib::trash_info::TrashInfo;

#[derive(Debug, PartialEq, StructOpt)]
pub struct Opt {
    #[structopt(short = "p", long = "path")]
    #[structopt(parse(from_os_str))]
    path: Option<PathBuf>,

    #[structopt(short = "d", long = "directory")]
    #[structopt(parse(from_os_str))]
    directory: Option<PathBuf>,
}

pub fn restore(opt: Opt) -> Result<()> {
    let path = &opt.path;

    if let Some(p) = path {
        restore_file(&p)?
    } else {
        let cwd = env::current_dir()?;
        restore_in_directory(&cwd)?;
    }
    Ok(())
}

fn restore_file(path: &Path) -> Result<()> {
    trash_lib::restore(path).map_err(Into::into)
}

fn restore_in_directory(dir: &Path) -> Result<()> {
    read_dir_trash_entries()?
        .map(|trash_entry| {
            let trash_info = trash_entry.parse_trash_info();
            (trash_entry, trash_info)
        })
        .inspect(|(_trash_entry, trash_info)| {
            if let Some(e) = trash_info.as_ref().err() {
                error!("{}", e);
            }
        })
        .filter_map(|(trash_entry, trash_info)| {
            trash_info.ok().map(|trash_info| (trash_entry, trash_info))
        })
        .filter(|(_trash_entry, trash_info)| -> bool {
            let decoded_res = trash_info.percent_path().decoded();
            if let Ok(decoded) = decoded_res {
                let decoded_path: &Path = decoded.as_ref().as_ref();
                in_dir(dir, decoded_path)
            } else {
                false
            }
        })
        .map(|(trash_entry, _trash_info)| trash_entry)
        .map(|trash_entry| {
            trash_entry
                .restore()
                .wrap_err("Failed to restore trash_entry")
        })
        .filter_map(|res| ok_log!(res => error!))
        .for_each(|_| ());

    Ok(())
}

pub fn in_dir(dir: &Path, path: &Path) -> bool {
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };
    dir == parent
}
