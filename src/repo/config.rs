use crate::dirs::Directories;
use clap::crate_name;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub name: String,
    pub files: Vec<File>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct File(
    /// file path without root dirs and user home path
    ///
    /// # Examples
    ///
    /// `/home/x/file` becomes `file`
    /// `/some/folder/on/root/file` becomes `some/folder/on/root/file`
    pub String,
    /// ~ or /
    pub String,
);

impl Config {
    /// Create a blank config
    ///
    /// This function will overwrite existent config
    ///
    /// # Panics
    ///
    /// This function will panic if the file can't be created
    pub fn new(name: &str) -> Self {
        let path = Self::filepath_from(name);

        let config = Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            files: vec![],
        };

        fs::File::options()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|file| {
                serde_yaml::to_writer(file, &config).expect(&format!(
                    "Error: failed to serialize blank config to '{:?}'",
                    path
                ));

                Ok(())
            })
            .expect(&format!("Error: failed to create the file '{:?}'", path));

        config
    }

    pub fn open(name: &str) -> Result<Self, Box<dyn Error>> {
        let path = Self::filepath_from(name);

        let contents = fs::read_to_string(&path)?;
        let parsed = serde_yaml::from_str(&contents)?;

        Ok(parsed)
    }

    pub(super) fn save(&mut self) -> Result<(), Box<dyn Error>> {
        self.files.sort();
        self.files.dedup();

        let saved = Self::open(&self.name)?;

        if *self != saved {
            let path = self.filepath();

            fs::File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&path)
                .and_then(|file| {
                    serde_yaml::to_writer(file, &self).expect(&format!(
                        "Error: failed to serialize config to '{:?}'",
                        path
                    ));

                    Ok(())
                })
                .expect(&format!("Error: failed to open the file '{:?}'", path));
        }

        Ok(())
    }

    fn filepath_from(name: &str) -> PathBuf {
        let filename = format!("{}.yml", crate_name!());
        Directories::Data.path().join(name).join(filename)
    }

    fn filepath(&self) -> PathBuf {
        let filename = format!("{}.yml", crate_name!());
        Directories::Data.path().join(&self.name).join(filename)
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> Self {
        let value = path.display().to_string();

        let home = Directories::Home.to_string();

        if path.starts_with(&home) {
            Self(value[home.len() + 1..].into(), "~".into())
        } else {
            Self(value[1..].into(), "/".into())
        }
    }
}
