mod list;

use snafu::Snafu;
use structopt::StructOpt;

use list::{trash_list, ListOpts};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    #[snafu(display("An error occured while running the subcommand list: {}", source))]
    List {
        source: list::Error,
    }
}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    List(ListOpts)
}

impl Subcommand {
    pub fn run(self) -> Result<()> {
        match self {
            Subcommand::List(_list_opts) => trash_list()?,
        }

        Ok(())
    }
}
