use std::path::Path;
use std::{fs, vec};

use chrono::naive::NaiveDateTime;
// use eyre::{eyre, Result};
use lscolors::{LsColors, Style};
use once_cell::sync::Lazy;
use trash::TrashItem;
// use once_cell::sync::Lazy;
// use prettytable::Cell;
// use trash_lib::trash_entry::TrashEntry;
// use trash_lib::trash_info::TrashInfo;
// use trash_lib::HOME_DIR;

static LS_COLORS: Lazy<LsColors> = Lazy::new(|| LsColors::from_env().unwrap_or_default());

// #[derive(Debug, PartialEq, Eq)]
// pub struct Pair(pub TrashEntry, pub TrashInfo);

// impl Pair {
//     pub fn new(trash_entry: TrashEntry) -> Result<Pair> {
//         let trash_info = trash_entry.parse_trash_info()?;
//         let pair = Pair(trash_entry, trash_info);
//         Ok(pair)
//     }

//     pub fn revert(self) -> TrashEntry {
//         self.0
//     }
// }

// impl PartialOrd for Pair {
//     fn partial_cmp(&self, other: &Pair) -> Option<Ordering> {
//         Some(self.1.cmp(&other.1))
//     }
// }

// impl Ord for Pair {
//     fn cmp(&self, other: &Pair) -> Ordering {
//         self.1.cmp(&other.1)
//     }
// }

// pub fn get_metadata(trash_entry: &TrashEntry) -> Result<fs::Metadata> {
//     let metadata = fs::symlink_metadata(trash_entry.file_path())?;
//     Ok(metadata)
// }
pub mod date {
    use chrono::{DateTime, Local};

    use super::*;


    // pub fn format_compact(date: NaiveDateTime) -> Vec<Cell> {
    //     let mm_dd = format!("{}", date.format("%m/%d"));
    //     let time = format!("{}", date.format("%H:%M:%S"));
    //     vec![Cell::new(&mm_dd), Cell::new(&time)]
    // }
}

pub mod path {
    use super::*;

    pub fn display(path: &Path) -> String {
        path.as_os_str().to_string_lossy().to_string()
    }

    pub fn display_colored(path: &Path) -> anyhow::Result<String> {
        let meta = fs::metadata(&path)?;
        let style = LS_COLORS.style_for_path_with_metadata(&path, Some(&meta));
        let ansi_style = style.map(Style::to_ansi_term_style).unwrap_or_default();
        let path_string = path.as_os_str().to_string_lossy();
        Ok(format!("{}", ansi_style.paint(path_string)))
    }

    pub fn style_for<'a>(path: &Path, metadata: &'a fs::Metadata) -> Option<&'a Style> {
        LS_COLORS.style_for_path_with_metadata(&path, Some(metadata))
    }

    // pub fn shorten<'a, T>(path: T) -> Result<String>
    // where
    //     T: AsRef<Path> + 'a,
    // {
    //     let path = path.as_ref();
    //     let path_str = path.to_str().ok_or_else(|| eyre!("Failed"))?;
    //     let home_dir = HOME_DIR.to_string_lossy();

    //     Ok(match path_str.find(&*home_dir) {
    //         Some(start_idx) if start_idx == 0 => {
    //             format!("{}{}", "~", &path_str[home_dir.len()..])
    //         }
    //         _ => path.to_string_lossy().into_owned(),
    //     })
    // }

    #[cfg(test)]
    mod tests {

        //     #[test]
        //     fn shorten_path_test() {
        //         assert_eq!(
        //             path::shorten(&format!("{}/project/brian", HOME_DIR.to_str().unwrap())).unwrap(),
        //             Cow::from("~/project/brian")
        //         );
        //     }

        //     #[test]
        //     fn short_path_not_beginning_test() {
        //         assert_eq!(
        //             path::shorten(&format!(
        //                 "{}/project/{}/code",
        //                 HOME_DIR.to_str().unwrap(),
        //                 HOME_DIR.to_str().unwrap()
        //             ))
        //             .unwrap(),
        //             format!("~/project/{}/code", HOME_DIR.to_str().unwrap())
        //         );
        //     }

        //     #[test]
        //     fn shorten_path_none_test() {
        //         let path = &format!("projects/{}/code", HOME_DIR.to_str().unwrap());
        //         assert_eq!(path::shorten(path).unwrap(), Cow::from(path));
        //     }
    }
}
