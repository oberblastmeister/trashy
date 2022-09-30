use anyhow::{bail, Context, Result};

use crate::{range::Range, range_set};

pub fn parse_range_set(s: &str) -> Result<range_set::RangeSet> {
    parse_ranges(s).collect()
}

pub fn parse_ranges(s: &str) -> impl Iterator<Item = Result<Range>> + '_ {
    s.split(char::is_whitespace).map(parse_range)
}

pub fn parse_range(s: &str) -> Result<Range> {
    if s.is_empty() {
        bail!("Could not parse empty string");
    }
    let mut split = s.split("..");
    let start = split.next().expect("BUG: must have at least one");
    let start = start
        .parse::<u32>()
        .with_context(|| format!("Failed to parse `{}` before - into a number", start))?;
    match split.next() {
        Some(end) => {
            let end = end
                .parse::<u32>()
                .with_context(|| format!("Failed to parse `{}` after - into a number", end))?;
            if split.next().is_some() {
                bail!("Unexpected second - found");
            }
            Ok(Range::new(start, end))
        }
        None => Ok(Range::new(start, start + 1)),
    }
}

#[cfg(test)]
mod tests {
    use std::ops;

    use crate::range_set::RangeSet;

    use super::*;

    pub fn parse_range_set(s: &str) -> Result<RangeSet> {
        parse_ranges(s).collect()
    }

    fn parse_succeed<const N: usize>(s: &str, expect: [ops::Range<u32>; N]) {
        assert_eq!(
            parse_range_set(s)
                .unwrap_or_else(|_| panic!("Failed to parse str `{}` into a restore index", s)),
            expect.into_iter().map(Into::into).collect()
        );
    }

    fn parse_fail(s: &str) {
        let res = parse_range_set(s);
        assert!(res.is_err(), "Expected parsing to fail, got {:?}", res.unwrap());
    }

    #[test]
    fn test_succeed() {
        parse_succeed("123..1234", [123..1234]);
        parse_succeed("7 8 4", [4..5, 7..8, 8..9]);
        parse_succeed("5", [5..6]);
    }

    #[test]
    fn parse_errors() {
        parse_fail("2340958..");
        parse_fail("2340958..    ");
        parse_fail("2340958.    ");
        parse_fail("..1234");
        parse_fail(".1234");
        parse_fail("");
    }
}
