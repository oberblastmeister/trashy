mod list;

use structopt::StructOpt;
use eyre::{WrapErr, Result};

use list::{trash_list, ListOpts};

#[derive(StructOpt, Debug, PartialEq)]
pub enum Subcommand {
    List(ListOpts)
}

impl Subcommand {
    pub fn run(self) -> Result<()> {
        match self {
            Subcommand::List(list_opts) => trash_list(list_opts)?,
        }

        Ok(())
    }
}
