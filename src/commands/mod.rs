mod export;
mod import;

use std::error::Error;

use clap::ArgMatches;
pub use export::Export;
pub use import::Import;

pub trait Command {
    fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>>;
}
