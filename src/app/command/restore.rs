use anyhow::Result;
use trash::TrashItem;

// use crate::{restore_index::RestoreIndexMultiple, table};
use clap::Parser;

use crate::app;

use super::list;
// use eyre::{bail, eyre, Result, WrapErr};
// use log::{debug, error, info, trace};
// use trash_lib::trash_entry::read_dir_trash_entries;
// use trash_lib::{ok_log, trash_entry::TrashEntry};

// use crate::exitcode::ExitCode;
// use crate::restore_index::RestoreIndex;
// #[cfg(feature = "readline")]
// use crate::rustyline::ReadLine;
// use crate::table::IndexedTable;
// use crate::utils::{sort_iterator, Pair};

#[derive(Debug, Parser)]
// wo wthis
pub struct Args {
    // /// The optional path to restore
    // #[clap(
    //     parse(from_os_str),
    //     short = 'p',
    //     long = "path",
    //     conflicts_with_all = &["directory", "interactive", "last"]
    // )]
    // path: Option<PathBuf>,

    // /// Optionally restore inside of a directory
    // #[clap(short = 'd', long = "directory", conflicts_with_all = &["interactive", "last"])]
    // directory: Option<PathBuf>,

    // // #[clap(flatten)]
    // // table_opt: table::Opt,

    // // /// Restore the last n trashed files. Uses the same indexes as interactive mode.
    // // #[clap(short, long)]
    // // last: Option<RestoreIndexMultiple>,

    // /// Go into interactive mode to restore files. The default when running with no flags.
    // #[clap(short, long)]
    // interactive: bool,
    #[clap(flatten)]
    query_args: list::QueryArgs,

    #[clap(long = "ix")]
    indices: Option<String>,
}

impl Args {
    #[cfg(target_os = "macos")]
    pub fn run(&self, _: &app::GlobalArgs) -> Result<()> {
        bail!("Restoring is not supported on MacOS");
    }

    #[cfg(not(target_os = "macos"))]
    pub fn run(&self, _: &app::GlobalArgs) -> Result<()> {
        use anyhow::bail;

        use crate::range_syntax;

        let items = self.query_args.list(true)?;
        if let Some(indices) = &self.indices {
            for range in range_syntax::parse_range_set(indices)? {
                if range.start() as usize > items.len() || range.end() as usize > items.len() {
                    bail!("Range is out of bounds");
                }
                trash::os_limited::restore_all(
                    items[range.to_std()]
                        .into_iter()
                        .map(|item| clone_trash_item(item)),
                )?;
            }
        } else {
            trash::os_limited::restore_all(items)?;
        }
        Ok(())
    }
}

fn clone_trash_item(item: &TrashItem) -> TrashItem {
    TrashItem {
        id: item.id.clone(),
        name: item.name.clone(),
        original_parent: item.original_parent.clone(),
        time_deleted: item.time_deleted.clone(),
    }
}
