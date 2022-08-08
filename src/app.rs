mod command;

use anyhow::Result;
use clap::{AppSettings, Parser};
use command::Command;

use command::put;

#[derive(Debug, Parser)]
#[clap(
    version,
    about,
    long_about = None,
    global_setting(AppSettings::DeriveDisplayOrder),
    after_help = "Note: `trash -h` prints a short and concise overview while `trash --help` gives all \
                 details.",
)]
pub struct Args {
    /// The command to run.
    #[clap(subcommand)]
    pub command: Option<Command>,

    #[clap(flatten)]
    config_args: ConfigArgs,

    #[clap(flatten)]
    put_args: put::Args,
}

#[derive(Debug, Parser)]
pub struct ConfigArgs {
    /// When to use colors
    ///
    /// Declare when to use color for the pattern match output:
    ///    'auto':      show colors if the output goes to an interactive console
    ///    'never':     do not use colorized output
    ///    'always':    always use colorized output,
    #[clap(
        short = 'c',
        long = "color",
        value_enum,
        default_value_t = Status::Auto,
        verbatim_doc_comment,
    )]
    pub color_status: Status,

    /// When to format as a table
    ///
    /// Declare when to use color for the pattern match output:
    ///    'auto':      format as a table if the output goes to an interactive console
    ///    'never':     do not format as a table
    ///    'always':    always format as a table
    #[clap(
        short = 't',
        long = "table",
        value_enum,
        default_value_t = Status::Auto,
        verbatim_doc_comment,
    )]
    pub table_status: Status,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Auto,
    Always,
    Never,
}

impl Status {
    pub fn merge(&self, is_atty: bool) -> bool {
        use Status::*;
        match self {
            Auto => is_atty,
            Always => true,
            Never => false,
        }
    }
}

impl Args {
    pub fn run(self) -> Result<()> {
        match self.command {
            None => self.put_args.run(&self.config_args)?,
            Some(command) => command.run(&self.config_args)?,
        }
        Ok(())
    }
}
