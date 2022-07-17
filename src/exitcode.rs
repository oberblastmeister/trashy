use std::fmt;
use std::process;

use crate::print;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExitCode {
    Success,
    Error,
    Interrupted,
}

impl From<ExitCode> for i32 {
    fn from(value: ExitCode) -> i32 {
        match value {
            ExitCode::Success => 0,
            ExitCode::Error => 1,
            ExitCode::Interrupted => 130,
        }
    }
}

impl ExitCode {
    pub fn exit_with_msg(self, msg: impl fmt::Display) -> ! {
        self.print_msg(msg);
        self.exit();
    }

    fn print_msg(self, msg: impl fmt::Display) {
        match self {
            ExitCode::Success => {
                println!("{}", msg);
            }
            ExitCode::Error => {
                print::err_display(msg);
            }
            ExitCode::Interrupted => {
                println!("Interrupted")
            }
        }
    }

    pub fn exit(self) -> ! {
        process::exit(self.into())
    }
}
