mod opt;
mod utils;
mod border;
mod table;

use std::process;
use std::fmt;

use env_logger::Builder;
use ansi_term::Color::Red;
use eyre::Result;
use log::{debug, LevelFilter};
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

fn try_main() -> Result<()> {
    let opt = Opt::from_args();
    start_logger(opt.verbosity);
    debug!("Opt: {:?}", opt);
    opt.run_or_default()?;
    Ok(())
}

fn main() {
    match try_main() {
        Ok(()) => process::exit(0),
        Err(e) => {
            print_err(e);
            process::exit(1);
        }
    }
}

fn print_err(s: impl fmt::Debug) {
    eprintln!("{}: {:?}", Red.bold().paint("Error"), s);
}
