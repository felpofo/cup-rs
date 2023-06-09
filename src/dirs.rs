use crate::Config;
use clap::crate_name;
use std::{
    env, fmt,
    path::{Path, PathBuf},
};
use directories::*;

#[derive(Debug, Copy, Clone)]
pub enum Dirs<'a> {
    Home,
    Data,
    Cache,
    Root,
    Files(&'a Config),
}

impl Dirs<'_> {
    pub fn path(&self) -> PathBuf {
        let dirs = BaseDirs::new().unwrap();
        let project = ProjectDirs::from("io", "felpofo", crate_name!()).unwrap();

        match &self {
            Self::Home => dirs.home_dir().to_owned(),
            Self::Root => PathBuf::from("/"),
            Self::Data => project.data_local_dir().to_owned(),
            Self::Cache => project.cache_dir().to_owned(),
            Self::Files(config) => config.path.parent().unwrap().join("files"),
        }
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path().join(path)
    }
}

impl fmt::Display for Dirs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}

impl From<Dirs<'_>> for PathBuf {
    fn from(value: Dirs<'_>) -> Self {
        value.path()
    }
}
