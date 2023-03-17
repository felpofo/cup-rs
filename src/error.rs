use std::path::PathBuf;

pub enum Error {
    AlreadyExists(PathBuf),
    InvalidRepository,
    SshKeyNotFound,
    Io(String),
    SerdeError(String),
    Git2(String),
    Regex(String),
    Other(&'static str),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyExists(path) => {
                write!(f, "`{}` already exists", path.display())
            }
            Error::InvalidRepository => write!(f, "Invalid repository"),
            Error::Other(msg) => write!(f, "Other: `{msg}`"),
            Error::Io(msg) => write!(f, "Io: `{msg}`"),
            Error::SshKeyNotFound => write!(f, "Ssh key not found"),
            Error::SerdeError(msg) => write!(f, "Serde: `{msg}`"),
            Error::Git2(msg) => write!(f, "Git2: `{msg}`"),
            Error::Regex(msg) => write!(f, "Regex: `{msg}`"),
        }
    }
}

impl From<git2::Error> for Error {
    fn from(value: git2::Error) -> Self {
        Error::Git2(value.message().into())
    }
}

impl From<regex::Error> for Error {
    fn from(value: regex::Error) -> Self {
        Error::Regex(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Error::SerdeError(value.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(_value: Box<dyn std::error::Error>) -> Self {
        Error::Other("")
    }
}
