use anyhow::Result;

use crate::trash_item::MaybeIndexedTrashItems;
use clap::Parser;
use either::Either::*;

use crate::app;

use super::{
    list,
    utils::{Force, Ranges},
};

#[derive(Debug, Parser)]
// wo wthis
pub struct Args {
    #[clap(flatten)]
    query_args: list::QueryArgs,

    #[clap(flatten)]
    ranges: Ranges,

    #[clap(flatten)]
    force: Force,
}

impl Args {
    pub fn run(&self, config_args: &app::ConfigArgs) -> Result<()> {
        let restore: Box<dyn Fn(_) -> _> = if self.force.force {
            Box::new(restore)
        } else {
            Box::new(|items| {
                super::utils::on_items_with_prompt(items, config_args, "restored", restore)
            })
        };

        if self.ranges.ranges.is_empty() {
            restore(MaybeIndexedTrashItems(Left(self.query_args.list(true)?)))?
        } else {
            restore(MaybeIndexedTrashItems(Right(
                self.query_args.list_ranged(true, self.ranges.parse()?)?,
            )))?;
        }
        Ok(())
    }
}

fn restore(items: MaybeIndexedTrashItems) -> Result<()> {
    trash::os_limited::restore_all(items.items())?;
    Ok(())
}
