use std::io;

use anyhow::Result;
use clap::{IntoApp, Parser};

#[derive(Debug, Parser)]
pub struct Args {}

impl Args {
    pub fn run(&self) -> Result<()> {
        let man = clap_mangen::Man::new(crate::Args::into_app());
        man.render(&mut io::stdout())?;
        Ok(())
    }
}
