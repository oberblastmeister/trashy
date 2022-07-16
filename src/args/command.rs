// mod completion;
// mod empty;
// mod list;
mod completions;
mod empty;
mod restore;
mod list;
mod manpage;
pub mod put;
// mod restore;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Command {
    /// List files
    List(list::Args),

    /// Put files
    Put(put::Args),

    /// PERMANANTLY removes files
    Empty(empty::Args),

    /// Restore files
    Restore(restore::Args),
    /// Generates completions for shell

    /// Generates completions
    Completions(completions::Args),

    Manpage(manpage::Args),
}

impl Command {
    pub fn run(self, global_args: &super::GlobalArgs) -> Result<()> {
        use Command::*;
        match self {
            // Command::List(opt) => list::list(opt)?,
            List(args) => args.run(global_args),
            Put(args) => args.run(global_args),
            Command::Empty(args) => args.run(),
            Command::Restore(args) => args.run(global_args),
            Completions(args) => args.run(),
            Manpage(args) => args.run(),
        }
    }
}
