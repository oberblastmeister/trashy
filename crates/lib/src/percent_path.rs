use std::borrow::Cow;
use std::fmt;
use core::str::Utf8Error;

use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use snafu::{Snafu, ResultExt};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The decoded form of the string `{}` was not well formed in utf-8: {}", s, source))]
    Decode {
        s: String,
        source: Utf8Error
    }
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq)]
pub struct PercentPath(String);

impl PercentPath {
    pub fn from_str(s: &str) -> Self {
        Self(utf8_percent_encode(s, NON_ALPHANUMERIC).to_string())
    }
    pub fn encoded(&self) -> &str {
        &self.0
    }

    pub fn decoded(&self) -> Result<Cow<'_, str>> {
        percent_decode_str(&self.0)
            .decode_utf8()
            .context(Decode { s: self.0 })
    }
}

impl fmt::Display for PercentPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
