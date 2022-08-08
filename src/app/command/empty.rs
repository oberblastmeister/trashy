use anyhow::Result;
use clap::Parser;
use either::Either::*;

use crate::{app, trash_item::MaybeIndexedTrashItems};

use super::{
    list,
    utils::{Force, Ranges},
};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: list::QueryArgs,

    /// Empty all files
    #[clap(long, conflicts_with_all = &list::QueryArgs::CONFLICTS)]
    all: bool,

    #[clap(flatten)]
    ranges: Ranges,

    #[clap(flatten)]
    force: Force,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        let empty: Box<dyn Fn(_) -> _> = if self.force.force {
            Box::new(empty)
        } else {
            Box::new(|items| {
                super::utils::on_items_with_prompt(items, config_args, "emptied", empty)
            })
        };

        if self.ranges.ranges.is_empty() {
            empty(MaybeIndexedTrashItems(Left(self.query_args.list(true)?)))?
        } else {
            empty(MaybeIndexedTrashItems(Right(
                self.query_args.list_ranged(true, self.ranges.parse()?)?,
            )))?;
        }
        Ok(())
    }
}

fn empty(items: MaybeIndexedTrashItems) -> Result<()> {
    trash::os_limited::purge_all(items.items())?;
    Ok(())
}
