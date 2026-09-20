//! The coupling gate: this crate's evaluated model couplings against MadGraph's
//! own two evaluations of the same model on the same parameter card.
//!
//! # What is compared
//!
//! Each committed table (`validation/madgraph/couplings/<key>.json`) carries one
//! `mg_amplitude` row's couplings from two independent MadGraph sources, plus the
//! `param_card.dat` both were evaluated with:
//!
//! 1. `python` — every coupling the restricted model defines, evaluated by
//!    `models.model_reader.ModelReader` from the UFO expressions themselves.
//! 2. `fortran` — the `COMMON/COUPLINGS/` block of the process's compiled matrix
//!    element, read out of the f2py module after `SETPARA` parsed the same card.
//!    Only the couplings the generated subprocess actually uses are declared
//!    there, and every one of them has been through MadGraph's Python-to-Fortran
//!    writer.
//!
//! This crate evaluates the same model and card a third time, and the trial
//! reports both differences.
//!
//! # Why a coupling-level gate at all
//!
//! The amplitude gate compares `AMP()` and `JAMP()`, where a coupling enters
//! multiplied by kinematics and summed with its neighbours. A coupling that is
//! off in its tenth digit therefore reads there as a spread of unit-modulus
//! per-diagram constants that agree to ten digits and not eleven — a symptom
//! with no name attached. Here the same defect reads as one coupling, by name,
//! with the size of its error, before any kinematics are involved.
//!
//! # The two tolerances, and which side is the arbiter
//!
//! MadGraph's Python is the arbiter: it computes from the model's own
//! expressions with no intermediate transcription, so a disagreement between it
//! and this crate is this crate's, and [`PYTHON_REL_TOL`] gates it. Any
//! expression this crate is known to read differently from Python is named in
//! [`KNOWN_CRATE_DEFECTS`], which is empty.
//!
//! MadGraph's Fortran is *not* an arbiter. Its writer emits the model's derived
//! parameters into `Source/couplings.f` at a fixed number of significant digits,
//! so a coupling built from a long UFO literal arrives short of a few digits
//! against the same model's Python. Those are recorded in
//! [`KNOWN_FORTRAN_DEVIATIONS`] by row, coupling and measured size; any other
//! Fortran/Python gap above [`FORTRAN_REL_TOL`] fails, and a listed entry that
//! stops deviating fails too, so the list cannot outlive what it describes.
//!
//! # Known blind spots
//!
//! - **A rounding both sides share is invisible.** The comparison is between
//!   three evaluations of one model against one card; where the card itself, or
//!   a literal in the UFO, is already the rounded number, all three agree on it
//!   and nothing here notices. This is the same blind spot the amplitude gate
//!   has, and it is why the gate is a comparison of implementations rather than
//!   a statement about physical inputs.
//! - **Couplings the restriction dropped are not compared.** MadGraph's
//!   `RestrictModel` deletes the couplings that vanish under the restrict card
//!   and merges the ones that coincide, so its `coupling_dict` is a subset of
//!   this crate's coupling table. The trial iterates over MadGraph's set; a
//!   coupling only this crate retains is unreachable from any vertex MadGraph
//!   kept, and the diagram gate is what compares the vertex sets.
//! - **Nothing here reaches the running strong coupling.** Both sides are read
//!   at the card's own `aS`; `validate_scale_couplings.rs` is what compares the
//!   constant pool as `aS` moves.

mod common;

use libtest_mimic::{Arguments, Failed, Trial};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::EvaluatedModel;

/// Largest relative gap allowed between this crate's coupling value and
/// MadGraph's Python on the same card.
///
/// Measured over all 41 rows: the suite maximum is 8.85e-15 (40 ulp), on the
/// interned Standard Model's `GC_64`; SMEFTsim's largest restriction, the 355
/// couplings of `ee_to_ttx_smeft`, reaches 6.48e-15 (29 ulp) and the two toy
/// models are exact. Both sides evaluate the same UFO expressions in binary64,
/// so what separates them is association order and the last bits of `sqrt`,
/// `atan` and the complex-power forms — tens of ulp, not the one or two an
/// "exact" comparison would suggest. The bound rides a factor of 11 above the
/// observed maximum: tight enough that a wrong expression cannot hide under it,
/// loose enough that a libm differing in its last bit does not fail the gate.
const PYTHON_REL_TOL: f64 = 1e-13;

