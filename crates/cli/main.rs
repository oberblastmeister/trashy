mod border;
mod opt;
mod table;
mod restore_index;
mod utils;
mod exitcode;

use std::fmt;

use ansi_term::Color::Red;
use env_logger::Builder;
use log::{debug, LevelFilter};
use structopt::StructOpt;
use eyre::Result;

use opt::Opt;
use exitcode::ExitCode;

/// Start the logger depending on the verbosity flag
fn start_logger(verbosity: u8) {
    Builder::from_default_env()
        .filter_level(convert_to_level_filter(verbosity))
        .init();
}

fn convert_to_level_filter(n: u8) -> LevelFilter {
    match n {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    }
}

fn try_main() -> Result<()> {
    let opt = Opt::from_args();
    start_logger(opt.verbosity);
    debug!("Opt: {:?}", opt);
    opt.run_or_default()?;
    Ok(())
}

fn main() {
    match try_main() {
        Ok(()) => ExitCode::Success.exit(),
        Err(e) => ExitCode::GeneralError.exit_with_msg(format!("{:?}", e)),
    }
}

fn print_err(s: impl fmt::Debug) {
    eprintln!("{}: {:?}", Red.bold().paint("Error"), s);
}

fn print_err_display(s: impl fmt::Display) {
    eprintln!("{}: {}", Red.bold().paint("Error"), s);
}
