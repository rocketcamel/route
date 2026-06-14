mod config;
mod error;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};
use console::style;
use thiserror_ext::AsReport;

use language::{
    analyze::analyze_routes, ast::Parser as RtParser, treewalker::{self, create_state, execute}
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

fn display_issues<I>(input: &[u8], issues: &[I]) {
    let source = String::from_utf8_lossy(input);
    let lines: Vec<&str> = source.lines().collect();

    eprintln!(
        "{}: route file has {} issue{}",
        style("error").red(),
        issues.len(),
        if issues.len() == 1 { "" } else { "s" }
    );

    for (idx, issue) in issues.iter().enumerate() {
        let line_number = issue.span.line;
        let col_number = issue.span.col.max(1);
        let line = lines.get(line_number - 1).copied().unwrap_or("");

        eprintln!("{}) {}:{} {}", idx + 1, line_number, col_number, issue.why);
        eprintln!("   {}", line);

        let caret_padding = " ".repeat(col_number.saturating_sub(1));
        let width = issue.span.end.saturating_sub(issue.span.start).max(1);
        let underline = format!("^{}", "~".repeat(width.saturating_sub(1)));

        eprintln!("   {}{}", caret_padding, underline);
    }
}

fn throw<T, I>(input: &[u8], issues: &[I]) -> crate::error::Result<T> {
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

            let execution = execute(create_state(), &ast);

            if let Err(issues) = execution {
                display_issues(&bytes, &issues);

                return Ok(())
            }

            let analysis = analyze_routes()
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", style("error").red(), e.as_report())
    }
}
