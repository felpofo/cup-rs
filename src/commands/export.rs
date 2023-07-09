use super::Command;
use crate::{dirs::Dirs, repository::config::File, Expand, Repository};
use anyhow::Result;
use clap::{arg, command, ArgMatches};
use dialoguer::{Confirm, MultiSelect};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct Export;

impl Command for Export {
    fn run(matches: &ArgMatches) -> Result<()> {
        let name = matches.get_one::<String>("NAME").unwrap();

        match matches.subcommand() {
            Some(("add", submatches)) => Self::add(name, matches, submatches),
            Some(("remove", submatches)) => Self::remove(name, matches, submatches),
            Some(("delete", submatches)) => Self::delete(name, matches, submatches),
            Some(("create", _)) => Self::create(name),
            _ => Ok(()),
        }
    }
}

impl Export {
    fn create(name: &str) -> Result<()> {
        let dest = Dirs::Data;

        Repository::init(name, &dest)?;

        Ok(())
    }

    fn add(name: &str, _matches: &ArgMatches, submatches: &ArgMatches) -> Result<()> {
        let path = Dirs::Data.join(name);
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
        let path = Dirs::Data.join(name);
        let mut repository = Repository::open(&path)?;

        let interactive = *submatches.get_one::<bool>("interactive").unwrap();

        let mut files: Vec<File> = match interactive {
            true => {
                let options: Vec<String> = repository
                    .config
                    .files
                    .iter()
                    .map(File::user_path)
                    .collect();

                if options.is_empty() {
                    println!("There are no files to remove");
                    return Ok(());
                }

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

    fn delete(name: &str, _matches: &ArgMatches, submatches: &ArgMatches) -> Result<()> {
        let path = Dirs::Data.join(name);

        let has_user_confirmation = *submatches.get_one::<bool>("yes").unwrap();

        if !has_user_confirmation
            || !Confirm::new()
                .with_prompt(format!("Do you really want to delete '{name}'?"))
                .interact()?
        {
            return Err(anyhow::Error::msg("Operation aborted"));
        }

        fs::remove_dir_all(&path)?;
        println!("Deleted '{}'", path.display());

        Ok(())
    }
}

#[allow(clippy::from_over_into)]
impl Into<clap::Command> for Export {
    fn into(self) -> clap::Command {
        command!("export")
            .about("Save your dotfiles")
            .arg(arg!(<NAME> "Export name"))
            .subcommands([
                command!("add")
                    .about("Add file(s)")
                    .arg(arg!(<FILES> ... "Files you want to add")),
                command!("remove")
                    .about("Remove file(s)")
                    .arg_required_else_help(true)
                    .args([
                        arg!([FILES] ... "Files you want to remove"),
                        arg!(-i --interactive "Choose the files you want to remove on the fly"),
                    ]),
                command!("create").about("Create a new export"),
                command!("delete")
                    .about("Deletes a export")
                    .arg(arg!(-y --yes "Don't ask confirmation")),
            ])
    }
}
