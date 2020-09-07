pub mod trash;
mod utils;
pub mod trash_info;
pub mod parser;

use lazy_static::lazy_static;
use directories::UserDirs;
use std::path::{Path, PathBuf};
use fs_extra::dir;
use fs_extra::file;

lazy_static! {
    static ref USER_DIRS: UserDirs = UserDirs::new().expect("Failed to determine user directories");
    static ref HOME_DIR: &'static Path = USER_DIRS.home_dir();
    static ref FILE_COPY_OPT: file::CopyOptions = file::CopyOptions::new();
    static ref DIR_COPY_OPT: dir::CopyOptions = dir::CopyOptions::new();
    pub static ref TRASH_DIR: PathBuf = HOME_DIR.join(".local/share/Trash");
    pub static ref TRASH_INFO_DIR: PathBuf = TRASH_DIR.join("info");
    pub static ref TRASH_FILE_DIR: PathBuf = TRASH_DIR.join("files");
}
