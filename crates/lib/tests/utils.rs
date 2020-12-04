use eyre::Result;
use xshell::{cmd, cwd};

pub fn setup_tmp() -> Result<()> {
    let crate_dir = cwd()?;
    let tests_dir = crate_dir.join("tests");
    let tmp_dir = tests_dir.join("tmp");
    let data_dir = tests_dir.join("data");

    cmd!("cp -r {data_dir} {tmp_dir}").run()?;

    Ok(())
}
