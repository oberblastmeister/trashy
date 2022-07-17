use std::cmp;

use anyhow::{bail, Context, Result};

use crate::{range::Range, range_set::RangeSet};

fn parse_range_set(s: &str) -> Result<RangeSet> {
    s.split(' ').map(parse_range).collect()
}

fn parse_range(s: &str) -> Result<Range> {
    if s.is_empty() {
        bail!("Could not parse empty string");
    }
    let mut split = s.split("..");
    let start = split.next().expect("BUG: must have at least one");
    let start = start
        .parse::<u32>()
        .with_context(|| format!("Failed to parse `{}` before - into a number", start))?
        - 1;
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
