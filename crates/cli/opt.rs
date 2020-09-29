mod subcommand;

use eyre::Result;
use structopt::clap::AppSettings;
use structopt::StructOpt;

use subcommand::SubCommand;

#[derive(Debug, StructOpt)]
#[structopt(
    global_settings(&[AppSettings::ColoredHelp]),
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub struct Opt {
    #[structopt(flatten)]
    put_opt: subcommand::put::Opt,

    /// How verbose to log. The verbosity is error by default.
    #[structopt(short = "v", long = "verbose")]
    #[structopt(parse(from_occurrences))]
    pub verbosity: u8,

    /// The subcommand to run. If none is specified, will run `trash put` by default
    #[structopt(subcommand)]
    pub subcmd: Option<SubCommand>,
}

impl Opt {
    pub fn run_or_default(self) -> Result<()> {
        match self.subcmd {
            Some(subcmd) => subcmd.run()?,
            None => SubCommand::Put(self.put_opt).run()?,
        }
        Ok(())
    }
}