/// Largest relative gap at which MadGraph's Fortran and its own Python count as
/// agreeing on a coupling.
///
/// The Fortran side recomputes the coupling from parameters its writer printed,
/// and the arithmetic is the same expression in the same precision, so the floor
/// is far below [`PYTHON_REL_TOL`]'s: 3.00e-16 (1.4 ulp) over every agreeing
/// coupling of every row. The bound sits 33 times above that and six orders of
/// magnitude below the one deviation there is, so it separates the writer's
/// precision loss from arithmetic noise without either being a judgement call.
/// Everything above it is named individually in [`KNOWN_FORTRAN_DEVIATIONS`].
const FORTRAN_REL_TOL: f64 = 1e-14;

/// Where MadGraph's generated Fortran differs from MadGraph's own Python by more
/// than [`FORTRAN_REL_TOL`], with the size measured here.
///
/// Each entry is `(row, coupling, relative deviation)`. The deviation is an
/// upper bound with headroom, not an equality: the entry asserts that the gap is
/// there and no larger, so the list fails both when a new coupling starts
/// deviating and when a listed one stops.
const KNOWN_FORTRAN_DEVIATIONS: &[(&str, &str, f64)] = &[
    // `GC_303 = 2i*gHza/vevhat`, the SMHLOOP h-Z-gamma effective coupling. The
    // UFO writes `gHza` from literals like `0.4583333333333333` (11/24) and
    // MadGraph's Fortran writer prints them at seven significant digits, so the
    // Fortran value is the rounded one. Reproducing it would mean rounding a
    // literal on purpose.
    ("ee_to_zh_smeft", "GC_303", 1.2e-8),
    ("wpwm_to_wpwmz_cw", "GC_303", 1.2e-8),
];

/// Where *this crate* differs from MadGraph's Python by more than
/// [`PYTHON_REL_TOL`], with the size measured here.
///
/// Each entry is `(model, coupling, relative deviation)`, the model named as the
/// manifest names it — `<dir>-<restrict>`, or `sm` for the interned Standard
/// Model — because such a defect belongs to the model's expression and not to
/// the row that happens to use it. Every listed entry must still be deviating,
/// so the list fails when its cause is repaired.
///
/// The list is empty: on every banked row, every coupling MadGraph's Python
/// defines agrees with this crate's within [`PYTHON_REL_TOL`]. It stays in the
/// gate because the alternative to naming a known difference is loosening the
/// tolerance that would otherwise catch it, and because the "must still deviate"
/// rule is what stops an entry from outliving its cause.
const KNOWN_CRATE_DEFECTS: &[(&str, &str, f64)] = &[];

fn tables_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/couplings")
}

fn amplitude_tables_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/madgraph/amplitudes")
}

/// One committed coupling table.
struct Table {
    key: String,
    process: String,
    model: Option<String>,
    restrict: Option<String>,
    param_card: String,
    python: BTreeMap<String, (f64, f64)>,
    fortran: BTreeMap<String, (f64, f64)>,
}

impl Table {
    /// The model this row was generated against, as `<dir>-<restrict>`, or `sm`
    /// where the row names none and so ran against MadGraph's own `sm`.
    fn model_id(&self) -> String {
        match (self.model.as_deref(), self.restrict.as_deref()) {
            (None, _) => "sm".to_owned(),
            (Some(dir), None) => dir.to_owned(),
            (Some(dir), Some(restrict)) => format!("{dir}-{restrict}"),
        }
    }
}

fn complex_map(json: &serde_json::Value, field: &str) -> BTreeMap<String, (f64, f64)> {
    json[field]
        .as_object()
        .unwrap_or_else(|| panic!("the table has no `{field}` object"))
        .iter()
        .map(|(name, z)| {
            let c = z.as_array().unwrap();
            (
                name.clone(),
                (c[0].as_f64().unwrap(), c[1].as_f64().unwrap()),
            )
        })
        .collect()
}

fn card_text(json: &serde_json::Value) -> String {
    json["param_card"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_table(json: &serde_json::Value) -> Table {
    Table {
        key: json["key"].as_str().unwrap().to_owned(),
        process: json["process"].as_str().unwrap().to_owned(),
        model: json["model"].as_str().map(str::to_owned),
        restrict: json["restrict"].as_str().map(str::to_owned),
        param_card: card_text(json),
        python: complex_map(json, "python"),
        fortran: complex_map(json, "fortran"),
    }
}

fn read_json(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()))
}

