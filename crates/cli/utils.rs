use std::cmp::Ordering;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use chrono::naive::NaiveDateTime;
use eyre::{eyre, Result, WrapErr};
use lscolors::{LsColors, Style};
use once_cell::sync::Lazy;
use prettytable::{cell, row, Cell, Row};
use trash_lib::trash_entry::TrashEntry;
use trash_lib::trash_info::TrashInfo;
use trash_lib::HOME_DIR;

static LS_COLORS: Lazy<LsColors> = Lazy::new(|| LsColors::from_env().unwrap_or_default());

#[derive(Debug, PartialEq, Eq)]
pub struct Pair(pub TrashEntry, pub TrashInfo);

impl Pair {
    pub fn new(trash_entry: TrashEntry) -> Result<Pair> {
        let trash_info = trash_entry.parse_trash_info()?;
        let pair = Pair(trash_entry, trash_info);
        Ok(pair)
    }

    pub fn revert(self) -> TrashEntry {
        self.0
    }
}

impl PartialOrd for Pair {
    fn partial_cmp(&self, other: &Pair) -> Option<Ordering> {
        Some(self.1.cmp(&other.1))
    }
}

impl Ord for Pair {
    fn cmp(&self, other: &Pair) -> Ordering {
        self.1.cmp(&other.1)
    }
}

pub fn map_trash_entry_keep(trash_entry: TrashEntry) -> Result<(TrashEntry, TrashInfo)> {
    let trash_info = trash_entry.parse_trash_info()?;
    Ok((trash_entry, trash_info))
}

pub fn get_metadata(trash_entry: &TrashEntry) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(trash_entry.file_path())?;
    Ok(metadata)
}

// pub fn map_to_row(pair: (TrashEntry, TrashInfo)) -> Result<Row> {
//     let (trash_entry, trash_info) = pair;
//     let metadata = get_metadata(&trash_entry)?;
//     let path = trash_info.percent_path().decoded()?;
//     let colorized_path = colorize_path(path.as_ref(), &metadata);
//     let mut res = format_date(trash_info.deletion_date());
//     res.push(Cell::new(&colorized_path));
//     Ok(Row::new(res))
// }

pub fn colorize_path(path: &str, metadata: &fs::Metadata) -> String {
    let style = LS_COLORS.style_for_path_with_metadata(path, Some(metadata));
    let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
    format!("{}", ansi_style.paint(path))
}

pub fn title_row() -> Row {
    row!["Year", "Month", "Day", "Time", "Path"]
}

pub fn format_date(date: NaiveDateTime) -> Vec<Cell> {
    let year = format!("{}", date.format("%y"));
    let month = format!("{}", date.format("%b"));
    let day = format!("{}", date.format("%d"));
    let time = format!("{}", date.format("%H:%M:%S"));
    vec![
        Cell::new(&year),
        Cell::new(&month),
        Cell::new(&day),
        Cell::new(&time),
    ]
}

pub fn format_date_compact(date: NaiveDateTime) -> Vec<Cell> {
    let mm_dd = format!("{}", date.format("%m/%d"));
    let time = format!("{}", date.format("%H:%M:%S"));
    vec![Cell::new(&mm_dd), Cell::new(&time)]
}

pub fn sort_iterator<T>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T>
where
    T: Ord,
{
    let mut v: Vec<_> = iter.collect();
    v.sort_unstable();
    v.into_iter()
}

pub fn input(msg: &str) -> Result<String> {
    let mut s = String::new();
    print!("{}", msg);
    stdout()
        .flush()
        .context("Failed to flush stdout to allow input")?;
    stdin()
        .read_line(&mut s)
        .context("Failed to get input from user")?;
    Ok(s)
}

pub fn shorten_path<'a, T>(path: T) -> Result<String>
where
    T: AsRef<Path> + 'a,
{
    let path = path.as_ref();
    let path_str = path.to_str().ok_or_else(|| eyre!("Failed"))?;
    let home_dir = HOME_DIR.to_string_lossy();

    Ok(match path_str.find(&*home_dir) {
        Some(start_idx) if start_idx == 0 => {
            format!("{}{}", "~", &path_str[home_dir.len()..])
        }
        _ => path.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn shorten_path_test() {
        assert_eq!(
            shorten_path(&format!("{}/project/brian", HOME_DIR.to_str().unwrap())).unwrap(),
            Cow::from("~/project/brian")
        );
    }

    #[test]
    fn short_path_not_beginning_test() {
        assert_eq!(
            shorten_path(&format!(
                "{}/project/{}/code",
                HOME_DIR.to_str().unwrap(),
                HOME_DIR.to_str().unwrap()
            )).unwrap(),
            format!("~/project/{}/code", HOME_DIR.to_str().unwrap())
        );
    }

    #[test]
    fn shorten_path_none_test() {
        let path = &format!("projects/{}/code", HOME_DIR.to_str().unwrap());
        assert_eq!(shorten_path(path).unwrap(), Cow::from(path));
    }
}
