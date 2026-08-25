mod config;
mod error;
// mod output;

use clap::{Parser, Subcommand};
use console::style;
use thiserror_ext::AsReport;

use language::{
    // analyze::analyze_routes,
    ast::Parser as RtParser,
    // compiler::{Compiler, VMState},
    // treewalker::{self, execute},
    // vm::VirtualMachine,
};

use crate::config::RouteConfig;

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

            println!("{ast:#?}")

            // let vm = treewalker::create_state();
            // let result = execute(vm, &ast);

            // if let Ok(result) = result {
            //     let analysis = analyze_routes(&result.routes);

            //     if !analysis.issues.is_empty() {
            //         eprintln!("issues: {:#?}", analysis.issues)
            //     }

            //     let result = render_output(&project, &analysis.http, &analysis.tcp);
            //     println!("{result}")
            // } else if let Err(issues) = result {
            //     eprintln!("issues: {:#?}", issues);
            //     return Ok(());
            // }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", style("error").red(), e.as_report())
    }
}
