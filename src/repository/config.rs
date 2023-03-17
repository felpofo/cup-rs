use crate::{Directories, Error, Expand};
use clap::crate_name;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    env::current_dir,
    fs,
    io,
    path::{Path, PathBuf},
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
    pub fn new(name: &str, dest: &Directories) -> Result<Self, Error> {
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

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref().join(format!("{}.yml", crate_name!()));

        let contents = fs::read_to_string(&path)?;
        let parsed = serde_yaml::from_str(&contents)?;

        let config = Self { path, ..parsed };

        Ok(config)
    }

    #[allow(unused_must_use)]
    pub fn save(&mut self) -> Result<(), Error> {
        let files = Directories::Files(self).path();

        for file in self.missing_files() {
            println!("Copying: {file:?}");

            let dest = files.join(file.to_string());
            fs::create_dir_all(dest.parent().unwrap());
            fs::copy(file.real_path(), &dest);
        }

        for file in self.lost_files() {
            println!("Removing: {file:?}");

            let path = files.join(file.to_string());

            fs::remove_file(path);
        }

        remove_empty_dir_all(&files);

        let f = fs::File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.path)?;

        serde_yaml::to_writer(f, &self)?;

        Ok(())
    }

    pub fn missing_files(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter(|file| {
                let path = Directories::Files(self).join(file.to_string());

                !path.exists()
            })
            .collect()
    }

    pub fn lost_files(&self) -> Vec<File> {
        let mut lost = vec![];

        let path = Directories::Files(self).path();

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
}

impl File {
    fn real_path(&self) -> PathBuf {
        match &self {
            Self::Root(ref file) => Directories::Root.join(file),
            Self::User(ref file) => Directories::Home.join(file),
        }
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> Self {
        let value = path.display().to_string();
        let home = Directories::Home.to_string();

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
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut path = PathBuf::from(value);

        if !path.is_absolute() {
            let current = current_dir()?;

            path = match &value[..2] {
                "~/" => Directories::Home.join(&value[2..]),
                "./" => current.join(&value[2..]),
                _ => current.join(value),
            };
        }

        if path.exists() {
            Ok(Self::from(path))
        } else {
            Err(Error::Other("Path does not exist"))
        }
    }
}

impl TryFrom<String> for File {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
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
