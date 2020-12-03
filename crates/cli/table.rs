use std::borrow::Cow;

use eyre::{eyre, Result};
use log::debug;
use log::info;
use log::trace;
use prettytable::{cell, row, Cell, Row, Table};
use terminal_size::{terminal_size, Width};

use crate::border::Border;
use crate::utils::{
    colorize_path, format_date, format_date_compact, get_metadata, shorten_path, Pair,
};

pub struct SizedTable {
    size: TableSize,
    table: Table,
}

impl SizedTable {
    pub fn new(border: Border) -> Result<Self> {
        let size: TableSize = get_terminal_width()?.into();
        info!("The table size is: {:?}", size);
        let table = create_table(size.get_title_row(), border);
        let sized_table = SizedTable { size, table };
        Ok(sized_table)
    }

    pub fn add_row(&mut self, pair: &Pair, dont_colorize: bool, dont_shorten: bool) -> Result<()> {
        let row = self.get_row(pair, dont_colorize, dont_shorten)?;
        self.table.add_row(row);
        Ok(())
    }

    fn get_row(&self, pair: &Pair, dont_colorize: bool, dont_shorten: bool) -> Result<Row> {
        let Pair(ref trash_entry, ref trash_info) = pair;

        let path = trash_info.percent_path();
        trace!("Path before decoded: {}", path);
        let path = path.decoded()?;
        trace!("After decoded path: {}", path);

        let mut displayed_path = if !dont_shorten {
            let path = path.as_ref();
            Cow::from(shorten_path(path).unwrap())
        } else {
            path
        };
        trace!("After displayed path (shorten stuff): {}", displayed_path);

        displayed_path = if !dont_colorize {
            let metadata = get_metadata(&trash_entry)?;
            Cow::from(colorize_path(displayed_path.as_ref(), &metadata))
        } else {
            displayed_path
        };
        trace!("After colorized path: {}", displayed_path);

        trace!("Add adding size {:?} row", self.size);
        let row = match self.size {
            TableSize::Minimal => row![displayed_path],
            TableSize::Compact => {
                let mut res = format_date_compact(trash_info.deletion_date());
                res.push(Cell::new(&displayed_path));
                Row::new(res)
            }
            TableSize::Full => {
                let mut res = format_date(trash_info.deletion_date());
                res.push(Cell::new(&displayed_path));
                Row::new(res)
            }
        };
        Ok(row)
    }

    pub fn print(&self) -> usize {
        self.table.printstd()
    }
}

pub struct IndexedTable(SizedTable);

impl IndexedTable {
    pub fn new(border: Border) -> Result<Self> {
        let size: TableSize = get_terminal_width()?.into();
        let table = create_table(size.get_title_row_index(), border);
        Ok(IndexedTable(SizedTable { size, table }))
    }

    pub fn add_row(&mut self, pair: &Pair, dont_colorize: bool, dont_shorten: bool) -> Result<()> {
        let mut row = self.0.get_row(pair, dont_colorize, dont_shorten)?;
        // insert the index
        let index = self.0.table.len() + 1;
        debug!("current index (1 based): {}", index);
        row.insert_cell(0, cell!(index));
        self.0.table.add_row(row);
        Ok(())
    }

    pub fn print(&self) -> usize {
        self.0.print()
    }
}

fn get_terminal_width() -> Result<Width> {
    let width = terminal_size()
        .ok_or_else(|| eyre!("Unable to get terminal size"))?
        .0;
    Ok(width)
}

#[derive(Debug, Copy, Clone)]
pub enum TableSize {
    /// only displays path
    Minimal,

    /// displays all informations in compact size
    Compact,

    /// displays all information fully
    Full,
}

impl From<Width> for TableSize {
    fn from(value: Width) -> TableSize {
        let Width(w) = value;
        match w {
            0..=45 => TableSize::Minimal,
            46..=90 => TableSize::Compact,
            _ => TableSize::Full,
        }
    }
}

impl TableSize {
    fn try_new() -> Result<TableSize> {
        Ok(get_terminal_width()?.into())
    }

    fn get_title_row(self) -> Row {
        match self {
            TableSize::Minimal => row!["Path"],
            TableSize::Compact => row!["Date", "Time", "Path"],
            TableSize::Full => row!["Month", "Day", "Time", "Path"],
        }
    }

    fn get_title_row_index(self) -> Row {
        let mut row = self.get_title_row();
        row.insert_cell(0, Cell::new("Index"));
        row
    }

    fn create_table(self, border: Border) -> Table {
        let mut table = Table::new();
        table.set_titles(self.get_title_row());
        table.set_format(border.into());
        table
    }
}

fn create_table(title_row: Row, border: Border) -> Table {
    let mut table = Table::new();
    table.set_titles(title_row);
    table.set_format(border.into());
    table
}
