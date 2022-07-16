use clap::Clap;
use eyre::{Result, WrapErr};
use log::{error, info};
use regex::Regex;

use crate::utils::map_trash_entry_keep;
use trash_lib::ok_log;
use trash_lib::trash_entry::read_dir_trash_entries;
use trash_lib::trash_info::TrashInfo;

#[derive(Clap, Debug)]
pub struct Opt {
    /// remove files that match this pattern
    pattern: String,
}

pub fn remove(opt: Opt) -> Result<()> {
    let re = Regex::new(&opt.pattern).wrap_err(format!(
        "Failed to create regex from string `{}`",
        &opt.pattern
    ))?;

    read_dir_trash_entries()?
        .map(map_trash_entry_keep)
        .filter_map(|res| ok_log!(res => error!))
        .filter(|(_trash_entry, trash_info)| {
            let res = filter_by_regex(&re, trash_info);
            ok_log!(res => error!).unwrap_or(false)
        })
        .inspect(|(trash_entry, _)| info!("Removing {:?}", trash_entry))
        .map(|(trash_entry, _trash_info)| trash_entry.remove())
        .filter_map(|res| ok_log!(res => error!))
        .for_each(|_| ());

    Ok(())
}

fn filter_by_regex(regex: &Regex, trash_info: &TrashInfo) -> Result<bool> {
    let percent_path = trash_info.percent_path().decoded()?;
    Ok(regex.is_match(percent_path.as_ref()))
}
