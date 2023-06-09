use clap::ArgMatches;
use anyhow::Result;

mod export;
mod import;
mod list;

pub use export::Export;
pub use import::Import;
pub use list::List;

pub trait Command {
    fn run(matches: &ArgMatches) -> Result<()>;
}
