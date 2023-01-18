use crate::{
    error_and_exit,
    prompt::{Prompt, Question},
    Directories,
};
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use regex::Regex;
use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

pub struct Repo {
    repository: Repository,
    path: PathBuf,
}

impl Repo {
    /// Clone a repository
    ///
    /// Overwrite if already exists
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
    pub fn clone(query: &str) -> Repo {
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

        if !regex.is_match(query) {
            error_and_exit("Invalid Repository");
        }

        let captures = regex.captures(query).unwrap();

        let user = captures.name("username").unwrap().as_str();
        let repo = captures.name("repository").unwrap().as_str();

        let dest = Directories::Data.path().join(repo);

        if dest.exists() {
            if let Err(err) = fs::remove_dir_all(&dest) {
                eprintln!("{:?}", err.kind());
                error_and_exit(&format!("Error deleting dir: {:?}", &dest));
            }
        }

        if let Some(_) = captures.name("ssh") {
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

            return match builder.clone(&query, &dest) {
                Ok(repository) => Repo {
                    repository,
                    path: dest,
                },
                Err(err) => error_and_exit(err.message()),
            };
        }

        if let Some(_) = captures.name("website") {
            return match Repository::clone(&query, &dest) {
                Ok(repository) => Repo {
                    repository,
                    path: dest,
                },
                Err(err) => error_and_exit(err.message()),
            };
        }

        let url = format!("https://github.com/{user}/{repo}");
        return match Repository::clone(&url, &dest) {
            Ok(repository) => Repo {
                repository,
                path: dest,
            },
            Err(err) => error_and_exit(err.message()),
        };
    }

    /// Delete a repository from filesystem
    pub fn delete(&self) -> Result<()> {
        fs::remove_dir_all(&self.path)
    }

    /// Check if a repository has the `yada.json` file
    fn check(url: &str) -> bool {
        unimplemented!();
        // TODO: check if file content is valid
        // TODO: check if all files mentioned by this file exists
        // TODO: make all of this before download the repo
    }
}
