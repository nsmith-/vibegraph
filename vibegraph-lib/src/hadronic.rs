//! Leading-order cross-section integrands built from the compiled helicity
//! amplitude ([`crate::helas::eval`]), the run-card cut filter ([`crate::cuts`]),
//! and the VEGAS integrator ([`crate::vegas`]).
//!
//! [`FixedBeamIntegrand`] covers fixed-energy partonic beams (`lpp = 0`): an
//! arbitrary MG-validated process with no PDF convolution over any final-state
//! multiplicity, sampled with flat RAMBO or — once
//! [`FixedBeamIntegrand::use_multichannel`] has run — a resonance-aware
//! per-diagram multichannel map that resolves Breit–Wigner peaks. Proton beams
//! (`lpp = 1`) are [`crate::proton`]'s flavour-group path, which shares this
//! module's subprocess compilation, scale prescription and averaging factors.
//!
//! The initial-state flux and spin×colour averaging factors are derived per
//! process from its incoming legs ([`initial_spin_color_average`]).
//!
//! # Frames
//!
//! A matrix element is evaluated in the **partonic CM** with the beams along
//! ±z — the frame the helicity-pruned [`BoundAmplitude::eval_m2`] requires — where
//! `|M|²` is a Lorentz invariant. A cut filter operates in the **lab frame**,
//! whose rapidity/pT observables are not z-boost invariant.

use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};

use rand::SeedableRng;
use thiserror::Error;
use thread_local::ThreadLocal;

use crate::artifact::{ChannelSampler, SamplerTopology};
use crate::budget::{
    integrate_channels, BlockAllocation, Budget, ConvergenceReport, StopSignal, MIN_CHANNEL_NEVAL,
};
use crate::coupling::alphas::{AlphaSError, AlphaSSource};
use crate::coupling::cluster::configs::{derive_channels, DerivedChannels};
use crate::coupling::cluster::graph::{ColorTable, MergeTablesByOrder};
use crate::coupling::cluster::setclscales::ScaleRefusal;
use crate::coupling::scales::{ClusterInput, EventScales, ScaleChoice, ScaleError, ScaleEvent};
use crate::cuts::{CutError, Cuts, ExternalLeg};
use crate::diagrams::diagram::Diagram;
use crate::diagrams::{DiagramError, DiagramSet};
use crate::helas::eval::{AmplitudeEvaluator, BoundAmplitude, ScaleAwareAmplitude, ScratchSpace};
use crate::helas::repr::lorentz::LorentzVector;
use crate::pdf::grid::AlphaSInfo;
use crate::phasespace::rng::{SubStream, SCALE_DRAW_STREAM_BASE};
use crate::phasespace::{
    identical_particle_factor, AlphaAdaptation, Channel, Combiner, DiagramChannel, MultiChannel,
    PhaseSpaceMap, PhaseSpacePoint, RamboChannel, GEV2_TO_PB,
};
use crate::progress;
use crate::runcard::RunCard;
use crate::select::select_index;
use crate::ufo::{EvaluatedModel, UFOModel};
use crate::unweight::ChannelIntegrand;
use crate::vegas::{IterationCombination, VegasGrid, VegasResult};

type V = LorentzVector<f64>;

pub const VEGAS_NBINS: usize = 64;
pub const VEGAS_ALPHA: f64 = 1.5;

/// Grid-damping exponent used once a resonance-aware multichannel map is
/// installed, in place of the [`VEGAS_ALPHA`] Lepage recommends for a raw
/// integrand.
///
/// Lepage's `1.5` assumes the grid must *discover* the integrand's structure. A
/// converged multichannel map has already flattened the peaks it knows about, so
/// what remains in the unit hypercube is close to featureless and the per-bin `f²`
/// statistics are dominated by sampling noise. At `1.5` the refinement amplifies
/// that noise: the grid concentrates into a spurious bin, later iterations sample
/// a narrow region where the integrand is smooth and so report a small integral
/// with a small variance, and — since iterations are combined by `1/σ²` — those
/// confident, wrong iterations dominate the result. Measured on
/// `e+ e- > mu+ mu- ta+ ta-` (25 channels), `1.5` collapses one seed in five to 36%
/// of the banked sigma with `chi2/dof ≈ 580`, while `0.5` is stable across every
/// seed *and* halves the error — the grid still absorbing the residual structure
/// the channel maps do not cover.
pub const VEGAS_ALPHA_MAPPED: f64 = 0.5;

/// RNG substream index the multichannel α-adaptation survey draws on, kept distinct
/// from the VEGAS integration substreams so the survey and the integral neither
/// share nor correlate their sampling sequences.
const MULTICHANNEL_ADAPT_STREAM: u64 = 0xA1FA_5EED;

/// First `ChaCha8Rng` stream id of the per-channel integrations, offset by the
/// channel index. Channel `j` draws from stream `CHANNEL_STREAM_BASE + j`, so the
/// terms of the channel-split estimator sample structurally independent sequences
/// under one seed and each replays on its own.
pub(crate) const CHANNEL_STREAM_BASE: u64 = 0xC7A0_0000;

/// RNG seed and draw budget for the setup-time probe that resolves a dynamic scale
/// once before integration begins.
pub(crate) const SCALE_PROBE_SEED: u64 = 0x5CA1_E9E0;
pub(crate) const SCALE_PROBE_DRAWS: usize = 64;

#[derive(Debug, Error)]
pub enum HadronicError {
    #[error("diagram enumeration failed: {0}")]
    Diagram(#[from] DiagramError),
    #[error("cut compilation failed: {0}")]
    Cut(#[from] CutError),
    #[error("amplitude compilation failed: {0}")]
    Compile(String),
    #[error("proc card generated no non-empty subprocess")]
    NoSubprocess,
    #[error(
        "fixed-energy beams require every subprocess to share the same external \
         particle content, but the generated subprocesses differ"
    )]
    InconsistentExternals,
    #[error("run card scale prescription: {0}")]
    Scale(#[from] ScaleError),
    #[error("running strong coupling: {0}")]
    AlphaS(#[from] AlphaSError),
    #[error("parameter card supplies no strong coupling to run from")]
    MissingAlphaS,
    #[error("channel forests for the clustering scale: {0}")]
    Channels(#[from] crate::coupling::cluster::configs::ConfigError),
    #[error(
        "every sampled point's factorisation scale fell below the 2 GeV floor a parton density \
         is fitted down to, so the cross section this card asks for is zero by construction"
    )]
    FactorisationScaleBelowFloor,
}

/// The run card's per-event scale prescription, bound to one process.
///
/// Holds the compiled [`ScaleChoice`], the channel forests its clustering branch
/// consults, and the running coupling that turns `μR` into `αs(μR)`.
///
/// The coupling is constructed only on request. `coupling::alphas` refuses a
/// `pdlabel` whose `αs` MadGraph delegates to LHAPDF, and a matrix element with no
/// strong coupling in it has no reason to meet that refusal, so a caller whose
/// amplitudes report [`ScaleAwareAmplitude::depends_on_alpha_s`] false asks for no
/// coupling and runs regardless of the label.
#[derive(Clone, Debug)]
pub struct EventScaleSource {
    kind: ScaleSourceKind,
    alpha_s: Option<AlphaSSource>,
    /// Whether the card's single-diagram enhancement weights an integration
    /// configuration by its squared amplitude alone.
    ///
    /// `matrix1.f`'s `MULTI_CHANNEL` block weights configuration `c` by
    /// `AMP2_c · CC_c`, and `genps.f`'s `get_channel_cut` returns `CC_c = 1`
    /// only when `sde_strat == 1` and `tmin_for_channel == -1`. Off that
    /// conjunction the weight is a product of propagator denominators — at
    /// `sde_strat == 2` the squared amplitude is discarded outright — and this
    /// crate has no implementation of it, so the scale keeps the sampler's
    /// channel there instead of drawing a configuration this rule does not
    /// describe.
    amp2_configuration_weights: bool,
}

#[derive(Clone, Debug)]
enum ScaleSourceKind {
    /// Every scale is a constant, so no event kinematics are read at all.
    Constant(EventScales),
    PerEvent {
        choice: ScaleChoice,
        /// One entry per subprocess whose diagrams a sampling channel can be
        /// drawn from — a single entry for a fixed-beam run, one per flavour
        /// group for a hadronic one. `None` for a prescription whose dynamic
        /// value is one of `setscales.f`'s closed forms, which read no merge
        /// graph.
        channels: Option<Box<[Channels]>>,
    },
}

/// Which sampling channel a phase-space point was drawn from, in the terms the
/// scale prescription reads it.
///
/// The cluster scale is a function of the event **and** of the channel being
/// integrated, so a point carries the channel that produced it all the way to
/// [`EventScaleSource::scales`]. A sampling channel is one diagram of one
/// subprocess, which is exactly the pair below.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampledChannel {
    /// Which channel set the draw belongs to: the flavour group on a hadronic
    /// run, and `0` wherever one subprocess supplies every channel.
    pub group: usize,
    /// The channel's diagram, indexed into the diagram slice its channel set
    /// was derived from.
    pub diagram: usize,
}

impl SampledChannel {
    /// The `diagram`-th channel of the only channel set — a fixed-beam run,
    /// where the sampling channel index *is* the diagram index.
    pub fn sole(diagram: usize) -> Self {
        SampledChannel { group: 0, diagram }
    }
}

/// The clustering's static side of one subprocess.
#[derive(Clone, Debug)]
pub struct Channels {
    derived: DerivedChannels,
    colors: ColorTable,
    /// The merge lookup each coupling order clusters against, hoisted out of the
    /// per-event path: it is a function of the channel forests and the order
    /// alone, and every event of a channel asks for the same one.
    tables: MergeTablesByOrder,
    /// The channel named by a draw that has none of its own.
    ///
    /// Two callers reach it: a flat sampler, which draws no channel at all, and
    /// a sampling channel whose diagram the vertex filter dropped, whose region
    /// of phase space MadGraph covers from the surviving channels instead. The
    /// banked replay in `validate_scales.rs` reports, per run, how many events
    /// the choice moves at all — it is the measurement this default rests on.
    default_config: usize,
}

impl Channels {
    pub fn input(&self, this_config: usize) -> ClusterInput<'_> {
        ClusterInput {
            set: &self.derived.set,
            colors: &self.colors,
            this_config,
            iproc: 1,
            tables: Some(self.tables.of(this_config)),
        }
    }

    /// The integration channel a point drawn from the `diagram`-th sampling
    /// channel was generated in, falling back to the set's default channel where
    /// that diagram has none.
    pub fn config_of_channel(&self, diagram: usize) -> usize {
        self.config_of_diagram(diagram)
            .unwrap_or(self.default_config)
    }

    /// The integration channel derived from one diagram, or `None` where the
    /// vertex filter dropped it. The mapping itself, without the fallback.
    pub fn config_of_diagram(&self, diagram: usize) -> Option<usize> {
        self.derived
            .config_of_diagram
            .get(diagram)
            .copied()
            .flatten()
    }

    /// How many diagrams the channel forests were derived from — the range of
    /// [`config_of_channel`](Channels::config_of_channel)'s argument, and the
    /// number of sampling channels the same diagram slice produces.
    pub fn diagram_count(&self) -> usize {
        self.derived.config_of_diagram.len()
    }

    /// How many sampling channels name no integration channel of their own.
    pub fn unmapped_channels(&self) -> usize {
        self.derived
            .config_of_diagram
            .iter()
            .filter(|c| c.is_none())
            .count()
    }

    /// How many integration channels the process has, which is the range
    /// [`Channels::input`] accepts.
    pub fn len(&self) -> usize {
        self.derived.set.configs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.derived.set.configs.is_empty()
    }
}

impl EventScaleSource {
    /// One constant scale on both beams and no running coupling — the prescription
    /// a caller that supplies `μF` directly is asking for.
    pub fn constant(mu: f64) -> Self {
        EventScaleSource {
            kind: ScaleSourceKind::Constant(EventScales {
                mu_r: mu,
                mu_f: [mu, mu],
            }),
            alpha_s: None,
            amp2_configuration_weights: false,
        }
    }

    /// Compile a run card's prescription. `channels` carries one channel set per
    /// subprocess a sampling channel can be drawn from and is consulted only by
    /// the clustering branch; `needs_alpha_s` decides whether a running coupling
    /// is built at all.
    pub fn from_run_card(
        card: &RunCard,
        param_card_as: f64,
        grid: Option<&AlphaSInfo>,
        channels: Option<Vec<Channels>>,
        needs_alpha_s: bool,
    ) -> Result<Self, HadronicError> {
        let choice = ScaleChoice::from_run_card(card)?;
        let alpha_s = needs_alpha_s
            .then(|| AlphaSSource::from_run_card(card, param_card_as, grid))
            .transpose()?;
        let kind = if choice.is_fully_fixed() {
            // A fully fixed prescription returns the card's constants without
            // reading the event, so any event resolves it.
            ScaleSourceKind::Constant(choice.scales(&ScaleEvent {
                incoming: [[0.0; 4]; 2],
                outgoing: &[],
            })?)
        } else {
            ScaleSourceKind::PerEvent {
                choice,
                channels: channels.map(Vec::into_boxed_slice),
            }
        };
        Ok(EventScaleSource {
            kind,
            alpha_s,
            amp2_configuration_weights: card.int("SDE_strategy") == 1
                && card.float("tmin_for_channel") == -1.0,
        })
    }

    /// Whether a point's integration configuration is drawn from the squared
    /// amplitude rather than taken from the sampling channel it came from.
    ///
    /// True only where all three hold: the prescription reads the event at all,
    /// it consults channel forests (the kT clustering, not one of `setscales.f`'s
    /// closed forms), and the card's enhancement weight is the squared amplitude
    /// alone.
    pub fn draws_configuration(&self) -> bool {
        self.amp2_configuration_weights
            && matches!(
                &self.kind,
                ScaleSourceKind::PerEvent {
                    channels: Some(_),
                    ..
                }
            )
    }

    /// The scales, when they are the same on every event.
    pub fn constant_scales(&self) -> Option<EventScales> {
        match self.kind {
            ScaleSourceKind::Constant(scales) => Some(scales),
            ScaleSourceKind::PerEvent { .. } => None,
        }
    }

    /// The strong coupling's source, or `None` when none was asked for.
    pub fn alpha_s(&self) -> Option<&AlphaSSource> {
        self.alpha_s.as_ref()
    }

    /// The channel forests the clustering branch consults, one set per
    /// subprocess a sampling channel can be drawn from.
    pub fn channels(&self) -> Option<&[Channels]> {
        match &self.kind {
            ScaleSourceKind::Constant(_) => None,
            ScaleSourceKind::PerEvent { channels, .. } => channels.as_deref(),
        }
    }

    /// The scales for one event, from lab-frame momenta and the sampling channel
    /// the point was drawn from.
    ///
    /// # Panics
    ///
    /// If `channel` names a group this prescription has no channel set for.
    pub fn scales(
        &self,
        incoming: [[f64; 4]; 2],
        outgoing: &[[f64; 4]],
        channel: SampledChannel,
    ) -> Result<EventScales, ScaleError> {
        match &self.kind {
            ScaleSourceKind::Constant(scales) => Ok(*scales),
            ScaleSourceKind::PerEvent { choice, channels } => {
                let event = ScaleEvent { incoming, outgoing };
                match channels {
                    Some(sets) => {
                        let set = sets.get(channel.group).unwrap_or_else(|| {
                            panic!(
                                "a point was drawn in channel group {} of {}",
                                channel.group,
                                sets.len()
                            )
                        });
                        choice.cluster_scales(
                            &event,
                            &set.input(set.config_of_channel(channel.diagram)),
                        )
                    }
                    None => choice.scales(&event),
                }
            }
        }
    }

    /// [`scales`](Self::scales), with the factorisation-floor refusal separated
    /// from the errors that mean the prescription itself does not apply.
    ///
    /// This is the only place the two are told apart. `reweight.f` answers a
    /// point below the floor by zeroing its weight and carrying on, so a caller
    /// that evaluates points has to be able to say "no weight" without saying
    /// "this run cannot proceed" — and every other error means exactly the
    /// latter, so it stays an `Err` and stays fatal at the call site.
    pub fn point_scales(
        &self,
        incoming: [[f64; 4]; 2],
        outgoing: &[[f64; 4]],
        channel: SampledChannel,
    ) -> Result<PointScales, ScaleError> {
        match self.scales(incoming, outgoing, channel) {
            Ok(scales) => Ok(PointScales::Scales(scales)),
            Err(ScaleError::Clustering(ScaleRefusal::FactorisationFloor)) => {
                Ok(PointScales::Vetoed)
            }
            Err(other) => Err(other),
        }
    }
}

