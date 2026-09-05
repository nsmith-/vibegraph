#![allow(dead_code)]

pub mod leshouche;
pub mod manifest;
pub mod pdfset;
pub mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, DiagramSet, ParsingOptions};
use vibegraph::ufo::sm::{sm_model as interned_sm, SMRestrict};
use vibegraph::ufo::UFOModel;

pub fn ufo_models_dir() -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest).join("../research/refs/mg5amcnlo/models")
}

pub fn ufo_path(model: &str) -> std::path::PathBuf {
    ufo_models_dir().join(model)
}

pub fn sm_model() -> Arc<UFOModel> {
    interned_sm(SMRestrict::Default)
}

/// SM loaded with the `lepton_masses` restriction (`import model sm-lepton_masses`),
/// which keeps Me/MM/MTA non-zero — unlike `restrict_default`, which locks them to
/// zero. Use this when a test needs settable, physical lepton masses.
pub fn sm_lepton_masses_model() -> Arc<UFOModel> {
    interned_sm(SMRestrict::LeptonMasses)
}

pub fn generate(process: &str) -> Vec<DiagramSet> {
    generate_with(process, sm_model().as_ref())
}

pub fn generate_with(process: &str, model: &UFOModel) -> Vec<DiagramSet> {
    let opts = ParsingOptions::default();
    let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
    generate_from_proc_card(&card, model).unwrap()
}

/// The accepted-point floor as a run realised it, one line.
///
/// The floor promises coverage in accepted points and is paid for in drawn ones,
/// so both are printed: the per-channel acceptance the correction read, the spend
/// it bought at, the spend an uncapped correction would have asked for, and the
/// coverage and zero-variance-iteration counts that say whether the promise was
/// kept.
pub fn floor_coverage_line(spend: &vibegraph::budget::ConvergenceReport) -> String {
    use vibegraph::budget::{MAX_FLOOR_ACCEPTANCE_SCALE, MIN_CHANNEL_NEVAL};

    let n = spend.channel_points.len();
    let acceptance: Vec<f64> = spend
        .channel_accepted
        .iter()
        .zip(&spend.channel_points)
        .map(|(&a, &p)| if p > 0 { a as f64 / p as f64 } else { 0.0 })
        .collect();
    let mut sorted = acceptance.clone();
    sorted.sort_by(f64::total_cmp);
    let q = |p: f64| sorted[(((sorted.len() - 1) as f64) * p).round() as usize];
    let dead = acceptance.iter().filter(|&&a| a == 0.0).count();
    // What the correction would have asked for with no cap on it: the number the
    // cap exists to keep an iteration away from. Channels that accepted nothing
    // have no finite ask at all, so they are counted separately rather than
    // folded in as an infinity.
    let uncapped: f64 = acceptance
        .iter()
        .filter(|&&a| a > 0.0)
        .map(|&a| (MIN_CHANNEL_NEVAL as f64 / a).ceil())
        .sum();
    format!(
        "{n} channels | acceptance min {:.4} p10 {:.4} p50 {:.4} p90 {:.4} max {:.4} \
         | zero-acceptance {dead} | capped (<1/{MAX_FLOOR_ACCEPTANCE_SCALE}) {} \
         | floor spend {MIN_CHANNEL_NEVAL}×n {} → realised {}/iter (uncapped floors would ask {:.3e}) \
         | min accepted/channel/iter {} | zero-variance kept iters {} \
         | points {} | achieved_rel {:.5} scaled_rel {:.5} | floor-bound channels {}",
        q(0.0),
        q(0.10),
        q(0.50),
        q(0.90),
        q(1.0),
        spend.floor_capped_channels,
        n * MIN_CHANNEL_NEVAL,
        spend.points_per_iteration,
        uncapped,
        spend.min_channel_accepted,
        spend.zero_variance_iterations,
        spend.points,
        spend.achieved_rel,
        spend.scaled_rel,
        spend.floor_bound_channels,
    )
}

