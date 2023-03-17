use clap::ArgMatches;
use crate::Error;

mod export;
mod import;

pub use export::Export;
pub use import::Import;

pub trait Command {
    fn run(matches: &ArgMatches) -> Result<(), Error>;
}
