use super::Command;
use crate::{
    dirs::Directories,
    prompt::{MultipleChoiceList, Prompt},
    repo::config::File,
    Repository,
};
use clap::{arg, command, ArgAction, ArgMatches};
use std::{env::current_dir, error::Error, fs, io, path::PathBuf};

#[derive(Debug)]
pub struct Export;

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
                    .arg(arg!(-i - -interactive).action(ArgAction::SetTrue)),
            ])
    }
}

impl Command for Export {
    fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
        let name = matches.get_one::<String>("NAME").unwrap();

        match matches.subcommand() {
            Some(("add", submatches)) => Self::add(name, matches, submatches),
            Some(("remove", submatches)) => Self::remove(name, matches, submatches),
            _ => Self::create(name),
        }
    }
}

impl Export {
    fn create(name: &str) -> Result<(), Box<dyn Error>> {
        Repository::init(name);

        Ok(())
    }

    fn add(
        name: &str,
        _matches: &ArgMatches,
        submatches: &ArgMatches,
    ) -> Result<(), Box<dyn Error>> {
        let mut repository = Repository::open(name);
        let mut files: Vec<_> = submatches
            .get_many::<String>("FILES")
            .unwrap()
            .map(PathBuf::from)
            .filter_map(resolve_path)
            .flat_map(read_files_all)
            .flatten()
            .map(File::from)
            .collect();

        repository.add_files(&mut files)
    }

    fn remove(
        name: &str,
        _matches: &ArgMatches,
        submatches: &ArgMatches,
    ) -> Result<(), Box<dyn Error>> {
        let mut repository = Repository::open(name);

        let interactive = *submatches.get_one::<bool>("interactive").unwrap();

        let files: Vec<_> = match interactive {
            true => {
                let options = repository
                    .config
                    .files
                    .iter()
                    .map(|file| match file.1.as_str() {
                        "~" => format!("{}{}", "~/", file.0),
                        "/" => format!("{}{}", "/", file.0),
                        _ => unreachable!(),
                    })
                    .map(|file| (file, false))
                    .collect();

                MultipleChoiceList::new("Select what files do you want to remove", options)
                    .prompt()
                    .into_iter()
                    .map(PathBuf::from)
                    .filter_map(resolve_path)
                    .map(File::from)
                    .collect()
            }
            false => submatches
                .get_many::<String>("FILES")
                .unwrap()
                .map(PathBuf::from)
                .filter_map(resolve_path)
                .flat_map(read_files_all)
                .flatten()
                .map(File::from)
                .collect(),
        };

        repository.remove_files(&files)
    }
}

fn resolve_path(path: PathBuf) -> Option<PathBuf> {
    let pathstr = path.display().to_string();

    if path.is_absolute() {
        return path.canonicalize().ok();
    }

    let current = current_dir().unwrap();

    let path = match &pathstr[..2] {
        "~/" => Directories::Home.path().join(&pathstr[2..]),
        "./" => current.join(&pathstr[2..]),
        _ => current.join(&pathstr),
    };

    if path.exists() {
        path.canonicalize().ok()
    } else {
        None
    }
}

fn read_files_all(dir: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut files = vec![];

    if dir.is_file() {
        files.push(dir);
    } else if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let mut inner = read_files_all(entry?.path())?;
            files.append(&mut inner);
        }
    }

    Ok(files)
}
