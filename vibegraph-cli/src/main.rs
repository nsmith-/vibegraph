//! `vibegraph` command-line entry point.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod integrate;

#[derive(Parser)]
#[command(
    name = "vibegraph",
    about = "Toy leading-order Monte-Carlo event generator",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Integrate the hadronic Drell–Yan cross section, saving the adapted VEGAS grid.
    Integrate(integrate::IntegrateArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Integrate(args) => integrate::run(&args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
