use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use crate::app;
// use eyre::{eyre, Result};
// use trash_lib::trash_entry::TrashEntry;

#[derive(Parser, Debug, PartialEq)]
pub struct Args {
    #[clap(parse(from_os_str))]
    pub paths: Vec<PathBuf>,

    // compatibility
    /// ignored (for GNU rm compatibility)
    #[clap(short, long)]
    pub directory: bool,

    /// ignored (for GNU rm compatibility)
    #[clap(short, long)]
    pub force: bool,

    /// ignored (for GNU rm compatibility)
    #[clap(short, long)]
    pub interactive: bool,

    /// ignored (for GNU rm compatibility)
    #[clap(short = 'r', visible_short_alias = 'R', long = "recursive")]
    pub recursive: bool,
}

impl Args {
    pub fn run(&self, global_args: &app::GlobalArgs) -> Result<()> {
        let paths = &self.paths;
        if paths.is_empty() {
            bail!("No paths were specified to trash");
        }
        Ok(trash::delete_all(paths)?)
    }
}
