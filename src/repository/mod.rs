pub mod config;
pub use config::Config;

use crate::{Directories, Error};
use git2::{self, build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use regex::Regex;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[allow(unused)]
pub struct Repository {
    repository: git2::Repository,
    pub config: Config,
    pub path: PathBuf,
}

impl Repository {
    /// ## Possible Inputs
    ///
    /// * `<username>/<repository>`  
    ///
    /// * `git@<ssh>:<username>/<repository>`  
    ///
    /// * `<protocol>://<website>/<username>/<repository>`  
    pub fn clone<P: AsRef<Path>>(url: &str, dest: P, overwrite: bool) -> Result<Self, Error> {
        // Adapted from `https://regexpattern.com/git-repository`
        // Fields: ([protocol   secure?   website]   OR   [git])   username   repository
        let regex = Regex::new(&format!(
            r"^(?:(?:{}|{})(?::(?://{}/)?))?{}/{}(?:\.git)?$",
            r"(?P<protocol>git|http(?P<secure>s)?)",
            r"(?P<ssh>git@[\w\d\.:]+)",
            r"(?P<website>[\w\d\.\[\]:_-]+?)",
            r"(?P<username>[\w\d-]+)",
            r"(?P<repository>[\w\d\._-]+?)",
        ))?;

        if !regex.is_match(url) {
            return Err(Error::InvalidRepository);
        }

        let captures = regex.captures(url).unwrap();

        let user = captures.name("username").unwrap().as_str();
        let repo = captures.name("repository").unwrap().as_str();

        let dest = dest.as_ref().join(repo);

        if dest.exists() {
            if overwrite {
                fs::remove_dir_all(&dest)?;
            } else {
                return Err(Error::AlreadyExists(dest));
            }
        }

        let make_repository = |repository| -> Result<Self, Error> {
            let config = Config::open(&dest)?;

            Ok(Self {
                repository,
                path: dest.clone(),
                config,
            })
        };

        if captures.name("ssh").is_some() || captures.name("website").is_some() {
            git2::Repository::clone(url, &dest).and_then(|r| Ok(make_repository(r)))?
        } else {
            let url = format!("git@github.com:{user}/{repo}");
            let key = get_ssh_key()?;

            let mut callbacks = RemoteCallbacks::new();
            let mut fetch_options = FetchOptions::new();
            let mut builder = RepoBuilder::new();

            callbacks.credentials(|_, username, _| {
                Cred::ssh_key(username.unwrap(), None, Path::new(&key), None)
            });

            fetch_options.remote_callbacks(callbacks);
            builder.fetch_options(fetch_options);

            builder
                .clone(&url, &dest)
                .and_then(|r| Ok(make_repository(r)))?
        }
    }

    pub fn init(name: &str, dest: &Directories) -> Result<Self, Error> {
        let path = dest.join(name);

        if path.exists() {
            return Err(Error::AlreadyExists(path));
        }

        let repository = git2::Repository::init(&path)?;

        let config = Config::new(name, dest)?;

        Ok(Repository {
            repository,
            config,
            path,
        })
    }

    pub fn open<P: AsRef<Path> + Copy>(path: P) -> Result<Self, Error> {
        let repository = git2::Repository::open(path)?;

        let config = Config::open(path)?;

        Ok(Self {
            repository,
            config,
            path: path.as_ref().into(),
        })
    }

    pub fn delete(self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }
}

impl std::fmt::Debug for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CupRepository")
            .field("config", &self.config)
            .field("path", &self.path)
            .finish()
    }
}

fn get_ssh_key() -> Result<String, Error> {
    let ssh_dir = Directories::Home.join(".ssh");

    let mut public_keys: Vec<_> = fs::read_dir(&ssh_dir)?
        .filter_map(|entry| Some(entry.ok()?.file_name().into_string().unwrap()))
        .filter(|file| file.ends_with(".pub"))
        .collect();

    public_keys.sort();

    if public_keys.is_empty() {
        Err(Error::SshKeyNotFound)
    } else {
        Ok(ssh_dir
            .join(&public_keys[0])
            .to_str()
            .unwrap()
            .strip_suffix(".pub")
            .unwrap()
            .into())
    }
}
