mod config;
mod error;
mod output;

use clap::{
    Parser as ClapParser, Subcommand,
    builder::{Styles, styling::AnsiColor},
};
use console::style;
use thiserror_ext::AsReport;

use language::{
    analyze::analyze_routes,
    ast::Parser,
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

#[derive(ClapParser, Debug)]
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
            let mut parser = Parser::new(&bytes)?;
            let ast = parser.parse()?;

            println!("{:#?}", ast);

            // let mut display = display::Display {
            //     source: Vec::new(),
            //     line_length: 0,
            //     pos: 0,
            //     tbs: 0,
            // };
            // display.display_block(&ast.block);
            // println!("{}", str::from_utf8(&display.source).unwrap());

            let vm = treewalker::create_state();
            let result = execute(vm, &ast);

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
