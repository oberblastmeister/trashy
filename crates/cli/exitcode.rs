use std::fmt;
use std::process;

use crate::print_err;
use crate::print_err_display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExitCode {
    Success,
    GeneralError,
}

impl From<ExitCode> for i32 {
    fn from(value: ExitCode) -> i32 {
        match value {
            ExitCode::Success => 0,
            ExitCode::GeneralError => 1,
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
            ExitCode::GeneralError => {
                print_err_display(msg);
            }
        }
    }

    pub fn exit(self) -> ! {
        match self {
            ExitCode::Success => {
                process::exit(self.into());
            }
            ExitCode::GeneralError => process::exit(self.into()),
        }
    }
}
