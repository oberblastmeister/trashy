mod app;
mod exitcode;
mod filter;
mod print;
mod range;
mod range_set;
mod range_syntax;
mod utils;

use anyhow::Result;
use app::Args;
use clap::Parser;
use exitcode::ExitCode;

fn main() {
    match try_main() {
        Ok(()) => ExitCode::Success.exit(),
        Err(e) => ExitCode::Error.exit_with_msg(format!("{e:#}")),
    }
}

fn try_main() -> Result<()> {
    Args::parse().run()
}
