use std::path::PathBuf;

use eyre::{WrapErr, Result, eyre};
use structopt::StructOpt;
use trash_lib::trash_entry::TrashEntry;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub paths: Vec<PathBuf>
}

pub fn put(opt: Opt) -> Result<Vec<TrashEntry>> {
    let paths = &opt.paths;
    if paths.is_empty() {
        return Err(eyre!("No paths were specified to trash!"));
    }
    trash_lib::put(paths).map_err(Into::into)
}
