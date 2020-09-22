mod list;
mod put;

use structopt::StructOpt;
use eyre::{WrapErr, Result};

use list::trash_list;
use put::trash_put;

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    List(list::Opt),
    Put(put::Opt),
}

impl SubCommand {
    pub fn run(self) -> Result<()> {
        match self {
            SubCommand::List(opt) => trash_list(opt)?,
            SubCommand::Put(opt) => {
                let _ = trash_put(opt)?;
            },
        }
        Ok(())
    }
}
