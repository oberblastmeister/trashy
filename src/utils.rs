use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use log::{debug, error, info, warn};
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    Utf8 { path: PathBuf },

    ReadDir { source: io::Error, path: PathBuf },

    ReadDirEntry { source: io::Error, path: PathBuf }
}

type Result<T, E = Error> = ::std::result::Result<T, E>;

pub fn convert_paths(paths: &[impl AsRef<Path>]) -> Vec<&str> {
    let paths = paths
        .into_iter()
        .map(|p| {
            let path = p.as_ref();
            path.to_str().context(Utf8 { path })
        })
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        })
        .filter_map(Result::ok)
        .collect();
    paths
}

/// finds a name that doesn't conflict with names already in the trash directory and names in
/// other
fn find_name(path: &str, existing: &[&str]) -> String {
    (1..)
        .map(|n| format!("{}_{}", path, n))
        .find(|new_path| !existing.contains(&&**new_path))
        .expect("BUG: path must be found, iterator is infinite")
}

pub fn find_names_multiple<'a>(paths: &[&'a str], mut existing: Vec<&'a str>) -> Vec<&'a str> {
    let new_name_start = existing.len();
    let mut result = Vec::with_capacity(paths.len());

    for (idx, &path) in paths.into_iter().enumerate() {
        existing.push(path);
        let item = find_name(&existing[new_name_start + idx], &existing);
        result.push(item)
    }
    existing.drain(..new_name_start);

    existing
}

pub fn read_dir_path<'a>(dir: &'a Path) -> Result<impl Iterator<Item = PathBuf> + 'a> {
    let paths = fs::read_dir(dir)
        .context(ReadDir { path: dir })?
        // context of dir_entry errors
        .map(move |dent_res| dent_res.context(ReadDirEntry { path: dir }))
        // log dir_entry errors
        .inspect(|res| {
            if let Some(e) = res.as_ref().err() {
                warn!("{}", e);
            }
        })
        // filter out errors
        .filter_map(Result::ok)
        // convert dir_entry to string
        .map(|d| d.path());

    Ok(paths)
}

pub fn convert_to_string(path: &Path) -> Result<String> {
    Ok(convert_to_str(path)?.to_string())
}

pub fn convert_to_str(path: &Path) -> Result<&str> {
    let s = path.to_str().context(Utf8 { path })?;
    Ok(s)
}
