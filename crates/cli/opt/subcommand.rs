mod list;
pub mod put;
mod empty;
mod restore;
mod remove;
mod completion;

use structopt::StructOpt;
use eyre::Result;

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// list valid files in the trash
    List(list::Opt),

    /// Put files into trash. Is run by default if no subcommand is specified.
    Put(put::Opt),

    /// PERMANANTLY removes ALL files in the trash
    Empty(empty::Opt),

    /// Restore files from the trash
    Restore(restore::Opt),

    /// PERMANANTLY removes files from the trash
    Remove(remove::Opt),

    /// Generates completions for shell
    Completion(completion::Opt)
}

impl SubCommand {
    pub fn run(self) -> Result<()> {
        match self {
            SubCommand::List(opt) => list::list(opt)?,
            SubCommand::Put(opt) => {
                put::put(opt)?;
            },
            SubCommand::Empty(opt) => empty::empty(opt)?,
            SubCommand::Restore(opt) => restore::restore(opt)?,
            SubCommand::Remove(opt) => remove::remove(opt)?,
            SubCommand::Completion(opt) => completion::completion(opt),
        }
        Ok(())
    }
}
