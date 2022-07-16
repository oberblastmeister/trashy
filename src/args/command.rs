// mod completion;
// mod empty;
// mod list;
mod completions;
mod empty;
mod list;
mod manpage;
pub mod put;
// mod restore;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Command {
    /// List files in the trash
    List(list::Args),

    /// Put files into the trash
    Put(put::Args),
    // /// PERMANANTLY removes files in the trash
    // Empty(empty::Opt),

    // /// Restore files from the trash
    // Restore(restore::Opt),
    /// Generates completions for shell
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
            // Command::Empty(opt) => empty::empty(opt)?,
            // Command::Restore(opt) => restore::restore(opt)?,
            Completions(args) => args.run(),
            Manpage(args) => args.run(),
        }
    }
}
