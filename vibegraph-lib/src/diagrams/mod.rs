//! Process grammar parser and diagram generation interface.
//!
//! Translates MadGraph-style process strings (`p p > e+ e- j QCD<=2 @1`) and
//! `proc_card.dat` files into feyngraph diagram-generation calls.
//!
//! Particle names are resolved case-insensitively, as in MadGraph: a proc card may
//! spell a leg `z`, `w+`, or `h` where the UFO model names it `Z`, `W+`, `H`. Tokens
//! are canonicalized to the model's casing before diagram generation.
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
pub mod diagram;
pub mod parse;
pub mod selector;

pub use alias::AliasTable;
pub use diagram::{ConvertError, Diagram};
pub use parse::{
    CouplingConstraint, CouplingOp, ModelImport, MultiparticleDef, ParsedProcCard, ParsingOptions,
    ProcessSpec,
};

use std::path::Path;
use std::time::Instant;

use feyngraph::topology::{Topology, TopologyGenerator, TopologyModel};
use feyngraph::DiagramGenerator;
use thiserror::Error;
use tracing::{debug, info, info_span, trace};

use crate::progress;
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
    #[error("diagram conversion error: {0}")]
    Convert(#[from] ConvertError),
    #[error("cannot build the enumeration worker pool: {0}")]
    Pool(#[from] rayon::ThreadPoolBuildError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Output type ───────────────────────────────────────────────────────────────

/// One concrete particle assignment together with its generated diagrams.
pub struct DiagramSet {
    pub particles_in: Vec<String>,
    pub particles_out: Vec<String>,
    pub diagrams: Vec<Diagram>,
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
    let card = inner_parse_proc_card(content, opts)?;
    let _span = info_span!("proc_card").entered();
    for spec in &card.processes {
        info!("generate {spec}");
    }
    for def in &card.defines {
        let except = if def.except.is_empty() {
            String::new()
        } else {
            format!(" / {}", def.except.join(" "))
        };
        debug!("define {} = {}{except}", def.alias, def.particles.join(" "));
    }
    Ok(card)
}

// ── Diagram generation API ────────────────────────────────────────────────────

/// How much of the ambient rayon pool enumeration may spread over.
///
/// feyngraph parallelises the topology search and the per-assignment diagram
/// construction internally, so whatever pool enumeration runs on is the pool
/// those loops use.
///
/// Which one is faster depends on the process: the fan-out is over topologies and
/// particle assignments whose bodies are short and share their output container,
/// so a small enumeration spends more on contention than it saves (`p p > j j j`
/// on 16 threads: 0.14 s against 0.08 s on one), while a large one is dominated by
/// the fan-out and gains (`p p > e+ e- j j j`: 3.4 s against 8.9 s). [`Serial`] is
/// the default because the small case is the common one and the large case is the
/// one worth passing a flag for.
///
/// [`Serial`]: EnumerationPool::Serial
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EnumerationPool {
    /// One thread, whatever the caller's pool is sized to.
    #[default]
    Serial,
    /// The caller's pool, however many threads it was sized to.
    Ambient,
}

/// High-level entry point: parse + expand + generate diagrams for every process
/// in a `ParsedProcCard`, on a single thread.
///
/// Returns one `DiagramSet` per concrete particle assignment across all processes.
pub fn generate_from_proc_card(
    proc_card: &ParsedProcCard,
    model: &UFOModel,
) -> Result<Vec<DiagramSet>, DiagramError> {
    generate_from_proc_card_in(proc_card, model, EnumerationPool::default())
}

/// [`generate_from_proc_card`] with an explicit choice of worker pool.
///
/// The result does not depend on the choice: enumeration order and diagram
/// identity are fixed by the topology and assignment enumeration, not by how the
/// work is scheduled. It is a timing knob only.
pub fn generate_from_proc_card_in(
    proc_card: &ParsedProcCard,
    model: &UFOModel,
    pool: EnumerationPool,
) -> Result<Vec<DiagramSet>, DiagramError> {
    match pool {
        EnumerationPool::Ambient => enumerate(proc_card, model),
        // A pool of its own rather than a serial code path: feyngraph's `par_iter`s
        // are internal, and one thread is the only way to keep them off the
        // caller's pool without forking the enumeration itself.
        EnumerationPool::Serial => rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()?
            .install(|| enumerate(proc_card, model)),
    }
}

fn enumerate(
    proc_card: &ParsedProcCard,
    model: &UFOModel,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let _span = info_span!("enumerate").entered();
    let started = Instant::now();
    let aliases = AliasTable::from_defines(&proc_card.defines);
    let mut sets = Vec::new();
    for spec in &proc_card.processes {
        sets.extend(generate_from_process_spec(spec, model, &aliases)?);
    }
    let diagrams: usize = sets.iter().map(|s| s.diagrams.len()).sum();
    let populated = sets.iter().filter(|s| !s.diagrams.is_empty()).count();
    info!(
        "enumerated {diagrams} diagrams in {populated} subprocess{} ({:.3} s)",
        if populated == 1 { "" } else { "es" },
        started.elapsed().as_secs_f64()
    );
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
    // Generate abstract graph topologies once for this (n_external, n_loops=0) combination.
    // All concrete subprocesses share the same topology set; reusing it avoids re-running
    // the O(n!) topology search for every one of the potentially thousands of particle
    // assignments produced by alias expansion (e.g. p p > q q~ l+ l- l+ l- has ~11k combos).
    let n_ext = spec.initial.len() + spec.final_state.len();
    let cached_topologies = generate_topologies(n_ext, &model.topo);

    if spec.coupling_constraints.is_empty() {
        // No explicit constraints: discover the minimum WEIGHTED order.
        let min_hier = model.order_hierarchy.values().copied().min().unwrap_or(1) as usize;
        let max_hier = model.order_hierarchy.values().copied().max().unwrap_or(2) as usize;
        let min_w = (n_ext - 2) * min_hier;
        let max_w = (n_ext - 2) * max_hier;

        let mut w = min_w;
        loop {
            let sets = generate_sets_inner(spec, model, aliases, Some(w), &cached_topologies)?;
            if sets.iter().any(|s| !s.diagrams.is_empty()) {
                debug!("lowest WEIGHTED order with diagrams: {w}");
                return Ok(sets);
            }
            if w >= max_w {
                debug!("no diagrams at any WEIGHTED order in {min_w}..={max_w}");
                return Ok(sets);
            }
            debug!("no diagrams at WEIGHTED {w}, raising the bound");
            w += 1;
        }
    } else {
        generate_sets_inner(spec, model, aliases, None, &cached_topologies)
    }
}

/// Pre-generate all abstract graph topologies for `n_ext` external legs at tree level.
/// Result is cached by the caller and passed into `generate_sets_inner` to avoid
/// recomputing the topology search (which is O(n!) in the number of internal vertices)
/// for every concrete subprocess.
fn generate_topologies(n_ext: usize, topo_model: &feyngraph::model::Model) -> Vec<Topology> {
    let started = Instant::now();
    let container =
        TopologyGenerator::new(n_ext, 0, TopologyModel::from(topo_model), None).generate();
    debug!(
        "{} tree topologies on {n_ext} legs in {:.3} s",
        container.len(),
        started.elapsed().as_secs_f64()
    );
    (0..container.len())
        .map(|i| container.get(i).clone())
        .collect()
}

/// Inner generation loop: expand aliases, deduplicate mirror processes, and call
/// feyngraph for each concrete subprocess.  `max_weighted` (when `Some`) adds an
/// extra diagram filter that rejects any diagram whose WEIGHTED order exceeds the
/// given bound.
///
/// `cached_topologies` must be pre-computed by the caller via `generate_topologies`.
/// Coupling constraints are enforced during particle assignment, not topology
/// generation, so no topology filtering is needed here.
fn generate_sets_inner(
    spec: &ProcessSpec,
    model: &UFOModel,
    aliases: &AliasTable,
    max_weighted: Option<usize>,
    cached_topologies: &[Topology],
) -> Result<Vec<DiagramSet>, DiagramError> {
    let mut sets = Vec::new();
    // Candidate assignments seen, and the two prefilters' kills: what the
    // enumeration paid for the assignments it never handed to feyngraph.
    let mut candidates = 0usize;
    let mut duplicates = 0usize;
    let mut charge_kills = 0usize;
    // Deduplicate on (sorted initial, sorted final): a concrete subprocess is
    // identified by the *unordered* content of each side, so a card whose
    // final-state slots draw on intersecting alias sets (`p p > j j`) yields
    // `g u > g u` once rather than once per ordering.  `g u > u g` is the same
    // subprocess: dPhi_n is integrated over the whole labelled region and every
    // run-card cut is a per-class one, so a permutation of the outgoing legs
    // relabels the integral without moving it, and enumerating both would add
    // its term twice.  Sorting is in the key only — the surviving representative
    // keeps the order the expansion emitted it in.
    //
    // Distinct final-state *content* never collapses, which is what
    // deduplicating on the initial state alone would get wrong: it would
    // silently drop subprocesses like `g d > e+ e- d` when the first
    // final-state combo tried for that initial (e.g. `g d > e+ e- g`) has no
    // diagrams at the active WEIGHTED bound.
    let mut seen_processes: std::collections::HashSet<(Vec<String>, Vec<String>)> =
        std::collections::HashSet::new();

    // MadGraph resolves particle names case-insensitively, so a proc card may spell
    // a leg `z`, `w+`, or `h` where the UFO model names it `Z`, `W+`, `H`. Canonicalize
    // each token to the model's casing before it reaches the charge check or feyngraph.
    // An exact match wins; a token with no case-insensitive match is left unchanged so
    // feyngraph still reports a genuinely unknown particle.
    let canon: std::collections::HashMap<String, &str> = model
        .particles
        .keys()
        .map(|k| (k.to_lowercase(), k.as_str()))
        .collect();
    let canonicalize = |name: &str| -> String {
        if model.particles.contains_key(name) {
            name.to_owned()
        } else {
            canon
                .get(&name.to_lowercase())
                .map(|s| (*s).to_owned())
                .unwrap_or_else(|| name.to_owned())
        }
    };

    for concrete in expand_process(spec, aliases) {
        candidates += 1;
        let mut concrete = concrete;
        for n in concrete
            .initial
            .iter_mut()
            .chain(concrete.final_state.iter_mut())
            .chain(concrete.forbidden_particles.iter_mut())
        {
            *n = canonicalize(n);
        }

        let mut initial_sorted = concrete.initial.clone();
        initial_sorted.sort();
        let mut final_sorted = concrete.final_state.clone();
        final_sorted.sort();
        if !seen_processes.insert((initial_sorted, final_sorted)) {
            duplicates += 1;
            continue;
        }

        // Charge conservation: skip subprocesses that can't conserve electric charge.
        // This fast O(n) check prunes the majority of alias-expanded candidates before
        // the expensive topology-assignment step (e.g. ~90% of pp→qq~4l subprocesses).
        let particle_charge =
            |name: &str| -> f64 { model.particles.get(name).map(|p| p.charge).unwrap_or(0.0) };
        let q_in: f64 = concrete.initial.iter().map(|p| particle_charge(p)).sum();
        let q_out: f64 = concrete
            .final_state
            .iter()
            .map(|p| particle_charge(p))
            .sum();
        if (q_in - q_out).abs() > 1e-6 {
            charge_kills += 1;
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
        let started = Instant::now();

        let generator =
            DiagramGenerator::new(&in_refs, &out_refs, 0, model.topo.clone(), Some(sel))?;
        // assign_topologies only errors on n_external/n_loops mismatch, which can't
        // happen here since cached_topologies was built for the same n_ext and n_loops=0.
        let container = generator
            .assign_topologies(cached_topologies)
            .expect("topology cache n_external/n_loops mismatch — impossible by construction");

        // Module boundary: convert feyngraph's borrowed views into owned, UFO-resolved
        // diagrams here and drop the container. feyngraph views never escape `diagrams/`.
        let diagrams = container
            .views()
            .map(|view| Diagram::from_view(&view, model))
            .collect::<Result<Vec<_>, _>>()?;

        let subprocess = format!("{} > {}", in_refs.join(" "), out_refs.join(" "));
        if !diagrams.is_empty() {
            info!("{} diagrams for {subprocess}", diagrams.len());
            debug!(
                "{subprocess}: enumerated in {:.3} s",
                started.elapsed().as_secs_f64()
            );
            report_vertex_assignments(&subprocess, &diagrams, model);
        }

        sets.push(DiagramSet {
            particles_in: concrete.initial,
            particles_out: concrete.final_state,
            diagrams,
        });
        progress::step(progress::stage::ENUMERATE, sets.len() as u64, None);
    }

    // Reported only for the pass that found something: under the automatic
    // WEIGHTED search the same expansion is walked once per candidate order, and
    // the prefilter counts are identical every time.
    if sets.iter().any(|s| !s.diagrams.is_empty()) {
        debug!(
            "{candidates} alias-expanded assignments: {duplicates} duplicate, {charge_kills} \
             charge-violating, {} enumerated",
            sets.len()
        );
    }

    Ok(sets)
}

/// Every diagram's vertices and internal lines, one line each.
///
/// Gated on the level rather than left to the macro because rendering a diagram
/// costs a string per vertex and per propagator, and a wide process has thousands
/// of them.
fn report_vertex_assignments(subprocess: &str, diagrams: &[Diagram], model: &UFOModel) {
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }
    for (d, diagram) in diagrams.iter().enumerate() {
        let vertices: Vec<&str> = diagram
            .vertices
            .iter()
            .map(|v| model.vertex_def(v.interaction).name.as_str())
            .collect();
        let props: Vec<&str> = diagram
            .props
            .iter()
            .map(|p| model.particle(p.particle).name.as_str())
            .collect();
        trace!(
            "{subprocess} diagram {}: vertices [{}], internal [{}]",
            d + 1,
            vertices.join(" "),
            props.join(" ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// The pool is a scheduling choice and nothing else: enumeration order and
    /// diagram content are fixed by the topology and assignment enumeration.
    #[test]
    fn the_pool_does_not_change_what_is_enumerated() {
        let model = sm_model(SMRestrict::Default);
        let card = parse_proc_card("generate u u~ > g g g", &ParsingOptions::default()).unwrap();
        let serial = generate_from_proc_card_in(&card, &model, EnumerationPool::Serial).unwrap();
        let ambient = generate_from_proc_card_in(&card, &model, EnumerationPool::Ambient).unwrap();

        assert_eq!(serial.len(), ambient.len());
        for (s, a) in serial.iter().zip(&ambient) {
            assert_eq!(s.particles_in, a.particles_in);
            assert_eq!(s.particles_out, a.particles_out);
            assert!(!s.diagrams.is_empty());
            let render = |d: &[Diagram]| d.iter().map(|d| format!("{d:?}")).collect::<Vec<_>>();
            assert_eq!(render(&s.diagrams), render(&a.diagrams));
        }
    }
}
