use anyhow::{bail, Context, Result};

use crate::range::Range;

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
        // parse_fail("1234");
    }

    // #[should_panic]
    // #[test]
    // fn missing_beginning_test() {
    //     "-123434".parse::<RestoreIndex>().unwrap();
    // }

    // #[should_panic]
    // #[test]
    // fn too_many_dashes_test() {
    //     "123---1234".parse::<RestoreIndex>().unwrap();
    // }

    // #[should_panic]
    // #[test]
    // fn not_a_number_test() {
    //     "hello".parse::<RestoreIndex>().unwrap();
    // }

    // #[should_panic]
    // #[test]
    // fn parse_nothing_test() {
    //     "".parse::<RestoreIndex>().unwrap();
    // }

    // #[test]
    // fn is_overlapping_same_range_test() {
    //     assert!((1..1).is_overlapping(&(1..1)));
    // }

    // #[test]
    // fn is_overlapping2_range_test() {
    //     assert!((1..10).is_overlapping(&(1..4)));
    // }

    // #[test]
    // fn is_not_overlapping_range_test() {
    //     assert!(!(1..4).is_overlapping(&(10..1234)));
    // }

    // #[test]
    // fn is_overlapping_test() {
    //     assert!(Range(1..3).is_overlapping(&Range(1..3)));
    // }

    // #[test]
    // fn is_overlapping2_test() {
    //     assert!(Range(1..9).is_overlapping(&Range(3..6)));
    // }

    // #[test]
    // fn is_not_overlapping_test() {
    //     assert!(!Range(1..3).is_overlapping(&Range(5..10)));
    // }

    // #[test]
    // fn is_overlapping_same_test() {
    //     assert!(Range(1..1).is_overlapping(&Range(1..1)));
    // }

    // #[test]
    // fn is_overlapping_different_test() {
    //     assert!(Point(5).is_overlapping(&(Range(1..15))));
    // }

    // #[test]
    // fn is_overlapping_different2_test() {
    //     assert!(Range(3..8).is_overlapping(&Point(4)));
    // }

    // #[test]
    // fn is_overlapping_points_test() {
    //     assert!(Point(4).is_overlapping(&Point(4)));
    // }

    // #[test]
    // fn get_multiple_test() {
    //     assert_eq!(
    //         "4 40 3 9-12".parse::<RestoreIndexMultiple>().unwrap().0,
    //         vec![Point(3), Point(39), Point(2), Range(8..12),]
    //     );
    // }

    // #[should_panic]
    // #[test]
    // fn get_multiple_overlapping_test() {
    //     "4 30 5-13 7-8 9".parse::<RestoreIndexMultiple>().unwrap();
    // }

    // #[should_panic]
    // #[test]
    // fn get_multiple_none() {
    //     "".parse::<RestoreIndexMultiple>().unwrap();
    // }
}
