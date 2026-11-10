//! The run's output mode, and the `--output-ldjson` stream.
//!
//! A run has three things to report: log events, progress events, and its
//! result or refusal. One mode, chosen once in `main` from the flags and the
//! terminal, formats all three:
//!
//! | mode | logs | progress | result or refusal |
//! |---|---|---|---|
//! | [`Mode::Plain`] | text lines on `stderr` | not shown | text lines (result on `stdout`) |
//! | [`Mode::Rich`] | the status pane's scrollback | the status pane | text lines, held until the pane comes down |
//! | [`Mode::Ldjson`] | `log` objects | `progress` objects | one `result` or `refusal` object |
//!
//! # The ldjson schema, version [`SCHEMA_VERSION`]
//!
//! Every line on `stdout` is one JSON object, and every object carries `kind`
//! and `schema_version`. A consumer refuses a `schema_version` it does not
//! know. Everything is on the one stream, so the objects arrive in the order
//! the run produced them; `stderr` carries nothing but what escapes the
//! subscriber (a panic's message).
//!
//! - **`log`**: one per log event at the visible level (`-v`, `-q`,
//!   `--log-level`, `RUST_LOG`), as `tracing-subscriber`'s JSON formatter writes
//!   it: `timestamp`, `level`, `target`, `fields` (the event's fields, with the
//!   text as `fields.message`) and, inside a stage's span, `span` (its `name`).
//! - **`progress`**: one per event on the progress target, whatever the visible
//!   level, in the same layout. `fields` carries the progress field contract of
//!   `vibegraph::progress` verbatim — `stage`, `done`, `total` when known, and
//!   the stage's own fields — so the VEGAS stage's `sigma` and `err` are in the
//!   integrand's natural units (GeV⁻² for a cross section, GeV for a width).
//! - **`result`**: the command's final object, once, after its output is
//!   written: [`IntegrateResult`] for `integrate` and `inspect`,
//!   [`GenerateResult`] for `generate`, and for `enumerate` the paths it wrote
//!   with the `diagrams.json` summary nested under `summary`.
//! - **`refusal`**: in place of a result, once: [`Refusal`]. A refused run
//!   writes no artifact, so this is the only record of why.
//!
//! Values are in base units at full precision: cross sections in pb, widths,
//! masses and scales in GeV. An undefined value (a χ²/dof over fewer than two
//! iterations) is `null`. Adding a field to an object keeps the version;
//! removing, renaming or re-meaning one, or adding an error kind, bumps it.

use std::io::Write;
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::{Map, Value};
use vibegraph::hadronic::Observable;

use crate::error::{CliError, ErrorKind};

/// The ldjson schema's version, on every line.
///
/// Adding a field to any object keeps it, so a reader ignores fields it does
/// not know; removing, renaming or re-meaning a field, or adding or removing an
/// [`ErrorKind`], bumps it.
pub(crate) const SCHEMA_VERSION: u32 = 1;

/// How one run formats its logs, progress and result.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Text lines, no progress: piped, in the background, or `--no-tui`.
    Plain,
    /// The live status pane on a terminal.
    Rich,
    /// One JSON object per line on `stdout`, under `--output-ldjson`.
    Ldjson,
}

static MODE: OnceLock<Mode> = OnceLock::new();

/// Fix the run's mode. Called once, by `main`, before any command runs; a
/// second call is ignored.
pub(crate) fn set_mode(mode: Mode) {
    let _ = MODE.set(mode);
}

/// The run's mode; [`Mode::Plain`] until `main` has chosen one.
pub(crate) fn mode() -> Mode {
    MODE.get().copied().unwrap_or(Mode::Plain)
}

/// Write `fields` as one line of `kind`, stamped with the schema version.
///
/// The line is written under the `stdout` lock in one call, so lines from the
/// subscriber's threads never interleave mid-object.
pub(crate) fn emit_object(kind: &str, mut fields: Map<String, Value>) {
    fields.insert("kind".to_string(), Value::from(kind));
    fields.insert("schema_version".to_string(), Value::from(SCHEMA_VERSION));
    let mut line =
        serde_json::to_vec(&Value::Object(fields)).expect("a JSON map always serializes");
    line.push(b'\n');
    let mut out = std::io::stdout().lock();
    // A reader that has gone away is not the run's failure.
    let _ = out.write_all(&line);
    let _ = out.flush();
}

