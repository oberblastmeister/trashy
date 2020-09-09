use std::fmt;

use chrono::NaiveDateTime;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::char;
use nom::combinator::all_consuming;
use nom::error::{context, convert_error, VerboseError};
use nom::sequence::delimited;
use nom::{Err, IResult};
use percent_encoding::percent_decode_str;
use snafu::{OptionExt, ResultExt, Snafu};

use crate::trash_info::TrashInfo;

pub const TRASH_DATETIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

fn key<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context("until equal", take_until("="))(i)
}

fn key_value<'a>(i: &'a str) -> IResult<&'a str, (&'a str, &'a str), VerboseError<&'a str>> {
    let (i, key) = context("getting key", key)(i)?;
    let (value, _) = char('=')(i)?;
    Ok(("", (key, value)))
}

fn path<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context("Path", all_consuming(tag("Path")))(i)
}

fn deletion_date<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context("DeletionDate", all_consuming(tag("DeletionDate")))(i)
}

fn check_key<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context("valid keys", alt((path, deletion_date)))(i)
}

fn header<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context(
        "delimited by [ and ]",
        all_consuming(delimited(char('['), is_not("]"), char(']'))),
    )(i)
}

fn trash_info_header<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context("valid header", all_consuming(tag("Trash Info")))(i)
}

fn parse_header_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    let (_, header) = header(i)?;
    trash_info_header(header)
}

fn parse_key_value_line<'a>(
    i: &'a str,
) -> IResult<&'a str, (&'a str, &'a str), VerboseError<&'a str>> {
    let (_, (key, value)) = key_value(i)?;
    let (_, key) = check_key(key)?;
    Ok(("", (key, value)))
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("There were not enough lines in the Trash Info file:\n{}", lines))]
    NotEnoughLines { lines: String },

    #[snafu(context(false))]
    #[snafu(display("Failed to parse Trash Info file:\n{}", source))]
    Nom { source: NomError },

    #[snafu(display("Failed to parse NaiveDateTime `{}`: {}", date, source))]
    ParseNaiveDate {
        date: String,
        source: chrono::format::ParseError,
    },

    #[snafu(display(
        "Percent-decoded bytes from {} are not well-formed in UTF-8: {}",
        percent_str,
        source
    ))]
    Utf8 {
        source: core::str::Utf8Error,
        percent_str: String,
    },
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug)]
pub struct NomError {
    source: String,
}

impl NomError {
    fn build(source: Err<VerboseError<&str>>, input: &str) -> Self {
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

pub fn parse_trash_info(i: &str) -> Result<TrashInfo> {
    let mut lines = i.lines();
    let first_line = lines.next().context(NotEnoughLines {
        lines: lines.clone().collect::<String>(),
    })?;

    let (_, _) = parse_header_line(first_line).map_err(|e| NomError::build(e, first_line))?;

    let mut path = None;
    let mut deletion_date = None;

    for line in lines {
        let (_, (key, value)) = parse_key_value_line(line).map_err(|e| NomError::build(e, line))?;

        match key {
            "Path" => {
                let percent_decode = percent_decode_str(value).decode_utf8().map(|cow| cow.into_owned()).context(Utf8 { percent_str: value })?;
                path = Some(percent_decode);
            },
            "DeletionDate" => {
                deletion_date = Some(NaiveDateTime::parse_from_str(value, TRASH_DATETIME_FORMAT).context(ParseNaiveDate { date: value })?);
            }
            s => panic!("BUG: key was checked above using check_key. Key cannot be invalid because check_key should have returned above. Invalid key: `{}`", s),
        }
    }

    let path = path.expect("BUG: must have path value");
    let deletion_date = deletion_date.expect("BUG: must have deletion date");

    Ok(TrashInfo {
        path,
        deletion_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    type ChronoResult<T> = chrono::format::ParseResult<T>;

    /// Only returns chrono result because if parsing with nom has failed this will return an error
    /// message and panic instead of returning a result.
    fn test_parse_trash_info(trash_info_str: &str, actual: (&str, &str)) -> Result<(), Error> {
        use std::borrow::Cow;

        let expected = parse_trash_info(trash_info_str)?;

        let date_time = NaiveDateTime::parse_from_str(actual.1, TRASH_DATETIME_FORMAT).context(
            ParseNaiveDate {
                date: actual.1.to_string(),
            },
        )?;

        // let actual = TrashInfo {
            // path: Cow::Borrowed(actual.0),
            // deletion_date: date_time,
        // };

        // assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn key_test() {
        assert_eq!(key("key=value"), Ok(("=value", "key")));
    }

    #[test]
    fn key_value_test() {
        assert_eq!(key_value("key=value"), Ok(("", ("key", "value"))));
    }

    #[test]
    fn header_test() {
        assert_eq!(header("[hello dude]"), Ok(("", "hello dude")));
    }

    #[test]
    fn trash_info_header_test() {
        assert_eq!(trash_info_header("Trash Info"), Ok(("", "Trash Info")));
    }

    #[test]
    fn parse_function_test() {
        let trash_info_str = r#"[Trash Info]
Path=/home/brian/dude.txt
DeletionDate=2020-08-28T16:16:55
"#;

        match test_parse_trash_info(
            trash_info_str,
            ("/home/brian/dude.txt", "2020-08-28T16:16:55"),
        ) {
            Err(e) => {
                println!("{}", e);
                panic!("an error occurred");
            }
            Ok(()) => (),
        }
    }
}
