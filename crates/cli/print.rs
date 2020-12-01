use std::fmt;

use ansi_term::Color::Red;

pub fn err(s: impl fmt::Debug) {
    eprintln!("{}: {:?}", Red.bold().paint("Error"), s);
}

pub fn err_display(s: impl fmt::Display) {
    eprintln!("{}: {}", Red.bold().paint("Error"), s);
}
