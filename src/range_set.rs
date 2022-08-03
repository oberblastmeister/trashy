use crate::range::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct RangeSet {
    ranges: Vec<Range>,
}

impl RangeSet {
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

impl From<Vec<Range>> for RangeSet {
    fn from(mut ranges: Vec<Range>) -> Self {
        ranges.sort_by_key(|range| range.start());
        normalize(&mut ranges);
        RangeSet { ranges }
    }
}

impl Into<Vec<Range>> for RangeSet {
    fn into(self) -> Vec<Range> {
        self.ranges
    }
}

impl FromIterator<Range> for RangeSet {
    fn from_iter<T: IntoIterator<Item = Range>>(iter: T) -> Self {
        RangeSet::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl IntoIterator for RangeSet {
    type Item = Range;
    type IntoIter = <Vec<Range> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
    }
}

fn normalize(ranges: &mut Vec<Range>) {
    if ranges.len() < 2 {
        return;
    }
    let mut i = ranges.len() - 1;
    let mut d = 0;
    loop {
        match ranges[i - 1].union(ranges[i]) {
            None => (),
            Some(range) => {
                ranges[i - 1] = range;
                d += 1;
            }
        }
        if i == 1 {
            break;
        }
        i -= 1;
    }
    ranges.drain((ranges.len() - d)..);
}
