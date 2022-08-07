use std::io;

use crate::app;
use anyhow::Result;
use clap::{IntoApp, Parser};

#[derive(Debug, Parser)]
pub struct Args {
    /// shell to generate completions for
    #[clap(arg_enum)]
    pub shell: clap_complete::Shell,
}

impl Args {
    pub fn run(&self) -> Result<()> {
        clap_complete::generate(self.shell, &mut app::Args::into_app(), "trash", &mut io::stdout());
        Ok(())
    }
}
