use std::borrow::Cow;
use std::fmt;
use std::path::Path;
use std::str::Utf8Error;

use crate::utils::{self, convert_to_str};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};
use snafu::{ResultExt, Snafu};

/// The excluded characters that must be escaped with percents in the percent path of each trash
/// info file.
pub const ASCII_SET: &AsciiSet = &CONTROLS
    // space
    .add(b' ')
    // delims
    .add(b'<')
    .add(b'>')
    .add(b'#')
    .add(b'%')
    .add(b'"')
    // unwise
    .add(b'{')
    .add(b'}')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'[')
    .add(b']')
    .add(b'`');

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The decoded form of the string `{}` was not well formed in utf-8", s,))]
    Decode { s: String, source: Utf8Error },

    #[snafu(display("Failed to convert path to a string to be able to percent encode it"))]
    ConvertPathDecode { source: utils::Error },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PercentPath(String);

impl PercentPath {
    /// Create a new percent path from a str. Will encode all characters from the string.
    pub fn from_str(s: &str) -> Self {
        Self(utf8_percent_encode(s, ASCII_SET).to_string())
    }

    /// Use only if the string is already percent encoded. Use inside of the parser because the
    /// trash info file should have already encoded the path.
    pub(crate) fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub(crate) fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let s = convert_to_str(path).context(ConvertPathDecode)?;
        Ok(Self(utf8_percent_encode(s, ASCII_SET).to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_spaces_test() {
        let percent_path = PercentPath::from_str("/this is a path/");
        assert_eq!(
            percent_path,
            PercentPath(String::from("/this%20is%20a%20path/"))
        );
    }

    #[test]
    fn decode_spaces_test() {
        let s = "/this is a path/";
        let percent_path = PercentPath::from_str("/this is a path/");
        let percent_path = percent_path.decoded().unwrap();
        assert_eq!(percent_path, s);
    }
}
