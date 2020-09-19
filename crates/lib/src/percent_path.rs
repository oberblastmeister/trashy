use core::str::Utf8Error;
use std::borrow::Cow;
use std::fmt;
use std::path::Path;

use crate::utils::{self, convert_to_str};
use percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display(
        "The decoded form of the string `{}` was not well formed in utf-8: {}",
        s,
        source
    ))]
    Decode { s: String, source: Utf8Error },

    #[snafu(context(false))]
    #[snafu(display("Utils error: {}", source))]
    Utils { source: utils::Error },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PercentPath(String);

impl PercentPath {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let s = convert_to_str(path)?;
        Ok(Self(utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()))
    }

    pub fn from_str(s: &str) -> Self {
        Self(utf8_percent_encode(s, NON_ALPHANUMERIC).to_string())
    }

    pub fn encoded(&self) -> &str {
        &self.0
    }

    pub fn decoded(&self) -> Result<Cow<'_, str>> {
        percent_decode_str(&self.0)
            .decode_utf8()
            .context(Decode { s: &self.0 })
    }
}

impl fmt::Display for PercentPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
