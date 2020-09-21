mod opt;

use structopt::StructOpt;

use crate::opt::Opt;

fn main() {
    let opt = Opt::from_args();
    opt.subcmd.run().unwrap();
}
