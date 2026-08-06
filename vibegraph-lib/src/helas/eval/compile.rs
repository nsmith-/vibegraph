//! Compilation: DiagramSet + UFOModel → a card-independent [`AmplitudeEvaluator`].
//!
//! This module orchestrates the compile-time phase of amplitude evaluation:
//! - pass 1+2: root each `DiagramView` into a [`DiagramEval`] (topology + Lorentz
//!   structures, still model-bound; see `root_diagram` / `root_lorentz`),
//! - pass 3a: [`lower`] inlines every diagram into one whole-amplitude `Ast<Sym>`,
//! - pass 3b: [`Folded::build`] interns the constants into a card-independent skeleton.
//!
//! The result is independent of both the parameter card and the scalar field `F`.
//! Resolving a card (and choosing `F`) happens in
//! [`BoundAmplitude::bind`](super::run::BoundAmplitude::bind), which produces the
//! runtime [`BoundAmplitude`](super::run::BoundAmplitude).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use num_rational::Ratio;
use rand::rngs::StdRng;
use rand::SeedableRng;
use tracing::{debug, info, info_span, trace};

use crate::diagrams::{Diagram, DiagramSet};
use crate::helas::color::colorize_process;
use crate::helas::color::flow_tags::{
    color_flow_tags, select_flow, select_flow_reached_by, ColorFlowTags, LeadingColorFlows,
    LegColor,
};
use crate::helas::repr::color::ColorRep;
use crate::helas::repr::lorentz::LorentzVector;
use crate::phasespace::rambo_massive;
use crate::select::select_index;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::error::EvalError;
use super::fold::Folded;
use super::lower;
use super::root_diagram::{compile_single_diagram, DiagramEval};
use super::run::BoundAmplitude;

/// What one colour flow is made of, as a structural key: per contribution to its
/// JAMP, the `(diagram, colour-index chain, power of Nc, |rational coefficient|)`,
/// sorted so the key does not depend on the order the contributions were collected.
///
/// The coefficient's **sign and its `i` phase are deliberately excluded**. Charge
/// conjugation flips a contribution's sign — `T^a → −T^{aᵀ}` puts a `(−1)ⁿ` on a
/// diagram with `n` gluon vertices — so two subprocesses that are each other's
/// conjugate carry the same flows with some signs flipped. What identifies a flow
/// across such a pair is which diagram lands on it at which power of `Nc`, and that
/// phase does not move it.
pub type FlowFingerprint = Vec<(usize, Vec<u8>, i32, Ratio<i64>)>;

/// Compiled amplitude evaluator for a whole process (card- and `F`-independent).
///
/// Built once into a [`Folded`] skeleton (pass 1+2 rooting → `lower` → `fold`).
/// [`BoundAmplitude::bind`](super::run::BoundAmplitude::bind) resolves a
/// `&EvaluatedModel` at a chosen scalar precision `F` into a runtime
/// [`BoundAmplitude`](super::run::BoundAmplitude), so the same evaluator works with any
/// parameter card and any precision.
#[derive(Debug)]
pub struct AmplitudeEvaluator {
    /// Folded whole-amplitude AST + constant-pool specs.
    folded: Folded,
    /// Helicity-expanded arena (every combination baked in under an `Op::Hels` root,
    /// hash-consed across combinations), built on first use — `eval_m2` is its only
    /// consumer, so compile-only and single-helicity users never pay the expansion.
    /// Shares the numeric pools with `folded`, so one `bind` serves both.
    folded_hel: OnceLock<Folded>,
    /// Number of external particles
    n_ext: usize,
    /// Number of incoming external particles
    n_in: usize,
    /// Number of diagrams folded into the amplitude
    n_diagrams: usize,
    /// External particle ids in process order (incoming first, then outgoing)
    ext_particle_ids: Vec<ParticleId>,
    /// All valid helicity combinations (precomputed)
    helicities: Vec<Vec<i32>>,
    /// Number of color flows (NCOLOR): the JAMP count. `1` for color-free and
    /// single-color-structure processes.
    n_flows: usize,
    /// Exact color-factor matrix `CF_{ij}` (row-major, `cf_matrix[i*n_flows + j]`),
    /// evaluated at `Nc = 3`. `BoundAmplitude::bind` resolves it to the scalar field.
    cf_matrix: Vec<Ratio<i64>>,
    /// The colour rep and direction of every external leg, in process order — the
    /// legs `color_flow_tags` was derived and checked against.
    leg_colors: Vec<LegColor>,
    /// Per flow, a sorted fingerprint of the contributions summing into its JAMP.
    /// See [`Self::flow_fingerprints`].
    flow_fingerprints: Vec<FlowFingerprint>,
    /// Per-flow Les Houches `(color, anticolor)` line labels for every external leg,
    /// derived from the same basis keys the flows are indexed by.
    color_flow_tags: ColorFlowTags,
    /// Which flows each diagram reaches at leading order in `Nc` (MadGraph's
    /// `ICOLAMP`), read off the same basis as the flow tags.
    leading_color_flows: LeadingColorFlows,
    /// The diagram behind each integration configuration, in configuration order —
    /// the diagrams MadGraph writes an `AMP2` for (see
    /// [`config_carrying_diagrams`]). Indexes [`Self::leading_color_flows`].
    config_diagrams: Vec<usize>,
    /// How many `(diagram, color chain)` amplitudes each configuration owns, parallel
    /// to `config_diagrams`. All but a four-point-vertex diagram carry exactly one;
    /// the sum is the width of the compiled program's configuration-amplitude row.
    config_spans: Vec<usize>,
    /// Set by [`prune_zero_helicities`](Self::prune_zero_helicities) once it has
    /// actually dropped combinations. `eval_m2` on a pruned evaluator only sums the
    /// survivors, so it is correct only under that method's kinematic contract
    /// (partonic-CM momenta, beams along ±z) — see [`Self::is_pruned`].
    pruned: bool,
    /// Helicity-expanded arena node counts before and after the zero-amplitude
    /// elimination pass (the second helicity-filter layer run by
    /// [`prune_zero_helicities`](Self::prune_zero_helicities)); both `0` until it runs.
    /// A diagnostic for how much the per-`(helicity, diagram)` skipping reclaims.
    zeroamp_nodes_before: usize,
    zeroamp_nodes_after: usize,
}

