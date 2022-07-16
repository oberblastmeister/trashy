//! Only compiled when the feature `readline` is not enabled

pub fn input(msg: &str) -> Result<String> {
    let mut s = String::new();
    print!("{}", msg);
    stdout()
        .flush()
        .context("Failed to flush stdout to allow input")?;
    stdin()
        .read_line(&mut s)
        .context("Failed to get input from user")?;
    Ok(s)
}

pub fn read_parse_loop<T, E>(prompt: &str) -> Result<T>
where
    T: FromStr<Err = E>,
    E: fmt::Display,
{
    loop {
        let s = input(prompt)?;
        match s.parse() {
            Ok(t) => break Ok(t),
            Err(e) => print::err_display(e),
        }
    }
}
