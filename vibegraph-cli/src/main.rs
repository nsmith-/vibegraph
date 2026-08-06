//! `vibegraph` command-line entry point.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod assets;
mod check;
mod fetch;
mod generate;
mod integrate;
mod network;
mod parallel;

use network::NetworkPolicy;

/// `--version` output: version line, then every license notice the binary is
/// required to carry. The binary is distributed bare (no accompanying files),
/// so it is the only vehicle for the notices its contents demand: vibegraph's
/// own MIT/Apache-2.0 texts, and THIRD-PARTY-NOTICES for the MadGraph5_aMC@NLO
/// SM model interned via `vibegraph-lib/src/ufo/sm_assets/` — its license
/// requires binary redistributions to reproduce the notice, conditions and
/// disclaimers. `-V` still prints just the version line.
const LONG_VERSION: &str = concat!(
    env!("VIBEGRAPH_VERSION"),
    "\n\nvibegraph is dual-licensed MIT OR Apache-2.0, and embeds material derived\n\
     from the MadGraph5_aMC@NLO Standard Model UFO model. All notices follow.\n\n\
     ================================================================================\n\n",
    include_str!("../../THIRD-PARTY-NOTICES"),
    "\n================================================================================\n\
     The MIT License (vibegraph)\n\
     ---------------------------\n\n",
    include_str!("../../LICENSE-MIT"),
    "\n================================================================================\n\
     The Apache License 2.0 (vibegraph)\n\
     ----------------------------------\n\n",
    include_str!("../../LICENSE-APACHE"),
);

#[derive(Parser)]
#[command(
    name = "vibegraph",
    about = "Toy leading-order Monte-Carlo event generator",
    version = env!("VIBEGRAPH_VERSION"),
    long_version = LONG_VERSION
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

#[cfg(test)]
mod tests {
    use super::LONG_VERSION;

    /// The bare-binary distribution carries its license notices nowhere but
    /// here; each required element must survive in the `--version` text.
    #[test]
    fn long_version_carries_every_required_notice() {
        // MadGraph5_aMC@NLO notice: copyright line, the binary-redistribution
        // condition itself, and the disclaimer.
        assert!(LONG_VERSION.contains("Copyright (c) 2009, 2013, the MadTeam"));
        assert!(LONG_VERSION.contains("Redistributions in binary form must reproduce"));
        assert!(LONG_VERSION.contains("THE SOFTWARE IS PROVIDED \"AS IS\""));
        // vibegraph's own dual license texts.
        assert!(LONG_VERSION.contains("Permission is hereby granted, free of charge"));
        assert!(LONG_VERSION.contains("Apache License"));
        assert!(LONG_VERSION.contains("Version 2.0, January 2004"));
    }
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
