use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::{move_file_or_dir, remove_file_or_dir};
use crate::trash_info::TrashInfo;
use crate::{TRASH_FILE_DIR, TRASH_INFO_DIR, TRASH_INFO_EXT};

/// Represents an entry in the trash directory. Includes the file path and the trash info path.
pub struct TrashEntry {
    file_path: PathBuf,
    trash_info_path: PathBuf,
}

impl TrashEntry {
    pub fn from_path(path: impl AsRef<Path>) -> TrashEntry {
        let path = path.as_ref();
        let name: &Path = if let Some(extension) = path.extension() {
            if extension == TRASH_INFO_EXT {
                path.file_stem().unwrap().as_ref()
            } else {
                panic!("Extension must be trashinfo")
            }
        } else {
            path.file_name().unwrap().as_ref()
        };

        let file_path = TRASH_FILE_DIR.join(name);
        let trash_info_path = TRASH_INFO_DIR.join(name);
        TrashEntry {
            file_path,
            trash_info_path,
        }
    }

    pub fn restore(self) {
        let original_path = &*TrashInfo::from_path(self.trash_info_path)
            .path_decoded()
            .unwrap();
        // let original_path = Path::new(original_path);
        move_file_or_dir(self.file_path, original_path).unwrap();
    }

    pub fn remove(self) {
        remove_file_or_dir(self.file_path);
        fs::remove_file(self.trash_info_path).unwrap();
    }
}
