use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub trait Expand {
    fn expand(&self) -> io::Result<Vec<PathBuf>>;
}

impl Expand for PathBuf {
    fn expand(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = vec![];

        if self.is_file() {
            files.push(self.into());
        } else if self.is_dir() {
            for entry in fs::read_dir(self)? {
                let mut inner = Self::expand(&entry?.path())?;
                files.append(&mut inner);
            }
        }

        Ok(files)
    }
}

impl Expand for Path {
    fn expand(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = vec![];

        if self.is_file() {
            files.push(self.into());
        } else if self.is_dir() {
            for entry in fs::read_dir(self)? {
                let mut inner = Self::expand(&entry?.path())?;
                files.append(&mut inner);
            }
        }

        Ok(files)
    }
}