/// What resolving one point's scales produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointScales {
    /// The scales to evaluate this point at.
    Scales(EventScales),
    /// The point carries no weight: a beam carrying a parton density ended below
    /// the factorisation floor, where MadGraph zero-weights the point and moves
    /// on rather than stopping.
    Vetoed,
}

/// What installing a run card's per-event scale did to one integrand.
#[derive(Clone, Debug)]
pub struct RunningCouplingReport {
    /// Whether any subprocess's matrix element moves with the strong coupling. When
    /// false no running coupling was constructed and the amplitudes are left where
    /// they were bound.
    pub depends_on_alpha_s: bool,
    /// How many integration channels the clustering branch was given, summed
    /// over its channel sets, or `None` where the prescription reads no merge
    /// graph.
    pub channels: Option<usize>,
    /// How many sampling channels of those sets name no integration channel of
    /// their own, because MadGraph's vertex filter drops the diagram they were
    /// built on. Such a draw takes the set's default channel, so a run that has
    /// any says so rather than absorbing it.
    pub unmapped_channels: usize,
    /// The scales, when the prescription resolves to constants — then no event
    /// kinematics are read and the coupling is applied once rather than per point.
    pub constant_scales: Option<EventScales>,
    /// The coupling at that constant renormalisation scale.
    pub constant_alpha_s: Option<f64>,
    /// The coupling the amplitudes were bound at.
    pub alpha_s_ref: Option<f64>,
    /// Why a subprocess must re-evaluate the whole model on each scale change
    /// instead of scaling its constant pools — roughly two orders of magnitude
    /// slower per event, so it is reported rather than absorbed.
    pub fallbacks: Vec<String>,
}

/// Components in the `[E, px, py, pz]` layout the scale prescription reads.
pub(crate) fn components(p: &V) -> [f64; 4] {
    [p.e(), p.px(), p.py(), p.pz()]
}

/// Boost a four-momentum along z with velocity `beta` (CM → lab for `beta > 0`).
pub(crate) fn boost_z(p: V, beta: f64) -> V {
    let gamma = 1.0 / (1.0 - beta * beta).sqrt();
    let e = gamma * (p.e() + beta * p.pz());
    let pz = gamma * (p.pz() + beta * p.e());
    V::new(e, p.px(), p.py(), pz)
}

/// Compile and helicity-prune a class amplitude from its representative
/// subprocess `DiagramSet`.
pub fn compile_class(
    set: &DiagramSet,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<AmplitudeEvaluator, HadronicError> {
    let mut evaluator = AmplitudeEvaluator::compile(set, model)
        .map_err(|e| HadronicError::Compile(e.to_string()))?;
    evaluator.prune_zero_helicities(evaluated);
    Ok(evaluator)
}

/// Number of spin (helicity / polarization) states of a particle, from its UFO
/// spin code (`2s+1`) and whether it is massless. A massless vector has no
/// longitudinal mode, so it carries two states rather than three.
fn spin_state_count(spin_code: i32, massless: bool) -> usize {
    match spin_code.abs() {
        1 => 1,                          // scalar
        2 => 2,                          // fermion
        3 => usize::from(!massless) + 2, // vector: 2 (massless) or 3 (massive)
        5 => 5,                          // spin-2
        other => panic!("unsupported spin code {other} for the spin average"),
    }
}

/// The initial-state spin×colour averaging factor `1 / Π_a (n_spin,a · n_colour,a)`
/// over the incoming legs of a compiled process.
///
/// Derived from the UFO particle data — spin code and colour-representation
/// dimension (`|color|`: singlet 1, fundamental 3, adjoint 8) — and the resolved
/// masses, so a process supplies its own averaging denominator instead of a
/// hand-coded constant (`1/(2·2·3·3) = 1/36` for a quark–antiquark initial state,
/// `1/(2·2) = 1/4` for `e⁺e⁻`, `1/(2·8·2·8) = 1/256` for `gg`).
pub fn initial_spin_color_average(
    eval: &AmplitudeEvaluator,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> f64 {
    let mut denom = 1.0f64;
    for &id in eval.external_particles().iter().take(eval.n_in()) {
        let particle = model.particle(id);
        let massless = evaluated.mass(id) == 0.0;
        let n_spin = spin_state_count(particle.spin, massless);
        let n_color = particle.color.unsigned_abs() as usize;
        denom *= (n_spin * n_color) as f64;
    }
    1.0 / denom
}

/// Build the [`ExternalLeg`] list (incoming legs first, then outgoing) for a
/// compiled process, reading PDG codes and pole masses from the model — the
/// input [`Cuts::compile`] classifies.
pub fn process_external_legs(
    eval: &AmplitudeEvaluator,
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Vec<ExternalLeg> {
    eval.external_particles()
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let pdg = model.particle(id).pdg_code as i32;
            let mass = evaluated.mass(id);
            if i < eval.n_in() {
                ExternalLeg::incoming(pdg, mass)
            } else {
                ExternalLeg::outgoing(pdg, mass)
            }
        })
        .collect()
}

/// Compile every non-empty subprocess of a generated proc card into a
/// helicity-pruned evaluator, requiring that they share one external-particle
/// sequence so a single RAMBO mass list and one cut filter serve them all.
pub fn compile_subprocesses(
    sets: &[DiagramSet],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
) -> Result<Vec<AmplitudeEvaluator>, HadronicError> {
    let total = sets.iter().filter(|s| !s.diagrams.is_empty()).count() as u64;
    let mut evals = Vec::new();
    for set in sets {
        if set.diagrams.is_empty() {
            continue;
        }
        evals.push(compile_class(set, model, evaluated)?);
        progress::step(progress::stage::COMPILE, evals.len() as u64, Some(total));
    }
    if evals.is_empty() {
        return Err(HadronicError::NoSubprocess);
    }
    let first: Vec<_> = evals[0].external_particles().to_vec();
    if evals[1..]
        .iter()
        .any(|e| e.external_particles() != first.as_slice())
    {
        return Err(HadronicError::InconsistentExternals);
    }
    Ok(evals)
}

/// What one subprocess of a summed matrix element computes, carrying no
/// evaluation state.
///
/// This is the half of a subprocess an integrand shares between threads. Setup
/// configures it — turning it scale-aware, moving it to a constant prescription's
/// coupling — and every thread then [binds](Self::bind) a [`BoundSubprocess`] of
/// its own from it.
pub(crate) struct SubprocessProto<'a> {
    amp: ProtoAmplitude<'a>,
    /// `1 / Π_s n_s!` over *this* amplitude's outgoing legs
    /// ([`identical_particle_factor`]). Held per subprocess because the terms of a
    /// summed matrix element need not share an outgoing multiset even when they
    /// share a phase-space map.
    symmetry_factor: f64,
}

/// A subprocess amplitude, held at the parameter card's own strong coupling or at
/// the one each event's renormalisation scale implies.
enum ProtoAmplitude<'a> {
    Fixed(&'a BoundAmplitude<'a, f64>),
    Running(ScaleAwareAmplitude<'a, f64>),
}

/// One thread's copy of a [`SubprocessProto`]: its own evaluation arena and — where
/// the coupling runs — its own constant pools.
///
/// The scale-aware form rewrites those pools per event, so it is mutable state and
/// is never shared. Sharing one would let two threads read each other's coupling,
/// giving a silently wrong `|M|²` with no panic and no NaN. The [`RefCell`]s are
/// what let evaluation take `&self` within the one thread that owns the copy.
pub(crate) struct BoundSubprocess<'a> {
    amp: SubAmplitude<'a>,
    scratch: RefCell<ScratchSpace<f64>>,
    symmetry_factor: f64,
}

enum SubAmplitude<'a> {
    Fixed(&'a BoundAmplitude<'a, f64>),
    Running(RefCell<ScaleAwareAmplitude<'a, f64>>),
}

impl<'a> SubprocessProto<'a> {
    pub(crate) fn fixed(amp: &'a BoundAmplitude<'a, f64>) -> Self {
        let eval = amp.evaluator();
        SubprocessProto {
            symmetry_factor: identical_particle_factor(&eval.external_particles()[eval.n_in()..]),
            amp: ProtoAmplitude::Fixed(amp),
        }
    }

    /// A copy for one thread to evaluate through.
    ///
    /// Copies are interchangeable, not merely equivalent: every pool value a scale
    /// change writes is recomputed from the card's own `αs`, so a subprocess's
    /// `|M|²` is a function of `(αs, momenta)` and not of the sequence of points it
    /// has already seen. Which thread evaluates a point therefore cannot move its
    /// value.
    pub(crate) fn bind(&self) -> BoundSubprocess<'a> {
        let (amp, scratch) = match &self.amp {
            ProtoAmplitude::Fixed(amp) => (SubAmplitude::Fixed(amp), amp.scratch_space()),
            ProtoAmplitude::Running(amp) => {
                let mine = amp.fork();
                let scratch = mine.amplitude().scratch_space();
                (SubAmplitude::Running(RefCell::new(mine)), scratch)
            }
        };
        BoundSubprocess {
            amp,
            scratch: RefCell::new(scratch),
            symmetry_factor: self.symmetry_factor,
        }
    }

    /// The evaluator this subprocess was built from, the input a scale-aware copy
    /// is derived from.
    pub(crate) fn evaluator(&self) -> &'a AmplitudeEvaluator {
        match &self.amp {
            ProtoAmplitude::Fixed(amp) => amp.evaluator(),
            ProtoAmplitude::Running(amp) => amp.amplitude().evaluator(),
        }
    }

    /// Replace the amplitude by a scale-aware copy of itself, bound against
    /// `evaluated`. The copy starts at the parameter card's own coupling, with
    /// pools bit-for-bit those of the amplitude it replaces.
    fn make_scale_aware(&mut self, evaluated: &EvaluatedModel) {
        self.amp =
            ProtoAmplitude::Running(ScaleAwareAmplitude::<f64>::new(self.evaluator(), evaluated));
    }

    fn scale_aware(&self) -> Option<&ScaleAwareAmplitude<'a, f64>> {
        match &self.amp {
            ProtoAmplitude::Fixed(_) => None,
            ProtoAmplitude::Running(amp) => Some(amp),
        }
    }

    pub(crate) fn set_alpha_s(&mut self, alpha_s: f64) {
        if let ProtoAmplitude::Running(amp) = &mut self.amp {
            amp.set_alpha_s(alpha_s);
        }
    }
}

impl<'a> BoundSubprocess<'a> {
    /// This subprocess's own identical-particle symmetry factor.
    pub(crate) fn symmetry_factor(&self) -> f64 {
        self.symmetry_factor
    }

    /// The bound evaluator this subprocess was built from, the input a scale-aware
    /// copy is derived from.
    pub(crate) fn evaluator(&self) -> &'a AmplitudeEvaluator {
        match &self.amp {
            SubAmplitude::Fixed(amp) => amp.evaluator(),
            SubAmplitude::Running(amp) => amp.borrow().amplitude().evaluator(),
        }
    }

    pub(crate) fn set_alpha_s(&self, alpha_s: f64) {
        if let SubAmplitude::Running(amp) = &self.amp {
            amp.borrow_mut().set_alpha_s(alpha_s);
        }
    }

    pub(crate) fn eval_m2(&self, momenta: &[V]) -> f64 {
        let scratch = &mut self.scratch.borrow_mut();
        match &self.amp {
            SubAmplitude::Fixed(amp) => amp.eval_m2(momenta, scratch),
            SubAmplitude::Running(amp) => amp.borrow().eval_m2(momenta, scratch),
        }
    }

    /// The per-configuration `AMP2` alone, at the current coupling — the weight
    /// vector MadEvent's single-diagram enhancement forms its configuration
    /// distribution from.
    pub(crate) fn eval_amp2(&self, momenta: &[V], amp2: &mut [f64]) {
        let scratch = &mut self.scratch.borrow_mut();
        match &self.amp {
            SubAmplitude::Fixed(amp) => amp.eval_amp2(momenta, scratch, amp2),
            SubAmplitude::Running(amp) => {
                amp.borrow().amplitude().eval_amp2(momenta, scratch, amp2)
            }
        }
    }

    /// The per-combination `|M_c|²`, per-configuration `AMP2` and per-flow `JAMP2`
    /// diagonals at the current coupling — the three categorical weight vectors an
    /// event record's helicity, integration configuration and colour flow are drawn
    /// from. None enters `eval_m2`, so none moves the cross section.
    pub(crate) fn eval_diagonals(
        &self,
        momenta: &[V],
        hel_m2: &mut [f64],
        amp2: &mut [f64],
        jamp2: &mut [f64],
    ) {
        let scratch = &mut self.scratch.borrow_mut();
        // The running form's own amplitude carries the pools the current scale set,
        // so every diagonal is read at the coupling `eval_m2` was taken at.
        match &self.amp {
            SubAmplitude::Fixed(amp) => {
                amp.eval_hel_m2(momenta, scratch, hel_m2);
                amp.eval_amp2(momenta, scratch, amp2);
                amp.eval_jamp2(momenta, scratch, jamp2);
            }
            SubAmplitude::Running(running) => {
                let amp = running.borrow();
                amp.amplitude().eval_hel_m2(momenta, scratch, hel_m2);
                amp.amplitude().eval_amp2(momenta, scratch, amp2);
                amp.amplitude().eval_jamp2(momenta, scratch, jamp2);
            }
        }
    }
}

/// What turning a set of subprocesses scale-aware revealed about them.
pub(crate) struct ScaleAwareness {
    pub(crate) depends_on_alpha_s: bool,
    fallbacks: Vec<String>,
    alpha_s_ref: Option<f64>,
}

