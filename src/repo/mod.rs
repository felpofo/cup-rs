pub mod config;
pub use config::Config;

use self::config::File;
use crate::{
    error_and_exit,
    prompt::{Prompt, Question},
    Directories,
};
use git2::{self, build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks};
use regex::Regex;
use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct Repository {
    repository: git2::Repository,
    pub config: Config,
    pub path: PathBuf,
}

impl Repository {
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
                    "Oh. It looks like you are trying to clone a repository from ssh, in this case type where is your ssh key",
                    &key
                ).prompt();

                if answer != key {
                    if answer.starts_with('~') {
                        answer = answer.replacen('~', &Directories::Home.to_string(), 1);
                    }

                    answer = answer.replace("//", "/");
                }

                Cred::ssh_key(
                    username_from_url.unwrap(),
                    None,
                    &Directories::Home.path().join(answer),
                    None,
                )
            });

            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            let mut builder = RepoBuilder::new();
            builder.fetch_options(fetch_options);

            match builder.clone(&url, &dest) {
                Ok(repository) => {
                    if !dest.exists() {
                        error_and_exit("Error: This is not a valid repository");
                    }

                    let config = Config {
                        id: Uuid::new_v4().to_string(),
                        name: repo.into(),
                        files: serde_yaml::from_str::<Vec<config::File>>(
                            &fs::read_to_string(&dest).unwrap(),
                        )
                        .unwrap(),
                    };

                    Self {
                        repository,
                        path: dest,
                        config,
                    }
                }
                Err(err) => error_and_exit(err.message()),
            }
        } else {
            let url = captures
                .name("website")
                .is_some()
                .then_some(url.to_string())
                .unwrap_or(format!("https://github.com/{user}/{repo}"));

            match git2::Repository::clone(&url, &dest) {
                Ok(repository) => {
                    if !dest.exists() {
                        error_and_exit("Error: This is not a valid repository");
                    }

                    let config = Config {
                        id: Uuid::new_v4().to_string(),
                        name: repo.into(),
                        files: serde_yaml::from_str::<Vec<config::File>>(
                            &fs::read_to_string(&dest).unwrap(),
                        )
                        .unwrap(),
                    };

                    Self {
                        repository,
                        path: dest,
                        config,
                    }
                }
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
    pub fn delete(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }

    /// Check if a repository has an ok `.yml` file
    fn check(&self) -> bool {
        let config = crate::Config::open(&self.config.name);

        if let Err(_) = config {
            return false;
        }

        let config = config.unwrap();

        for file in &config.files {
            let path = self.files_dir().join(&file.0);

            if !path.exists() {
                return false;
            }
        }

        true
    }

    /// Create an empty cup export on user data directory
    pub fn init(name: &str) -> Self {
        let path = Directories::Data.path().join(name);

        if path.exists() {
            panic!("Folder already exists");
        }

        let repository = git2::Repository::init(&path);
        let config = crate::Config::new(name);

        match repository {
            Ok(repository) => Repository {
                repository,
                config,
                path,
            },
            Err(err) => error_and_exit(&format!(
                "Error creating repository '{}': {}",
                name,
                err.message()
            )),
        }
    }

    pub fn open(name: &str) -> Self {
        let path = Directories::Data.path().join(name);

        let config = Config::open(name).unwrap();

        let repository = git2::Repository::open(&path)
            .expect(&format!("Error opening repository on '{:?}'", path));

        Self {
            repository,
            config,
            path,
        }
    }

    pub fn add_files(&mut self, files: &mut Vec<File>) -> Result<(), Box<dyn Error>> {
        for file in files {
            if self
                .config
                .files
                .iter()
                .position(|saved| file == saved)
                .is_some()
            {
                continue;
            }

            match file.1.as_str() {
                "/" => {
                    let dest = self.files_dir().join(&file.0);
                    let file = PathBuf::from("/").join(&file.0);

                    println!("copying {:?} into {:?}", file, dest);

                    fs::create_dir_all(&dest.parent().unwrap())?;
                    fs::copy(file, &dest)?;
                }
                "~" => {
                    let dest = self.files_dir().join("home").join(&file.0);
                    let file = Directories::Home.path().join(&file.0);

                    println!("copying {:?} into {:?}", file, dest);

                    fs::create_dir_all(&dest.parent().unwrap())?;
                    fs::copy(file, &dest)?;
                }
                _ => unreachable!(),
            };

            self.config.files.push(file.clone());
        }

        self.config.save()
    }

    pub fn remove_files(&mut self, files: &Vec<File>) -> Result<(), Box<dyn Error>> {
        for file in files {
            self.config
                .files
                .iter()
                .position(|saved| file == saved)
                .and_then(|index| {
                    Some({
                        let dir = self.files_dir();

                        match file.1.as_str() {
                            "/" => fs::remove_file(dir.join(&file.0)).ok()?,
                            "~" => fs::remove_file(dir.join("home").join(&file.0)).ok()?,
                            _ => unreachable!(),
                        };

                        self.config.files.remove(index);
                    })
                });
        }

        self.config.save()?;

        remove_empty_dir_all(&self.files_dir(), &self.files_dir())?;

        Ok(())
    }

    fn files_dir(&self) -> PathBuf {
        self.path.join("files")
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

fn remove_empty_dir_all(dir: &Path, top: &Path) -> Result<(), Box<dyn Error>> {
    let entries: Vec<_> = fs::read_dir(dir)?.collect();

    for entry in &entries {
        let path = entry.as_ref().unwrap().path();

        if path.is_dir() {
            remove_empty_dir_all(&path, top)?;
        }
    }

    if dir != top {
        if entries.is_empty() {
            fs::remove_dir(dir)?;
            remove_empty_dir_all(dir.parent().unwrap(), top)?;
        }
    }

    Ok(())
}
