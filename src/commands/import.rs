use super::Command;
use crate::{directories::Directories, Error, Repository};
use clap::{arg, command, ArgAction, ArgMatches};

#[derive(Debug)]
pub struct Import;

impl Into<clap::Command> for Import {
    fn into(self) -> clap::Command {
        command!("import")
            .about("Import dotfiles")
            .arg_required_else_help(true)
            .arg(arg!(<URL> "Repo url"))
            .arg(arg!(-o --overwrite).action(ArgAction::SetTrue))
    }
}

impl Command for Import {
    fn run(matches: &ArgMatches) -> Result<(), Error> {
        let url = matches.get_one::<String>("URL").unwrap();

        let overwrite = *matches.get_one::<bool>("overwrite").unwrap();

        match matches.subcommand() {
            _ if overwrite => Self::import_overwrite(url),
            _ => Self::import(url),
        }
    }
}

impl Import {
    fn import(url: &str) -> Result<(), Error> {
        let dest = Directories::Data.path();

        Repository::clone(url, &dest, false)?;

        Ok(())
    }

    #[allow(unused_must_use)]
    fn import_overwrite(url: &str) -> Result<(), Error> {
        let dest = Directories::Data.path();

        Repository::clone(url, &dest, true)?;

        Ok(())
    }
}
