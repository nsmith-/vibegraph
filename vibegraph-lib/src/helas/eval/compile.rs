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

use crate::diagrams::DiagramSet;
use crate::helas::color::colorize_process;
use crate::helas::repr::lorentz::LorentzVector;
use crate::phasespace::rambo_massive;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::{EvaluatedModel, UFOModel};

use super::error::EvalError;
use super::fold::Folded;
use super::lower;
use super::root_diagram::{compile_single_diagram, DiagramEval};
use super::run::BoundAmplitude;

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
        // `NCOLOR = 1`), then intern the constants into the folded skeleton.
        let n_diagrams = set.diagrams.len();
        let symbolic = lower::optimize(lower::lower_flows(&basis, &evals));
        let folded = Folded::build(&symbolic);

        Ok(Self {
            folded,
            folded_hel: OnceLock::new(),
            n_ext,
            n_in: set.particles_in.len(),
            n_diagrams,
            ext_particle_ids,
            helicities,
            n_flows: basis.ncolor(),
            cf_matrix: basis.cf_matrix,
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

    /// Return the number of color flows (NCOLOR).
    pub fn n_flows(&self) -> usize {
        self.n_flows
    }

    /// Return the exact color-factor matrix `CF_{ij}` (row-major,
    /// `cf_matrix[i*n_flows + j]`, evaluated at `Nc = 3`).
    pub fn cf_matrix(&self) -> &[Ratio<i64>] {
        &self.cf_matrix
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

/// The `validate_helas_mg` process suite (`EXPECT_MATCH` in
/// `tests/validate_helas_mg.rs`: bit-for-bit for the 11 `NCOLOR=1` processes,
/// `REL_TOL`-enforced via the CF-weighted multi-flow contraction for
/// `uux_to_uux`, `gg_to_ttx`, and `gg_to_gg`). Keep in sync with the `PROCESSES`
/// registry in `validation/madgraph/gen_amplitude.py`.
#[cfg(test)]
pub(super) const MG_VALIDATED_PROCESSES: [&str; 14] = [
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
];

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
    /// one MG-validated process — the bit-for-bit `validate_helas_mg` net exercises the
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
