use crate::dirs::get_data_dir;
use git2::Repository;
use regex::Regex;
use std::fs;
use std::io::{self, Error, ErrorKind};

pub fn clone_repo(str: &str) -> io::Result<Repository> {
    let str_regex = Regex::new(r"^(\w+)/(\w+)$").unwrap();

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

    let dest = get_data_dir();
    if dest.is_err() { return Err(dest.unwrap_err()); }
    let dest = dest.unwrap();

    if dest.exists() {
        fs::remove_dir_all(&dest).expect(&format!("Error deleting dir {}", dest.to_str().unwrap()));
    }

    match Repository::clone(&url, dest.join(repo)) {
        Ok(r) => Ok(r),
        Err(err) => Err(Error::new(ErrorKind::Other, err.message())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_repo() {
        let repo = clone_repo("felpofo/testrepo").expect("Error cloning repo 'felpofo/testrepo'");

        println!("{}", repo
            .path()
            .to_str()
            .unwrap());

        assert!(repo
            .path()
            .to_str()
            .unwrap()
            .ends_with("rust-dotfiles/testrepo/.git/"));

        fs::remove_dir_all(repo.path()).expect(&format!("Error deleting dir {}", repo.path().to_str().unwrap()));
    }
}
