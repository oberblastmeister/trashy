use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use crate::args;
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
    #[clap(short, long = "R")]
    pub recursive: bool,
    // #[clap(long = "recursive")]
    // pub recursive_long: bool,
}

impl Args {
    pub fn run(&self, global_args: &args::GlobalArgs) -> Result<()> {
        let paths = &self.paths;
        if paths.is_empty() {
            bail!("No paths were specified to trash");
        }
        Ok(trash::delete_all(paths)?)
    }
}
