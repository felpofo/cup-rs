use anyhow::Result;
use clap::command;
use cup::commands::*;

fn main() -> Result<()> {
    let cmd = parse_args();

    match cmd.get_matches().subcommand() {
        Some(("import", matches)) => Import::run(matches),
        Some(("export", matches)) => Export::run(matches),
        Some(("list", matches)) => List::run(matches),
        _ => Ok(()),
    }
}

pub fn parse_args() -> clap::Command {
    let app = command!()
        .about("I bet you can't hold it")
        .disable_help_subcommand(true)
        .subcommand_required(true);

    app.subcommand(Import).subcommand(Export).subcommand(List)
}
