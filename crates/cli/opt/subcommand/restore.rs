use std::env;
use std::path::{Path, PathBuf};

use eyre::{bail, Result, WrapErr};
use log::debug;
use log::error;
use log::info;
use log::trace;
use clap::{ArgEnum, Clap};
use trash_lib::ok_log;
use trash_lib::trash_entry::read_dir_trash_entries;

use crate::border::Border;
use crate::exitcode::ExitCode;
use crate::print_err_display;
use crate::table::IndexedTable;
use crate::utils::{Pair, sort_iterator};
use crate::restore_index::input_restore_indices;

#[derive(Debug, Clap)]
pub struct Opt {
    #[clap(short = 'p', long = "path")]
    #[clap(parse(from_os_str))]
    path: Option<PathBuf>,

    #[clap(short = 'd', long = "directory")]
    #[clap(parse(from_os_str))]
    directory: Option<PathBuf>,

    #[clap(arg_enum)]
    #[clap(short = 's', long = "style", default_value = "Sharp", case_insensitive = true)]
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
            let cwd = env::current_dir().wrap_err("Failed to find current working directory")?;
            info!("Cwd is `{}`", cwd.display());
            restore_in_directory(&cwd, border)?;
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
        .filter(|pair| filter_by_in_dir(pair, dir))
        .map(|pair| table.add_row(&pair).map(|_| (pair)))
        .filter_map(|res| ok_log!(res => error!))
        .map(|pair| pair.revert())
        .collect();

    if trash_entries.is_empty() {
        ExitCode::Success.exit_with_msg(format!(
            "No entries to restore in directory `{}`",
            dir.display()
        ))
    }

    table.print();
    trace!("The final vector of trash entries is {:?}", trash_entries);

    let indices = loop {
        match input_restore_indices("Input the index or range or trash entries to restore: ") {
            Ok(inp) => break inp,
            Err(e) => print_err_display(e),
        }
    };

    info!("Indices were {:?}", indices);
    for idx in indices {
        trash_entries[idx].into_iter()
            .map(|entry| {
                info!("Restoring {:?}", entry);
                entry.restore()
            })
            .filter_map(|res| ok_log!(res => error!))
            .for_each(|_| ());
    }

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
    trace!(
        "The original path of the trash entry file: {:?}",
        decoded_res
    );
    if let Ok(decoded) = decoded_res {
        let decoded_path: &Path = decoded.as_ref().as_ref();
        let in_dir = in_dir(dir, decoded_path);
        debug!(
            "path {} in the dir {}: {}",
            decoded_path.display(),
            dir.display(),
            in_dir
        );
        in_dir
    } else {
        false
    }
}
