//! Process grammar parser and diagram generation interface.
//!
//! Translates MadGraph-style process strings (`p p > e+ e- j QCD<=2 @1`) and
//! `proc_card.dat` files into feyngraph diagram-generation calls.
//!
//! ## Typical usage
//!
//! ```rust,ignore
//! use vibegraph::diagrams::{parse_proc_card, ParsingOptions, generate_from_proc_card};
//! use vibegraph::ufo::UFOModel;
//!
//! let model = UFOModel::load(ufo_path, None).expect("failed to load UFO model");
//! let opts  = ParsingOptions::default();
//! let card  = parse_proc_card("generate e+ e- > mu+ mu-", &opts).expect("failed to parse process");
//! let sets  = generate_from_proc_card(&card, &model).expect("diagram generation failed");
//! println!("{} diagram sets generated", sets.len());
//! ```

pub mod alias;
pub mod parse;
pub mod selector;

pub use alias::AliasTable;
pub use parse::{
    CouplingConstraint, CouplingOp, ModelImport, MultiparticleDef, ParsedProcCard, ParsingOptions,
    ProcessSpec,
};

use std::path::Path;

use feyngraph::diagram::DiagramContainer;
use thiserror::Error;

use crate::ufo::UFOModel;

use alias::expand_process;
use parse::parse_proc_card as inner_parse_proc_card;
use selector::build_selector;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum DiagramError {
    #[error("Process parse error: {0}")]
    Parse(#[from] parse::ParseError),
    #[error("Unknown particle '{0}'")]
    UnknownParticle(String),
    #[error("feyngraph error: {0}")]
    FeynGraph(#[from] feyngraph::model::ModelError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Output type ───────────────────────────────────────────────────────────────

/// One concrete particle assignment together with its generated diagrams.
pub struct DiagramSet {
    pub particles_in: Vec<String>,
    pub particles_out: Vec<String>,
    pub diagrams: DiagramContainer,
}

// ── Public parsing API ────────────────────────────────────────────────────────

/// Parse a `proc_card.dat` file from disk.
pub fn parse_proc_card_file(
    path: &Path,
    opts: &ParsingOptions,
) -> Result<ParsedProcCard, DiagramError> {
    let content = std::fs::read_to_string(path)?;
    parse_proc_card(content.as_str(), opts)
}

/// Parse a `proc_card.dat` from a string.
pub fn parse_proc_card(
    content: &str,
    opts: &ParsingOptions,
) -> Result<ParsedProcCard, DiagramError> {
    Ok(inner_parse_proc_card(content, opts)?)
}

// ── Diagram generation API ────────────────────────────────────────────────────

/// High-level entry point: parse + expand + generate diagrams for every process
/// in a `ParsedProcCard`.
///
/// Returns one `DiagramSet` per concrete particle assignment across all processes.
pub fn generate_from_proc_card(
    proc_card: &ParsedProcCard,
    model: &UFOModel,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let aliases = AliasTable::from_defines(&proc_card.defines);
    let mut sets = Vec::new();
    for spec in &proc_card.processes {
        sets.extend(generate_from_process_spec(spec, model, &aliases)?);
    }
    Ok(sets)
}

/// Generate diagrams for a single parsed `ProcessSpec`.
///
/// Expands multiparticle aliases, builds a `DiagramSelector` for each concrete
/// particle assignment, then calls `feyngraph::generate_diagrams`.
///
/// When the process has no explicit coupling constraints, the WEIGHTED coupling
/// order filter is applied automatically: the minimum WEIGHTED value that produces
/// any diagrams is found iteratively, then only diagrams at that value are kept.
/// This mirrors MadGraph's default behaviour of selecting the lowest perturbative
/// order.  WEIGHTED = Σ_i (hierarchy_i × n_i) where hierarchy comes from the
/// UFO `coupling_orders.py` (e.g. QCD→1, QED→2 in the SM).
fn generate_from_process_spec(
    spec: &ProcessSpec,
    model: &UFOModel,
    aliases: &AliasTable,
) -> Result<Vec<DiagramSet>, DiagramError> {
    if spec.coupling_constraints.is_empty() {
        // No explicit constraints: discover the minimum WEIGHTED order.
        let n_ext = spec.initial.len() + spec.final_state.len();
        let min_hier = model.order_hierarchy.values().copied().min().unwrap_or(1) as usize;
        let max_hier = model.order_hierarchy.values().copied().max().unwrap_or(2) as usize;
        let min_w = (n_ext - 2) * min_hier;
        let max_w = (n_ext - 2) * max_hier;

        let mut w = min_w;
        loop {
            let sets = generate_sets_inner(spec, model, aliases, Some(w))?;
            if sets.iter().any(|s| !s.diagrams.is_empty()) {
                return Ok(sets);
            }
            if w >= max_w {
                return Ok(sets);
            }
            w += 1;
        }
    } else {
        generate_sets_inner(spec, model, aliases, None)
    }
}

/// Inner generation loop: expand aliases, deduplicate mirror processes, and call
/// feyngraph for each concrete subprocess.  `max_weighted` (when `Some`) adds an
/// extra diagram filter that rejects any diagram whose WEIGHTED order exceeds the
/// given bound.
fn generate_sets_inner(
    spec: &ProcessSpec,
    model: &UFOModel,
    aliases: &AliasTable,
    max_weighted: Option<usize>,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let mut sets = Vec::new();
    // Deduplicate on (sorted_initial, final_state): skip only when both the
    // initial state (as an unordered set) AND the final state are identical to
    // a previously-seen process.  Deduplicating on the initial state alone
    // would silently drop subprocesses like `g d > e+ e- d` when the first
    // final-state combo tried for that initial (e.g. `g d > e+ e- g`) has no
    // diagrams at the active WEIGHTED bound.
    let mut seen_processes: std::collections::HashSet<(Vec<String>, Vec<String>)> =
        std::collections::HashSet::new();

    for concrete in expand_process(spec, aliases) {
        let mut initial_sorted = concrete.initial.clone();
        initial_sorted.sort();
        let key = (initial_sorted, concrete.final_state.clone());
        if !seen_processes.insert(key) {
            continue;
        }

        let mut sel = build_selector(&concrete);

        // WEIGHTED coupling-order filter: reject diagrams whose weighted sum exceeds max_weighted.
        if let Some(max_w) = max_weighted {
            use std::sync::Arc;
            let hierarchy = model.order_hierarchy.clone();
            let weighted_fn: Arc<
                dyn Fn(&feyngraph::diagram::view::DiagramView) -> bool + Send + Sync,
            > = Arc::new(move |diag_view| {
                let w: usize = hierarchy
                    .iter()
                    .map(|(coupling, &h)| diag_view.order(coupling) * h as usize)
                    .sum();
                w <= max_w
            });
            sel.add_custom_function(weighted_fn);
        }

        let in_refs: Vec<&str> = concrete.initial.iter().map(String::as_str).collect();
        let out_refs: Vec<&str> = concrete.final_state.iter().map(String::as_str).collect();

        let diagrams = feyngraph::generate_diagrams(
            &in_refs,
            &out_refs,
            0, // LO tree-level only
            model.topo.clone(),
            sel,
        )?;

        sets.push(DiagramSet {
            particles_in: concrete.initial,
            particles_out: concrete.final_state,
            diagrams,
        });
    }

    Ok(sets)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    static UFO_MODEL: OnceLock<UFOModel> = OnceLock::new();

    fn sm_model() -> &'static UFOModel {
        UFO_MODEL.get_or_init(|| {
            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = std::path::Path::new(&manifest).join("../research/refs/mg5amcnlo/models/sm");
            UFOModel::load(&path, None)
                .expect("SM UFO not found — run: git submodule update --init --recursive")
        })
    }

    fn generate(process: &str) -> Vec<DiagramSet> {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
        let model = sm_model();
        generate_from_proc_card(&card, model).unwrap()
    }

    fn total_diagrams(sets: &[DiagramSet]) -> usize {
        sets.iter().map(|s| s.diagrams.len()).sum()
    }

    #[test]
    fn test_parse_proc_card_string() {
        let card = "generate e+ e- > mu+ mu-\nadd process e+ e- > ta+ ta-\n";
        let opts = ParsingOptions::default();
        let parsed = parse_proc_card(card, &opts).unwrap();
        assert_eq!(parsed.processes.len(), 2);
    }

    #[test]
    fn test_alias_table_built_from_defines() {
        let card = "define myp = u d\ngenerate myp > e+ e-\n";
        let opts = ParsingOptions::default();
        let parsed = parse_proc_card(card, &opts).unwrap();
        let table = AliasTable::from_defines(&parsed.defines);
        assert_eq!(table.expand_name("myp"), vec!["u", "d"]);
    }

    // ── SM diagram-count tests ────────────────────────────────────────────────
    // Expected counts validated against MadGraph5_aMC@NLO reference output.
    // All use automatic WEIGHTED coupling-order selection unless noted.

    /// e+e- → μ+μ-: two s-channel diagrams (γ and Z).
    #[test]
    fn test_generate_ee_to_mumu() {
        let sets = generate("e+ e- > mu+ mu-");
        assert_eq!(sets.len(), 1);
        assert_eq!(
            sets[0].diagrams.len(),
            2,
            "expected γ and Z exchange diagrams"
        );
    }

    /// uu~ → gg: three pure-QCD diagrams (s-channel 3g vertex, t- and u-channel quark).
    #[test]
    fn test_generate_uux_to_gg() {
        let sets = generate("u u~ > g g");
        assert_eq!(total_diagrams(&sets), 3);
    }

    /// gg → uu~: crossing of uu~ → gg, also 3 diagrams.
    #[test]
    fn test_generate_gg_to_uux() {
        let sets = generate("g g > u u~");
        assert_eq!(total_diagrams(&sets), 3);
    }

    /// gg → gg: four pure-QCD diagrams (s-, t-, u-channel gluon + 4-gluon contact).
    #[test]
    fn test_generate_gg_to_gg() {
        let sets = generate("g g > g g");
        assert_eq!(total_diagrams(&sets), 4);
    }

    /// uu~ → dd~: automatic WEIGHTED ordering selects only the QCD (gluon) s-channel diagram.
    /// γ, Z, and W+ exchange (all WEIGHTED=4) are excluded at the minimum WEIGHTED=2.
    #[test]
    fn test_generate_uux_to_ddx_weighted_lo() {
        let sets = generate("u u~ > d d~");
        assert_eq!(
            total_diagrams(&sets),
            1,
            "only s-channel gluon at minimum WEIGHTED order"
        );
        let prop_name = sets[0]
            .diagrams
            .views()
            .next()
            .unwrap()
            .propagators()
            .next()
            .unwrap()
            .particle()
            .name()
            .to_string();
        assert_eq!(prop_name, "g", "single diagram should be s-channel gluon");
    }

    /// uu~ → dd~ QED<=2: explicit constraint admits s-channel g (QED=0) and s-channel
    /// γ, Z plus t-channel W+ via CKM (all QED=2). Higgs exchange is absent because
    /// the light-quark Yukawa couplings are zero in the SM restrict_default.dat.
    #[test]
    fn test_generate_uux_to_ddx_explicit_qed() {
        let sets = generate("u u~ > d d~ QED<=2");
        assert_eq!(
            total_diagrams(&sets),
            4,
            "gluon + photon + Z (s-channel) + W+ (t-channel CKM)"
        );
    }

    /// Required s-channel: e+e- > Z > μ+μ-.
    /// Momentum-flow filtering is not yet implemented in feyngraph (see selector.rs TODO),
    /// so the Z requirement is currently ignored and both γ and Z diagrams are returned.
    #[test]
    #[ignore = "required_s_channel filtering not yet implemented (selector.rs TODO)"]
    fn test_generate_ee_to_mumu_required_z() {
        let sets = generate("e+ e- > Z > mu+ mu-");
        assert_eq!(
            sets[0].diagrams.len(),
            1,
            "only Z exchange when Z is required s-channel"
        );
    }

    /// Forbidden mediators: e+e- → μ+μ- with both γ and Z forbidden gives zero diagrams.
    #[test]
    fn test_no_diagrams_when_both_mediators_forbidden() {
        let sets = generate("e+ e- > mu+ mu- / a / Z");
        assert_eq!(total_diagrams(&sets), 0);
    }

    /// Forbidden propagator: forbidding u as propagator in uu~ → gg removes
    /// the t- and u-channel diagrams, leaving only the s-channel gluon diagram.
    #[test]
    fn test_forbidden_u_propagator_in_uux_to_gg() {
        let sets = generate("u u~ > g g / u");
        assert_eq!(
            total_diagrams(&sets),
            1,
            "only s-channel gluon without quark propagators"
        );
    }
}