/// Replace every subprocess amplitude by a scale-aware copy of itself.
pub(crate) fn make_subs_scale_aware(
    subs: &mut [SubprocessProto<'_>],
    evaluated: &EvaluatedModel,
) -> ScaleAwareness {
    for sub in subs.iter_mut() {
        sub.make_scale_aware(evaluated);
    }
    let scale_aware = || subs.iter().filter_map(SubprocessProto::scale_aware);
    ScaleAwareness {
        depends_on_alpha_s: scale_aware().any(|a| a.depends_on_alpha_s()),
        fallbacks: scale_aware()
            .filter_map(|a| a.fallback().map(|f| f.to_string()))
            .collect(),
        alpha_s_ref: scale_aware().next().map(|a| a.alpha_s_ref()),
    }
}

/// Compile the run card's prescription for a process, deriving the channel
/// forests its clustering branch consults from the process's own diagrams.
///
/// One `(representative amplitude, diagrams)` pair per subprocess a sampling
/// channel can be drawn from, in the order the sampler pools them: a fixed-beam
/// run passes one, a hadronic run one per flavour group. The pairing is what
/// makes a drawn channel nameable to the scale — channel `j` of set `g` is
/// diagram `j` of `subprocesses[g].1`.
///
/// The coupling is built only when some subprocess actually moves with it, which is
/// what keeps a matrix element with no QCD in it away from a `pdlabel` whose
/// `αs` lives in a PDF set the caller may not have loaded. `grid` is that set's
/// `AlphaS_*` metadata, for the label that demands it.
pub(crate) fn compile_scale_source(
    subprocesses: &[(&AmplitudeEvaluator, &[Diagram])],
    model: &UFOModel,
    evaluated: &EvaluatedModel,
    card: &RunCard,
    grid: Option<&AlphaSInfo>,
    needs_alpha_s: bool,
) -> Result<EventScaleSource, HadronicError> {
    let colors = ColorTable::new(
        model
            .particles
            .values()
            .map(|p| (p.pdg_code, p.color))
            .collect::<Vec<(i64, i32)>>(),
        card.maxjetflavor,
    );
    let mut sets = Vec::with_capacity(subprocesses.len());
    for (rep, diagrams) in subprocesses {
        let derived = derive_channels(
            diagrams,
            rep.external_particles(),
            rep.n_in(),
            model,
            evaluated,
        )?;
        let tables = MergeTablesByOrder::build(&derived.set);
        sets.push(Channels {
            derived,
            colors: colors.clone(),
            tables,
            default_config: 1,
        });
    }
    let param_card_as = evaluated.alpha_s().ok_or(HadronicError::MissingAlphaS)?;
    EventScaleSource::from_run_card(card, param_card_as, grid, Some(sets), needs_alpha_s)
}

/// Hold every subprocess at the coupling a constant prescription implies, and
/// assemble the report describing what was installed.
pub(crate) fn constant_scale_report(
    subs: &mut [SubprocessProto<'_>],
    source: Option<&EventScaleSource>,
    awareness: ScaleAwareness,
) -> RunningCouplingReport {
    let constant_scales = source.and_then(EventScaleSource::constant_scales);
    let constant_alpha_s = match (constant_scales, source.and_then(|s| s.alpha_s())) {
        (Some(scales), Some(running)) => Some(running.eval(scales.mu_r)),
        _ => None,
    };
    if let Some(alpha_s) = constant_alpha_s {
        for sub in subs.iter_mut() {
            sub.set_alpha_s(alpha_s);
        }
    }
    RunningCouplingReport {
        depends_on_alpha_s: awareness.depends_on_alpha_s,
        channels: source
            .and_then(EventScaleSource::channels)
            .map(|sets| sets.iter().map(Channels::len).sum()),
        unmapped_channels: source
            .and_then(EventScaleSource::channels)
            .map(|sets| sets.iter().map(Channels::unmapped_channels).sum())
            .unwrap_or(0),
        constant_scales,
        constant_alpha_s,
        alpha_s_ref: awareness.alpha_s_ref,
        fallbacks: awareness.fallbacks,
    }
}

/// A ready-to-integrate cross section for a **fixed-energy, no-PDF** beam
/// configuration (`lpp = 0`) and an arbitrary final-state multiplicity, sampled
/// under VEGAS.
///
/// The incoming particles *are* the beam particles: `√ŝ = E₁ + E₂` is fixed, so
/// there is no `τ`/`x` sampling and no PDF luminosity. The phase-space
/// [`PhaseSpaceMap`] maps the VEGAS uniforms to `n` on-shell final-state momenta
/// summing to `(√ŝ, 0, 0, 0)` and supplies the invariant-volume weight; the two
/// beams sit at `√ŝ/2` along ±z, so the full external set is already the
/// partonic-CM, ±z-beam frame the helicity-pruned [`BoundAmplitude::eval_m2`]
/// requires, and (for symmetric beams) the lab frame coincides with it — the same
/// momenta feed the cut filter.
///
/// The map is flat [`RamboChannel`] by default. [`use_multichannel`] swaps in a
/// resonance-aware per-diagram [`MultiChannel`] combiner, α-adapted to this very
/// integrand, so a narrow Breit–Wigner peak (which flat RAMBO under-samples) is
/// importance-mapped and the integral converges at far lower variance. Both maps
/// carry the *same* invariant-volume weight normalisation (`R_n`, no `2π`), so the
/// master formula below is unchanged — only the sampling density is.
///
/// [`use_multichannel`]: FixedBeamIntegrand::use_multichannel
///
/// # Master formula
///
/// ```text
/// σ̂ = 1/(2ŝ) · ⟨spin·colour avg⟩ · ∫ dΦ_n Σ_sub S_sub |M_sub|²
///    = 1/(2ŝ) · avg · (2π)^{4−3n} · ⟨weight · Σ_sub S_sub |M_sub|²⟩_uniform
/// ```
///
/// where `|M_sub|²` is a subprocess's colour+helicity-summed matrix element
/// ([`eval_m2`]), the `(2π)^{4−3n}` factor turns the map's invariant volume `R_n`
/// into the full `dΦ_n` measure, and `S_sub = 1/Π_s n_s!` is that subprocess's own
/// identical-particle symmetry factor ([`identical_particle_factor`]), undoing
/// `dΦ_n`'s over-counting of the permutations of its identical outgoing legs. It
/// sits inside the sum because subprocesses sharing one map — one outgoing mass
/// list — need not share an outgoing multiset.
///
/// [`eval_m2`]: BoundAmplitude::eval_m2
pub struct FixedBeamIntegrand<'a> {
    subs: Vec<SubprocessProto<'a>>,
    cuts: &'a Cuts,
    sqrt_s: f64,
    /// Unit-hypercube → phase-space map over the outgoing legs, on the fixed `√ŝ`
    /// and masses: flat [`RamboChannel`] by default, a resonance-aware
    /// [`MultiChannel`] once [`use_multichannel`](Self::use_multichannel) has run.
    sampler: Sampler,
    /// What the composition rule chose for each channel of an installed
    /// multichannel map, read off the channels as they were built and kept
    /// because they are type-erased behind [`Channel`] afterwards. Empty under
    /// flat RAMBO, which is not a rule-based composition.
    channel_samplers: Vec<ChannelSampler>,
    /// The outgoing pole masses in leg order, the map's targets.
    final_masses: Vec<f64>,
    /// `1 / Π_a (n_spin · n_colour)` over the incoming legs.
    spin_color_avg: f64,
    /// The `(2π)^{4−3n}` measure factor.
    lips_2pi: f64,
    /// Beam energy `√ŝ/2`.
    beam_e: f64,
    /// Grid-damping exponent for the VEGAS pass, following the active sampler:
    /// [`VEGAS_ALPHA`] over the raw flat map, [`VEGAS_ALPHA_MAPPED`] once a
    /// multichannel map has already flattened the integrand's known peaks.
    vegas_alpha: f64,
    /// The run card's per-event renormalisation scale, once
    /// [`use_running_coupling`](Self::use_running_coupling) has installed it.
    /// `None`, or a prescription that resolves to a constant, leaves every
    /// subprocess at one coupling and costs nothing per point.
    scales: Option<EventScaleSource>,
    /// One evaluation context per thread that has evaluated a point, forked from
    /// `subs` on first use. See [`FixedBeamScratch`].
    scratch: ThreadLocal<FixedBeamScratch<'a>>,
    /// Length the configuration draw's `AMP2` buffer is sized to, zero where no
    /// draw runs. Held on the integrand because a per-thread scratch is built
    /// after the prescription that decides it.
    amp2_len: usize,
    /// The coupling the configuration draw forms `AMP2` at, so the drawn
    /// configuration is a function of the momenta and not of whatever scale the
    /// previous point left bound. `None` where no draw runs.
    amp2_alpha_s: Option<f64>,
    /// Points whose `AMP2` carried no probability at all, where the draw kept the
    /// sampling channel instead. Such a point's every diagram amplitude vanishes,
    /// so it carries no weight either — the count is here to say so rather than to
    /// absorb it.
    scale_draw_fallbacks: AtomicU64,
}

/// One thread's private half of a [`FixedBeamIntegrand`].
///
/// Everything a point evaluation writes lives here, so the integrand itself is
/// immutable while an integration runs and can be shared across a rayon pool.
/// Nothing in it carries information from one point to the next that changes a
/// value: the amplitudes rescale from the card's own `αs` rather than from the
/// previous point's, and the coupling memo returns what a recomputation would.
/// A point's value is therefore the same whichever thread takes it.
struct FixedBeamScratch<'a> {
    /// This thread's own amplitudes ([`SubprocessProto::bind`]).
    subs: Vec<BoundSubprocess<'a>>,
    /// Reused marshalling buffer for the outgoing momenta the scale reads.
    scale_buf: RefCell<Vec<[f64; 4]>>,
    /// The last `(μR, αs(μR))` pair, so a repeated scale does not repeat the
    /// coupling's Newton solve. The prescription can be dynamic and still land on
    /// one value every event — a fixed-beam `2 → 2` clusters to `√ŝ/2` whatever the
    /// point — and that case is only visible per event, not from the run card.
    last_coupling: Cell<(f64, f64)>,
    /// Reused `AMP2` buffer for the configuration draw, one entry per integration
    /// configuration of the subprocess the channel forests were derived from.
    amp2_buf: RefCell<Vec<f64>>,
}

/// The phase-space map a [`FixedBeamIntegrand`] draws through.
///
/// Held as a closed set rather than a trait object because the two arms are
/// integrated differently: the flat map is one integral over one grid, while the
/// combiner's integral splits into one term — and one grid — per channel.
enum Sampler {
    Flat(RamboChannel<f64>),
    Multi(MultiChannel<f64>),
}

impl Sampler {
    fn ndim(&self) -> usize {
        match self {
            Sampler::Flat(c) => c.ndim(),
            Sampler::Multi(c) => c.ndim(),
        }
    }

    /// One draw, with the channel it was made in. The flat map has a single
    /// channel and so always reports `0`.
    fn sample_from(&self, u: &[f64]) -> (usize, PhaseSpacePoint<f64>) {
        match self {
            Sampler::Flat(c) => (0, c.sample(u)),
            Sampler::Multi(c) => c.sample_from(u),
        }
    }
}

/// One channel's share of a per-channel integration
/// ([`FixedBeamIntegrand::adapt_grids`]).
///
/// The integral is estimated as `Σⱼ ∫ dΦ f·αⱼgⱼ/g` — one VEGAS pass per term,
/// each over its channel's own `channel_ndim` coordinates — so every channel
/// carries its own trained grid, its own σⱼ ± Δσⱼ, and the sample budget it was
/// given. A run sampled by the flat map is the one-term case: `alpha = 1` and the
/// whole budget on a single grid.
#[derive(Debug, Clone)]
pub struct ChannelIntegration {
    /// The channel's selection weight `αⱼ` — both the weight in its term's
    /// integrand and the share of the sample budget it was allocated.
    pub alpha: f64,
    /// Evaluations per iteration this channel actually received.
    pub neval: usize,
    /// The grid trained on this channel's term.
    pub grid: VegasGrid,
    /// This term's integral and error, in natural units (GeV⁻²).
    pub result: VegasResult,
}

/// The discrete labels an accepted event carries besides its momenta, drawn by
/// [`FixedBeamIntegrand::select_event`].
///
/// Every one of them is summed over in the cross section, so none of them is a
/// sampling channel: they are read off diagonal accumulators after the fact, to
/// fill in an event record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSelection {
    /// Index into the integrand's subprocesses
    /// ([`FixedBeamIntegrand::subprocess_evaluator`]).
    pub subprocess: usize,
    /// The helicity of each external leg, in process order.
    pub helicity: Vec<i32>,
    /// Index into the subprocess's colour-flow basis, and so into its
    /// [`ColorFlowTags`](crate::helas::color::flow_tags::ColorFlowTags) table.
    pub flow: usize,
}

/// The phase-space maps the channels were built from: how many there are, how many
/// carry a peripheral spine, and — in detail — what each one draws against.
pub(crate) fn report_channel_maps(samplers: &[ChannelSampler]) {
    let spine = samplers
        .iter()
        .filter(|s| s.topology == SamplerTopology::Spine)
        .count();
    tracing::info!(
        "phase space: {} channels, {spine} peripheral, {} all-timelike",
        samplers.len(),
        samplers.len() - spine
    );
    if tracing::enabled!(tracing::Level::DEBUG) {
        for (j, sampler) in samplers.iter().enumerate() {
            tracing::debug!(
                "channel {j}: {:?}, {} resonances, {} spacelike lines",
                sampler.topology,
                sampler.resonances.len(),
                sampler.t_channels.len()
            );
        }
    }
}

/// Sum the per-channel terms: integrals add, errors add in quadrature, and the
/// χ²/dof is the pooled statistic (`Σ χ²ⱼ` over `Σ dofⱼ`) rather than any single
/// channel's.
pub(crate) fn combine_channels(per_channel: &[ChannelIntegration], niter: usize) -> VegasResult {
    let integral: f64 = per_channel.iter().map(|c| c.result.integral).sum();
    let variance: f64 = per_channel
        .iter()
        .map(|c| c.result.std_dev * c.result.std_dev)
        .sum();
    let dof_each = niter.saturating_sub(1);
    let chi2_per_dof = if dof_each > 0 && !per_channel.is_empty() {
        per_channel
            .iter()
            .map(|c| c.result.chi2_per_dof)
            .sum::<f64>()
            / per_channel.len() as f64
    } else {
        0.0
    };
    VegasResult {
        integral,
        std_dev: variance.sqrt(),
        chi2_per_dof,
    }
}

/// A channel's per-iteration evaluation count: its share `αⱼ · neval` of the
/// budget, floored at [`MIN_CHANNEL_NEVAL`] so no channel goes unsampled.
pub(crate) fn channel_neval(alpha: f64, neval: usize) -> usize {
    let share = (alpha * neval as f64).round();
    let share = if share.is_finite() && share > 0.0 {
        share as usize
    } else {
        0
    };
    share.max(MIN_CHANNEL_NEVAL)
}

impl<'a> FixedBeamIntegrand<'a> {
    /// Build the integrand from one or more bound subprocess amplitudes sharing
    /// the same external state.
    ///
    /// * `amps` — bound amplitudes whose colour+helicity-summed |M|² are added
    ///   (a single subprocess for a fully-specified initial state), each weighted
    ///   by its own identical-particle symmetry factor.
    /// * `cuts` — the compiled cut filter.
    /// * `sqrt_s` — the fixed partonic energy `E₁ + E₂`.
    /// * `final_masses` — outgoing pole masses in leg order (the RAMBO targets).
    /// * `spin_color_avg` — the initial-state average ([`initial_spin_color_average`]).
    pub fn new(
        amps: Vec<&'a BoundAmplitude<'a, f64>>,
        cuts: &'a Cuts,
        sqrt_s: f64,
        final_masses: Vec<f64>,
        spin_color_avg: f64,
    ) -> Self {
        let n = final_masses.len();
        let subs = amps.into_iter().map(SubprocessProto::fixed).collect();
        let sampler = Sampler::Flat(RamboChannel::new(sqrt_s, final_masses.clone()));
        FixedBeamIntegrand {
            subs,
            cuts,
            sqrt_s,
            sampler,
            channel_samplers: Vec::new(),
            final_masses,
            spin_color_avg,
            lips_2pi: (2.0 * PI).powi(4 - 3 * n as i32),
            beam_e: sqrt_s / 2.0,
            vegas_alpha: VEGAS_ALPHA,
            scales: None,
            scratch: ThreadLocal::new(),
            amp2_len: 0,
            amp2_alpha_s: None,
            scale_draw_fallbacks: AtomicU64::new(0),
        }
    }

