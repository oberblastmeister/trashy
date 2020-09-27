use prettytable::{cell, Cell, Row, Table, row};
use eyre::{eyre, Result};
use terminal_size::{terminal_size, Width};

use crate::border::Border;
use crate::utils::{colorize_path, get_metadata, format_date, Pair};

pub struct SizedTable {
    size: TableSize,
    table: Table
}

impl SizedTable {
    pub fn new(border: Border) -> Result<Self> {
        let size: TableSize = get_terminal_width()?.into();
        let table = size.create_table(border);
        let sized_table = SizedTable {
            size,
            table
        };
        Ok(sized_table)
    }

    pub fn add_row(&mut self, pair: &Pair) -> Result<()> {
        let Pair(trash_entry, trash_info) = pair;
        let metadata = get_metadata(&trash_entry)?;
        let path = trash_info.percent_path().decoded()?;
        let colorized_path = colorize_path(path.as_ref(), &metadata);
        let row = match self.size {
            TableSize::Minimal => {
                row![colorized_path]
            }
            TableSize::Compact => {
                let mut res = format_date(trash_info.deletion_date());
                res.push(Cell::new(&colorized_path));
                Row::new(res)
            }
            TableSize::Full => {
                let mut res = format_date(trash_info.deletion_date());
                res.push(Cell::new(&colorized_path));
                Row::new(res)
            }
        };
        self.table.add_row(row);
        Ok(())
    }

    pub fn print(&self) -> usize {
        self.table.printstd()
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
            TableSize::Compact => row!["Date", "Path"],
            TableSize::Full => row!["Year", "Month", "Day", "Time", "Path"]    
        }
    }

    fn create_table(self, border: Border) -> Table {
        let mut table = Table::new();
        table.set_titles(self.get_title_row());
        table.set_format(border.into());
        table
    }
}
