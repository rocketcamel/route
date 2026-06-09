mod analyze;
mod ast;
mod config;
mod error;
mod output;
mod treewalker;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use console::style;
use thiserror_ext::AsReport;

use crate::{
    ast::Parser as RtParser,
    config::RouteConfig,
    output::render_output,
    treewalker::{create_state, execute},
};

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
            let project = RouteConfig::read()?;

            let bytes = std::fs::read(&project.input.module_path)?;
            let mut parser = RtParser::new(&bytes)?;
            let ast = parser.parse()?;

            match execute(create_state(), &ast) {
                Ok(result) => {
                    let output = render_output(&result.http, &result.tcp);
                    println!("{output}");
                }
                Err(issues) => {
                    eprintln!("{:#?}", issues)
                }
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", style("error").red(), e.as_report())
    }
}