/// `value`'s fields as a JSON map.
fn fields_of(value: &impl Serialize) -> Map<String, Value> {
    match serde_json::to_value(value).expect("a result object serializes") {
        Value::Object(map) => map,
        other => panic!("a result object serializes to an object, not {other}"),
    }
}

/// Emit the command's final object, in ldjson mode only: the text modes have
/// printed their result lines already.
pub(crate) fn result(value: &impl Serialize) {
    if mode() == Mode::Ldjson {
        emit_object("result", fields_of(value));
    }
}

/// A line of the command's text result: printed in the text modes, and
/// dropped in ldjson mode, where the `result` object carries it instead.
pub(crate) fn text_line(line: std::fmt::Arguments<'_>) {
    if mode() != Mode::Ldjson {
        crate::tui::result_line(line);
    }
}

/// The `refusal` object: why the command failed, as a stable kind and the
/// human message.
#[derive(Debug, Serialize, PartialEq)]
struct Refusal {
    error_kind: ErrorKind,
    message: String,
}

/// Emit `err` as the run's `refusal` object.
pub(crate) fn refusal(err: &CliError) {
    emit_object(
        "refusal",
        fields_of(&Refusal {
            error_kind: err.kind(),
            message: err.message().to_string(),
        }),
    );
}

/// What an integral measures, by name.
#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObservableName {
    /// A `2 → n` cross section, in pb.
    CrossSection,
    /// A `1 → n` partial width, in GeV.
    PartialWidth,
}

impl From<Observable> for ObservableName {
    fn from(o: Observable) -> Self {
        match o {
            Observable::CrossSection => ObservableName::CrossSection,
            Observable::PartialWidth => ObservableName::PartialWidth,
        }
    }
}

/// The factorization scale the parton densities were read at.
#[derive(Copy, Clone, Debug, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum MuF {
    /// No parton densities were read: fixed-energy beams, or a decay.
    NoDensities,
    /// One constant on both beams.
    Constant { gev: f64 },
    /// A constant on each beam, the two different.
    PerBeam { gev: [f64; 2] },
    /// Chosen per event, by the run card's `dynamical_scale_choice`.
    PerEvent { dynamical_scale_choice: i64 },
}

/// The model a run compiled its amplitudes from.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ModelRef {
    pub(crate) label: String,
    /// The model's content digest, as the artifact records it.
    pub(crate) digest: String,
}

/// Which channel of which decomposition a result belongs to: the artifact's
/// channel key, tagged by `type`.
#[derive(Copy, Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ChannelKeyObject {
    Whole,
    Channel {
        channel: usize,
    },
    GroupChannel {
        group: usize,
        channel: usize,
    },
    MultiplicityChannel {
        final_state: usize,
        group: usize,
        channel: usize,
    },
    MergedChannel {
        final_state: usize,
        group: usize,
        channel: usize,
        pairs: usize,
    },
}

/// One phase-space channel's share of the integral.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ChannelResult {
    pub(crate) key: ChannelKeyObject,
    /// The channel selection weight its term carries.
    pub(crate) alpha: f64,
    /// Evaluations per iteration the channel received.
    pub(crate) neval: usize,
    pub(crate) sigma: f64,
    pub(crate) sigma_err: f64,
    pub(crate) chi2_per_dof: Option<f64>,
}

/// One final-state phase space's total, at proton beams.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct MultiplicityResult {
    /// Outgoing legs.
    pub(crate) final_state: usize,
    /// The process numbers (`@N`) the part integrates.
    pub(crate) process_ids: Vec<u32>,
    /// The outgoing masses (GeV) its phase space is built for.
    pub(crate) final_masses: Vec<f64>,
    /// How many of the run's channels belong to it.
    pub(crate) channels: usize,
    pub(crate) sigma: f64,
    pub(crate) sigma_err: f64,
    pub(crate) chi2_per_dof: Option<f64>,
}

