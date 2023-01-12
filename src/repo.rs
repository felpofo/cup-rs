use crate::dirs::get_data_dir;
use git2::Repository;
use regex::Regex;
use std::fs;
use std::io::{self, Error, ErrorKind};

pub fn clone(str: &str) -> io::Result<Repository> {
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

    if dest.is_err() {
        return Err(dest.unwrap_err());
    }
    let dest = dest.unwrap();

    if dest.exists() {
        fs::remove_dir_all(&dest).expect(&format!("Error deleting dir {}", dest.to_str().unwrap()));
    }

    match Repository::clone(&url, dest.join(repo)) {
        Ok(r) => Ok(r),
        Err(err) => Err(Error::new(ErrorKind::Other, err.message())),
    }
}

pub fn remove(repo: &Repository) -> io::Result<()> {
    fs::remove_dir_all(repo.workdir().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::repo;

    #[test]
    fn clone_and_delete_repo() {
        let repo = repo::clone("felpofo/testrepo").expect("Error cloning repo 'felpofo/testrepo'");

        assert!(repo.path().exists(), "Repo directory does not exist");

        repo::remove(&repo).expect(&format!(
            "Error deleting dir {}",
            repo.path().to_str().unwrap()
        ));

        assert!(!repo.path().exists(), "Repo directory exists");
    }
}
