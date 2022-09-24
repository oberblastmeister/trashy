use std::io;

use anyhow::Result;
use clap::{CommandFactory, Parser};

use crate::app;

#[derive(Debug, Parser)]
pub struct Args {}

impl Args {
    pub fn run(&self) -> Result<()> {
        let man = clap_mangen::Man::new(app::Args::command());
        man.render(&mut io::stdout())?;
        Ok(())
    }
}
