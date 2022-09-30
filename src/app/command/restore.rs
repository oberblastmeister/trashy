use anyhow::Result;

use crate::{range_syntax, trash_item::MaybeIndexedTrashItems};
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

        let filters = self.query_args.filter_args.to_filters()?;
        if filters.is_empty() && self.ranges.ranges.is_empty() {
            let items = list::list(self.query_args.rev, self.query_args.max, filters)?;
            list::display_items(&items, config_args)?;
            let ranges = dialoguer::Input::<String>::new().with_prompt("restore ranges").interact_text()?;
            restore(MaybeIndexedTrashItems(Right(list::filter_by_ranges(
                &items,
                range_syntax::parse_range_set(&ranges)?,
            )?)))?
        } else if self.ranges.ranges.is_empty() {
            restore(MaybeIndexedTrashItems(Left(list::list(
                self.query_args.rev,
                self.query_args.max,
                filters,
            )?)))?
        } else {
            restore(MaybeIndexedTrashItems(Right(list::list_ranged(
                self.query_args.rev,
                self.query_args.max,
                filters,
                self.ranges.parse()?,
            )?)))?;
        }
        Ok(())
    }
}

fn restore(items: MaybeIndexedTrashItems) -> Result<()> {
    trash::os_limited::restore_all(items.items())?;
    Ok(())
}
