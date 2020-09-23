mod subcommand;

use std::path::PathBuf;

use structopt::StructOpt;
use eyre::Result;

use subcommand::{put, SubCommand};

#[derive(Debug, StructOpt)]
pub struct Opt {
    // #[structopt(name = "PATHS")]
    #[structopt(parse(from_os_str))]
    pub paths: Vec<PathBuf>,

    #[structopt(short = "v", long = "verbose")]
    #[structopt(parse(from_occurrences))]
    pub verbosity: u8,

    #[structopt(subcommand)]
    pub subcmd: Option<SubCommand>,

    // compatibility
    /// ignored (for GNU rm compatibility)
    #[structopt(short, long)]
    pub directory: bool,

    /// ignored (for GNU rm compatibility)
    #[structopt(short, long)]
    pub force: bool,

    /// ignored (for GNU rm compatibility)
    #[structopt(short, long)]
    pub interactive: bool,

    /// ignored (for GNU rm compatibility)
    #[structopt(short, long = "R")]
    pub recursive: bool,

    #[structopt(long = "recursive")]
    pub recursive_long: bool,
}

impl Opt {
    pub fn run_or_default(self) -> Result<()> {
        match self.subcmd {
            Some(subcmd) => subcmd.run()?,
            None => SubCommand::Put(put::Opt { paths: self.paths }).run()?,
        }
        Ok(())
    }
}
