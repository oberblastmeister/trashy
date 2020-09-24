use std::result::Result as StdResult;

use trash_lib::trash_entry::{self, TrashEntry};
use trash_lib::trash_info::TrashInfo;
use eyre::{WrapErr, Result};

pub fn trash_entry_error_context(res: StdResult<TrashEntry, trash_entry::Error>) -> Result<TrashEntry> {
    res.wrap_err("Failed to create trash entry")
}

pub fn map_trash_entry_keep(trash_entry: TrashEntry) -> Result<(TrashEntry, TrashInfo)> {
    let trash_info = trash_entry.parse_trash_info()?;
    Ok((trash_entry, trash_info))
}

