mod subcommand;

use structopt::StructOpt;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    subcmd: SubCommand,
}
