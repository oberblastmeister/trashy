use std::env;
use std::path::{Path, PathBuf};

use eyre::{bail, Result, WrapErr};
use log::error;
use log::info;
use structopt::StructOpt;
use trash_lib::ok_log;
use trash_lib::trash_entry::read_dir_trash_entries;

use crate::border::Border;
use crate::print_err;
use crate::table::IndexedTable;
use crate::utils::input_number;
use crate::utils::sort_iterator;
use crate::utils::Pair;

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
            path: Some(path), ..
        } => {
            info!("Restoring path {}", path.display());
            restore_file(&path)?;
        }
        Opt {
            directory: Some(directory),
            border,
            ..
        } => {
            info!("Restoring in directory {}", directory.display());
            restore_in_directory(&directory, border)?
        }
        Opt {
            path: None,
            directory: None,
            border,
        } => {
            info!("Restoring in current working directory");
            restore_in_directory(
                &env::current_dir().wrap_err("Failed to find current working directory")?,
                border,
            )?;
        }
    }

    Ok(())
}

fn restore_file(path: &Path) -> Result<()> {
    trash_lib::restore(path).map_err(Into::into)
}

fn restore_in_directory(dir: &Path, border: Border) -> Result<()> {
    let mut table = IndexedTable::new(border)?;

    let trash_entry_iter = read_dir_trash_entries()?
        .map(Pair::new)
        .filter_map(|res| ok_log!(res => error!));

    let trash_entries: Vec<_> = sort_iterator(trash_entry_iter)
        .map(|pair| table.add_row(&pair).map(|_| (pair)))
        .filter_map(|res| ok_log!(res => error!))
        .filter(|pair| filter_by_in_dir(pair, dir))
        .map(|pair| pair.revert())
        .collect();
    table.print();

    let inp = loop {
        match input_number("Input the index or range or trash entries to restore: ") {
            Ok(inp) => break inp,
            Err(e) => print_err(e),
        }
    };

    info!("Restoring {:?}", trash_entries[inp as usize]);
    trash_entries[inp as usize].restore()?;

    Ok(())
}

fn in_dir(dir: &Path, path: &Path) -> bool {
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };
    dir == parent
}

fn filter_by_in_dir(pair: &Pair, dir: &Path) -> bool {
    let decoded_res = pair.1.percent_path().decoded();
    if let Ok(decoded) = decoded_res {
        let decoded_path: &Path = decoded.as_ref().as_ref();
        in_dir(dir, decoded_path)
    } else {
        false
    }
}
