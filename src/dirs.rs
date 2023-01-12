use std::{env, fs, io};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use directories::BaseDirs;
use uuid::Uuid;

pub fn create_dir(dir: &PathBuf) -> io::Result<PathBuf> {
    match fs::create_dir(&dir) {
        Ok(_) => Ok(PathBuf::clone(dir)),
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            format!("Error creating dir: '{}'", dir.display()).as_str(),
        )),
    }
}

pub fn get_tmp_dir() -> io::Result<PathBuf> {
    let uuid = Uuid::new_v4().to_string().replace("-", "");
    let dir = env::temp_dir().join(format!("rust-dotfiles_{}", uuid));

    create_dir(&dir)
}

pub fn get_data_dir() -> io::Result<PathBuf> {
    match BaseDirs::new() {
        Some(dirs) => {
            let dir = dirs.data_dir().join("rust-dotfiles");

            if !dir.exists() {
                if let Err(err) = create_dir(&dir) {
                    return Err(err);
                };
            }

            Ok(dir)
        }
        None => Err(Error::new(
            ErrorKind::Other,
            format!("System data directory does not exist"),
        ))
    }
}
