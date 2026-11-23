//! Streamed phase timing: a record when a phase of a run starts and another
//! when it ends, each written and flushed as it happens.
//!
//! A run killed from outside (a batch system's SIGKILL at its time cap) runs no
//! destructor and prints no summary. What it leaves is what it wrote before the
//! signal, so the start record is written before the phase does any work: the
//! last `start` without its `end` names the phase the run was in.
//!
//! Under `--output-ldjson` each record is a `phase` object on `stdout`, whatever
//! the visible level. In every mode the same record is a log event: the start
//! at `debug` and the end, with its wall time and sizes, at `info`, so
//! `--log-file` holds both.
//!
//! The records time the CLI's own calls and touch nothing they time: no random
//! number is drawn, and no input of a phase is read or changed.

use std::sync::OnceLock;
use std::time::Instant;

use serde_json::{Map, Value};
use tracing::{debug, info};

use crate::output::{self, Mode};

/// The phases of `integrate` and `generate`, in the order a run enters them.
///
/// A run enters each at most once, and a run that ends early (a refusal,
/// `--check`) stops at the phase it ended in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum PhaseName {
    /// Reading the card or artifact and loading the model it names.
    Input,
    /// Enumerating the proc card's diagrams.
    Enumerate,
    /// Compiling each diagram set, once for the run and across the pool, and
    /// recording its amplitude probe through the evaluator.
    AmplitudeProbe,
    /// Locating and reading the parton densities.
    Pdf,
    /// Splitting the sets into flavour groups over the evaluators the probe
    /// compiled (compiling again only a set whose outgoing legs a group
    /// reorders), with the cuts and the checks that read them.
    Compile,
    /// Building the phase-space maps and the integrand over its channels. On
    /// fixed-energy beams and decays the per-diagram channel maps themselves are
    /// built in [`PhaseName::AlphaSurvey`], which builds them as it surveys.
    Channels,
    /// Adapting the channel selection weights on survey points.
    AlphaSurvey,
    /// The VEGAS iterations, warm-up included.
    Vegas,
    /// The per-configuration shares and channel records read off the trained
    /// grids.
    Configurations,
    /// The artifact assembled and written.
    Bank,
    /// `generate`'s sample: the amplitude-probe replay, compilation, the
    /// weight scan and the unweighted draw, whose own `progress` stages
    /// resolve it further.
    Generate,
}

impl PhaseName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            PhaseName::Input => "input",
            PhaseName::Enumerate => "enumerate",
            PhaseName::AmplitudeProbe => "amplitude_probe",
            PhaseName::Pdf => "pdf",
            PhaseName::Compile => "compile",
            PhaseName::Channels => "channels",
            PhaseName::AlphaSurvey => "alpha_survey",
            PhaseName::Vegas => "vegas",
            PhaseName::Configurations => "configurations",
            PhaseName::Bank => "bank",
            PhaseName::Generate => "generate",
        }
    }
}

/// The instant every record's `elapsed_s` is measured from.
static ORIGIN: OnceLock<Instant> = OnceLock::new();

/// Fix the origin of `elapsed_s` at the start of the process. Called once, by
/// `main`; a later call is ignored. Without it, the first phase fixes it.
pub(crate) fn mark_origin() {
    ORIGIN.get_or_init(Instant::now);
}

/// Seconds since the origin, on the monotonic clock.
fn elapsed() -> f64 {
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64()
}

/// One phase in progress. Its end record is written by [`Phase::end`], or, if
/// the phase is left by an error return, when it is dropped, with `outcome`
/// `failed`.
#[must_use = "a phase ends when its guard is ended or dropped"]
pub(crate) struct Phase {
    name: PhaseName,
    started: f64,
    sizes: Map<String, Value>,
    ended: bool,
}

/// Enter `name`: its start record is written and flushed before this returns.
pub(crate) fn start(name: PhaseName) -> Phase {
    let started = elapsed();
    debug!(phase = name.as_str(), "phase {}: started", name.as_str());
    if output::mode() == Mode::Ldjson {
        let mut fields = Map::new();
        fields.insert("event".to_string(), Value::from("start"));
        fields.insert("phase".to_string(), Value::from(name.as_str()));
        fields.insert("elapsed_s".to_string(), Value::from(started));
        output::emit_object("phase", fields);
    }
    Phase {
        name,
        started,
        sizes: Map::new(),
        ended: false,
    }
}

impl Phase {
    /// Record a size that explains the phase's time, carried by its end record.
    pub(crate) fn size(&mut self, key: &'static str, n: usize) {
        self.sizes.insert(key.to_string(), Value::from(n));
    }

    /// End the phase as completed.
    pub(crate) fn end(mut self) {
        self.finish("ok");
    }

    fn finish(&mut self, outcome: &'static str) {
        self.ended = true;
        let now = elapsed();
        let wall = now - self.started;
        let name = self.name.as_str();
        let sizes = self
            .sizes
            .iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect::<Vec<_>>()
            .join(", ");
        info!(
            phase = name,
            wall_s = wall,
            outcome,
            "phase {name}: {wall:.3} s{}{}",
            if outcome == "ok" { "" } else { ", failed" },
            if sizes.is_empty() {
                String::new()
            } else {
                format!(" ({sizes})")
            }
        );
        if output::mode() == Mode::Ldjson {
            let mut fields = Map::new();
            fields.insert("event".to_string(), Value::from("end"));
            fields.insert("phase".to_string(), Value::from(name));
            fields.insert("elapsed_s".to_string(), Value::from(now));
            fields.insert("wall_s".to_string(), Value::from(wall));
            fields.insert("outcome".to_string(), Value::from(outcome));
            fields.insert(
                "sizes".to_string(),
                Value::Object(std::mem::take(&mut self.sizes)),
            );
            output::emit_object("phase", fields);
        }
    }
}

impl Drop for Phase {
    fn drop(&mut self) {
        if !self.ended {
            self.finish("failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_time_never_runs_backwards() {
        mark_origin();
        let a = elapsed();
        let b = elapsed();
        assert!(b >= a && a >= 0.0);
    }

    #[test]
    fn phase_names_are_distinct() {
        let names = [
            PhaseName::Input,
            PhaseName::Enumerate,
            PhaseName::AmplitudeProbe,
            PhaseName::Pdf,
            PhaseName::Compile,
            PhaseName::Channels,
            PhaseName::AlphaSurvey,
            PhaseName::Vegas,
            PhaseName::Configurations,
            PhaseName::Bank,
            PhaseName::Generate,
        ];
        let mut seen: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), names.len());
    }
}
