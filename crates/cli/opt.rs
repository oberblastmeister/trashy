mod subcommand;

use structopt::StructOpt;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short = "v", long = "verbose")]
    #[structopt(parse(from_occurrences))]
    pub verbosity: u8,

    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}