/// `|a − b| / |b|`, or the absolute gap where the reference value vanishes.
///
/// Comparing complex values by their modulus rather than component-wise is what
/// makes the number a phase-independent statement about the coupling: a coupling
/// that is purely imaginary has a zero real part on both sides, and a relative
/// comparison of two zeros says nothing.
fn deviation(ours: (f64, f64), theirs: (f64, f64)) -> f64 {
    let gap = ((ours.0 - theirs.0).powi(2) + (ours.1 - theirs.1).powi(2)).sqrt();
    let scale = (theirs.0.powi(2) + theirs.1.powi(2)).sqrt();
    if scale > 0.0 {
        gap / scale
    } else {
        gap
    }
}

/// The worst deviation of a comparison, and which coupling carried it.
struct Worst {
    name: String,
    value: f64,
}

impl Worst {
    fn new() -> Worst {
        Worst {
            name: "—".to_owned(),
            value: 0.0,
        }
    }

    fn offer(&mut self, name: &str, value: f64) {
        if value > self.value {
            self.name = name.to_owned();
            self.value = value;
        }
    }
}

fn run_trial(path: PathBuf) -> Result<(), Failed> {
    let table = parse_table(&read_json(&path));
    let name = table.key.as_str();

    // The row's declaration is the manifest's, so a table generated for one
    // model cannot be checked against another.
    let declared_process = common::manifest::mg_amplitude_processes()
        .get(name)
        .cloned()
        .ok_or_else(|| format!("[{name}] no `mg_amplitude` row in the manifest"))?;
    if declared_process != table.process {
        return Err(format!(
            "[{name}] the manifest registers '{declared_process}' and the coupling table \
             was banked for '{}'",
            table.process
        )
        .into());
    }
    let row_model = common::manifest::row_models().get(name).cloned();
    let declared = (
        row_model.as_ref().map(|m| m.dir.clone()),
        row_model.as_ref().and_then(|m| m.restrict.clone()),
    );
    if declared != (table.model.clone(), table.restrict.clone()) {
        return Err(format!(
            "[{name}] the manifest names model {declared:?} and the coupling table was \
             banked under {:?}",
            (&table.model, &table.restrict)
        )
        .into());
    }

    // The card is the whole comparison's common input, and the amplitude gate's
    // table carries its own copy of the same file. Tying them together is what
    // makes this gate a statement about the couplings the amplitude gate ran on.
    let amplitude_table = amplitude_tables_dir().join(format!("{name}.json"));
    if amplitude_table.exists() {
        let theirs = card_text(&read_json(&amplitude_table));
        if theirs != table.param_card {
            return Err(format!(
                "[{name}] the amplitude table and the coupling table carry different \
                 param cards"
            )
            .into());
        }
    }

    let model = common::model_for_row(name)?;
    let card = table
        .param_card
        .parse::<ParamCard>()
        .map_err(|e| format!("[{name}] banked param card: {e:?}"))?;
    let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

    // ── this crate against MadGraph's Python ──────────────────────────────────

    let crate_defects: BTreeMap<&str, f64> = KNOWN_CRATE_DEFECTS
        .iter()
        .filter(|(model_id, _, _)| *model_id == table.model_id())
        .map(|(_, coupling, bound)| (*coupling, *bound))
        .collect();
    let mut vg_worst = Worst::new();
    let mut vg_defects = Vec::new();
    let mut failures = Vec::new();
    for (coupling, &theirs) in &table.python {
        let Some(id) = model.coupling_id(coupling) else {
            failures.push(format!(
                "{coupling}: MadGraph defines it, this crate does not"
            ));
            continue;
        };
        let ours = evaluated.coupling(id);
        let dev = deviation((ours.re, ours.im), theirs);
        let known = crate_defects.get(coupling.as_str()).copied();
        if dev <= PYTHON_REL_TOL {
            vg_worst.offer(coupling, dev);
            if known.is_some() {
                failures.push(format!(
                    "{coupling}: listed as a known crate defect, but this crate and \
                     MadGraph's Python now agree to {dev:.3e}"
                ));
            }
            continue;
        }
        vg_defects.push(format!("{coupling} {dev:.3e}"));
        match known {
            Some(bound) if dev <= bound => {}
            Some(bound) => failures.push(format!(
                "{coupling}: deviation from MadGraph's Python {dev:.3e} exceeds the \
                 recorded {bound:.3e}"
            )),
            None => failures.push(format!(
                "{coupling}: vibegraph {ours:.17e}, MadGraph Python \
                 {:.17e}{:+.17e}i, relative {dev:.3e}",
                theirs.0, theirs.1
            )),
        }
    }
    for coupling in crate_defects.keys() {
        if !table.python.contains_key(*coupling) {
            failures.push(format!(
                "{coupling}: listed as a known crate defect, but the model does not \
                 define it"
            ));
        }
    }

    // ── MadGraph's Fortran against MadGraph's Python ──────────────────────────

    let known: BTreeMap<&str, f64> = KNOWN_FORTRAN_DEVIATIONS
        .iter()
        .filter(|(row, _, _)| *row == name)
        .map(|(_, coupling, bound)| (*coupling, *bound))
        .collect();
    let mut fortran_worst = Worst::new();
    let mut deviating = Vec::new();
    for (coupling, &fortran) in &table.fortran {
        let Some(&python) = table.python.get(coupling) else {
            failures.push(format!(
                "{coupling}: declared in the generated coupl.inc, absent from the \
                 restricted model's coupling_dict"
            ));
            continue;
        };
        let dev = deviation(fortran, python);
        if dev <= FORTRAN_REL_TOL {
            fortran_worst.offer(coupling, dev);
            if known.contains_key(coupling.as_str()) {
                failures.push(format!(
                    "{coupling}: listed as a known Fortran deviation, but Fortran and \
                     Python now agree to {dev:.3e}"
                ));
            }
            continue;
        }
        deviating.push(format!("{coupling} {dev:.3e}"));
        match known.get(coupling.as_str()) {
            Some(&bound) if dev <= bound => {}
            Some(&bound) => failures.push(format!(
                "{coupling}: Fortran/Python deviation {dev:.3e} exceeds the recorded \
                 {bound:.3e}"
            )),
            None => failures.push(format!(
                "{coupling}: MadGraph's Fortran differs from its own Python by \
                 {dev:.3e} and is not a recorded deviation — Fortran \
                 {:.17e}{:+.17e}i, Python {:.17e}{:+.17e}i",
                fortran.0, fortran.1, python.0, python.1
            )),
        }
    }

    for coupling in known.keys() {
        if !table.fortran.contains_key(*coupling) {
            failures.push(format!(
                "{coupling}: listed as a known Fortran deviation, but the generated \
                 coupl.inc does not declare it"
            ));
        }
    }

    println!(
        "  {name}: {} model couplings, worst agreeing gap vs MadGraph Python {:.2e} \
         ({}, {:.0} ulp), crate defects [{}]; {} in the subprocess, worst agreeing \
         Fortran gap {:.2e} ({}), writer deviations [{}]",
        table.python.len(),
        vg_worst.value,
        vg_worst.name,
        vg_worst.value / f64::EPSILON,
        vg_defects.join(", "),
        table.fortran.len(),
        fortran_worst.value,
        fortran_worst.name,
        deviating.join(", "),
    );

    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("[{name}]\n  {}", failures.join("\n  ")).into())
    }
}

