mod command;

use anyhow::Result;
use clap::Parser;
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
    #[clap(
        short = 'c',
        long = "color",
        value_enum,
        default_value_t = Status::Auto,
        global = true
    )]
    pub color_status: Status,
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
