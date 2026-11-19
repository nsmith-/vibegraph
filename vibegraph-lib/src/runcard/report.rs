//! What a run card resolves to and how each of its parameters is treated,
//! reported without running anything.
//!
//! Two views come from the same lists the parser, the field classification and
//! the cut compiler already hold:
//!
//! - [`describe_card`]: one card's resolved parameters, MadGraph's LO defaults,
//!   whether the card moves each, each parameter's class, and the refusal an
//!   integration would emit for it;
//! - [`classification_table`]: the per-release table of every known parameter,
//!   with no card involved.
//!
//! The refusals are the ones the run-card stages of an integration emit: the
//! parse and resolution refusals, the unimplemented-cut detector and the split
//! into integration and generation settings. A refusal that needs a process
//! (the cut compiler's, the phase-space guards) is outside this view.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Number, Value};

use super::classes::{FieldClass, Stage, FIELD_CLASSES};
pub use super::scan::UnknownKey;
use super::scan::{scan, Scanned};
use super::{param_default, ParamValue, RunCard, RunCardError, PARAM_DEFAULTS};
use crate::cuts::{
    cutuse_refusal, describe, detect_unimplemented, inert_values, pdg_cut_refusal,
    unimplemented_cut_names, unimplemented_cut_refusal, CutError,
};

/// How a parameter is treated, in the three words a consumer acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Read and acted on.
    Honoured,
    /// Not read, and unable to reach the output.
    Ignored,
    /// Refused at some value: every value but the ones the entry's
    /// `refused_values` leaves out.
    Refused,
}

/// The class a parameter has in the code's own list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Consumed,
    IgnoredBenign,
    IgnoredPhysics,
}

/// The stage that reads a parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageName {
    Integrate,
    Generate,
}

impl From<Stage> for StageName {
    fn from(stage: Stage) -> Self {
        match stage {
            Stage::Integrate => StageName::Integrate,
            Stage::Generate => StageName::Generate,
        }
    }
}

/// One parameter of the per-release table.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TableEntry {
    pub name: &'static str,
    pub stage: StageName,
    pub class: Class,
    pub status: Status,
    /// MadGraph's LO default.
    pub mg_default: Value,
    /// For a refused or partly refused parameter, the values it refuses, in
    /// words; `None` when no value is refused.
    pub refused_values: Option<String>,
    /// One line on what the code does with the parameter.
    pub note: String,
}

/// Values a consumed parameter refuses, which no field class records: each
/// is a [`RunCardError`] variant or a cut refusal raised for a named
/// parameter.
const VALUE_REFUSALS: &[(&str, &str)] = &[
    ("lpp1", "any pair of beams other than (1, 1) and (0, 0)"),
    ("lpp2", "any pair of beams other than (1, 1) and (0, 0)"),
    (
        "ebeam1",
        "below the proton mass, 0.938 GeV, on proton beams",
    ),
    (
        "ebeam2",
        "below the proton mass, 0.938 GeV, on proton beams",
    ),
    ("ickkw", "any value outside 0 and 1"),
    ("maxjetflavor", "6 together with ickkw = 1"),
    ("event_norm", "any value outside average, sum and unity"),
    ("iseed", "a negative seed"),
    ("nevents", "a negative event count"),
    ("me_frame", "anything but a list of leg numbers"),
    ("pdlabel", "any built-in parameterisation on proton beams"),
    ("pdlabel1", "a set differing from pdlabel2 on proton beams"),
    ("pdlabel2", "a set differing from pdlabel1 on proton beams"),
    (
        "xqcut",
        "a positive value above the jet pT threshold the card leaves",
    ),
    (
        "use_syst",
        "True on fixed-energy beams, when the card writes it",
    ),
    (
        "systematics_program",
        "a program other than the one this build runs",
    ),
    (
        "pt_min_pdg",
        "a negative code, more than 25 codes, or a light-flavour code MadGraph refuses",
    ),
    (
        "mxx_only_part_antipart",
        "a negative code, more than 25 codes, or a light-flavour code MadGraph refuses",
    ),
    (
        "cutuse",
        "any value but 0 while a jet rank bound is on: ptj1min-ptj4min positive, or \
         ptj1max-ptj4max at or above 0 and below (ebeam1 + ebeam2) / 2",
    ),
];

