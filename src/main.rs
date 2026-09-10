mod cli;
mod commands;
mod config;

use std::{error::Error, process};

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let Cli { config, command } = Cli::parse();

    match command {
        Command::Build(args) => commands::build::run(&config, args)?,
        Command::Check => commands::check::run(&config)?,
        Command::Guide => commands::guide::run(),
    }

    Ok(())
}
