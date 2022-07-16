use std::fmt;

use ansi_term::Colour::Red;

pub fn err(s: impl fmt::Debug) {
    eprintln!("{}: {s:?}", Red.bold().paint("error:"));
}

pub fn err_display(s: impl fmt::Display) {
    eprintln!("{} {s}", Red.bold().paint("error:"));
}