/// Every `mg_amplitude` row has a committed coupling table, and no table belongs
/// to a row that is not one.
fn coverage() -> Result<(), Failed> {
    let declared: Vec<String> = common::manifest::mg_amplitude_processes()
        .into_keys()
        .collect();
    let mut banked: Vec<String> = table_paths()
        .iter()
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    banked.sort();
    if declared != banked {
        return Err(format!(
            "the manifest declares {} `mg_amplitude` rows and {} coupling tables are \
             committed; only in the manifest: {:?}; only on disk: {:?}",
            declared.len(),
            banked.len(),
            declared
                .iter()
                .filter(|k| !banked.contains(k))
                .collect::<Vec<_>>(),
            banked
                .iter()
                .filter(|k| !declared.contains(k))
                .collect::<Vec<_>>(),
        )
        .into());
    }
    println!("  {} rows covered", declared.len());
    Ok(())
}

fn table_paths() -> Vec<PathBuf> {
    let dir = tables_dir();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    paths
}

fn main() {
    let args = Arguments::from_args();

    let paths = table_paths();
    assert!(
        !paths.is_empty(),
        "no coupling tables in {} — the committed references are the gate's only input",
        tables_dir().display()
    );

    let mut trials: Vec<Trial> = paths
        .into_iter()
        .map(|p| {
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            Trial::test(name, move || run_trial(p))
        })
        .collect();
    trials.push(Trial::test("every_mg_amplitude_row_is_covered", coverage));

    libtest_mimic::run(&args, trials).exit();
}
