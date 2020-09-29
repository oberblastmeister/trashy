use std::path::PathBuf;

use eyre::{eyre, Result};
use structopt::StructOpt;
use trash_lib::trash_entry::TrashEntry;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    #[structopt(parse(from_os_str), conflicts_with("subcmd"))]
    pub paths: Vec<PathBuf>,

    // compatibility
    /// ignored (for GNU rm compatibility)
    #[structopt(short, long)]
    pub directory: bool,

    /// ignored (for GNU rm compatibility)
    #[structopt(short, long)]
    pub force: bool,

    /// ignored (for GNU rm compatibility)
    #[structopt(short, long)]
    pub interactive: bool,

    /// ignored (for GNU rm compatibility)
    #[structopt(short, long = "R")]
    pub recursive: bool,

    #[structopt(long = "recursive")]
    pub recursive_long: bool,
}

pub fn put(opt: Opt) -> Result<Vec<TrashEntry>> {
    let paths = &opt.paths;
    if paths.is_empty() {
        return Err(eyre!("No paths were specified to trash!"));
    }
    trash_lib::put(paths).map_err(Into::into)
}