/// The model a manifest row's gate evaluates in: the interned Standard Model
/// where the row names none, and the vendored UFO directory under its restrict
/// card where it does.
///
/// The `Err` is the message the informational cell carries. A row whose model
/// this crate cannot read yet is exactly why the SMEFTsim cells are registered
/// informational, so the failure is a measurement to report rather than a
/// condition to hide: it is returned, never unwrapped.
pub fn model_for_row(key: &str) -> Result<Arc<UFOModel>, String> {
    // Reading a UFO directory is not cheap and a sweep asks for the same row's
    // model once per subprocess, so the outcome -- the failure as much as the
    // model -- is kept.
    static CACHE: OnceLock<Mutex<ModelCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(hit) = cache.lock().unwrap().get(key) {
        return hit.clone();
    }
    let loaded = load_model_for_row(key);
    cache
        .lock()
        .unwrap()
        .insert(key.to_string(), loaded.clone());
    loaded
}

/// What [`model_for_row`] remembers: the load's outcome per row key.
type ModelCache = BTreeMap<String, Result<Arc<UFOModel>, String>>;

fn load_model_for_row(key: &str) -> Result<Arc<UFOModel>, String> {
    let Some(row) = manifest::row_models().get(key).cloned() else {
        return Ok(sm_model());
    };
    let dir = row.dir_path();
    let card = row.restrict_card();
    if let Some(card) = card.as_ref() {
        if !card.exists() {
            return Err(format!(
                "no restrict card at {} for `{}`",
                card.display(),
                row.restrict.as_deref().unwrap_or("")
            ));
        }
    }
    UFOModel::load(&dir, card.as_deref()).map_err(|e| {
        format!(
            "cannot load {}{}: {e}",
            row.dir,
            row.restrict
                .as_ref()
                .map(|r| format!("-{r}"))
                .unwrap_or_default()
        )
    })
}

/// The `import model` line of a row's `.mg5` script, as `<dir>-<restrict>`.
///
/// The manifest is what the Rust side reads and the script is what MadGraph
/// read; a gate compares the two so a row cannot silently be generated under one
/// card and checked under another.
pub fn script_model_import(script: &str) -> Option<String> {
    script.lines().find_map(|line| {
        line.split('#')
            .next()
            .unwrap_or("")
            .trim()
            .strip_prefix("import model ")
            .map(|rest| rest.trim().to_string())
    })
}

/// The manifest row a generated subprocess file belongs to.
///
/// MadGraph writes each run into `validation/madgraph/output/<key>/`, and
/// `<key>` is what the manifest keys the row by, so a sweep that walks
/// `SubProcesses/P*/…` recovers the row from the path it is already holding.
pub fn row_key_of(path: &Path) -> String {
    path.ancestors()
        .nth(3)
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string()
}

/// Whether a row's amplitude-level gates assert or only report.
///
/// The colour basis is a factor of the amplitude, not a category of its own, so
/// the colour oracles take their enforcement from the row's `amplitudes` cell:
/// a row that cell declares `info` is measured and printed, and one that
/// declares `gate` -- or declares no mode at all, which is every row that was
/// enforced before any cell was informational -- is asserted.
pub fn amplitudes_enforced(key: &str) -> bool {
    static INFO: OnceLock<BTreeSet<String>> = OnceLock::new();
    let info = INFO.get_or_init(|| {
        manifest::category_modes("amplitudes")
            .into_iter()
            .filter(|(_, mode)| mode == "info")
            .map(|(key, _)| key)
            .collect()
    });
    !info.contains(key)
}

/// Run a comparison whose failure is only being reported, turning a panic into
/// its message.
///
/// A row registered informational is measured against code that does not claim
/// to handle it yet, and "does not handle it" arrives as an `Err` from a loader
/// or as a panic from deeper in — the colour algebra refusing to reduce a
/// structure it has no rule for, say — with equal legitimacy. Both are the
/// measurement. Only the non-enforced path goes through here: a panic on a row
/// that is asserted stays a panic.
pub fn catching_panics<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic payload of an unprintable type".to_string());
        Err(format!("panicked: {message}"))
    })
}