    /// This thread's evaluation context, forked from the integrand's own
    /// subprocesses the first time the thread evaluates a point.
    ///
    /// Setup takes `&mut self` and so runs before any of these exist; a fork
    /// therefore starts from the fully configured amplitudes.
    fn scratch(&self) -> &FixedBeamScratch<'a> {
        self.scratch.get_or(|| FixedBeamScratch {
            subs: self.subs.iter().map(SubprocessProto::bind).collect(),
            scale_buf: RefCell::new(Vec::with_capacity(self.final_masses.len())),
            last_coupling: Cell::new((f64::NAN, f64::NAN)),
            amp2_buf: RefCell::new(vec![0.0; self.amp2_len]),
        })
    }

    /// Discard every thread's evaluation context, so the next point forks a fresh
    /// one from the integrand's own subprocesses.
    ///
    /// Setup that changes what a subprocess evaluates — installing a scale
    /// prescription, moving the coupling — must run this, or a thread that has
    /// already evaluated a point would keep the amplitudes it forked before.
    fn reset_scratch(&mut self) {
        self.scratch.clear();
    }

    /// Evaluate the matrix element at the strong coupling the run card's per-event
    /// renormalisation scale implies, instead of at the parameter card's own.
    ///
    /// Each subprocess is replaced by a scale-aware copy owning its constant pools;
    /// the channel forests the `-1` scale clusters against are derived from
    /// `diagrams` rather than declared per process.
    /// A fixed-beam run has no parton distributions, so only `μR` is consumed here —
    /// the per-beam `μF` the same prescription produces has nothing to feed.
    ///
    /// Call this **before** [`use_multichannel`](Self::use_multichannel), so the
    /// α-adaptation survey sees the integrand the integration will see.
    ///
    /// The scale is resolved once here on a sampled, cut-passing phase-space point,
    /// so a prescription this crate refuses — an unimplemented clustering above all —
    /// stops the run at setup instead of at the first VEGAS point.
    pub fn use_running_coupling(
        &mut self,
        diagrams: &[Diagram],
        model: &UFOModel,
        evaluated: &EvaluatedModel,
        card: &RunCard,
    ) -> Result<RunningCouplingReport, HadronicError> {
        let awareness = make_subs_scale_aware(&mut self.subs, evaluated);
        // With no parton distributions to read and no strong coupling in the matrix
        // element, neither scale the prescription produces has a consumer, so the
        // prescription is not compiled at all — a process whose cluster scale this
        // crate refuses still integrates, because its cross section does not depend
        // on the scale that was refused.
        if !awareness.depends_on_alpha_s {
            self.scales = None;
            return Ok(constant_scale_report(&mut self.subs, None, awareness));
        }
        // A fixed-beam run has no parton distributions, so `pdlabel` never reaches
        // the branch that would want a set's alpha_s tabulation.
        let source = compile_scale_source(
            &[(self.subs[0].evaluator(), diagrams)],
            model,
            evaluated,
            card,
            None,
            true,
        )?;
        if source.constant_scales().is_none() {
            self.probe_scale(&source)?;
        }
        let report = constant_scale_report(&mut self.subs, Some(&source), awareness);
        if source.draws_configuration() {
            self.amp2_len = self.subs[0].evaluator().n_configs();
            self.amp2_alpha_s = report.alpha_s_ref;
        }
        self.scales = Some(source);
        self.reset_scratch();
        Ok(report)
    }

    /// Resolve the scale on the cut-passing points of a fixed pseudo-random draw,
    /// so a refusal that would otherwise surface mid-integration surfaces at setup.
    ///
    /// The draw goes through the cut filter because the scale is only ever asked for
    /// on points that pass it: an unphysical configuration — a leg carrying no
    /// energy, say — has no beam measure to compare, and refusing on one would say
    /// nothing about the process.
    ///
    /// A point below the factorisation floor is not a refusal — it is one ordinary
    /// zero-weight point — so the probe steps over it and keeps drawing. Only the
    /// degenerate case is reported: cut-passing points exist and *every* one of them
    /// was vetoed, which makes the cross section zero by construction and is worth
    /// saying before the integration spends anything on it. A run whose support is
    /// partly below the floor integrates normally, as it does in MadGraph.
    fn probe_scale(&self, source: &EventScaleSource) -> Result<(), HadronicError> {
        use rand::Rng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SCALE_PROBE_SEED);
        let mut any_passed_cuts = false;
        for _ in 0..SCALE_PROBE_DRAWS {
            let u: Vec<f64> = (0..self.sampler.ndim())
                .map(|_| rng.random::<f64>())
                .collect();
            let (channel, point) = self.sampler.sample_from(&u);
            let mut ext: Vec<V> = Vec::with_capacity(2 + point.momenta.len());
            ext.push(V::new(self.beam_e, 0.0, 0.0, self.beam_e));
            ext.push(V::new(self.beam_e, 0.0, 0.0, -self.beam_e));
            ext.extend_from_slice(&point.momenta);
            if self.cuts.pass(&ext) {
                any_passed_cuts = true;
                if let PointScales::Scales(_) =
                    self.point_scales_of(self.scratch(), source, &point.momenta, channel)?
                {
                    return Ok(());
                }
            }
        }
        if any_passed_cuts {
            return Err(HadronicError::FactorisationScaleBelowFloor);
        }
        // No draw passed the cuts, which says something about the cuts and
        // nothing about the scale.
        Ok(())
    }

    /// The scales at one phase-space point, from the beams, the outgoing momenta
    /// and the sampling channel the point was drawn in.
    fn event_scales_of(
        &self,
        sc: &FixedBeamScratch<'a>,
        source: &EventScaleSource,
        momenta: &[V],
        channel: usize,
    ) -> Result<EventScales, ScaleError> {
        Ok(match self.point_scales_of(sc, source, momenta, channel)? {
            PointScales::Scales(scales) => scales,
            // The floor applies only to a beam carrying a parton density, and
            // `ScaleChoice::from_run_card` reads that off the card as
            // `lpp != 0`. A fixed-energy card gives `false` on both beams, so
            // the comparison is short-circuited and the refusal is never built.
            PointScales::Vetoed => unreachable!(
                "the factorisation floor was applied on beams carrying no parton density"
            ),
        })
    }

    /// [`event_scales_of`](Self::event_scales_of) keeping the floor refusal,
    /// which only the setup probe has a use for.
    fn point_scales_of(
        &self,
        sc: &FixedBeamScratch<'a>,
        source: &EventScaleSource,
        momenta: &[V],
        channel: usize,
    ) -> Result<PointScales, ScaleError> {
        let mut buf = sc.scale_buf.borrow_mut();
        buf.clear();
        buf.extend(momenta.iter().map(components));
        let beams = [
            [self.beam_e, 0.0, 0.0, self.beam_e],
            [self.beam_e, 0.0, 0.0, -self.beam_e],
        ];
        source.point_scales(beams, &buf, SampledChannel::sole(channel))
    }

    /// The scales this integrand evaluates a point at, when a prescription was
    /// installed by [`use_running_coupling`](Self::use_running_coupling).
    ///
    /// An event record has to report the scale its matrix element actually ran
    /// at, so it reads it from here rather than compiling a second prescription
    /// off the same run card and hoping the two agree. `None` when nothing in the
    /// matrix element moves with the strong coupling and so no prescription was
    /// installed at all — a record then takes its factorisation scale from the run
    /// card directly, no cross section having depended on it.
    /// `channel` is the sampling channel the point came from, which the
    /// clustering prescription reads: an event record's scale is the one its own
    /// draw implied, not the one some other channel would have given.
    pub fn event_scales(
        &self,
        momenta: &[V],
        channel: usize,
    ) -> Option<Result<EventScales, ScaleError>> {
        let source = self.scales.as_ref()?;
        Some(self.event_scales_of(self.scratch(), source, momenta, channel))
    }

    /// The source an event record's `AQCDUP` is evaluated from, when one was built.
    pub fn alpha_s_source(&self) -> Option<&AlphaSSource> {
        self.scales.as_ref()?.alpha_s()
    }

    /// The compiled per-event prescription, once
    /// [`use_running_coupling`](Self::use_running_coupling) installed one.
    pub fn scale_source(&self) -> Option<&EventScaleSource> {
        self.scales.as_ref()
    }

    /// Move every subprocess to the coupling this point's renormalisation scale
    /// implies. A constant prescription was applied once at installation, and a
    /// matrix element with no strong coupling in it has no coupling to move, so
    /// both return here without touching the momenta.
    fn apply_scale(&self, sc: &FixedBeamScratch<'a>, momenta: &[V], channel: usize) {
        let Some(source) = &self.scales else { return };
        if source.constant_scales().is_some() {
            return;
        }
        let Some(running) = source.alpha_s() else {
            return;
        };
        let scales = self
            .event_scales_of(sc, source, momenta, channel)
            .unwrap_or_else(|e| panic!("per-event scale on a sampled point: {e}"));
        let (last_mu_r, last_alpha_s) = sc.last_coupling.get();
        let alpha_s = if scales.mu_r == last_mu_r {
            last_alpha_s
        } else {
            let alpha_s = running.eval(scales.mu_r);
            sc.last_coupling.set((scales.mu_r, alpha_s));
            alpha_s
        };
        for sub in &sc.subs {
            sub.set_alpha_s(alpha_s);
        }
    }

    /// The uniforms the active phase-space map consumes as one mixture: `4n` for
    /// flat RAMBO, and `3n − 3` for the multichannel combiner (one
    /// channel-selection coordinate plus the `3n − 4` invariant/angle
    /// coordinates).
    ///
    /// This is the dimensionality of the single grid
    /// [`adapt_grid`](Self::adapt_grid) builds. The per-channel grids
    /// [`adapt_grids`](Self::adapt_grids) builds are over
    /// [`channel_grid_ndim`](Self::channel_grid_ndim) instead.
    pub fn vegas_ndim(&self) -> usize {
        self.sampler.ndim() + self.scale_draw_ndim()
    }

    /// The colour+helicity-summed `Σ_sub S_sub |M_sub|²` at the outgoing momenta
    /// `momenta`, in the partonic-CM frame, with the beams prepended and the cut
    /// filter applied — the matrix-element part of the integrand as a function of the
    /// phase-space point. A configuration failing a cut returns exactly `0.0`, so it
    /// drops out of both the cross section and the α-adaptation survey.
    ///
    /// Each subprocess enters weighted by its own identical-particle factor, so a
    /// summed matrix element whose terms have different outgoing multisets is right
    /// term by term. The survey sees the same weighting the integral does.
    fn matrix_element(&self, sc: &FixedBeamScratch<'a>, momenta: &[V], channel: usize) -> f64 {
        let ext = self.externals(momenta);
        if !self.cuts.pass(&ext) {
            return 0.0;
        }
        self.matrix_element_at(sc, &ext, momenta, channel)
    }

    /// [`matrix_element`](Self::matrix_element) for a point whose externals
    /// already passed the cut filter, so it neither rebuilds `ext` nor re-checks
    /// it — the seam that lets a caller share the cut verdict with a
    /// configuration draw taken on the same point.
    fn matrix_element_at(
        &self,
        sc: &FixedBeamScratch<'a>,
        ext: &[V],
        momenta: &[V],
        channel: usize,
    ) -> f64 {
        self.apply_scale(sc, momenta, channel);

        let mut m2 = 0.0;
        for sub in &sc.subs {
            m2 += sub.symmetry_factor() * sub.eval_m2(ext);
        }
        m2
    }

    /// The integrand value at a VEGAS point `u ∈ [0,1]^ndim`, in natural units
    /// (GeV⁻²); its VEGAS integral is the partonic cross section. Points whose
    /// momenta fail a cut contribute exactly zero and return before the
    /// configuration draw runs, so a rejected point never pays for it.
    pub fn value(&self, u: &[f64]) -> f64 {
        let map_ndim = self.sampler.ndim();
        assert_eq!(
            u.len(),
            map_ndim + self.scale_draw_ndim(),
            "a mixture point is {map_ndim} map coordinates and {} scale-draw uniforms",
            self.scale_draw_ndim()
        );
        let (map_u, scale_u) = u.split_at(map_ndim);
        let (channel, point) = self.sampler.sample_from(map_u);
        let ext = self.externals(&point.momenta);
        if !self.cuts.pass(&ext) {
            return 0.0;
        }
        let sc = self.scratch();
        let scale_channel = self.scale_channel(sc, &ext, channel, scale_u);
        let m2 = self.matrix_element_at(sc, &ext, &point.momenta, scale_channel);
        if m2 == 0.0 {
            return 0.0;
        }
        self.prefactor() * point.weight * m2
    }

    /// The constants in front of `weight · Σ_sub S_sub |M_sub|²`: the `1/(2ŝ)` flux,
    /// the initial-state spin×colour average, and the `(2π)^{4−3n}` measure factor.
    /// The identical-particle factors are not among them — they are per subprocess,
    /// applied inside the sum.
    fn prefactor(&self) -> f64 {
        let flux = 1.0 / (2.0 * self.sqrt_s * self.sqrt_s);
        flux * self.spin_color_avg * self.lips_2pi
    }

    /// Replace flat RAMBO with a resonance-aware per-diagram [`MultiChannel`] built
    /// from `diagrams` (one [`DiagramChannel`] each, its propagator poles read from
    /// `model`), then α-adapt the channel mixture to *this* integrand and install
    /// the adapted combiner as the sampler.
    ///
    /// The α-adaptation surveys the combiner under the process's own `Σ|M|²` (the
    /// [`matrix_element`](Self::matrix_element) shape, cut included), so weight
    /// flows to the channels that carry the integrand's variance. Constant
    /// prefactors (flux, spin/colour average, the `2π` measure) are omitted from
    /// the survey integrand: they scale every channel's variance share equally and
    /// so leave the Kleiss–Pittau reallocation unchanged, while keeping the survey
    /// cheaper. The combiner shares RAMBO's `R_n` weight normalisation, so the
    /// master formula and every prefactor are untouched — only the sampling density
    /// changes, and the estimator stays unbiased for the same `σ̂`.
    ///
    /// The peripheral channels are regulated at [`Cuts::spacelike_floor`], the scale
    /// this integrand's own cuts imply — the same input
    /// [`ProtonIntegrand`](crate::proton::ProtonIntegrand) passes, read off the cuts
    /// rather than taken from the caller so a run cannot regulate at a scale its
    /// cuts do not have. Without one, a final state of more than two legs has no
    /// peripheral spine at all, and a spacelike line's transfer is drawn flat.
    ///
    /// Returns the α refinement path, or `None` if `diagrams` is empty (the flat
    /// sampler is then left in place).
    pub fn use_multichannel(
        &mut self,
        diagrams: &[Diagram],
        model: &EvaluatedModel,
        n_survey: usize,
        n_iter: usize,
        seed: u64,
    ) -> Option<AlphaAdaptation<f64>> {
        let floor = self.cuts.spacelike_floor();
        let built: Vec<DiagramChannel<f64>> = diagrams
            .iter()
            .map(|d| DiagramChannel::from_diagram_regulated(d, model, self.sqrt_s, floor))
            .collect();
        if built.is_empty() {
            return None;
        }
        self.assert_channels_match_the_scale_source(built.len());
        let samplers: Vec<ChannelSampler> = built.iter().map(ChannelSampler::of).collect();
        report_channel_maps(&samplers);
        let channels: Vec<Box<dyn Channel<f64>>> = built
            .into_iter()
            .map(|c| Box::new(c) as Box<dyn Channel<f64>>)
            .collect();
        let mut combiner = MultiChannel::uniform(channels);
        // The survey has to see the integrand the integration will, so it draws
        // the scale configuration too — off a stream of its own, leaving the
        // survey's own point sequence untouched.
        let scale_draw = RefCell::new(SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE));
        let scale_ndim = self.scale_draw_ndim();
        let report = combiner.adapt_alphas(
            |momenta, channel| {
                let scale_u: Vec<f64> = (0..scale_ndim)
                    .map(|_| scale_draw.borrow_mut().next_uniform::<f64>())
                    .collect();
                let sc = self.scratch();
                let ext = self.externals(momenta);
                self.matrix_element(sc, momenta, self.scale_channel(sc, &ext, channel, &scale_u))
            },
            seed,
            MULTICHANNEL_ADAPT_STREAM,
            n_survey,
            n_iter,
            0.5,
        );
        self.sampler = Sampler::Multi(combiner);
        self.channel_samplers = samplers;
        self.vegas_alpha = VEGAS_ALPHA_MAPPED;
        Some(report)
    }

    /// Install the same per-diagram [`MultiChannel`] with selection weights taken
    /// from a completed integration instead of re-surveyed.
    ///
    /// A sampling phase that replays trained grids has to reproduce the *exact*
    /// integrand those grids were trained on, and `αⱼ` enters a channel's weight —
    /// so re-running the α-adaptation would reproduce it only by accident, and
    /// would silently stop doing so the moment the survey budget changed. Reading
    /// the converged weights back is exact by construction and costs no survey.
    ///
    /// `Err` carries the channel count actually built when it disagrees with
    /// `alphas` — which is what a proc card describing a different process looks
    /// like from here. `None` if `diagrams` is empty, leaving the flat sampler in
    /// place.
    ///
    /// # Panics
    ///
    /// If `alphas` are not a normalised set of positive selection weights.
    pub fn use_multichannel_with_alphas(
        &mut self,
        diagrams: &[Diagram],
        model: &EvaluatedModel,
        alphas: &[f64],
    ) -> Option<Result<(), usize>> {
        let floor = self.cuts.spacelike_floor();
        let built: Vec<DiagramChannel<f64>> = diagrams
            .iter()
            .map(|d| DiagramChannel::from_diagram_regulated(d, model, self.sqrt_s, floor))
            .collect();
        if built.is_empty() {
            return None;
        }
        self.assert_channels_match_the_scale_source(built.len());
        let samplers: Vec<ChannelSampler> = built.iter().map(ChannelSampler::of).collect();
        let channels: Vec<Box<dyn Channel<f64>>> = built
            .into_iter()
            .map(|c| Box::new(c) as Box<dyn Channel<f64>>)
            .collect();
        if channels.len() != alphas.len() {
            return Some(Err(channels.len()));
        }
        report_channel_maps(&samplers);
        let mut combiner = MultiChannel::uniform(channels);
        combiner.set_alphas(alphas.to_vec());
        self.sampler = Sampler::Multi(combiner);
        self.channel_samplers = samplers;
        self.vegas_alpha = VEGAS_ALPHA_MAPPED;
        Some(Ok(()))
    }

    /// A sampling channel names an integration channel by its diagram index, so
    /// the two sides have to be built from one diagram slice. A caller that
    /// installed a clustering prescription from different diagrams than it
    /// sampled would silently cluster every point in the wrong channel; the
    /// count is what says so.
    fn assert_channels_match_the_scale_source(&self, built: usize) {
        let Some(sets) = self.scales.as_ref().and_then(EventScaleSource::channels) else {
            return;
        };
        assert_eq!(
            sets.len(),
            1,
            "a fixed-beam run draws every channel from one subprocess"
        );
        assert_eq!(
            built,
            sets[0].diagram_count(),
            "the multichannel map and the cluster scale were built from different diagrams, \
             so a sampled channel would name the wrong integration channel"
        );
    }

    /// The channels the integral is split across: one per diagram once a
    /// multichannel combiner is installed, and `1` under the flat map.
    pub fn channel_count(&self) -> usize {
        match &self.sampler {
            Sampler::Flat(_) => 1,
            Sampler::Multi(c) => c.channels().len(),
        }
    }

    /// What the rule-based composition chose for each channel, in channel order.
    /// Empty under flat RAMBO, whose single channel is not composed from a
    /// diagram's propagator structure.
    pub fn channel_samplers(&self) -> &[ChannelSampler] {
        &self.channel_samplers
    }

    /// The converged channel selection weights, or `[1.0]` under the flat map.
    pub fn channel_alphas(&self) -> Vec<f64> {
        match &self.sampler {
            Sampler::Flat(_) => vec![1.0],
            Sampler::Multi(c) => c.alphas().to_vec(),
        }
    }

    /// The uniforms one channel's grid is built over: `channel_ndim` for the
    /// combiner (no channel-selection coordinate — the channel is frozen), and the
    /// full map dimension under flat RAMBO.
    pub fn channel_grid_ndim(&self) -> usize {
        match &self.sampler {
            Sampler::Flat(c) => c.ndim(),
            Sampler::Multi(c) => c.channel_ndim(),
        }
    }

    /// The uniforms a point carries beyond its channel's grid coordinates: one for
    /// the scale prescription's configuration draw where that draw is live, and
    /// none otherwise.
    ///
    /// It is not a grid coordinate. The grids stay over
    /// [`channel_grid_ndim`](Self::channel_grid_ndim) exactly, so installing the
    /// draw leaves every trained grid, every channel weight and every point
    /// sequence what it was.
    pub fn scale_draw_ndim(&self) -> usize {
        self.scales
            .as_ref()
            .map_or(0, |s| usize::from(s.draws_configuration()))
    }

    /// The full length of the coordinate slice a point is evaluated at:
    /// its channel's grid coordinates followed by
    /// [`scale_draw_ndim`](Self::scale_draw_ndim) trailing uniforms.
    pub fn point_ndim(&self) -> usize {
        self.channel_grid_ndim() + self.scale_draw_ndim()
    }

    /// The `j`-th term of the channel-split estimator at `u ∈ [0,1]^point_ndim`,
    /// in natural units (GeV⁻²): the point is drawn from channel `j` alone and
    /// weighted by `αⱼ/g`, so the sum over channels of these terms' integrals is
    /// the same cross section [`value`](Self::value) integrates from the mixture.
    ///
    /// Under the flat map there is one channel and this is [`value`](Self::value).
    ///
    /// # Panics
    ///
    /// If `channel` is not a channel index ([`channel_count`](Self::channel_count)),
    /// or if `u` is not [`point_ndim`](Self::point_ndim) long. The length is
    /// asserted rather than tolerated: a driver that handed over only the grid's
    /// coordinates would silently drop the scale draw and evaluate every point in
    /// the sampler's own channel, which is a difference no cross section announces.
    pub fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        let (grid_u, scale_u) = self.split_point(u);
        let point = self.sample_channel(channel, grid_u);
        let ext = self.externals(&point.momenta);
        if !self.cuts.pass(&ext) {
            return 0.0;
        }
        let sc = self.scratch();
        let scale_channel = self.scale_channel(sc, &ext, channel, scale_u);
        let m2 = self.matrix_element_at(sc, &ext, &point.momenta, scale_channel);
        if m2 == 0.0 {
            return 0.0;
        }
        self.prefactor() * point.weight * m2
    }

    /// Which of the process's channels names the integration configuration this
    /// point's scale is clustered in.
    ///
    /// Without the draw it is the sampling channel the point came from. With it,
    /// the configuration is drawn `∝ AMP2_c(p)` — MadEvent's single-diagram
    /// enhancement weight under the card condition
    /// [`EventScaleSource::draws_configuration`] tests — and named back through
    /// *its own diagram*, because the evaluator's configuration order and the
    /// channel forests' are two independent derivations from one diagram slice
    /// and the diagram is their common ground.
    ///
    /// `AMP2` is formed at the coupling the amplitudes were bound at rather than
    /// at the one the previous point left, which is what makes the drawn
    /// configuration a function of the momenta alone: the scale is not known until
    /// after the draw, so evaluating "at the event's coupling" would be circular.
    fn scale_channel(
        &self,
        sc: &FixedBeamScratch<'a>,
        ext: &[V],
        channel: usize,
        scale_u: &[f64],
    ) -> usize {
        let [v] = scale_u else { return channel };
        let sub = &sc.subs[0];
        if let Some(alpha_s) = self.amp2_alpha_s {
            sub.set_alpha_s(alpha_s);
        }
        let mut amp2 = sc.amp2_buf.borrow_mut();
        sub.eval_amp2(ext, &mut amp2);
        match select_index(&amp2, *v) {
            Some(c) => sub.evaluator().config_diagrams()[c],
            // Every diagram amplitude vanished here, so the coherent sum does too
            // and this point carries no weight whichever channel names its scale.
            None => {
                self.scale_draw_fallbacks.fetch_add(1, Ordering::Relaxed);
                channel
            }
        }
    }

    /// Points on which the configuration draw found no probability and kept the
    /// sampling channel. Expected to be zero on a run that produces anything.
    pub fn scale_draw_fallbacks(&self) -> u64 {
        self.scale_draw_fallbacks.load(Ordering::Relaxed)
    }

    /// [`value_in_channel`](Self::value_in_channel) with the outgoing momenta kept:
    /// `momenta` is overwritten with the point the value was taken at, in
    /// outgoing-leg order and in the partonic-CM frame.
    ///
    /// An accept/reject pass needs the momenta only for the points it keeps, so the
    /// trial loop runs on `value_in_channel` and reconstructs an accepted point
    /// through this — the same map at the same `u`, hence the same weight.
    pub fn event_in_channel(&self, channel: usize, u: &[f64], momenta: &mut Vec<V>) -> f64 {
        let (grid_u, scale_u) = self.split_point(u);
        let point = self.sample_channel(channel, grid_u);
        momenta.clear();
        momenta.extend_from_slice(&point.momenta);
        let ext = self.externals(momenta);
        if !self.cuts.pass(&ext) {
            return 0.0;
        }
        let sc = self.scratch();
        let scale_channel = self.scale_channel(sc, &ext, channel, scale_u);
        let m2 = self.matrix_element_at(sc, &ext, momenta, scale_channel);
        if m2 == 0.0 {
            return 0.0;
        }
        self.prefactor() * point.weight * m2
    }

    /// The scales a point drawn at `u` in `channel` was evaluated at — the scales
    /// an event record must report, since they are the ones its own weight was
    /// computed with.
    ///
    /// Takes the whole coordinate slice rather than the channel alone because
    /// with the configuration draw live the scale is a function of the trailing
    /// uniform too. A caller replaying an accepted point hands over that point's
    /// own `u`, which is what makes the recorded scale the one the accept/reject
    /// weight was taken at.
    pub fn event_scales_at(
        &self,
        momenta: &[V],
        channel: usize,
        u: &[f64],
    ) -> Option<Result<EventScales, ScaleError>> {
        let (_, scale_u) = self.split_point(u);
        let ext = self.externals(momenta);
        self.event_scales(
            momenta,
            self.scale_channel(self.scratch(), &ext, channel, scale_u),
        )
    }

    /// Split a point's coordinates into the ones its channel's map consumes and
    /// the trailing ones the scale prescription does.
    fn split_point<'u>(&self, u: &'u [f64]) -> (&'u [f64], &'u [f64]) {
        let grid_ndim = self.channel_grid_ndim();
        assert_eq!(
            u.len(),
            grid_ndim + self.scale_draw_ndim(),
            "a point is {} map coordinates and {} scale-draw uniforms",
            grid_ndim,
            self.scale_draw_ndim()
        );
        u.split_at(grid_ndim)
    }

    /// Draw the phase-space point channel `channel` maps `u` to, with its weight.
    fn sample_channel(&self, channel: usize, u: &[f64]) -> PhaseSpacePoint<f64> {
        match &self.sampler {
            Sampler::Flat(c) => {
                assert_eq!(channel, 0, "the flat map has a single channel");
                c.sample(u)
            }
            Sampler::Multi(c) => c.sample_channel(channel, u),
        }
    }

    /// The two incoming momenta: `√ŝ/2` along ±z, the beam configuration every
    /// evaluation in this integrand is made in.
    pub fn beams(&self) -> [V; 2] {
        [
            V::new(self.beam_e, 0.0, 0.0, self.beam_e),
            V::new(self.beam_e, 0.0, 0.0, -self.beam_e),
        ]
    }

    /// The external momenta an amplitude is evaluated at: the beams, then the
    /// outgoing legs.
    fn externals(&self, momenta: &[V]) -> Vec<V> {
        let mut ext = Vec::with_capacity(2 + momenta.len());
        ext.extend_from_slice(&self.beams());
        ext.extend_from_slice(momenta);
        ext
    }

    /// The subprocesses whose `|M|²` this integrand adds.
    pub fn subprocess_count(&self) -> usize {
        self.subs.len()
    }

    /// The compiled evaluator of one subprocess — the source of the external
    /// particle ids, the helicity combinations and the colour-flow tag table an
    /// event record is written from.
    pub fn subprocess_evaluator(&self, subprocess: usize) -> &'a AmplitudeEvaluator {
        self.subs[subprocess].evaluator()
    }

    /// Fill in an accepted event's discrete labels: which subprocess produced it,
    /// which helicity combination, and which colour flow.
    ///
    /// `momenta` are the accepted point's outgoing momenta and `u` four independent
    /// uniforms. The subprocess is drawn `∝ |M_s|²` (the incoherent sum this
    /// integrand forms), then within it the helicity `∝ |M_c|²` (MadGraph's
    /// `SELECT_HEL`), and finally the colour flow through
    /// [`AmplitudeEvaluator::select_color_flow`] — the integration configuration
    /// `∝ AMP2(d)` from `u[2]` and the flow `∝ JAMP2(i)` within that
    /// configuration's admitted set from `u[3]` (`SELECT_COLOR`).
    ///
    /// All of them are *selections*, not sampling channels: the cross section sums
    /// over subprocesses, helicities, configurations and flows, and this reads
    /// accumulators that decomposition already contains. It enters no integrand and
    /// moves no cross section — a caller may skip it entirely and integrate the same
    /// number.
    ///
    /// `None` when the point carries no weight at all (outside the cuts, or a
    /// vanishing matrix element), where no label is defined.
    pub fn select_event(
        &self,
        momenta: &[V],
        channel: usize,
        u: [f64; 4],
    ) -> Option<EventSelection> {
        let ext = self.externals(momenta);
        let sc = self.scratch();
        // The diagonals are read at the event's own coupling, the one its |M|² was
        // taken at — which is the point's own sampling channel's, since the
        // cluster scale reads it.
        self.apply_scale(sc, momenta, channel);

        let m2: Vec<f64> = sc.subs.iter().map(|s| s.eval_m2(&ext)).collect();
        let subprocess = select_index(&m2, u[0])?;
        let sub = &sc.subs[subprocess];
        let eval = sub.evaluator();

        let mut hel_m2 = vec![0.0; eval.helicities().len()];
        let mut amp2 = vec![0.0; eval.n_configs()];
        let mut jamp2 = vec![0.0; eval.n_flows()];
        sub.eval_diagonals(&ext, &mut hel_m2, &mut amp2, &mut jamp2);

        Some(EventSelection {
            subprocess,
            helicity: eval.select_helicity(&hel_m2, u[1])?.to_vec(),
            flow: eval.select_color_flow(&amp2, &jamp2, [u[2], u[3]])?,
        })
    }

    /// The final-state pole masses in outgoing-leg order.
    pub fn final_masses(&self) -> &[f64] {
        &self.final_masses
    }

    /// Integrate the cross section with VEGAS, returning `(σ, Δσ)` in picobarns.
    pub fn integrate(&self, neval: usize, niter: usize, seed: u64) -> (f64, f64) {
        let result = self.adapt_grids(neval, niter, seed).1;
        (result.integral * GEV2_TO_PB, result.std_dev * GEV2_TO_PB)
    }

    /// Run VEGAS adaptation over the mixture map as a single integral, returning
    /// the one trained grid alongside the result.
    ///
    /// The combiner's channel selection then occupies a coordinate of that grid.
    /// [`adapt_grids`](Self::adapt_grids) is what the cross section is taken
    /// from; this is the undivided comparison point.
    pub fn adapt_grid(&self, neval: usize, niter: usize, seed: u64) -> (VegasGrid, VegasResult) {
        let mut grid = VegasGrid::new(self.vegas_ndim(), VEGAS_NBINS, self.vegas_alpha);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let result = grid.adapt(|u| self.value(u), neval, niter, &mut rng);
        (grid, result)
    }

    /// Run one VEGAS adaptation **per channel**, returning each channel's trained
    /// grid and term alongside their sum — the primitive the `integrate` CLI
    /// command serializes into its artifact.
    ///
    /// Channel `j` is integrated over its own `channel_ndim` coordinates with the
    /// channel frozen ([`value_in_channel`](Self::value_in_channel)), on a sample
    /// budget of `αⱼ · neval` per iteration and its own RNG substream. The terms sum
    /// to the same cross section the mixture integrates, but each grid now refines a
    /// density *conditional* on its channel — the correlation a single separable
    /// grid over the mixture cannot represent, since the useful shape of the
    /// remaining coordinates depends on which channel was selected.
    ///
    /// Under the flat map this is a single full-budget pass, identical to
    /// [`adapt_grid`](Self::adapt_grid).
    pub fn adapt_grids(
        &self,
        neval: usize,
        niter: usize,
        seed: u64,
    ) -> (Vec<ChannelIntegration>, VegasResult) {
        self.adapt_grids_with(neval, niter, seed, self.vegas_alpha)
    }

    /// [`adapt_grids`](Self::adapt_grids) with the VEGAS grid-damping exponent
    /// supplied instead of taken from the active sampler — the seam a study of the
    /// refinement's own stability drives.
    pub fn adapt_grids_with(
        &self,
        neval: usize,
        niter: usize,
        seed: u64,
        vegas_alpha: f64,
    ) -> (Vec<ChannelIntegration>, VegasResult) {
        let (per_channel, total, _) = integrate_channels(
            self,
            &self.channel_alphas(),
            vegas_alpha,
            IterationCombination::default(),
            Budget::Fixed { neval, niter },
            BlockAllocation::ByAlpha,
            seed,
            &StopSignal::default(),
        );
        (per_channel, total)
    }

    /// Integrate under an arbitrary [`Budget`] and channel-allocation rule,
    /// reporting what was spent alongside the terms.
    ///
    /// [`adapt_grids`](Self::adapt_grids) is the [`Budget::Fixed`],
    /// [`BlockAllocation::ByAlpha`] case of this, under a signal nobody holds. A
    /// raised `stop` ends the run at the next iteration boundary, with the grids
    /// and terms of the iterations it did complete.
    pub fn adapt_grids_budget(
        &self,
        budget: Budget,
        allocation: BlockAllocation,
        seed: u64,
        stop: &StopSignal,
    ) -> (Vec<ChannelIntegration>, VegasResult, ConvergenceReport) {
        integrate_channels(
            self,
            &self.channel_alphas(),
            self.vegas_alpha,
            IterationCombination::default(),
            budget,
            allocation,
            seed,
            stop,
        )
    }
}

