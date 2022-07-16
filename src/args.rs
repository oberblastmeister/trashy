mod command;

use std::path::{Path, PathBuf};

// use clap::{crate_authors, crate_description, crate_version, AppSettings, Clap};
// use eyre::Result;
use anyhow::{anyhow, bail, Ok, Result};
use clap::{
    builder::{ValueParser, ValueParserFactory},
    Parser,
};
use command::Command;

#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// The command to run.
    #[clap(subcommand)]
    pub command: Command,

    #[clap(flatten)]
    global_args: GlobalArgs,
}

#[derive(Debug, Parser)]
pub struct GlobalArgs {
    #[clap(short = 'c', long = "color", value_enum, default_value_t = Status::Auto, global = true)]
    pub color_status: Status,
}

pub fn ensure_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("The path does not exist")
    }
    Ok(())
}

pub fn ensure_is_dir(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("The path is not a directory")
    }
    Ok(())
    
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Auto,
    Always,
    Never,
}

impl Status {
    pub fn color(&self) -> bool {
        use Status::*;
        match self {
            Auto => atty::is(atty::Stream::Stdout),
            Always => true,
            Never => false,
        }
    }
}

impl Args {
    pub fn run(self) -> Result<()> {
        self.command.run(&self.global_args)?;
        Ok(())
    }
}
