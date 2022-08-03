use std::{cmp, ops};

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

    pub fn to_std(self) -> ops::Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn into_iter(self) -> impl Iterator<Item = u32> {
        self.start..self.end
    }
}

impl From<ops::Range<u32>> for Range {
    fn from(range: ops::Range<u32>) -> Self {
        Range {
            start: range.start,
            end: range.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union() {
        assert_eq!(
            Range::new(1, 2).union(Range::new(2, 3)),
            Some(Range::new(1, 3))
        );
        assert_eq!(Range::new(1, 3).union(Range::new(4, 5)), None);
        assert_eq!(
            Range::new(1, 3).union(Range::new(2, 5)),
            Some(Range::new(1, 5))
        );
    }
}