fn class_text(class: &FieldClass) -> (Class, &'static str) {
    match class {
        FieldClass::Consumed(s) => (Class::Consumed, s),
        FieldClass::IgnoredBenign(s) => (Class::IgnoredBenign, s),
        FieldClass::IgnoredPhysics { why } => (Class::IgnoredPhysics, why),
    }
}

/// The first sentence of `text`, whitespace collapsed and capped at a line.
fn one_line(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let end = flat
        .find(". ")
        .map(|i| i + 1)
        .unwrap_or(flat.len())
        .min(flat.len());
    let sentence = &flat[..end];
    const CAP: usize = 200;
    if sentence.chars().count() <= CAP {
        return sentence.to_string();
    }
    let cut: String = sentence.chars().take(CAP).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{cut}…")
}

/// A value as plain JSON: a number, boolean or string.
fn json_value(v: &ParamValue) -> Value {
    match v {
        ParamValue::Float(x) => Number::from_f64(*x)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(x.to_string())),
        ParamValue::Int(i) => Value::Number((*i).into()),
        ParamValue::Bool(b) => Value::Bool(*b),
        ParamValue::Str(s) | ParamValue::Opaque(s) => Value::String(s.clone()),
    }
}

/// Every known run-card parameter with its treatment, in MadGraph's template
/// order. Built from [`FIELD_CLASSES`], [`PARAM_DEFAULTS`] and the cut
/// compiler's own lists, so it moves with them.
pub fn classification_table() -> Vec<TableEntry> {
    let refusals: BTreeMap<&str, &str> = VALUE_REFUSALS.iter().copied().collect();
    FIELD_CLASSES
        .iter()
        .map(|(name, stage, class)| {
            let (kind, text) = class_text(class);
            let default = param_default(name).expect("a classified parameter has a default");
            let (status, refused_values, note) = if let Some(inert) = inert_values(name) {
                (
                    Status::Refused,
                    Some(format!("any value but: {inert}")),
                    "Parsed but not applied: a value that would cut is a hard error".to_string(),
                )
            } else {
                match kind {
                    Class::IgnoredPhysics => (
                        Status::Refused,
                        Some("any value but MadGraph's default".to_string()),
                        one_line(text),
                    ),
                    Class::IgnoredBenign => (Status::Ignored, None, one_line(text)),
                    Class::Consumed => (
                        Status::Honoured,
                        refusals.get(name).map(|r| (*r).to_string()),
                        format!("Read by {}", one_line(text)),
                    ),
                }
            };
            debug_assert!(!unimplemented_cut_names().contains(name) || status == Status::Refused);
            TableEntry {
                name,
                stage: (*stage).into(),
                class: kind,
                status,
                mg_default: json_value(&default),
                refused_values,
                note,
            }
        })
        .collect()
}

/// One refusal the run-card stages of an integration emit.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Refusal {
    /// Where it arises: `parse` (reading and resolving the card), `cuts` (the
    /// unimplemented-cut detector) or `split` (the integration and generation
    /// settings).
    pub stage: &'static str,
    pub message: String,
}

/// One parameter of a resolved card.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct KeyReport {
    pub name: &'static str,
    pub stage: StageName,
    pub class: Class,
    pub status: Status,
    /// MadGraph's LO default.
    pub mg_default: Value,
    /// The value the card writes, `None` when it has no line for the name.
    pub card_value: Option<Value>,
    /// The line of that value.
    pub line: Option<usize>,
    /// The value the run reads, after MadGraph's own edits of the card.
    pub resolved: Value,
    /// The card writes a value other than MadGraph's default.
    pub moved_by_card: bool,
    /// The resolved value differs from what the card wrote or, with no line,
    /// from the default: MadGraph's edits moved it.
    pub moved_by_resolution: bool,
    /// The refusal an integration emits for this parameter's value.
    pub refusal: Option<String>,
    pub note: String,
}

/// A parameter the process, rather than the card, gives a default.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImpliedDefault {
    pub name: &'static str,
    pub mg_default: Value,
    pub process_default: Value,
    /// The card writes this parameter, so the implied default does not apply.
    pub overridden_by_card: bool,
}

/// A resolved run card.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardReport {
    /// The first refusal an integration of this card emits, `None` when the
    /// card passes every run-card stage.
    pub refusal: Option<Refusal>,
    pub keys: Vec<KeyReport>,
    pub unknown_keys: Vec<UnknownKey>,
    pub warnings: Vec<String>,
    /// Parameters the process moves from MadGraph's LO default in the default
    /// card it would write; empty without a process.
    pub process_defaults: Vec<ImpliedDefault>,
}

