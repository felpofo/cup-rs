use crate::{dirs::Directories, repo::config::File, Repository};
use clap::ArgMatches;
use std::{env::current_dir, error::Error, fs, io, path::PathBuf};

#[derive(Debug)]
pub struct Export;

impl Export {
    pub fn create(name: &str) -> Result<(), Box<dyn Error>> {
        Repository::init(name);

        Ok(())
    }

    pub fn add(
        name: &str,
        matches: &ArgMatches,
        submatches: &ArgMatches,
    ) -> Result<(), Box<dyn Error>> {
        let mut repository = Repository::open(name);
        let mut files: Vec<_> = submatches
            .get_many::<String>("FILES")
            .unwrap()
            .map(PathBuf::from)
            .filter_map(resolve_path)
            .flat_map(read_dir)
            .flatten()
            .map(File::from)
            .collect();

        repository.add_files(&mut files)?;

        Ok(())
    }

    pub fn remove(
        name: &str,
        matches: &ArgMatches,
        submatches: &ArgMatches,
    ) -> Result<(), Box<dyn Error>> {
        let mut repository = Repository::open(name);
        let files: Vec<_> = submatches
            .get_many::<String>("FILES")
            .unwrap()
            .map(PathBuf::from)
            .filter_map(resolve_path)
            .flat_map(read_dir)
            .flatten()
            .map(File::from)
            .collect();

        repository.remove_files(&files)?;

        Ok(())
    }
}

fn resolve_path(path: PathBuf) -> Option<PathBuf> {
    let pathstr = path.display().to_string();

    if path.is_absolute() {
        return Some(path);
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

fn read_dir(dir: PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut files = vec![];

    if dir.is_file() {
        files.push(dir);
    } else if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let mut inner = read_dir(entry?.path())?;
            files.append(&mut inner);
        }
    }

    Ok(files)
}
