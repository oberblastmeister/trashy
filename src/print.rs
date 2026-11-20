use std::fmt;

use nu_ansi_term::Color::Red;

pub fn err_display(s: impl fmt::Display) {
    eprintln!("{} {s}", Red.bold().paint("error:"));
}
