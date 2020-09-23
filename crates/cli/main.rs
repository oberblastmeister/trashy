mod opt;

use std::error::Error;
use std::process;

use ansi_term::Colour::Red;
use ansi_term::Style;
use env_logger::Builder;
use eyre::{EyreHandler, Result};
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
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

// fn report(error: &(dyn Error + 'static)) {
//     let s = error.to_string;
//     if let Some(idx) = s.find(':') {

//     }
// }

// struct Handler;

// impl EyreHandler for Handler {
//     fn debug(
//         &self,
//         error: &(dyn Error + 'static),
//         f: &mut core::fmt::Formatter<'_>,
//     ) -> core::fmt::Result {
//         writeln!(
//             f,
//             "{}: {}",
//             Style::new().bold().fg(Red).paint("error"),
//             error
//         )
//         .unwrap();
//         if let Some(source) = error.source() {
//             writeln!(f, "\n\nCaused by:").unwrap();
//             for (i, e) in std::iter::successors(Some(source), |e| e.source()).enumerate() {
//                 writeln!(f, "   {}: {}", i, e).unwrap();
//             }
//         }
//         Ok(())
//     }
// }