/// Resolve a card's text and report every parameter.
///
/// `text` is `None` for the card MadGraph would write for the process, which
/// `process_default` supplies. `decay` selects the decay's settings split.
/// `process_default` is that default card for the process, whose differences
/// from MadGraph's LO template are listed whether or not `text` overrides them.
pub fn describe_card(
    text: Option<&str>,
    decay: bool,
    process_default: Option<&RunCard>,
) -> CardReport {
    let scanned = match text {
        Some(text) => scan(text),
        None => scan_of_default(process_default),
    };
    let text_given = text.is_some();

    let mut refusal = scanned.first_error.as_ref().map(|e| Refusal {
        stage: "parse",
        message: e.to_string(),
    });
    let (resolved, warnings) = resolve(&scanned, &mut refusal);
    if refusal.is_none() {
        refusal = resolved.as_ref().and_then(|rc| stage_refusal(rc, decay));
    }

    let table: BTreeMap<&str, TableEntry> = classification_table()
        .into_iter()
        .map(|e| (e.name, e))
        .collect();
    let context = KeyContext {
        scanned: &scanned,
        resolved: resolved.as_ref(),
        text_given,
        beam_sum: resolved.as_ref().map(|rc| rc.ebeam1 + rc.ebeam2),
        cutuse_refused: resolved
            .as_ref()
            .and_then(cutuse_refusal)
            .map(|e| e.to_string()),
        pdg_refusal: resolved.as_ref().and_then(pdg_cut_refusal),
    };
    let keys = PARAM_DEFAULTS
        .iter()
        .map(|(name, _)| key_report(name, &table[name], &context))
        .collect();
    let process_defaults = process_defaults(process_default, &scanned, text_given);

    CardReport {
        refusal,
        keys,
        unknown_keys: scanned.unknown,
        warnings,
        process_defaults,
    }
}

/// The values of the card MadGraph would write for the process: no lines, no
/// unknown names, no error.
fn scan_of_default(process_default: Option<&RunCard>) -> Scanned {
    let base = process_default.cloned().unwrap_or_default();
    Scanned {
        values: base.values.clone(),
        lines: BTreeMap::new(),
        written: base.written.clone(),
        unknown: Vec::new(),
        first_error: None,
    }
}

/// The resolved card and the notices its resolution logged. A card the strict
/// resolution refuses is resolved again leniently, so the report can read the
/// rest of it, and the refusal is recorded unless `refusal` already holds one.
fn resolve(scanned: &Scanned, refusal: &mut Option<Refusal>) -> (Option<RunCard>, Vec<String>) {
    let (card, warnings) = match RunCard::resolve_values(scanned.values.clone(), true) {
        Ok(resolved) => (Some(resolved.0), resolved.1),
        Err(e) => {
            refusal.get_or_insert(Refusal {
                stage: "parse",
                message: e.to_string(),
            });
            match RunCard::resolve_values(scanned.values.clone(), false) {
                Ok((card, warnings)) => (Some(card), warnings),
                Err(_) => (None, Vec::new()),
            }
        }
    };
    let card = card.map(|mut card| {
        card.written = scanned.written.clone();
        card
    });
    (card, warnings)
}

/// The refusal the cut detector or the settings split gives a resolved card.
fn stage_refusal(rc: &RunCard, decay: bool) -> Option<Refusal> {
    if let Err(e) = detect_unimplemented(rc) {
        return Some(Refusal {
            stage: "cuts",
            message: e.to_string(),
        });
    }
    rc.clone().split_as(decay).err().map(|e| Refusal {
        stage: "split",
        message: e.to_string(),
    })
}

/// What [`key_report`] reads besides the key itself.
struct KeyContext<'a> {
    scanned: &'a Scanned,
    resolved: Option<&'a RunCard>,
    text_given: bool,
    /// `ebeam1 + ebeam2` of the resolved card.
    beam_sum: Option<f64>,
    cutuse_refused: Option<String>,
    pdg_refusal: Option<CutError>,
}

