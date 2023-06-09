use super::Command;
use crate::{directories::Directories, Error, Repository};
use clap::{arg, command, ArgAction, ArgMatches};
use std::fs;

#[derive(Debug)]
pub struct Import;

impl Command for Import {
    fn run(matches: &ArgMatches) -> Result<(), Error> {
        let url = matches.get_one::<String>("URL").unwrap();

        let overwrite = *matches.get_one::<bool>("overwrite").unwrap();

        match matches.subcommand() {
            _ if overwrite => Self::import(url, true),
            _ => Self::import(url, false),
        }
    }
}

impl Import {
    fn import(url: &str, overwrite: bool) -> Result<(), Error> {
        let dest = Directories::Data.path();

        let repository = Repository::clone(url, dest)?;

        for file in &repository.config.files {
            let from = Directories::Files(&repository.config).join(file.to_string());
            let to = file.path();

            if !overwrite && to.exists() {
                let name = &to.file_name().unwrap().to_str().unwrap();
                let old = format!("{}.old", name);
                let old = &to.parent().unwrap().join(old);

                fs::copy(&to, &old)?;
            }

            fs::copy(&from, &to)?;
        }

        Ok(())
    }
}

#[allow(clippy::from_over_into)]
impl Into<clap::Command> for Import {
    fn into(self) -> clap::Command {
        command!("import")
            .about("Import dotfiles")
            .arg_required_else_help(true)
            .arg(arg!(<URL> "Repo url"))
            .arg(arg!(-o --overwrite).action(ArgAction::SetTrue))
    }
}
