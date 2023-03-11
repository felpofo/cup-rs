use crate::Repository;
use super::Command;
use clap::{arg, command, ArgMatches};
use std::error::Error;

#[derive(Debug)]
pub struct Import;

impl Into<clap::Command> for Import {
    fn into(self) -> clap::Command {
        command!("import")
            .about("Import dotfiles")
            .arg_required_else_help(true)
            .arg(arg!(<NAME> "Repo name"))
    }
}

impl Command for Import {
    fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let name = matches.get_one::<String>("NAME").unwrap();

        match matches.subcommand() {
            _ => Self::create(name),
        }
    }
}

impl Import {
    fn create(url: &str) -> Result<(), Box<dyn Error>> {
        Repository::clone(url);

        Ok(())
    }
}
