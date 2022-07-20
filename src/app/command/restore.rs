use anyhow::{bail, Result};

use clap::Parser;

use crate::{app, utils};

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

    #[clap(long)]
    ranges: Option<String>,
}

impl Args {
    pub fn run(&self, _: &app::ConfigArgs) -> Result<()> {
        use crate::range_syntax;

        let items = self.query_args.list(true)?;
        if let Some(indices) = &self.ranges {
            for range in range_syntax::parse_range_set(indices)? {
                if range.start() as usize > items.len() || range.end() as usize > items.len() {
                    bail!("Range is out of bounds");
                }
                trash::os_limited::restore_all(
                    items[range.to_std()]
                        .into_iter()
                        .map(|item| utils::clone_trash_item(item)),
                )?;
            }
        } else {
            trash::os_limited::restore_all(items)?;
        }
        Ok(())
    }
}
