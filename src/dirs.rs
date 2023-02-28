use crate::error_and_exit;
use clap::crate_name;
use std::{env, fmt, path::PathBuf};

pub enum Directories {
    Home,
    Data,
    Cache,
}

impl Directories {
    /// Get the path of some directory
    ///
    /// # Panics
    ///
    /// This function will panic if `HOME` environment variable is not set
    pub fn path(&self) -> PathBuf {
        let os_str = env::var_os("HOME");

        if os_str.is_none() {
            error_and_exit("`HOME` environment variable is not set");
        }

        let home = PathBuf::from(os_str.unwrap());

        match &self {
            Directories::Home => home,
            Directories::Data => env::var_os("XDG_DATA_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or_else(|| home.join(".local/share").join(crate_name!())),
            Directories::Cache => env::var_os("XDG_CACHE_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or_else(|| home.join(".cache").join(crate_name!())),
        }
    }
}

impl fmt::Display for Directories {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}
