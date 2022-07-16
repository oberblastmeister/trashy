use aho_corasick::AhoCorasick;
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use clap::{clap_derive::ArgEnum, Parser};
// use eyre::{eyre, Result, WrapErr};
// use log::error;

use dialoguer::Confirm;
use regex::{Regex, RegexSet};
use trash::TrashItem;

use crate::filter::FilterArgs;

// use crate::table;
// use crate::utils::Pair;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    filter_args: FilterArgs,

    /// Empty all files
    #[clap(long, conflicts_with_all = &["before", "within", "patterns"])]
    all: bool,

    /// Skip confirmation
    #[clap(long)]
    force: bool,
}

enum Patterns {
    Regex(RegexSet),
    Substring(AhoCorasick),
}

impl Patterns {
    fn is_match(&self, s: &str) -> bool {
        match self {
            Patterns::Regex(re_set) => re_set.is_match(s),
            Patterns::Substring(ac) => ac.is_match(s),
        }
    }
}

#[derive(Debug, ArgEnum, Clone, Copy)]
enum Match {
    Regex,
    Substring,
}

impl Args {
    #[cfg(target_os = "macos")]
    pub fn run(&self, global_args: &args::GlobalArgs) -> Result<()> {
        bail!("Emptying is not supported on MacOS");
    }

    #[cfg(not(target_os = "macos"))]
    pub fn run(&self) -> Result<()> {
        let items = trash::os_limited::list()?;
        let empty = if self.force { empty } else { empty_with_prompt };
        if self.all {
            empty(items)?;
            return Ok(());
        }
        let filters = self.filter_args.to_filters()?;
        if filters.is_empty() {
            // TODO: better error message
            bail!("Must match something");
        }
        let items = items
            .into_iter()
            .filter(|item| filters.is_match(item))
            .collect();
        empty(items)?;
        Ok(())
    }
}

fn empty_with_prompt(items: Vec<TrashItem>) -> Result<()> {
    println!("{} items will be emptied from the trash", items.len());
    if Confirm::new().with_prompt("Are you sure?").interact()? {
        empty(items)?;
    }
    Ok(())
}

fn empty(items: impl IntoIterator<Item = TrashItem>) -> Result<()> {
    trash::os_limited::purge_all(items)?;
    Ok(())
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
// type DynIterator<'a> = &'a mut dyn Iterator<Item = TrashEntry>;

// pub fn empty(opt: Opt) -> Result<()> {
//     // must have these variables in the outer scope because
//     // returning a reference requires referent to outlive it
//     let mut regex_iter;
//     let mut days_iter;
//     let mut start_iter = read_dir_trash_entries()?;

//     let mut iter: DynIterator<'_> = if let Some(days) = opt.days {
//         regex_iter = start_iter.filter(move |tentry| {
//             let res = filter_by_days(days.clone(), tentry);
//             ok_log!(res => error!).unwrap_or(false)
//         });
//         &mut regex_iter
//     } else {
//         &mut start_iter
//     };

//     let regex = if let Some(ref re) = opt.regex {
//         Some(Regex::new(re).wrap_err(format!("Failed to create regex from string `{}`", re))?)
//     } else {
//         None
//     };

//     let second_iter: DynIterator<'_> = if let Some(ref re) = regex {
//         days_iter = iter
//             .map(Pair::new)
//             .filter_map(|res| ok_log!(res => error!))
//             .filter(move |Pair(_trash_entry, ref trash_info)| {
//                 let res = filter_by_regex(&re, trash_info);
//                 ok_log!(res => error!).unwrap_or(false)
//             })
//             .map(Pair::revert);
//         &mut days_iter
//     } else {
//         &mut iter
//     };

//     second_iter
//         .map(|trash_entry| trash_entry.remove())
//         .filter_map(|res| ok_log!(res => error!))
//         .for_each(|_| ());

//     Ok(())
// }

// fn filter_by_days(days: u64, trash_entry: &TrashEntry) -> Result<bool> {
//     // the limit where the deleting will stop
//     let limit = Local::today()
//         .checked_sub_signed(Duration::days(days as i64))
//         .ok_or_else(|| eyre!("Overflow when subtracting {} from a date", days))?
//         .naive_local();
//     let trash_info = trash_entry.parse_trash_info()?;
//     // get the deletion date of the trash_info struct
//     let deletion_date = trash_info.deletion_date().date();

//     // check if the deletion return if the deletion date is less than the limit, if it is, delete
//     Ok(deletion_date < limit)
// }

// fn filter_by_regex(regex: &Regex, trash_info: &TrashInfo) -> Result<bool> {
//     let percent_path = trash_info.percent_path().decoded()?;
//     Ok(regex.is_match(percent_path.as_ref()))
// }
