//! Process grammar and diagram generation interface.
//!
//! A `proc_card.dat` goes through three stages before feyngraph sees it:
//!
//! 1. [`parse::parse_proc_card_ast`] reads the whole of MadGraph's process
//!    language into a [`ProcCardAst`](parse::ProcCardAst), dropping nothing;
//! 2. [`check::check_supported`] refuses, all at once, every feature this
//!    generator does not honour, and narrows the card to a [`SupportedCard`];
//! 3. enumeration resolves the names against the model (case-insensitively, as
//!    MadGraph does: a card may spell a leg `z` where the UFO says `Z`) and
//!    generates the diagrams of every concrete subprocess.
//!
//! [`parse_proc_card`] runs the first two.
//!
//! ## Typical usage
//!
//! ```rust,ignore
//! use vibegraph::diagrams::{parse_proc_card, ParsingOptions, generate_from_proc_card};
//! use vibegraph::ufo::UFOModel;
//!
//! let model = UFOModel::load(ufo_path, None).expect("failed to load UFO model");
//! let card  = parse_proc_card("generate e+ e- > mu+ mu-", &ParsingOptions::default())
//!     .expect("failed to parse process");
//! let sets  = generate_from_proc_card(&card, &model).expect("diagram generation failed");
//! println!("{} diagram sets generated", sets.len());
//! ```

pub(crate) mod alias;
pub mod artifact;
mod chain;
#[cfg(test)]
pub(crate) use chain::generate_chain_with_merges;
pub mod check;
pub mod diagram;
pub mod draw;
pub mod parse;
mod reduction;
pub mod resolve;
pub mod schannel;
pub(crate) mod selector;
pub mod squared;

pub use check::{check_supported, SupportedCard, Unsupported};
use check::{AmplitudeOrder, SupportedLeg, SupportedProcess, UnsupportedCard};
use diagram::ConvertError;
pub use diagram::Diagram;
pub(crate) use parse::CouplingOp;
pub use parse::{parse_proc_card_ast, ModelImport};

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use feyngraph::topology::{Topology, TopologyGenerator, TopologyModel};
use feyngraph::DiagramGenerator;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, info_span, trace, warn};

use crate::progress;
use crate::ufo::UFOModel;

