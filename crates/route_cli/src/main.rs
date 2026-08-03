mod config;
mod error;

use std::collections::HashMap;

use clap::{Parser, Subcommand};
use console::style;
use thiserror_ext::AsReport;

use language::{
    analyze::{self, analyze_routes},
    ast::{Parser as RtParser, ast::Span},
    compiler::{Compiler, VMState},
    treewalker::{self, create_state, execute},
    vm::VirtualMachine,
};

use crate::{config::RouteConfig, error::Error};

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

            let mut compiler = Compiler {
                vm_state: VMState {
                    instructions: Vec::new(),
                    locals: Vec::new(),
                },
                next_instrucion: 1,
            };

            let instructions = compiler.compile(ast);
            let mut vm = VirtualMachine {
                locals: Vec::new(),
                globals: HashMap::new(),
                instruction_at: 0,
                instruction_end: instructions.len() - 1,
                stack: Vec::new(),
                n: 0,
                routes: Vec::new(),
            };

            let routes = vm.run(instructions);
            let analysis = analyze_routes(&routes);

            if !analysis.issues.is_empty() {
                eprintln!("issues: {:#?}", analysis.issues)
            }

            println!(
                "analysis: HTTP: {:#?}, TCP: {:#?}",
                analysis.http, analysis.tcp
            )

            // let execution = execute(create_state(), &ast).map_err(|issues| {
            //     display_issues(&bytes, &issues);
            //     Error::execution(issues.len())
            // })?;

            // let analysis = analyze_routes(&execution.routes);
            // if !analysis.issues.is_empty() {
            //     display_issues(&bytes, &analysis.issues);
            //     return Err(Error::execution(analysis.issues.len()));
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
