use eyre::Result;
use lazy_static::lazy_static;
use log::{info, error};
use regex::Regex;
use structopt::StructOpt;

use crate::utils::map_trash_entry_keep;
use trash_lib::ok_log;
use trash_lib::trash_entry::read_dir_trash_entries;
use trash_lib::trash_info::TrashInfo;

#[derive(StructOpt, Debug)]
pub struct Opt {
    pattern: String,
}

pub fn remove(opt: Opt) -> Result<()> {
    read_dir_trash_entries()?
        .map(map_trash_entry_keep)
        .filter_map(|res| ok_log!(res => error!))
        .filter(|(trash_entry, trash_info)| {
            let res = try_filter_by_regex(&opt.pattern, trash_info);
            match ok_log!(res => error!) {
                Some(t) => t,
                None => false,
            }
        })
        .inspect(|(trash_entry, _)| info!("Removing {:?}", trash_entry))
        // .map(|(trash_entry, _trash_info)| trash_entry.remove())
        // .filter_map(|res| ok_log!(res => error!))
        .for_each(|_| ());

    Ok(())
}

fn try_filter_by_regex(pattern: &str, trash_info: &TrashInfo) -> Result<bool> {
    let re = Regex::new(pattern).unwrap();
    let percent_path = trash_info.percent_path().decoded()?;
    Ok(re.is_match(percent_path.as_ref()))
}
