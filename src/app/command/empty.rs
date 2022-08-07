use anyhow::Result;
use clap::Parser;
use either::Either::*;

use crate::{app, trash_item::MaybeIndexedTrashItems};

use super::list;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: list::QueryArgs,

    /// Empty all files
    #[clap(long, conflicts_with_all = &["before", "within", "patterns"])]
    all: bool,

    #[clap(short, long, conflicts_with = "rev")]
    ranges: Option<String>,

    /// Skip confirmation
    ///
    /// By default, 'trashy' will ask for confirmation before permanently removing files.
    /// You can opt out of this by adding '--force'.
    /// This can be useful in scripts.
    #[clap(short, long)]
    force: bool,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        let empty: Box<dyn Fn(_) -> _> = if self.force {
            Box::new(empty)
        } else {
            Box::new(|items| empty_with_prompt(items, config_args))
        };

        if let Some(ranges) = &self.ranges {
            empty(MaybeIndexedTrashItems(Right(self.query_args.list_ranged(true, ranges)?)))?;
        } else {
            empty(MaybeIndexedTrashItems(Left(self.query_args.list(true)?)))?
        }
        Ok(())
    }
}

fn empty_with_prompt(items: MaybeIndexedTrashItems, config_args: &app::ConfigArgs) -> Result<()> {
    use dialoguer::Confirm;

    println!("{} items will be emptied", items.len());
    list::display_indexed_items(items.indexed_items(), config_args)?;
    if Confirm::new().with_prompt("Are you sure?").interact()? {
        empty(items)?;
    }
    Ok(())
}

fn empty(items: MaybeIndexedTrashItems) -> Result<()> {
    trash::os_limited::purge_all(items.items())?;
    Ok(())
}
