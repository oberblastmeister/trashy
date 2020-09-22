use std::path::PathBuf;

use eyre::{WrapErr, Result};
use structopt::StructOpt;
use trash_lib::trash_entry::TrashEntry;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    pub paths: Vec<PathBuf>
}

pub fn trash_put(opt: Opt) -> Result<Vec<TrashEntry>> {
    trash_lib::put(&opt.paths).map_err(Into::into)
}
