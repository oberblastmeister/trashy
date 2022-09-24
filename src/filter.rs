use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use aho_corasick::AhoCorasick;
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use clap::{ArgAction, Parser, ValueEnum};

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
    ///     --before '2018-10-27 10:00:00'
    ///     --older-than 2weeks
    ///     --older 2018-10-27
    #[arg(long, visible_alias = "older-than", visible_alias = "older", action = ArgAction::Append, verbatim_doc_comment)]
    pub before: Vec<String>,

    /// Filter by time
    ///
    /// Filter results based on when the file was trashed. The argument can be provided
    /// as a specific point in time (YYYY-MM-DD HH:MM:SS) or as a duration (10h, 1d, 35min).
    /// '--newer-than' or '--newer' can be used as aliases.
    /// This option can be used in 'list', 'restore', and 'empty'
    /// Examples:
    ///     --changed-within 2weeks
    ///     --change-newer-than '2018-10-27 10:00:00'
    ///     --newer 2018-10-27
    #[arg(long, visible_alias = "newer-than", visible_alias = "newer", action = ArgAction::Append, verbatim_doc_comment)]
    pub within: Vec<String>,

    /// Filter by regex
    #[arg(long, action = ArgAction::Append)]
    pub regex: Vec<String>,

    /// Filter by glob
    #[arg(long, action = ArgAction::Append)]
    pub glob: Vec<String>,

    /// Filter by substring
    #[arg(long, action = ArgAction::Append)]
    pub substring: Vec<String>,

    /// Filter by exact match
    #[arg(long, action = ArgAction::Append)]
    pub exact: Vec<String>,

    /// Filter by pattern
    ///
    /// This will filter using a pattern type specified in '--match'.
    /// Using <PATTERNS> and '--match' gives the same effect as passing one of the pattern options explicitly.
    /// So for example
    /// trash restore '~/projects/**' '~/builds/**' --match=glob
    /// is the same as
    /// trash restore --glob='~/project/**' --glob='~/builds/**'
    #[arg(verbatim_doc_comment)]
    pub patterns: Vec<String>,

    /// What type of pattern to use
    ///
    /// This will choose the pattern type used in <PATTERNS>.
    /// Each pattern type has it's own explicit option.
    #[arg(short, long, value_enum, default_value_t = Match::Regex)]
    pub r#match: Match,

    /// Filter by directory
    #[arg(short = 'd', long = "directory", visible_alias = "dir", action = ArgAction::Append)]
    pub directories: Vec<PathBuf>,
}

impl FilterArgs {
    pub fn to_filters(&self) -> Result<Filters> {
        let now = Utc::now();
        let parse_time =
            |s| parse_time_filter(now, s).ok_or_else(|| anyhow!("Invalid duration or date"));
        let mut filters = Vec::new();
        if !self.before.is_empty() {
            filters.extend(
                self.before
                    .iter()
                    .map(|ref s| Ok(Filter::Time(TimeFilter::Before(parse_time(s)?))))
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        if !self.within.is_empty() {
            filters.extend(
                self.within
                    .iter()
                    .map(|s| Ok(Filter::Time(TimeFilter::After(parse_time(s)?))))
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        if !self.directories.is_empty() {
            let dirs = Filter::Directories(
                self.directories
                    .iter()
                    .map(|p| {
                        if !p.is_dir() {
                            bail!("Path must be a directory");
                        };
                        Ok(fs::canonicalize(p)?)
                    })
                    .collect::<Result<Vec<_>>>()?,
            );
            filters.push(dirs);
        }
        if !self.regex.is_empty() {
            filters.push(Filter::PatternSet(PatternSet::new_regex(self.regex.iter())?));
        }
        if !self.glob.is_empty() {
            filters.push(Filter::PatternSet(PatternSet::new_glob(self.glob.iter())?));
        }
        if !self.substring.is_empty() {
            filters.push(Filter::PatternSet(PatternSet::new_substring(self.substring.iter())));
        }
        if !self.exact.is_empty() {
            filters.push(Filter::PatternSet(PatternSet::new_exact(self.exact.iter())));
        }
        if !self.patterns.is_empty() {
            filters.push(Filter::PatternSet(match self.r#match {
                Match::Regex => PatternSet::new_regex(self.patterns.iter())?,
                Match::Substring => PatternSet::new_substring(self.patterns.iter()),
                Match::Glob => PatternSet::new_glob(self.patterns.iter())?,
                Match::Exact => PatternSet::new_exact(self.patterns.iter()),
            }));
        }
        Ok(Filters(filters))
    }
}

pub struct Filters(pub Vec<Filter>);

impl Filters {
    pub fn is_match(&self, item: &TrashItem) -> bool {
        self.0.iter().all(|filter| filter.is_match(item))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug)]
pub enum Filter {
    PatternSet(PatternSet),
    Time(TimeFilter),
    Directories(Vec<PathBuf>),
}

impl Filter {
    pub fn is_match(&self, item: &TrashItem) -> bool {
        match self {
            Filter::PatternSet(patterns) => {
                patterns.is_match(&item.original_path().to_string_lossy())
            }
            Filter::Time(time_filter) => time_filter.is_match(Utc.timestamp(item.time_deleted, 0)),
            Filter::Directories(directories) => {
                directories.iter().all(|p| item.original_path().starts_with(p))
            }
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum PatternSet {
    Regex(RegexSet),
    Substring(Box<AhoCorasick>),
    Glob(GlobSet),
    Exact(HashSet<String>),
}

impl PatternSet {
    fn is_match(&self, s: &str) -> bool {
        match self {
            PatternSet::Regex(re_set) => re_set.is_match(s),
            PatternSet::Substring(ac) => ac.is_match(s),
            PatternSet::Glob(glob) => glob.is_match(Path::new(s)),
            PatternSet::Exact(set) => set.contains(s),
        }
    }

    fn new_regex(patterns: impl Iterator<Item = impl AsRef<str>>) -> Result<PatternSet> {
        Ok(PatternSet::Regex(RegexSet::new(patterns)?))
    }

    fn new_substring(patterns: impl Iterator<Item = impl AsRef<[u8]>>) -> PatternSet {
        PatternSet::Substring(Box::new(AhoCorasick::new(patterns)))
    }

    fn new_glob(patterns: impl Iterator<Item = impl AsRef<str>>) -> Result<PatternSet> {
        let mut builder = GlobSetBuilder::new();
        for s in patterns {
            builder.add(Glob::new(s.as_ref())?);
        }
        Ok(PatternSet::Glob(builder.build()?))
    }

    fn new_exact(patterns: impl Iterator<Item = impl AsRef<str>>) -> PatternSet {
        PatternSet::Exact(patterns.map(|s| String::from(s.as_ref())).collect())
    }
}

#[derive(Debug, ValueEnum, Clone, Copy)]
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
