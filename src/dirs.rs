use std::{env, path::PathBuf, process};
use termion::color::{Fg, Red, Reset};

pub enum Directories {
    Data,
    Config,
    Cache,
}

impl Directories {
    /// Get the path of some directory
    ///
    /// # Panics
    ///
    /// This function will panic if neither `XDG_xxx_HOME` or `HOME` environment variable are set
    pub fn path(&self) -> PathBuf {
        let home = || {
            let os_str = env::var_os("HOME");
            
            if let Some(home) = os_str {
                return PathBuf::from(&home);
            }
            
            eprintln!("{}ERROR: `HOME` environment variable is not set{}", Fg(Red), Fg(Reset));
            process::exit(1);
        };

        match &self {
            Directories::Data => env::var_os("XDG_DATA_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or_else(|| home().join(".local/share/yada")),
            Directories::Config => env::var_os("XDG_CONFIG_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or_else(|| home().join(".config/yada")),
            Directories::Cache => env::var_os("XDG_CACHE_HOME")
                .and_then(|s| Some(PathBuf::from(s)))
                .unwrap_or_else(|| home().join(".cache/yada")),
        }
    }
}
