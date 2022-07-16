mod args;
mod exitcode;
mod utils;

use anyhow::Result;
use args::Args;
use clap::Parser;
use exitcode::ExitCode;
// mod border;
// mod exitcode;

// #[cfg(not(feature = "readline"))]
// mod input;
// mod opt;
mod print;
// mod restore_index;
// #[cfg(feature = "readline")]
// mod rustyline;
// mod table;
// mod utils;

// use clap::Clap;
// use env_logger::Builder;
// use eyre::Result;
// use log::{debug, LevelFilter};

// pub use exitcode::ExitCode;
// use opt::Opt;

// /// Start the logger depending on the verbosity flag
// fn start_logger(verbosity: u8) {
//     Builder::from_default_env()
//         .filter_level(convert_to_level_filter(verbosity))
//         .init();
// }

// fn convert_to_level_filter(n: u8) -> LevelFilter {
//     match n {
//         0 => LevelFilter::Error,
//         1 => LevelFilter::Warn,
//         2 => LevelFilter::Info,
//         3 => LevelFilter::Debug,
//         4 => LevelFilter::Trace,
//         _ => LevelFilter::Off,
//     }
// }

fn main() {
    match try_main() {
        Ok(()) => ExitCode::Success.exit(),
        Err(e) => ExitCode::Error.exit_with_msg(format!("{e:#}")),
    }
}

fn try_main() -> Result<()> {
    Args::parse().run()
}
