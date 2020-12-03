//! Only compiled when the feature `readline` is enabled

use std::result::Result as StdResult;
use std::{fmt, str::FromStr};

use eyre::Result;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::print;
use crate::ExitCode;

pub struct ReadLine(Editor<()>);

impl ReadLine {
    pub fn new() -> ReadLine {
        ReadLine(Editor::new())
    }

    pub fn read(&mut self, prompt: &str) -> StdResult<String, ReadlineError> {
        self.0.readline(prompt)
    }

    /// will exit if the user has clicked Ctrl-C or Ctrl-D or typed exit. If the user has given
    /// something that the parser function will return Err, the loop start over and asks the user
    /// to type again.
    pub fn read_parse_loop<T, E>(&mut self, prompt: &str) -> Result<T>
    where
        T: FromStr<Err = E>,
        E: fmt::Display,
    {
        loop {
            match self.read(prompt) {
                Ok(ref cmd) if cmd == "exit" => ExitCode::Success.exit(),
                Ok(line) => match line.parse() {
                    Ok(t) => break Ok(t),
                    Err(e) => print::err_display(e),
                },
                Err(ReadlineError::Interrupted) => {
                    ExitCode::Interrupted.exit_with_msg("Ctrl-C");
                }
                Err(ReadlineError::Eof) => ExitCode::Success.exit_with_msg("exited"),
                Err(e) => return Err(e.into()),
            }
        }
    }
}
