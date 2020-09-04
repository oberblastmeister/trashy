use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::str::FromStr;

use chrono::NaiveDateTime;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::char;
use nom::combinator::{all_consuming, map_res};
use nom::error::{context, convert_error, VerboseError};
use nom::sequence::delimited;
use nom::{Err, IResult};
use percent_encoding::percent_decode_str;
use snafu::{ResultExt, Snafu};

use crate::trash_info::{self, TrashInfo};

pub const TRASH_DATETIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

fn parse_line<'a>(
    i: &'a str,
    key: &'static str,
) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    let (i, _) = context(key, tag(key))(i)?;
    let (i, _) = context("equal sign", char('='))(i)?;
    is_not("\n")(i)
}

fn parse_path_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    parse_line(i, "Path")
}

fn parse_deletion_date_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    parse_line(i, "DeletionDate")
}

fn parse_header_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    context("header", delimited(char('['), tag("Trash Info"), char(']')))(i)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(context(false))]
    #[snafu(display("Failed to parse Trash Info file:\n{}", source))]
    NomError {
        source: NomError,
    },

    #[snafu(display("Failed to parse NaiveDateTime from string {}: {}", date, source))]
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

struct TrashInfoStr<'a, 'b> {
    path: Cow<'a, str>,
    deletion_date: &'b str,
}

impl<'a, 'b> TryFrom<TrashInfoStr<'a, 'b>> for TrashInfo {
    type Error = Error;

    fn try_from(value: TrashInfoStr<'a, 'b>) -> Result<TrashInfo> {
        let path = value.path.into_owned();
        let deletion_date =
            NaiveDateTime::parse_from_str(&value.deletion_date, TRASH_DATETIME_FORMAT).context(
                ParseNaiveDate {
                    date: value.deletion_date,
                },
            )?;

        Ok(TrashInfo::new(path, deletion_date)?)
    }
}

fn parse_trash_info_str<'a>(i: &'a str) -> IResult<&str, TrashInfoStr, VerboseError<&'a str>> {
    let (i, _) = parse_header_line(i)?;
    let (i, _) = char('\n')(i)?;
    let (i, path) = context(
        "percent str",
        map_res(parse_path_line, |s| percent_decode_str(s).decode_utf8()),
    )(i)?;
    let (i, _) = char('\n')(i)?;
    let (i, deletion_date) = all_consuming(context("date", parse_deletion_date_line))(i)?;

    Ok((
        i,
        TrashInfoStr {
            path,
            deletion_date,
        },
    ))
}

pub fn parse_trash_info<'a>(i: &'a str) -> Result<TrashInfo, Error> {
    let (_, trash_info_str) = parse_trash_info_str(i).map_err(|e| NomError::build(e, i))?;
    trash_info_str.try_into()
}

impl FromStr for TrashInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<TrashInfo> {
        let trash_info = parse_trash_info(s)?;
        Ok(trash_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only returns chrono result because if parsing with nom has failed this will return an error
    /// message and panic instead of returning a result.
    // fn test_parse_trash_info<'a>(trash_info_str: &'a str, actual: (&str, &str)) {
    //     use std::borrow::Cow;

    //     let (_, expected) = match parse_trash_info(trash_info_str)
    //         .map_err(|e| NomError::build(e, trash_info_str))
    //     {
    //         Ok(expected) => expected,
    //         Err(e) => {
    //             println!("{}", e);
    //             panic!("an error occurred");
    //         }
    //     };

    //     let date_time = NaiveDateTime::parse_from_str(actual.1, TRASH_DATETIME_FORMAT)
    //         .expect("Actual date was not correct");

    //     let actual = TrashInfo {
    //         path: Cow::Borrowed(actual.0),
    //         deletion_date: date_time,
    //     };

    //     assert_eq!(expected, actual);
    // }

    #[test]
    fn parse_header_line_test() {
        assert_eq!(parse_header_line("[Trash Info]"), Ok(("", "Trash Info")));
    }

    #[test]
    fn tag_whitespace_test() {
        assert_eq!(
            tag::<&str, &str, VerboseError<&str>>("Trash Info")("Trash Info "),
            Ok((" ", "Trash Info"))
        );
    }

    #[test]
    fn value_path_test() {
        assert_eq!(
            parse_path_line("Path=/home/brian/.config"),
            Ok(("", "/home/brian/.config"))
        );
    }

    #[test]
    fn value_path_whitespace_test() {
        assert_eq!(
            parse_path_line("Path= /home/brian/.config"),
            Ok(("", " /home/brian/.config"))
        );
    }

    // #[test]
    // fn parse_function_test() {
    //     let trash_info_str =
    //         "[Trash Info]\nPath=/home/brian/dude.txt\nDeletionDate=2020-08-28T16:16:55";

    //     test_parse_trash_info(
    //         trash_info_str,
    //         ("/home/brian/dude.txt", "2020-08-28T16:16:55"),
    //     )
    // }
}
