mod list;

use structopt::StructOpt;
use eyre::{WrapErr, Result};

use list::{trash_list, ListOpt};

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    List(ListOpt),
}

impl SubCommand {
    pub fn run(self) -> Result<()> {
        match self {
            SubCommand::List(list_opt) => trash_list(list_opt)?,
        }
        Ok(())
    }
}
