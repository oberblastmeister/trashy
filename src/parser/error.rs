use std::fmt;
use nom::error::{convert_error, VerboseError};
use nom::Err;
use snafu::Snafu;

use crate::trash_info;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    #[snafu(display("Failed to parse Trash Info file:\n{}", source))]
    NomError {
        source: NomError,
    },

    #[snafu(display("Failed to parse NaiveDateTime from string {}: {}", date, source))]
    #[snafu(visibility(pub(super)))]
    ParseNaiveDate {
        source: chrono::format::ParseError,
        date: String,
    },

    #[snafu(context(false))]
    #[snafu(display("Could not create TrashInfo struct: {}", source))]
    TrashInfoCreation {
        source: trash_info::Error,
    },
}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug)]
pub struct NomError {
    source: String,
}

impl NomError {
    pub fn build(source: Err<VerboseError<&str>>, input: &str) -> Self {
        let source = match source {
            Err::Incomplete(_) => format!("{}", source),
            Err::Failure(e) | Err::Error(e) => convert_error(input, e),
        };

        NomError { source }
    }
}

impl fmt::Display for NomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl std::error::Error for NomError {}
