mod border;
mod exitcode;
mod opt;
mod restore_index;
mod table;
mod utils;
mod rustyline;

use std::fmt;

use ansi_term::Color::Red;
use clap::Clap;
use env_logger::Builder;
use eyre::Result;
use log::{debug, LevelFilter};

use exitcode::ExitCode;
use opt::Opt;

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
    let opt = Opt::parse();
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
