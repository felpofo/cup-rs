use super::Command;
use crate::dirs::Dirs;
use anyhow::Result;
use clap::{command, ArgMatches};
use std::fs;

#[derive(Debug)]
pub struct List;

impl Command for List {
    fn run(_matches: &ArgMatches) -> Result<()> {
        let path = Dirs::Data.path();
        let exports = fs::read_dir(path)?;

        exports.for_each(|export| {
            let name = export.unwrap().file_name();

            println!("{}", name.to_string_lossy());
        });

        Ok(())
    }
}

#[allow(clippy::from_over_into)]
impl Into<clap::Command> for List {
    fn into(self) -> clap::Command {
        command!("list").about("List dotfiles")
    }
}
