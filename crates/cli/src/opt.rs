use snafu::Snafu;
use structopt::StructOpt;

mod subcommand;

use subcommand::Subcommand;

#[derive(Debug, Snafu)]
pub enum Error {
    
}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcmd: Subcommand
}
