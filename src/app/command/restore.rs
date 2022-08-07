use anyhow::Result;

use crate::trash_item::MaybeIndexedTrashItems;
use clap::Parser;
use either::Either::*;

use crate::app;

use super::list;

#[derive(Debug, Parser)]
// wo wthis
pub struct Args {
    // /// Optionally restore inside of a directory
    // #[clap(short = 'd', long = "directory", conflicts_with_all = &["interactive", "last"])]
    // directory: Option<PathBuf>,

    // /// Go into interactive mode to restore files. The default when running with no flags.
    // #[clap(short, long)]
    // interactive: bool,
    #[clap(flatten)]
    query_args: list::QueryArgs,

    #[clap(short, long, conflicts_with = "rev")]
    ranges: Option<String>,

    #[clap(short, long)]
    force: bool,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        let restore: Box<dyn Fn(_) -> _> = if self.force {
            Box::new(restore)
        } else {
            Box::new(|items| restore_with_prompt(items, config_args))
        };

        if let Some(ranges) = &self.ranges {
            restore(MaybeIndexedTrashItems(Right(self.query_args.list_ranged(true, ranges)?)))?;
        } else {
            restore(MaybeIndexedTrashItems(Left(self.query_args.list(true)?)))?
        }
        Ok(())
    }
}

fn restore_with_prompt(items: MaybeIndexedTrashItems, config_args: &app::ConfigArgs) -> Result<()> {
    use dialoguer::Confirm;

    println!("{} items will be restored", items.len());
    list::display_indexed_items(items.indexed_items(), config_args)?;
    if Confirm::new().with_prompt("Are you sure?").interact()? {
        restore(items)?;
    }
    Ok(())
}

fn restore(items: MaybeIndexedTrashItems) -> Result<()> {
    trash::os_limited::restore_all(items.items())?;
    Ok(())
}
