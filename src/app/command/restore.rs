use anyhow::Result;

use crate::trash_item::MaybeIndexedTrashItems;
use clap::Parser;
use either::Either::*;

use crate::app;

use super::list;

#[derive(Debug, Parser)]
// wo wthis
pub struct Args {
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
            Box::new(|items| {
                super::utils::on_items_with_prompt(items, config_args, "restored", restore)
            })
        };

        if let Some(ranges) = &self.ranges {
            restore(MaybeIndexedTrashItems(Right(self.query_args.list_ranged(true, ranges)?)))?;
        } else {
            restore(MaybeIndexedTrashItems(Left(self.query_args.list(true)?)))?
        }
        Ok(())
    }
}

fn restore(items: MaybeIndexedTrashItems) -> Result<()> {
    trash::os_limited::restore_all(items.items())?;
    Ok(())
}
