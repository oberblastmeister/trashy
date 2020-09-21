mod opt;

use structopt::StructOpt;
use eyre::{WrapErr, Result};

use opt::Opt;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    opt.subcmd.run()?;
    Ok(())
}
