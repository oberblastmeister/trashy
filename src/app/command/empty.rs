use anyhow::Result;
use clap::Parser;
use dialoguer::Confirm;
use trash::TrashItem;

use super::list;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    list_args: list::QueryArgs,

    /// Empty all files
    #[clap(long, conflicts_with_all = &["before", "within", "patterns"])]
    all: bool,

    /// Skip confirmation
    #[clap(long)]
    force: bool,
}

impl Args {
    #[cfg(target_os = "macos")]
    pub fn run(&self, global_args: &args::GlobalArgs) -> Result<()> {
        bail!("Emptying is not supported on MacOS");
    }

    #[cfg(not(target_os = "macos"))]
    pub fn run(&self) -> Result<()> {
        let empty = if self.force { empty } else { empty_with_prompt };
        if self.all {
            empty(trash::os_limited::list()?)?;
        } else {
            empty(self.list_args.list(true)?)?;
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
