mod command;

use anyhow::Result;
use clap::Parser;
use command::Command;

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = None,
    after_help = "Note: `trashy -h` prints a short and concise overview while `trashy --help` gives all \
                 details.",
)]
pub struct Args {
    /// The command to run.
    #[clap(subcommand)]
    pub command: Command,

    #[clap(flatten)]
    config_args: ConfigArgs,
}

#[derive(Debug, Parser)]
pub struct ConfigArgs {
    /// When to use colors
    ///
    /// Declare when to use color for the pattern match output:
    ///    'auto':      show colors if the output goes to an interactive console
    ///    'never':     do not use colorized output
    ///    'always':    always use colorized output,
    #[arg(
        short = 'c',
        long = "color",
        value_enum,
        default_value_t = Status::Auto,
        verbatim_doc_comment,
    )]
    pub color_status: Status,

    /// When to use time
    ///
    /// Declare when to use time for the pattern match output:
    ///    'precise':      use "%d/%m/%Y %H:%M" format for time output
    ///    'imprecise':    use text representation for time output
    #[arg(
        long = "time",
        value_enum,
        default_value_t = TimeDisplayMode::Imprecise,
        verbatim_doc_comment,
    )]
    pub time_display_mode: TimeDisplayMode,

    /// When to format as a table
    ///
    /// Declare when to use color for the pattern match output:
    ///    'auto':      format as a table if the output goes to an interactive console
    ///    'never':     do not format as a table
    ///    'always':    always format as a table
    #[arg(
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

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDisplayMode {
    Precise,
    Imprecise,
}

impl Args {
    pub fn run(self) -> Result<()> {
        self.command.run(&self.config_args)?;
        Ok(())
    }
}