/// The seam an accept/reject pass drives this integrand through: the channels its
/// integral is split across, and one term's value at a point of that channel's own
/// grid.
impl ChannelIntegrand for FixedBeamIntegrand<'_> {
    fn channel_count(&self) -> usize {
        FixedBeamIntegrand::channel_count(self)
    }

    fn channel_grid_ndim(&self) -> usize {
        FixedBeamIntegrand::channel_grid_ndim(self)
    }

    fn scale_draw_ndim(&self) -> usize {
        FixedBeamIntegrand::scale_draw_ndim(self)
    }

    fn value_in_channel(&self, channel: usize, u: &[f64]) -> f64 {
        FixedBeamIntegrand::value_in_channel(self, channel, u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::runcard::RunCard;
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn model() -> std::sync::Arc<UFOModel> {
        sm_model(SMRestrict::Default)
    }

    /// Partonic-CM external momenta `[q, q̄, e⁺, e⁻]` of a back-to-back dilepton
    /// configuration, beams along ±z. `cos_theta` is the CM polar angle of `e⁺`
    /// and the azimuth is fixed, which the total cross section is symmetric in.
    fn dilepton_cm(sqrt_shat: f64, cos_theta: f64) -> Vec<V> {
        let half = sqrt_shat / 2.0;
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        vec![
            V::new(half, 0.0, 0.0, half),
            V::new(half, 0.0, 0.0, -half),
            V::new(half, half * sin_theta, 0.0, half * cos_theta),
            V::new(half, -half * sin_theta, 0.0, -half * cos_theta),
        ]
    }

    #[test]
    fn class_matrix_element_is_flavor_independent_within_class() {
        // The "one σ̂ per class" premise: c c̄ must give the same |M|² as u ū at
        // identical partonic-CM kinematics (massless quarks, same couplings). A
        // convention/coupling regression that made them differ would fail here.
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let build = |proc: &str| {
            let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
            let sets = generate_from_proc_card(&card, &m).unwrap();
            let set = sets.into_iter().find(|s| !s.diagrams.is_empty()).unwrap();
            compile_class(&set, &m, &evaluated).unwrap()
        };
        let uu = build("u u~ > e+ e-");
        let cc = build("c c~ > e+ e-");
        let b_uu = BoundAmplitude::<f64>::bind(&uu, &evaluated);
        let b_cc = BoundAmplitude::<f64>::bind(&cc, &evaluated);
        let mut s_uu = b_uu.scratch_space();
        let mut s_cc = b_cc.scratch_space();

        let sqrt_shat = 200.0;
        for &cos in &[-0.7, -0.2, 0.3, 0.85] {
            let cm = dilepton_cm(sqrt_shat, cos);
            let a = b_uu.eval_m2(&cm, &mut s_uu);
            let b = b_cc.eval_m2(&cm, &mut s_cc);
            let rel = (a - b).abs() / a.abs().max(1e-30);
            assert!(
                rel < 1e-12,
                "u ū vs c c̄ |M|² differ: {a} vs {b} (rel {rel:.2e})"
            );
        }
    }

    fn build_evaluator(proc: &str, m: &UFOModel, evaluated: &EvaluatedModel) -> AmplitudeEvaluator {
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
        let sets = generate_from_proc_card(&card, m).unwrap();
        let set = sets.into_iter().find(|s| !s.diagrams.is_empty()).unwrap();
        compile_class(&set, m, evaluated).unwrap()
    }

    #[test]
    fn spin_color_average_is_process_derived() {
        // The averaging denominator must fall out of the incoming legs' spin code
        // and colour dimension — not a hand-coded constant. These pin that
        // hypothesis: a miscount of spin states or colour dimension fails here.
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());

        // q q̄: 2 spin × 3 colour, twice.
        let uu = build_evaluator("u u~ > e+ e-", &m, &evaluated);
        assert_eq!(initial_spin_color_average(&uu, &m, &evaluated), 1.0 / 36.0);
        // e⁺e⁻: 2 spin, colour singlet, twice.
        let ee = build_evaluator("e+ e- > mu+ mu-", &m, &evaluated);
        assert_eq!(initial_spin_color_average(&ee, &m, &evaluated), 1.0 / 4.0);
        // gg: massless vector (2 spin) × adjoint colour (8), twice.
        let gg = build_evaluator("g g > t t~", &m, &evaluated);
        assert_eq!(initial_spin_color_average(&gg, &m, &evaluated), 1.0 / 256.0);
    }

    /// The identical-particle factor must fall out of a compiled process's own
    /// outgoing legs, not out of a table: `g g → g g` is the only MG-validated
    /// process with a repeated outgoing particle, and it is exactly the row whose
    /// cross section comes out twice MadGraph's without the factor.
    ///
    /// `u ū → g g` and `u ū → d d̄` are the pair that says the outgoing *mass* list
    /// cannot own the factor: both are `[0, 0]` and they need `1/2` and `1`.
    #[test]
    fn subprocess_symmetry_factor_counts_its_own_outgoing_legs() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let factor = |proc: &str| {
            let eval = build_evaluator(proc, &m, &evaluated);
            identical_particle_factor(&eval.external_particles()[eval.n_in()..])
        };
        assert_eq!(factor("g g > g g"), 0.5);
        assert_eq!(factor("g g > t t~"), 1.0);
        assert_eq!(factor("u u~ > u u~"), 1.0);
        assert_eq!(factor("e+ e- > mu+ mu-"), 1.0);
        assert_eq!(factor("u u~ > g g"), 0.5);
        assert_eq!(factor("u u~ > d d~"), 1.0);
    }

    /// A summed matrix element weights each subprocess by *its own* factor.
    ///
    /// `u ū → g g` (`1/2`) and `u ū → d d̄` (`1`) share a phase-space map — same
    /// outgoing masses, same cut filter — and differ in the factor, so an integrand
    /// that derived one factor from its first subprocess and applied it to all of
    /// them would halve the `d d̄` term. The reference is the two subprocesses
    /// integrated separately over the same map and seed, whose sum the combined
    /// integrand must reproduce to the identity of the sampling, not to Monte Carlo
    /// error: both sides draw the same points.
    #[test]
    fn a_summed_matrix_element_weights_each_subprocess_by_its_own_factor() {
        use crate::phasespace::rng::SubStream;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let gg = build_evaluator("u u~ > g g", &m, &evaluated);
        let ddx = build_evaluator("u u~ > d d~", &m, &evaluated);
        let b_gg = BoundAmplitude::<f64>::bind(&gg, &evaluated);
        let b_ddx = BoundAmplitude::<f64>::bind(&ddx, &evaluated);

        let legs = process_external_legs(&gg, &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let avg = initial_spin_color_average(&gg, &m, &evaluated);
        let masses = vec![0.0, 0.0];
        let sqrt_s = 400.0;
        let both = FixedBeamIntegrand::new(vec![&b_gg, &b_ddx], &cuts, sqrt_s, masses.clone(), avg);
        let only_gg = FixedBeamIntegrand::new(vec![&b_gg], &cuts, sqrt_s, masses.clone(), avg);
        let only_ddx = FixedBeamIntegrand::new(vec![&b_ddx], &cuts, sqrt_s, masses, avg);

        let mut stream = SubStream::from_stream(0x5111_5EED, 3);
        let mut nonzero = 0;
        for _ in 0..64 {
            let u = stream.uniforms::<f64>(both.vegas_ndim());
            let sum = only_gg.value(&u) + only_ddx.value(&u);
            let combined = both.value(&u);
            if combined == 0.0 {
                continue;
            }
            nonzero += 1;
            let rel = (combined - sum).abs() / sum.abs();
            assert!(
                rel < 1e-14,
                "summed integrand {combined:.17e} vs term sum {sum:.17e} (rel {rel:.2e})"
            );
            // The two terms must actually be comparable in size, or the `d d̄` term
            // could be halved without moving the sum enough to see.
            let share = only_ddx.value(&u) / combined;
            assert!(
                share > 1e-3,
                "the d d̄ term is {share:.2e} of the sum; a lost factor would hide"
            );
        }
        assert!(nonzero > 32, "only {nonzero} points passed the cuts");
    }

    /// Permutations of identical outgoing legs are not enumerated as extra sampling
    /// channels, on the claim that the per-diagram set is already closed under them:
    /// the image of a diagram under a swap of two identical outgoing legs is another
    /// diagram of the same process. That claim is testable at the level of the
    /// mixture it licenses — under uniform selection weights the combined density of
    /// `g g → g g`'s channels must be invariant under exchanging the two outgoing
    /// momenta, since a channel whose image were missing would peak on one
    /// assignment only.
    ///
    /// The control is the second half: dropping a single channel has to break the
    /// symmetry, or the invariance is a property of the density formula rather than
    /// of the set and the first half sees nothing. The channels are built at the
    /// spacelike floor a hadronic run gives them, which is what makes the peripheral
    /// ones peak on one leg assignment each; at floor zero they collapse to a common
    /// all-timelike map whose density is symmetric one channel at a time, and the
    /// control refuses that configuration.
    #[test]
    fn the_channel_set_of_identical_outgoing_legs_is_permutation_closed() {
        use crate::phasespace::rng::SubStream;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let diagrams: Vec<_> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let sqrt_s = 500.0;
        let floor = 400.0;
        let build = |skip: Option<usize>| {
            let channels: Vec<Box<dyn Channel<f64>>> = diagrams
                .iter()
                .enumerate()
                .filter(|(i, _)| Some(*i) != skip)
                .map(|(_, d)| {
                    Box::new(DiagramChannel::from_diagram_regulated(
                        d, &evaluated, sqrt_s, floor,
                    )) as Box<dyn Channel<f64>>
                })
                .collect();
            MultiChannel::uniform(channels)
        };

        let full = build(None);
        let mut stream = SubStream::from_stream(0xC105_ED, 1);
        let points: Vec<Vec<V>> = (0..32)
            .map(|_| full.sample(&stream.uniforms::<f64>(full.ndim())).momenta)
            .collect();
        let asymmetry = |mc: &MultiChannel<f64>, p: &[V]| {
            let swapped = vec![p[1], p[0]];
            let (a, b) = (mc.density(p), mc.density(&swapped));
            (a - b).abs() / a.abs().max(b.abs())
        };

        for p in &points {
            let rel = asymmetry(&full, p);
            assert!(
                rel < 1e-12,
                "the channel set is not permutation closed: {rel:.2e}"
            );
        }

        let broken = (0..diagrams.len()).any(|k| {
            let short = build(Some(k));
            points.iter().any(|p| asymmetry(&short, p) > 1e-6)
        });
        assert!(
            broken,
            "no single channel carries the symmetry, so the closure check sees nothing"
        );
    }

    #[test]
    fn fixed_beam_integrand_finite_positive_2to2() {
        // The flat-RAMBO fixed-energy path on a clean s-channel 2→2 process:
        // a finite, positive σ, with the CM kinematics satisfying the pruned
        // evaluator's ±z-beam frame contract (it would assert otherwise).
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();

        let legs = process_external_legs(&evals[0], &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = evals[0].external_particles()[evals[0].n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(&evals[0], &m, &evaluated);

        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let integ = FixedBeamIntegrand::new(amps, &cuts, 500.0, masses, avg);
        assert_eq!(integ.vegas_ndim(), 8);
        let (sigma, err) = integ.integrate(20_000, 4, 0x5EED);
        assert!(sigma.is_finite() && sigma > 0.0, "sigma = {sigma}");
        assert!(err.is_finite() && err >= 0.0, "err = {err}");
    }

    /// Build the fixed-energy integrand builder + diagram list for a fixed-energy
    /// process at `sqrt_s`, holding the amplitude/cut/model state the closure borrows.
    fn fixed_energy_case(
        proc: &str,
    ) -> (
        std::sync::Arc<UFOModel>,
        EvaluatedModel,
        Vec<DiagramSet>,
        Vec<Diagram>,
    ) {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let card = parse_proc_card(&format!("generate {proc}"), &opts).unwrap();
        let sets = generate_from_proc_card(&card, &m).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        (m, evaluated, sets, diagrams)
    }

    /// Diagnostic: does the `from_diagram` multichannel reproduce the *phase-space
    /// volume* (integrand ≡ 1) for a real process? An unbiased combiner must, for
    /// any channel set — so a volume that departs from the analytic massless `V_n`
    /// localises a density/reciprocity bug in the per-diagram tree, independent of
    /// |M|². Run with `--ignored --nocapture`.
    #[test]
    #[ignore]
    fn probe_from_diagram_volume() {
        use crate::phasespace::rambo::massless_volume;
        use crate::phasespace::rng::SubStream;
        use crate::phasespace::{PhaseSpaceMap, RamboChannel};

        for proc in [
            "e+ e- > mu+ mu-",
            "e+ e- > ta+ ta- H",
            "e+ e- > mu+ mu- a",
            "e+ e- > mu+ mu- ta+ ta-",
        ] {
            let (m, evaluated, _sets, diagrams) = fixed_energy_case(proc);
            let rep_set = crate::diagrams::generate_from_proc_card(
                &parse_proc_card(&format!("generate {proc}"), &ParsingOptions::default()).unwrap(),
                &m,
            )
            .unwrap();
            let n_out = rep_set[0].particles_out.len();
            let sqrt_s = 500.0;
            let masses: Vec<f64> = vec![0.0; n_out];

            let channels: Vec<Box<dyn Channel<f64>>> = diagrams
                .iter()
                .map(|d| {
                    Box::new(DiagramChannel::from_diagram(d, &evaluated, sqrt_s))
                        as Box<dyn Channel<f64>>
                })
                .collect();
            let multi = MultiChannel::uniform(channels);
            let flat = RamboChannel::new(sqrt_s, masses.clone());
            let analytic = massless_volume(sqrt_s, n_out);

            let mc_vol = |map: &dyn PhaseSpaceMap<f64>, stream: u64| -> (f64, f64) {
                let mut s = SubStream::from_stream(0x5107, stream);
                let (mut sum, mut sq) = (0.0, 0.0);
                let nsamp = 2_000_000usize;
                for _ in 0..nsamp {
                    let u = s.uniforms::<f64>(map.ndim());
                    let w = map.sample(&u).weight;
                    sum += w;
                    sq += w * w;
                }
                let mean = sum / nsamp as f64;
                let err = ((sq / nsamp as f64 - mean * mean).max(0.0) / nsamp as f64).sqrt();
                (mean, err)
            };

            let (v_multi, e_multi) = mc_vol(&multi, 1);
            let (v_flat, e_flat) = mc_vol(&flat, 2);
            eprintln!(
                "[{proc}] n={n_out} {} diag | analytic V={analytic:.6e} | \
                 multi {v_multi:.6e} ± {e_multi:.2e} (dev {:+.2e}) | \
                 flat {v_flat:.6e} ± {e_flat:.2e} (dev {:+.2e})",
                diagrams.len(),
                v_multi / analytic - 1.0,
                v_flat / analytic - 1.0,
            );
        }
    }

    /// The production wiring's efficiency win: on a genuinely resonant fixed-energy
    /// process (`e+ e- > ta+ ta- h` at √s = 500 GeV, a Z → τ⁺τ⁻ pole in the τ-pair
    /// invariant) the per-diagram α-adapted [`MultiChannel`] sampler converges to a
    /// sharp σ̂ at a budget where flat RAMBO cannot resolve the pole at all.
    ///
    /// Flat RAMBO is the known-wrong baseline kept running alongside: because it
    /// almost never lands on the narrow peak, at an equal budget it under-counts σ̂ by
    /// orders of magnitude *and* its relative error stays large — the exact failure
    /// that lists this process SKIP for the flat sampler. The load-bearing figure of
    /// merit is therefore *relative* precision at equal budget (a peak-missing flat
    /// run has a small absolute error precisely because its samples are all ≈ 0), and
    /// the multichannel is orders of magnitude more precise. A wrong multichannel
    /// density would show up as a σ̂ that fails to match the (independently
    /// MG-banked) value in the σ gate, not here.
    #[test]
    fn multichannel_resolves_resonant_pole_flat_rambo_misses() {
        let (m, evaluated, sets, diagrams) = fixed_energy_case("e+ e- > ta+ ta- h");
        assert!(!diagrams.is_empty(), "process must enumerate diagrams");

        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let sqrt_s = 500.0;

        let build = || {
            let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
            FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses.clone(), avg)
        };

        // Flat RAMBO (the known-wrong baseline for a narrow pole) at a matched budget.
        let flat = build();
        let (sigma_flat, err_flat) = flat.integrate(60_000, 8, 0x5EED_1);

        // Per-diagram multichannel, α-adapted to this integrand, at the same budget.
        let mut multi = build();
        let report = multi
            .use_multichannel(&diagrams, &evaluated, 20_000, 6, 0x5EED_2)
            .expect("resonant process yields channels");
        let (sigma_mc, err_mc) = multi.integrate(60_000, 8, 0x5EED_3);

        let rel_flat = err_flat / sigma_flat.abs().max(1e-300);
        let rel_mc = err_mc / sigma_mc.abs().max(1e-300);
        eprintln!(
            "resonant σ̂(e+e- > ta+ ta- h): flat RAMBO {sigma_flat:.6e} ± {err_flat:.2e} pb \
             ({} dim, rel {rel_flat:.2e}) | multichannel {sigma_mc:.6e} ± {err_mc:.2e} pb \
             ({} dim, {} channels, rel {rel_mc:.2e}) | α = {:?}",
            flat.vegas_ndim(),
            multi.vegas_ndim(),
            diagrams.len(),
            report.trajectory.last().unwrap(),
        );

        assert!(
            sigma_mc.is_finite() && sigma_mc > 0.0,
            "multichannel σ̂ finite positive: {sigma_mc}"
        );
        // The multichannel converged to a sharp estimate.
        assert!(
            rel_mc < 1e-2,
            "multichannel did not converge: rel error {rel_mc:.2e}"
        );
        // The efficiency win at equal budget: the multichannel is far more precise
        // relative to its own estimate than flat RAMBO, which fails to resolve the pole.
        assert!(
            rel_mc < 0.1 * rel_flat,
            "multichannel not decisively more precise than flat RAMBO: \
             rel_mc {rel_mc:.2e} vs rel_flat {rel_flat:.2e}"
        );
        // And flat RAMBO visibly under-counts by missing the peak — the known-wrong
        // baseline firing.
        assert!(
            sigma_flat < 0.5 * sigma_mc,
            "flat RAMBO did not under-count the resonant σ̂ as expected: \
             flat {sigma_flat:.6e} vs multichannel {sigma_mc:.6e}"
        );
    }

    /// The production wiring's unbiasedness: on a fixed-energy process where flat
    /// RAMBO *does* converge (`e+ e- > mu+ mu-` at √s = 200 GeV, smooth and
    /// off-resonance), the per-diagram multichannel sampler integrates to the same
    /// σ̂ within the combined Monte-Carlo error. Swapping the flat map for the
    /// resonance-aware combiner must not move the cross section.
    #[test]
    fn multichannel_unbiased_vs_flat_where_both_converge() {
        let (m, evaluated, sets, diagrams) = fixed_energy_case("e+ e- > mu+ mu-");
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let sqrt_s = 200.0;

        let build = || {
            let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
            FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses.clone(), avg)
        };

        let flat = build();
        let (sigma_flat, err_flat) = flat.integrate(60_000, 8, 0x5EED_4);

        let mut multi = build();
        multi
            .use_multichannel(&diagrams, &evaluated, 20_000, 6, 0x5EED_5)
            .expect("process yields channels");
        let (sigma_mc, err_mc) = multi.integrate(60_000, 8, 0x5EED_6);

        eprintln!(
            "convergent σ̂(e+e- > mu+ mu- @200): flat RAMBO {sigma_flat:.6e} ± {err_flat:.2e} pb | \
             multichannel {sigma_mc:.6e} ± {err_mc:.2e} pb ({} channels)",
            diagrams.len(),
        );

        let comb = (err_flat * err_flat + err_mc * err_mc).sqrt();
        assert!(
            (sigma_flat - sigma_mc).abs() < 5.0 * comb,
            "multichannel σ̂ {sigma_mc:.6e} ± {err_mc:.2e} disagrees with flat RAMBO \
             {sigma_flat:.6e} ± {err_flat:.2e} (5σ = {:.2e})",
            5.0 * comb
        );
    }

    /// Build the flat-map fixed-energy integrand for a coloured `2 → 2` process,
    /// returning it alongside the amplitude the selections are checked against.
    fn colored_2to2_case(
        m: &UFOModel,
        evaluated: &EvaluatedModel,
        sqrt_s: f64,
    ) -> (Vec<AmplitudeEvaluator>, Cuts, Vec<f64>, f64) {
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate u u~ > u u~", &opts).unwrap();
        let sets = generate_from_proc_card(&card, m).unwrap();
        let evals = compile_subprocesses(&sets, m, evaluated).unwrap();
        let rep = &evals[0];
        let legs = process_external_legs(rep, m, evaluated);
        // The default card's jet cuts keep the t-channel singularity out of the
        // sampled region, so the probe point below is an ordinary phase-space point.
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, m, evaluated);
        let _ = sqrt_s;
        (evals, cuts, masses, avg)
    }

    /// The sampler's channels and the scale prescription's integration channels
    /// are one numbering apart, and this is where the two meet.
    ///
    /// `g g → g g` is the process where they provably differ: four diagrams, four
    /// sampling channels, three integration channels, because MadGraph's
    /// generator drops the four-gluon diagram. So the map cannot be the identity
    /// and cannot be assumed — a point drawn in the sampler's channel 2 is
    /// clustered in integration channel 2, not 3, and the channel with no config
    /// of its own falls back rather than indexing past the end.
    ///
    /// What this pins that `configs`' own test cannot is the *wiring*: that the
    /// integrand hands the scale source the same diagram slice its sampler is
    /// built from, so the two indices mean the same thing.
    #[test]
    fn a_sampled_channel_names_the_integration_channel_of_its_own_diagram() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let opts = ParsingOptions::default();
        let proc = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&proc, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        assert_eq!(diagrams.len(), 4);

        // Both scales free at `dynamical_scale_choice = -1`, which is the branch
        // that reads a channel at all.
        let card = RunCard::parse(
            "  0 = lpp1
  0 = lpp2
  250.0 = ebeam1
  250.0 = ebeam2
             \x20 False = fixed_ren_scale
  False = fixed_fac_scale1
             \x20 False = fixed_fac_scale2
  -1 = dynamical_scale_choice
             \x20 4 = maxjetflavor
",
        )
        .expect("run card");

        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&card, &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let mut integ =
            FixedBeamIntegrand::new(bounds.iter().collect(), &cuts, sqrt_s, masses, avg);
        let report = integ
            .use_running_coupling(&diagrams, &m, &evaluated, &card)
            .expect("the clustering scale compiles for g g > g g");
        assert_eq!(report.channels, Some(3));
        assert_eq!(report.unmapped_channels, 1);

        let sets = integ
            .scale_source()
            .and_then(EventScaleSource::channels)
            .expect("the clustering branch carries channel forests");
        assert_eq!(sets.len(), 1, "one subprocess, one channel set");
        let channels = &sets[0];
        assert_eq!(channels.diagram_count(), diagrams.len());
        assert_eq!(
            (0..4)
                .map(|j| channels.config_of_diagram(j))
                .collect::<Vec<_>>(),
            vec![None, Some(1), Some(2), Some(3)],
        );
        // The unmapped channel takes the default rather than an out-of-range
        // index or the identity's `1`-off answer.
        assert_eq!(
            (0..4)
                .map(|j| channels.config_of_channel(j))
                .collect::<Vec<_>>(),
            vec![1, 1, 2, 3],
        );

        let diagram_count = channels.diagram_count();
        // And the sampler is built over the same slice, so `j` means the same
        // thing on both sides.
        integ.use_multichannel(&diagrams, &evaluated, 512, 1, 0x5EED_C6);
        assert_eq!(integ.channel_count(), diagram_count);
    }

    /// The evaluator's `AMP2` configuration order and the channel forests' agree,
    /// over a process where the two are known to be derived independently.
    ///
    /// `AMP2` index `c` names a configuration in
    /// [`AmplitudeEvaluator::config_diagrams`]' order; `ClusterInput::this_config`
    /// names one in `derive_channels`' order. Both are derived from the same
    /// diagram slice, by different code. Production does not assume they coincide
    /// — the configuration draw composes through the diagram index, which is the
    /// pair's common ground — so this is not a correctness requirement. It exists
    /// so that a reorder on either side becomes a named finding here instead of a
    /// silent reshuffle of every event's scale.
    ///
    /// `g g → g g` is the case worth pinning: four diagrams give three
    /// configurations, so the diagram→configuration map is not the identity and
    /// an off-by-one would survive a process where it is.
    #[test]
    fn the_amp2_configuration_order_matches_the_forest_order() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let proc = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&proc, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let card = RunCard::parse(
            "  0 = lpp1
  0 = lpp2
  250.0 = ebeam1
  250.0 = ebeam2
  False = fixed_ren_scale
  False = fixed_fac_scale1
  False = fixed_fac_scale2
  -1 = dynamical_scale_choice
  4 = maxjetflavor
",
        )
        .expect("run card");
        let source =
            compile_scale_source(&[(&evals[0], &diagrams)], &m, &evaluated, &card, None, true)
                .expect("the clustering scale compiles");
        let channels = &source.channels().expect("channel forests")[0];
        let eval = &evals[0];

        assert_eq!(
            eval.n_configs(),
            channels.len(),
            "the evaluator and the channel forests disagree about how many \
             integration configurations this process has"
        );
        for (c, &diagram) in eval.config_diagrams().iter().enumerate() {
            assert_eq!(
                channels.config_of_diagram(diagram),
                Some(c + 1),
                "AMP2 configuration {c} is diagram {diagram}, which the channel \
                 forests number differently"
            );
        }
    }

    /// MadEvent's configuration weight is the squared amplitude alone only under a
    /// run-card conjunction, and the prescription reports which side of it a card
    /// falls on.
    ///
    /// `matrix1.f`'s enhancement block weights configuration `c` by
    /// `AMP2_c · CC_c`, and `genps.f`'s `get_channel_cut` short-circuits to
    /// `CC_c = 1` only for `sde_strat == 1` *and* `tmin_for_channel == -1`. At
    /// `sde_strategy = 2` the squared amplitude is discarded outright and the
    /// weight is a product of propagator denominators; with `tmin_for_channel`
    /// set, the same product multiplies it. This crate implements neither, so on
    /// those cards the scale keeps the sampling channel.
    ///
    /// Without this the condition is an unexercised branch: every banked run whose
    /// scale clusters *and* whose matrix element carries the strong coupling sits
    /// on the drawing side of it, so no gate reaches the other.
    #[test]
    fn the_configuration_draw_needs_both_run_card_fields() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let opts = ParsingOptions::default();
        let proc = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&proc, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();

        let source_for = |extra: &str| {
            let card = RunCard::parse(&format!(
                "  0 = lpp1
  0 = lpp2
  250.0 = ebeam1
  250.0 = ebeam2
  False = fixed_ren_scale
  False = fixed_fac_scale1
  False = fixed_fac_scale2
  -1 = dynamical_scale_choice
  4 = maxjetflavor
{extra}"
            ))
            .expect("run card");
            compile_scale_source(&[(&evals[0], &diagrams)], &m, &evaluated, &card, None, true)
                .expect("the clustering scale compiles")
        };

        assert!(
            source_for("  1 = sde_strategy\n").draws_configuration(),
            "the default conjunction is what MadEvent's simple rule holds under"
        );
        assert!(
            source_for("").draws_configuration(),
            "both fields absent is the default conjunction"
        );
        assert!(
            !source_for("  2 = sde_strategy\n").draws_configuration(),
            "at sde_strategy = 2 the configuration weight is not the squared amplitude"
        );
        // The other half of the conjunction is refused a step earlier: a card that
        // sets `tmin_for_channel` at all does not parse, so the scale source's own
        // test of it can never be the thing that fires. Both guards are kept —
        // this is what says the refusal is what stands between such a card and a
        // cross section taken under a rule that does not describe it.
        assert!(RunCard::parse(
            "  0 = lpp1
  0 = lpp2
  250.0 = ebeam1
  250.0 = ebeam2
  10.0 = tmin_for_channel
"
        )
        .is_err());
        // A card that fixes every scale resolves to a constant before any event is
        // seen, so there is no configuration to draw whatever the two fields say.
        let fixed = RunCard::parse(
            "  0 = lpp1
  0 = lpp2
  250.0 = ebeam1
  250.0 = ebeam2
  True = fixed_ren_scale
  True = fixed_fac_scale1
  True = fixed_fac_scale2
  -1 = dynamical_scale_choice
  1 = sde_strategy
",
        )
        .expect("run card");
        assert!(!compile_scale_source(
            &[(&evals[0], &diagrams)],
            &m,
            &evaluated,
            &fixed,
            None,
            true,
        )
        .expect("a fixed prescription compiles")
        .draws_configuration());
    }

    /// The parallel channel integration reproduces the sequential one it replaced,
    /// point for point.
    ///
    /// A pool-size comparison cannot see this: an addressing error in the
    /// grid's substream is the same error on every thread, so both arms of a
    /// thread-count test agree on the wrong answer. What pins it is a sequential
    /// reference — one generator, consumed point by point, exactly as a serial
    /// integrator would — written out here rather than shared with the
    /// implementation, so the two cannot drift together.
    ///
    /// This covers the grid coordinates and the per-channel budgets. It says
    /// nothing about the *configuration-draw* substream: a fixed-beam `2 → 2`
    /// clusters to one scale whichever configuration is drawn, so the trailing
    /// uniform is inert here and both sides would agree however it were addressed.
    /// The proton path's `the_parallel_integration_reproduces_a_sequential_one`
    /// is where that stream carries weight.
    #[test]
    fn adapt_grids_reproduces_a_sequential_integration() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let opts = ParsingOptions::default();
        let proc = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&proc, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let card = RunCard::parse(
            "  0 = lpp1\n  0 = lpp2\n  250.0 = ebeam1\n  250.0 = ebeam2\n\
             \x20 False = fixed_ren_scale\n  -1 = dynamical_scale_choice\n\
             \x20 4 = maxjetflavor\n",
        )
        .expect("run card");
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&card, &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let mut integ =
            FixedBeamIntegrand::new(bounds.iter().collect(), &cuts, sqrt_s, masses, avg);
        integ
            .use_running_coupling(&diagrams, &m, &evaluated, &card)
            .expect("a fixed-beam card is never refused for the factorisation floor");
        integ.use_multichannel(&diagrams, &evaluated, 2_000, 3, 0x5EED_9C);

        let (seed, neval, niter) = (0x5EED_9D, 3_000, 4);
        let (got, _) = integ.adapt_grids(neval, niter, seed);

        let ndim = integ.channel_grid_ndim();
        let point_ndim = integ.point_ndim();
        let alphas = integ.channel_alphas();
        for (j, &alpha) in alphas.iter().enumerate() {
            let n_j = if alphas.len() == 1 {
                neval
            } else {
                channel_neval(alpha, neval)
            };
            let mut grid = VegasGrid::new(ndim, VEGAS_NBINS, VEGAS_ALPHA_MAPPED);
            let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            rng.set_stream(CHANNEL_STREAM_BASE + j as u64);
            rng.set_word_pos(0);
            let mut scale_draw = SubStream::from_stream(seed, SCALE_DRAW_STREAM_BASE + j as u64);
            let mut point = vec![0.0; point_ndim];
            let want = grid.adapt(
                |u| {
                    point[..ndim].copy_from_slice(u);
                    scale_draw.fill_uniforms(&mut point[ndim..]);
                    integ.value_in_channel(j, &point)
                },
                n_j,
                niter,
                &mut rng,
            );
            assert_eq!(
                got[j].result.integral.to_bits(),
                want.integral.to_bits(),
                "channel {j} term"
            );
            assert_eq!(got[j].neval, n_j, "channel {j} budget");
            for (dim, (a, b)) in got[j].grid.xi().iter().zip(grid.xi()).enumerate() {
                assert_eq!(a, b, "channel {j} grid edges of dim {dim}");
            }
        }
    }

    /// The channel-split integration is the same numbers at any pool size — the
    /// property the parallel CLI rests on, checked on the integrand rather than on
    /// a plain closure.
    ///
    /// `g g > g g` with a per-event renormalisation scale is the case a shared
    /// integrand would corrupt: every point moves the amplitude's constant pools,
    /// so two threads sharing one would read each other's coupling and return a
    /// wrong `|M|²` with no panic and no NaN. The trained grid edges are compared
    /// as well as the estimate, because a refinement fed differently-rounded
    /// histograms agrees on the first iteration and diverges after it.
    #[test]
    fn adapt_grids_does_not_depend_on_the_pool_size() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let opts = ParsingOptions::default();
        let proc = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&proc, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        let card = RunCard::parse(
            "  0 = lpp1\n  0 = lpp2\n  250.0 = ebeam1\n  250.0 = ebeam2\n\
             \x20 False = fixed_ren_scale\n  -1 = dynamical_scale_choice\n\
             \x20 4 = maxjetflavor\n",
        )
        .expect("run card");
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&card, &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let mut integ =
            FixedBeamIntegrand::new(bounds.iter().collect(), &cuts, sqrt_s, masses, avg);
        integ
            .use_running_coupling(&diagrams, &m, &evaluated, &card)
            .expect("a fixed-beam card is never refused for the factorisation floor");
        integ.use_multichannel(&diagrams, &evaluated, 2_000, 3, 0x5EED_9A);

        let run = |threads: usize| {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| integ.adapt_grids(4_000, 4, 0x5EED_9B))
        };
        let (one_channels, one) = run(1);
        let (many_channels, many) = run(8);

        assert_eq!(one.integral.to_bits(), many.integral.to_bits(), "integral");
        assert_eq!(one.std_dev.to_bits(), many.std_dev.to_bits(), "std_dev");
        assert!(one.integral > 0.0, "the test integrated nothing");
        assert_eq!(one_channels.len(), many_channels.len());
        for (j, (a, b)) in one_channels.iter().zip(&many_channels).enumerate() {
            assert_eq!(
                a.result.integral.to_bits(),
                b.result.integral.to_bits(),
                "channel {j} term"
            );
            for (dim, (ea, eb)) in a.grid.xi().iter().zip(b.grid.xi()).enumerate() {
                assert_eq!(ea, eb, "channel {j} grid edges of dim {dim}");
            }
        }
    }

    /// The factorisation floor is unreachable on fixed-energy beams, and a
    /// `scalefact` that would put a proton run entirely below it changes nothing
    /// here.
    ///
    /// `ScaleChoice::from_run_card` reads the floor's applicability off the card
    /// as `lpp != 0`, so a fixed-energy card gives `false` on both beams and
    /// `setclscales` short-circuits before the comparison — the refusal is never
    /// constructed. That argument is upstream of how a refusal is routed, which is
    /// why the routing carries an `unreachable!` rather than a branch; this is
    /// what would fail if the `beam_has_pdf` guard were dropped or inverted.
    ///
    /// It says nothing about proton runs.
    #[test]
    fn the_factorisation_floor_is_unreachable_on_fixed_beams() {
        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let opts = ParsingOptions::default();
        let proc = parse_proc_card("generate g g > g g", &opts).unwrap();
        let sets = generate_from_proc_card(&proc, &m).unwrap();
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let diagrams: Vec<Diagram> = sets
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();

        // The same scalefact that leaves a proton run with no support above the
        // floor at all, with `μR` pinned to the card so the comparison is about
        // the factorisation scale alone: `scalefact` scales both, and a `μR` of
        // order 0.01 GeV would leave the strong coupling below its own table
        // long before any factorisation floor came into it.
        let card = RunCard::parse(
            "  0 = lpp1\n  0 = lpp2\n  250.0 = ebeam1\n  250.0 = ebeam2\n\
             \x20 True = fixed_ren_scale\n  91.188 = scale\n\
             \x20 False = fixed_fac_scale1\n  False = fixed_fac_scale2\n\
             \x20 -1 = dynamical_scale_choice\n  1e-4 = scalefact\n  4 = maxjetflavor\n",
        )
        .expect("run card");

        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&card, &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let mut integ =
            FixedBeamIntegrand::new(bounds.iter().collect(), &cuts, sqrt_s, masses, avg);
        integ
            .use_running_coupling(&diagrams, &m, &evaluated, &card)
            .expect("a fixed-beam card is never refused for the factorisation floor");
        integ.use_multichannel(&diagrams, &evaluated, 512, 1, 0x5EED_C7);

        let mut stream = crate::phasespace::rng::SubStream::from_stream(0x00C2_C333, 3);
        let mut carried = 0usize;
        let draws = 200usize;
        for i in 0..draws {
            let u = stream.uniforms::<f64>(integ.point_ndim());
            if FixedBeamIntegrand::value_in_channel(&integ, i % integ.channel_count(), &u) > 0.0 {
                carried += 1;
            }
        }
        assert!(
            carried > 0,
            "no fixed-beam point carried weight, so the floor's absence was never exercised"
        );
        eprintln!(
            "factorisation floor: {carried} of {draws} fixed-beam points carry weight at \
             scalefact = 1e-4, where a proton run has no support above the floor"
        );
    }

    /// The per-event helicity and colour-flow draws must reproduce the diagonals
    /// they read — the property `SELECT_HEL` and `SELECT_COLOR` rest on, and the one
    /// a wrong accumulator or a mis-indexed draw would break while still returning
    /// perfectly plausible labels.
    #[test]
    fn selected_helicity_and_flow_frequencies_follow_the_diagonals() {
        use crate::phasespace::rng::SubStream;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let (evals, cuts, masses, avg) = colored_2to2_case(&m, &evaluated, sqrt_s);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses, avg);

        // One ordinary phase-space point of the flat map.
        let u: Vec<f64> = (0..integ.point_ndim())
            .map(|k| 0.11 + 0.07 * (k as f64 % 7.0))
            .collect();
        let mut momenta = Vec::new();
        let value = integ.event_in_channel(0, &u, &mut momenta);
        assert!(value > 0.0, "the probe point must pass the cuts");

        // The diagonals the draws are supposed to follow, taken directly.
        let eval = integ.subprocess_evaluator(0);
        let mut scratch = bounds[0].scratch_space();
        let mut ext = integ.beams().to_vec();
        ext.extend_from_slice(&momenta);
        let mut hel_m2 = vec![0.0; eval.helicities().len()];
        let mut amp2 = vec![0.0; eval.n_configs()];
        let mut jamp2 = vec![0.0; eval.n_flows()];
        bounds[0].eval_hel_m2(&ext, &mut scratch, &mut hel_m2);
        bounds[0].eval_amp2(&ext, &mut scratch, &mut amp2);
        bounds[0].eval_jamp2(&ext, &mut scratch, &mut jamp2);
        assert!(
            eval.n_flows() > 1,
            "a single flow makes the colour draw vacuous"
        );
        assert!(
            eval.n_configs() > 1,
            "a single configuration makes the conditioning vacuous"
        );

        // The colour law the draw is supposed to follow, composed here rather than
        // read off one accumulator: the configuration share times that
        // configuration's own admitted-flow share.
        let flow_law = {
            let total: f64 = amp2.iter().sum();
            let mut p = vec![0.0; jamp2.len()];
            for (c, &w) in amp2.iter().enumerate() {
                let reached = eval
                    .leading_color_flows()
                    .reached_by(eval.config_diagrams()[c]);
                let masked: Vec<f64> = jamp2
                    .iter()
                    .zip(reached)
                    .map(|(&j, &ok)| if ok { j } else { 0.0 })
                    .collect();
                // `select_flow_reached_by` drops a mask that carries no probability.
                let admitted: f64 = masked.iter().sum();
                let (weights, norm) = if admitted > 0.0 {
                    (&masked, admitted)
                } else {
                    (&jamp2, jamp2.iter().sum::<f64>())
                };
                for (acc, &j) in p.iter_mut().zip(weights.iter()) {
                    *acc += (w / total) * (j / norm);
                }
            }
            p
        };
        assert!(
            hel_m2.iter().filter(|&&w| w > 0.0).count() > 1,
            "a single contributing helicity makes the helicity draw vacuous"
        );

        let n = 200_000;
        let mut hel_counts = vec![0usize; hel_m2.len()];
        let mut flow_counts = vec![0usize; jamp2.len()];
        let mut s = SubStream::from_stream(0x5E1E_C701, 4);
        for _ in 0..n {
            let sel = integ
                .select_event(
                    &momenta,
                    0,
                    [
                        s.next_uniform::<f64>(),
                        s.next_uniform::<f64>(),
                        s.next_uniform::<f64>(),
                        s.next_uniform::<f64>(),
                    ],
                )
                .expect("a point with weight carries labels");
            let c = eval
                .helicities()
                .iter()
                .position(|h| h.as_slice() == sel.helicity.as_slice())
                .expect("a selected helicity is one of the combinations");
            hel_counts[c] += 1;
            flow_counts[sel.flow] += 1;
        }

        let check = |counts: &[usize], weights: &[f64], what: &str| {
            let total: f64 = weights.iter().sum();
            for (i, (&c, &w)) in counts.iter().zip(weights).enumerate() {
                let p = w / total;
                let f = c as f64 / n as f64;
                let sigma = (p * (1.0 - p) / n as f64).sqrt();
                assert!(
                    (f - p).abs() <= 5.0 * sigma + 1e-12,
                    "{what} {i}: frequency {f:.5} vs {p:.5} (5σ = {:.5})",
                    5.0 * sigma
                );
            }
        };
        check(&hel_counts, &hel_m2, "helicity");
        check(&flow_counts, &flow_law, "flow");
    }

    /// Selection must be provably neutral, not neutral by construction: the same
    /// accept/reject pass, driven from the same RNG, must produce bit-for-bit the
    /// same trials, the same accepted points and the same cross section whether or
    /// not every accepted event is labelled along the way.
    ///
    /// This fires on the two ways a selection can leak — consuming from the
    /// sampling stream, and leaving the amplitude's per-event state (its coupling,
    /// its scratch) different from how the integrand left it.
    #[test]
    fn labelling_events_moves_neither_the_trials_nor_the_cross_section() {
        use crate::phasespace::rng::SubStream;
        use crate::unweight::Unweighter;

        let m = model();
        let evaluated = EvaluatedModel::from_model(m.clone());
        let sqrt_s = 500.0;
        let (evals, cuts, masses, avg) = colored_2to2_case(&m, &evaluated, sqrt_s);
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
        let mut integ = FixedBeamIntegrand::new(amps, &cuts, sqrt_s, masses, avg);
        let card = parse_proc_card("generate u u~ > u u~", &ParsingOptions::default()).unwrap();
        let diagrams: Vec<Diagram> = generate_from_proc_card(&card, &m)
            .unwrap()
            .iter()
            .flat_map(|s| s.diagrams.iter().cloned())
            .collect();
        integ.use_multichannel(&diagrams, &evaluated, 5_000, 4, 0x5EED_0);

        let (channels, _) = integ.adapt_grids(4_000, 3, 0x5EED);
        let build = || {
            Unweighter::scan(
                &integ,
                channels.iter().map(|c| (&c.grid, 20_000)),
                0x5CA7_0FF1,
            )
        };

        let mut labelled = build();
        let mut plain = build();
        let mut rng_a = rand_chacha::ChaCha8Rng::seed_from_u64(0xE7E7);
        let mut rng_b = rand_chacha::ChaCha8Rng::seed_from_u64(0xE7E7);
        // The labels ride on their own stream, the discipline a generator follows.
        let mut labels = SubStream::from_stream(0x5E1E_C702, 4);
        let mut momenta = Vec::new();
        let mut seen_helicities = std::collections::BTreeSet::new();
        let mut seen_flows = std::collections::BTreeSet::new();

        for _ in 0..20_000 {
            let a = labelled.trial(&integ, &mut rng_a);
            let b = plain.trial(&integ, &mut rng_b);
            match (&a, &b) {
                (Some(x), Some(y)) => {
                    assert_eq!(x.channel, y.channel);
                    assert_eq!(x.u, y.u);
                    assert_eq!(x.weight, y.weight);
                }
                (None, None) => {}
                _ => panic!("labelling changed which trials were accepted"),
            }
            if let Some(point) = a {
                integ.event_in_channel(point.channel, &point.u, &mut momenta);
                let sel = integ
                    .select_event(
                        &momenta,
                        point.channel,
                        [
                            labels.next_uniform::<f64>(),
                            labels.next_uniform::<f64>(),
                            labels.next_uniform::<f64>(),
                            labels.next_uniform::<f64>(),
                        ],
                    )
                    .expect("an accepted point carries labels");
                seen_helicities.insert(sel.helicity.clone());
                seen_flows.insert(sel.flow);
            }
        }

        let (sa, sb) = (labelled.stats(), plain.stats());
        assert_eq!(sa.trials, sb.trials);
        assert_eq!(sa.accepted, sb.accepted);
        assert_eq!(sa.ratio_sum, sb.ratio_sum, "the weight sum moved");
        assert_eq!(
            labelled.sigma_from_events(),
            plain.sigma_from_events(),
            "labelling moved the cross section"
        );
        assert!(
            sa.accepted > 100,
            "too few events for the check to mean much"
        );
        // A run that only ever produced one label would satisfy the above
        // vacuously. Only the helicities carry that check: conditioning the colour
        // draw on the integration configuration puts 99.96% of this process's
        // events on one flow, so a sample of this size seeing a single flow is the
        // rule working rather than a dead label — the law itself is checked at
        // 200k draws by `selected_helicity_and_flow_frequencies_follow_the_diagonals`.
        assert!(seen_helicities.len() > 1);
        assert!(!seen_flows.is_empty());
    }

    /// Replaying banked channel weights must rebuild the *same* integrand, not a
    /// similar one: a sampling phase draws against grids trained on it, and `αⱼ`
    /// enters every channel's weight. Bit-for-bit is the right bar — anything
    /// looser would let a re-survey pass while the grids no longer fit.
    #[test]
    fn banked_channel_weights_rebuild_the_integrand_bit_for_bit() {
        // A resonant multi-channel process, so the adaptation converges somewhere
        // far from uniform. On a two-channel process it converges *to* uniform,
        // where `αⱼ` cancels between a channel's weight and the mixture density and
        // the comparison below cannot see whether the weights were installed at all.
        let (m, evaluated, sets, diagrams) = fixed_energy_case("e+ e- > ta+ ta- h");
        let evals = compile_subprocesses(&sets, &m, &evaluated).unwrap();
        let bounds: Vec<_> = evals
            .iter()
            .map(|e| BoundAmplitude::<f64>::bind(e, &evaluated))
            .collect();
        let rep = &evals[0];
        let legs = process_external_legs(rep, &m, &evaluated);
        let cuts = Cuts::compile(&RunCard::default(), &legs).unwrap();
        let masses: Vec<f64> = rep.external_particles()[rep.n_in()..]
            .iter()
            .map(|&id| evaluated.mass(id))
            .collect();
        let avg = initial_spin_color_average(rep, &m, &evaluated);
        let build = || {
            let amps: Vec<&BoundAmplitude<f64>> = bounds.iter().collect();
            FixedBeamIntegrand::new(amps, &cuts, 500.0, masses.clone(), avg)
        };

        let mut adapted = build();
        adapted
            .use_multichannel(&diagrams, &evaluated, 3_000, 4, 0x5EED_A)
            .expect("channels");
        let alphas = adapted.channel_alphas();
        let uniform = 1.0 / alphas.len() as f64;
        assert!(
            alphas.iter().any(|a| (a - uniform).abs() > 0.05),
            "the adaptation left the weights uniform ({alphas:?}), where they cancel out of the \
             mixture and the comparison below would be blind to them"
        );

        let mut replayed = build();
        replayed
            .use_multichannel_with_alphas(&diagrams, &evaluated, &alphas)
            .expect("channels")
            .expect("one weight per channel");

        assert_eq!(replayed.channel_count(), adapted.channel_count());
        assert_eq!(replayed.channel_grid_ndim(), adapted.channel_grid_ndim());
        let ndim = adapted.point_ndim();
        let mut compared = 0;
        for channel in 0..adapted.channel_count() {
            for step in 1..=7 {
                let u: Vec<f64> = (0..ndim)
                    .map(|d| ((step * (d + 3) + channel) % 9) as f64 / 10.0 + 0.05)
                    .collect();
                let a = adapted.value_in_channel(channel, &u);
                let b = replayed.value_in_channel(channel, &u);
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "channel {channel} at {u:?}: {a} vs {b}"
                );
                compared += usize::from(a > 0.0);
            }
        }
        assert!(
            compared > 0,
            "every probe was cut away, so nothing was compared"
        );

        // The comparison above is only evidence if the weights reach the mixture:
        // installing different ones has to move the same probes.
        let mut skewed = build();
        let mut other = vec![0.1 / (alphas.len() - 1) as f64; alphas.len()];
        other[0] = 0.9;
        skewed
            .use_multichannel_with_alphas(&diagrams, &evaluated, &other)
            .expect("channels")
            .expect("one weight per channel");
        let mut moved = 0;
        for channel in 0..adapted.channel_count() {
            for step in 1..=7 {
                let u: Vec<f64> = (0..ndim)
                    .map(|d| ((step * (d + 3) + channel) % 9) as f64 / 10.0 + 0.05)
                    .collect();
                let a = adapted.value_in_channel(channel, &u);
                moved += usize::from(a.to_bits() != skewed.value_in_channel(channel, &u).to_bits());
            }
        }
        assert!(
            moved > 0,
            "the selection weights are not reaching the mixture, so replaying them proves nothing"
        );

        // A weight list of the wrong length is what a proc card for a different
        // process looks like from here, and is reported rather than installed.
        let mut wrong = build();
        let short = &alphas[..alphas.len() - 1];
        assert_eq!(
            wrong.use_multichannel_with_alphas(&diagrams, &evaluated, short),
            Some(Err(diagrams.len()))
        );
    }
}
