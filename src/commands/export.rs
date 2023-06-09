use super::Command;
use crate::{
    directories::Directories,
    repository::config::File,
    Expand, Repository,
};
use clap::{arg, command, ArgAction, ArgMatches};
use dialoguer::MultiSelect;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug)]
pub struct Export;

impl Command for Export {
    fn run(matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("NAME").unwrap();

        match matches.subcommand() {
            Some(("add", submatches)) => Self::add(name, matches, submatches),
            Some(("remove", submatches)) => Self::remove(name, matches, submatches),
            //TODO Some(("list", submatches)) => Self::list(name, matches, submatches),
            //TODO Some(("delete", submatches)) => Self::delete(name, matches, submatches),
            _ => Self::create(name),
        }
    }
}

impl Export {
    fn create(name: &str) -> Result<()> {
        let dest = Directories::Data;

        Repository::init(name, &dest)?;

        Ok(())
    }

    fn add(name: &str, _matches: &ArgMatches, submatches: &ArgMatches) -> Result<()> {
        let path = Directories::Data.join(name);
        let mut repository = Repository::open(&path)?;

        let mut files: Vec<_> = submatches
            .get_many::<String>("FILES")
            .unwrap()
            .map(PathBuf::from)
            .filter_map(|p| p.expand().ok())
            .flatten()
            .map(File::from)
            .collect();

        repository.config.append(&mut files);
        repository.config.save()?;

        Ok(())
    }

    fn remove(name: &str, _matches: &ArgMatches, submatches: &ArgMatches) -> Result<()> {
        let path = Directories::Data.join(name);
        let mut repository = Repository::open(&path)?;

        let interactive = *submatches.get_one::<bool>("interactive").unwrap();

        let mut files: Vec<File> = match interactive {
            true => {
                let options: Vec<String> = repository
                    .config
                    .files
                    .iter()
                    .map(File::to_user_str)
                    .collect();

                MultiSelect::new()
                    .items(&options)
                    .interact()?
                    .iter()
                    .map(|&i| &options[i])
                    .filter_map(|s| File::try_from(s).ok())
                    .collect()
            }
            false => submatches
                .get_many::<String>("FILES")
                .unwrap()
                .map(PathBuf::from)
                .filter_map(|path| path.expand().ok())
                .flatten()
                .map(File::from)
                .collect(),
        };

        repository.config.remove(&mut files);
        repository.config.save()?;

        Ok(())
    }
}

#[allow(clippy::from_over_into)]
impl Into<clap::Command> for Export {
    fn into(self) -> clap::Command {
        command!("export")
            .about("Save your dotfiles")
            .arg_required_else_help(true)
            .arg(arg!(<NAME> "Export name"))
            .subcommands([
                command!("add")
                    .about("Add file(s)")
                    .arg_required_else_help(true)
                    .arg(arg!(<FILES> ... "Files you want to add")),
                command!("remove")
                    .about("Remove file(s)")
                    .arg_required_else_help(true)
                    .arg(arg!([FILES] ... "Files you want to remove"))
                    .arg(arg!(-i --interactive).action(ArgAction::SetTrue)),
            ])
    }
}
