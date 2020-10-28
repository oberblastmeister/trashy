use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::input;
use clap::{ArgEnum, Clap};
use eyre::{bail, Result, WrapErr};
use log::debug;
use log::error;
use log::info;
use log::trace;
use trash_lib::ok_log;
use trash_lib::trash_entry::read_dir_trash_entries;

use crate::border::Border;
use crate::exitcode::ExitCode;
use crate::print_err_display;
use crate::restore_index::RestoreIndex;
use crate::table::IndexedTable;
use crate::utils::{sort_iterator, Pair};

#[derive(Debug, Clap)]
pub struct Opt {
    #[clap(parse(from_os_str), short = 'p', long = "path")]
    path: Option<PathBuf>,

    #[clap(parse(from_os_str), short = 'd', long = "directory")]
    directory: Option<PathBuf>,

    #[clap(
        arg_enum,
        short = 's',
        long = "style",
        default_value = "Sharp",
        case_insensitive = true
    )]
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
            directory: Some(ref directory),
            border,
            ..
        } => {
            if !directory.is_dir() {
                bail!("The path `{}` is not a directory", directory.display());
            }
            let directory = fs::canonicalize(directory).wrap_err(format!(
                "Failed to canonicalize directory `{}`",
                directory.display()
            ))?;
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

/// Restore thing in a directory. Must take absolute dir path instead of relative path to avoid
/// issues. Path must be a directory
fn restore_in_directory(dir: &Path, border: Border) -> Result<()> {
    let mut table = IndexedTable::new(border)?;

    let trash_entry_iter = read_dir_trash_entries()?
        .map(Pair::new)
        .filter_map(|res| ok_log!(res => error!));

    let trash_entries: Vec<_> = sort_iterator(trash_entry_iter)
        .filter(|pair| filter_by_in_dir(pair, &dir))
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
        match input("Input the index or range of trash entries to restore: ") {
            Ok(inp) => match RestoreIndex::get_multiple_non_overlapping(&inp) {
                Ok(indices) => break indices,
                Err(e) => print_err_display(e),
            },
            Err(e) => print_err_display(e),
        }
    };

    info!("Indices are {:?}", indices);

    for idx in indices {
        match idx {
            RestoreIndex::Point(p) => trash_entries[p].restore()?,
            RestoreIndex::Range(range) => {
                trash_entries[range]
                    .into_iter()
                    .map(|trash_entry| trash_entry.restore())
                    .filter_map(|res| ok_log!(res => error!))
                    .for_each(|_| ());
            }
        }
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
