mod config;
mod error;
mod output;

use clap::{Parser, Subcommand};
use console::style;
use thiserror_ext::AsReport;

use language::{
    analyze::analyze_routes,
    ast::Parser as RtParser,
    treewalker::{self, execute},
};

use crate::{config::RouteConfig, output::render_output};

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

            let vm = treewalker::create_state();
            let result = execute(vm, &ast);

            match result {
                Ok(result) => {
                    let analysis = analyze_routes(&result.routes);

                    if !analysis.issues.is_empty() {
                        eprintln!("issues: {:#?}", analysis.issues)
                    }

                    let output = render_output(&project, &analysis.http, &analysis.tcp);
                    println!("{output}")
                }
                Err(issues) => {
                    eprintln!("issues: {issues:#?}")
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
