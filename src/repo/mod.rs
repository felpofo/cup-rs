mod config;

use config::CupConfig;

use crate::{
    error_and_exit,
    prompt::{Prompt, Question},
    Directories,
};

use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

use clap::crate_name;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use regex::Regex;

pub struct CupRepository {
    repository: Repository,
    path: PathBuf,
}

impl CupRepository {
    /// Download a repository
    ///
    /// Overwrite if already exists on filesystem (Default path: `~/.local/share/cup`)
    ///
    /// # Panics
    ///
    /// This function will panic if the supplied string does not match any of the options below:
    ///
    /// `<username>/<repository>`
    ///
    /// `git@<ssh>:<username>/<repository>`
    ///
    /// `<protocol>://<website>/<username>/<repository>`
    pub fn clone(url: &str) -> Self {
        // Adapted from `https://regexpattern.com/git-repository`
        // Fields: ([protocol   secure?   website]   OR   [git])   username   repository
        let regex = Regex::new(&format!(
            r"^(?:(?:{}|{})(?::(?://{}/)?))?{}/{}(?:\.git)?$",
            r"(?P<protocol>git|ssh|http(?P<secure>s)?)",
            r"(?P<ssh>git@[\w\d\.:]+)",
            r"(?P<website>[\w\d\.\[\]:_-]+?)",
            r"(?P<username>[\w\d-]+)",
            r"(?P<repository>[\w\d\._-]+?)",
        ))
        .unwrap();

        if !regex.is_match(url) {
            error_and_exit("Invalid Repository");
        }

        let captures = regex.captures(url).unwrap();

        let user = captures.name("username").unwrap().as_str();
        let repo = captures.name("repository").unwrap().as_str();

        let dest = Directories::Data.path().join(repo);

        if dest.exists() {
            if let Err(err) = fs::remove_dir_all(&dest) {
                eprintln!("{}", err.to_string());
                error_and_exit(&format!("Error deleting dir: {:?}", &dest));
            }
        }

        let repository = if let Some(_) = captures.name("ssh") {
            let mut callbacks = RemoteCallbacks::new();

            callbacks.credentials(|_, username_from_url, _| {
                let key = {
                    let ssh_dir = Directories::Home.path().join(".ssh");

                    match fs::read_dir(&ssh_dir).ok() {
                        Some(dir_content) => {
                            let mut keys: Vec<String> = dir_content
                                .filter_map(|e| e.ok().unwrap().file_name().into_string().ok())
                                .filter(|n| n.starts_with("id") && !n.ends_with(".pub"))
                                .collect();

                            keys.sort();

                            if keys.is_empty() {
                                String::new()
                            } else {
                                ssh_dir.join(&keys[0])
                                .to_str()
                                .unwrap()
                                .into()
                            }
                        }
                        None => String::new(),
                    }
                };

                let mut answer = Question::new(
                    "Oh. It looks like you are trying to clone a repository from ssh, in this case type where is your ssh key"
                ).default(&key).prompt();

                if answer != key {
                    if answer.starts_with('~') {
                        answer = answer.replacen('~', &Directories::Home.to_string(), 1);
                    }

                    answer = answer.replace("//", "/");
                }

                Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    Path::new(&format!("{}/{answer}", Directories::Home)),
                    None,
                )
            });

            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            let mut builder = RepoBuilder::new();
            builder.fetch_options(fetch_options);

            match builder.clone(&url, &dest) {
                Ok(repository) => Self {
                    repository,
                    path: dest,
                },
                Err(err) => error_and_exit(err.message()),
            }
        } else {
            let url = captures
                .name("website")
                .is_some()
                .then_some(url.to_string())
                .unwrap_or(format!("https://github.com/{user}/{repo}"));

            match Repository::clone(&url, &dest) {
                Ok(repository) => Self {
                    repository,
                    path: dest,
                },
                Err(err) => error_and_exit(err.message()),
            }
        };

        if !repository.check() {
            eprintln!("Error: repository does not contain dotfiles from this program");
            if let Err(err) = repository.delete() {
                eprintln!("{}", err.to_string());
                error_and_exit(&format!("Error deleting dir: {:?}", &repository.path));
            }
        }

        repository
    }

    /// Delete a repository from filesystem
    pub fn delete(&self) -> Result<()> {
        fs::remove_dir_all(&self.path)
    }

    /// Check if a repository has an ok `.yml` file
    pub fn check(&self) -> bool {
        let config_path = self.path.join(format!("{}.yml", crate_name!()));

        if !config_path.exists() {
            return false;
        }

        let config = match fs::read_to_string(&config_path) {
            Ok(content) => {
                let config = serde_yaml::from_str::<CupConfig>(&content);

                if let Err(err) = config {
                    error_and_exit(&format!(
                        "Error parsing file `{:?}`: {}",
                        &config_path,
                        err.to_string()
                    ))
                };

                config.unwrap()
            }
            Err(err) => error_and_exit(&format!(
                "Error reading file `{:?}`: {}",
                &config_path,
                err.to_string()
            )),
        };

        for file in &config.files {
            let from = self.files_dir().join(&file.from);

            if !from.exists() {
                error_and_exit(&format!("Error: file `{}` does not exist", from.display()))
            }
        }

        true
    }

    fn files_dir(&self) -> PathBuf {
        self.path.join("files")
    }
}
