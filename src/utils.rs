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
