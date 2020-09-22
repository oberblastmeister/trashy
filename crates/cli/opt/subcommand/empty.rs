use structopt::StructOpt;
use eyre::Result;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    #[structopt(short = "s", long = "keep-strays")]
    keep_strays: bool,
}

pub fn empty(opt: Opt) -> Result<()> {
    trash_lib::empty(!opt.keep_strays)?;
    Ok(())
}
