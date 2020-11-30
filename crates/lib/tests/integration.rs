mod utils;
mod macros;

use std::path::PathBuf;

use eyre::{Result, bail};

#[test]
fn setup_tmp_test() -> Result<()> {
    utils::setup_tmp()?;
    Ok(())
}

pub fn put_test(path: PathBuf) -> Result<()> {
    use trash_lib::put;

    put(&[path])?;

    Ok(())
}
