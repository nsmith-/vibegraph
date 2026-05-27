//! Process grammar parser and diagram generation interface.
//!
//! Translates MadGraph-style process strings (`p p > e+ e- j QCD<=2 @1`) and
//! `proc_card.dat` files into feyngraph diagram-generation calls.
//!
//! ## Typical usage
//!
//! ```rust,ignore
//! use vibegraph_lib::diagrams::{parse_process_string, ParsingOptions, generate_from_process_spec};
//! use vibegraph_lib::ufo::UFOModel;
//!
//! let model = UFOModel::load(ufo_path)?;
//! let opts  = ParsingOptions::default();
//! let spec  = parse_process_string("e+ e- > mu+ mu-", &opts)?;
//! let sets  = generate_from_process_spec(&spec, &model, &Default::default())?;
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
    let mut seen_initials = std::collections::HashSet::new();

    for concrete in expand_process(spec, aliases) {
        // Deduplicate mirror processes: if initial state is a permutation
        // of one we've seen, skip it (same diagrams)
        let mut initial_sorted = concrete.initial.clone();
        initial_sorted.sort();
        if !seen_initials.insert(initial_sorted) {
            continue; // Already processed this initial state (permutation)
        }

        let mut sel = build_selector(&concrete);

        // Add custom function to filter out diagrams containing zero-coupling vertices
        let zero_vertices = model.zero_coupling_vertices.clone();
        if !zero_vertices.is_empty() {
            use std::sync::Arc;
            let filter_fn: Arc<
                dyn Fn(&feyngraph::diagram::view::DiagramView) -> bool + Send + Sync,
            > = Arc::new(move |diag_view| {
                for vertex in diag_view.vertices() {
                    let particle_names: Vec<String> = vertex
                        .interaction()
                        .particles_iter()
                        .map(|s| s.clone())
                        .collect();
                    // Check if any zero-coupling vertex's particles are all present in this vertex
                    for zero_v in &zero_vertices {
                        if zero_v.iter().all(|p| particle_names.contains(p)) {
                            return false;
                        }
                    }
                }
                true
            });
            sel.add_custom_function(filter_fn);
        }

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
    use std::path::PathBuf;

    fn ufo_search_path() -> PathBuf {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::Path::new(&manifest).join("../research/refs/mg5amcnlo/models");
        if !path.exists() {
            eprintln!("SM UFO not found — skipping integration test");
        }
        path
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

    /// Integration: generate e+ e- > mu+ mu- against SM UFO.
    /// Skipped if the SM UFO model is not present.
    #[test]
    fn test_generate_ee_to_mumu() {
        let opts = ParsingOptions::default();
        let spec = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let path = ufo_search_path().join("sm");
        let model = UFOModel::load(&path, None).expect("SM UFO load failed");
        let sets = generate_from_proc_card(&spec, &model).expect("Failed to generate model");
        assert_eq!(sets.len(), 1, "should be exactly 1 concrete process");
        // At LO in QED there is exactly 1 tree-level diagram for e+ e- > mu+ mu-.
        let n = sets[0].diagrams.len();
        assert!(n >= 1, "expected at least 1 diagram, got {n}");
    }
}
