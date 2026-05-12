mod ast;
mod error;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use console::style;
use thiserror_ext::AsReport;

use crate::ast::Parser as RtParser;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    GenerateRoutes,
}

fn run() -> crate::error::Result<()> {
    let cli = Args::parse();

    match &cli.command {
        Commands::GenerateRoutes => {
            let bytes = std::fs::read(PathBuf::from("./test.rt"))?;
            let mut parser = RtParser::new(&bytes)?;
            let ast = parser.parse()?;
            println!("{ast:?}")
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", style("error").red(), e.as_report())
    }
}
