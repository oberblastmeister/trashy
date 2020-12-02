use eyre::Result;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fmt;
use std::process;

pub fn input(prompt: &str) -> Result<Option<String>> {
    let mut rl = Editor::<()>::new();
    // Todo: load history

    loop {
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                // Todo, add history
                break Ok(Some(line));
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break Ok(None);
            }
            Err(ReadlineError::Eof) => {
                println!("")
            }
            e @ Err(_) => break e.map_err(Into::into).map(Option::Some),
        }
    }
}

pub fn input_parse_loop<T>(prompt: &str, parser: impl Fn(&str) -> Result<T>) -> Result<T> {
    loop {
        match input(prompt) {
            Ok(None) => process::exit(0),
            Ok(Some(inp)) => match parser(&inp) {
                Ok(t) => break Ok(t),
                Err(e) => {
                    crate::print_err_display(e);
                }
            },
            Err(e) => crate::print_err_display(e),
        }
    }
}
