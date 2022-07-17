// use std::ops::Range;
use std::str::FromStr;
use std::{cmp, vec};

use anyhow::{bail, Context, Result};

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum RestoreIndex {
//     /// includes both start and end
//     Range(Range<usize>),
//     Point(usize),
// }
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
    start: u32,
    end: u32,
}

impl Range {
    pub fn new(start: u32, end: u32) -> Range {
        let new_start = cmp::min(start, end);
        let new_end = cmp::max(start, end);
        Range {
            start: new_start,
            end: new_end,
        }
    }

    pub fn start(self) -> u32 {
        self.start
    }

    pub fn end(self) -> u32 {
        self.end
    }

    pub fn is_contiguous(self, other: Range) -> bool {
        cmp::max(self.start, other.start) <= cmp::min(self.end, other.end)
    }

    pub fn union(self, other: Range) -> Option<Range> {
        if self.is_contiguous(other) {
            Some(Range {
                start: cmp::min(self.start, other.start),
                end: cmp::max(self.end, other.end),
            })
        } else {
            None
        }
    }
}

// impl FromStr for RestoreIndex {
//     type Err = eyre::Report;

//     /// converts from a one based index including the end to 0 based index excluding the end
//     fn from_str(s: &str) -> Result<RestoreIndex> {
//         Ok(RestoreIndex::Range(start..end))
//     }
// }

// #[derive(Debug, Clone)]
// pub struct RestoreIndexMultiple(Vec<RestoreIndex>);

// impl FromStr for RestoreIndexMultiple {
//     type Err = eyre::Report;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if s.is_empty() {
//             bail!("Could not parse empty string into restore indexes")
//         }
//         let mut res = Vec::new();
//         for s in s.split_whitespace() {
//             let restore_index = s.parse::<RestoreIndex>()?;
//             if res.is_empty() {
//                 res.push(restore_index)
//             } else {
//                 if res
//                     .iter()
//                     .any(|existing| existing.is_overlapping(&restore_index))
//                 {
//                     bail!("Overlapping range found: {:?}", restore_index);
//                 }
//                 res.push(restore_index)
//             }
//         }
//         Ok(RestoreIndexMultiple(res))
//     }
// }

// impl IntoIterator for RestoreIndexMultiple {
//     type Item = RestoreIndex;
//     type IntoIter = vec::IntoIter<RestoreIndex>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iter()
//     }
// }

// trait Overlap {
//     fn is_overlapping(&self, other: &Self) -> bool;
// }

// impl Overlap for usize {
//     fn is_overlapping(&self, other: &usize) -> bool {
//         self == other
//     }
// }

// impl Overlap for Range<usize> {
//     fn is_overlapping(&self, other: &Self) -> bool {
//         cmp::max(self.start, other.start) <= cmp::min(self.end, other.end)
//     }
// }

// impl Overlap for RestoreIndex {
//     fn is_overlapping(&self, other: &Self) -> bool {
//         match (self, other) {
//             (RestoreIndex::Point(p), RestoreIndex::Range(range)) => range.contains(p),
//             (RestoreIndex::Range(range), RestoreIndex::Point(p)) => range.contains(p),
//             (RestoreIndex::Range(range1), RestoreIndex::Range(range2)) => {
//                 range1.is_overlapping(range2)
//             }
//             (RestoreIndex::Point(p1), RestoreIndex::Point(p2)) => p1.is_overlapping(p2),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use RestoreIndex::*;

//     fn test_parse_restore_index(s: &str, actual: RestoreIndex) {
//         assert_eq!(
//             s.parse::<RestoreIndex>()
//                 .expect(&format!("Failed to parse str `{}` into a restore index", s)),
//             actual
//         );
//     }

//     #[test]
//     fn from_str() {
//         test_parse_restore_index("123-1234", Range(122..1234));
//     }

//     #[test]
//     fn point_from_str_test() {
//         test_parse_restore_index("5", Point(4));
//     }

//     #[test]
//     fn another_point_from_str_multiple_char_test() {
//         test_parse_restore_index("12347", Point(12346));
//     }

//     #[should_panic]
//     #[test]
//     fn missing_end_test() {
//         "2340958-".parse::<RestoreIndex>().unwrap();
//     }

//     #[should_panic]
//     #[test]
//     fn missing_beginning_test() {
//         "-123434".parse::<RestoreIndex>().unwrap();
//     }

//     #[should_panic]
//     #[test]
//     fn too_many_dashes_test() {
//         "123---1234".parse::<RestoreIndex>().unwrap();
//     }

//     #[should_panic]
//     #[test]
//     fn not_a_number_test() {
//         "hello".parse::<RestoreIndex>().unwrap();
//     }

//     #[should_panic]
//     #[test]
//     fn parse_nothing_test() {
//         "".parse::<RestoreIndex>().unwrap();
//     }

//     #[test]
//     fn is_overlapping_same_range_test() {
//         assert!((1..1).is_overlapping(&(1..1)));
//     }

//     #[test]
//     fn is_overlapping2_range_test() {
//         assert!((1..10).is_overlapping(&(1..4)));
//     }

//     #[test]
//     fn is_not_overlapping_range_test() {
//         assert!(!(1..4).is_overlapping(&(10..1234)));
//     }

//     #[test]
//     fn is_overlapping_test() {
//         assert!(Range(1..3).is_overlapping(&Range(1..3)));
//     }

//     #[test]
//     fn is_overlapping2_test() {
//         assert!(Range(1..9).is_overlapping(&Range(3..6)));
//     }

//     #[test]
//     fn is_not_overlapping_test() {
//         assert!(!Range(1..3).is_overlapping(&Range(5..10)));
//     }

//     #[test]
//     fn is_overlapping_same_test() {
//         assert!(Range(1..1).is_overlapping(&Range(1..1)));
//     }

//     #[test]
//     fn is_overlapping_different_test() {
//         assert!(Point(5).is_overlapping(&(Range(1..15))));
//     }

//     #[test]
//     fn is_overlapping_different2_test() {
//         assert!(Range(3..8).is_overlapping(&Point(4)));
//     }

//     #[test]
//     fn is_overlapping_points_test() {
//         assert!(Point(4).is_overlapping(&Point(4)));
//     }

//     #[test]
//     fn get_multiple_test() {
//         assert_eq!(
//             "4 40 3 9-12".parse::<RestoreIndexMultiple>().unwrap().0,
//             vec![Point(3), Point(39), Point(2), Range(8..12),]
//         );
//     }

//     #[should_panic]
//     #[test]
//     fn get_multiple_overlapping_test() {
//         "4 30 5-13 7-8 9".parse::<RestoreIndexMultiple>().unwrap();
//     }

//     #[should_panic]
//     #[test]
//     fn get_multiple_none() {
//         "".parse::<RestoreIndexMultiple>().unwrap();
//     }
// }
