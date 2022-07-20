use std::{collections::HashSet, path::Path};

use aho_corasick::AhoCorasick;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use clap::{clap_derive::ArgEnum, Parser};

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::RegexSet;
use trash::TrashItem;

#[derive(Parser, Debug)]
pub struct FilterArgs {
    /// Filter by time (older than)
    /// 
    /// Filter results based on when the file was trashed. The argument can be provided 
    /// as a specific point in time (YYYY-MM-DD HH:MM:SS) or as a duration (10h, 1d, 35min).
    /// '--older-than' or '--older' can be used as aliases.
    /// This option can be used in 'list', 'restore', and 'empty'
    /// Examples:
    /// --before '2018-10-27 10:00:00'
    /// --older-than 2weeks
    /// --older 2018-10-27
    #[clap(long, alias = "older-than", alias = "older")]
    pub before: Option<String>,

    /// Filter by time (newer than)
    ///
    /// Filter results based on when the file was trashed. The argument can be provided
    /// as a specific point in time (YYYY-MM-DD HH:MM:SS) or as a duration (10h, 1d, 35min).
    /// '--newer-than' or '--newer' can be used as aliases.
    /// This option can be used in 'list', 'restore', and 'empty'
    /// Examples:
    ///     --changed-within 2weeks
    ///     --change-newer-than '2018-10-27 10:00:00'
    ///     --newer 2018-10-27
    #[clap(long, alias = "newer-than", alias = "newer")]
    pub within: Option<String>,

    /// Filter by pattern
    /// 
    /// This will filter using a pattern type specified in '--match'.
    pub patterns: Vec<String>,

    /// What type of pattern to use
    /// 
    /// This will choose the pattern type use in <PATTERNS>
    #[clap(short, long, arg_enum, default_value_t = Match::Regex)]
    pub r#match: Match,
}

impl FilterArgs {
    pub fn to_filters(&self) -> Result<Filters> {
        if self.patterns.is_empty() && self.within.is_none() && self.before.is_none() {
            return Ok(Filters(Vec::new()));
        }
        let now = Utc::now();
        let parse_time = |s| -> Result<Option<DateTime<Utc>>> {
            Ok(match s {
                None => None,
                Some(s) => Some(
                    parse_time_filter(now, s).ok_or_else(|| anyhow!("Invalid duration or date"))?,
                ),
            })
        };
        let before = parse_time(self.before.as_deref())?;
        let within = parse_time(self.within.as_deref())?;
        let patterns = match self.r#match {
            Match::Regex => Patterns::Regex(RegexSet::new(&self.patterns)?),
            Match::Substring => Patterns::Substring(AhoCorasick::new(&self.patterns)),
            Match::Glob => Patterns::Glob(new_globset(self.patterns.iter().map(|s| &**s))?),
            Match::Exact => Patterns::Exact(self.patterns.iter().cloned().collect()),
        };
        let filters = [
            before.map(|time| Filter::Time(TimeFilter::Before(time))),
            within.map(|time| Filter::Time(TimeFilter::After(time))),
            Some(Filter::Patterns(patterns)),
        ]
        .into_iter()
        .flatten()
        .collect();
        Ok(Filters(filters))
    }
}

fn new_globset<'a>(i: impl IntoIterator<Item = &'a str>) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for s in i {
        builder.add(Glob::new(s)?);
    }
    Ok(builder.build()?)
}

pub struct Filters(pub Vec<Filter>);

impl Filters {
    pub fn is_match(&self, item: &TrashItem) -> bool {
        self.is_empty() || self.0.iter().any(|filter| filter.is_match(item))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
pub enum Filter {
    Patterns(Patterns),
    Time(TimeFilter),
}

impl Filter {
    pub fn is_match(&self, item: &TrashItem) -> bool {
        match self {
            Filter::Patterns(patterns) => {
                patterns.is_match(&item.original_path().to_string_lossy())
            }
            Filter::Time(time_filter) => time_filter.is_match(Utc.timestamp(item.time_deleted, 0)),
        }
    }
}

pub enum TimeFilter {
    Before(DateTime<Utc>),
    After(DateTime<Utc>),
}

impl TimeFilter {
    fn is_match(&self, datetime: DateTime<Utc>) -> bool {
        match self {
            TimeFilter::Before(limit) => datetime < *limit,
            TimeFilter::After(limit) => datetime > *limit,
        }
    }
}

pub enum Patterns {
    Regex(RegexSet),
    Substring(AhoCorasick),
    Glob(GlobSet),
    Exact(HashSet<String>),
}

impl Patterns {
    fn is_match(&self, s: &str) -> bool {
        match self {
            Patterns::Regex(re_set) => re_set.is_match(s),
            Patterns::Substring(ac) => ac.is_match(s),
            Patterns::Glob(glob) => glob.is_match(Path::new(s)),
            Patterns::Exact(set) => set.contains(s),
        }
    }
}

#[derive(Debug, ArgEnum, Clone, Copy)]
pub enum Match {
    Regex,
    Substring,
    Glob,
    Exact,
}

fn parse_time_filter(ref_time: DateTime<Utc>, s: &str) -> Option<DateTime<Utc>> {
    humantime::parse_duration(s)
        .ok()
        .and_then(|duration| Some(ref_time - chrono::Duration::from_std(duration).ok()?))
        .or_else(|| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.into())
                .ok()
                .or_else(|| {
                    NaiveDate::parse_from_str(s, "%F")
                        .map(|nd| nd.and_hms(0, 0, 0))
                        .ok()
                        .and_then(|ndt| Local.from_local_datetime(&ndt).single())
                })
                .or_else(|| Local.datetime_from_str(s, "%F %T").ok())
                .map(|dt| dt.into())
        })
}
