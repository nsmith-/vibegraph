//! `vibegraph` command-line entry point.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod assets;
mod check;
mod fetch;
mod generate;
mod integrate;
mod network;

use network::NetworkPolicy;

#[derive(Parser)]
#[command(
    name = "vibegraph",
    about = "Toy leading-order Monte-Carlo event generator",
    version = env!("VIBEGRAPH_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Never download anything; a missing asset becomes a refusal stating the
    /// URL, size and checksum it would have fetched. `$VIBEGRAPH_NO_NETWORK`
    /// does the same for a whole environment, and either one outranks `--yes`.
    #[arg(long, global = true)]
    no_network: bool,

    /// Answer yes to any "may I download this?" question instead of asking.
    /// Needed to fetch an asset from a script, a CI job, or anything else with
    /// no terminal, where the default is to refuse.
    #[arg(long, short = 'y', global = true)]
    yes: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Integrate the hadronic Drell–Yan cross section, saving the adapted VEGAS grid.
    Integrate(integrate::IntegrateArgs),
    /// Generate an unweighted event sample from a saved grid, as a Les Houches file.
    Generate(generate::GenerateArgs),
    /// Read a Les Houches file back and check that it is internally consistent.
    CheckEvents(check::CheckArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let network = NetworkPolicy::from_env(cli.no_network, cli.yes);
    let result = match cli.command {
        Command::Integrate(args) => integrate::run(&args, network),
        Command::Generate(args) => generate::run(&args, network),
        Command::CheckEvents(args) => check::run(&args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
