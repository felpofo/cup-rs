use clap::command;
use cup::{
    commands::{Command, Export, Import},
    Error,
};

fn main() -> Result<(), Error> {
    let cmd = parse_args();

    match cmd.get_matches().subcommand() {
        Some(("import", matches)) => Import::run(matches),
        Some(("export", matches)) => Export::run(matches),
        _ => Ok(()),
    }
}

pub fn parse_args() -> clap::Command {
    let app = command!()
        .about("I bet you can't hold it")
        .subcommand_required(true);

    app.subcommand(Import).subcommand(Export)
}
