use eyre::{eyre, Result};
use itertools::Itertools;
use log::error;
use prettytable::Table;
use structopt::StructOpt;

use crate::border::Border;
use crate::utils::{map_trash_entry_keep, title_row, custom_cmp, map_to_row};
use trash_lib::trash_entry::{self, read_dir_trash_entries};
use trash_lib::ok_log;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short = "s", long = "style", default_value = "Sharp", possible_values = &Border::variants(), case_insensitive = true)]
    pub border: Border,
}

pub fn list(opt: Opt) -> Result<()> {
    let res = read_dir_trash_entries();
    let iter = match res {
        Err(ref e) => match e {
            trash_entry::Error::NotFound { .. } => return Err(eyre!("should repeat this process")),
            _ => res?,
        },
        Ok(iter) => iter,
    };
    let mut table = Table::new();
    table.set_format(opt.border.into());
    table.set_titles(title_row());

    iter.map(map_trash_entry_keep)
        .filter_map(|res| ok_log!(res => error!))
        .sorted_by(custom_cmp)
        .map(map_to_row)
        .filter_map(|res| ok_log!(res => error!))
        .for_each(|row| {
            table.add_row(row);
        });

    table.printstd();
    Ok(())
}
