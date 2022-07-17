use crate::range::Range;

pub struct RangeSet {
    ranges: Vec<Range>,
}

impl RangeSet {
    pub fn into_vec(self) -> Vec<Range> {
        self.ranges
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

fn normalize(ranges: &mut Vec<Range>) {
    loop {
        match (ranges.pop(), ranges.pop()) {
            (Some(r1), None) => break ranges.push(r1),
            (None, _) => break,
            (Some(r1), Some(r2)) => match r1.union(r2) {
                None => ranges.push(r2),
                Some(r3) => ranges.push(r3),
            },
        }
    }
}
