use std::process::exit;
use termion::{
    color::{self, Fg, Red},
    style::{self, Bold},
};

mod dirs;
mod export;
mod repo;
pub mod prompt;

pub use dirs::Directories;
pub use export::Export;
pub use repo::Repo;

fn error_and_exit(error_message: &str) -> ! {
    eprintln!(
        "{Bold}{}{}{}{}",
        Fg(Red),
        error_message,
        style::Reset,
        Fg(color::Reset)
    );
    exit(1);
}
