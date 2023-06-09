mod directories;
mod error;
mod path;
mod prompt;

pub mod commands;
pub mod repository;

use directories::Directories;
pub use error::Error;
pub use path::expand::Expand;
pub use repository::{Config, Repository};

pub fn warn(msg: &str) {
    use termion::{
        color::{self, Fg, Yellow},
        style::{self, Bold},
    };

    eprintln!(
        "{}{Bold}WARNING{}{}: {msg}",
        Fg(Yellow),
        Fg(color::Reset),
        style::Reset
    );
}
