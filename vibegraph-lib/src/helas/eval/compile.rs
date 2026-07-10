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

use std::collections::HashSet;

use crate::diagrams::DiagramSet;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;
use crate::ufo::UFOModel;

use super::error::EvalError;
use super::fold::Folded;
use super::lower;
use super::root_diagram::compile_diagram_ast;

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

        let diagrams = compile_diagram_ast(set, model)?;
        let n_ext = ext_particle_ids.len();

        // Compile phase should preserve process external-leg count consistency.
        if let Some(ast) = diagrams.first() {
            if ast.n_ext != n_ext {
                return Err(EvalError::TopologyError(format!(
                    "process has {n_ext}, AST has {}",
                    ast.n_ext
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

        // Pass 3: inline every diagram into one whole-amplitude AST (structure pass is
        // a no-op for now), then intern the constants into the folded skeleton.
        let n_diagrams = diagrams.len();
        let symbolic = lower::optimize(lower::lower(&diagrams));
        let folded = Folded::build(&symbolic);

        Ok(Self {
            folded,
            n_ext,
            n_in: set.particles_in.len(),
            n_diagrams,
            ext_particle_ids,
            helicities,
        })
    }

    /// The folded whole-amplitude skeleton (arena + pool specs).
    pub(super) fn folded(&self) -> &Folded {
        &self.folded
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

    use super::AmplitudeEvaluator;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::helas::eval::op::Op;
    use crate::helas::eval::tree::Tree;
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// The `validate_helas_mg` process suite (bit-for-bit MG-validated). Keep in sync
    /// with the `PROCESSES` registry in `validation/madgraph/gen_amplitude.py`.
    const MG_VALIDATED_PROCESSES: [&str; 11] = [
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
    ];

    /// Ops with no MG-validated process coverage. `MetricNegI` needs a diagram whose
    /// *amplitude* contraction is a pure-metric (VVS) vertex — the root-at-`VtxIdx(0)`
    /// policy never places one there in this suite, which roots every such diagram on a
    /// fermion line instead. `IdentityAmp` needs a UFO model with an `Identity` scalar
    /// bilinear; the SM has none (its Yukawas are `ProjM + ProjP`). Both kernels are
    /// pinned algebraically against MG-covered ops in `kernel::tests`; process-level
    /// coverage is a validation-sprint item (TODO.md).
    const KNOWN_UNCOVERED: [Op; 2] = [Op::MetricNegI, Op::IdentityAmp];

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
        let missing: Vec<&str> = Op::ALL
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
}
