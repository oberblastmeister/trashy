mod parser;
mod error;

pub use parser::{parse_trash_info, TRASH_DATETIME_FORMAT};
pub use error::Error;