/// One parameter of the resolved card.
fn key_report(name: &'static str, entry: &TableEntry, cx: &KeyContext<'_>) -> KeyReport {
    let default = param_default(name).expect("known name");
    let resolved_value = cx
        .resolved
        .and_then(|rc| rc.values.get(name))
        .or_else(|| cx.scanned.values.get(name))
        .cloned()
        .expect("every parameter has a value");
    let card_value = (cx.text_given && cx.scanned.lines.contains_key(name))
        .then(|| cx.scanned.values[name].clone());
    let moved_by_card = card_value.as_ref().is_some_and(|v| *v != default);
    let moved_by_resolution = match &card_value {
        Some(v) => *v != resolved_value,
        None => resolved_value != default,
    };
    let mut refusal = None;
    if entry.class == Class::IgnoredPhysics && resolved_value != default {
        refusal = Some(
            RunCardError::UnsupportedField {
                name: name.to_string(),
                value: describe(&resolved_value),
                default: describe(&default),
                why: ignored_physics_why(name),
            }
            .to_string(),
        );
    } else if let Some(beam_sum) = cx.beam_sum {
        refusal = unimplemented_cut_refusal(name, &resolved_value, beam_sum).map(|e| e.to_string());
    }
    if name == "cutuse" && cx.cutuse_refused.is_some() {
        refusal = cx.cutuse_refused.clone();
    }
    if let Some(CutError::PdgCutRefused { field, .. }) = &cx.pdg_refusal {
        if *field == name {
            refusal = cx.pdg_refusal.as_ref().map(|e| e.to_string());
        }
    }
    KeyReport {
        name,
        stage: entry.stage,
        class: entry.class,
        status: entry.status,
        mg_default: entry.mg_default.clone(),
        card_value: card_value.as_ref().map(json_value),
        line: cx
            .scanned
            .lines
            .get(name)
            .copied()
            .filter(|_| cx.text_given),
        resolved: json_value(&resolved_value),
        moved_by_card,
        moved_by_resolution,
        refusal,
        note: entry.note.clone(),
    }
}

/// The parameters the process moves from MadGraph's LO template in the default
/// card it would write.
fn process_defaults(
    process_default: Option<&RunCard>,
    scanned: &Scanned,
    text_given: bool,
) -> Vec<ImpliedDefault> {
    let Some(process) = process_default else {
        return Vec::new();
    };
    let template = RunCard::default();
    PARAM_DEFAULTS
        .iter()
        .filter_map(|(name, _)| {
            let base = template.values.get(*name)?;
            let implied = process.values.get(*name)?;
            (base != implied).then(|| ImpliedDefault {
                name,
                mg_default: json_value(base),
                process_default: json_value(implied),
                overridden_by_card: text_given && scanned.lines.contains_key(name),
            })
        })
        .collect()
}

