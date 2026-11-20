use std::fs;
use std::path::Path;

use lscolors::{LsColors, Style};
use once_cell::sync::Lazy;
use trash::TrashItem;

static LS_COLORS: Lazy<LsColors> = Lazy::new(|| LsColors::from_env().unwrap_or_default());

pub mod path {
    use super::*;

    pub fn display(path: &Path) -> String {
        path.as_os_str().to_string_lossy().to_string()
    }

    pub fn style_for<'a>(path: &Path, metadata: &'a fs::Metadata) -> Option<&'a Style> {
        LS_COLORS.style_for_path_with_metadata(path, Some(metadata))
    }
}

pub fn clone_trash_item(item: &TrashItem) -> TrashItem {
    TrashItem {
        id: item.id.clone(),
        name: item.name.clone(),
        original_parent: item.original_parent.clone(),
        time_deleted: item.time_deleted,
    }
}

pub fn swap<T, U>((t, u): (T, U)) -> (U, T) {
    (u, t)
}
