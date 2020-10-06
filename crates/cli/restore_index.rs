use std::ops::Index;
use std::slice;
use std::str::FromStr;

use eyre::{eyre, Context, Result};

use crate::utils::input;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RestoreIndex {
    /// includes both start and end
    Range { start: usize, end: usize },
    Point(usize),
}

impl FromStr for RestoreIndex {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<RestoreIndex> {
        let mut split = s.split('-');
        // let start = split.next().ok_or(err)
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
        };

        let end = split
            .next()
            .ok_or(eyre!("Expected number after `-`, got none"))?;
        let end = end
            .parse::<usize>()
            .wrap_err(format!("Failed to parse `{}` into a number", end))?;
        Ok(RestoreIndex::Range { start, end })
    }
}

impl<T> Index<RestoreIndex> for Vec<T> {
    type Output = [T];

    fn index(&self, input_type: RestoreIndex) -> &Self::Output {
        match input_type {
            RestoreIndex::Range { start, end } => &self[start..=end],
            RestoreIndex::Point(n) => slice::from_ref(&self[n]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestoreIndices(Vec<RestoreIndex>);

impl FromStr for RestoreIndices {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input_types = s
            .split_ascii_whitespace()
            .map(|s| s.parse::<RestoreIndex>())
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(input_types))
    }
}

impl IntoIterator for RestoreIndices {
    type Item = RestoreIndex;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub fn input_restore_indices(msg: &str) -> Result<RestoreIndices> {
    let s = input(msg)?;
    let res = s.parse::<RestoreIndices>()?;
    Ok(res)
}
