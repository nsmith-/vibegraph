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
    CouplingConstraint, CouplingOp, MultiparticleDef, ParsedProcCard, ParsingOptions, ProcessSpec,
};

use std::path::Path;

use feyngraph::diagram::DiagramContainer;
use thiserror::Error;

use crate::ufo::UFOModel;

use alias::expand_process;
use parse::{parse_proc_card as inner_parse_proc_card, parse_process_string as inner_parse};
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
    Ok(inner_parse_proc_card(&content, opts)?)
}

/// Parse a `proc_card.dat` from a string.
pub fn parse_proc_card(
    content: &str,
    opts: &ParsingOptions,
) -> Result<ParsedProcCard, DiagramError> {
    Ok(inner_parse_proc_card(content, opts)?)
}

/// Parse a single MadGraph process string (e.g. `"p p > e+ e- j QCD<=2 @1"`).
pub fn parse_process_string(s: &str, opts: &ParsingOptions) -> Result<ProcessSpec, DiagramError> {
    Ok(inner_parse(s, opts)?)
}

/// Build an `AliasTable` seeded with default SM aliases plus the `define` commands
/// from a parsed proc_card.
pub fn build_alias_table(defines: &[MultiparticleDef]) -> AliasTable {
    AliasTable::from_defines(defines)
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
    let aliases = build_alias_table(&proc_card.defines);
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
pub fn generate_from_process_spec(
    spec: &ProcessSpec,
    model: &UFOModel,
    aliases: &AliasTable,
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

    // Deduplicate by topology fingerprint: group DiagramSets with identical
    // propagator PDG signatures and keep one representative per group
    deduplicate_by_topology(sets)
}

/// Deduplicate diagram sets by topology fingerprint.
/// Two sets are considered duplicates if they have identical topology
/// (i.e., same propagators with same PDG codes in each diagram).
fn deduplicate_by_topology(sets: Vec<DiagramSet>) -> Result<Vec<DiagramSet>, DiagramError> {
    use std::collections::HashMap;

    let mut topology_groups: HashMap<Vec<Vec<i32>>, DiagramSet> = HashMap::new();

    for set in sets {
        // Compute topology fingerprint for this set
        let fingerprint = compute_fingerprint(&set);

        // Keep only the first representative for each topology
        topology_groups.entry(fingerprint).or_insert(set);
    }

    Ok(topology_groups.into_values().collect())
}

/// Compute a topology fingerprint: for each diagram, collect and sort the
/// propagator PDG codes, then return the list of these sorted PDG lists.
fn compute_fingerprint(set: &DiagramSet) -> Vec<Vec<i32>> {
    let mut fingerprint = Vec::new();

    for diagram in set.diagrams.views() {
        let mut pdg_codes: Vec<i32> = diagram
            .propagators()
            .map(|prop| prop.particle().pdg() as i32)
            .collect();
        pdg_codes.sort();
        fingerprint.push(pdg_codes);
    }

    fingerprint.sort();
    fingerprint
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sm_ufo_path() -> std::path::PathBuf {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        std::path::Path::new(&manifest).join("../research/refs/mg5amcnlo/models/sm")
    }

    #[test]
    fn test_parse_simple_process() {
        let opts = ParsingOptions::default();
        let spec = parse_process_string("e+ e- > mu+ mu-", &opts).unwrap();
        assert_eq!(spec.initial.len(), 2);
        assert_eq!(spec.final_state.len(), 2);
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
        let table = build_alias_table(&parsed.defines);
        assert_eq!(table.expand_name("myp"), vec!["u", "d"]);
    }

    /// Integration: generate e+ e- > mu+ mu- against SM UFO.
    /// Skipped if the SM UFO model is not present.
    #[test]
    fn test_generate_ee_to_mumu() {
        let path = sm_ufo_path();
        if !path.exists() {
            eprintln!("SM UFO not found — skipping integration test");
            return;
        }
        let model = UFOModel::load(&path, None).expect("SM UFO load failed");
        let opts = ParsingOptions::default();
        let spec = parse_process_string("e+ e- > mu+ mu-", &opts).unwrap();
        let aliases = AliasTable::default_sm();
        let sets =
            generate_from_process_spec(&spec, &model, &aliases).expect("diagram generation failed");
        assert_eq!(sets.len(), 1, "should be exactly 1 concrete process");
        // At LO in QED there is exactly 1 tree-level diagram for e+ e- > mu+ mu-.
        let n = sets[0].diagrams.len();
        assert!(n >= 1, "expected at least 1 diagram, got {n}");
    }
}
