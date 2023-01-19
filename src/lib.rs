mod prompt;

mod dirs;
mod repo;

use dirs::Directories;
pub use repo::CupRepository;

fn error_and_exit(error_message: &str) -> ! {
    use termion::{
        color::{self, Fg, Red},
        style::{self, Bold},
    };

    eprintln!(
        "{Bold}{}{}{}{}",
        Fg(Red),
        error_message,
        style::Reset,
        Fg(color::Reset)
    );

    std::process::exit(1);
}
