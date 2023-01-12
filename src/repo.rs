use crate::dirs::get_data_dir;
use git2::Repository;
use regex::Regex;
use std::fs;
use std::io::{self, Error, ErrorKind};

/// Clone a repository from github from short syntax, i mean`<username>/<repo>
/// By now its hardcoded for github site only
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

/// Delete a repository from filesystem
pub fn remove(repo: &Repository) -> io::Result<()> {
    fs::remove_dir_all(repo.workdir().unwrap())
}

/// Check if the repository has the `rust-dotfiles.json` file
pub fn check(repo: &Repository) -> io::Result<bool> {
    match fs::read(repo.workdir().unwrap().join("rust-dotfiles.json")) {
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

#[cfg(test)]
mod tests {
    use crate::repo;
    use std::fs;

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

    #[test]
    fn check_repo() {
        let repo = repo::clone("felpofo/testrepo").expect("Error cloning repo 'felpofo/testrepo'");
        assert!(repo.path().exists(), "Repo directory does not exist");

        let is_valid = repo::check(&repo).expect("Error checking repo");
        assert!(is_valid, "should be valid");

        fs::remove_file(repo.workdir().unwrap().join("rust-dotfiles.json"))
            .expect("Error removing program's file");

        let is_valid = repo::check(&repo).expect("Error checking repo");
        assert!(!is_valid, "should be invalid");

        repo::remove(&repo).expect("Failed to remove repo");
    }
}
