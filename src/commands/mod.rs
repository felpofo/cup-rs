use clap::ArgMatches;
use anyhow::Result;

mod export;
mod import;

pub use export::Export;
pub use import::Import;

pub trait Command {
    fn run(matches: &ArgMatches) -> Result<()>;
}
