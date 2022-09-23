mod completions;
mod empty;
mod list;
mod manpage;
pub mod put;
mod restore;
mod utils;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Command {
    /// List files
    List(list::Args),

    /// Put files
    Put(put::PutArgs),

    /// PERMANANTLY removes files
    Empty(empty::Args),

    /// Restore files
    Restore(restore::Args),

    /// Generates completion for a shell
    Completions(completions::Args),

    /// Generates manpages
    Manpage(manpage::Args),
}

impl Command {
    pub fn run(self, config_args: &super::ConfigArgs) -> Result<()> {
        use Command::*;
        match self {
            List(args) => args.run(config_args),
            Put(args) => args.run(config_args),
            Empty(args) => args.run(config_args),
            Restore(args) => args.run(config_args),
            Completions(args) => args.run(),
            Manpage(args) => args.run(),
        }
    }
}
