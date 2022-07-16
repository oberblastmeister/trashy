use std::{
    borrow::Cow,
    cmp, env, fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use clap::Parser;

use anyhow::{bail, Result};
use comfy_table as table;
use trash::TrashItem;

use crate::{args, utils};

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        short,
        long,
        value_parser = |s: &str| -> Result<PathBuf> {
            let p = PathBuf::from(s);
            args::ensure_exists(&p)?;
            args::ensure_is_dir(&p)?;
            Ok(p)
        },
    )]
    pub path: Option<PathBuf>,

    /// Display all trashed files.
    ///
    /// By default trashed files in the current directory will be displayed.
    #[clap(short, long, conflicts_with = "path")]
    pub all: bool,
}

impl Args {
    pub fn run(&self, global_args: &args::GlobalArgs) -> Result<()> {
        if cfg!(target_os = "macos") {
            bail!("Listing is not supported on MacOS");
        }
        let path: Cow<_> = match &self.path {
            _ if self.all => Path::new("").into(),
            None => Path::new("").into(),
            Some(path) => path.into(),
        };
        let items = {
            let items = trash::os_limited::list()?;
            let mut items = if self.all {
                items
            } else {
                items
                    .into_iter()
                    .filter(|item| item.original_path().starts_with(&path))
                    .collect()
            };
            items.sort_by_key(|item| cmp::Reverse(item.time_deleted));
            items
        };
        let mut failed = 0;
        let mut table = new_table();
        table.set_header(["Date", "Path"]);
        items
            .into_iter()
            .filter_map(
                |item| match display_item(&item, &path, global_args.color_status.color()) {
                    Ok(s) => Some(s),
                    Err(_) => {
                        failed += 1;
                        None
                    }
                },
            )
            .for_each(|row_iter| {
                table.add_row(row_iter);
            });
        println!("{table}");
        Ok(())
    }
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
    base: &Path,
    color: bool,
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
    format!(
        "{} {}, {}",
        datetime.format("%B"),
        datetime.format("%d"),
        datetime.format("%H:%M")
    )
}

pub fn files_path_from_info_path(info_path: &Path) -> PathBuf {
    let file_name = Path::new(info_path.file_name().unwrap()).with_extension("");
    let mut files_path =
        (|| Some(info_path.parent()?.parent()?.join("files")))().expect("Invalid info_path");
    files_path.push(file_name);
    files_path
}
