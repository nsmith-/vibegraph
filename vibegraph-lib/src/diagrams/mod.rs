//! Process grammar and diagram generation interface.
//!
//! A `proc_card.dat` goes through three stages before feyngraph sees it:
//!
//! 1. [`parse::parse_proc_card_ast`] reads the whole of MadGraph's process
//!    language into a [`ProcCardAst`], dropping nothing;
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

pub mod alias;
mod chain;
pub mod check;
pub mod diagram;
pub mod parse;
pub mod resolve;
pub mod schannel;
pub mod selector;

pub use alias::AliasTable;
pub use check::{
    check_supported, AmplitudeOrder, SupportedCard, SupportedLeg, SupportedProcess, Unsupported,
    UnsupportedCard,
};
pub use diagram::{ConvertError, Diagram};
pub use parse::{
    parse_proc_card_ast, CouplingConstraint, CouplingOp, LegParticle, ModelImport,
    MultiparticleDef, ProcCardAst,
};

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use feyngraph::topology::{Topology, TopologyGenerator, TopologyModel};
use feyngraph::DiagramGenerator;
use itertools::Itertools;
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
    /// Per external leg, incoming first: the helicities a polarized leg is
    /// restricted to, as MadGraph's `NHEL` values in the order the card listed
    /// them, or `None` for a leg summed over all its states.
    pub polarizations: Vec<Option<Vec<i32>>>,
    pub diagrams: Vec<Diagram>,
}

impl DiagramSet {
    /// Whether any leg is polarized.
    pub fn is_polarized(&self) -> bool {
        self.polarizations.iter().any(Option::is_some)
    }

