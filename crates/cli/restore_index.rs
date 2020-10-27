use std::ops::{Range, Index};
use std::slice;
use std::str::FromStr;
use std::cmp;

use eyre::{eyre, Context, Result, bail};

use crate::utils::input;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreIndex {
    /// includes both start and end
    Range(Range<usize>),
    Point(usize),
}

impl FromStr for RestoreIndex {
    type Err = eyre::Report;

    /// converts from a one based index including the end to 0 based index excluding the end
    fn from_str(s: &str) -> Result<RestoreIndex> {
        let mut split = s.split('-');
        let start = match split.next() {
            Some(start) => start.parse::<usize>().wrap_err(format!(
                "Failed to parse `{}` before - into a number",
                start
            ))?,
            None => {
                let n = s.parse::<usize>().wrap_err(format!(
                    "Failed to parse `{}` into number after trying range",
                    s
                ))?;
                return Ok(RestoreIndex::Point(n));
            }
        } - 1;

        let end = split
            .next()
            .ok_or(eyre!("Expected something after `-`"))?;
        let end = end
            .parse::<usize>()
            .wrap_err(format!("Failed to parse `{}` into a number", end))?;
        Ok(RestoreIndex::Range(start..end))
    }
}

impl RestoreIndex {
    pub fn get_multiple(s: &str) -> Result<Vec<RestoreIndex>> {
        let mut res = Vec::new();
        for s in s.split_whitespace() {
            let restore_index = s.parse::<RestoreIndex>()?;
            if res.is_empty() {
                res.push(restore_index)
            } else {
                if res.iter().any(|existing| existing.is_overlapping(&restore_index)) {
                    bail!("Overlapping range found: {:?}", restore_index);
                }
                res.push(restore_index)
            }
        }
        Ok(res)
    }
}

trait Overlap {
    fn is_overlapping(&self, other: &Self) -> bool;
}

impl Overlap for usize {
    fn is_overlapping(&self, other: &usize) -> bool {
        self == other
    }
}

impl Overlap for Range<usize> {
    fn is_overlapping(&self, other: &Self) -> bool {
        cmp::max(self.start, other.start) <= cmp::min(self.end, other.end)
    }
}

impl Overlap for RestoreIndex {
    fn is_overlapping(&self, other: &Self) -> bool {
        if self == other { return false };
        match (self, other) {
            (RestoreIndex::Point(p), RestoreIndex::Range(range)) => {
                range.contains(p)
            }
            (RestoreIndex::Range(range), RestoreIndex::Point(p)) => {
                range.contains(p)
            }
            (RestoreIndex::Range(range1), RestoreIndex::Range(range2)) => {
                range1.is_overlapping(range2)
            }
            (RestoreIndex::Point(p1), RestoreIndex::Point(p2)) => {
                p1.is_overlapping(p2)
            }
        }
    }
}