impl AmplitudeEvaluator {
    /// Compile from a DiagramSet + UFO model (symbolic, no param card needed).
    pub fn compile(set: &DiagramSet, model: &UFOModel) -> Result<Self, EvalError> {
        let subprocess = format!(
            "{} > {}",
            set.particles_in.join(" "),
            set.particles_out.join(" ")
        );
        let _span = info_span!("compile", process = %subprocess).entered();
        let started = std::time::Instant::now();
        let ext_particle_names = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .cloned()
            .collect::<Vec<_>>();

        let ext_particle_ids = ext_particle_names
            .iter()
            .map(|name| {
                model
                    .particle_id(name)
                    .ok_or_else(|| EvalError::ParticleNotFound(name.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Pass C: factorize color into a basis of flows + the exact CF matrix. Each
        // contribution names an amplitude by `(diagram, color-index chain)`.
        let basis = colorize_process(model, &set.diagrams)?;
        let n_flows = basis.ncolor();
        info!("{n_flows} colour flows, CF matrix {n_flows}×{n_flows}");
        report_cf_matrix(n_flows, &basis.cf_matrix);

        // Root each distinct `(diagram, chain)` amplitude the flows reference. A chain
        // selects one color structure per vertex; for single-structure vertices this
        // is the all-zero chain and matches the color-free rooting exactly.
        let mut evals: HashMap<(usize, Vec<u8>), DiagramEval> = HashMap::new();
        for elem in &basis.elements {
            for contrib in &elem.contributions {
                let key = (contrib.diagram, contrib.chain.clone());
                if let std::collections::hash_map::Entry::Vacant(slot) = evals.entry(key) {
                    let eval = compile_single_diagram(
                        &set.diagrams[contrib.diagram],
                        model,
                        &contrib.chain,
                    )?;
                    slot.insert(eval);
                }
            }
        }
        let n_ext = ext_particle_ids.len();

        // Compile phase should preserve process external-leg count consistency.
        if let Some(eval) = evals.values().next() {
            if eval.n_ext != n_ext {
                return Err(EvalError::TopologyError(format!(
                    "process has {n_ext}, AST has {}",
                    eval.n_ext
                )));
            }
        }

        let helicity_states = ext_particle_ids
            .iter()
            .map(|&pid| {
                let particle = model.particle(pid);
                helicity_states_for_spin(particle.spin, particle.mass_param == "ZERO")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let helicities = cartesian_helicity_product(&helicity_states);

        // Pass 3: inline the color-factorized diagrams into one whole-amplitude AST
        // (one JAMP per flow under a `Flows` root, or a single scalar root when
        // `NCOLOR = 1`), then intern the constants into the folded skeleton. The
        // configuration amplitudes ride under the same root.
        let n_diagrams = set.diagrams.len();
        let mut config_diagrams: Vec<usize> = Vec::new();
        let mut configs: Vec<Vec<(usize, Vec<u8>)>> = Vec::new();
        for d in config_carrying_diagrams(&set.diagrams) {
            let mut chains: Vec<Vec<u8>> = evals
                .keys()
                .filter(|(diagram, _)| *diagram == d)
                .map(|(_, chain)| chain.clone())
                .collect();
            chains.sort();
            // A diagram the color basis never references contributes nothing to any
            // flow, so it has no amplitude to square and no configuration either.
            if chains.is_empty() {
                continue;
            }
            config_diagrams.push(d);
            configs.push(chains.into_iter().map(|chain| (d, chain)).collect());
        }
        let config_spans: Vec<usize> = configs.iter().map(Vec::len).collect();
        let symbolic = lower::optimize(lower::lower_flows(&basis, &evals, &configs));
        let folded = Folded::build(&symbolic);
        let program = folded.program();
        info!("compiled evaluator: {} ops", program.instrs.len());
        debug!(
            arena_nodes = folded.ast.len(),
            arena_slots = program.arena_sizes.iter().sum::<u32>(),
            amplitudes = evals.len(),
            configurations = configs.len(),
            helicities = helicities.len(),
            "compiled program"
        );

        // Read each flow's basis key back as color lines, giving the Les Houches
        // `(color, anticolor)` labels an event record carries per leg.
        let n_in = set.particles_in.len();
        let leg_colors = ext_particle_ids
            .iter()
            .enumerate()
            .map(|(leg, &pid)| {
                let charge = model.particle(pid).color;
                ColorRep::from_ufo(charge)
                    .map(|rep| LegColor {
                        rep,
                        incoming: leg < n_in,
                    })
                    .ok_or_else(|| {
                        EvalError::TopologyError(format!(
                            "external leg {} has unsupported color charge {charge}",
                            leg + 1
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let color_flow_tags = color_flow_tags(&basis, &leg_colors)?;
        report_flow_tags(&color_flow_tags);
        let leading_color_flows = LeadingColorFlows::of(&basis, n_diagrams);
        let flow_fingerprints: Vec<FlowFingerprint> = basis
            .elements
            .iter()
            .map(|elem| {
                let mut key: FlowFingerprint = elem
                    .contributions
                    .iter()
                    .map(|c| {
                        (
                            c.diagram,
                            c.chain.clone(),
                            c.coeff.nc_power,
                            if c.coeff.q < Ratio::from_integer(0) {
                                -c.coeff.q
                            } else {
                                c.coeff.q
                            },
                        )
                    })
                    .collect();
                key.sort();
                key
            })
            .collect();

        debug!("compiled in {:.3} s", started.elapsed().as_secs_f64());

        Ok(Self {
            folded,
            folded_hel: OnceLock::new(),
            n_ext,
            n_in,
            n_diagrams,
            ext_particle_ids,
            helicities,
            n_flows: basis.ncolor(),
            cf_matrix: basis.cf_matrix,
            leg_colors,
            flow_fingerprints,
            color_flow_tags,
            leading_color_flows,
            config_diagrams,
            config_spans,
            pruned: false,
            zeroamp_nodes_before: 0,
            zeroamp_nodes_after: 0,
        })
    }

    /// The folded whole-amplitude skeleton (arena + pool specs).
    pub(super) fn folded(&self) -> &Folded {
        &self.folded
    }

    /// The helicity-expanded skeleton (see [`Folded::expand_helicities`]), built on
    /// first use and cached.
    pub(super) fn folded_hel(&self) -> &Folded {
        self.folded_hel
            .get_or_init(|| self.folded.expand_helicities(&self.helicities))
    }

    /// Return the number of external legs.
    pub fn n_ext(&self) -> usize {
        self.n_ext
    }

    /// Return the number of incoming external legs.
    pub fn n_in(&self) -> usize {
        self.n_in
    }

    /// Return external particle ids in process order (incoming, then outgoing).
    pub fn external_particles(&self) -> &[ParticleId] {
        &self.ext_particle_ids
    }

    /// Return the number of compiled diagrams.
    pub fn n_diagrams(&self) -> usize {
        self.n_diagrams
    }

    /// Return the valid helicity combinations.
    pub fn helicities(&self) -> &[Vec<i32>] {
        &self.helicities
    }

    /// The helicity combination drawn with probability
    /// `|M_c(p)|² / Σ_d |M_d(p)|²` from a uniform variate `u ∈ [0, 1)`, off the
    /// per-combination diagonal
    /// [`BoundAmplitude::eval_hel_m2`](super::run::BoundAmplitude::eval_hel_m2)
    /// fills; `None` when the weights carry no probability.
    ///
    /// This is MadGraph's `SELECT_HEL`: a categorical draw that fills in an event
    /// record's helicities once a phase-space point has been accepted. It has no
    /// effect on the cross section, which sums over the combinations.
    pub fn select_helicity(&self, hel_m2: &[f64], u: f64) -> Option<&[i32]> {
        // Asserted rather than debug-asserted: a short weight vector would draw from
        // a prefix of the combinations and return a helicity that looks valid, which
        // nothing downstream can detect.
        assert_eq!(
            hel_m2.len(),
            self.helicities.len(),
            "helicity weights must cover the surviving combinations"
        );
        select_index(hel_m2, u).map(|c| self.helicities[c].as_slice())
    }

    /// The colour flow an accepted event is written with, drawn from two uniform
    /// variates — MadEvent's `SELECT_COLOR`.
    ///
    /// `u[0]` draws the integration configuration `∝ AMP2(d)`
    /// ([`BoundAmplitude::eval_amp2`](super::run::BoundAmplitude::eval_amp2)), and
    /// `u[1]` then draws the flow `∝ JAMP2(i)` over the flows *that configuration's
    /// diagram reaches at leading colour* — its `ICOLAMP` row. Both steps are
    /// selections: the cross section sums over configurations and over flows, and
    /// this reads accumulators that decomposition already contains.
    ///
    /// The configuration is drawn rather than taken from the sampler's channel
    /// because MadEvent's is not a sampling label either: under single-diagram
    /// enhancement configuration `j`'s integrand carries `AMP2_j / Σ_i AMP2_i`, so
    /// the configurations of the events a run writes follow that amplitude share
    /// whatever the sampler did. Conditioning on our own sampled channel instead is
    /// measurably not the same thing: for a process whose propagators are all
    /// massless the per-diagram channel maps degenerate onto one another, so the
    /// channel index carries no information about which diagram produced the point.
    ///
    /// A process whose colour basis has one flow reduces to a no-op: every diagram
    /// reaches the single flow, so the mask admits everything and the draw returns
    /// flow 0 for any variate. `None` when no flow carries weight at all.
    pub fn select_color_flow(&self, amp2: &[f64], jamp2: &[f64], u: [f64; 2]) -> Option<usize> {
        // Asserted rather than debug-asserted for the reason `select_helicity`
        // gives: a short weight vector draws from a prefix and returns a label that
        // looks valid.
        assert_eq!(
            amp2.len(),
            self.n_configs(),
            "amp2 weights must cover the integration configurations"
        );
        assert_eq!(
            jamp2.len(),
            self.n_flows,
            "jamp2 weights must cover the color flows"
        );
        match select_index(amp2, u[0]) {
            Some(c) => select_flow_reached_by(
                jamp2,
                self.leading_color_flows.reached_by(self.config_diagrams[c]),
                u[1],
            ),
            // No configuration carries weight here (or the process has none), so
            // there is nothing to condition on and the draw runs over every flow —
            // the same fallback `SELECT_COLOR` takes when its masked cumulant ends
            // at zero.
            None => select_flow(jamp2, u[1]),
        }
    }

    /// Return the number of color flows (NCOLOR).
    pub fn n_flows(&self) -> usize {
        self.n_flows
    }

    /// Return the exact color-factor matrix `CF_{ij}` (row-major,
    /// `cf_matrix[i*n_flows + j]`, evaluated at `Nc = 3`).
    pub fn cf_matrix(&self) -> &[Ratio<i64>] {
        &self.cf_matrix
    }

    /// Return the per-flow Les Houches `(color, anticolor)` line labels, in the
    /// same flow order as the JAMPs and the CF matrix.
    pub fn color_flow_tags(&self) -> &ColorFlowTags {
        &self.color_flow_tags
    }

    /// Return the colour rep and direction of every external leg, in process order.
    ///
    /// These are the legs [`Self::color_flow_tags`] was derived on, so a consumer
    /// carrying that table somewhere else reads the reps it has to compare against
    /// from the compiled amplitude rather than from a PDG table of its own.
    pub fn external_colors(&self) -> &[LegColor] {
        &self.leg_colors
    }

    /// Return each flow's structural fingerprint ([`FlowFingerprint`]), in the same
    /// flow order as the JAMPs and the CF matrix.
    ///
    /// Two subprocesses that share a matrix element carry the same flows in
    /// generally different orders, and this is what pairs them up: the flow of one
    /// that corresponds to flow `f` of the other is the one built from the same
    /// contributions. Matching on it is a statement about the colour algebra rather
    /// than about numbers that happen to agree at a probe point.
    pub fn flow_fingerprints(&self) -> &[FlowFingerprint] {
        &self.flow_fingerprints
    }

    /// Return which colour flows each diagram reaches at leading order in `Nc`
    /// (MadGraph's `ICOLAMP`), in the same diagram order as the compiled
    /// diagrams and the same flow order as the JAMPs.
    pub fn leading_color_flows(&self) -> &LeadingColorFlows {
        &self.leading_color_flows
    }

    /// The diagram behind each integration configuration, in the configuration order
    /// [`BoundAmplitude::eval_amp2`](super::run::BoundAmplitude::eval_amp2) fills and
    /// MadGraph's `ICOLAMP` columns run in. Indexes [`Self::leading_color_flows`], so
    /// `leading_color_flows().reached_by(config_diagrams()[c])` is configuration `c`'s
    /// admitted-flow mask.
    pub fn config_diagrams(&self) -> &[usize] {
        &self.config_diagrams
    }

    /// The number of integration configurations — the length of an `AMP2` vector.
    pub fn n_configs(&self) -> usize {
        self.config_diagrams.len()
    }

    /// How many `(diagram, color chain)` amplitudes each configuration owns —
    /// the grouping MadGraph's `AMP2(k)` accumulator lines carry, and the layout
    /// [`BoundAmplitude::run_config_amps`](super::run::BoundAmplitude::run_config_amps)
    /// returns its values in.
    pub fn config_amp_counts(&self) -> &[usize] {
        &self.config_spans
    }

    /// Whether [`prune_zero_helicities`](Self::prune_zero_helicities) has dropped
    /// any combinations. `eval_m2` on a pruned evaluator only sums the survivors
    /// and is correct only for partonic-CM momenta with beams along ±z (see that
    /// method's doc for why: some survivors are frame-bound zeros, not identities).
    pub(super) fn is_pruned(&self) -> bool {
        self.pruned
    }

    /// MadGraph-style helicity filtering: drop the helicity combinations whose
    /// amplitude is identically zero, so `eval_m2` never evaluates them. Returns
    /// the number of combinations dropped.
    ///
    /// MadGraph filters numerically: its runtime `GOODHEL` loop evaluates every
    /// combination for the first phase-space points and keeps the contributing
    /// ones, and its helicity-recycling codegen bakes the same filter into the
    /// generated source via an init-mode survey (criterion
    /// `DABS(TS(I)) .GT. ANS*LIMHEL/NCOMB`, `LIMHEL = 1e-8`), emitting only the
    /// surviving `NHEL` rows. This method reproduces that filter against this
    /// parameter card: it probes the full helicity expansion on a deterministic
    /// set of generic on-shell partonic-CM points (two energy scales), keeps
    /// every combination over threshold at any point, and re-expands the arena
    /// over the survivors.
    ///
    /// The threshold ([`HEL_PRUNE_REL`]) is far below MadGraph's `LIMHEL`, in the
    /// gap of the strongly bimodal per-combination spectrum: identically-zero
    /// combinations sit at exact `0.0` (chirality-forbidden ones propagate the
    /// structural zeros of the massless-spinor components) or below ~1e-30 of the
    /// helicity sum (MHV-type zeros cancel across diagrams, leaving O(ε²)
    /// residues), while the smallest genuine contributions observed are ≳1e-12
    /// even for doubly mass-suppressed combinations. A combination that
    /// contributes anywhere on the on-shell manifold is (almost surely, over
    /// random probe momenta) over threshold at every probe point. Because every
    /// dropped term is ≲1e-30 of the sum — far below half an ulp of any partial
    /// sum it enters — the pruned helicity sum is bit-for-bit the unpruned one.
    ///
    /// A pruned evaluator adopts MadGraph's kinematic contract: `eval_m2` momenta
    /// must be **partonic-CM kinematics with the beams along ±z** — the frame
    /// madevent, the VEGAS driver, and the validation samples all evaluate in.
    /// Some pruned combinations (e.g. same-helicity gluons with opposite-helicity
    /// massive quarks in `g g > t t~`) vanish by J_z conservation about the beam
    /// axis in that frame rather than identically: massive-particle helicity is
    /// not boost invariant (even under z-boosts), so those combinations contribute
    /// in any other frame and the pruned helicity sum would come out low there.
    /// The probe set is therefore pure-CM, matching MadGraph's survey.
    ///
    /// Filtering is skipped (returning 0) when `n_ext ≤ 3` (MadGraph disables the
    /// filter there too — near-degenerate 2→1 kinematics), when the process is not
    /// 2→n, and when no combination survives (a degenerate card zeroing the whole
    /// amplitude should stay visible rather than be pruned away).
    pub fn prune_zero_helicities(&mut self, evaluated: &EvaluatedModel) -> usize {
        if self.n_ext <= 3 || self.n_in != 2 {
            return 0;
        }
        let points = self.generic_probe_points(evaluated);

        let mut good = vec![false; self.helicities.len()];
        {
            let bound = BoundAmplitude::<f64>::bind(self, evaluated);
            let mut scratch = bound.scratch_space();
            for p in &points {
                bound.mark_contributing_helicities(p, HEL_PRUNE_REL, &mut scratch, &mut good);
            }
        }

        let n_good = good.iter().filter(|&&g| g).count();
        if n_good == 0 || n_good == self.helicities.len() {
            return 0;
        }
        let dropped = self.helicities.len() - n_good;
        self.helicities = self
            .helicities
            .iter()
            .zip(&good)
            .filter(|(_, &g)| g)
            .map(|(h, _)| h.clone())
            .collect();
        self.folded_hel = OnceLock::new();
        self.pruned = true;

        // Second helicity-filter layer: within the surviving combinations, reclaim the
        // per-diagram amplitudes that are still identically zero (MadGraph's `ZEROAMP`).
        self.prune_zero_amplitudes(evaluated, &points);
        dropped
    }

    /// A deterministic set of generic on-shell partonic-CM probe points: two incoming
    /// legs along ±z at two energy scales, the outgoing legs from seeded massive RAMBO.
    /// Two scales guard against a kinematic coincidence at one energy; the non-round
    /// multipliers avoid special mass ratios. Shared by both helicity-filter layers so
    /// they probe identical kinematics.
    fn generic_probe_points(&self, evaluated: &EvaluatedModel) -> Vec<Vec<LorentzVector<f64>>> {
        let masses: Vec<f64> = self
            .ext_particle_ids
            .iter()
            .map(|&pid| evaluated.mass(pid))
            .collect();
        let (m_in, m_out) = masses.split_at(self.n_in);
        let mut rng = StdRng::seed_from_u64(0x600D_4E15);
        let threshold = (m_in.iter().sum::<f64>())
            .max(m_out.iter().sum::<f64>())
            .max(1.0);
        let mut points = Vec::with_capacity(10);
        for scale in [3.7, 11.3] {
            let sqrt_s = scale * threshold;
            let s = sqrt_s * sqrt_s;
            let e1 = (s + m_in[0] * m_in[0] - m_in[1] * m_in[1]) / (2.0 * sqrt_s);
            let pz = (e1 * e1 - m_in[0] * m_in[0]).max(0.0).sqrt();
            for _ in 0..5 {
                let mut p = vec![
                    LorentzVector::new(e1, 0.0, 0.0, pz),
                    LorentzVector::new(sqrt_s - e1, 0.0, 0.0, -pz),
                ];
                p.extend(rambo_massive(sqrt_s, m_out, &mut rng));
                points.push(p);
            }
        }
        points
    }

    /// Reclaim the identically-zero per-diagram amplitude contributions inside the
    /// surviving helicity combinations (see
    /// [`Folded::prune_zero_scalar_operands`](super::fold::Folded::prune_zero_scalar_operands)),
    /// replacing the helicity-expanded arena with the dead-code-eliminated one. The
    /// removal is byte-for-byte with the full expansion (only structural zeros drop),
    /// so `eval_m2` is unchanged. Only meaningful after the combination filter has run
    /// (`pruned`), under whose partonic-CM contract the probe points sit.
    fn prune_zero_amplitudes(
        &mut self,
        evaluated: &EvaluatedModel,
        points: &[Vec<LorentzVector<f64>>],
    ) {
        let expanded = self.folded.expand_helicities(&self.helicities);
        let (consts_c, consts_f) = expanded.pools::<f64>(evaluated);
        let (pruned, before, after) =
            expanded.prune_zero_scalar_operands(&consts_c, &consts_f, points);
        self.zeroamp_nodes_before = before;
        self.zeroamp_nodes_after = after;
        self.folded_hel = OnceLock::new();
        let _ = self.folded_hel.set(pruned);
    }

    /// Helicity-expanded arena node counts `(before, after)` the zero-amplitude
    /// elimination pass, or `(0, 0)` if [`prune_zero_helicities`](Self::prune_zero_helicities)
    /// has not run. A diagnostic for the per-`(helicity, diagram)` skipping headroom.
    pub fn zeroamp_node_reduction(&self) -> (usize, usize) {
        (self.zeroamp_nodes_before, self.zeroamp_nodes_after)
    }

    /// Return all coupling and particle ids needed to evaluate the amplitude.
    ///
    /// Can be used for prefetching from EvaluatedModel if desired.
    pub fn coupling_particle_ids(&self) -> (HashSet<CouplingId>, HashSet<ParticleId>) {
        (
            self.folded.coupling_ids().collect(),
            self.folded.particle_ids().collect(),
        )
    }
}

/// The colour-factor matrix, one row per line.
///
/// The entries are exact rationals at `Nc = 3`, so this is the whole colour
/// algebra of a process in a form another implementation's matrix can be compared
/// against term by term. A wide process's matrix is quadratic in the flow count,
/// which is why it is rendered only when someone asked for that much detail.
fn report_cf_matrix(n_flows: usize, cf_matrix: &[Ratio<i64>]) {
    if !tracing::enabled!(tracing::Level::TRACE) {
        return;
    }
    for i in 0..n_flows {
        let row: Vec<String> = cf_matrix[i * n_flows..(i + 1) * n_flows]
            .iter()
            .map(|entry| entry.to_string())
            .collect();
        trace!("CF row {i}: {}", row.join(" "));
    }
}

/// Each flow's Les Houches `(colour, anticolour)` label per external leg.
fn report_flow_tags(tags: &ColorFlowTags) {
    if !tracing::enabled!(tracing::Level::DEBUG) {
        return;
    }
    for f in 0..tags.n_flows() {
        let legs: Vec<String> = tags
            .flow(f)
            .iter()
            .map(|[c, a]| format!("({c},{a})"))
            .collect();
        debug!("flow {f} legs: {}", legs.join(" "));
    }
}

/// The processes these library-level sweeps compile, all of them gated against
/// MadGraph by `tests/amplitude_oracle.rs`.
///
/// It is every concrete subprocess that gate covers, the four `p p > l+ l- j`
/// rows included, so a sweep driven by this list reaches a coloured `2 -> 3`
/// amplitude and not only colourless ones and `2 -> 2`s. A process added to the
/// amplitude gate belongs here too; the cost is another full re-rooting sweep to
/// re-verify.
#[cfg(test)]
pub(super) const MG_VALIDATED_PROCESSES: [&str; 19] = [
    "e+ e- > mu+ mu-",
    "u u~ > mu+ mu-",
    "e+ e- > e+ e-",
    "e+ e- > mu+ mu- a",
    "e+ e- > t t~",
    "e+ e- > W+ W-",
    "e+ e- > Z H",
    "e+ e- > ta+ ta- H",
    "e+ e- > mu+ mu- ta+ ta- QCD=0",
    "u u~ > c c~ e+ e- mu+ mu- QCD=0",
    "b b~ > c c~ e+ e- mu+ mu- QCD=0",
    "u u~ > u u~",
    "g g > t t~",
    "g g > g g",
    "u u~ > e+ e- g QCD=2 QED=2",
    "d d~ > e+ e- g QCD=2 QED=2",
    "g u > e+ e- u QCD=2 QED=2",
    "g u~ > e+ e- u~ QCD=2 QED=2",
    "u d > e+ e- u d QCD=0",
];

/// The diagrams MadGraph gives an integration configuration — and therefore an
/// `AMP2` accumulator and an `ICOLAMP` column — as indices into `diagrams`.
///
/// The rule is `get_amp2_lines` (`madgraph/iolibs/export_v4.py`): over the diagrams
/// that have vertices at all, take the smallest of their largest vertex arities, and
/// drop every diagram whose largest vertex exceeds it. In practice that keeps the
/// diagrams built from three-point vertices only and drops the four-point contact
/// ones — `g g > g g`'s four-gluon diagram gets no `AMP2` and no configuration, so
/// nothing can be drawn to it and its colour structures never mask a flow.
///
/// A contact diagram still contributes to `|M|²` and to every JAMP it reaches; what
/// it does not get is a *channel*, because it has no propagator to enhance.
fn config_carrying_diagrams(diagrams: &[Diagram]) -> Vec<usize> {
    let widest = |d: &Diagram| d.vertices.iter().map(|v| v.rays.len()).max();
    let minvert = diagrams.iter().filter_map(widest).min();
    (0..diagrams.len())
        .filter(|&i| match (widest(&diagrams[i]), minvert) {
            (Some(w), Some(m)) => w <= m,
            _ => true,
        })
        .collect()
}

/// Helicity-filter threshold: a combination whose CF-contracted |M_c|² stays below
/// `Σ_c |M_c|² · HEL_PRUNE_REL / NCOMB` at every probe point is dropped (MadGraph's
/// `LIMHEL` criterion, tightened from its 1e-8 into the bimodal gap between
/// cancellation residues (≲1e-30 of the sum) and the smallest genuine
/// contributions (≳1e-12), so pruning provably cannot touch a contributing
/// combination and the pruned sum stays bit-for-bit; see
/// [`AmplitudeEvaluator::prune_zero_helicities`]).
const HEL_PRUNE_REL: f64 = 1e-24;

fn helicity_states_for_spin(spin_code: i32, massless: bool) -> Result<Vec<i32>, EvalError> {
    // UFO spin code convention is 2s+1 with negative values reserved for ghosts.
    // A massless vector has no longitudinal mode (and `vxxxxx`'s massless branch
    // only defines helicities ±1), so 0 is dropped from its state list.
    match (spin_code.abs(), massless) {
        (1, _) => Ok(vec![0]),               // scalar
        (2, _) => Ok(vec![-1, 1]),           // fermion
        (3, false) => Ok(vec![-1, 0, 1]),    // massive vector
        (3, true) => Ok(vec![-1, 1]),        // massless vector
        (5, _) => Ok(vec![-2, -1, 0, 1, 2]), // spin-2 (future-proof)
        (other, _) => Err(EvalError::UnsupportedSpin(other)),
    }
}

fn cartesian_helicity_product(states: &[Vec<i32>]) -> Vec<Vec<i32>> {
    let mut out = vec![Vec::new()];
    for leg_states in states {
        let mut next = Vec::with_capacity(out.len() * leg_states.len());
        for partial in &out {
            for &h in leg_states {
                let mut combo = partial.clone();
                combo.push(h);
                next.push(combo);
            }
        }
        out = next;
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::lower;
    use super::super::root_diagram::compile_diagram_ast;
    use super::{AmplitudeEvaluator, MG_VALIDATED_PROCESSES};
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::eval::op::Op;
    use crate::helas::eval::tree::Tree;
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// Ops absent from the *compiled* (`folded().ast`) arenas this test scans.
    /// `IdentityAmp` needs a UFO model with an `Identity` scalar bilinear; the SM has
    /// none (its Yukawas are `ProjM + ProjP`). Its kernel is pinned algebraically
    /// against MG-covered ops in `kernel::tests`; process-level coverage remains a
    /// future item. `Hels` is never emitted at compile time at all — it is the root
    /// the helicity expansion (`Folded::expand_helicities`) derives from every one of
    /// these arenas, and `eval_m2` reads it on every MG-gated |M|² comparison, so it
    /// is exercised by the same net through a different door. `Flows` and `CoeffRat`
    /// are only emitted for processes whose color basis has more than one flow
    /// (multi-flow color algebra); `uux_to_uux` (`NCOLOR=2`), `gg_to_ttx` (`NCOLOR=2`)
    /// and `gg_to_gg` (`NCOLOR=6`) now bit-validate both.
    const KNOWN_UNCOVERED: [Op; 2] = [Op::Hels, Op::IdentityAmp];

    /// Every `Op` outside [`KNOWN_UNCOVERED`] appears in the compiled AST of at least
    /// one MG-validated process — the bit-for-bit `amplitude_oracle` net exercises the
    /// whole primitive set. Two-way: an op newly covered by the suite must be removed
    /// from the allowlist.
    #[test]
    fn mg_validated_suite_exercises_every_op() {
        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            let mut per_process: BTreeMap<&'static str, usize> = BTreeMap::new();
            for set in &sets {
                let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
                let ast = &eval.folded().ast;
                for id in ast.iter() {
                    *per_process.entry(ast.value(id).op.name()).or_insert(0) += 1;
                }
            }
            println!("[{process}] {per_process:?}");
            for (name, n) in per_process {
                *counts.entry(name).or_insert(0) += n;
            }
        }
        let missing: Vec<&str> = <Op as strum::VariantArray>::VARIANTS
            .iter()
            .map(|op| op.name())
            .filter(|name| !counts.contains_key(name))
            .collect();
        let expected_missing: Vec<&str> = KNOWN_UNCOVERED.iter().map(|op| op.name()).collect();
        assert_eq!(
            missing, expected_missing,
            "MG-validated op coverage changed (left: actually missing, right: KNOWN_UNCOVERED)\nop counts: {counts:#?}"
        );
    }

    /// Every `Add`/`Mul` node in the symbolic [`lower`](crate::helas::eval::lower::lower)
    /// output has exactly two children — the static-arity form an egg rewrite stage
    /// requires. Checked across the full MG-validated suite. (`optimize` then
    /// re-n-aryfies the sums for evaluation, so the folded arena is intentionally
    /// *not* binary.)
    #[test]
    fn lowered_add_mul_are_binary() {
        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let diagrams = compile_diagram_ast(set, &model).unwrap();
                let ast = lower::lower(&diagrams);
                for id in ast.iter() {
                    let op = ast.value(id).op;
                    if matches!(op, Op::Add | Op::Mul) {
                        assert_eq!(
                            ast.children_ids(id).len(),
                            2,
                            "[{process}] {op:?} node {id} is not binary"
                        );
                    }
                }
            }
        }
    }

    /// The zero-amplitude elimination pass is bit-for-bit: on colored processes that
    /// carry per-diagram structural zeros inside their surviving helicity combinations,
    /// the pruned evaluator's helicity-summed |M|² equals the unpruned one to the byte
    /// at generic partonic-CM points, and the pass actually reclaims arena nodes.
    #[test]
    fn zeroamp_pass_is_bit_exact_and_fires() {
        use crate::helas::eval::BoundAmplitude;
        use crate::helas::LorentzVector;
        use crate::phasespace::rambo_massless;
        use crate::ufo::EvaluatedModel;
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x2E20_A11F);
        let sqrt_s = 500.0;

        // Colored 2→2s carry ZEROAMP contributions within surviving combinations; the
        // color-singlet 2→3 exercises a single-flow amplitude sum.
        let processes = ["u u~ > u u~", "g g > g g", "e+ e- > mu+ mu- a"];
        let mut any_fired = false;
        for process in processes {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");

            let unpruned = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            let bound = BoundAmplitude::<f64>::bind(&unpruned, &evaluated);
            let mut scratch = bound.scratch_space();

            let mut pruned = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            pruned.prune_zero_helicities(&evaluated);
            let bound_pruned = BoundAmplitude::<f64>::bind(&pruned, &evaluated);
            let mut scratch_pruned = bound_pruned.scratch_space();

            let (before, after) = pruned.zeroamp_node_reduction();
            assert!(
                after <= before,
                "[{process}] node count grew: {before} -> {after}"
            );
            any_fired |= after < before;

            for _ in 0..32 {
                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massless(sqrt_s, unpruned.n_ext() - 2, &mut rng));
                let m2 = bound.eval_m2(&p, &mut scratch);
                let m2_pruned = bound_pruned.eval_m2(&p, &mut scratch_pruned);
                assert_eq!(
                    m2.to_bits(),
                    m2_pruned.to_bits(),
                    "[{process}] zeroamp pruning changed |M|²: {m2:e} vs {m2_pruned:e}"
                );
            }
        }
        assert!(
            any_fired,
            "zero-amplitude pass reclaimed no nodes on any probed process — it is inert"
        );
    }

    /// The four-gluon contact diagram carries no integration configuration, and
    /// every other diagram carries exactly one — MadGraph's `get_amp2_lines` rule,
    /// on the process it is visible in. `u u~ > u u~` is the control: no contact
    /// diagram, so both of its diagrams are configurations.
    #[test]
    fn the_four_point_contact_diagram_carries_no_configuration() {
        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for (process, n_diagrams, configs) in [
            ("u u~ > u u~", 2, vec![0, 1]),
            ("g g > t t~", 3, vec![0, 1, 2]),
            // Diagram 0 is the four-gluon vertex; the s/t/u gluon exchanges follow.
            ("g g > g g", 4, vec![1, 2, 3]),
        ] {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            assert_eq!(eval.n_diagrams(), n_diagrams, "[{process}] diagram count");
            assert_eq!(
                eval.config_diagrams(),
                configs,
                "[{process}] configuration-carrying diagrams"
            );
            // The contact diagram's three colour structures are three amplitudes; a
            // configuration diagram has one apiece.
            assert!(
                eval.config_amp_counts().iter().all(|&n| n == 1),
                "[{process}] configuration amplitude counts {:?}",
                eval.config_amp_counts()
            );
        }
    }

    /// The colour draw is conditioned on the configuration, not merely masked by
    /// something: on `u u~ > u u~` each configuration's `ICOLAMP` row admits exactly
    /// one flow — the *other* one — so the drawn flow is fixed by the configuration
    /// alone, and swapping the two configurations swaps the flow. A draw that
    /// ignored the configuration, or that read the rows off by one, cannot
    /// reproduce both rows.
    #[test]
    fn the_flow_is_decided_by_the_drawn_configuration() {
        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        let pc = parse_proc_card("generate u u~ > u u~", &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();

        // Lopsided weights so the configuration draw is decided by u[0] alone, and
        // JAMP2 weights that would send an unconditioned draw to flow 0 nine times
        // in ten.
        let jamp2 = [9.0, 1.0];
        for (amp2, u0, want) in [
            ([1.0, 0.0], 0.5, 1),
            ([0.0, 1.0], 0.5, 0),
            ([1.0, 1.0], 0.25, 1),
            ([1.0, 1.0], 0.75, 0),
        ] {
            for u1 in [0.0, 0.5, 0.99] {
                assert_eq!(
                    eval.select_color_flow(&amp2, &jamp2, [u0, u1]),
                    Some(want),
                    "amp2 {amp2:?} at u = [{u0}, {u1}]"
                );
            }
        }
    }

    /// A colourless process reduces the rule to a no-op: one flow, one all-admitting
    /// `ICOLAMP` row, so every configuration draw lands on flow 0 and Drell-Yan
    /// events are labelled exactly as they were before the rule existed.
    #[test]
    fn a_single_flow_process_always_selects_its_only_flow() {
        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        let pc = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        assert_eq!(eval.n_flows(), 1);
        assert_eq!(eval.n_configs(), 2, "both diagrams carry a configuration");
        for u0 in [0.0, 0.3, 0.999] {
            for u1 in [0.0, 0.3, 0.999] {
                assert_eq!(
                    eval.select_color_flow(&[1.0, 3.0], &[7.0], [u0, u1]),
                    Some(0)
                );
            }
        }
        // Even with no configuration carrying weight, the event still gets its flow.
        assert_eq!(
            eval.select_color_flow(&[0.0, 0.0], &[7.0], [0.5, 0.5]),
            Some(0)
        );
    }

    /// Diagnostic (run with `--ignored --nocapture`): per-process helicity-expanded node
    /// count before and after the zero-amplitude elimination pass.
    #[test]
    #[ignore]
    fn zeroamp_node_reduction_table() {
        use crate::ufo::EvaluatedModel;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        println!(
            "{:<34} {:>10} {:>10} {:>8}",
            "process", "before", "after", "drop%"
        );
        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            eval.prune_zero_helicities(&evaluated);
            let (before, after) = eval.zeroamp_node_reduction();
            let pct = if before > 0 {
                100.0 * (before - after) as f64 / before as f64
            } else {
                0.0
            };
            println!("{process:<34} {before:>10} {after:>10} {pct:>7.2}%");
        }
    }
}