fn ignored_physics_why(name: &str) -> &'static str {
    FIELD_CLASSES
        .iter()
        .find_map(|(n, _, class)| match class {
            FieldClass::IgnoredPhysics { why } if *n == name => Some(*why),
            _ => None,
        })
        .expect("an IgnoredPhysics name")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key<'a>(report: &'a CardReport, name: &str) -> &'a KeyReport {
        report.keys.iter().find(|k| k.name == name).expect(name)
    }

    /// Every recognised name has one table entry, and the entry's status
    /// follows the class and the cut compiler's lists.
    #[test]
    fn the_table_has_one_entry_per_parameter() {
        let table = classification_table();
        assert_eq!(table.len(), PARAM_DEFAULTS.len());
        for (name, _) in PARAM_DEFAULTS {
            let entries: Vec<_> = table.iter().filter(|e| e.name == *name).collect();
            assert_eq!(entries.len(), 1, "{name}");
            assert!(!entries[0].note.is_empty(), "{name}");
            assert!(!entries[0].note.contains('\n'), "{name}");
        }
        for entry in &table {
            match entry.status {
                Status::Refused => assert!(entry.refused_values.is_some(), "{}", entry.name),
                Status::Ignored => assert_eq!(entry.class, Class::IgnoredBenign),
                Status::Honoured => assert_eq!(entry.class, Class::Consumed),
            }
        }
    }

    /// A parameter added to a code list without a table entry fails here: every
    /// name in the cut compiler's list and in the value-refusal list is a table
    /// entry, and every unimplemented cut is refused there.
    #[test]
    fn every_listed_parameter_has_an_entry() {
        let table = classification_table();
        for name in unimplemented_cut_names() {
            let entry = table.iter().find(|e| e.name == *name).expect(name);
            assert_eq!(entry.status, Status::Refused, "{name}");
            assert!(entry.refused_values.as_ref().unwrap().contains("default"));
        }
        for (name, _) in VALUE_REFUSALS {
            let entry = table.iter().find(|e| e.name == *name).expect(name);
            assert_eq!(entry.status, Status::Honoured, "{name}");
            assert!(entry.refused_values.is_some(), "{name}");
        }
        for (name, _, class) in FIELD_CLASSES {
            if matches!(class, FieldClass::IgnoredPhysics { .. }) {
                let entry = table.iter().find(|e| e.name == *name).expect(name);
                assert_eq!(entry.status, Status::Refused, "{name}");
            }
        }
    }

    #[test]
    fn the_first_sentence_is_the_note() {
        assert_eq!(one_line("a  b\n c. d e"), "a b c.");
        assert_eq!(one_line("no stop"), "no stop");
        let long = "word ".repeat(100);
        let line = one_line(&long);
        assert!(line.chars().count() <= 201);
        assert!(line.ends_with('…'));
    }

    /// An empty card reports every key at its default, none moved, no refusal.
    #[test]
    fn an_empty_card_moves_nothing() {
        let report = describe_card(Some(""), false, None);
        assert_eq!(report.refusal, None);
        assert_eq!(report.keys.len(), PARAM_DEFAULTS.len());
        assert!(report.unknown_keys.is_empty());
        for k in &report.keys {
            assert_eq!(k.resolved, k.mg_default, "{}", k.name);
            assert!(!k.moved_by_card && !k.moved_by_resolution, "{}", k.name);
            assert_eq!(k.card_value, None);
            assert_eq!(k.refusal, None, "{}", k.name);
        }
    }

    /// The resolved values are the values `RunCard::parse` holds, key by key,
    /// and the card's own value and line are reported.
    #[test]
    fn the_report_reads_what_the_parser_reads() {
        let text = "  6500 = ebeam1\n  6500 = ebeam2\n  25 = ptj\n  30 = xqcut\n  0.3 = drjj\n";
        let rc = RunCard::parse(text).unwrap();
        let report = describe_card(Some(text), false, None);
        assert_eq!(report.refusal, None);
        for (name, value) in rc.iter() {
            assert_eq!(key(&report, name).resolved, json_value(value), "{name}");
        }
        let ptj = key(&report, "ptj");
        assert_eq!(ptj.card_value, Some(Value::from(25.0)));
        assert_eq!(ptj.line, Some(3));
        assert!(ptj.moved_by_card);
        assert!(ptj.moved_by_resolution, "xqcut moves ptj to 30");
        assert_eq!(ptj.resolved, Value::from(30.0));
        let drjj = key(&report, "drjj");
        assert!(drjj.moved_by_card && drjj.moved_by_resolution);
        assert_eq!(drjj.resolved, Value::from(0.0));
        assert!(!key(&report, "ebeam1").moved_by_card);
        assert_eq!(key(&report, "ebeam1").line, Some(1));
    }

    /// A refused card still reports: the unknown name is listed and the card's
    /// first refusal is the parser's own message.
    #[test]
    fn a_refused_card_still_reports_its_keys() {
        let text = "5 = ptj\n$DEFAULT_PDF_SETS = lhaid\n1 = nonsense_key\n";
        let report = describe_card(Some(text), false, None);
        let parse_error = RunCard::parse(text).unwrap_err().to_string();
        let refusal = report.refusal.as_ref().unwrap();
        assert_eq!(refusal.stage, "parse");
        assert_eq!(refusal.message, parse_error);
        assert_eq!(report.unknown_keys.len(), 2);
        assert_eq!(report.unknown_keys[0].name, "lhaid");
        assert_eq!(report.unknown_keys[0].problem, "placeholder");
        assert_eq!(report.unknown_keys[0].value, "$DEFAULT_PDF_SETS");
        assert_eq!(report.unknown_keys[1].name, "nonsense_key");
        assert_eq!(report.unknown_keys[1].problem, "unknown_name");
        assert_eq!(key(&report, "ptj").resolved, Value::from(5.0));
        assert_eq!(report.keys.len(), PARAM_DEFAULTS.len());
    }

    /// A field refused off its default is refused by the report as the parser
    /// refuses it, and the other keys still resolve.
    #[test]
    fn an_ignored_physics_field_carries_its_refusal() {
        let text = "  1 = lpp1\n  1 = lpp2\n  3 = nb_proton1\n  25 = ptj\n";
        let report = describe_card(Some(text), false, None);
        let error = RunCard::parse(text).unwrap_err().to_string();
        assert_eq!(report.refusal.as_ref().unwrap().message, error);
        let k = key(&report, "nb_proton1");
        assert_eq!(k.refusal.as_deref(), Some(error.as_str()));
        assert_eq!(k.status, Status::Refused);
        assert_eq!(key(&report, "ptj").resolved, Value::from(25.0));
        assert_eq!(key(&report, "ptj").refusal, None);
    }

    /// An active unimplemented cut is refused as `detect_unimplemented` refuses
    /// it, on its own key, while an inert sentinel passes.
    #[test]
    fn an_unimplemented_cut_carries_the_detectors_refusal() {
        let text = "  6500 = ebeam1\n  6500 = ebeam2\n  40 = misset\n  -1 = ptheavy\n";
        let report = describe_card(Some(text), false, None);
        let rc = RunCard::parse(text).unwrap();
        let error = detect_unimplemented(&rc).unwrap_err().to_string();
        let refusal = report.refusal.as_ref().unwrap();
        assert_eq!(
            (refusal.stage, refusal.message.as_str()),
            ("cuts", error.as_str())
        );
        assert_eq!(
            key(&report, "misset").refusal.as_deref(),
            Some(error.as_str())
        );
        assert_eq!(key(&report, "ptheavy").refusal, None);
        assert!(key(&report, "ptheavy").moved_by_card);
    }

    /// The resolution notices ride in `warnings`.
    #[test]
    fn a_resolution_notice_is_a_warning() {
        let report = describe_card(Some("30 = xqcut\n"), false, None);
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("ickkw = 0"));
    }

    /// The process's defaults are the differences between its default card and
    /// the LO template, and a card line overrides them.
    #[test]
    fn process_defaults_are_the_differences_from_the_template() {
        let decay = RunCard::decay_default();
        let report = describe_card(Some("5 = ptj\n"), true, Some(&decay));
        let ptj = report
            .process_defaults
            .iter()
            .find(|d| d.name == "ptj")
            .expect("a decay removes ptj");
        assert_eq!(ptj.process_default, Value::from(0.0));
        assert!(ptj.overridden_by_card);
        let drjj = report
            .process_defaults
            .iter()
            .find(|d| d.name == "drjj")
            .unwrap();
        assert!(!drjj.overridden_by_card);
        assert!(describe_card(Some(""), false, None)
            .process_defaults
            .is_empty());
    }

    /// With no card the report is of the process's default card.
    #[test]
    fn no_card_reports_the_process_default() {
        let decay = RunCard::decay_default();
        let report = describe_card(None, true, Some(&decay));
        assert_eq!(report.refusal, None);
        assert_eq!(key(&report, "ptj").resolved, Value::from(0.0));
        assert!(key(&report, "ptj").moved_by_resolution);
        assert!(!key(&report, "ptj").moved_by_card);
    }

    /// Every committed validation run card reports exactly what `RunCard::parse`
    /// holds, key by key, and the report's refusal is the one the parse and the
    /// cut detector give.
    ///
    /// What this cannot see is a parser error shared by both sides: the report
    /// is built from the parser's own resolution.
    #[test]
    fn the_committed_validation_cards_report_what_the_parser_reads() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph");
        let mut cards: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with("run_card.dat"))
            })
            .collect();
        cards.sort();
        assert!(cards.len() >= 20, "{} cards found", cards.len());
        for path in &cards {
            let text = std::fs::read_to_string(path).unwrap();
            let report = describe_card(Some(&text), false, None);
            let expected = RunCard::parse(&text)
                .map_err(|e| e.to_string())
                .and_then(|rc| detect_unimplemented(&rc).map_err(|e| e.to_string()));
            assert_eq!(
                report.refusal.as_ref().map(|r| r.message.clone()),
                expected.clone().err(),
                "{}",
                path.display()
            );
            assert!(report.unknown_keys.is_empty(), "{}", path.display());
            if let Ok(rc) = RunCard::parse(&text) {
                assert_eq!(report.keys.len(), rc.iter().count());
                for (name, value) in rc.iter() {
                    assert_eq!(
                        key(&report, name).resolved,
                        json_value(value),
                        "{} {name}",
                        path.display()
                    );
                }
            }
        }
    }
}
