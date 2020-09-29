use eyre::{eyre, Result};
use log::{debug, error};
use structopt::StructOpt;

use crate::border::Border;
use crate::exitcode::ExitCode;
use crate::table::SizedTable;
use crate::utils::{sort_iterator, Pair};
use trash_lib::ok_log;
use trash_lib::trash_entry::{self, read_dir_trash_entries};

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
    debug!("creating a new sized table");
    let mut table = SizedTable::new(opt.border)?;

    let iter = iter.map(Pair::new).filter_map(|res| ok_log!(res => error!));

    let mut peekable = sort_iterator(iter)
        .map(|pair| table.add_row(&pair))
        .filter_map(|res| ok_log!(res => error!))
        .peekable();
    
    match peekable.peek() {
        Some(_) => peekable.for_each(|_| ()),
        None => ExitCode::Success.exit_with_msg("There are no trash entries to list"),
    }

    table.print();
    Ok(())
}
