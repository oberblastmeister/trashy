use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{restore_index::RestoreIndexMultiple, table};
use clap::Clap;
use eyre::{bail, eyre, Result, WrapErr};
use log::{debug, error, info, trace};
use trash_lib::trash_entry::read_dir_trash_entries;
use trash_lib::{ok_log, trash_entry::TrashEntry};

use crate::exitcode::ExitCode;
use crate::restore_index::RestoreIndex;
#[cfg(feature = "readline")]
use crate::rustyline::ReadLine;
use crate::table::IndexedTable;
use crate::utils::{sort_iterator, Pair};

#[derive(Debug, Clap)]
// wo wthis
pub struct Opt {
    /// The optional path to restore
    #[clap(
        parse(from_os_str),
        short = 'p',
        long = "path",
        conflicts_with_all = &["directory", "interactive", "last"]
    )]
    path: Option<PathBuf>,

    /// Optionally restore inside of a directory
    #[clap(short = 'd', long = "directory", conflicts_with_all = &["interactive", "last"])]
    directory: Option<PathBuf>,

    #[clap(flatten)]
    table_opt: table::Opt,

    /// Restore the last n trashed files. Uses the same indexes as interactive mode.
    #[clap(short, long)]
    last: Option<RestoreIndexMultiple>,

    /// Go into interactive mode to restore files. The default when running with no flags.
    #[clap(short, long)]
    interactive: bool,
}

pub fn restore(opt: Opt) -> Result<()> {
    match opt {
        Opt {
            path: Some(_),
            directory: Some(_),
            ..
        } => unreachable!(),
        Opt {
            path: None,
            directory: None,
            interactive: false,
            last: Some(indices),
            ..
        } => {
            restore_from_indexes(
                sort_iterator(get_trash_entries_in_dir(&env::current_dir()?)?)
                    .map(Pair::revert)
                    .collect(),
                indices,
            )?;
        }
        Opt {
            path: Some(path), ..
        } => {
            info!("Restoring path {}", path.display());
            restore_file(&path)?;
        }
        Opt {
            directory: Some(directory),
            table_opt,
            ..
        } => {
            if !directory.is_dir() {
                bail!("The path `{}` is not a directory", directory.display());
            }
            let directory = fs::canonicalize(&directory).wrap_err(format!(
                "Failed to canonicalize directory `{}`",
                directory.display()
            ))?;
            info!("Restoring in directory {}", directory.display());
            restore_in_directory(&directory, table_opt)?
        }
        Opt {
            path: None,
            directory: None,
            table_opt,
            ..
        } => {
            info!("Restoring in current working directory");
            let cwd = env::current_dir().wrap_err("Failed to find current working directory")?;

            info!("Cwd is `{}`", cwd.display());
            restore_in_directory(&cwd, table_opt)?;
        }
    }

    Ok(())
}

fn restore_file(path: &Path) -> Result<()> {
    trash_lib::restore(path).map_err(Into::into)
}

/// gets all the trash entries in a directory
fn get_trash_entries_in_dir(dir: &Path) -> Result<impl Iterator<Item = Pair> + '_> {
    let iter = read_dir_trash_entries()?
        .map(Pair::new)
        .filter_map(|res| ok_log!(res => error!))
        .filter(move |pair| filter_by_in_dir(pair, dir));
    Ok(iter)
}

/// Restore thing in a directory. Must take absolute dir path instead of relative path to avoid
/// issues. Path must be a directory
fn restore_in_directory(dir: &Path, table_opt: table::Opt) -> Result<()> {
    let mut table = IndexedTable::new(table_opt)?;

    let trash_entries: Vec<_> = sort_iterator(get_trash_entries_in_dir(dir)?)
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

    println!("Input the index or range of trash entries to restore:");

    #[cfg(feature = "readline")]
    let indices: RestoreIndexMultiple = ReadLine::new().read_parse_loop(">> ")?;

    info!("Indices are {:?}", indices);

    restore_from_indexes(trash_entries, indices)?;

    Ok(())
}

fn restore_from_indexes<U>(trash_entries: Vec<TrashEntry>, indices: U) -> Result<()>
where
    U: IntoIterator<Item = RestoreIndex>,
{
    for idx in indices {
        match idx {
            RestoreIndex::Point(p) => trash_entries
                .get(p)
                .ok_or_else(|| eyre!("{} is not a valid index that is in bounds", p))?
                .restore()?,
            RestoreIndex::Range(range) => {
                let slice = trash_entries.get(range.clone()).ok_or_else(|| {
                    eyre!(
                        "{}-{} is not a valid range that is in bounds",
                        &range.start,
                        &range.end
                    )
                })?;
                slice
                    .iter()
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
