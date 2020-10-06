use std::borrow::Cow;
use std::convert::TryInto;

use chrono::NaiveDateTime;
// use nom::bytes::complete::{is_not, tag};
// use nom::character::complete::char;
// use nom::combinator::{all_consuming, map_res};
// use nom::error::{context, VerboseError};
// use nom::sequence::delimited;
// use nom::IResult;
use percent_encoding::percent_decode_str;
use snafu::{ResultExt, Snafu};

use super::error::ParseNaiveDate;
use super::error::{NomError, Result};
use crate::percent_path::PercentPath;
use crate::trash_info::TrashInfo;

pub const TRASH_DATETIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

// fn parse_line<'a>(
//     i: &'a str,
//     key: &'static str,
// ) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
//     let (i, _) = context(key, tag(key))(i)?;
//     let (i, _) = context("equal sign", char('='))(i)?;
//     is_not("\n")(i)
// }

// fn parse_path_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
//     parse_line(i, "Path")
// }

// fn parse_deletion_date_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
//     parse_line(i, "DeletionDate")
// }

// fn parse_header_line<'a>(i: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
//     context("header", delimited(char('['), tag("Trash Info"), char(']')))(i)
// }

// fn parse_trash_info_str<'a>(i: &'a str) -> IResult<&str, TrashInfoStr, VerboseError<&'a str>> {
//     let (i, _) = parse_header_line(i)?;
//     let (i, _) = char('\n')(i)?;
//     let (i, path) = context(
//         "percent str",
//         map_res(parse_path_line, |s| percent_decode_str(s).decode_utf8()),
//     )(i)?;
//     let (i, _) = char('\n')(i)?;
//     let (i, deletion_date) = all_consuming(context("date", parse_deletion_date_line))(i)?;

//     Ok((
//         i,
//         TrashInfoStr {
//             path,
//             deletion_date,
//         },
//     ))
// }

fn parse_header(i: &str) -> ParseError<&str, &str> {
    tag("[Trash Info]")(i)
}

type ParseError<I, O, E = Error> = Result<(I, O), E>;

fn tag(tag: &str) -> impl Fn(&str) -> ParseError<&str, &str> + '_ {
    move |i| {
        if i.starts_with(tag) {
            let idx = tag.len();
            Ok((&i[idx..], &i[..idx]))
        } else {
            Tag { i }.fail()
        }
    }
}

fn char(c: char) -> impl Fn(&str) -> ParseError<&str, char> {
    move |i| {
        if i.starts_with(c) {
            Ok((&i[1..], c))
        } else {
            Tag { i }.fail()
        }
    }
}

fn all_consuming<O, F>(f: F) -> impl Fn(&str) -> ParseError<&str, O>
where
    F: Fn(&str) -> ParseError<&str, O>,
{
    move |i| {
        let (i, o) = f(i)?;
        if i.len() == 0 {
            Ok((i, o))
        } else {
            Eof { i }.fail()
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    Tag { i: String },
    Char { i: String },
    Eof { i: String },
}

pub fn parse_trash_info<'a>(i: &'a str) -> Result<TrashInfo> {
    let (_, trash_info_str) = parse_trash_info_str(i).map_err(|e| NomError::build(e, i))?;
    trash_info_str.try_into()
}

struct TrashInfoStr<'a, 'b> {
    path: Cow<'a, str>,
    deletion_date: &'b str,
}

impl<'a, 'b> TryInto<TrashInfo> for TrashInfoStr<'a, 'b> {
    type Error = Error;

    fn try_into(self: TrashInfoStr<'a, 'b>) -> Result<TrashInfo> {
        let percent_path = PercentPath::from_str(self.path.as_ref());
        let deletion_date =
            NaiveDateTime::parse_from_str(&self.deletion_date, TRASH_DATETIME_FORMAT).context(
                ParseNaiveDate {
                    date: self.deletion_date,
                },
            )?;

        Ok(TrashInfo::new(percent_path, Some(deletion_date)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Context, Result};
    use std::str::FromStr;

    /// Only returns chrono result because if parsing with nom has failed this will return an error
    /// message and panic instead of returning a result.
    fn test_parse_trash_info<'a>(trash_info_str: &'a str, actual: (&str, &str)) -> Result<()> {
        let expected = match TrashInfo::from_str(trash_info_str) {
            Err(e) => {
                eprintln!("{}", e);
                panic!("An error occurred");
            }
            Ok(trash_info) => trash_info,
        };

        let actual = TrashInfo::new(
            PercentPath::from_str(actual.0),
            Some(NaiveDateTime::parse_from_str(
                actual.1,
                TRASH_DATETIME_FORMAT,
            )?),
        );

        assert_eq!(expected, actual);

        Ok(())
    }

    fn test_parse_trash_info_run(trash_info_str: &str, actual: (&str, &str)) {
        match test_parse_trash_info(trash_info_str, actual) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("{}", e);
                panic!("An error occurred when testing parse_trash_info");
            }
        }
    }

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

    #[ignore]
    #[test]
    fn trash_info_parse_test() {
        test_parse_trash_info_run(
            "[Trash Info]\nPath=/home/brian/dude.txt\nDeletionDate=2020-08-28T16:16:55",
            ("/home/brian/dude.txt", "2020-09-21T08:34:36"),
        )
    }
}
