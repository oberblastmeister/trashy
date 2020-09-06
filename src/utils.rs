use std::borrow::Cow;
use std::fs;
use std::io;
use std::iter;
use std::path::{Path, PathBuf};

use log::{debug, error, info, warn};
use snafu::{OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    Utf8 { path: PathBuf },

    ReadDir { source: io::Error, path: PathBuf },

    ReadDirEntry { source: io::Error, path: PathBuf },
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
// fn find_name(path: &str, existing: &[impl AsRef<str>]) -> String {
//     let existing: Vec<&str> = existing.into_iter().map(|s| s.as_ref()).collect();
//     (0..)
//         .map(|n| {
//             if n == 0 {
//                 String::from(path)
//             } else {
//                 format!("{}_{}", path, n)
//             }
//         })
//         .find(|new_path| !existing.contains(&&**new_path))
//         .expect("BUG: path must be found, iterator is infinite")
// }

pub fn find_name<'a>(path: &'a str, existing: &[impl AsRef<str>]) -> Cow<'a, str> {
    let existing: Vec<&str> = existing.into_iter().map(|s| s.as_ref()).collect();
    (0..)
        .map(|n| {
            if n == 0 {
                Cow::Borrowed(path)
            } else {
                Cow::Owned(format!("{}_{}", path, n))
            }
        })
        .find(|new_path| !existing.contains(&&**new_path))
        .expect("BUG: path must be found, iterator is infinite")
}

// pub fn find_names_multiple_concat<'a>(paths: &[&'a str], existing: Vec<&'a str>) -> Vec<String> {
//     let mut results: Vec<String> = Vec::with_capacity(paths.len());

//     for &path in paths.into_iter() {
//         let results_str = results.iter().map(|s| &**s).collect::<Vec<&str>>();
//         let existing_and_results = [existing.as_slice(), results_str.as_slice()].concat();
//         let new_name = find_name(path, &existing_and_results);

//         results.push(new_name)
//     }
//     results
// }

pub fn find_names_multiple<'a>(paths: &[&'a str], existing: Vec<String>) -> Vec<Cow<'a, str>> {
    let mut existing: Vec<_> = existing.into_iter().map(|s| Cow::from(s)).collect();
    let new_name_start = existing.len();

    for path in paths.into_iter() {
        let new_name = find_name(path, &existing);
        existing.push(new_name);
    }
    existing.drain(..new_name_start);
    existing
}

// pub fn find_names_multiple<'a>(paths: &[&'a str], existing: Vec<String>) -> Vec<Cow<'a, str>> {
//     let mut results: Vec<_> = Vec::with_capacity(paths.len());
//     let mut existing: Vec<_> = existing.into_iter().map(|s| Cow::from(s)).collect();

//     for path in paths.into_iter() {
//         let new_name = find_name(path, &existing);
//         existing.push(new_name.clone());

//         results.push(new_name)
//     }
//     results
// }

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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! string_vec {
        ($($x:expr),*) => {
            vec![$(String::from($x)),*]
        }
    }

    #[test]
    fn find_names_test() {
        assert_eq!(find_name("vim.log", &["vim.log", "vim.log2"]), "vim.log_1");
    }

    #[test]
    fn find_names_test_2_test() {
        assert_eq!(find_name("vim.log", &["vim.log", "vim.log_1"]), "vim.log_2");
    }

    #[test]
    fn find_names_test_none_test() {
        assert_eq!(find_name("vim.log", &[""]), "vim.log");
    }

    #[test]
    fn find_names_multiple_test() {
        assert_eq!(
            find_names_multiple(
                &["vim.log", "dude.txt", "vim.log"],
                string_vec!["vim.log", "vim.log_1"]
            ),
            vec!["vim.log_2", "dude.txt", "vim.log_3"]
        );
    }

    #[test]
    fn find_names_multiple2_test() {
        assert_eq!(
            find_names_multiple(
                &["vim.log", "vim.log", "vim.log_2", "vim.log"],
                string_vec!["vim.log", "vim.log_1", "vim.log_3"]
            ),
            vec!["vim.log_2", "vim.log_4", "vim.log_2_1", "vim.log_5"]
        );
    }

    #[test]
    fn find_names_multiple_same_naming_test() {
        assert_eq!(
            find_names_multiple(
                &["vim.log_1", "vim.log_2", "vim.log_3"],
                string_vec!["vim.log_1", "vim.log_2", "vim.log_3"]
            ),
            vec!["vim.log_1_1", "vim.log_2_1", "vim.log_3_1"]
        );
    }
}
