use crate::Config;
use clap::crate_name;
use std::{
    env, fmt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Directories<'a> {
    Home,
    Data,
    Cache,
    Root,
    Files(&'a Config),
}

impl Directories<'_> {
    pub fn path(&self) -> PathBuf {
        let home = match env::var_os("HOME") {
            Some(s) => PathBuf::from(s),
            None => panic!("`HOME` environment variable is not set"),
        };

        match &self {
            Self::Home => home,
            Self::Root => PathBuf::from("/"),
            Self::Data => env::var_os("XDG_DATA_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or(home.join(".local/share").join(crate_name!())),
            Self::Cache => env::var_os("XDG_CACHE_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or(home.join(".cache").join(crate_name!())),
            Self::Files(config) => config.path.parent().unwrap().join("files"),
        }
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path().join(path)
    }
}

impl fmt::Display for Directories<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}

impl Into<PathBuf> for Directories<'_> {
    fn into(self) -> PathBuf {
        self.path()
    }
}
