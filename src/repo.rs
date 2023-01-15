use crate::dirs::Directories;
use git2::Repository;
use regex::Regex;
use std::fs;
use std::io::{Error, ErrorKind, Result};

/// Clone a repository from github from short syntax, i mean`<username>/<repo>
/// By now its hardcoded for github site only
pub fn clone(str: &str) -> Result<Repository> {
    let str_regex = Regex::new(r"^(.+)/(.+)$").unwrap();

    if !str_regex.is_match(str) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Bad github repo. Good repo example: <username>/<repo>",
        ));
    }

    let captures = str_regex.captures(str).unwrap();
    let user = captures.get(1).unwrap().as_str();
    let repo = captures.get(2).unwrap().as_str();

    let url = format!("https://github.com/{user}/{repo}");
    let dest = Directories::Data.path().join(repo);

    if dest.exists() {
        fs::remove_dir_all(&dest).expect(&format!("Error deleting dir {:?}", dest));
    }

    match Repository::clone(&url, dest.join(repo)) {
        Ok(clone) => Ok(clone),
        Err(err) => Err(Error::new(ErrorKind::Other, err.message())),
    }
}

/// Delete a repository from filesystem
pub fn remove(repo: &Repository) -> Result<()> {
    fs::remove_dir_all(repo.workdir().unwrap())
}

/// Check if the repository has the `yada.json` file
pub fn check(repo: &Repository) -> Result<bool> {
    match fs::read(repo.workdir().unwrap().join("yada.json")) {
        Ok(buffer) => {
            let content = String::from_utf8(buffer).unwrap();
            // For now, this only checks if file exists
            // TODO: check if file content is valid
            // TODO: check if all files mentioned by this file exists
            Ok(true)
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Ok(false),
            _ => Err(err),
        },
    }
}
