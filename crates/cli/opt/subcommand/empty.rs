use chrono::{Duration, Local};
use clap::Clap;
use eyre::{eyre, Result, WrapErr};
use log::error;

use regex::Regex;
use trash_lib::trash_entry::{read_dir_trash_entries, TrashEntry};
use trash_lib::{ok_log, trash_info::TrashInfo};

use crate::utils::Pair;

#[derive(Clap, Debug, PartialEq)]
pub struct Opt {
    /// keep stray files (not valid trash entries)
    #[clap(short = 's', long)]
    keep_strays: bool,

    /// delete files older than amount of days, can be used with regex
    #[clap(short, long)]
    days: Option<u64>,

    /// delete files matching regex, can be used with days
    #[clap(short, long)]
    regex: Option<String>,

    /// deletes ALL files from the trash
    #[clap(long, conflicts_with_all = &["days", "regex"])]
    all: bool,

    /// skips asking for confirmation before deleting all files
    #[clap(long)]
    noconfirm: bool,
}

type DynIterator<'a> = &'a mut dyn Iterator<Item = TrashEntry>;

pub fn empty(opt: Opt) -> Result<()> {
    // must have these variables in the outer scope because
    // returning a reference requires referent to outlive it
    let mut regex_iter;
    let mut days_iter;
    let mut start_iter = read_dir_trash_entries()?;

    let mut iter: DynIterator<'_> = if let Some(days) = opt.days {
        regex_iter = start_iter.filter(move |tentry| {
            let res = filter_by_days(days.clone(), tentry);
            ok_log!(res => error!).unwrap_or(false)
        });
        &mut regex_iter
    } else {
        &mut start_iter
    };

    let regex = if let Some(ref re) = opt.regex {
        Some(Regex::new(re).wrap_err(format!("Failed to create regex from string `{}`", re))?)
    } else {
        None
    };

    let second_iter: DynIterator<'_> = if let Some(ref re) = regex {
        days_iter = iter
            .map(Pair::new)
            .filter_map(|res| ok_log!(res => error!))
            .filter(move |Pair(_trash_entry, ref trash_info)| {
                let res = filter_by_regex(&re, trash_info);
                ok_log!(res => error!).unwrap_or(false)
            })
            .map(Pair::revert);
        &mut days_iter
    } else {
        &mut iter
    };

    second_iter
        .map(|trash_entry| trash_entry.remove())
        .filter_map(|res| ok_log!(res => error!))
        .for_each(|_| ());

    Ok(())
}

fn filter_by_days(days: u64, trash_entry: &TrashEntry) -> Result<bool> {
    // the limit where the deleting will stop
    let limit = Local::today()
        .checked_sub_signed(Duration::days(days as i64))
        .ok_or_else(|| eyre!("Overflow when subtracting {} from a date", days))?
        .naive_local();
    let trash_info = trash_entry.parse_trash_info()?;
    // get the deletion date of the trash_info struct
    let deletion_date = trash_info.deletion_date().date();

    // check if the deletion return if the deletion date is less than the limit, if it is, delete
    Ok(deletion_date < limit)
}

fn filter_by_regex(regex: &Regex, trash_info: &TrashInfo) -> Result<bool> {
    let percent_path = trash_info.percent_path().decoded()?;
    Ok(regex.is_match(percent_path.as_ref()))
}
