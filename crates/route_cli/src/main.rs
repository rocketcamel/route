mod config;
mod error;

use std::{collections::HashMap, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
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

pub trait Issue {
    fn why(&self) -> &str;
    fn span(&self) -> Span;
}

#[rustfmt::skip]
impl Issue for treewalker::Issue {
    fn span(&self) -> Span { self.span }
    fn why(&self) -> &str { &self.why }
}

#[rustfmt::skip]
impl Issue for analyze::Issue {
    fn span(&self) -> Span { self.span }
    fn why(&self) -> &str { &self.why }
}

fn display_issues<I: Issue>(input: &[u8], issues: &[I]) {
    let source = String::from_utf8_lossy(input);
    let lines: Vec<&str> = source.lines().collect();

    eprintln!(
        "{}: route file has {} issue{}",
        style("error").red(),
        issues.len(),
        if issues.len() == 1 { "" } else { "s" }
    );

    for (idx, issue) in issues.iter().enumerate() {
        let span = issue.span();

        let line_number = span.line;
        let col_number = span.col.max(1);
        let line = lines.get(line_number - 1).copied().unwrap_or("");

        eprintln!(
            "{}) {}:{} {}",
            idx + 1,
            line_number,
            col_number,
            issue.why()
        );
        eprintln!("   {}", line);

        let caret_padding = " ".repeat(col_number.saturating_sub(1));
        let width = span.end.saturating_sub(span.start).max(1);
        let underline = format!("^{}", "~".repeat(width.saturating_sub(1)));

        eprintln!("   {}{}", caret_padding, underline);
    }
}

fn throw<T, I: Issue>(input: &[u8], issues: &[I]) -> crate::error::Result<T> {
    display_issues(input, issues);
    Err(crate::error::Error::execution(issues.len()))
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
            };
            vm.run(instructions);

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