/// `integrate`'s final object, and `inspect`'s for the artifact it reads.
///
/// Every field but `observable` and `unit` is read from the artifact. Those two
/// are the run's integrand's for `integrate`, and for `inspect` the recorded
/// proc card's (`integrate::recorded_observable`), which name the same
/// observable; so `inspect` of a run's artifact emits the object the run
/// emitted.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct IntegrateResult {
    /// The process string the run integrated.
    pub(crate) process: String,
    pub(crate) model: ModelRef,
    pub(crate) observable: ObservableName,
    /// `pb` for a cross section, `GeV` for a width: the unit of every `sigma`
    /// and `sigma_err` in the object.
    pub(crate) unit: &'static str,
    pub(crate) sigma: f64,
    pub(crate) sigma_err: f64,
    /// The iteration χ²/dof; `null` where fewer than two iterations entered.
    pub(crate) chi2_per_dof: Option<f64>,
    pub(crate) mu_f: MuF,
    /// The integration seed.
    pub(crate) seed: u64,
    /// Each final-state phase space's total at proton beams, in part order;
    /// empty for fixed-energy beams and decays.
    pub(crate) multiplicities: Vec<MultiplicityResult>,
    /// Every channel, in the artifact's channel order.
    pub(crate) channels: Vec<ChannelResult>,
    /// The artifact's path, as the command was given it.
    pub(crate) artifact: String,
    /// The artifact's format version.
    pub(crate) format_version: u32,
    /// The SHA-256 of the diagram sets the run integrated: the digest a
    /// `vibegraph enumerate` run of the same card records.
    pub(crate) diagram_set_digest: String,
    /// Whether the run read a param card, whose text the artifact banks.
    pub(crate) param_card: bool,
}

/// `generate`'s final object.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct GenerateResult {
    /// Event records written to the file.
    pub(crate) events_written: u64,
    /// Accepted events drawn from the grids; fewer than written when a strategy
    /// writes an overweight event more than once.
    pub(crate) events_drawn: usize,
    /// The Les Houches file written.
    pub(crate) output: String,
    /// The artifact the sample was drawn from.
    pub(crate) artifact: String,
    pub(crate) observable: ObservableName,
    /// `pb` or `GeV`, the unit of the four values below.
    pub(crate) unit: &'static str,
    /// The cross section the accept/reject sample itself estimates.
    pub(crate) sample_sigma: f64,
    pub(crate) sample_sigma_err: f64,
    /// The integration's, as the artifact records it.
    pub(crate) integration_sigma: f64,
    pub(crate) integration_sigma_err: f64,
    /// `(sample − integration) / √(err_sample² + err_integration²)`.
    pub(crate) pull: f64,
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::{ChannelKeyObject, MuF, Refusal};
    use crate::error::ErrorKind;

    fn v(x: &impl serde::Serialize) -> Value {
        serde_json::to_value(x).unwrap()
    }

    /// The tagged enums' wire shapes, which a consumer reads by name.
    #[test]
    fn the_tagged_objects_serialize_as_documented() {
        assert_eq!(v(&MuF::NoDensities), json!({"type": "no_densities"}));
        assert_eq!(
            v(&MuF::Constant { gev: 91.188 }),
            json!({"type": "constant", "gev": 91.188})
        );
        assert_eq!(
            v(&MuF::PerBeam { gev: [50.0, 60.0] }),
            json!({"type": "per_beam", "gev": [50.0, 60.0]})
        );
        assert_eq!(
            v(&MuF::PerEvent {
                dynamical_scale_choice: 3
            }),
            json!({"type": "per_event", "dynamical_scale_choice": 3})
        );
        assert_eq!(
            v(&ChannelKeyObject::MergedChannel {
                final_state: 4,
                group: 1,
                channel: 2,
                pairs: 3
            }),
            json!({"type": "merged_channel", "final_state": 4, "group": 1, "channel": 2, "pairs": 3})
        );
        assert_eq!(v(&ChannelKeyObject::Whole), json!({"type": "whole"}));
        assert_eq!(
            v(&Refusal {
                error_kind: ErrorKind::Support,
                message: "m".to_string()
            }),
            json!({"error_kind": "support", "message": "m"})
        );
    }

    /// Full precision: the value a reader parses is the value written, bit for
    /// bit, where a text mode would have rounded it.
    #[test]
    fn values_round_trip_at_full_precision() {
        let x = 934.420_000_000_123_4_f64;
        let text = serde_json::to_string(&MuF::Constant { gev: x }).unwrap();
        let back: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(back["gev"].as_f64().unwrap().to_bits(), x.to_bits());
    }
}
