use structopt::StructOpt;
use snafu::Snafu;

use trash_cli::opt::Opt;

#[derive(Debug, Snafu)]
enum Error {

}

fn main() {
    let opt = Opt::from_args();
    opt.subcmd.run().unwrap();
}
