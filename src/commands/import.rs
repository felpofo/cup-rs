use super::Command;
use crate::{dirs::Dirs, Repository};
use anyhow::Result;
use clap::{arg, command, ArgMatches};
use std::fs;

#[derive(Debug)]
pub struct Import;

impl Command for Import {
    fn run(matches: &ArgMatches) -> Result<()> {
        let url = matches.get_one::<String>("URL").unwrap();

        let overwrite = *matches.get_one::<bool>("overwrite").unwrap();
        let quiet = *matches.get_one::<bool>("quiet").unwrap();

        match matches.subcommand() {
            _ => Self::import(url, overwrite, quiet),
        }
    }
}

impl Import {
    fn import(url: &str, overwrite: bool, quiet: bool) -> Result<()> {
        let dest = Dirs::Data.path();

        let repository = Repository::clone(url, dest)?;

        for file in &repository.config.files {
            let name = file.name();
            let from = Dirs::Files(&repository.config).join(file.to_string());
            let to = file.stored_path();

            if !overwrite && to.exists() {
                let old = &to.parent().unwrap().join(format!("{name}.bcup"));

                match fs::copy(&to, old) {
                    Ok(_) => if !quiet { println!("Backed up existent '{name}'") },
                    Err(err) => return Err(err.into()),
                };
            }

            match fs::copy(&from, &to) {
                Ok(_) => if !quiet { println!("Imported '{name}'") },
                Err(err) => return Err(err.into()),
            };
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
            .args([
                arg!(<URL> "Repo url"),
                arg!(-o --overwrite "Ignores if some file already exists"),
                arg!(-q --quiet "Do not output any information"),
            ])
    }
}
