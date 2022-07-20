use anyhow::{bail, Result};
use clap::Parser;
use dialoguer::Confirm;
use trash::TrashItem;

use crate::{app, utils};

use super::list;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: list::QueryArgs,

    /// Empty all files
    #[clap(long, conflicts_with_all = &["before", "within", "patterns"])]
    all: bool,

    /// Skip confirmation
    /// 
    /// By default, 'trashy' will ask for confirmation before permanently removing files.
    /// You can opt out of this by adding '--force'.
    /// This can be useful in scripts.
    #[clap(long)]
    force: bool,

    #[clap(long)]
    ranges: Option<String>,
}

impl Args {
    pub fn run(&self, _: &app::ConfigArgs) -> Result<()> {
        use crate::range_syntax;

        let empty = if self.force { empty } else { empty_with_prompt };

        let items = self.query_args.list(true)?;
        if let Some(ranges) = &self.ranges {
            for range in range_syntax::parse_range_set(ranges)? {
                if range.start() as usize > items.len() || range.end() as usize > items.len() {
                    bail!("Range is out of bounds");
                }
                empty(
                    items[range.to_std()]
                        .into_iter()
                        .map(|item| utils::clone_trash_item(item))
                        .collect(),
                )?;
            }
        } else {
            empty(items)?;
        }
        Ok(())
    }
}

fn empty_with_prompt(items: Vec<TrashItem>) -> Result<()> {
    println!("{} items will be emptied from the trash", items.len());
    if Confirm::new().with_prompt("Are you sure?").interact()? {
        empty(items)?;
    }
    Ok(())
}

fn empty(items: impl IntoIterator<Item = TrashItem>) -> Result<()> {
    trash::os_limited::purge_all(items)?;
    Ok(())
}
