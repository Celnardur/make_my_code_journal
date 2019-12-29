pub mod diffs;
pub use diffs::JournalDiff;
pub mod config;
pub use config::Config;
pub mod entry;
pub use entry::Entry;

use std::error;
use std::fmt;
use std::string::String;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error { message: String::from(message) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Journal Error: {}", self.message)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