use resolve::{
    check_order_name, forbidden_propagator_names, forbidden_s_channel_ids, leg_names,
    leg_polarization_codes, model_aliases, polarizations_unambiguous, required_s_channel_ids,
    ResolveError,
};
use schannel::SChannelFilter;
use selector::{build_selector, ConcreteProcess};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum DiagramError {
    #[error("Process parse error: {0}")]
    Parse(#[from] parse::ParseError),
    #[error("{0}")]
    Unsupported(#[from] UnsupportedCard),
    #[error("{0}")]
    Resolve(#[from] ResolveError),
    #[error(
        "subprocess '{subprocess}' is produced by two process lines ('{first}' and '{second}'); \
         both would be added to the cross section, so the card is refused rather than \
         counting it twice"
    )]
    DuplicateSubprocess {
        subprocess: String,
        first: String,
        second: String,
    },
    #[error("'{process}' has {n_in} initial-state particles; a decay has exactly one")]
    NotADecay { process: String, n_in: usize },
    /// A decay-chain line whose decays cannot be stitched onto its core the way it is
    /// written.
    #[error("decay chain '{process}': {reason}")]
    DecayChain { process: String, reason: String },
    /// MadGraph's `NoDiagramException`: a process line none of whose
    /// subprocesses has a diagram is an error, not an empty contribution.
    #[error("no diagrams for '{process}': no subprocess it describes has a diagram")]
    NoDiagrams { process: String },
    /// Two process lines of one card name different `$` lists. The generated
    /// subprocesses do not record which line produced them, so one veto has to
    /// serve the whole card.
    #[error(
        "'{a}' and '{b}' forbid different on-shell s-channels ('$'); a card whose process \
         lines differ in their '$' list is not supported"
    )]
    MixedOnShellVeto { a: String, b: String },
    /// MadGraph's automatic WEIGHTED search lowers a trial's bound by the QCD
    /// hierarchy per removed gluon, and raises `KeyError` on a model whose
    /// `coupling_orders.py` gives QCD none.
    #[error(
        "'{process}': the automatic WEIGHTED search weighs each final-state gluon by the \
         QCD hierarchy, and the model's coupling orders give QCD none; give the process \
         explicit coupling orders"
    )]
    GluonTrialWithoutQcd { process: String },
    /// Two process lines share a process number and differ in their
    /// squared-order constraints. The diagrams record only the number of the
    /// line they came from, which is what an artifact reader derives each set's
    /// selection from.
    #[error(
        "'{a}' and '{b}' share a process number and differ in their squared-order \
         constraints; give them different process numbers (@N)"
    )]
    MixedSquaredOrders { a: String, b: String },
    /// A recorded diagram set that the squared-order filter of its own line
    /// would not leave unchanged: it was not enumerated from that line by this
    /// build.
    #[error(
        "'{process}': the recorded diagrams are not what the line's squared-order \
         constraints keep; enumerate the proc card again"
    )]
    SquaredOrderMismatch { process: String },
    #[error("feyngraph error: {0}")]
    FeynGraph(#[from] feyngraph::model::ModelError),
    #[error("{0}")]
    Convert(#[from] ConvertError),
    #[error("cannot build the enumeration worker pool: {0}")]
    Pool(#[from] rayon::ThreadPoolBuildError),
    /// feyngraph refused to assign particles to the line's cached topologies.
    #[error("'{subprocess}': feyngraph cannot assign the line's topologies: {why}")]
    TopologyAssignment { subprocess: String, why: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ── Output type ───────────────────────────────────────────────────────────────

/// One concrete particle assignment together with its generated diagrams.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagramSet {
    pub particles_in: Vec<String>,
    pub particles_out: Vec<String>,
    /// Per external leg, incoming first: the helicities a polarized leg is
    /// restricted to, as MadGraph's `NHEL` values in the order the card listed
    /// them, or `None` for a leg summed over all its states.
    pub polarizations: Vec<Option<Vec<i32>>>,
    pub diagrams: Vec<Diagram>,
    /// Whether the process line that produced this set also generated it with
    /// its two incoming legs exchanged, so that the set stands for both beam
    /// orderings (MadGraph's `has_mirror_process`). Never set when the two
    /// incoming legs are the same state. A line written with one ordering
    /// (`u u~ > e+ e-`) generates only that one, and its subprocess carries no
    /// luminosity in the other.
    pub mirrored: bool,
    /// The squared-order selection this subprocess's |M|² is formed with: the
    /// terms `A_a A_b*` whose summed coupling orders the line's squared-order
    /// constraints accept ([`squared`]). `None` for a line without any, whose
    /// |M|² is the whole amplitude squared.
    ///
    /// Not serialized: it is a function of the card line, which every artifact
    /// records, and of the set's own diagrams, so the artifact readers derive
    /// it again ([`squared::annotate_sets`]) and the bytes of a set — its
    /// digest, and every artifact of a card without squared orders — are what
    /// they were before the selection existed.
    #[serde(skip)]
    pub squared_orders: Option<squared::SquaredOrders>,
}

impl DiagramSet {
    /// The same subprocess with its outgoing legs in another order: outgoing leg
    /// `order[i]` of `self` becomes outgoing leg `i`. Every diagram is relabelled
    /// with it, its momenta and fermion sign rebuilt from the graph, so the result
    /// is the enumeration the reordered process line would have produced.
    ///
    /// `None` when `order` is not a permutation of the outgoing legs.
    pub(crate) fn with_final_order(&self, order: &[usize], model: &UFOModel) -> Option<DiagramSet> {
        let n_in = self.particles_in.len();
        let n_out = self.particles_out.len();
        let mut seen = vec![false; n_out];
        if order.len() != n_out
            || order
                .iter()
                .any(|&k| k >= n_out || std::mem::replace(&mut seen[k], true))
        {
            return None;
        }
        // `relabelled` moves outgoing position `i` to `to[i]`.
        let mut to = vec![0; n_out];
        for (i, &k) in order.iter().enumerate() {
            to[k] = i;
        }
        let mut polarizations = self.polarizations[..n_in].to_vec();
        polarizations.extend(order.iter().map(|&k| self.polarizations[n_in + k].clone()));
        Some(DiagramSet {
            particles_in: self.particles_in.clone(),
            particles_out: order
                .iter()
                .map(|&k| self.particles_out[k].clone())
                .collect(),
            polarizations,
            diagrams: self
                .diagrams
                .iter()
                .map(|d| chain::relabelled(d, &to, n_in, model))
                .collect(),
            mirrored: self.mirrored,
            squared_orders: self.squared_orders.clone(),
        })
    }

    /// `u u~ > z{0} g`: the subprocess with each polarized leg's helicities.
    pub fn label(&self) -> String {
        let n_in = self.particles_in.len();
        let side = |names: &[String], offset: usize| {
            names
                .iter()
                .enumerate()
                .map(|(i, name)| match &self.polarizations.get(offset + i) {
                    Some(Some(h)) => format!(
                        "{name}{{{}}}",
                        h.iter().map(i32::to_string).collect::<Vec<_>>().join(",")
                    ),
                    _ => name.clone(),
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        format!(
            "{} > {}",
            side(&self.particles_in, 0),
            side(&self.particles_out, n_in)
        )
    }
}

// ── Public parsing API ────────────────────────────────────────────────────────

/// Options for [`parse_proc_card`].
///
/// There are none to set: every construct is parsed, and [`check_supported`]
/// alone decides what is refused, so no caller can opt into a card whose
/// meaning would be silently narrowed.
#[derive(Debug, Clone, Default)]
pub struct ParsingOptions {}

/// Parse a `proc_card.dat` file from disk and check it.
pub fn parse_proc_card_file(
    path: &Path,
    opts: &ParsingOptions,
) -> Result<SupportedCard, DiagramError> {
    let content = std::fs::read_to_string(path)?;
    parse_proc_card(content.as_str(), opts)
}

/// Parse a `proc_card.dat` from a string and check it: a card with any
/// unsupported feature is refused with every such feature listed.
pub fn parse_proc_card(
    content: &str,
    _opts: &ParsingOptions,
) -> Result<SupportedCard, DiagramError> {
    let ast = parse_proc_card_ast(content)?;
    let card = check_supported(&ast)?;
    // At `debug`: a caller that reads a card from disk announces it at `info`
    // itself, and one reparsing a card an artifact recorded has read none.
    let _span = info_span!("proc_card").entered();
    for process in &card.processes {
        debug!("generate {process} @{}", process.id);
    }
    for command in &ast.commands {
        if let parse::Command::Define(def) = command {
            let groups: Vec<String> = def.groups.iter().map(|g| g.join(" ")).collect();
            let except = if def.except.is_empty() {
                String::new()
            } else {
                format!(" / {}", def.except.join(" "))
            };
            debug!("define {} = {}{except}", def.alias, groups.join(" | "));
        }
    }
    Ok(card)
}

// ── Diagram generation API ────────────────────────────────────────────────────

/// How much of the ambient rayon pool enumeration may spread over.
///
/// A line's walk hands its surviving subprocesses to feyngraph in batches, each
/// batch across the pool, and feyngraph parallelises the topology assignment of
/// each subprocess internally on whatever pool it runs on. The sets come back in
/// the walk's order and the logs are written in it, so the pool changes the
/// timing and nothing else.
///
/// Which one is faster depends on the walk: feyngraph's internal fan-out is over
/// topologies whose bodies are short and allocate, so a small enumeration spends
/// more on contention than it saves (`p p > j j j` on 16 threads: 0.14 s against
/// 0.08 s on one), while a walk of many subprocesses is dominated by them.
/// [`Auto`] decides per walk, from the walk's size before it starts
/// ([`AUTO_PARALLEL_COST`]).
///
/// [`Auto`]: EnumerationPool::Auto
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EnumerationPool {
    /// The caller's pool for a walk whose candidate assignments times its tree
    /// topologies reach [`AUTO_PARALLEL_COST`], one thread for a smaller one.
    #[default]
    Auto,
    /// One thread, whatever the caller's pool is sized to.
    Serial,
    /// The caller's pool, however many threads it was sized to.
    Ambient,
}

/// The size of a walk, its candidate assignments times the tree topologies of
/// its leg count, from which [`EnumerationPool::Auto`] runs it on the caller's
/// pool rather than on one thread.
///
/// Measured at four threads, dev profile: every walk from `p p > e+ e- j`
/// (1331 candidates on 25 topologies, 33 275) up enumerated no slower on the
/// pool than on one thread (`p p > j j`, 58 564: 0.027–0.033 s against
/// 0.042–0.052 s; `p p > e+ e- j j`: 0.09–0.11 s against 0.24 s). Below it the
/// walks measured (`e+ e- > mu+ mu-`, `p p > e+ e-`) take milliseconds either
/// way and stay on one thread.
pub const AUTO_PARALLEL_COST: u64 = 1 << 14;

/// Surviving subprocesses handed to feyngraph per batch, per worker thread.
const BATCH_PER_THREAD: usize = 16;

/// A one-thread pool for the walks [`EnumerationPool::Auto`] keeps serial, built
/// once per process.
fn serial_pool() -> Result<&'static rayon::ThreadPool, DiagramError> {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    if let Some(pool) = POOL.get() {
        return Ok(pool);
    }
    let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build()?;
    Ok(POOL.get_or_init(|| pool))
}

/// High-level entry point: expand and generate diagrams for every process of a
/// checked card, on a single thread.
///
/// Returns one `DiagramSet` per concrete particle assignment across all processes
/// that has at least one diagram; an assignment the model gives no diagram is
/// left out.
pub fn generate_from_proc_card(
    proc_card: &SupportedCard,
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
    proc_card: &SupportedCard,
    model: &UFOModel,
    pool: EnumerationPool,
) -> Result<Vec<DiagramSet>, DiagramError> {
    enumerate_lines_in(proc_card, model, pool).map(|lines| lines.sets)
}

/// A card's diagram sets together with how many of them each process line
/// produced.
#[derive(Clone, Debug)]
pub struct LineSets {
    /// Every diagram set, as [`generate_from_proc_card_in`] returns them: line
    /// by line, in card order, each line's in its own enumeration order.
    pub sets: Vec<DiagramSet>,
    /// The number of [`sets`](Self::sets) each process line produced, one entry
    /// per line in card order; they sum to `sets.len()`.
    pub per_line: Vec<usize>,
}

impl LineSets {
    /// Each process line's sets, in card order.
    pub fn lines(&self) -> impl Iterator<Item = &[DiagramSet]> {
        let mut rest = self.sets.as_slice();
        self.per_line.iter().map(move |&n| {
            let (line, tail) = rest.split_at(n);
            rest = tail;
            line
        })
    }
}

/// [`generate_from_proc_card_in`], keeping which process line produced each
/// set.
pub fn enumerate_lines_in(
    proc_card: &SupportedCard,
    model: &UFOModel,
    pool: EnumerationPool,
) -> Result<LineSets, DiagramError> {
    squared::refuse_mixed_squared_orders(proc_card)?;
    match pool {
        EnumerationPool::Ambient | EnumerationPool::Auto => enumerate(proc_card, model, pool),
        // A pool of its own rather than a serial code path: feyngraph's `par_iter`s
        // are internal, and one thread is the only way to keep them off the
        // caller's pool without forking the enumeration itself.
        EnumerationPool::Serial => rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()?
            .install(|| enumerate(proc_card, model, pool)),
    }
}

/// The card's forbidden on-shell s-channels (`$ A`) as PDG codes, read as the
/// `$$` list is: the particle as written, compared against a propagator's
/// oriented s-channel id, so `$ t` leaves an s-channel `t~` alone. Every
/// process line has to name the same set (empty included), since the veto is
/// installed on the integrand the card's subprocesses share.
pub fn forbidden_onshell_ids(
    proc_card: &SupportedCard,
    model: &UFOModel,
) -> Result<Vec<i64>, DiagramError> {
    let mut common: Option<(&SupportedProcess, Vec<i64>)> = None;
    for process in &proc_card.processes {
        let aliases = model_aliases(&process.aliases, model);
        let mut ids =
            forbidden_s_channel_ids(&process.forbidden_onshell_s_channels, &aliases, model)?;
        ids.sort_unstable();
        match &common {
            None => common = Some((process, ids)),
            Some((first, seen)) if *seen != ids => {
                return Err(DiagramError::MixedOnShellVeto {
                    a: first.to_string(),
                    b: process.to_string(),
                })
            }
            Some(_) => {}
        }
    }
    Ok(common.map(|(_, ids)| ids).unwrap_or_default())
}

/// A leg as a subprocess's identity sees it: the particle and, for a
/// polarized leg, its helicities.
type KeyLeg = (String, Option<Vec<i32>>);

/// A subprocess's identity: the unordered content of each side. A polarized
/// leg and an unpolarized one of the same particle are different content, as
/// they are in MadGraph's own deduplication (`diagram_generation.py:1765`
/// keys the final state on `(id, polarization)`).
type SubprocessKey = (Vec<KeyLeg>, Vec<KeyLeg>);

fn subprocess_key(
    initial: &[String],
    final_state: &[String],
    polarizations: &[Option<Vec<i32>>],
) -> SubprocessKey {
    let side = |names: &[String], pols: &[Option<Vec<i32>>]| {
        let mut legs: Vec<KeyLeg> = names.iter().cloned().zip(pols.iter().cloned()).collect();
        legs.sort();
        legs
    };
    let (pi, pf) = polarizations.split_at(initial.len());
    (side(initial, pi), side(final_state, pf))
}

/// The unordered particle content of each side, without polarizations.
type FlavourKey = (Vec<String>, Vec<String>);

fn flavour_key(set: &DiagramSet) -> FlavourKey {
    let mut i = set.particles_in.clone();
    i.sort();
    let mut f = set.particles_out.clone();
    f.sort();
    (i, f)
}

/// One decay assignment as a diagram carries it: per forced line, the line's
/// particle and the particles of the final-state legs it decays to, each sorted,
/// so that it does not depend on the order of the set's outgoing legs.
type Assignment = Vec<(usize, Vec<usize>)>;

/// The decay assignments of `set`'s diagrams, or `None` when a diagram has no
/// forced line (a line that is not a decay chain).
fn assignments(set: &DiagramSet) -> Option<std::collections::HashSet<Assignment>> {
    use diagram::{OnShell, PropIdx};
    set.diagrams
        .iter()
        .map(|d| {
            let mut forced: Assignment = (0..d.props.len())
                .filter(|&p| d.props[p].onshell == OnShell::Forced)
                .map(|p| {
                    let mut products: Vec<usize> = d
                        .final_state_side(PropIdx(p))
                        .unwrap_or_default()
                        .into_iter()
                        .map(|leg| d.legs[leg.0].particle.index())
                        .collect();
                    products.sort_unstable();
                    (d.props[p].particle.index(), products)
                })
                .collect();
            forced.sort_unstable();
            (!forced.is_empty()).then_some(forced)
        })
        .collect()
}

/// Whether two sets of one subprocess are decay chains whose diagrams share no
/// decay assignment: each set forces its own resonances onto its own legs, as two
/// assignments of one line do.
fn distinct_assignments(a: &DiagramSet, b: &DiagramSet) -> bool {
    match (assignments(a), assignments(b)) {
        (Some(a), Some(b)) => a.is_disjoint(&b),
        _ => false,
    }
}

/// One helicity state of a subprocess: the `(particle, helicity)` of each beam
/// in beam order, and the unordered `(particle, helicity)` content of the final
/// state.
type SummedState = (Vec<(String, i32)>, Vec<(String, i32)>);

/// Every helicity state a subprocess sums over, in every beam ordering it
/// covers: the physical states two subprocesses of the same particles would
/// both count. The beams are kept in order, because `u u~ > e+ e-` and
/// `u~ u > e+ e-` are the two orderings of one initial state and add without
/// double counting; a set that stands for both ([`DiagramSet::mirrored`])
/// counts both.
fn summed_states(
    set: &DiagramSet,
    model: &UFOModel,
) -> Result<std::collections::HashSet<SummedState>, DiagramError> {
    let names: Vec<&String> = set.particles_in.iter().chain(&set.particles_out).collect();
    let states = names
        .iter()
        .zip(&set.polarizations)
        .map(|(name, pol)| match pol {
            Some(h) => Ok(h.clone()),
            None => {
                let particle = model
                    .particles
                    .get(name.as_str())
                    .ok_or_else(|| ResolveError::UnknownParticle((*name).clone()))?;
                Ok(particle.helicity_states().unwrap_or_default())
            }
        })
        .collect::<Result<Vec<_>, ResolveError>>()?;
    let n_in = set.particles_in.len();
    let mut out = std::collections::HashSet::new();
    for combo in states.iter().multi_cartesian_product() {
        let mut legs: Vec<(String, i32)> = names
            .iter()
            .zip(combo)
            .map(|(n, h)| ((*n).clone(), *h))
            .collect();
        let mut fin = legs.split_off(n_in);
        fin.sort();
        if set.mirrored {
            let mut exchanged = legs.clone();
            exchanged.reverse();
            out.insert((exchanged, fin.clone()));
        }
        out.insert((legs, fin));
    }
    Ok(out)
}

/// How many candidate assignments pass between two progress reports while the
/// walk is only pruning them; an assignment that is enumerated reports itself.
const PRUNED_REPORT_INTERVAL: usize = 4096;

/// Where a process line's walk sits in the card, for the progress it reports.
#[derive(Clone, Copy, Debug)]
pub(crate) struct LinePosition {
    /// The line's place in the card, from 1.
    line: usize,
    /// The lines the card holds.
    lines: usize,
    /// Diagrams the card's earlier lines produced.
    diagrams_before: usize,
    /// The pool the line's walks may spread over.
    pool: EnumerationPool,
}

fn enumerate(
    proc_card: &SupportedCard,
    model: &UFOModel,
    pool: EnumerationPool,
) -> Result<LineSets, DiagramError> {
    let _span = info_span!("enumerate").entered();
    let started = Instant::now();
    let mut sets: Vec<DiagramSet> = Vec::new();
    let mut per_line = Vec::with_capacity(proc_card.processes.len());
    // Every process line's subprocesses are summed into one cross section, so a
    // subprocess two lines both produce would be counted twice. MadGraph's own
    // duplicate check compares whole amplitudes, process number included, and
    // lets exactly this through (`generate p p > e+ e-` then `add process
    // u u~ > e+ e-` generates `u u~ > e+ e-` twice), so the refusal here is on
    // the subprocess alone.
    //
    // Two lines may produce the same particles with polarizations that sum
    // over disjoint helicity states (`e+ e- > z{0} h` and `e+ e- > z{T} h`),
    // or in the two beam orderings of one initial state (`u u~ > e+ e-` and
    // `u~ u > e+ e-`): those add without double counting and are kept, each
    // line's subprocess in its own ordering, as MadGraph keeps them.
    //
    // Two decay-chain lines may also reach one subprocess through different
    // decay assignments (`p p > w+ j j, w+ > j j` and `p p > w- j j, w- > j j`
    // both reach `g g > u d~ d u~`, once through W+ > u d~ and once through
    // W- > d u~). Those are the two subprocesses one line gives for the same
    // case, summed incoherently as MadGraph sums them, and are kept. Two lines
    // with the same assignment, or a line without one, would count the same
    // diagrams twice; any state both would count is refused.
    let mut owner: HashMap<FlavourKey, Vec<(usize, usize)>> = HashMap::new();
    for (index, process) in proc_card.processes.iter().enumerate() {
        let position = LinePosition {
            line: index + 1,
            lines: proc_card.processes.len(),
            diagrams_before: sets.iter().map(|s| s.diagrams.len()).sum(),
            pool,
        };
        let process_sets = generate_from_process(process, model, ProcessRole::TopLevel, position)?;
        if process_sets.iter().all(|s| s.diagrams.is_empty()) {
            return Err(DiagramError::NoDiagrams {
                process: process.to_string(),
            });
        }
        for (offset, set) in process_sets.iter().enumerate() {
            if set.diagrams.is_empty() {
                continue;
            }
            let earlier = owner.entry(flavour_key(set)).or_default();
            for &(first, at) in earlier.iter().filter(|(line, _)| *line != index) {
                if distinct_assignments(set, &sets[at]) {
                    continue;
                }
                let states = summed_states(set, model)?;
                if !summed_states(&sets[at], model)?.is_disjoint(&states) {
                    return Err(DiagramError::DuplicateSubprocess {
                        subprocess: set.label(),
                        first: proc_card.processes[first].to_string(),
                        second: process.to_string(),
                    });
                }
            }
            earlier.push((index, sets.len() + offset));
        }
        per_line.push(
            process_sets
                .iter()
                .filter(|s| !s.diagrams.is_empty())
                .count(),
        );
        sets.extend(process_sets);
    }
    // A particle assignment with no diagram is not a subprocess: MadGraph
    // generates no matrix element for it. The duplicate check above indexes
    // `sets` by position, so the empty ones are dropped only once it is done.
    sets.retain(|s| !s.diagrams.is_empty());
    let diagrams: usize = sets.iter().map(|s| s.diagrams.len()).sum();
    let populated = sets.len();
    info!(
        "enumerated {diagrams} diagrams in {populated} subprocess{} ({:.3} s)",
        if populated == 1 { "" } else { "es" },
        started.elapsed().as_secs_f64()
    );
    Ok(LineSets { sets, per_line })
}

/// A process with its names resolved against the model: the model particles
/// each leg may be, in label-member order, the forbidden propagators, and the
/// s-channel restrictions.
struct ExpandedProcess {
    initial: Vec<Vec<String>>,
    final_state: Vec<Vec<String>>,
    /// Per leg, incoming first: a polarized leg's helicity codes as the card
    /// wrote them, before any concrete particle is chosen.
    polarizations: Vec<Option<LegPolarization>>,
    forbidden_particles: Vec<String>,
    /// The coupling-order constraints feyngraph selects on; `WEIGHTED` is not
    /// among them.
    orders: Vec<AmplitudeOrder>,
    /// `WEIGHTED==n` or `WEIGHTED>n`: MadGraph's `constrained_orders` filter on
    /// the diagrams' weighted order (`filter_constrained_orders`,
    /// `base_objects.py:2897-2911`), beside any upper bound.
    weighted_constraint: Option<(CouplingOp, i64)>,
    schannels: SChannelFilter,
    /// The line's squared-order constraints, applied to each subprocess's
    /// diagrams once its other filters have run.
    squared: Option<squared::SquaredRequest>,
}

/// A polarized leg of a process line, before a concrete particle is chosen.
struct LegPolarization {
    /// The leg as written, `w+{0}`, for error messages.
    text: String,
    codes: Vec<i64>,
}

/// A polarized leg's helicities on one concrete particle, or `None` when none
/// is left and MadGraph drops the assignment.
///
/// Helicity 0 of a massless boson is kept by MadGraph's parser, so that a
/// multiparticle mixing massive and massless bosons can be polarized
/// longitudinally, and removed at generation (`diagram_generation.py:1751`
/// and `:1791`); a leg left with no helicity drops the assignment. Codes
/// MadGraph reads but that are no helicity state of the particle (`z{2}`,
/// `h{R}`) and a code listed twice (`z{00}`), which MadGraph would sum twice,
/// are refused rather than handed to the wavefunction routines.
fn concrete_polarization(
    model: &UFOModel,
    name: &str,
    pol: &LegPolarization,
) -> Result<Option<Vec<i32>>, ResolveError> {
    let particle = model
        .particles
        .get(name)
        .ok_or_else(|| ResolveError::UnknownParticle(name.to_owned()))?;
    let massless = particle.mass_param == "ZERO";
    let err = |why| ResolveError::Polarization {
        leg: name.to_owned(),
        pol: pol.text.clone(),
        why,
    };
    let codes: Vec<i32> = pol
        .codes
        .iter()
        .filter(|&&c| !(c == 0 && massless && matches!(particle.spin.abs(), 3 | 5)))
        .map(|&c| c as i32)
        .collect();
    if codes.is_empty() {
        return Ok(None);
    }
    let states = particle
        .helicity_states()
        .ok_or_else(|| err("the particle's spin has no helicity states here"))?;
    if codes.iter().any(|c| !states.contains(c)) {
        return Err(err("a listed helicity is not a state of the particle"));
    }
    let mut unique = codes.clone();
    unique.sort_unstable();
    unique.dedup();
    if unique.len() != codes.len() {
        return Err(err("a helicity is listed twice and would be summed twice"));
    }
    Ok(Some(codes))
}

/// The name MadGraph gives the hierarchy-weighted sum of a diagram's coupling
/// orders.
const WEIGHTED: &str = "WEIGHTED";

/// Where a process sits in the card, which decides whether a one-particle
/// process gets the automatic WEIGHTED search.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProcessRole {
    /// A process the card asks for, or a decay chain's core.
    TopLevel,
    /// A decay inside a chain, which MadGraph flags `is_decay_chain`
    /// (`diagram_generation.py:1381-1382`).
    ChainDecay,
}

/// Generate diagrams for one process of a checked card.
///
/// Resolves the legs against the model, builds a `DiagramSelector` for each
/// concrete particle assignment, then calls `feyngraph::generate_diagrams`.
///
/// When the process has no explicit coupling constraints, the WEIGHTED coupling
/// order filter is applied automatically, as MadGraph's
/// `MultiProcess.find_optimal_process_orders` applies it: the bound starts at
/// the model's estimate for the process's legs, rises by one until some
/// subprocess has a diagram, and stops at `(n_ext − 2)` times the largest
/// WEIGHTED of any interaction, where the diagrams at the bound, if any, are
/// the result. WEIGHTED = Σ_i (hierarchy_i × n_i) where hierarchy comes from the
/// UFO `coupling_orders.py` (e.g. QCD→1, QED→2 in the SM). An explicit
/// `WEIGHTED<=n` is that same filter at a bound the card chose. A one-particle
/// process runs no search unless it is a decay inside a chain
/// ([`ProcessRole::ChainDecay`]): every order is generated
/// (`diagram_generation.py:1978-1980`).
///
/// The s-channel restrictions filter the converted diagrams inside the search,
/// as they do inside MadGraph's (`find_optimal_process_orders` generates each
/// trial with the process's required and forbidden s-channels): an order whose
/// diagrams they all remove moves the search on.
fn generate_from_process(
    process: &SupportedProcess,
    model: &UFOModel,
    role: ProcessRole,
    position: LinePosition,
) -> Result<Vec<DiagramSet>, DiagramError> {
    if !process.decays.is_empty() {
        return chain::generate_chain(process, model, position);
    }
    let mut sets = generate_undecayed(process, model, role, position)?;
    for diagram in sets.iter_mut().flat_map(|s| s.diagrams.iter_mut()) {
        diagram.provenance.process = process.id;
    }
    Ok(sets)
}

/// [`generate_from_process`] for a process without decays.
fn generate_undecayed(
    process: &SupportedProcess,
    model: &UFOModel,
    role: ProcessRole,
    position: LinePosition,
) -> Result<Vec<DiagramSet>, DiagramError> {
    // MadGraph accepts `EW` on a model whose order is called `QED` (and the
    // reverse), then constrains the name as written, which no vertex carries:
    // the constraint does nothing. Refusing an order the model does not define
    // is the one reading that cannot silently do nothing.
    for order in process.orders.iter().filter(|o| o.name != WEIGHTED) {
        check_order_name(model, &order.name)?;
    }
    let aliases = model_aliases(&process.aliases, model);
    let legs = |legs: &[SupportedLeg]| -> Result<Vec<Vec<String>>, ResolveError> {
        legs.iter()
            .map(|l| leg_names(model, &l.particle, &l.token, &aliases))
            .collect()
    };
    let polarization = |l: &SupportedLeg| -> Result<Option<LegPolarization>, ResolveError> {
        l.polarization
            .as_ref()
            .map(|pol| {
                Ok(LegPolarization {
                    text: l.to_string(),
                    codes: leg_polarization_codes(model, &l.particle, &l.token, pol, &aliases)?,
                })
            })
            .transpose()
    };
    let polarizations = process
        .initial
        .iter()
        .chain(&process.final_state)
        .map(polarization)
        .collect::<Result<Vec<_>, _>>()?;
    let ids = |l: &SupportedLeg| -> Result<Vec<i64>, ResolveError> {
        leg_names(model, &l.particle, &l.token, &aliases)?
            .iter()
            .map(|n| {
                model
                    .particles
                    .get(n.as_str())
                    .map(|p| p.pdg_code)
                    .ok_or_else(|| ResolveError::UnknownParticle(n.clone()))
            })
            .collect()
    };
    let n_in = process.initial.len();
    let checked = process
        .initial
        .iter()
        .chain(&process.final_state)
        .zip(&polarizations)
        .enumerate()
        .map(|(i, (l, p))| {
            Ok((
                i >= n_in,
                ids(l)?,
                p.as_ref().map(|p| p.codes.clone()).unwrap_or_default(),
            ))
        })
        .collect::<Result<Vec<_>, ResolveError>>()?;
    if !polarizations_unambiguous(&checked) {
        return Err(ResolveError::AmbiguousPolarization(process.to_string()).into());
    }
    let schannels = SChannelFilter {
        required: required_s_channel_ids(&process.required_s_channels, &aliases, model)?,
        forbidden: forbidden_s_channel_ids(&process.forbidden_s_channels, &aliases, model)?,
    };
    if !schannels.is_empty() {
        // MadGraph's own caution, from the release that introduced `$$`
        // (UpdateNotes, 1.4.3): selecting diagrams by their s-channels is in
        // general not gauge invariant. It prints nothing at generation time.
        warn!(
            "'{process}' selects diagrams by their s-channel propagators; the result is in \
             general not gauge invariant"
        );
    }
    let squared = squared::SquaredRequest::resolve(&process.squared_orders, model)?;
    // MadGraph's `do_add` allows a negative value on one order only, across the
    // amplitude and the squared-order constraints together.
    if let Some(sq) = &squared {
        let negative = process.orders.iter().filter(|o| o.value < 0).count()
            + sq.constraints().iter().filter(|c| c.value < 0).count();
        if negative > 1 {
            return Err(ResolveError::NegativeOrders.into());
        }
    }
    // A line with squared orders and no amplitude-level bound of its own is
    // bounded by its squared values, MadGraph's `orders == {}` rule; `>` is not
    // a bound there, as it adds nothing to MadGraph's `orders`.
    let bounded =
        |o: &AmplitudeOrder| matches!(o.op, CouplingOp::Le | CouplingOp::Eq | CouplingOp::ExactEq);
    let mut amplitude_orders = process.orders.clone();
    if let Some(sq) = squared
        .as_ref()
        .filter(|_| !process.orders.iter().any(bounded))
    {
        for (name, bound) in sq.implied_amplitude_bounds() {
            if let Some(value) = bound {
                amplitude_orders.push(AmplitudeOrder {
                    name,
                    op: CouplingOp::Le,
                    value,
                });
            }
        }
    }
    // `WEIGHTED<=n`, `=n` and `==n` bound the diagrams' weighted order from
    // above (MadGraph's `orders`); `==n` and `>n` also filter on it (its
    // `constrained_orders`). The leftmost constraint of each kind wins, as for
    // every order.
    let explicit_weighted = amplitude_orders
        .iter()
        .find(|o| o.name == WEIGHTED && bounded(o))
        .map(|o| o.value.max(0) as usize);
    let weighted_constraint = amplitude_orders
        .iter()
        .find(|o| o.name == WEIGHTED && matches!(o.op, CouplingOp::ExactEq | CouplingOp::Gt))
        .map(|o| (o.op, o.value));
    let expanded = ExpandedProcess {
        initial: legs(&process.initial)?,
        final_state: legs(&process.final_state)?,
        polarizations,
        forbidden_particles: forbidden_propagator_names(
            model,
            &process.forbidden_particles,
            &aliases,
        )?,
        orders: amplitude_orders
            .iter()
            .filter(|o| o.name != WEIGHTED)
            .cloned()
            .collect(),
        weighted_constraint,
        schannels,
        squared,
    };

    // Generate abstract graph topologies once for this (n_external, n_loops=0) combination.
    // All concrete subprocesses share the same topology set; reusing it avoids re-running
    // the O(n!) topology search for every one of the potentially thousands of particle
    // assignments produced by alias expansion (e.g. p p > q q~ l+ l- l+ l- has ~11k combos).
    let n_ext = expanded.initial.len() + expanded.final_state.len();
    let cached_topologies = generate_topologies(n_ext, &model.topo);

    // The model's own per-order caps ride on top of whatever the process asked for,
    // but they never decide whether the automatic WEIGHTED search runs: MadGraph
    // applies them (`Process.check_expansion_orders`) only after
    // `find_optimal_process_orders` has looked at the process's own orders.
    let top_level_decay = n_in == 1 && role == ProcessRole::TopLevel;
    let auto_weighted = process.orders.is_empty() && expanded.squared.is_none() && !top_level_decay;
    let expanded = ExpandedProcess {
        orders: capped_orders(&expanded.orders, model),
        ..expanded
    };

    if auto_weighted {
        let leg_ids: Vec<Vec<i64>> = checked.iter().map(|(_, ids, _)| ids.clone()).collect();
        search_weighted(
            &process.to_string(),
            &expanded,
            model,
            &leg_ids,
            &cached_topologies,
            position,
        )
    } else {
        generate_sets_inner(
            &expanded,
            model,
            explicit_weighted,
            &cached_topologies,
            Some(position),
            position.pool,
        )
    }
}

/// MadGraph's automatic WEIGHTED search (`find_optimal_process_orders`) for
/// `expanded`, whose legs' PDG codes are `leg_ids`: the diagram sets at the
/// bound the search settles on. `process` names the process in a refusal.
fn search_weighted(
    process: &str,
    expanded: &ExpandedProcess,
    model: &UFOModel,
    leg_ids: &[Vec<i64>],
    cached_topologies: &[Topology],
    position: LinePosition,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let search = &model.weighted_search;
    let start = i64::from(search.minimum_weighted(leg_ids, &expanded.schannels.required));
    let n_ext = expanded.initial.len() + expanded.final_state.len();
    let cap = search.maximum_weighted(n_ext);

    // `find_optimal_process_orders`: start at the model's estimate for the legs
    // (`get_minimum_WEIGHTED`), raise the bound by one (:2111) while it is
    // below the cap (:2013) and no trial has a diagram, then generate at
    // the bound the loop stopped at, `max(start, cap)` (:2115, :1683-1686);
    // its diagrams, if any, are the result.
    let mut w = start;
    if gluons_leave_trials(expanded, model) {
        let mut trials = GluonFreeTrials::new(process, expanded, model, position.pool);
        while w < cap {
            if trials.any_diagram(w)? {
                debug!("lowest WEIGHTED order with a gluon-free trial diagram: {w}");
                break;
            }
            debug!("no gluon-free trial diagram at WEIGHTED {w}, raising the bound");
            w += 1;
        }
        return generate_sets_inner(
            expanded,
            model,
            Some(w as usize),
            cached_topologies,
            Some(position),
            position.pool,
        );
    }
    while w < cap {
        let sets = generate_sets_inner(
            expanded,
            model,
            Some(w as usize),
            cached_topologies,
            Some(position),
            position.pool,
        )?;
        if sets.iter().any(|s| !s.diagrams.is_empty()) {
            debug!("lowest WEIGHTED order with diagrams: {w}");
            return Ok(sets);
        }
        debug!("no diagrams at WEIGHTED {w}, raising the bound");
        w += 1;
    }
    debug!("no diagrams below the WEIGHTED cap {cap}; generating at {w}");
    generate_sets_inner(
        expanded,
        model,
        Some(w as usize),
        cached_topologies,
        Some(position),
        position.pool,
    )
}

/// The PDG code MadGraph's automatic WEIGHTED search removes from the final state
/// of each trial (`diagram_generation.py:2040-2045`).
const GLUON: i64 = 21;

/// Whether the automatic WEIGHTED search trials a process other than the one it
/// bounds: the gluon is in the lowest hierarchy class
/// (`Model.get_particles_hierarchy`'s first class, `diagram_generation.py:2042`)
/// and some final-state leg can be one.
///
/// Otherwise every trial is a subprocess of the process itself, and the search
/// trials the process whole, keeping the diagrams of the first bound that has any.
/// MadGraph's trials differ from that only by building each leg from its
/// particle id alone (a polarized leg's helicities are not read) and by skipping
/// the crossings of a trial that failed, both of which can move only the bound a
/// refusal quotes.
fn gluons_leave_trials(process: &ExpandedProcess, model: &UFOModel) -> bool {
    let lowest = model.weighted_search.particle_classes.first();
    lowest.is_some_and(|(_, particles)| particles.contains(&GLUON))
        && process.final_state.iter().flatten().any(|name| {
            model
                .particles
                .get(name)
                .is_some_and(|p| p.pdg_code == GLUON)
        })
}

/// The trials of MadGraph's automatic WEIGHTED search when the gluon is in the
/// lowest hierarchy class (`find_optimal_process_orders`,
/// `diagram_generation.py:2015-2108`).
///
/// Per initial-state assignment, each final-state assignment (one per sorted
/// content) is trialled with its gluons removed, at the bound lowered by the
/// QCD hierarchy once per removed gluon. A lowered bound below zero bounds
/// nothing: `reduce_orders` ignores a negative order (`:1062-1083`). A trial
/// left with two legs has the single diagram MadGraph builds from a
/// particle-antiparticle pair, the identity vertex its `ref_dict_to0` holds for
/// every particle (`base_objects.py:579-589`), unless an s-channel is required;
/// one left with fewer has none. A trial whose crossing, all legs outgoing, has
/// already failed at this bound is skipped unless the process forbids
/// s-channels (`:2091-2096`).
struct GluonFreeTrials<'a> {
    process: &'a str,
    expanded: &'a ExpandedProcess,
    model: &'a UFOModel,
    pool: EnumerationPool,
    topologies: HashMap<usize, Vec<Topology>>,
}

impl<'a> GluonFreeTrials<'a> {
    fn new(
        process: &'a str,
        expanded: &'a ExpandedProcess,
        model: &'a UFOModel,
        pool: EnumerationPool,
    ) -> Self {
        GluonFreeTrials {
            process,
            expanded,
            model,
            pool,
            topologies: HashMap::new(),
        }
    }

    /// Whether any trial has a diagram at the bound `w`.
    fn any_diagram(&mut self, w: i64) -> Result<bool, DiagramError> {
        let model = self.model;
        let particle = |name: &String| {
            model
                .particles
                .get(name)
                .ok_or_else(|| ResolveError::UnknownParticle(name.clone()))
        };
        let mut failed: std::collections::HashSet<Vec<String>> = Default::default();
        let initial_combos = self
            .expanded
            .initial
            .iter()
            .cloned()
            .multi_cartesian_product();
        for initial in initial_combos {
            let mut seen: std::collections::HashSet<Vec<String>> = Default::default();
            for final_state in self
                .expanded
                .final_state
                .iter()
                .cloned()
                .multi_cartesian_product()
            {
                if !seen.insert(final_state.iter().cloned().sorted().collect()) {
                    continue;
                }
                let mut reduced = Vec::with_capacity(final_state.len());
                let mut gluons = 0i64;
                for name in final_state {
                    if particle(&name)?.pdg_code == GLUON {
                        gluons += 1;
                    } else {
                        reduced.push(name);
                    }
                }
                let bound = if gluons == 0 {
                    w
                } else {
                    let qcd = model.order_hierarchy.get("QCD").ok_or_else(|| {
                        DiagramError::GluonTrialWithoutQcd {
                            process: self.process.to_owned(),
                        }
                    })?;
                    w - gluons * i64::from(*qcd)
                };
                let crossed: Vec<String> = initial
                    .iter()
                    .map(|name| particle(name).map(|p| p.antiname.clone()))
                    .chain(reduced.iter().map(|name| Ok(name.clone())))
                    .collect::<Result<Vec<_>, ResolveError>>()?
                    .into_iter()
                    .sorted()
                    .collect();
                if self.expanded.schannels.forbidden.is_empty() && failed.contains(&crossed) {
                    continue;
                }
                if self.trial_has_diagram(&initial, reduced, &crossed, bound)? {
                    return Ok(true);
                }
                failed.insert(crossed);
            }
        }
        Ok(false)
    }

    fn trial_has_diagram(
        &mut self,
        initial: &[String],
        final_state: Vec<String>,
        crossed: &[String],
        bound: i64,
    ) -> Result<bool, DiagramError> {
        let n_ext = initial.len() + final_state.len();
        if n_ext < 2 {
            return Ok(false);
        }
        if n_ext == 2 {
            let required = self
                .expanded
                .schannels
                .required
                .iter()
                .any(|r| !r.is_empty());
            let pair = self
                .model
                .particles
                .get(&crossed[0])
                .is_some_and(|p| p.antiname == crossed[1]);
            return Ok(pair && !required);
        }
        let model = self.model;
        let topologies = self
            .topologies
            .entry(n_ext)
            .or_insert_with(|| generate_topologies(n_ext, &model.topo));
        let trial = ExpandedProcess {
            initial: initial.iter().map(|n| vec![n.clone()]).collect(),
            polarizations: (0..n_ext).map(|_| None).collect(),
            final_state: final_state.into_iter().map(|n| vec![n]).collect(),
            forbidden_particles: self.expanded.forbidden_particles.clone(),
            orders: self.expanded.orders.clone(),
            weighted_constraint: self.expanded.weighted_constraint,
            schannels: self.expanded.schannels.clone(),
            squared: self.expanded.squared.clone(),
        };
        let max_weighted = usize::try_from(bound).ok();
        let sets = generate_sets_inner(&trial, model, max_weighted, topologies, None, self.pool)?;
        Ok(sets.iter().any(|s| !s.diagrams.is_empty()))
    }
}

/// `orders` with the model's `expansion_order` caps folded in (every model in
/// reach caps nothing: `expansion_order` is 99 throughout the SM and SMEFTsim,
/// and SMEFTsim's `NPprop = 0` falls outside MadGraph's `0 < v < 99` window).
///
/// MadGraph's `Process.check_expansion_orders` writes the cap straight into the
/// process's `orders` dict — lowering an order the process bounded above the cap,
/// and adding one it left unconstrained. `orders` there means `<=`, so an
/// explicit constraint using any other comparison is left alone rather than
/// reinterpreted.
fn capped_orders(orders: &[AmplitudeOrder], model: &UFOModel) -> Vec<AmplitudeOrder> {
    let mut out = orders.to_vec();
    for (order, cap) in crate::ufo::expansion_order_caps(&model.expansion_order) {
        let cap = cap as i64;
        match out.iter_mut().find(|c| c.name == order) {
            Some(c) if matches!(c.op, CouplingOp::Le | CouplingOp::Eq) => {
                if c.value > cap {
                    debug!("{order}<={} lowered to the model's cap {cap}", c.value);
                    c.value = cap;
                }
            }
            Some(_) => {}
            None => out.push(AmplitudeOrder {
                name: order,
                op: CouplingOp::Le,
                value: cap,
            }),
        }
    }
    out
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

/// Whether `initial` with its legs' `polarizations` (incoming first) is the
/// initial state `set_in` with polarizations `set_pol` with the two beams
/// exchanged, and a different state from it.
fn is_exchanged_ordering(
    set_in: &[String],
    set_pol: &[Option<Vec<i32>>],
    initial: &[String],
    polarizations: &[Option<Vec<i32>>],
) -> bool {
    let [a, b] = initial else {
        return false;
    };
    let [sa, sb] = set_in else {
        return false;
    };
    let (pa, pb) = (&polarizations[0], &polarizations[1]);
    let (spa, spb) = (&set_pol[0], &set_pol[1]);
    (a, pa) == (sb, spb) && (b, pb) == (sa, spa) && (a, pa) != (b, pb)
}

/// A walk's surviving assignment, waiting for its topology assignment.
struct Pending {
    concrete: ConcreteProcess,
    polarizations: Vec<Option<Vec<i32>>>,
    mirrored: bool,
}

/// What topology assignment made of one [`Pending`] assignment, with the
/// counts its log lines report.
struct Assigned {
    set: DiagramSet,
    /// Diagrams before the s-channel restrictions.
    enumerated: usize,
    /// Diagrams before the squared-order constraints, when the line has some.
    before_squared: Option<usize>,
    elapsed: f64,
}

/// What every assignment of one walk shares.
struct Walk<'a> {
    process: &'a ExpandedProcess,
    model: &'a UFOModel,
    max_weighted: Option<usize>,
    cached_topologies: &'a [Topology],
    budgets: std::collections::BTreeMap<String, i64>,
}

impl Walk<'_> {
    /// Assign the cached topologies to one subprocess and convert and filter
    /// its diagrams. Touches nothing but its own assignment, so any number of
    /// them run at once.
    fn assign(&self, pending: Pending) -> Result<Assigned, DiagramError> {
        let Walk {
            process,
            model,
            max_weighted,
            cached_topologies,
            ref budgets,
        } = *self;
        let Pending {
            concrete,
            polarizations,
            mirrored,
        } = pending;
        let mut sel = build_selector(&concrete, &model.negative_orders);

        // WEIGHTED coupling-order filter: reject diagrams whose weighted sum, each
        // order's signed power times its hierarchy, exceeds max_weighted.
        if let Some(max_w) = max_weighted {
            use std::sync::Arc;
            let hierarchy = model.order_hierarchy.clone();
            let negative = model.negative_orders.clone();
            let max_w = max_w as i64;
            let weighted_fn: Arc<
                dyn Fn(&feyngraph::diagram::view::DiagramView) -> bool + Send + Sync,
            > = Arc::new(move |diag_view| {
                let w: i64 = hierarchy
                    .iter()
                    .map(|(coupling, &h)| diag_view.order(coupling) as i64 * h as i64)
                    .sum();
                let w = if negative.is_empty() {
                    w
                } else {
                    w + negative.weighted(diag_view, &hierarchy)
                };
                w <= max_w
            });
            sel.add_custom_function(weighted_fn);
        }
        if let Some((op, value)) = process.weighted_constraint {
            use std::sync::Arc;
            let hierarchy = model.order_hierarchy.clone();
            let negative = model.negative_orders.clone();
            let constraint_fn: Arc<
                dyn Fn(&feyngraph::diagram::view::DiagramView) -> bool + Send + Sync,
            > = Arc::new(move |diag_view| {
                let w: i64 = hierarchy
                    .iter()
                    .map(|(coupling, &h)| diag_view.order(coupling) as i64 * h as i64)
                    .sum::<i64>()
                    + negative.weighted(diag_view, &hierarchy);
                match op {
                    CouplingOp::ExactEq => w == value,
                    _ => w > value,
                }
            });
            sel.add_custom_function(constraint_fn);
        }

        let in_refs: Vec<&str> = concrete.initial.iter().map(String::as_str).collect();
        let out_refs: Vec<&str> = concrete.final_state.iter().map(String::as_str).collect();
        let started = Instant::now();

        let generator =
            DiagramGenerator::new(&in_refs, &out_refs, 0, model.topo.clone(), Some(sel))?;
        // The cached topologies were built for this line's leg count at tree level,
        // which is all `assign_topologies` checks.
        let container = generator
            .assign_topologies(cached_topologies)
            .map_err(|e| DiagramError::TopologyAssignment {
                subprocess: format!("{} > {}", in_refs.join(" "), out_refs.join(" ")),
                why: e.to_string(),
            })?;

        // Module boundary: convert feyngraph's borrowed views into owned, UFO-resolved
        // diagrams here and drop the container. feyngraph views never escape `diagrams/`.
        let mut diagrams = container
            .views()
            .map(|view| Diagram::from_view(&view, model))
            .collect::<Result<Vec<_>, _>>()?;
        if !model.negative_orders.is_empty() {
            diagrams.retain(|d| {
                reduction::within_running_budget(d, model, budgets, max_weighted.map(|w| w as i64))
            });
        }
        let enumerated = diagrams.len();
        diagrams.retain(|d| process.schannels.keeps(d, model));
        for diagram in &diagrams {
            diagram.check_fermion_flow(model)?;
        }
        let mut before_squared = None;
        let squared_orders = process.squared.as_ref().map(|sq| {
            before_squared = Some(diagrams.len());
            sq.filter(&mut diagrams, model)
        });
        Ok(Assigned {
            set: DiagramSet {
                particles_in: concrete.initial,
                particles_out: concrete.final_state,
                polarizations,
                diagrams,
                mirrored,
                squared_orders,
            },
            enumerated,
            before_squared,
            elapsed: started.elapsed().as_secs_f64(),
        })
    }
}

/// Log what [`Walk::assign`] made of one assignment, as the walk reaches it.
fn report_assigned(assigned: &Assigned, model: &UFOModel) {
    let set = &assigned.set;
    let subprocess = format!(
        "{} > {}",
        set.particles_in.join(" "),
        set.particles_out.join(" ")
    );
    let after_schannels = assigned.before_squared.unwrap_or(set.diagrams.len());
    if after_schannels != assigned.enumerated {
        debug!(
            "{subprocess}: the s-channel restrictions keep {after_schannels} of {} diagrams",
            assigned.enumerated
        );
    }
    if let Some(before) = assigned.before_squared {
        if set.diagrams.len() != before {
            debug!(
                "{subprocess}: the squared-order constraints keep {} of {before} diagrams",
                set.diagrams.len()
            );
        }
    }
    if !set.diagrams.is_empty() {
        info!("{} diagrams for {subprocess}", set.diagrams.len());
        debug!("{subprocess}: enumerated in {:.3} s", assigned.elapsed);
        report_vertex_assignments(&subprocess, &set.diagrams, model);
    }
}

/// Inner generation loop: expand aliases, deduplicate mirror processes, and call
/// feyngraph for each concrete subprocess.  `max_weighted` (when `Some`) adds an
/// extra diagram filter that rejects any diagram whose WEIGHTED order exceeds the
/// given bound.
///
/// `cached_topologies` must be pre-computed by the caller via `generate_topologies`.
/// Coupling constraints are enforced during particle assignment, not topology
/// generation, so no topology filtering is needed here.
///
/// `pool` decides whether the walk's subprocesses are assigned across the
/// caller's pool ([`EnumerationPool`]); the sets are the same either way.
fn generate_sets_inner(
    process: &ExpandedProcess,
    model: &UFOModel,
    max_weighted: Option<usize>,
    cached_topologies: &[Topology],
    position: Option<LinePosition>,
    pool: EnumerationPool,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let threads = rayon::current_num_threads();
    let candidates = process
        .initial
        .iter()
        .chain(&process.final_state)
        .fold(1u64, |n, leg| n.saturating_mul(leg.len() as u64));
    let cost = candidates.saturating_mul(cached_topologies.len() as u64);
    let parallel = threads > 1
        && match pool {
            EnumerationPool::Serial => false,
            EnumerationPool::Ambient => true,
            EnumerationPool::Auto => position.is_some() && cost >= AUTO_PARALLEL_COST,
        };
    if !parallel && threads > 1 && pool != EnumerationPool::Ambient {
        // feyngraph's internal fan-out would otherwise run on the caller's pool.
        let serial = serial_pool()?;
        return serial.install(|| {
            walk_assignments(process, model, max_weighted, cached_topologies, position, 1)
        });
    }
    let batch = if parallel {
        BATCH_PER_THREAD * threads
    } else {
        1
    };
    walk_assignments(
        process,
        model,
        max_weighted,
        cached_topologies,
        position,
        batch,
    )
}

/// [`generate_sets_inner`]'s walk, handing feyngraph `batch` surviving
/// subprocesses at a time, across the pool when there is more than one.
fn walk_assignments(
    process: &ExpandedProcess,
    model: &UFOModel,
    max_weighted: Option<usize>,
    cached_topologies: &[Topology],
    position: Option<LinePosition>,
    batch: usize,
) -> Result<Vec<DiagramSet>, DiagramError> {
    let mut sets = Vec::new();
    // The `<=` bounds MadGraph spends vertex by vertex while it builds a diagram
    // (`reduce_orders`); the leftmost constraint on an order wins. They drop
    // diagrams the total-sum selection keeps only where a vertex carries a
    // negative power ([`reduction`]).
    let mut budgets: std::collections::BTreeMap<String, i64> = Default::default();
    for o in &process.orders {
        if matches!(o.op, CouplingOp::Le | CouplingOp::Eq) {
            budgets.entry(o.name.clone()).or_insert(o.value);
        }
    }
    let walk = Walk {
        process,
        model,
        max_weighted,
        cached_topologies,
        budgets,
    };
    // Candidate assignments seen, and the prefilters' kills: what the
    // enumeration paid for the assignments it never handed to feyngraph.
    let mut candidates = 0usize;
    let mut duplicates = 0usize;
    let mut charge_kills = 0usize;
    let mut flavour_kills = 0usize;
    let mut dropped_polarizations = 0usize;
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
    //
    // A duplicate whose incoming legs are an enumerated set's exchanged is that
    // set's mirrored beam ordering: it is not enumerated again, and the set
    // records that its line generated both orderings. MadGraph does the same
    // within one process definition (`generate_multi_amplitudes`, the
    // `collect_mirror_procs` branch), and only there: a line written in one
    // ordering generates no mirror.
    let mut seen_processes: std::collections::HashSet<SubprocessKey> =
        std::collections::HashSet::new();
    // A surviving assignment's place in the walk: in `sets` once assigned, in
    // `pending` (offset by `sets.len()`) while it waits for its batch.
    let mut enumerated_at: HashMap<SubprocessKey, usize> = HashMap::new();
    let mut pending: Vec<Pending> = Vec::with_capacity(batch);

    // Each leg independently takes each of its particles; the Cartesian product
    // over the legs, initial side outermost, is the concrete subprocess list.
    // For `p p > e+ e-` this yields 9 × 9 = 81 candidates.
    let initial_combos: Vec<Vec<String>> = process
        .initial
        .iter()
        .cloned()
        .multi_cartesian_product()
        .collect();
    let final_combos: Vec<Vec<String>> = process
        .final_state
        .iter()
        .cloned()
        .multi_cartesian_product()
        .collect();

    let total = (initial_combos.len() * final_combos.len()) as u64;
    // A trial enumeration (`position` is `None`) is not the line's walk, so it
    // reports no progress.
    let report = |done: usize, sets: &[DiagramSet]| {
        let Some(position) = position else {
            return;
        };
        progress::enumeration(
            done as u64,
            total,
            (position.line as u64, position.lines as u64),
            sets.len() as u64,
            (position.diagrams_before + sets.iter().map(|s| s.diagrams.len()).sum::<usize>())
                as u64,
        );
    };
    // Assign every pending subprocess, across the pool when there are several,
    // and append them to `sets` in walk order. The first failure in walk order is
    // the walk's, as it is when they are assigned one at a time.
    let flush = |pending: &mut Vec<Pending>,
                 sets: &mut Vec<DiagramSet>,
                 done: usize|
     -> Result<(), DiagramError> {
        let assigned: Vec<Result<Assigned, DiagramError>> = if pending.len() > 1 {
            use rayon::prelude::*;
            let span = tracing::Span::current();
            pending
                .par_drain(..)
                .map(|p| {
                    let _entered = span.enter();
                    walk.assign(p)
                })
                .collect()
        } else {
            pending.drain(..).map(|p| walk.assign(p)).collect()
        };
        for assigned in assigned {
            let assigned = assigned?;
            report_assigned(&assigned, model);
            sets.push(assigned.set);
            report(done, sets);
        }
        Ok(())
    };
    report(0, &sets);
    'assignments: for (initial, final_state) in itertools::iproduct!(initial_combos, final_combos) {
        candidates += 1;
        if candidates.is_multiple_of(PRUNED_REPORT_INTERVAL) {
            report(candidates, &sets);
        }
        let concrete = ConcreteProcess {
            initial,
            final_state,
            forbidden_particles: process.forbidden_particles.clone(),
            orders: process.orders.clone(),
        };

        let mut polarizations = Vec::with_capacity(process.polarizations.len());
        for (name, pol) in concrete
            .initial
            .iter()
            .chain(&concrete.final_state)
            .zip(&process.polarizations)
        {
            match pol {
                None => polarizations.push(None),
                Some(pol) => match concrete_polarization(model, name, pol)? {
                    Some(h) => polarizations.push(Some(h)),
                    None => {
                        dropped_polarizations += 1;
                        continue 'assignments;
                    }
                },
            }
        }

        let key = subprocess_key(&concrete.initial, &concrete.final_state, &polarizations);
        if !seen_processes.insert(key.clone()) {
            if let Some(&at) = enumerated_at.get(&key) {
                let exchanged = |set_in: &[String], set_pol: &[Option<Vec<i32>>]| {
                    is_exchanged_ordering(set_in, set_pol, &concrete.initial, &polarizations)
                };
                match at.checked_sub(sets.len()) {
                    None => {
                        let set = &mut sets[at];
                        if exchanged(&set.particles_in, &set.polarizations) {
                            set.mirrored = true;
                        }
                    }
                    Some(waiting) => {
                        let p = &mut pending[waiting];
                        if exchanged(&p.concrete.initial, &p.polarizations) {
                            p.mirrored = true;
                        }
                    }
                }
            }
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
        // Every additive number the model's vertices conserve: an assignment that
        // breaks one has no diagram ([`crate::ufo::conserved`]).
        if !flavour_balanced(model, &concrete.initial, &concrete.final_state) {
            flavour_kills += 1;
            continue;
        }

        enumerated_at.insert(key, sets.len() + pending.len());
        pending.push(Pending {
            concrete,
            polarizations,
            mirrored: false,
        });
        if pending.len() >= batch {
            flush(&mut pending, &mut sets, candidates)?;
        }
    }
    flush(&mut pending, &mut sets, candidates)?;
    report(candidates, &sets);

    // Reported only for the pass that found something: under the automatic
    // WEIGHTED search the same expansion is walked once per candidate order, and
    // the prefilter counts are identical every time.
    if sets.iter().any(|s| !s.diagrams.is_empty()) {
        debug!(
            "{candidates} alias-expanded assignments: {duplicates} duplicate, {charge_kills} \
             charge-violating, {flavour_kills} violating another conserved number, \
             {dropped_polarizations} with no helicity left, {} enumerated",
            sets.len()
        );
    }

    Ok(sets)
}

/// Whether the incoming `initial` and outgoing `final_state` balance in every
/// additive number the model conserves. A name the model does not know
/// balances, for feyngraph to refuse.
fn flavour_balanced(model: &UFOModel, initial: &[String], final_state: &[String]) -> bool {
    let conserved = model.conserved_numbers();
    if conserved.is_empty() {
        return true;
    }
    let ids = |names: &[String]| -> Option<Vec<crate::ufo::particles::ParticleId>> {
        names.iter().map(|n| model.particle_id(n)).collect()
    };
    match (ids(initial), ids(final_state)) {
        (Some(i), Some(f)) => conserved.balanced(&i, &f),
        _ => true,
    }
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
    use crate::ufo::sm::{sm_model, sm_parsed_model, SMRestrict};

    /// Particle assignments the model gives no diagram are not returned. Under
    /// `p p > e+ e- j` with `p = g u d u~ d~`, `g g > e+ e- g` and the quark pairs
    /// that cannot annihilate (`u d > e+ e- d~`) have none. What is left is
    /// MadGraph's subprocess list for the card: the four Compton channels and the
    /// two annihilations, each standing for both beam orderings through
    /// `mirrored`. The exact list is what fails if empty sets come back, since
    /// every returned set would still be a valid one.
    #[test]
    fn assignments_without_diagrams_are_not_subprocesses() {
        let card = parse_proc_card(
            "define p = g u d u~ d~\ndefine j = p\ngenerate p p > e+ e- j",
            &ParsingOptions::default(),
        )
        .unwrap();
        let sets = generate_from_proc_card(&card, &sm_model(SMRestrict::Default)).unwrap();
        assert!(sets.iter().all(|s| !s.diagrams.is_empty()));
        let labels: Vec<String> = sets.iter().map(DiagramSet::label).collect();
        assert_eq!(
            labels,
            [
                "g u > e+ e- u",
                "g d > e+ e- d",
                "g u~ > e+ e- u~",
                "g d~ > e+ e- d~",
                "u u~ > e+ e- g",
                "d d~ > e+ e- g",
            ]
        );
        assert!(sets.iter().all(|s| s.mirrored), "{labels:?}");
    }

    /// The model's `expansion_order` cap reaches diagram generation, and only
    /// through MadGraph's `0 < v < 99` window.
    ///
    /// No model this loader reads declares a cap inside that window — SMEFTsim's
    /// only non-99 order is `NPprop = 0`, which the window excludes — so the
    /// window is pinned on an SM deliberately given one: `e+ e- > mu+ mu-` needs
    /// `QED = 2`, so a cap of 1 must empty it, while a cap of 0 (or 99, or a
    /// negative one) must leave its two diagrams alone. Without the window the
    /// declared 0 would read as "at most zero QED vertices" and empty it too.
    #[test]
    fn expansion_order_caps_reach_generation_only_inside_madgraphs_window() {
        let capped_model = |expansion: i64| {
            let mut parsed = sm_parsed_model();
            parsed.expansion_order.insert("QED".to_owned(), expansion);
            let card = SMRestrict::Default
                .restrict_card_text()
                .parse()
                .expect("parse the interned SM restrict card");
            parsed
                .into_model(Some(&card))
                .expect("build the SM with a synthetic expansion_order")
        };
        let count = |model: &UFOModel, process: &str| -> usize {
            let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
                .unwrap();
            match generate_from_proc_card(&card, model) {
                Ok(sets) => sets.iter().map(|s| s.diagrams.len()).sum(),
                Err(DiagramError::NoDiagrams { .. }) => 0,
                Err(e) => panic!("{process}: {e}"),
            }
        };

        let plain = sm_model(SMRestrict::Default);
        assert_eq!(count(&plain, "e+ e- > mu+ mu-"), 2);

        // Inside the window: the cap applies, to an unconstrained order and to an
        // explicit one alike.
        let inside = capped_model(1);
        assert_eq!(count(&inside, "e+ e- > mu+ mu-"), 0);
        assert_eq!(count(&inside, "e+ e- > mu+ mu- QED<=2"), 0);

        // Outside it: 0, 99 and a negative value all cap nothing.
        for expansion in [0, 99, -1] {
            let outside = capped_model(expansion);
            assert_eq!(
                count(&outside, "e+ e- > mu+ mu-"),
                2,
                "expansion_order = {expansion} must not cap"
            );
        }
    }

    /// Every set, byte for byte as the artifact serializes it, with the fields
    /// the serialization skips.
    fn rendered(sets: &[DiagramSet]) -> (Vec<u8>, String) {
        let bytes = bincode::serialize(sets).expect("the sets serialize");
        (bytes, format!("{sets:?}"))
    }

    fn sets_in(card: &str, model: &UFOModel, pool: EnumerationPool) -> Vec<DiagramSet> {
        let card = parse_proc_card(card, &ParsingOptions::default()).unwrap();
        generate_from_proc_card_in(&card, model, pool).unwrap()
    }

    /// Several walks, each with several batches of subprocesses, a decay chain, a
    /// polarized leg and both mirrored and unmirrored sets.
    const PARALLEL_CARD: &str = "\
define p = g u c d s u~ c~ d~ s~
define j = p
generate p p > e+ e- j j
add process p p > w+ j, w+ > e+ ve
add process u u~ > z{0} g
add process p p > mu+ mu- j";

    /// Walks assigned across a four-thread pool give the sets a one-thread walk
    /// gives, byte for byte and in the same order, under every pool choice. It
    /// cannot see an output the scheduling changes only on another thread count.
    #[test]
    fn a_parallel_walk_gives_the_serial_walk_byte_for_byte() {
        let model = sm_model(SMRestrict::NoBMass);
        let serial = sets_in(PARALLEL_CARD, &model, EnumerationPool::Serial);
        assert!(serial.len() > BATCH_PER_THREAD * 4, "{} sets", serial.len());
        assert!(serial.iter().any(|s| s.mirrored));
        assert!(serial.iter().any(|s| !s.mirrored));
        let four = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        for pool in [EnumerationPool::Ambient, EnumerationPool::Auto] {
            let parallel = four.install(|| sets_in(PARALLEL_CARD, &model, pool));
            assert!(rendered(&parallel) == rendered(&serial), "{pool:?}");
        }
    }

    /// A refusal raised while subprocesses are assigned across the pool is the
    /// refusal the serial walk raises: the first in walk order, whichever thread
    /// finishes first. With custom propagators on both `u` and `d`,
    /// `u u~ > w+ w-` exchanges a `d` and `d d~ > w+ w-` a `u`, both in one
    /// batch; the walk reaches `u u~` first, so the refusal names `d`.
    #[test]
    fn a_refusal_inside_the_pool_is_the_serial_refusal() {
        use crate::ufo::propagators::Propagator;
        let mut parsed = sm_parsed_model();
        parsed.propagators.insert(
            "F1".to_owned(),
            Propagator {
                python_name: "F1".to_owned(),
                name: "F1".to_owned(),
                numerator: "complex(0,1)*(PSlash(1,2,id)+Mass(id)*Identity(1,2))".to_owned(),
                denominator: Some("P('mu', id) * P('mu', id)".to_owned()),
            },
        );
        for quark in ["u", "d"] {
            parsed.particles.get_mut(quark).expect("quark").propagator = Some("F1".to_owned());
        }
        let restrict = SMRestrict::Default
            .restrict_card_text()
            .parse()
            .expect("parse the interned SM restrict card");
        let model = parsed.into_model(Some(&restrict)).expect("build the model");
        let card = parse_proc_card(
            "define p = g u d u~ d~\ngenerate p p > w+ w- QED<=2",
            &ParsingOptions::default(),
        )
        .unwrap();
        let refusal = |pool| match generate_from_proc_card_in(&card, &model, pool) {
            Ok(_) => panic!("a propagating custom quark must be refused"),
            Err(e) => e.to_string(),
        };
        let serial = refusal(EnumerationPool::Serial);
        assert!(serial.contains("'d'"), "{serial}");
        let four = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
        for _ in 0..8 {
            assert_eq!(four.install(|| refusal(EnumerationPool::Ambient)), serial);
        }
    }

    /// The conserved-number prune hands feyngraph fewer subprocesses and drops
    /// none that has a diagram: with the prune and with no conserved number at
    /// all, the sets are identical, on the diagonal model, where it prunes, and
    /// on the CKM restriction, where the generation numbers are not conserved.
    /// It cannot see an assignment that has diagrams only on a card this one
    /// does not reach.
    #[test]
    fn the_flavour_prune_drops_no_subprocess_with_a_diagram() {
        let card = "\
define p = g u c d s u~ c~ d~ s~
define j = p
define l+ = e+ mu+
define vl = ve vm
generate p p > l+ vl j
add process p p > e+ e- j j";
        for variant in [SMRestrict::NoBMass, SMRestrict::Ckm] {
            let model = sm_model(variant);
            let mut unpruned = (*model).clone();
            unpruned.conserved = crate::ufo::conserved::ConservedNumbers::none();
            let pruned = sets_in(card, &model, EnumerationPool::Serial);
            let all = sets_in(card, &unpruned, EnumerationPool::Serial);
            assert!(!pruned.is_empty());
            assert!(rendered(&pruned) == rendered(&all), "{variant:?}");
        }
        let diagonal = sm_model(SMRestrict::NoBMass);
        let names = |n: &[&str]| n.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(!flavour_balanced(
            &diagonal,
            &names(&["u", "c~"]),
            &names(&["e+", "ve", "g"])
        ));
        assert!(!flavour_balanced(
            &diagonal,
            &names(&["u", "d~"]),
            &names(&["mu+", "ve", "g"])
        ));
        assert!(flavour_balanced(
            &sm_model(SMRestrict::Ckm),
            &names(&["u", "s~"]),
            &names(&["e+", "ve", "g"])
        ));
    }

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
