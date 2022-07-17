use std::{
    cmp, fs, iter,
    path::{Path, PathBuf},
};

use chrono::{Local, TimeZone};
use clap::Parser;

use anyhow::{bail, Result};
use comfy_table as table;
use table::Table;
use trash::TrashItem;

use crate::{app, filter::FilterArgs, utils};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    query_args: QueryArgs,
}

impl Args {
    #[cfg(target_os = "macos")]
    pub fn run(&self, global_args: &app::GlobalArgs) -> Result<()> {
        bail!("Listing is not supported on MacOS");
    }

    #[cfg(not(target_os = "macos"))]
    pub fn run(&self, global_args: &app::GlobalArgs) -> Result<()> {
        let items = self.query_args.list(false)?;
        display_items(
            items.into_iter(),
            global_args.color_status.color(),
            Path::new(""),
        )?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct QueryArgs {
    #[clap(flatten)]
    filter_args: FilterArgs,

    #[clap(long)]
    rev: bool,

    #[clap(short)]
    n: Option<u32>,
}

impl QueryArgs {
    pub fn list(&self, nonempty: bool) -> Result<Vec<TrashItem>> {
        let filters = self.filter_args.to_filters()?;
        if nonempty && filters.is_empty() {
            bail!("Must match something");
        }
        let items = {
            let mut items = trash::os_limited::list()?;
            if !filters.is_empty() {
                items = items
                    .into_iter()
                    .filter(|item| filters.is_match(item))
                    .collect()
            };
            if self.rev {
                items.sort_by_key(|item| cmp::Reverse(item.time_deleted));
            } else {
                items.sort_by_key(|item| item.time_deleted);
            }
            items
        };
        Ok(match self.n {
            Some(n) => items.into_iter().take(n as usize).collect(),
            None => items,
        })
    }
}

pub fn display_items(
    items: impl ExactSizeIterator<Item = TrashItem>,
    color: bool,
    base: &Path,
) -> Result<()> {
    let table = items_to_table(items, color, base)?;
    println!("{table}");
    Ok(())
}

pub fn items_to_table(
    items: impl ExactSizeIterator<Item = TrashItem>,
    color: bool,
    base: &Path,
) -> Result<Table> {
    let mut failed = 0;
    let mut table = new_table();
    table.set_header(["i", "Time", "Path"]);
    {
        let vec: Vec<_> = items
            .filter_map(|item| match display_item(&item, color, base) {
                Ok(s) => Some(s),
                Err(_) => {
                    failed += 1;
                    None
                }
            })
            .collect();
        let len = vec.len();
        vec.into_iter()
            .zip((0..len).rev())
            .for_each(|(row_iter, i)| {
                table.add_row(cons_iter(table::Cell::new(i), row_iter));
            });
    }
    Ok(table)
}

fn cons_iter<T, I>(t: T, iter: I) -> impl IntoIterator<Item = T>
where
    I: IntoIterator<Item = T>,
{
    iter::once(t).chain(iter)
}

fn new_table() -> table::Table {
    use table::modifiers::*;
    use table::presets::*;

    let mut table = table::Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS);
    table
}

pub fn display_item(
    item: &TrashItem,
    color: bool,
    base: &Path,
) -> Result<impl Iterator<Item = comfy_table::Cell>> {
    use comfy_table::Cell;
    let displayed_path = utils::path::display(&item.original_path().strip_prefix(base).unwrap());
    let mut path_cell = Cell::new(displayed_path);
    if cfg!(target_os = "linux") && color {
        if let Some(style) = item_lscolors(item)? {
            path_cell = add_style_to_cell(style, path_cell);
        }
    }
    Ok([Cell::new(display_item_date(item)), path_cell].into_iter())
}

pub fn item_lscolors(item: &TrashItem) -> Result<Option<lscolors::Style>> {
    if cfg!(target_os = "linux") {
        let files_path = files_path_from_info_path(Path::new(&item.id));
        if !files_path.exists() {
            Ok(None)
        } else {
            let meta = fs::metadata(&files_path)?;
            Ok(utils::path::style_for(&item.original_path(), &meta).cloned())
        }
    } else {
        Ok(None)
    }
}

fn add_style_to_cell(style: lscolors::Style, mut path_cell: table::Cell) -> table::Cell {
    if let Some(fg) = style.foreground {
        path_cell = path_cell.fg(fg.to_crossterm_color());
    }
    if let Some(bg) = style.background {
        path_cell = path_cell.bg(bg.to_crossterm_color());
    }
    let attrs = style.font_style.to_crossterm_attributes();
    path_cell = path_cell.add_attributes(
        table::Attribute::iterator()
            .filter(|&attr| attrs.has(attr))
            .collect(),
    );
    path_cell
}

pub fn display_item_date(item: &TrashItem) -> String {
    let datetime = Local.timestamp(item.time_deleted, 0);
    let humantime = chrono_humanize::HumanTime::from(datetime);
    format!("{humantime}")
    // format!(
    //     "{} {}, {}",
    //     datetime.format("%B"),
    //     datetime.format("%d"),
    //     datetime.format("%H:%M")
    // )
}

pub fn files_path_from_info_path(info_path: &Path) -> PathBuf {
    let file_name = Path::new(info_path.file_name().unwrap()).with_extension("");
    let mut files_path =
        (|| Some(info_path.parent()?.parent()?.join("files")))().expect("Invalid info_path");
    files_path.push(file_name);
    files_path
}
