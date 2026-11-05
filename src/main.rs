use clap::Parser;
use std::process;
use arclang::cli::{Cli, CliRunner};

fn main() {
    let cli = Cli::parse();
    if let Some(origin) = &cli.origin {
        arclang::compiler::set_origin(origin.clone());
    }
    let runner = CliRunner::new(&cli);

    if let Err(e) = runner.run(cli.command) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
