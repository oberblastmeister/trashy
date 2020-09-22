mod opt;

use std::iter::Inspect;
use std::process;

use ansi_term::Colour::Red;
use env_logger::Builder;
use eyre::{Result, WrapErr};
use log::{debug, error, info, warn, LevelFilter};
use structopt::StructOpt;

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

fn format_err(s: impl std::fmt::Display) -> String {
    format!("{}: {}", Red.paint("Error"), s)
}

fn run() -> Result<()> {
    let opt = Opt::from_args();
    start_logger(opt.verbosity);
    opt.subcmd.run()?;
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => process::exit(0),
        Err(e) => {
            println!("{}", format_err(e));
            process::exit(1);
        }
    }
}
