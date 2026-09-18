mod config;
mod error;
mod output;

use clap::{
    Parser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};
use console::style;
use thiserror_ext::AsReport;

use language::{
    analyze::analyze_routes,
    ast::Parser as RtParser,
    treewalker::{self, execute, types::Source},
};

use crate::{config::RouteConfig, output::render_output};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().bold())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Blue.on_default().bold())
    .placeholder(AnsiColor::Magenta.on_default())
    .error(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Green.on_default().bold())
    .invalid(AnsiColor::Yellow.on_default().bold());

#[derive(Parser, Debug)]
#[command(styles = STYLES)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// generate kubernetes gateway api routes
    Generate,
}

fn run() -> crate::error::Result<()> {
    let cli = Args::parse();

    match &cli.command {
        Commands::Generate => {
            let project = RouteConfig::read()?;

            let bytes = std::fs::read(&project.input.module_path)?;
            let mut parser = RtParser::new(&bytes)?;
            let ast = parser.parse()?;

            let source = Source {
                source: bytes,
                ast: ast,
            };

            let vm = treewalker::create_state(&source);
            let result = execute(vm, &source.ast);

            match result {
                Ok(result) => {
                    let analysis = analyze_routes(&result.routes);

                    if !analysis.issues.is_empty() {
                        eprintln!("issues: {:#?}", analysis.issues)
                    }

                    let output = render_output(&project, &analysis.http, &analysis.tcp);
                    std::fs::write(&project.output.path, output)?;

                    println!(
                        "wrote routes to {}",
                        project.output.path.canonicalize().unwrap().display()
                    )
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
