use std::fs;
use std::path::PathBuf;

use eyre::{bail, Result};
use xshell::read_dir;
use xshell::{cmd, cwd};

pub fn setup_tmp() -> Result<()> {
    get_dir()?;

    cmd!("cp -r tests/data tests/tmp").run()?;

    Ok(())
}

pub fn get_dir() -> Result<PathBuf> {
    let cwd = cwd()?;

    if !cwd
        .to_str()
        .expect("Failed to convert path to str")
        .contains("trashy/crates/lib")
    {
        println!("{:?}", cwd);
        panic!("Did not contain correct path")
    } else {
        Ok(cwd)
    }
}
