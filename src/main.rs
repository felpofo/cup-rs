use clap::{arg, command, ArgAction, Command};
use cup::commands::Export;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cmd = parse_args();

    match cmd.get_matches().subcommand() {
        Some(("export", matches)) => {
            let name = matches.get_one::<String>("NAME").expect("Clap shows help if not exists");

            match matches.subcommand() {
                Some(("add", submatches)) => Export::add(name, matches, submatches),
                Some(("remove", submatches)) => Export::remove(name, matches, submatches),
                _ => Export::create(name),
            }
        }
        _ => Ok(()),
    }
}

pub fn parse_args() -> Command {
    let app = command!()
        .about("I bet you can't hold it")
        .subcommand_required(true);

    let export = command!("export")
        .about("Save your dotfiles")
        .arg_required_else_help(true)
        .arg(arg!(<NAME> "Export name"))
        .subcommands([
            command!("add")
                .about("Add file(s)")
                .arg_required_else_help(true)
                .arg(arg!(<FILES> ... "Files you want to add")),
            command!("remove")
                .about("Remove file(s)")
                .arg_required_else_help(true)
                .arg(arg!([FILES] ... "Files you want to remove"))
                .arg(arg!(-i --interactive).action(ArgAction::SetTrue)),
        ]);

    app.subcommand(export)
}