    /// The same subprocess with its outgoing legs in another order: outgoing leg
    /// `order[i]` of `self` becomes outgoing leg `i`. Every diagram is relabelled
    /// with it, its momenta and fermion sign rebuilt from the graph, so the result
    /// is the enumeration the reordered process line would have produced.
    ///
    /// `None` when `order` is not a permutation of the outgoing legs.
    pub fn with_final_order(&self, order: &[usize], model: &UFOModel) -> Option<DiagramSet> {
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
    let _span = info_span!("proc_card").entered();
    for process in &card.processes {
        info!("generate {process} @{}", process.id);
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

/// High-level entry point: expand and generate diagrams for every process of a
/// checked card, on a single thread.
///
/// Returns one `DiagramSet` per concrete particle assignment across all processes.
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

/// Enumerate one `1 → n` decay on its own: one [`DiagramSet`] per concrete
/// assignment of its legs (`w+ > j j` has one per quark pair), every diagram with
/// the decaying particle as external leg `0` and `n_in = 1`.
///
/// This is the enumeration a `1 → n` process line runs, exposed as a unit so a
/// decay can be enumerated apart from any process it attaches to. The automatic
/// lowest-`WEIGHTED` search runs over the decay alone, as MadGraph's
/// `DecayChainAmplitude` generates each decay as a separate amplitude.
pub fn enumerate_decay(
    decay: &SupportedProcess,
    model: &UFOModel,
) -> Result<Vec<DiagramSet>, DiagramError> {
    if decay.initial.len() != 1 {
        return Err(DiagramError::NotADecay {
            process: decay.to_string(),
            n_in: decay.initial.len(),
        });
    }
    generate_from_process(decay, model)
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

/// One helicity state of a subprocess: the unordered `(particle, helicity)`
/// content of each side.
type SummedState = (Vec<(String, i32)>, Vec<(String, i32)>);

/// Every helicity state a subprocess sums over, each as the unordered
/// `(particle, helicity)` content of each side: the physical states two
/// subprocesses of the same particles would both count.
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
        legs.sort();
        fin.sort();
        out.insert((legs, fin));
    }
    Ok(out)
}

fn enumerate(proc_card: &SupportedCard, model: &UFOModel) -> Result<Vec<DiagramSet>, DiagramError> {
    let _span = info_span!("enumerate").entered();
    let started = Instant::now();
    let mut sets = Vec::new();
    // Every process line's subprocesses are summed into one cross section, so a
    // subprocess two lines both produce would be counted twice. MadGraph's own
    // duplicate check compares whole amplitudes, process number included, and
    // lets exactly this through (`generate p p > e+ e-` then `add process
    // u u~ > e+ e-` generates `u u~ > e+ e-` twice), so the refusal here is on
    // the subprocess alone.
    //
    // Two lines may produce the same particles with polarizations that sum
    // over disjoint helicity states (`e+ e- > z{0} h` and `e+ e- > z{T} h`):
    // those add without double counting and are kept. Any state both would
    // count is refused.
    let mut owner: HashMap<FlavourKey, Vec<(usize, usize)>> = HashMap::new();
    for (index, process) in proc_card.processes.iter().enumerate() {
        let process_sets = generate_from_process(process, model)?;
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
        sets.extend(process_sets);
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
    schannels: SChannelFilter,
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

/// Generate diagrams for one process of a checked card.
///
/// Resolves the legs against the model, builds a `DiagramSelector` for each
/// concrete particle assignment, then calls `feyngraph::generate_diagrams`.
///
/// When the process has no explicit coupling constraints, the WEIGHTED coupling
/// order filter is applied automatically: the minimum WEIGHTED value that produces
/// any diagrams is found iteratively, then only diagrams at that value are kept.
/// This mirrors MadGraph's default behaviour of selecting the lowest perturbative
/// order.  WEIGHTED = Σ_i (hierarchy_i × n_i) where hierarchy comes from the
/// UFO `coupling_orders.py` (e.g. QCD→1, QED→2 in the SM). An explicit
/// `WEIGHTED<=n` is that same filter at a bound the card chose.
///
/// The s-channel restrictions filter the converted diagrams inside the search,
/// as they do inside MadGraph's (`find_optimal_process_orders` generates each
/// trial with the process's required and forbidden s-channels): an order whose
/// diagrams they all remove moves the search on.
fn generate_from_process(
    process: &SupportedProcess,
    model: &UFOModel,
) -> Result<Vec<DiagramSet>, DiagramError> {
    if !process.decays.is_empty() {
        return chain::generate_chain(process, model);
    }
    let mut sets = generate_undecayed(process, model)?;
    for diagram in sets.iter_mut().flat_map(|s| s.diagrams.iter_mut()) {
        diagram.provenance.process = process.id;
    }
    Ok(sets)
}

/// [`generate_from_process`] for a process without decays.
fn generate_undecayed(
    process: &SupportedProcess,
    model: &UFOModel,
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
    // `WEIGHTED<=n` (the check admits no other comparison) bounds the diagrams'
    // weighted order; the leftmost constraint wins, as for every order.
    let explicit_weighted = process
        .orders
        .iter()
        .find(|o| o.name == WEIGHTED)
        .map(|o| o.value.max(0) as usize);
    let expanded = ExpandedProcess {
        initial: legs(&process.initial)?,
        final_state: legs(&process.final_state)?,
        polarizations,
        forbidden_particles: forbidden_propagator_names(
            model,
            &process.forbidden_particles,
            &aliases,
        )?,
        orders: process
            .orders
            .iter()
            .filter(|o| o.name != WEIGHTED)
            .cloned()
            .collect(),
        schannels,
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
    let auto_weighted = process.orders.is_empty();
    let expanded = ExpandedProcess {
        orders: capped_orders(&expanded.orders, model),
        ..expanded
    };

    if auto_weighted {
        // No explicit constraints: discover the minimum WEIGHTED order.
        let min_hier = model.order_hierarchy.values().copied().min().unwrap_or(1) as usize;
        let max_hier = model.order_hierarchy.values().copied().max().unwrap_or(2) as usize;
        let min_w = (n_ext - 2) * min_hier;
        let max_w = (n_ext - 2) * max_hier;

        let mut w = min_w;
        loop {
            let sets = generate_sets_inner(&expanded, model, Some(w), &cached_topologies)?;
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
        generate_sets_inner(&expanded, model, explicit_weighted, &cached_topologies)
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

/// Inner generation loop: expand aliases, deduplicate mirror processes, and call
/// feyngraph for each concrete subprocess.  `max_weighted` (when `Some`) adds an
/// extra diagram filter that rejects any diagram whose WEIGHTED order exceeds the
/// given bound.
///
/// `cached_topologies` must be pre-computed by the caller via `generate_topologies`.
/// Coupling constraints are enforced during particle assignment, not topology
/// generation, so no topology filtering is needed here.
fn generate_sets_inner(
    process: &ExpandedProcess,
    model: &UFOModel,
    max_weighted: Option<usize>,
    cached_topologies: &[Topology],
) -> Result<Vec<DiagramSet>, DiagramError> {
    let mut sets = Vec::new();
    // Candidate assignments seen, and the two prefilters' kills: what the
    // enumeration paid for the assignments it never handed to feyngraph.
    let mut candidates = 0usize;
    let mut duplicates = 0usize;
    let mut charge_kills = 0usize;
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
    let mut seen_processes: std::collections::HashSet<SubprocessKey> =
        std::collections::HashSet::new();

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

    'assignments: for (initial, final_state) in itertools::iproduct!(initial_combos, final_combos) {
        candidates += 1;
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

        if !seen_processes.insert(subprocess_key(
            &concrete.initial,
            &concrete.final_state,
            &polarizations,
        )) {
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
        let mut diagrams = container
            .views()
            .map(|view| Diagram::from_view(&view, model))
            .collect::<Result<Vec<_>, _>>()?;
        let enumerated = diagrams.len();
        diagrams.retain(|d| process.schannels.keeps(d, model));
        if diagrams.len() != enumerated {
            debug!(
                "{} > {}: the s-channel restrictions keep {} of {enumerated} diagrams",
                in_refs.join(" "),
                out_refs.join(" "),
                diagrams.len()
            );
        }

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
            polarizations,
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
             charge-violating, {dropped_polarizations} with no helicity left, {} enumerated",
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
    use crate::ufo::sm::{sm_model, sm_parsed_model, SMRestrict};

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
