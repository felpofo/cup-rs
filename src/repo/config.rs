use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CupConfig {
    pub name: String,
    pub files: Vec<File>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct File {
    pub from: String,
    pub to: String,
}
