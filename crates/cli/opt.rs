mod subcommand;

use structopt::StructOpt;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(parse(from_occurrences))]
    pub verbosity: u8,

    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}
