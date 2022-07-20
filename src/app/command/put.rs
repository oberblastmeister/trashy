use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use crate::app;

#[derive(Parser, Debug, PartialEq)]
pub struct Args {
    /// The paths to put into the trash.
    #[clap(parse(from_os_str))]
    pub paths: Vec<PathBuf>,
}

impl Args {
    pub fn run(&self, _: &app::ConfigArgs) -> Result<()> {
        let paths = &self.paths;
        if paths.is_empty() {
            bail!("No paths were specified to trash");
        }
        Ok(trash::delete_all(paths)?)
    }
}
