use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

use crate::app;

#[derive(Parser, Debug, PartialEq)]
pub struct PutArgs {
    /// The paths to put into the trash.
    pub paths: Vec<PathBuf>,
}

impl PutArgs {
    pub fn run(&self, _: &app::ConfigArgs) -> Result<()> {
        let paths = &self.paths;
        if paths.is_empty() {
            bail!("No paths were specified to trash");
        }
        Ok(trash::delete_all(paths)?)
    }
}
