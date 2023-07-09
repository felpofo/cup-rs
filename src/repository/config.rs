use crate::{Dirs, Expand};
use anyhow::Result;
use clap::crate_name;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    env::current_dir,
    fs, io,
    path::{Path, PathBuf}
};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub id: String,
    pub name: String,
    pub files: Vec<File>,

    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum File {
    User(String),
    Root(String),
}

impl Config {
    pub fn new(name: &str, dest: &Dirs) -> Result<Self> {
        let path = dest.join(name).join(format!("{}.yml", crate_name!()));

        let config = Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            files: vec![],
            path: path.clone(),
        };

        let f = fs::File::options()
            .write(true)
            .create_new(true)
            .open(&path)?;

        serde_yaml::to_writer(f, &config)?;

        Ok(config)
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().join(format!("{}.yml", crate_name!()));

        let contents = fs::read_to_string(&path)?;
        let parsed = serde_yaml::from_str(&contents)?;

        let config = Self { path, ..parsed };

        Ok(config)
    }

    pub fn save(&mut self) -> Result<()> {
        let files = Dirs::Files(self).path();

        for file in self.missing_files() {
            println!("Copying: {file:?}");

            let dest = files.join(file.to_string());
            fs::create_dir_all(dest.parent().unwrap())?;
            fs::copy(file.stored_path(), &dest)?;
        }

        for file in self.lost_files() {
            println!("Removing: {file:?}");

            let path = files.join(file.to_string());

            fs::remove_file(path)?;
        }

        remove_empty_dir_all(&files)?;

        let f = fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.path)?;

        serde_yaml::to_writer(f, &self)?;

        self.commit_changes()?;

        Ok(())
    }

    fn missing_files(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter(|file| {
                let path = Dirs::Files(self).join(file.to_string());

                !path.exists()
            })
            .collect()
    }

    fn lost_files(&self) -> Vec<File> {
        let mut lost = vec![];

        let path = Dirs::Files(self).path();

        match path.expand().ok() {
            Some(found) => {
                let found: Vec<_> = found.into_iter().map(File::from).collect();

                for file in found {
                    if !self.files.iter().any(|f| *f == file) {
                        lost.push(file);
                    }
                }

                lost
            }
            None => vec![],
        }
    }

    pub fn append(&mut self, other: &mut Vec<File>) {
        for file in other {
            let repeated = self.files.iter().any(|f| f == file);

            if repeated {
                continue;
            }

            self.files.push(file.clone());
        }
    }

    pub fn remove(&mut self, other: &mut Vec<File>) {
        for file in other {
            let index = self.files.iter().position(|f| f == file);

            if let Some(index) = index {
                self.files.swap_remove(index);
            }
        }
    }

    pub fn commit_changes(&self) -> Result<()> {
        let repo_path = self.path.parent().unwrap();

        let message = format!("{}", chrono::offset::Utc::now().format("%Y-%m-%d %H:%M:%S"));

        // don't like this but works, who cares after all?
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo_path)
            .output()?;

        std::process::Command::new("git")
            .args(["commit", "-m", &message])
            .current_dir(repo_path)
            .output()?;

        Ok(())
    }
}

impl File {
    pub fn stored_path(&self) -> PathBuf {
        match &self {
            Self::Root(ref file) => Dirs::Root.join(file),
            Self::User(ref file) => Dirs::Home.join(file),
        }
    }

    pub fn user_path(&self) -> String {
        match self {
            File::User(file) => format!("~/{}", file),
            File::Root(file) => format!("/{}", file),
        }
    }

    pub fn name(&self) -> &String {
        match self {
            File::User(file) => file,
            File::Root(file) => file,
        }
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> Self {
        let value = path.display().to_string();
        let home = Dirs::Home.to_string();

        let regex = Regex::new(r".+/files/(?P<type>user|root)/(?P<file>.+)").unwrap();

        if regex.is_match(&value) {
            let captures = regex.captures(&value).unwrap();
            let file = captures.name("file").unwrap().as_str().into();

            match captures.name("type").unwrap().as_str() {
                "user" => return Self::User(file),
                "root" => return Self::Root(file),
                _ => unreachable!(),
            }
        }

        if path.starts_with(&home) {
            Self::User(value[home.len() + 1..].into())
        } else {
            Self::Root(value[1..].into())
        }
    }
}

impl TryFrom<&str> for File {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut path = PathBuf::from(value);

        if !path.is_absolute() {
            let current = current_dir()?;

            path = match &value[..2] {
                "~/" => Dirs::Home.join(&value[2..]),
                "./" => current.join(&value[2..]),
                _ => current.join(value),
            };
        }

        if path.exists() {
            Ok(Self::from(path))
        } else {
            Err(anyhow::Error::msg("Path does not exist"))
        }
    }
}

impl TryFrom<&String> for File {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}

impl ToString for File {
    fn to_string(&self) -> String {
        match self {
            File::Root(ref file) => PathBuf::from("root").join(file),
            File::User(ref file) => PathBuf::from("user").join(file),
        }
        .display()
        .to_string()
    }
}

fn remove_empty_dir_all<P: AsRef<Path>>(dir: P) -> Result<(), io::Error> {
    remove_empty_dir_all_impl(dir.as_ref(), dir.as_ref())
}

fn remove_empty_dir_all_impl(dir: &Path, top: &Path) -> Result<(), io::Error> {
    let entries: Vec<_> = fs::read_dir(dir)?.collect();

    for entry in &entries {
        let path = entry.as_ref().unwrap().path();

        if path.is_dir() {
            remove_empty_dir_all_impl(&path, top)?;
        }
    }

    if dir != top && entries.is_empty() {
        fs::remove_dir(dir)?;
        remove_empty_dir_all_impl(dir.parent().unwrap(), top)?;
    }

    Ok(())
}
