//! `vibegraph` command-line entry point.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod assets;
mod check;
mod fetch;
mod generate;
mod integrate;
mod logging;
mod network;
mod parallel;
mod si;
mod tui;

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

    #[command(flatten)]
    log: logging::LogArgs,
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

impl Command {
    /// Whether this command has a run to watch. `check-events` reads a file and
    /// prints a report; there is no progress to show and its report *is* the
    /// output, so it is left alone.
    fn is_watchable(&self) -> bool {
        match self {
            Command::Integrate(_) | Command::Generate(_) => true,
            Command::CheckEvents(_) => false,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    // The display takes the terminal before the subscriber is built, because
    // where the lines go depends on whether it got it.
    let (display, unavailable) = if cli.log.wants_tui() && cli.command.is_watchable() {
        match tui::Tui::start() {
            Ok(display) => (Some(display), None),
            Err(why) => (None, Some(why)),
        }
    } else {
        (None, None)
    };
    // Nothing above this line may log: until the subscriber is installed there
    // is nowhere for an event to go.
    let logging = match logging::init(&cli.log, display.as_ref().map(tui::Tui::feed)) {
        Ok(handle) => handle,
        Err(why) => {
            if let Some(display) = display {
                display.finish();
            }
            eprintln!("error: {why}");
            return ExitCode::FAILURE;
        }
    };
    // The display's level and scope keys drive the filter through this handle;
    // with no display running nobody holds it and the level is the one the flags
    // asked for, for the whole run.
    match &display {
        Some(display) => display.attach(logging),
        None => drop(logging),
    }
    if let Some(why) = unavailable {
        tracing::debug!("reporting in plain lines: {why}");
    }
    let network = NetworkPolicy::from_env(cli.no_network, cli.yes);
    let result = match cli.command {
        Command::Integrate(args) => integrate::run(&args, network),
        Command::Generate(args) => generate::run(&args, network),
        Command::CheckEvents(args) => check::run(&args),
    };
    // Reported before the display comes down, so the message lands in the
    // scrollback with the rest of the run rather than after its closing line.
    if let Err(err) = &result {
        tracing::error!("{err}");
    }
    if let Some(display) = display {
        display.finish();
    }
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
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
