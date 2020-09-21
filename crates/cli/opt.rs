mod subcommand;

use structopt::StructOpt;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}
