use chrono::{DateTime, Duration, Local};
use eyre::{eyre, Result};
use log::error;
use structopt::StructOpt;

use trash_lib::ok_log;
use trash_lib::trash_entry::{read_dir_trash_entries, TrashEntry};
use trash_lib::trash_info::TrashInfo;

#[derive(StructOpt, Debug, PartialEq)]
pub struct Opt {
    /// keep stray files (not valid trash entries)
    #[structopt(short = "s", long = "keep-strays")]
    keep_strays: bool,

    /// delete files olders than amount of days
    days: Option<u64>,
}

pub fn empty(opt: Opt) -> Result<()> {
    match opt.days {
        Some(days) => {
            read_dir_trash_entries()?
                .filter(|trash_entry| filter_by_days(days, trash_entry))
                .map(|trash_entry| trash_entry.remove())
                .filter_map(|res| ok_log!(res => error!))
                .for_each(|_| ());
        }
        None => trash_lib::empty(opt.keep_strays)?,
    }

    Ok(())
}

fn filter_by_days(days: u64, trash_entry: &TrashEntry) -> bool {
    try_filter_by_days(days, trash_entry).unwrap_or(false)
}

fn try_filter_by_days(days: u64, trash_entry: &TrashEntry) -> Result<bool> {
    // the limit where the deleting will stop
    let limit = Local::today()
        .checked_sub_signed(Duration::days(days as i64))
        .ok_or_else(|| eyre!("Overflow when subtracting {} from a date", days))?
        .naive_local();
    let trash_info = trash_entry.parse_trash_info()?;
    // get the deletion date of the trash_info struct
    let deletion_date = trash_info.deletion_date().date();
    // check if the deletion date
    Ok(deletion_date < limit)
}
