use structopt::StructOpt;

mod subcommand;

use subcommand::Subcommand;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcmd: Subcommand
}
