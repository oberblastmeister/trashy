use std::io;

use anyhow::Result;
use clap::{IntoApp, Parser};

use crate::app;

#[derive(Debug, Parser)]
pub struct Args {}

impl Args {
    pub fn run(&self) -> Result<()> {
        let man = clap_mangen::Man::new(app::Args::into_app());
        man.render(&mut io::stdout())?;
        Ok(())
    }
}
