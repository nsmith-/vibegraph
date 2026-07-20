//! Runtime amplitude evaluation: a single forward pass over the folded `Ast`.
//!
//! [`BoundAmplitude`] holds a compiled [`AmplitudeEvaluator`] together with its
//! card-resolved constant pools (see [`BoundAmplitude::bind`]). For each
//! phase-space point it walks the arena in storage (topological) order, reducing each
//! node from its already-computed children via the single [`apply`] match.

use crate::helas::repr::lorentz::{Bispinor, Bra, ComplexVector, Ket, LorentzVector};
use crate::helas::repr::numbers::{Charge, SpinorHelicity};
use crate::helas::repr::{Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};
use num_traits::{FromPrimitive, Zero};

use numeric_array::generic_array::typenum::Const as LaneLen;
use numeric_array::generic_array::IntoArrayLength;
use numeric_array::NumericArray;

use super::analysis::NodeAnalysis;
use super::compile::AmplitudeEvaluator;
use super::fold::{ExtLeg, Folded};
use super::kernel;
use super::lanes::{transpose_points, unpack, LaneField};
use super::layout::{Instr, OperandRef, RootKind, N_ARENAS};
use super::op::{Const, ConstKind, Node, NodeId, Op};
#[cfg(test)]
use super::tree::Tree;
use super::waveform_slot::WaveformSlot;
use crate::ufo::EvaluatedModel;

#[cfg(test)]
use super::ast::Ast;
#[cfg(test)]
use super::lower;
#[cfg(test)]
use super::root_diagram::{compile_diagram_ast, DiagramEval};

/// Reusable evaluation workspace for a [`BoundAmplitude`].
///
/// Owns the evaluator's heap storage so the hot loops (helicity combinations
/// inside `eval_m2`, phase-space points outside it) share one allocation. Create
/// with the amplitude's `scratch_space()`; any `ScratchSpace` of matching `F` works
/// with any bound amplitude (it grows on demand). Opaque so the evaluation strategy
/// can change without touching the API.
#[derive(Debug, Default)]
pub struct ScratchSpace<F: Real> {
    /// One result arena per output class (real / scalar / vector / flow-in / flow-out
    /// fermion). Each node writes its result to the arena its statically-known class
    /// selects, and consumers read operands directly from the correct arena — no
    /// per-value enum tag. The elements are momentum-stripped (a bare `C`/`ComplexVector`/
    /// `Bispinor`): a current's routing momentum lives in `moms` instead, keyed by the
    /// node's momentum id. Filled in storage (topological) order; every slot is written
    /// before it is read within a pass, so arenas are never cleared between passes.
    reals: Vec<F>,
    scalars: Vec<C<F>>,
    vectors: Vec<ComplexVector<F>>,
    fin: Vec<Bispinor<F, Ket>>,
    fout: Vec<Bispinor<F, Bra>>,
    /// Per-point momentum pool: `moms[id]` is the external-momentum combination the
    /// momentum table interns at `id` (`Σ ± p_leg`), resolved once per phase-space point.
    moms: Vec<LorentzVector<F>>,
}

impl<F: Real> ScratchSpace<F> {
    /// Grow every result arena to at least the program's slot count, so each node
    /// writes its result by direct index (`arena[loc]`). Stale values from a previous
    /// pass are never read — within a pass, every slot is written by the node the
    /// allocator assigned it to before any consumer reads it — so nothing is cleared.
    /// Leaves `moms` untouched — it is resolved per point by [`resolve_moms`].
    #[inline]
    fn ensure_sizes(&mut self, sizes: &[u32; N_ARENAS]) {
        if self.reals.len() < sizes[0] as usize {
            self.reals.resize(sizes[0] as usize, F::zero());
        }
        if self.scalars.len() < sizes[1] as usize {
            self.scalars.resize(sizes[1] as usize, C::zero());
        }
        if self.vectors.len() < sizes[2] as usize {
            self.vectors
                .resize(sizes[2] as usize, ComplexVector::zero());
        }
        if self.fin.len() < sizes[3] as usize {
            self.fin.resize(sizes[3] as usize, Bispinor::zero());
        }
        if self.fout.len() < sizes[4] as usize {
            self.fout.resize(sizes[4] as usize, Bispinor::zero());
        }
    }
}

/// The per-evaluation immutable context every node reduction reads: the bound
/// constant pools and leg table, plus one phase-space point's kinematics.
pub(super) struct EvalEnv<'a, F: Real> {
    pub consts_c: &'a [C<F>],
    pub consts_f: &'a [F],
    pub ext_legs: &'a [ExtLeg],
    pub momenta: &'a [LorentzVector<F>],
    pub helicities: &'a [i32],
    pub ward_leg: Option<usize>,
}

/// A compiled amplitude bound to a parameter card at scalar precision `F`.
///
/// Created by [`BoundAmplitude::bind`]: it borrows the card-independent
/// [`AmplitudeEvaluator`] and owns the resolved constant pools (`consts_c` couplings,
/// `consts_f` masses/widths/coeffs), so evaluation is pure kinematics — no parameter
/// lookups on the hot path.
#[derive(Debug)]
pub struct BoundAmplitude<'a, F: Real> {
    eval: &'a AmplitudeEvaluator,
    consts_c: Box<[C<F>]>,
    consts_f: Box<[F]>,
    /// The exact color-factor matrix resolved to `F` (row-major,
    /// `cf[i*n_flows + j] = CF_{ij}`).
    cf: Box<[F]>,
}

impl<'a, F: Real + FromPrimitive> BoundAmplitude<'a, F> {
    /// Resolve a parameter card at scalar precision `F` against a compiled
    /// [`AmplitudeEvaluator`], baking all couplings/masses/widths into the constant
    /// pools. The same evaluator binds against any card and precision.
    pub fn bind(eval: &'a AmplitudeEvaluator, evaluated: &EvaluatedModel) -> Self {
        let (consts_c, consts_f) = eval.folded().pools::<F>(evaluated);
        // Resolve the exact rational CF matrix to `F` (each num/den converted before
        // the division, mirroring the constant-pool rationals).
        let cf: Box<[F]> = eval
            .cf_matrix()
            .iter()
            .map(|r| {
                F::from_i64(*r.numer()).expect("CF numerator convertible to scalar")
                    / F::from_i64(*r.denom()).expect("CF denominator convertible to scalar")
            })
            .collect();
        BoundAmplitude::new(eval, consts_c, consts_f, cf)
    }
}

impl<'a, F: Real> BoundAmplitude<'a, F> {
    /// Build from a compiled evaluator and its card-resolved pools (see [`bind`]).
    ///
    /// [`bind`]: BoundAmplitude::bind
    pub(super) fn new(
        eval: &'a AmplitudeEvaluator,
        consts_c: Box<[C<F>]>,
        consts_f: Box<[F]>,
        cf: Box<[F]>,
    ) -> Self {
        BoundAmplitude {
            eval,
            consts_c,
            consts_f,
            cf,
        }
    }

    /// The compiled (card-independent) evaluator this amplitude is bound to.
    pub fn evaluator(&self) -> &'a AmplitudeEvaluator {
        self.eval
    }

    /// A workspace sized for this amplitude. Create once and pass to every
    /// `eval_*` call; reuse across points to keep the hot path allocation-free.
    pub fn scratch_space(&self) -> ScratchSpace<F> {
        let sizes = self.eval.folded().program().arena_sizes;
        ScratchSpace {
            reals: Vec::with_capacity(sizes[0] as usize),
            scalars: Vec::with_capacity(sizes[1] as usize),
            vectors: Vec::with_capacity(sizes[2] as usize),
            fin: Vec::with_capacity(sizes[3] as usize),
            fout: Vec::with_capacity(sizes[4] as usize),
            moms: Vec::with_capacity(self.eval.folded().analysis().mom_table().len()),
        }
    }

    /// Evaluate the color- and helicity-summed |M|² (MadGraph's MATRIX1 convention,
    /// not averaged).
    ///
    /// `momenta` are the external 4-momenta `[E, px, py, pz]`, incoming legs first then
    /// outgoing.
    ///
    /// Runs the helicity-expanded program (see `Folded::expand_helicities`): every
    /// combination's currents live in one arena, hash-consed so a current shared
    /// between combinations is computed exactly once per phase-space point, and one
    /// linear pass fills them all. The per-combination amplitudes are then read off
    /// the [`RootKind::Hels`] root scalars in combination order.
    ///
    /// For a single color flow (`NCOLOR = 1`) each combination's root is a scalar `M`,
    /// and the constant color factor `CF(1,1)` multiplies the helicity sum *after* it
    /// is formed (`CF · Σ_hel |M|²`) — preserving the exact floating-point operation
    /// order of a color-free evaluation, since `Σ (CF·x_h) ≠ CF·Σ x_h` bitwise. For
    /// several flows the per-combination JAMPs `J_i` are contracted through the CF
    /// matrix in MadGraph's ZTEMP order, `Σ_i (Σ_j CF_{ji} J_j) · J_i*`, and summed
    /// over combinations. The CF weights scale each JAMP as a real factor, matching
    /// MADGRAPH's real-matrix × complex-JAMP product.
    ///
    /// On a helicity-filtered evaluator (see
    /// [`AmplitudeEvaluator::prune_zero_helicities`]) the sum runs over the
    /// surviving combinations only — bit-for-bit with the full sum, but under that
    /// method's kinematic contract: partonic-CM momenta with the beams along ±z.
    pub fn eval_m2(&self, momenta: &[LorentzVector<F>], scratch: &mut ScratchSpace<F>) -> F {
        if momenta.len() != self.eval.n_ext() {
            return F::zero();
        }
        if self.eval.is_pruned() {
            assert_partonic_cm_beams_along_z(momenta, self.eval.n_in());
        }
        let folded = self.eval.folded_hel();
        resolve_moms(folded, momenta, scratch);
        // Baked-helicity externals never read the per-evaluation helicity assignment.
        let env = self.eval_env(folded, momenta, &[], None);
        fill_arenas(folded, &env, scratch);

        let RootKind::Hels { n_flows, locs } = &folded.program().root else {
            panic!("eval_m2 on a program without a helicity-expanded root");
        };
        let n = *n_flows as usize;
        debug_assert_eq!(n, self.eval.n_flows());

        if n == 1 {
            let mut hel_sum = F::zero();
            for &l in locs.iter() {
                hel_sum = hel_sum + scratch.scalars[l as usize].norm_sqr();
            }
            return hel_sum * self.cf[0];
        }

        // MATRIX1 += Σ_i ( Σ_j CF_{ji}·J_j ) · conj(J_i); the imaginary parts cancel
        // over the i-sum (MADGRAPH assigns the complex product to a real accumulator),
        // so only the real part contributes.
        let mut total = F::zero();
        for jamps in locs.chunks_exact(n) {
            for i in 0..n {
                let mut ztemp = C::new(F::zero(), F::zero());
                for (j, &lj) in jamps.iter().enumerate() {
                    ztemp = ztemp + scratch.scalars[lj as usize].scale(self.cf[j * n + i]);
                }
                total = total + (ztemp * scratch.scalars[jamps[i] as usize].conj()).re;
            }
        }
        total
    }

    /// Mark the helicity combinations that contribute at this phase-space point:
    /// sets `good[c] = true` for every combination whose |M_c|² (CF-contracted, as
    /// `eval_m2` forms it) exceeds `Σ_c |M_c|² · rel_threshold / NCOMB`; other
    /// entries are left untouched, so repeated calls accumulate across probe points.
    ///
    /// This is MadGraph's helicity-filter criterion (`DABS(TS(I)) .GT.
    /// ANS*LIMHEL/NCOMB` in its init-mode survey). A relative threshold — rather
    /// than an exact-zero test — is needed because identically-zero combinations
    /// come in two kinds: chirality-forbidden ones propagate the *structural* zeros
    /// of the massless-spinor components and evaluate to exact `0.0`, but
    /// MHV-type zeros (e.g. all-plus gluons) cancel *across* diagrams and leave
    /// O(ε²) floating-point residues in |M_c|².
    /// Backs [`AmplitudeEvaluator::prune_zero_helicities`], which chooses the
    /// threshold. Marks nothing when the point's total is zero or non-finite.
    pub(super) fn mark_contributing_helicities(
        &self,
        momenta: &[LorentzVector<F>],
        rel_threshold: F,
        scratch: &mut ScratchSpace<F>,
        good: &mut [bool],
    ) {
        let folded = self.eval.folded_hel();
        resolve_moms(folded, momenta, scratch);
        let env = self.eval_env(folded, momenta, &[], None);
        fill_arenas(folded, &env, scratch);
        let RootKind::Hels { n_flows, locs } = &folded.program().root else {
            panic!("helicity probe on a program without a helicity-expanded root");
        };
        let n = *n_flows as usize;
        debug_assert_eq!(locs.len(), good.len() * n);

        // Per-combination T_c: the CF-contracted |M_c|² (nonnegative — CF is
        // positive semidefinite). For a single flow the constant CF(1,1) factor is
        // omitted; it cancels in the relative test.
        let mut ts = vec![F::zero(); good.len()];
        for (t, jamps) in ts.iter_mut().zip(locs.chunks_exact(n)) {
            if n == 1 {
                *t = scratch.scalars[jamps[0] as usize].norm_sqr();
            } else {
                for i in 0..n {
                    let mut ztemp = C::new(F::zero(), F::zero());
                    for (j, &lj) in jamps.iter().enumerate() {
                        ztemp = ztemp + scratch.scalars[lj as usize].scale(self.cf[j * n + i]);
                    }
                    *t = *t + (ztemp * scratch.scalars[jamps[i] as usize].conj()).re;
                }
            }
        }
        let ans = ts.iter().fold(F::zero(), |acc, &t| acc + t);
        if !(ans > F::zero()) {
            return;
        }
        let ncomb = num_traits::cast::<usize, F>(good.len()).expect("NCOMB representable");
        let cut = ans * rel_threshold / ncomb;
        for (g, &t) in good.iter_mut().zip(&ts) {
            *g = *g || t > cut;
        }
    }

    /// Evaluate the complex amplitude M for a single helicity configuration (the
    /// coherent sum over all diagrams).
    pub fn eval_amplitude(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        scratch: &mut ScratchSpace<F>,
    ) -> C<F> {
        if momenta.len() != self.eval.n_ext() || helicities.len() != self.eval.n_ext() {
            return C::new(F::zero(), F::zero());
        }
        self.run(momenta, helicities, None, scratch)
    }

    /// The per-evaluation immutable context for one (momenta, helicity) point,
    /// reading `folded`'s external-leg table (the base or helicity-expanded arena).
    fn eval_env<'e>(
        &'e self,
        folded: &'e Folded,
        momenta: &'e [LorentzVector<F>],
        helicities: &'e [i32],
        ward_leg: Option<usize>,
    ) -> EvalEnv<'e, F> {
        EvalEnv {
            consts_c: &self.consts_c,
            consts_f: &self.consts_f,
            ext_legs: folded.ext_legs(),
            momenta,
            helicities,
            ward_leg,
        }
    }

    /// Walk the folded arena for one (momenta, helicity) point. `ward_leg` gauge-
    /// substitutes one external boson's polarisation with its momentum (test-only).
    fn run(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        ward_leg: Option<usize>,
        scratch: &mut ScratchSpace<F>,
    ) -> C<F> {
        let folded = self.eval.folded();
        resolve_moms(folded, momenta, scratch);
        let env = self.eval_env(folded, momenta, helicities, ward_leg);
        run_forward_typed(folded, &env, scratch)
    }

    /// Walk the folded arena for one (momenta, helicity) point, returning the
    /// per-flow JAMPs `J_i` — the scalar children of the [`Op::Flows`] root. Backs the
    /// per-flow JAMP probes ([`eval_m2`](Self::eval_m2) reads its JAMPs from the
    /// helicity-expanded root instead).
    #[cfg(test)]
    fn run_flows(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        scratch: &mut ScratchSpace<F>,
    ) -> Vec<C<F>> {
        let folded = self.eval.folded();
        resolve_moms(folded, momenta, scratch);
        let env = self.eval_env(folded, momenta, helicities, None);
        run_forward_flows_typed(folded, &env, scratch)
    }

    /// Test-only: evaluate the amplitude with one external boson's polarisation ε^μ
    /// replaced by its 4-momentum q^μ (full-amplitude Ward-identity check).
    #[cfg(test)]
    fn eval_amplitude_ward(
        &self,
        momenta: &[LorentzVector<F>],
        helicities: &[i32],
        ward_leg: usize,
        scratch: &mut ScratchSpace<F>,
    ) -> C<F> {
        self.run(momenta, helicities, Some(ward_leg), scratch)
    }
}

impl<'a> BoundAmplitude<'a, f64> {
    /// Rebind this scalar amplitude onto an `N`-wide SIMD lane pack, splatting each
    /// card constant across all lanes. The lane amplitude borrows the same compiled
    /// evaluator; build it once, pair it with its own
    /// [`scratch_space`](BoundAmplitude::scratch_space), and evaluate `N` points at a
    /// time with [`eval_m2_lanes`] to keep the hot loop allocation-free.
    ///
    /// [`eval_m2_lanes`]: fn@eval_m2_lanes
    pub fn broadcast_lanes<const N: usize>(&self) -> BoundAmplitude<'a, LaneField<N>>
    where
        LaneLen<N>: IntoArrayLength,
        LaneField<N>: Real,
    {
        let consts_c = self
            .consts_c
            .iter()
            .map(|z| C::new(NumericArray::splat(z.re), NumericArray::splat(z.im)))
            .collect();
        let consts_f = self
            .consts_f
            .iter()
            .map(|&x| NumericArray::splat(x))
            .collect();
        let cf = self.cf.iter().map(|&x| NumericArray::splat(x)).collect();
        BoundAmplitude::new(self.eval, consts_c, consts_f, cf)
    }
}

/// Color- and helicity-summed |M|² for `N` phase-space points in one lane-batched
/// pass. `points[k]` is the external 4-momenta of point `k` in [`eval_m2`] order;
/// the returned lane `k` is bit-identical to the scalar [`eval_m2`] at `points[k]`,
/// provided the batch is kinematically homogeneous (see the [`lanes`](super::lanes)
/// contract). `amp` is a lane amplitude from [`BoundAmplitude::broadcast_lanes`].
///
/// [`eval_m2`]: BoundAmplitude::eval_m2
pub fn eval_m2_lanes<const N: usize>(
    amp: &BoundAmplitude<'_, LaneField<N>>,
    points: &[&[LorentzVector<f64>]; N],
    scratch: &mut ScratchSpace<LaneField<N>>,
) -> [f64; N]
where
    LaneLen<N>: IntoArrayLength,
    LaneField<N>: Real,
{
    let momenta = transpose_points(points);
    unpack(amp.eval_m2(&momenta, scratch))
}

/// Test helper: evaluate the folded arena through the generic [`WaveformSlot`] forward
/// pass, returning the root slot. Nodes are visited in arena (storage) order; each node's
/// children are already computed and read from `res` by id, so a shared (DAG) node is
/// evaluated exactly once. Rooting a sub-tree returns that node's slot, which the
/// per-diagram probes read through production kernels.
#[cfg(test)]
pub(super) fn run_forward_slot<F: Real>(
    folded: &Folded,
    env: &EvalEnv<'_, F>,
    res: &mut Vec<WaveformSlot<F>>,
) -> WaveformSlot<F> {
    let ast = &folded.ast;
    res.clear();
    res.reserve(ast.len());
    for id in ast.iter() {
        let ids = ast.children_ids(id);
        let value = apply(ast.value(id), ids.len(), |i| &res[ids[i] as usize], env);
        cross_check_node(
            folded.analysis(),
            id,
            &value,
            env.momenta,
            env.ward_leg.is_none(),
        );
        res.push(value);
    }
    res[ast.root() as usize]
}

/// Check the kinematic contract a helicity-pruned evaluator requires: two incoming
/// legs, each along the z axis (zero transverse momentum), with zero total incoming
/// 3-momentum (partonic CM). A combination `prune_zero_helicities` drops can be an
/// exact identity in this frame yet contribute at the ~1e-3 level once the event is
/// boosted — massive-particle helicity is not boost invariant, so J_z conservation
/// about the beam axis (which zeroes those combinations) only holds in the frame the
/// pruning survey used. Compiled into debug builds and `extended-validation` only;
/// released binaries pay nothing for it.
#[inline]
fn assert_partonic_cm_beams_along_z<F: Real>(momenta: &[LorentzVector<F>], n_in: usize) {
    #[cfg(any(debug_assertions, feature = "extended-validation"))]
    {
        assert_eq!(
            n_in, 2,
            "pruned evaluator has {n_in} incoming legs; the pruning survey only \
             probes 2 → n kinematics"
        );
        let tol: F = num_traits::cast(1e-6_f64).expect("tolerance representable");
        let scale = momenta[0].e().abs().max(momenta[1].e().abs()).max(F::one());
        let small = |x: F| x.abs() <= tol * scale;

        assert!(
            small(momenta[0].px())
                && small(momenta[0].py())
                && small(momenta[1].px())
                && small(momenta[1].py()),
            "pruned evaluator requires beams along the z axis; got incoming transverse \
             momenta ({:?}, {:?}) and ({:?}, {:?})",
            momenta[0].px(),
            momenta[0].py(),
            momenta[1].px(),
            momenta[1].py(),
        );

        let px_tot = momenta[0].px() + momenta[1].px();
        let py_tot = momenta[0].py() + momenta[1].py();
        let pz_tot = momenta[0].pz() + momenta[1].pz();
        assert!(
            small(px_tot) && small(py_tot) && small(pz_tot),
            "pruned evaluator requires partonic-CM kinematics (zero total incoming \
             3-momentum); got ({px_tot:?}, {py_tot:?}, {pz_tot:?}) — a boosted input \
             silently revives helicity combinations the pruning survey dropped"
        );
    }
    #[cfg(not(any(debug_assertions, feature = "extended-validation")))]
    {
        let _ = (momenta, n_in);
    }
}

/// Resolve the per-point momentum pool: `scratch.moms[id]` becomes the external-momentum
/// combination the momentum table interns at `id`. Helicity-independent, so callers that
/// loop over helicities (`eval_m2`) resolve it once per point and reuse it.
fn resolve_moms<F: Real>(
    folded: &Folded,
    momenta: &[LorentzVector<F>],
    scratch: &mut ScratchSpace<F>,
) {
    let table = folded.analysis().mom_table();
    scratch.moms.clear();
    scratch.moms.reserve(table.len());
    for id in 0..table.len() as u32 {
        scratch.moms.push(table.resolve(id, momenta));
    }
}

/// Evaluate the whole-amplitude arena through the typed instruction stream, returning the
/// root scalar = M. Assumes [`resolve_moms`] has populated the momentum pool for this point.
fn run_forward_typed<F: Real>(
    folded: &Folded,
    env: &EvalEnv<'_, F>,
    scratch: &mut ScratchSpace<F>,
) -> C<F> {
    fill_arenas(folded, env, scratch);
    match &folded.program().root {
        RootKind::Single(loc) => scratch.scalars[*loc as usize],
        _ => panic!("run_forward_typed on a multi-root amplitude"),
    }
}

/// Evaluate a `Flows`-rooted arena through the typed instruction stream, returning each
/// JAMP scalar (the root's children) in flow order. Assumes [`resolve_moms`] has populated
/// the momentum pool for this point. Backs the per-flow JAMP probes.
#[cfg(test)]
fn run_forward_flows_typed<F: Real>(
    folded: &Folded,
    env: &EvalEnv<'_, F>,
    scratch: &mut ScratchSpace<F>,
) -> Vec<C<F>> {
    fill_arenas(folded, env, scratch);
    match &folded.program().root {
        RootKind::Flows(locs) => locs.iter().map(|&l| scratch.scalars[l as usize]).collect(),
        _ => panic!("run_forward_flows_typed on a non-Flows amplitude"),
    }
}

/// Run the typed instruction stream, filling the per-class result arenas. Every node
/// writes its result — by direct index (`arena[loc]`) — to the arena its statically-known
/// output class selects; operands are read directly from the arena their class fixes, so no
/// per-value type dispatch happens on the hot path. The remaining per-iteration branches are
/// the instruction dispatch and the slice bounds checks on arena indexing.
fn fill_arenas<F: Real>(folded: &Folded, env: &EvalEnv<'_, F>, scratch: &mut ScratchSpace<F>) {
    let prog = folded.program();
    scratch.ensure_sizes(&prog.arena_sizes);
    let ops = &prog.operands;
    let mom_ops = &prog.mom_operands;

    for ((id, instr), &loc) in prog.instrs.iter().enumerate().zip(prog.loc.iter()) {
        let loc = loc as usize;
        match *instr {
            Instr::ComplexConst { pool } => scratch.scalars[loc] = env.consts_c[pool as usize],
            Instr::RealConst { pool } => scratch.reals[loc] = env.consts_f[pool as usize],
            Instr::ExternalScalar { leg } => {
                let WaveformSlot::Scalar(s) = build_external_slot(env, leg as usize) else {
                    panic!("external scalar leg produced a non-scalar slot");
                };
                scratch.scalars[loc] = s.value;
            }
            Instr::ExternalVector { leg } => {
                let WaveformSlot::Vector(v) = build_external_slot(env, leg as usize) else {
                    panic!("external vector leg produced a non-vector slot");
                };
                scratch.vectors[loc] = v.eps;
            }
            Instr::ExternalFin { leg } => {
                let WaveformSlot::FermionIn(f) = build_external_slot(env, leg as usize) else {
                    panic!("external ket leg produced a non-fermion-in slot");
                };
                scratch.fin[loc] = f.spinor;
            }
            Instr::ExternalFout { leg } => {
                let WaveformSlot::FermionOut(f) = build_external_slot(env, leg as usize) else {
                    panic!("external bra leg produced a non-fermion-out slot");
                };
                scratch.fout[loc] = f.spinor;
            }
            Instr::PropagateScalar {
                input,
                mass,
                width,
                mom,
            } => {
                let out = kernel::propagate_scalar_bare(
                    scratch.scalars[input as usize],
                    &scratch.moms[mom as usize],
                    scratch.reals[mass as usize],
                    scratch.reals[width as usize],
                );
                scratch.scalars[loc] = out;
            }
            Instr::PropagateVector {
                input,
                mass,
                width,
                mom,
            } => {
                let out = kernel::propagate_vector_bare(
                    &scratch.vectors[input as usize],
                    &scratch.moms[mom as usize],
                    scratch.reals[mass as usize],
                    scratch.reals[width as usize],
                );
                scratch.vectors[loc] = out;
            }
            Instr::PropagateFin {
                input,
                mass,
                width,
                mom,
            } => {
                let out = kernel::propagate_fin_bare(
                    &scratch.fin[input as usize],
                    &scratch.moms[mom as usize],
                    scratch.reals[mass as usize],
                    scratch.reals[width as usize],
                );
                scratch.fin[loc] = out;
            }
            Instr::PropagateFout {
                input,
                mass,
                width,
                mom,
            } => {
                let out = kernel::propagate_fout_bare(
                    &scratch.fout[input as usize],
                    &scratch.moms[mom as usize],
                    scratch.reals[mass as usize],
                    scratch.reals[width as usize],
                );
                scratch.fout[loc] = out;
            }
            Instr::AddScalar { start, len } => {
                let slice = &ops[start as usize..(start + len) as usize];
                let mut value = scratch.scalars[slice[0].index()];
                for op in &slice[1..] {
                    value = value + scratch.scalars[op.index()];
                }
                scratch.scalars[loc] = value;
            }
            Instr::AddVector { start, len } => {
                let slice = &ops[start as usize..(start + len) as usize];
                let mut eps = scratch.vectors[slice[0].index()];
                for op in &slice[1..] {
                    eps = eps + scratch.vectors[op.index()];
                }
                scratch.vectors[loc] = eps;
            }
            Instr::AddFin { start, len } => {
                let slice = &ops[start as usize..(start + len) as usize];
                let mut spinor = scratch.fin[slice[0].index()];
                for op in &slice[1..] {
                    spinor = spinor + scratch.fin[op.index()];
                }
                scratch.fin[loc] = spinor;
            }
            Instr::AddFout { start, len } => {
                let slice = &ops[start as usize..(start + len) as usize];
                let mut spinor = scratch.fout[slice[0].index()];
                for op in &slice[1..] {
                    spinor = spinor + scratch.fout[op.index()];
                }
                scratch.fout[loc] = spinor;
            }
            Instr::Mul { start, len } => {
                exec_mul(scratch, loc, &ops[start as usize..(start + len) as usize]);
            }
            Instr::GammaVout { bra, ket, reversed } => {
                let out = kernel::gamma_vout_bare(
                    &scratch.fout[bra as usize],
                    &scratch.fin[ket as usize],
                    reversed,
                );
                scratch.vectors[loc] = out;
            }
            Instr::FfvVout {
                bra,
                ket,
                gl,
                gr,
                reversed,
            } => {
                let out = kernel::ffv_vout_bare(
                    &scratch.fout[bra as usize],
                    &scratch.fin[ket as usize],
                    scratch.scalars[gl as usize],
                    scratch.scalars[gr as usize],
                    reversed,
                );
                scratch.vectors[loc] = out;
            }
            Instr::GammaFin { v, f } => {
                let eps = scratch.vectors[v as usize];
                let out = kernel::off_shell_fin_bare(&eps, &scratch.fin[f as usize]);
                scratch.fin[loc] = out;
            }
            Instr::GammaFout { v, f } => {
                let eps = scratch.vectors[v as usize];
                let out = kernel::off_shell_fout_bare(&eps, &scratch.fout[f as usize]);
                scratch.fout[loc] = out;
            }
            Instr::FfvFin { v, f, gl, gr } => {
                let eps = scratch.vectors[v as usize];
                let out = kernel::ffv_fin_bare(
                    &eps,
                    &scratch.fin[f as usize],
                    scratch.scalars[gl as usize],
                    scratch.scalars[gr as usize],
                );
                scratch.fin[loc] = out;
            }
            Instr::FfvFout { v, f, gl, gr } => {
                let eps = scratch.vectors[v as usize];
                let out = kernel::ffv_fout_bare(
                    &eps,
                    &scratch.fout[f as usize],
                    scratch.scalars[gl as usize],
                    scratch.scalars[gr as usize],
                );
                scratch.fout[loc] = out;
            }
            Instr::ProjFin { f, chirality } => {
                let out = kernel::proj_fin_bare(&scratch.fin[f as usize], chirality);
                scratch.fin[loc] = out;
            }
            Instr::ProjFout { f, chirality } => {
                let out = kernel::proj_fout_bare(&scratch.fout[f as usize], chirality);
                scratch.fout[loc] = out;
            }
            Instr::Bilinear {
                bra,
                ket,
                chirality,
            } => {
                let out = kernel::scalar_bilinear_bare(
                    &scratch.fout[bra as usize],
                    &scratch.fin[ket as usize],
                    chirality,
                );
                scratch.scalars[loc] = out;
            }
            Instr::Metric { a, b } => {
                let out =
                    kernel::metric_bare(&scratch.vectors[a as usize], &scratch.vectors[b as usize]);
                scratch.scalars[loc] = out;
            }
            Instr::MetricVout { v } => {
                let out = kernel::metric_vout_bare(&scratch.vectors[v as usize]);
                scratch.vectors[loc] = out;
            }
            Instr::PMom { mom } => {
                let out = kernel::pmom_bare(&scratch.moms[mom as usize]);
                scratch.vectors[loc] = out;
            }
            Instr::PMomOut { start, len } => {
                let slice = &mom_ops[start as usize..(start + len) as usize];
                let mut acc = LorentzVector::zero();
                for &mid in slice {
                    acc = acc + scratch.moms[mid as usize];
                }
                let neg = -acc;
                scratch.vectors[loc] = kernel::pmom_bare(&neg);
            }
            Instr::Flows | Instr::Hels => {}
        }

        cross_check_typed(folded, id as NodeId, scratch, env);
    }
}

/// n-ary product on bare typed operands (mirrors [`mul_apply`]): reals fold into `F`,
/// scalars into a complex coefficient, and the single non-scalar current is scaled by the
/// combined coefficient. Momentum is not routed here — each current's momentum lives in the
/// pool keyed by its momentum id. Writes the result to index `loc` of the arena its class
/// selects.
#[inline]
fn exec_mul<F: Real>(scratch: &mut ScratchSpace<F>, loc: usize, operands: &[OperandRef]) {
    let mut real_acc = F::one();
    let mut cplx_acc = C::new(F::one(), F::zero());
    let mut current = MulCurrent::None;
    for op in operands {
        match op.class() {
            0 => real_acc = real_acc * scratch.reals[op.index()],
            1 => cplx_acc = cplx_acc * scratch.scalars[op.index()],
            2 => {
                debug_assert!(
                    matches!(current, MulCurrent::None),
                    "Mul: at most one non-scalar child"
                );
                current = MulCurrent::Vector(scratch.vectors[op.index()]);
            }
            3 => {
                debug_assert!(
                    matches!(current, MulCurrent::None),
                    "Mul: at most one non-scalar child"
                );
                current = MulCurrent::Fin(scratch.fin[op.index()]);
            }
            _ => {
                debug_assert!(
                    matches!(current, MulCurrent::None),
                    "Mul: at most one non-scalar child"
                );
                current = MulCurrent::Fout(scratch.fout[op.index()]);
            }
        }
    }
    let coeff = cplx_acc * real_acc;
    match current {
        MulCurrent::None => scratch.scalars[loc] = coeff,
        MulCurrent::Vector(eps) => scratch.vectors[loc] = eps * coeff,
        MulCurrent::Fin(spinor) => scratch.fin[loc] = spinor * coeff,
        MulCurrent::Fout(spinor) => scratch.fout[loc] = spinor * coeff,
    }
}

/// The non-scalar current a [`Instr::Mul`] carries (at most one).
enum MulCurrent<F: Real> {
    None,
    Vector(ComplexVector<F>),
    Fin(Bispinor<F, Ket>),
    Fout(Bispinor<F, Bra>),
}

/// Reconstruct a node's slot from its bare typed arena — re-attaching the routing momentum
/// from the pool (keyed by the node's momentum id) — and cross-check it against the static
/// analysis (debug / extended-validation only), so the typed forward pass carries the same
/// output-type/constness guarantees as the generic one. With momentum now sourced from the
/// pool, the momentum-agreement leg of the check confirms pool-indexing consistency.
#[inline(always)]
fn cross_check_typed<F: Real>(
    folded: &Folded,
    id: NodeId,
    scratch: &ScratchSpace<F>,
    env: &EvalEnv<'_, F>,
) {
    #[cfg(any(debug_assertions, feature = "extended-validation"))]
    {
        use super::analysis::Storage;
        let an = folded.analysis();
        if let Some(class) = an.out_type(id).storage() {
            let loc = folded.program().loc[id as usize] as usize;
            let mom = scratch.moms[an.mom_id(id) as usize];
            let slot = match class {
                Storage::Real => WaveformSlot::Real(scratch.reals[loc]),
                Storage::Scalar => WaveformSlot::Scalar(ScalarWf {
                    value: scratch.scalars[loc],
                    momentum: mom,
                }),
                Storage::Vector => WaveformSlot::Vector(VectorWf {
                    eps: scratch.vectors[loc],
                    momentum: mom,
                }),
                Storage::FermionIn => {
                    WaveformSlot::FermionIn(InDiracWf::from_spinor(scratch.fin[loc], mom))
                }
                Storage::FermionOut => {
                    WaveformSlot::FermionOut(OutDiracWf::from_spinor(scratch.fout[loc], mom))
                }
            };
            cross_check_node(an, id, &slot, env.momenta, env.ward_leg.is_none());
        }
    }
    #[cfg(not(any(debug_assertions, feature = "extended-validation")))]
    {
        let _ = (folded, id, scratch, env);
    }
}

/// Build one external wavefunction slot from the folded leg table, applying the
/// (test-only) Ward gauge substitution when this leg is the chosen one. A leg entry
/// with a baked helicity (helicity-expanded arenas) uses it; otherwise the helicity
/// comes from the per-evaluation assignment.
fn build_external_slot<F: Real>(env: &EvalEnv<'_, F>, leg_table_idx: usize) -> WaveformSlot<F> {
    let leg = env.ext_legs[leg_table_idx];
    let leg_idx = leg.leg_idx as usize;
    // Ward-identity gauge substitution (test-only): replace the chosen external boson's
    // polarisation ε^μ with its own 4-momentum q^μ. The coherent diagram sum must then
    // vanish (current conservation).
    if env.ward_leg == Some(leg_idx) {
        let q = env.momenta[leg_idx];
        return WaveformSlot::Vector(VectorWf {
            eps: ComplexVector::from(q),
            momentum: q,
        });
    }
    let helicity = match leg.hel {
        Some(h) => i32::from(h),
        None => env.helicities[leg_idx],
    };
    build_external_core(
        env.momenta[leg_idx],
        helicity,
        leg.spin,
        leg.charge,
        leg.incoming,
        env.consts_f[leg.mass as usize],
    )
}

/// Cross-check the static [`NodeAnalysis`] against a node's freshly-computed slot: the
/// predicted output type must match the slot variant, a node marked constant must carry
/// no momentum, and the predicted momentum combination must resolve to the slot's routed
/// momentum. Enabled in debug builds and under `extended-validation` (so it runs over the
/// full unit suite and both MG gate suites); compiled out of release builds.
#[inline(always)]
fn cross_check_node<F: Real>(
    analysis: &NodeAnalysis,
    id: NodeId,
    slot: &WaveformSlot<F>,
    momenta: &[LorentzVector<F>],
    check_momentum: bool,
) {
    #[cfg(any(debug_assertions, feature = "extended-validation"))]
    {
        use super::analysis::{MomTable, NodeType};
        use crate::helas::repr::lorentz::LorentzVector as LV;

        let ty = analysis.out_type(id);
        let type_ok = match slot {
            WaveformSlot::Real(_) => ty == NodeType::RealConst,
            WaveformSlot::Scalar(_) => {
                matches!(ty, NodeType::ScalarConst | NodeType::ScalarWf)
            }
            WaveformSlot::Vector(_) => ty == NodeType::Vector,
            WaveformSlot::FermionIn(_) => ty == NodeType::FermionIn,
            WaveformSlot::FermionOut(_) => ty == NodeType::FermionOut,
            // A structurally-zero node (empty product/sum) leaves the type unconstrained.
            WaveformSlot::Empty => true,
        };
        assert!(
            type_ok,
            "analysis node {id}: predicted output type {ty:?} but slot is {slot:?}"
        );

        // A card-time constant must be momentum-free (Real, or a zero-momentum Scalar).
        if analysis.is_const(id) {
            if let Some(m) = slot.momentum() {
                assert!(
                    m == LV::zero(),
                    "analysis node {id}: marked constant but slot carries momentum {m:?}"
                );
            }
        }

        // The predicted momentum combination must resolve to the slot's routed momentum
        // (up to the reassociation rounding of summing externals in a different order).
        // Skipped under the Ward gauge substitution, which deliberately overrides an
        // external boson's stored momentum with its raw `+q` off convention.
        let predicted = analysis.resolve_mom(id, momenta);
        match slot.momentum() {
            _ if !check_momentum => {}
            Some(actual) => {
                let tol: F = num_traits::cast(1e-9_f64).expect("tolerance representable");
                let close =
                    |x: F, y: F| -> bool { (x - y).abs() <= tol * x.abs().max(y.abs()) + tol };
                assert!(
                    close(predicted.e(), actual.e())
                        && close(predicted.px(), actual.px())
                        && close(predicted.py(), actual.py())
                        && close(predicted.pz(), actual.pz()),
                    "analysis node {id}: predicted momentum {predicted:?} \
                     disagrees with slot momentum {actual:?}"
                );
            }
            None => assert!(
                analysis.mom_id(id) == MomTable::ZERO,
                "analysis node {id}: momentum-free slot but non-zero momentum id"
            ),
        }
    }
    #[cfg(not(any(debug_assertions, feature = "extended-validation")))]
    {
        let _ = (analysis, id, slot, momenta, check_momentum);
    }
}

/// Reduce one folded node from its children's already-evaluated results, read through
/// the `kid` accessor — the forward scan hands out references into its result buffer
/// by child id. Constant leaves resolve from the pools;
/// `External` reads the leg table; `Mul`/`Add` fold over the accessor; the Lorentz
/// primitives have their true arity in the `kernel::` signatures, so operands pass
/// straight through by reference — no staging array, no slice bounds checks in the
/// outlined kernels.
pub(super) fn apply<'a, F: Real + 'a>(
    node: &Node<Const>,
    n_kids: usize,
    kid: impl Fn(usize) -> &'a WaveformSlot<F>,
    env: &EvalEnv<'_, F>,
) -> WaveformSlot<F> {
    match node.op {
        Op::Coupling => {
            debug_assert_eq!(node.leaf.kind(), ConstKind::Complex);
            WaveformSlot::Scalar(ScalarWf {
                value: env.consts_c[node.leaf.index() as usize],
                momentum: LorentzVector::zero(),
            })
        }
        Op::Mass | Op::Width | Op::Coeff => {
            debug_assert_eq!(node.leaf.kind(), ConstKind::Real);
            WaveformSlot::Real(env.consts_f[node.leaf.index() as usize])
        }
        // Folds to a real-pool index when its rational carries no factor of `i`, else
        // a complex-pool index (see `fold::Folded::build`).
        Op::CoeffRat => match node.leaf.kind() {
            ConstKind::Real => WaveformSlot::Real(env.consts_f[node.leaf.index() as usize]),
            ConstKind::Complex => WaveformSlot::Scalar(ScalarWf {
                value: env.consts_c[node.leaf.index() as usize],
                momentum: LorentzVector::zero(),
            }),
            _ => panic!("CoeffRat node without a resolved pool index"),
        },
        Op::External => {
            debug_assert_eq!(node.leaf.kind(), ConstKind::Ext);
            build_external_slot(env, node.leaf.index() as usize)
        }
        Op::Propagate => kernel::propagate(kid(0), kid(1), kid(2)),
        Op::Add => (0..n_kids).fold(WaveformSlot::Empty, |acc, i| acc + *kid(i)),
        Op::Mul => mul_apply((0..n_kids).map(|i| *kid(i))),
        // Lorentz primitives: each `Op` maps 1-to-1 to a `kernel::` fn named for it.
        Op::GammaVout => kernel::gamma_vout(kid(0), kid(1)),
        Op::FfvVout => kernel::ffv_vout(kid(0), kid(1), kid(2), kid(3)),
        Op::FfvIout => kernel::ffv_iout(kid(0), kid(1), kid(2), kid(3)),
        Op::FfvOout => kernel::ffv_oout(kid(0), kid(1), kid(2), kid(3)),
        Op::GammaIout => kernel::gamma_iout(kid(0), kid(1)),
        Op::GammaOout => kernel::gamma_oout(kid(0), kid(1)),
        Op::ProjM => kernel::proj_m(kid(0)),
        Op::ProjP => kernel::proj_p(kid(0)),
        Op::ProjMAmp => kernel::proj_m_amp(kid(0), kid(1)),
        Op::ProjPAmp => kernel::proj_p_amp(kid(0), kid(1)),
        Op::Metric => kernel::metric(kid(0), kid(1)),
        Op::MetricVout => kernel::metric_vout(kid(0)),
        Op::IdentityAmp => kernel::identity_amp(kid(0), kid(1)),
        Op::PMom => kernel::pmom(kid(0)),
        // n-ary (all vertex inputs): the one variadic kernel takes the operands as
        // an iterator of references.
        Op::PMomOut => kernel::pmom_out((0..n_kids).map(kid)),
        // The variadic roots are only ever amplitude roots: the CF-weighted /
        // helicity-summed `eval_m2` (not this per-node, single-scalar `apply`) is
        // what consumes their children's slots directly.
        Op::Flows => panic!("Op::Flows has no single-scalar node evaluation"),
        Op::Hels => panic!("Op::Hels has no single-scalar node evaluation"),
    }
}

/// n-ary product (the `Mul` op). Scalar/real children fold into a complex coefficient
/// (reals kept in `F`); at most one non-scalar child carries the output type and absorbs
/// the scalar momentum.
pub fn mul_apply<F: Real>(children: impl IntoIterator<Item = WaveformSlot<F>>) -> WaveformSlot<F> {
    let mut real_acc = F::one();
    let mut cplx_acc = C::new(F::one(), F::zero());
    let mut scalar_mom = LorentzVector::zero();
    let mut non_scalar = WaveformSlot::Empty;
    for child in children {
        match child {
            WaveformSlot::Real(r) => real_acc = real_acc * r,
            WaveformSlot::Scalar(s) => {
                cplx_acc = cplx_acc * s.value;
                scalar_mom = scalar_mom + s.momentum;
            }
            WaveformSlot::Empty => {}
            other => {
                assert!(
                    matches!(non_scalar, WaveformSlot::Empty),
                    "Mul: at most one non-scalar child"
                );
                non_scalar = other;
            }
        }
    }
    let coeff = cplx_acc * real_acc;
    match non_scalar {
        WaveformSlot::Empty => WaveformSlot::Scalar(ScalarWf {
            value: coeff,
            momentum: scalar_mom,
        }),
        // Route the scalar factors' momentum into the surviving non-scalar current so
        // the propagator sees the conserved q. A continuing *fermion* current follows
        // the HELAS off-shell conventions (`fsixxx` q = fi − s, `fsoxxx` q = fo + s,
        // mirroring `fvixxx`/`fvoxxx`): ket subtracts the absorbed boson's
        // momentum, bra adds it. Pinned by e+e-→τ+τ-H (H emitted off the τ
        // ket line) vs MadGraph per-diagram AMP().
        other => match coeff * other {
            WaveformSlot::Vector(mut v) => {
                v.momentum = v.momentum + scalar_mom;
                WaveformSlot::Vector(v)
            }
            WaveformSlot::FermionIn(mut f) => {
                f.momentum = f.momentum - scalar_mom;
                WaveformSlot::FermionIn(f)
            }
            WaveformSlot::FermionOut(mut f) => {
                f.momentum = f.momentum + scalar_mom;
                WaveformSlot::FermionOut(f)
            }
            scaled => scaled,
        },
    }
}

/// Test helper: lower a single diagram (symmetry × Fermi sign folded in) and run the
/// unified forward pass, returning the root [`WaveformSlot`]. With a `ContractAmplitude`
/// root this is the scalar amplitude; rooting at an off-shell current returns that
/// current, which lets the cross-checks read an intermediate node through production.
#[cfg(test)]
fn eval_single_diagram_slot<F: Real + FromPrimitive>(
    diagram: &DiagramEval,
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    evaluated: &EvaluatedModel,
) -> WaveformSlot<F> {
    let symbolic = lower::lower(std::slice::from_ref(diagram));
    let folded = Folded::build(&symbolic);
    let (consts_c, consts_f) = folded.pools::<F>(evaluated);
    let env = EvalEnv {
        consts_c: &consts_c,
        consts_f: &consts_f,
        ext_legs: folded.ext_legs(),
        momenta,
        helicities,
        ward_leg: None,
    };
    run_forward_slot(&folded, &env, &mut Vec::new())
}

/// Test helper: the scalar amplitude of a single diagram (see
/// [`eval_single_diagram_slot`]). Used by the per-diagram probes.
#[cfg(test)]
fn eval_single_diagram<F: Real + FromPrimitive>(
    diagram: &DiagramEval,
    momenta: &[LorentzVector<F>],
    helicities: &[i32],
    evaluated: &EvaluatedModel,
) -> C<F> {
    match eval_single_diagram_slot(diagram, momenta, helicities, evaluated) {
        WaveformSlot::Scalar(s) => s.value,
        WaveformSlot::Empty => C::new(F::zero(), F::zero()),
        other => panic!("amplitude root is not a scalar: {other:?}"),
    }
}

/// Build an external wavefunction from its kinematics + interned mass.
fn build_external_core<F: Real>(
    momentum: LorentzVector<F>,
    helicity: i32,
    spin: i32,
    charge: Charge,
    is_incoming: bool,
    mass: F,
) -> WaveformSlot<F> {
    match spin {
        1 => WaveformSlot::Scalar(ScalarWf::sxxxxx(momentum, if is_incoming { -1 } else { 1 })),
        2 => {
            let hel = match helicity {
                -1 => SpinorHelicity::Down,
                1 => SpinorHelicity::Up,
                other => panic!("invalid fermion helicity {other}"),
            };
            // HELAS external adjoint: a leg is a ket (ket, ixxxxx) iff it is an
            // incoming particle or an outgoing antiparticle; otherwise it is a bra
            // (bra, oxxxxx). Equivalently ket ⟺ (is_incoming == is_particle).
            let is_particle = matches!(charge, Charge::Particle);
            if is_incoming == is_particle {
                WaveformSlot::FermionIn(InDiracWf::from_momentum(momentum, mass, hel, charge))
            } else {
                WaveformSlot::FermionOut(OutDiracWf::from_momentum(momentum, mass, hel, charge))
            }
        }
        3 => {
            let wf = VectorWf::vxxxxx(momentum, mass, helicity, if is_incoming { -1 } else { 1 });
            WaveformSlot::Vector(wf)
        }
        other => panic!("unsupported external spin code: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use itertools::iproduct;
    use num_complex::Complex64;

    use super::kernel::*;
    use super::*;
    use crate::helas::repr::lorentz::{SpinorRepr, VectorRepr};
    use crate::helas::repr::numbers::Chirality;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::{
        helas::{
            eval::diagram_eval::{ExtLegInfo, PropInfo, VertexInfo, VertexTerm},
            eval::root_diagram::{EvalNode, EvalNodeId},
            ffv2_4_3, iovxxx, jioxxx,
            repr::numbers::Charge,
            OutDiracWf,
        },
        ufo::color::ColorExpr,
        ufo::slha::ParamCard,
    };
    use num_complex::ComplexFloat;

    /// Placeholder color factor for hand-built `VertexTerm`s in tests that don't
    /// exercise color at all — `VertexTerm::from_ufo` ignores its `_color` arg.
    fn no_color() -> ColorExpr {
        ColorExpr {
            coeff: 1,
            atoms: vec![],
        }
    }

    /// Uncrossed per-leg binding shorthand for hand-built adjoint vectors (the
    /// hand-built diagrams bind wavefunctions in MG order, so no crossing).
    #[cfg(feature = "extended-validation")]
    fn lf(
        adjoint: crate::helas::eval::root_lorentz::Adjoint,
    ) -> Option<crate::helas::eval::root_lorentz::LegAdjoint> {
        Some(crate::helas::eval::root_lorentz::LegAdjoint {
            adjoint,
            crossed: false,
        })
    }

    /// Cross-check the VVS off-shell *vector* current (`MetricVout` node) against
    /// ALOHA `VVS1P1N_1.f` times +i. ALOHA's routine (coupling stripped) is
    ///   V1(3) = -i·V2(3)·S ;  V1(4..6) = +i·V2(4..6)·S    (i.e. -i·g·V2·S);
    /// vibegraph's producer is the bare physical current `g^{μν}V2_ν·S = V2^μ·S`
    /// (i·ALOHA): the −i ALOHA folds into the vertex routine lives in vibegraph's
    /// vector propagator instead (see `metric_vout` / `propagate_core`). vibegraph
    /// applies the coupling separately, so the bare dispatch tree for `Metric(1,2)`
    /// rooted at vector leg 1 must reproduce this.
    #[test]
    fn test_metric_vout_vs_aloha_vvs1p1n1() {
        let v2 = VectorWf {
            eps: ComplexVector::new([
                C::new(2.0, 1.0),
                C::new(3.0, -1.0),
                C::new(5.0, 2.0),
                C::new(7.0, -3.0),
            ]),
            momentum: LorentzVector::new(10.0, 1.0, 2.0, 3.0),
        };
        let s = ScalarWf {
            value: C::new(2.0, 0.0),
            momentum: LorentzVector::new(4.0, 0.0, 0.0, 1.0),
        };
        // VVS1 `Metric(1,2)` rooted at vector leg 1 is a `MetricVout` current on the
        // partner vector V2, with the spectator scalar leg S multiplied in (the `Mul`
        // the rooted tree carries). Both primitives here are the production helpers.
        let out = mul_apply([
            metric_vout(&WaveformSlot::Vector(v2)),
            WaveformSlot::Scalar(s),
        ]);
        // `MetricVout` emits the physical contravariant current `V2^μ`.
        let WaveformSlot::Vector(out) = out else {
            panic!("VVS rooted at a vector leg must produce a contravariant vector current");
        };

        // +i × ALOHA VVS1P1N_1 (coupling stripped): g·V2 · S.value = V2^μ · S.value
        let sv = s.value;
        let expect = [
            v2.eps.component(0) * sv,
            v2.eps.component(1) * sv,
            v2.eps.component(2) * sv,
            v2.eps.component(3) * sv,
        ];
        for (mu, &exp) in expect.iter().enumerate() {
            let got = out.eps.component(mu);
            assert!(
                (got - exp).norm() < 1e-12,
                "component {mu}: got {got:?}, ALOHA expects {exp:?}",
            );
        }
        // Momentum is conserved through the vertex: q = p_V2 + p_S.
        assert_eq!(out.momentum, v2.momentum + s.momentum);
    }

    /// Harness smoke test: drive a real (private) eval kernel through the `prop_harness`
    /// toolbox, proving the generators + `check_agree` driver wire up to the production
    /// kernels. `kernel::metric` realises the symmetric bilinear `g_{μν} V^μ W^ν`, so
    /// contracting two random vectors in either order must agree; this exercises the vector
    /// generators, the driver, and the scalar comparison path. It is a harness self-check,
    /// not a kernel-equivalence certificate.
    #[test]
    fn prop_harness_drives_metric_symmetry() {
        use crate::helas::eval::prop_harness::{check_agree, rand_vector};
        check_agree(
            256,
            0xC0FFEE,
            1e-11,
            |rng| vec![rand_vector(rng), rand_vector(rng)],
            |c| metric(&c[0], &c[1]),
            |c| metric(&c[1], &c[0]),
        );
    }

    /// Cross-check the s-channel FFV current and amplitude — evaluated through the
    /// production `run_forward` path — against the `jioxxx`/`iovxxx` reference routines.
    ///
    /// Hand-built single diagrams (e⁺e⁻ → boson* [→ μ⁺μ⁻]) are assembled from `EvalNode`s
    /// and run through the same lower → fold → `run_forward` runtime used for real
    /// amplitudes. Rooting at the propagator returns the dressed s-channel current, which
    /// must equal `jioxxx` for every FFV structure (vector / left / left+2·right) and both
    /// the photon and Z propagators. For the unambiguous vector coupling (FFV1) the full
    /// μ⁺μ⁻ amplitude is also cross-checked against `iovxxx(·, jioxxx(·))`.
    #[test]
    fn test_eval_jioxxx() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());

        // This doesn't matter so much, it's pure imaginary and just scales the lorentz structure
        let coupling_id = model.coupling_id("GC_3").unwrap();

        // FFV1 is L+R, FFV2 is L, FFV4 is L+2R
        let coups = vec![("FFV1", 1.0), ("FFV2", 0.0), ("FFV4", 2.0)];
        let props = vec!["a", "Z"];
        for ((coup_str, gr_fact), prop_name) in iproduct!(coups, props) {
            let lorentz_id = model.lorentz_id(coup_str).unwrap();
            let gc = evaluated.coupling(coupling_id);
            let gc = [gc.im, gr_fact * gc.im];

            let inpart_id = model.particle_id("e+").unwrap();
            let inpart_p_id = model.particle_id("e-").unwrap();
            let m_in = evaluated.mass(inpart_id);

            let outpart_id = model.particle_id("mu+").unwrap();
            let outpart_p_id = model.particle_id("mu-").unwrap();
            let m_out = evaluated.mass(outpart_id);

            let prop_id = model.particle_id(prop_name).unwrap();
            let mprop = evaluated.mass(prop_id);
            let wprop = evaluated.width(prop_id);

            let sqrts = 1.0;
            let p3_in = (sqrts * sqrts / 4.0 - m_in * m_in).sqrt();
            let p_in_m = LorentzVector::from_pxpypzmass(0.0, 0.0, -p3_in, m_in);
            let p_in_p = LorentzVector::from_pxpypzmass(0.0, 0.0, p3_in, m_in);
            let p3_out = (sqrts * sqrts / 4.0 - m_out * m_out).sqrt();
            let costheta = -0.9_f64;
            let sintheta = (1.0 - costheta * costheta).sqrt();
            let p_out_m =
                LorentzVector::from_pxpypzmass(p3_out * sintheta, 0.0, p3_out * costheta, m_out);
            let p_out_p =
                LorentzVector::from_pxpypzmass(-p3_out * sintheta, 0.0, -p3_out * costheta, m_out);

            // Set up runtime evaluator data
            let leg1_info = ExtLegInfo {
                leg_idx: 0,
                id: inpart_id,
                spin: 2,
                charge: Charge::Particle,
                incoming: true,
            };
            let leg2_info = ExtLegInfo {
                leg_idx: 1,
                id: inpart_p_id,
                spin: 2,
                charge: Charge::Antiparticle,
                incoming: true,
            };
            let leg3_info = ExtLegInfo {
                leg_idx: 2,
                id: outpart_id,
                spin: 2,
                charge: Charge::Particle,
                incoming: false,
            };
            let leg4_info = ExtLegInfo {
                leg_idx: 3,
                id: outpart_p_id,
                spin: 2,
                charge: Charge::Antiparticle,
                incoming: false,
            };
            let vertex_info = VertexInfo {
                terms: vec![VertexTerm::from_ufo(
                    &model,
                    lorentz_id,
                    &no_color(),
                    coupling_id,
                    Some(2),
                    &[],
                )
                .unwrap()],
            };
            let prop_info = PropInfo {
                id: prop_id,
                t_channel: false,
            };
            let amp_info = VertexInfo {
                terms: vec![VertexTerm::from_ufo(
                    &model,
                    lorentz_id,
                    &no_color(),
                    coupling_id,
                    None,
                    &[],
                )
                .unwrap()],
            };

            // Single s-channel current sub-diagram e⁺e⁻ → (FFV) → boson*: the two
            // externals feed the off-shell current (rooted at the vector leg), and the
            // propagator dresses it. Rooting at the propagator makes `run_forward` return
            // the dressed current itself (a vector), so we read it straight from the
            // production pass. Children reference earlier nodes by index.
            let current_diagram = DiagramEval::from_nodes(
                2,
                vec![
                    EvalNode::External(leg1_info.clone()),
                    EvalNode::External(leg2_info.clone()),
                    EvalNode::OffShellCurrent {
                        info: vertex_info.clone(),
                        adjoint: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: prop_info.clone(),
                        adjoint: None,
                        child: EvalNodeId::new(2),
                    },
                ],
            );
            // The full diagram extends it with the μ⁺μ⁻ sink contraction (a scalar M).
            let amp_diagram = DiagramEval::from_nodes(
                4,
                vec![
                    EvalNode::External(leg1_info),
                    EvalNode::External(leg2_info),
                    EvalNode::OffShellCurrent {
                        info: vertex_info,
                        adjoint: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: prop_info,
                        adjoint: None,
                        child: EvalNodeId::new(2),
                    },
                    EvalNode::External(leg3_info),
                    EvalNode::External(leg4_info),
                    EvalNode::ContractAmplitude {
                        info: amp_info,
                        children: vec![EvalNodeId::new(4), EvalNodeId::new(5), EvalNodeId::new(3)],
                    },
                ],
            );

            let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
            for (hel1, hel2, hel3, hel4) in iproduct!(hels, hels, hels, hels) {
                // Physical adjoint (per the leg charge labels): leg1 (Particle, in) and
                // leg4 (Antiparticle, out) are kets; leg2 (Antiparticle, in) and
                // leg3 (Particle, out) are bras. The reference s-channel current is
                // jioxxx(fo=leg2 bra, fi=leg1 ket); the sink is iovxxx.
                let fi_em = InDiracWf::from_momentum(p_in_m, m_in, hel1, Charge::Particle);
                let fo_ep = OutDiracWf::from_momentum(p_in_p, m_in, hel2, Charge::Antiparticle);
                let v_gamma_exp = jioxxx(&fo_ep, &fi_em, gc, mprop, wprop);

                // The dressed s-channel current from the production pass must match jioxxx
                // exactly (value + routed momentum jmom = fo.p − fi.p), for every FFV
                // structure (vector / left / left+2·right) and both propagators.
                let WaveformSlot::Vector(v_gamma) = eval_single_diagram_slot(
                    &current_diagram,
                    &[p_in_m, p_in_p],
                    &[hel1.sign(), hel2.sign()],
                    &evaluated,
                ) else {
                    panic!("s-channel current must evaluate to a vector");
                };
                assert_eq!(
                    v_gamma.momentum, v_gamma_exp.momentum,
                    "current momentum ({coup_str}/{prop_name}, hel {hel1}{hel2})"
                );
                let cdiff: f64 = (v_gamma.eps - v_gamma_exp.eps).bare_norm_sq();
                assert!(
                    cdiff < 1e-8,
                    "current vs jioxxx ({coup_str}/{prop_name}, hel {hel1}{hel2}): diff={cdiff}"
                );

                // The vector (FFV1) coupling has no chirality ambiguity, so the full
                // amplitude reproduces the composed reference iovxxx∘jioxxx up to one
                // global convention factor of −i (the i the routines fold into the
                // amplitude vs. the i the UFO coupling carries at each vertex; it drops
                // out of |M|²). The pure-chiral FFV2/FFV4 sinks use a different HELAS
                // gc=[gl,gr] decomposition, so only their current is cross-checked above;
                // the chiral amplitude sink is covered by the full-process tests
                // (`test_whole_amplitude_equals_diagram_sum_eemumu`, `validate_helas`).
                if coup_str == "FFV1" {
                    let fo_out_m =
                        OutDiracWf::from_momentum(p_out_m, m_out, hel3, Charge::Particle);
                    let fi_out_p =
                        InDiracWf::from_momentum(p_out_p, m_out, hel4, Charge::Antiparticle);
                    let amp_exp = iovxxx(&fo_out_m, &fi_out_p, &v_gamma_exp, gc);

                    let momenta = [p_in_m, p_in_p, p_out_m, p_out_p];
                    let hel_codes = [hel1.sign(), hel2.sign(), hel3.sign(), hel4.sign()];
                    let got = eval_single_diagram(&amp_diagram, &momenta, &hel_codes, &evaluated);

                    let want = amp_exp * -Complex64::i();
                    let diff = (got - want).norm();
                    assert!(
                        diff < 1e-8,
                        "amplitude vs iovxxx∘jioxxx ({coup_str}/{prop_name}, \
                         hel {hel1}{hel2}{hel3}{hel4}): got={got:.6e} want={want:.6e} diff={diff}"
                    );
                }
            }
        }
    }

    /// Cross-check the production *combined* SM Z off-shell current — built through
    /// `run_forward` from a two-term (FFV2 ⊕ FFV4) vertex — against the ALOHA
    /// `FFV2_4_3` reference routine.
    ///
    /// `FFV2_4_3` adds the pure-left (FFV2, ProjM) and left+2·right (FFV4,
    /// ProjM + 2·ProjP) Lorentz structures with independent couplings — exactly the
    /// SM ℓ̄ℓZ current. Here both structures carry the same coupling `GC_3`, so the
    /// evaluator's combined current equals `jioxxx([2g, 2g])` and ALOHA's
    /// `ffv2_4_3(g, g)`. The two differ only by the global `−i` that ALOHA folds into
    /// each Lorentz structure while vibegraph carries it in the UFO coupling, so the
    /// production current matches `i · ffv2_4_3`. Both the massless photon (no
    /// longitudinal term) and the massive Z (OM3 ≠ 0, exercising the `P3·P3/M²`
    /// longitudinal subtraction) propagators are checked.
    #[test]
    fn test_eval_ffv2_4_3() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());

        let coupling_id = model.coupling_id("GC_3").unwrap();
        let g = evaluated.coupling(coupling_id).im; // real chiral coupling

        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();

        let inpart_id = model.particle_id("e+").unwrap();
        let inpart_p_id = model.particle_id("e-").unwrap();
        let m_in = evaluated.mass(inpart_id);

        let leg1_info = ExtLegInfo {
            leg_idx: 0,
            id: inpart_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: true,
        };
        let leg2_info = ExtLegInfo {
            leg_idx: 1,
            id: inpart_p_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };

        // Two-term vertex: FFV2 (left) ⊕ FFV4 (left + 2·right), both with GC_3.
        let vertex_info = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(&model, ffv2_id, &no_color(), coupling_id, Some(2), &[])
                    .unwrap(),
                VertexTerm::from_ufo(&model, ffv4_id, &no_color(), coupling_id, Some(2), &[])
                    .unwrap(),
            ],
        };

        let i = Complex64::i();
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        // q² = s, so sqrts ≈ MZ drives the internal Z onto its pole — the regime where
        // the longitudinal q^μq^ν/m² subtraction dominates and any spinor-basis or OM3
        // mismatch would show up. sqrts = 1 keeps a deep-off-pole point for contrast.
        for (sqrts, prop_name) in iproduct!([1.0_f64, 91.188], ["a", "Z"]) {
            let p3_in = (sqrts * sqrts / 4.0 - m_in * m_in).sqrt();
            let p_in_m = LorentzVector::from_pxpypzmass(0.0, 0.0, -p3_in, m_in);
            let p_in_p = LorentzVector::from_pxpypzmass(0.0, 0.0, p3_in, m_in);

            let prop_id = model.particle_id(prop_name).unwrap();
            let mprop = evaluated.mass(prop_id);
            let wprop = evaluated.width(prop_id);

            let current_diagram = DiagramEval::from_nodes(
                2,
                vec![
                    EvalNode::External(leg1_info.clone()),
                    EvalNode::External(leg2_info.clone()),
                    EvalNode::OffShellCurrent {
                        info: vertex_info.clone(),
                        adjoint: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo {
                            id: prop_id,
                            t_channel: false,
                        },
                        adjoint: None,
                        child: EvalNodeId::new(2),
                    },
                ],
            );

            for (hel1, hel2) in iproduct!(hels, hels) {
                let fi_em = InDiracWf::from_momentum(p_in_m, m_in, hel1, Charge::Particle);
                let fo_ep = OutDiracWf::from_momentum(p_in_p, m_in, hel2, Charge::Antiparticle);

                // Literal ALOHA FFV2_4_3 reference: FFV2(g) + FFV4(g).
                let aloha = ffv2_4_3(
                    &fi_em,
                    &fo_ep,
                    Complex64::from(g),
                    Complex64::from(g),
                    mprop,
                    wprop,
                );

                // Faithfulness: the transcribed ALOHA current equals our validated
                // `jioxxx` in the equivalent [gL, gR] = [2g, 2g] chiral basis, times −i.
                let jio = jioxxx(&fo_ep, &fi_em, [2.0 * g, 2.0 * g], mprop, wprop);
                for mu in 0..4 {
                    let diff = (aloha.eps.component(mu) - (-i) * jio.eps.component(mu)).norm();
                    assert!(
                        diff < 1e-10,
                        "ffv2_4_3 vs −i·jioxxx (√s={sqrts}, {prop_name}, hel {hel1}{hel2}, μ={mu}): diff={diff}"
                    );
                }

                // Headline: the production combined current (run_forward) matches the
                // ALOHA reference up to the global −i UFO-coupling convention factor.
                let WaveformSlot::Vector(got) = eval_single_diagram_slot(
                    &current_diagram,
                    &[p_in_m, p_in_p],
                    &[hel1.sign(), hel2.sign()],
                    &evaluated,
                ) else {
                    panic!("combined Z current must evaluate to a vector");
                };
                assert_eq!(
                    got.momentum, aloha.momentum,
                    "current momentum (√s={sqrts}, {prop_name}, hel {hel1}{hel2})"
                );
                for mu in 0..4 {
                    let diff = (got.eps.component(mu) - i * aloha.eps.component(mu)).norm();
                    assert!(
                        diff < 1e-8,
                        "eval current vs i·ffv2_4_3 (√s={sqrts}, {prop_name}, hel {hel1}{hel2}, μ={mu}): diff={diff}"
                    );
                }
            }
        }
    }

    /// Ward identity for the off-shell **Z** current: built from a **massless** fermion
    /// pair it must be transverse, `q_μ J^μ = 0`, so the `q^μq^ν/m²` longitudinal piece
    /// of the massive-vector propagator decouples.
    ///
    /// This targets the one continuum-residual mechanism that survives every other test:
    /// the longitudinal Z mode on the massless spine. Unlike the external-photon Ward
    /// tests (`test_ward_identity_full_amplitude_*`), which only constrain the conserved
    /// **vector** current, this uses the real `ℓ̄ℓZ` couplings (FFV2·GC_50 + FFV4·GC_59,
    /// `gL ≠ gR`) so the current carries a genuine **axial** part — the parity-odd piece
    /// that, if its conservation were broken on the massless line, would leave a residual
    /// longitudinal contribution and reweight the L/R (parity-conjugate) helicities. The
    /// axial current's divergence is `∝ 2m·(pseudoscalar)`, so transversality is exact
    /// only for massless fermions; the contraction is checked at the very `q²/m_Z²`
    /// (`√s = m_Z`) where the longitudinal numerator is largest.
    #[test]
    fn test_longitudinal_z_current_transverse_for_massless_fermions() {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());

        // Real ℓ̄ℓZ vertex (SM V_107): FFV2·GC_50 (pure left) ⊕ FFV4·GC_59 (left+2·right),
        // i.e. gL = GC_50+GC_59, gR = 2·GC_59 — a parity-violating (gL ≠ gR) current.
        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        // Sanity: this is genuinely chiral (the axial part is non-trivial).
        assert_ne!(
            evaluated.coupling(gc50),
            evaluated.coupling(gc59),
            "test needs gL ≠ gR to exercise the axial current"
        );

        let inpart_id = model.particle_id("e+").unwrap();
        let inpart_p_id = model.particle_id("e-").unwrap();
        let m_in = evaluated.mass(inpart_id);
        assert_eq!(
            m_in, 0.0,
            "Ward identity requires massless producing fermions"
        );

        let leg1_info = ExtLegInfo {
            leg_idx: 0,
            id: inpart_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: true,
        };
        let leg2_info = ExtLegInfo {
            leg_idx: 1,
            id: inpart_p_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };

        let vertex_info = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(&model, ffv2_id, &no_color(), gc50, Some(2), &[]).unwrap(),
                VertexTerm::from_ufo(&model, ffv4_id, &no_color(), gc59, Some(2), &[]).unwrap(),
            ],
        };

        let z_id = model.particle_id("Z").unwrap();
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        // √s = m_Z drives q² to the pole region, maximising the longitudinal numerator.
        for sqrts in [1.0_f64, 91.1876] {
            let p3 = sqrts / 2.0; // massless ⇒ |p| = √s/2
            let p_in_m = LorentzVector::from_pxpypzmass(0.0, 0.0, -p3, 0.0);
            let p_in_p = LorentzVector::from_pxpypzmass(0.0, 0.0, p3, 0.0);

            let current_diagram = DiagramEval::from_nodes(
                2,
                vec![
                    EvalNode::External(leg1_info.clone()),
                    EvalNode::External(leg2_info.clone()),
                    EvalNode::OffShellCurrent {
                        info: vertex_info.clone(),
                        adjoint: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo {
                            id: z_id,
                            t_channel: false,
                        },
                        adjoint: None,
                        child: EvalNodeId::new(2),
                    },
                ],
            );

            // Track the largest current over helicities so transversality is not
            // vacuously satisfied (the chiral coupling kills the equal-helicity combos).
            let mut max_jnorm = 0.0_f64;
            for (hel1, hel2) in iproduct!(hels, hels) {
                let WaveformSlot::Vector(j) = eval_single_diagram_slot(
                    &current_diagram,
                    &[p_in_m, p_in_p],
                    &[hel1.sign(), hel2.sign()],
                    &evaluated,
                ) else {
                    panic!("Z current must evaluate to a vector");
                };

                // Minkowski contraction q·J = q⁰J⁰ − q⃗·J⃗ with the current's own momentum.
                let q = j.momentum;
                let qdotj = Complex64::from(q.e()) * j.eps.component(0)
                    - Complex64::from(q.px()) * j.eps.component(1)
                    - Complex64::from(q.py()) * j.eps.component(2)
                    - Complex64::from(q.pz()) * j.eps.component(3);

                let jnorm = (0..4)
                    .map(|k| j.eps.component(k).norm_sqr())
                    .sum::<f64>()
                    .sqrt();
                let qnorm =
                    (q.e() * q.e() + q.px() * q.px() + q.py() * q.py() + q.pz() * q.pz()).sqrt();
                max_jnorm = max_jnorm.max(jnorm);

                // q·J must vanish relative to |q||J| (absolute floor covers the
                // equal-helicity combos where the current itself is zero).
                assert!(
                    qdotj.norm() < 1e-9 * qnorm * jnorm + 1e-12,
                    "longitudinal Z fails to decouple for massless fermions \
                     (√s={sqrts}, hel {hel1}{hel2}): q·J={qdotj} vs |q||J|={}",
                    qnorm * jnorm
                );
            }
            assert!(
                max_jnorm > 1e-6,
                "Z current vacuously zero at all helicities (√s={sqrts})"
            );
        }
    }

    /// Cross-check the production off-shell fermion current (`off_shell_fermion_current`
    /// + `propagate_core`) against the `fvixxx`/`fvoxxx` reference routines.
    ///
    /// The current follows the input fermion's adjoint: seeding it from a ket
    /// fermion is `fvixxx`; from a bra fermion is `fvoxxx`. The runtime
    /// applies the bare γ^μ vertex structure and the Dirac propagator as two steps, so
    /// we compare against the reference (which folds both in) with a unit coupling. As
    /// in `test_eval_jioxxx`, the propagator carries the routed momentum unchanged.
    #[test]
    fn test_eval_off_shell_fermion_vs_fvixxx() {
        use crate::helas::vertex::{fvixxx, fvoxxx};

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());

        // Off-shell fermion line propagates an electron.
        let prop_id = model.particle_id("e-").unwrap();
        let mass = evaluated.mass(prop_id);
        let width = evaluated.width(prop_id);

        // UFO convention: coupling includes i. fvixxx/fvoxxx fold the vertex factor in,
        // so we cross-check the bare structure + propagator at unit coupling [1, 1].
        let g = Complex64::new(0.0, 1.0);

        // Generic (unphysical) vector input — any ε works for an impl cross-check.
        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
            // ── fvixxx: off-shell current seeded from the ket fermion ──
            let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);
            let vertex =
                off_shell_fermion_current(&WaveformSlot::Vector(v), &WaveformSlot::FermionIn(fi));
            let WaveformSlot::FermionIn(got) = propagate_core(&vertex, mass, width) else {
                panic!("expected ket fermion from propagation");
            };
            let want = fvixxx(&fi, &v, [g.im, g.im], mass, width);
            // The fermion propagator carries the accumulated momentum unchanged
            // (no flip), matching fvixxx's `q = fi.p + v.p`. Production carries the
            // Feynman-propagator i that the HELAS transcription leaves to the vertex
            // routine, hence the relative factor i.
            assert_eq!(
                got.momentum, want.momentum,
                "fvixxx momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor * g).bare_norm_sq();
            assert!(
                diff < 1e-10,
                "off-shell current vs i·fvixxx diff={diff} (hel {hel}, {charge:?})"
            );

            // ── fvoxxx: off-shell current seeded from the bra fermion ──
            // The current follows the input fermion's adjoint, so the input slot must
            // itself be bra (a bra) to produce a bra current.
            let fo = fi.to_outgoing();
            let vertex =
                off_shell_fermion_current(&WaveformSlot::Vector(v), &WaveformSlot::FermionOut(fo));
            let WaveformSlot::FermionOut(got) = propagate_core(&vertex, mass, width) else {
                panic!("expected bra fermion from propagation");
            };
            let want = fvoxxx(&fo, &v, [g.im, g.im], mass, width);
            assert_eq!(
                got.momentum, want.momentum,
                "fvoxxx momentum (hel {hel}, {charge:?})"
            );
            let diff: f64 = (got.spinor - want.spinor * g).bare_norm_sq();
            assert!(
                diff < 1e-10,
                "off-shell current vs i·fvoxxx diff={diff} (hel {hel}, {charge:?})"
            );
        }
    }

    /// Cross-check the production *chiral* off-shell fermion current — the path an
    /// e-line uses when it absorbs an internal **Z** (FFV2/FFV4, gL≠gR) — against the
    /// independent ALOHA `FFV2_2` routine.
    ///
    /// This is the one Z-specific fermion path never validated before: SESSION 6b's
    /// chain check used a pure-vector `γ q̸ γ` (no projector), and
    /// `test_eval_off_shell_fermion_vs_fvixxx` uses the vector coupling `gc=[g,g]`
    /// (P_L+P_R, projector-insensitive). The per-diagram matcher shows each internal Z
    /// injects a ~5% helicity-dependent error while photons (vector current) are exact,
    /// pointing straight at the chiral fermion current.
    ///
    /// The production tree for `Gamma(3,2,-1)·ProjM(-1,1)` rooted at the output fermion
    /// is `Propagate ∘ off_shell_fermion_current ∘ chiral_project(Left)`. It must equal
    /// `i·fvixxx([1,0])` (self-consistency; the i is the Feynman-propagator phase) and
    /// `−1 ·` ALOHA `FFV2_2` (the −1 = that i × the −i-per-Lorentz-structure ALOHA
    /// folds into its coupling).
    #[test]
    fn test_chiral_off_shell_fermion_vs_ffv2_2() {
        use crate::helas::vertex::{ffv2_2, ffv4_2, fvixxx};

        // Generic vector input (transverse + longitudinal parts) — any ε exercises the
        // linear map; an internal-Z current is just one such ε.
        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let i = Complex64::i();
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        // Massless (electron line) and massive (exercises the FFV2_2 M2 terms F2(5,6)).
        for (mass, width) in [(0.0_f64, 0.0_f64), (0.106, 0.0)] {
            let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
            for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
                let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);

                // Production composition for the chiral (ProjM) fermion current.
                let projected = chiral_project(&WaveformSlot::FermionIn(fi), Chirality::Left);
                let vertex = off_shell_fermion_current(&WaveformSlot::Vector(v), &projected);
                let WaveformSlot::FermionIn(got) = propagate_core(&vertex, mass, width) else {
                    panic!("expected ket fermion from chiral propagation");
                };

                // (a) Self-consistency: i × our pure-left fvixxx helper (the i is the
                // Feynman-propagator phase the production chain carries; the HELAS
                // transcription leaves it to the vertex routine).
                let fvi = fvixxx(&fi, &v, [1.0, 0.0], mass, width);
                assert_eq!(got.momentum, fvi.momentum);
                let d_self: f64 = (got.spinor - fvi.spinor * i).bare_norm_sq();
                assert!(
                    d_self < 1e-10,
                    "chiral current vs i·fvixxx[1,0] (m={mass}, {hel}, {charge:?}): diff={d_self}"
                );

                // (b) Independent ALOHA FFV2_2 (the decisive check), up to the global
                // −1 (= i·i: the propagator i above × the −i-per-Lorentz-structure
                // ALOHA folds into its coupling while vibegraph keeps it in the UFO
                // coupling value).
                let aloha = ffv2_2(&fi, &v, Complex64::from(1.0), mass, width);
                assert_eq!(
                    got.momentum, aloha.momentum,
                    "ffv2_2 momentum (m={mass}, {hel}, {charge:?})"
                );
                let d_aloha: f64 = (got.spinor + aloha.spinor).bare_norm_sq();
                assert!(
                    d_aloha < 1e-10,
                    "chiral current vs -ffv2_2 (m={mass}, {hel}, {charge:?}): diff={d_aloha}"
                );

                // (c) Full Z fermion current FFV4 = P_L + 2·P_R (exercises the ProjP /
                // right path and its coefficient): the tree sums the two projected
                // slashes BEFORE the shared propagator.
                let WaveformSlot::FermionIn(left) = off_shell_fermion_current(
                    &WaveformSlot::Vector(v),
                    &chiral_project(&WaveformSlot::FermionIn(fi), Chirality::Left),
                ) else {
                    unreachable!()
                };
                let WaveformSlot::FermionIn(right) = off_shell_fermion_current(
                    &WaveformSlot::Vector(v),
                    &chiral_project(&WaveformSlot::FermionIn(fi), Chirality::Right),
                ) else {
                    unreachable!()
                };
                let summed = WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    left.spinor + right.spinor * 2.0,
                    left.momentum,
                ));
                let WaveformSlot::FermionIn(got4) = propagate_core(&summed, mass, width) else {
                    unreachable!()
                };
                let aloha4 = ffv4_2(&fi, &v, Complex64::from(1.0), mass, width);
                let d4: f64 = (got4.spinor + aloha4.spinor).bare_norm_sq();
                assert!(
                    d4 < 1e-10,
                    "FFV4 chiral current vs -ffv4_2 (m={mass}, {hel}, {charge:?}): diff={d4}"
                );
            }
        }
    }

    /// Validate **both leg rootings** of the production *chiral* off-shell fermion
    /// current against a textbook Dirac-matrix reconstruction (adjoint-IN / ket input).
    ///
    /// An FFV2/FFV4 vertex `ψ̄ γ^μ P ψ` rooted at a fermion output leg can land the
    /// projector on either side of the gamma:
    ///   • **leg-0** (`Some(0)`, `ProjM`/column leg): `Propagate ∘ chiral_project ∘
    ///     off_shell_fermion_current` = `P·ε̸·ψ` (projector AFTER the gamma);
    ///   • **leg-1** (`Some(1)`, gamma's row leg): `Propagate ∘ off_shell_fermion_current
    ///     ∘ chiral_project` = `ε̸·P·ψ` (projector BEFORE the gamma).
    /// Since `γ^μ P_L = P_R γ^μ`, the two carry OPPOSITE chirality — genuinely distinct
    /// code paths that ee→μμ (vector output) and the leg-2 tests never exercise. The
    /// production e-spine uses **leg-1**; this test pins the **adjoint-IN (ket)**
    /// realization of *both* rootings against the textbook (the adjoint-OUT realization
    /// is pinned by `test_chiral_off_shell_fermion_flowout_vs_textbook`).
    ///
    /// Reference: `S(q)·P·ε̸·ψ` (leg-0) and `S(q)·ε̸·P·ψ` (leg-1), Dirac propagator
    /// `S(q) = (q̸ + m)/(q² − m²)`, `q = ψ.p − v.p`, built from explicit Weyl-basis γ
    /// matrices independent of the evaluator's representation. The evaluator's
    /// Feynman-propagator phase `−i` from `propagate_core` is folded into the
    /// reference. FFV2 uses `P = P_L`; FFV4 uses `P = P_L + 2P_R`.
    #[test]
    fn test_chiral_off_shell_fermion_espine_vs_textbook() {
        // Weyl basis γ^μ = [[0,σ^μ],[σ̄^μ,0]], σ^μ=(I,σ_i), σ̄^μ=(I,−σ_i); metric (+,−,−,−).
        type M4 = [[Complex64; 4]; 4];
        let z = Complex64::new(0.0, 0.0);
        let o = Complex64::new(1.0, 0.0);
        let ii = Complex64::new(0.0, 1.0);
        let g0: M4 = [[z, z, o, z], [z, z, z, o], [o, z, z, z], [z, o, z, z]];
        let g1: M4 = [[z, z, z, o], [z, z, o, z], [z, -o, z, z], [-o, z, z, z]];
        let g2: M4 = [[z, z, z, -ii], [z, z, ii, z], [z, ii, z, z], [-ii, z, z, z]];
        let g3: M4 = [[z, z, o, z], [z, z, z, -o], [-o, z, z, z], [z, o, z, z]];
        let matvec = |m: &M4, x: &[Complex64; 4]| -> [Complex64; 4] {
            core::array::from_fn(|r| (0..4).map(|c| m[r][c] * x[c]).sum())
        };
        let add = |a: [Complex64; 4], b: [Complex64; 4]| -> [Complex64; 4] {
            core::array::from_fn(|k| a[k] + b[k])
        };
        let scale = |s: Complex64, a: [Complex64; 4]| -> [Complex64; 4] {
            core::array::from_fn(|k| s * a[k])
        };
        // Covariant slash v̸ = γ^0 v^0 − γ^1 v^1 − γ^2 v^2 − γ^3 v^3.
        let slash = |v: &[Complex64; 4], x: &[Complex64; 4]| -> [Complex64; 4] {
            let mut r = scale(v[0], matvec(&g0, x));
            r = add(r, scale(-v[1], matvec(&g1, x)));
            r = add(r, scale(-v[2], matvec(&g2, x)));
            add(r, scale(-v[3], matvec(&g3, x)))
        };

        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let eps = core::array::from_fn(|k| v.eps.component(k));
        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (mass, width) in [(0.0_f64, 0.0_f64), (0.106, 0.0)] {
            let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
            for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
                let fi = InDiracWf::from_momentum(p_f, mass, hel, charge);
                let psi: [Complex64; 4] = core::array::from_fn(|k| fi.spinor.component(k));
                let q = fi.momentum - v.momentum;
                let qvec = [
                    Complex64::from(q.e()),
                    Complex64::from(q.px()),
                    Complex64::from(q.py()),
                    Complex64::from(q.pz()),
                ];
                let denom = Complex64::from(q.m2() - mass * mass);

                // Textbook S(q)·P·ε̸·ψ for P = P_L (FFV2) and P = P_L + 2P_R (FFV4),
                // with the evaluator's overall −i from propagate_core folded in.
                let textbook = |pl: Complex64, pr: Complex64| -> [Complex64; 4] {
                    let eps_psi = slash(&eps, &psi);
                    // P · ε̸ψ with chiral weights (P_L keeps [0,1], P_R keeps [2,3]).
                    let projected = [
                        pl * eps_psi[0],
                        pl * eps_psi[1],
                        pr * eps_psi[2],
                        pr * eps_psi[3],
                    ];
                    // (q̸ + m)/(q²−m²), then ×(−i).
                    let qslash = slash(&qvec, &projected);
                    let massterm = scale(Complex64::from(mass), projected);
                    scale(-ii / denom, add(qslash, massterm))
                };

                // ── FFV2 e-spine current: ProjM(ε̸·ψ) propagated ─────────────────
                let curr = off_shell_fermion_current(
                    &WaveformSlot::Vector(v),
                    &WaveformSlot::FermionIn(fi),
                );
                let WaveformSlot::FermionIn(got2) =
                    propagate_core(&chiral_project(&curr, Chirality::Left), mass, width)
                else {
                    panic!("expected ket fermion from chiral propagation");
                };
                let want2 = textbook(o, z);
                for k in 0..4 {
                    let d = (got2.spinor.component(k) - want2[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV2 e-spine vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }

                // ── FFV4 e-spine current: ProjM(ε̸ψ) + 2·ProjP(ε̸ψ) propagated ───
                let mk = |chi| {
                    let WaveformSlot::FermionIn(c) = chiral_project(
                        &off_shell_fermion_current(
                            &WaveformSlot::Vector(v),
                            &WaveformSlot::FermionIn(fi),
                        ),
                        chi,
                    ) else {
                        unreachable!()
                    };
                    c
                };
                let left = mk(Chirality::Left);
                let right = mk(Chirality::Right);
                let summed = WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    left.spinor + right.spinor * 2.0,
                    left.momentum,
                ));
                let WaveformSlot::FermionIn(got4) = propagate_core(&summed, mass, width) else {
                    unreachable!()
                };
                let want4 = textbook(o, Complex64::new(2.0, 0.0));
                for k in 0..4 {
                    let d = (got4.spinor.component(k) - want4[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV4 e-spine vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }

                // ══ The OTHER rooting: leg-1 (`Some(1)`), projector BEFORE the gamma ══
                // The production e+-spine Z absorption roots the FFV2/FFV4 vertex at the
                // gamma's row/output leg, giving `GammaXout(V, ProjM(F))` = `ε̸·P_χ·ψ` —
                // the mirror of the leg-0 `P_χ·ε̸·ψ` above. Since `γ^μ P_L = P_R γ^μ` the
                // two rootings carry OPPOSITE chirality, so this is a genuinely distinct
                // current. Here the input is a adjoint-IN ket, so this exercises the
                // `fvixxx`/`GammaIout` realization, which equals ALOHA FFV2_2 (see
                // `test_chiral_off_shell_fermion_vs_ffv2_2`); the adjoint-OUT (bra)
                // realization is pinned by `test_chiral_off_shell_fermion_flowout_vs_textbook`
                // and, end-to-end vs MadGraph, by `test_espine_eline_z_absorption_ratio_vs_mg`.
                let textbook_proj_first = |pl: Complex64, pr: Complex64| -> [Complex64; 4] {
                    // ε̸ · (P_L+P_R-weighted ψ), then (q̸+m)/(q²−m²), then ×(−i).
                    let projected = [pl * psi[0], pl * psi[1], pr * psi[2], pr * psi[3]];
                    let eps_proj = slash(&eps, &projected);
                    let qslash = slash(&qvec, &eps_proj);
                    let massterm = scale(Complex64::from(mass), eps_proj);
                    scale(-ii / denom, add(qslash, massterm))
                };

                // FFV2 leg-1: ε̸·P_L·ψ propagated.
                let WaveformSlot::FermionIn(g2b) = propagate_core(
                    &off_shell_fermion_current(
                        &WaveformSlot::Vector(v),
                        &chiral_project(&WaveformSlot::FermionIn(fi), Chirality::Left),
                    ),
                    mass,
                    width,
                ) else {
                    panic!("expected ket fermion");
                };
                let want2b = textbook_proj_first(o, z);
                for k in 0..4 {
                    let d = (g2b.spinor.component(k) - want2b[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV2 leg-1 (ε̸·P_L·ψ) vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }

                // FFV4 leg-1: ε̸·(P_L+2P_R)·ψ propagated — project the INPUT first, slash
                // after (mirror of `mk`, which projects after the slash for leg-0).
                let mk1 = |chi| {
                    let WaveformSlot::FermionIn(c) = off_shell_fermion_current(
                        &WaveformSlot::Vector(v),
                        &chiral_project(&WaveformSlot::FermionIn(fi), chi),
                    ) else {
                        unreachable!()
                    };
                    c
                };
                let l1 = mk1(Chirality::Left);
                let r1 = mk1(Chirality::Right);
                let summed1 = WaveformSlot::FermionIn(InDiracWf::from_spinor(
                    l1.spinor + r1.spinor * 2.0,
                    l1.momentum,
                ));
                let WaveformSlot::FermionIn(g4b) = propagate_core(&summed1, mass, width) else {
                    unreachable!()
                };
                let want4b = textbook_proj_first(o, Complex64::new(2.0, 0.0));
                for k in 0..4 {
                    let d = (g4b.spinor.component(k) - want4b[k]).norm();
                    assert!(
                        d < 1e-10,
                        "FFV4 leg-1 (ε̸·(P_L+2P_R)·ψ) vs textbook (m={mass}, {hel}, {charge:?}, comp {k}): {d}"
                    );
                }
            }
        }
    }

    /// Pin the **adjoint-OUT (bra)** leg-1 chiral-absorption primitives: `chiral_project`
    /// on a bra acts ADJACENT to the bra, so the raw composition
    /// `off_shell_fermion_current ∘ chiral_project` gives `ψ̄·P_χ·ε̸·S` — which is NOT
    /// the physical vertex action `ψ̄·ε̸·P_χ·S` (they differ by `ψ̄·ε̸·γ5·S`, a
    /// polarisation-dependent term; for a ket input the same composition IS physical,
    /// see the leg-1 half of `test_chiral_off_shell_fermion_espine_vs_textbook`).
    /// The physical vertex is recovered one level up: the adjoint-driven rooting
    /// (`root_lorentz.rs`) hands a bra-side leg-1 absorption the conjugated projector
    /// (`P_χ → P_χ̄`), and `P_χ̄·ε̸ = ε̸·P_χ` lands it on the physical side. That full
    /// composition is pinned end-to-end vs MadGraph by
    /// `test_espine_eline_z_absorption_ratio_vs_mg` (this bra/ket asymmetry, applied
    /// unconjugated, was the historical hel-42 0.6403 continuum bug).
    ///
    /// This test reconstructs a storage-independent **bra** textbook via the bilinear
    /// scalar `R_out · χ_ref` (a bra spinor dotted with a ket gives the Lorentz
    /// scalar `ψ̄ … χ`), validated FIRST on the chirality-blind photon. It then pins the
    /// primitive composition to `ψ̄·P_χ·ε̸·S` and asserts it differs from `ψ̄·ε̸·P_χ·S`,
    /// so any silent change to which side the bra projector acts on is caught here.
    #[test]
    fn test_chiral_off_shell_fermion_flowout_vs_textbook() {
        type M4 = [[Complex64; 4]; 4];
        let z = Complex64::new(0.0, 0.0);
        let o = Complex64::new(1.0, 0.0);
        let ii = Complex64::new(0.0, 1.0);
        let g0: M4 = [[z, z, o, z], [z, z, z, o], [o, z, z, z], [z, o, z, z]];
        let g1: M4 = [[z, z, z, o], [z, z, o, z], [z, -o, z, z], [-o, z, z, z]];
        let g2: M4 = [[z, z, z, -ii], [z, z, ii, z], [z, ii, z, z], [-ii, z, z, z]];
        let g3: M4 = [[z, z, o, z], [z, z, z, -o], [-o, z, z, z], [z, o, z, z]];
        let matmul = |a: &M4, b: &M4| -> M4 {
            core::array::from_fn(|r| {
                core::array::from_fn(|c| (0..4).map(|k| a[r][k] * b[k][c]).sum())
            })
        };
        let matadd = |a: &M4, b: &M4| -> M4 {
            core::array::from_fn(|r| core::array::from_fn(|c| a[r][c] + b[r][c]))
        };
        let smul = |s: Complex64, a: &M4| -> M4 {
            core::array::from_fn(|r| core::array::from_fn(|c| s * a[r][c]))
        };
        let ident: M4 =
            core::array::from_fn(|r| core::array::from_fn(|c| if r == c { o } else { z }));
        // Slash matrix v̸ = γ^0 v0 − γ^1 v1 − γ^2 v2 − γ^3 v3 (contravariant v).
        let slashm = |v: &[Complex64; 4]| -> M4 {
            let mut m = smul(v[0], &g0);
            m = matadd(&m, &smul(-v[1], &g1));
            m = matadd(&m, &smul(-v[2], &g2));
            matadd(&m, &smul(-v[3], &g3))
        };
        let rowmat = |r: &[Complex64; 4], m: &M4| -> [Complex64; 4] {
            core::array::from_fn(|c| (0..4).map(|k| r[k] * m[k][c]).sum())
        };
        let dot = |a: &[Complex64; 4], b: &[Complex64; 4]| -> Complex64 {
            (0..4).map(|k| a[k] * b[k]).sum()
        };

        let v = VectorWf {
            eps: ComplexVector::new([
                Complex64::new(1.0, 0.0),
                Complex64::new(0.5, 0.2),
                Complex64::new(0.3, 0.1),
                Complex64::new(0.4, 0.0),
            ]),
            momentum: LorentzVector::new(50.0, 10.0, 0.0, 20.0),
        };
        let eps: [Complex64; 4] = core::array::from_fn(|k| v.eps.component(k));
        let eslash = slashm(&eps);
        // Arbitrary reference ket (probes all components).
        let chi_ref = [
            Complex64::new(0.7, 0.1),
            Complex64::new(-0.2, 0.4),
            Complex64::new(0.5, -0.3),
            Complex64::new(0.1, 0.6),
        ];
        let proj = |pl: Complex64, pr: Complex64| -> M4 {
            let mut m = ident;
            m[0][0] = pl;
            m[1][1] = pl;
            m[2][2] = pr;
            m[3][3] = pr;
            m
        };

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (mass, width) in [(0.0_f64, 0.0_f64), (0.106, 0.0)] {
            let p_f = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
            for (hel, charge) in iproduct!(hels, [Charge::Particle, Charge::Antiparticle]) {
                let fo = OutDiracWf::from_momentum(p_f, mass, hel, charge);
                let ket = InDiracWf::from_momentum(p_f, mass, hel, charge);
                let psi: [Complex64; 4] = core::array::from_fn(|k| ket.spinor.component(k));
                // Physical bra ψ̄ = ψ† γ⁰  (OutDiracWf::from_momentum == bar(InDiracWf)).
                let psibar: [Complex64; 4] =
                    core::array::from_fn(|j| (0..4).map(|i| psi[i].conj() * g0[i][j]).sum());

                // q = fo.p + v.p (bra current momentum); S(q) with eval's −1.
                let q = fo.momentum + v.momentum;
                let qvec = [
                    Complex64::from(q.e()),
                    Complex64::from(q.px()),
                    Complex64::from(q.py()),
                    Complex64::from(q.pz()),
                ];
                let denom = Complex64::from(q.m2() - mass * mass);
                let sprop = smul(
                    -ii / denom,
                    &matadd(&slashm(&qvec), &smul(Complex64::from(mass), &ident)),
                );

                // Textbook scalar for a bra operator `op` (applied to ψ̄ from the right),
                // propagated, then contracted with the reference ket.
                let textbook = |op: &M4| -> Complex64 {
                    let r = rowmat(&psibar, op);
                    let r = rowmat(&r, &sprop);
                    dot(&r, &chi_ref)
                };
                // Eval bra scalar: dot the produced bra with the reference ket.
                let eval_scalar = |r: &OutDiracWf<f64>| -> Complex64 {
                    let comps: [Complex64; 4] = core::array::from_fn(|k| r.spinor.component(k));
                    dot(&comps, &chi_ref)
                };

                // ── Photon (no projector): validates the bra machinery end-to-end ──
                let WaveformSlot::FermionOut(ph) = propagate_core(
                    &off_shell_fermion_current(
                        &WaveformSlot::Vector(v),
                        &WaveformSlot::FermionOut(fo),
                    ),
                    mass,
                    width,
                ) else {
                    panic!("expected bra fermion");
                };
                let s_ph_eval = eval_scalar(&ph);
                let s_ph_book = textbook(&eslash); // ψ̄·ε̸·S
                let d_ph = (s_ph_eval - s_ph_book).norm();
                assert!(
                    d_ph < 1e-9 * (s_ph_book.norm() + 1.0),
                    "PHOTON bra bra recon failed (m={mass}, {hel}, {charge:?}): \
                     eval={s_ph_eval:.5e} book={s_ph_book:.5e} d={d_ph:.2e}"
                );

                // Project-bra-first builder for the eval bra current.
                let eval_flowout = |proj_chi: Option<Chirality>, tworight: bool| -> Complex64 {
                    // FFV2: single P_L term. FFV4: P_L + 2·P_R.
                    let build = |chi: Chirality| {
                        let WaveformSlot::FermionOut(c) = off_shell_fermion_current(
                            &WaveformSlot::Vector(v),
                            &chiral_project(&WaveformSlot::FermionOut(fo.clone()), chi),
                        ) else {
                            unreachable!()
                        };
                        c
                    };
                    let combined = match (proj_chi, tworight) {
                        (Some(chi), false) => build(chi).spinor, // FFV2: P_L only
                        (None, true) => {
                            build(Chirality::Left).spinor + build(Chirality::Right).spinor * 2.0
                        } // FFV4
                        _ => unreachable!(),
                    };
                    let WaveformSlot::FermionOut(r) = propagate_core(
                        &WaveformSlot::FermionOut(OutDiracWf::from_spinor(
                            combined,
                            fo.momentum + v.momentum,
                        )),
                        mass,
                        width,
                    ) else {
                        unreachable!()
                    };
                    eval_scalar(&r)
                };

                // The eval projects the bra BEFORE slashing: `ψ̄·P_χ·ε̸·S`. Because
                // `P_L·ε̸ = ε̸·P_R`, that is the OPPOSITE-chirality action of the nominal
                // `P_L` vertex on the bra — the precise bra structure, here pinned
                // against the textbook for every helicity/charge/mass.
                // FFV2 (P_L vertex):
                let s_ffv2 = eval_flowout(Some(Chirality::Left), false);
                let book_ffv2 = textbook(&matmul(&proj(o, z), &eslash)); // ψ̄·P_L·ε̸·S
                let d2 = (s_ffv2 - book_ffv2).norm();
                assert!(
                    d2 < 1e-9 * (book_ffv2.norm() + 1.0),
                    "FFV2 bra (m={mass}, {hel}, {charge:?}): eval={s_ffv2:.5e} vs ψ̄·P_L·ε̸·S={book_ffv2:.5e}, d={d2:.2e}"
                );
                // It must NOT equal the gamma-first `ψ̄·ε̸·P_L·S` (off the massless point).
                let book_gammafirst = textbook(&matmul(&eslash, &proj(o, z)));
                if mass == 0.0 && (book_ffv2 - book_gammafirst).norm() > 1e-6 {
                    assert!(
                        (s_ffv2 - book_gammafirst).norm() > 1e-6,
                        "FFV2 bra unexpectedly matched the gamma-first ordering"
                    );
                }

                // FFV4 (P_L + 2P_R vertex):
                let s_ffv4 = eval_flowout(None, true);
                // Eval order: ψ̄·(P_L+2P_R)·ε̸·S.
                let book_ffv4 = textbook(&matmul(
                    &matadd(&proj(o, z), &smul(o + o, &proj(z, o))),
                    &eslash,
                ));
                let d4 = (s_ffv4 - book_ffv4).norm();
                assert!(
                    d4 < 1e-9 * (book_ffv4.norm() + 1.0),
                    "FFV4 bra (m={mass}, {hel}, {charge:?}): eval={s_ffv4:.5e} vs ψ̄·(P_L+2P_R)·ε̸·S={book_ffv4:.5e}, d={d4:.2e}"
                );
            }
        }
    }

    /// Fermion-line reversal: a single fermion line absorbing two vectors must give
    /// the SAME amplitude whether the off-shell current is seeded from the ket end
    /// (`fvixxx`) or the bra end (`fvoxxx`). This is the consistency MadGraph relies
    /// on — it builds the e-line spine from the e⁺ (bra) end via FFV1_1, while
    /// vibegraph always seeds from the FermionIn (ket) end. If these disagree by a
    /// sign, every incoming-spine diagram (e-line off-shell) gets a spurious −1.
    #[test]
    fn test_fermion_line_reversal_ket_vs_bra() {
        use crate::helas::vertex::{fvixxx, fvoxxx};

        let mass = 0.0_f64; // massless internal fermion (the continuum case)
        let width = 0.0_f64;
        let gc = [1.0_f64, 2.0];

        // Cover all charge/helicity combinations: the e-line spine has an
        // ANTIparticle bra (e⁺) while μ/τ-line spines have an ANTIparticle ket
        // (μ⁺/τ⁺). The reversal identity is algebraic, so it must hold for every
        // combination; a break on a specific charge isolates the continuum −1.
        for (qi, qo, hi, ho) in iproduct!(
            [Charge::Particle, Charge::Antiparticle],
            [Charge::Particle, Charge::Antiparticle],
            [SpinorHelicity::Down, SpinorHelicity::Up],
            [SpinorHelicity::Down, SpinorHelicity::Up]
        ) {
            let fi_spinor = InDiracWf::from_momentum(
                LorentzVector::from_pxpypzmass(12.0, -7.0, 3.0, 0.0),
                0.0,
                hi,
                qi,
            )
            .spinor;
            let fo_spinor = OutDiracWf::from_momentum(
                LorentzVector::from_pxpypzmass(-4.0, 9.0, -5.0, 0.0),
                0.0,
                ho,
                qo,
            )
            .spinor;

            let v1 = VectorWf {
                eps: ComplexVector::new([
                    Complex64::new(1.0, 0.2),
                    Complex64::new(0.5, -0.1),
                    Complex64::new(0.3, 0.4),
                    Complex64::new(-0.2, 0.0),
                ]),
                momentum: LorentzVector::new(40.0, 10.0, -5.0, 20.0),
            };
            let v2 = VectorWf {
                eps: ComplexVector::new([
                    Complex64::new(0.7, -0.3),
                    Complex64::new(-0.4, 0.6),
                    Complex64::new(0.2, 0.1),
                    Complex64::new(0.9, -0.2),
                ]),
                momentum: LorentzVector::new(55.0, -15.0, 8.0, -10.0),
            };

            // Momentum conservation along the line: the intermediate momentum seen by
            // fvixxx (fi.p − v1.p) must equal that seen by fvoxxx (fo.p + v2.p).
            let fi_mom = LorentzVector::new(120.0, 5.0, 0.0, 30.0);
            let fo_mom = fi_mom - v1.momentum - v2.momentum;
            let fi = InDiracWf::from_spinor(fi_spinor, fi_mom);
            let fo = OutDiracWf::from_spinor(fo_spinor, fo_mom);

            // A: seed from the ket (fvixxx absorbs v1), amplitude with bra + v2.
            let off_ket = fvixxx(&fi, &v1, gc, mass, width);
            let a = iovxxx(&fo, &off_ket, &v2, gc);

            // B: seed from the bra (fvoxxx absorbs v2), amplitude with ket + v1.
            let off_bra = fvoxxx(&fo, &v2, gc, mass, width);
            let b = iovxxx(&off_bra, &fi, &v1, gc);

            let diff = (a - b).norm();
            assert!(
                diff < 1e-9,
                "fermion line reversal broken (qi={qi:?} qo={qo:?} hi={hi} ho={ho}): \
             ket-build={a:.6e} bra-build={b:.6e} diff={diff:.3e}"
            );
        }
    }

    /// Process-parameterized per-diagram probe (paired with the MG-side matcher
    /// `validation/madgraph/compare_amps.py`), so any failing process gets the
    /// note-12 diagnostic treatment immediately.
    ///
    /// Reads everything from the process's generated reference data:
    /// momenta + MG |M|² from CSV point 0 of `output/<NAME>_amplitude.csv` (whose
    /// `# process:` header is the proc-card line), the param card from
    /// `output/<NAME>/Cards/param_card.dat`.  Dumps the full [diagram × helicity]
    /// complex amplitudes to `output/vibegraph_amps_<NAME>.txt` (match against an
    /// MG AMP-dump probe), prints the coherent |M|² vs the MG reference, and runs
    /// sign-flip experiments — exhaustive over diagram subsets when the diagram
    /// count is small, else one flip per propagator-signature class — reporting any
    /// assignment that collapses the residual (instant diagnosis for sign bugs).
    ///
    /// For a multi-flow process (NCOLOR > 1) it additionally binds the whole
    /// evaluator and dumps the per-flow JAMPs `J_f` (the [`Op::Flows`] root
    /// children, one column per helicity) to `output/vibegraph_jamps_<NAME>.txt`,
    /// matched flow-by-flow against MadGraph's `JAMP()` by `compare_amps.py`.
    ///
    /// Run: VG_PROBE_NAME=ee_to_ee cargo test -p vibegraph-lib --features extended-validation \
    ///        --lib helas::eval::run::tests::probe_process_diagrams -- --ignored --nocapture
    #[test]
    #[ignore]
    #[cfg(feature = "extended-validation")]
    fn probe_process_diagrams() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let name = std::env::var("VG_PROBE_NAME").expect("set VG_PROBE_NAME=<process name>");

        let model = sm_model(SMRestrict::Default);
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let out_dir = std::path::Path::new(&manifest).join("../validation/madgraph/output");

        // CSV: process string from the header, momenta + m2_ref from point 0.
        let csv = std::fs::read_to_string(out_dir.join(format!("{name}_amplitude.csv")))
            .expect("amplitude CSV (run generate-amplitude first)");
        let process_str = csv
            .lines()
            .find_map(|l| l.trim().strip_prefix("# process:"))
            .expect("# process: header")
            .trim()
            .to_owned();
        let n_ext: usize = csv
            .lines()
            .find_map(|l| l.trim().strip_prefix("# n_ext:"))
            .expect("# n_ext: header")
            .trim()
            .parse()
            .unwrap();
        let row: Vec<f64> = csv
            .lines()
            .filter(|l| !l.trim().starts_with('#') && !l.trim().is_empty())
            .nth(1) // skip the column-header row
            .expect("data row")
            .split(',')
            .map(|c| c.trim().parse().unwrap())
            .collect();
        assert_eq!(row.len(), 1 + 4 * n_ext);
        let m2_ref = row[0];
        let p: Vec<LorentzVector<f64>> = (0..n_ext)
            .map(|i| {
                let b = 1 + 4 * i;
                LorentzVector::new(row[b], row[b + 1], row[b + 2], row[b + 3])
            })
            .collect();

        let card = std::fs::read_to_string(out_dir.join(format!("{name}/Cards/param_card.dat")))
            .expect("param_card.dat")
            .parse::<ParamCard>()
            .unwrap();
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

        let opts = ParsingOptions::default();
        let pc = parse_proc_card(&format!("generate {process_str}"), &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let set = &sets[0];
        let asts = compile_diagram_ast(set, &model).unwrap();
        let n = asts.len();
        let evaluator = AmplitudeEvaluator::compile(set, &model).unwrap();
        let combos = evaluator.helicities();
        println!(
            "[{name}] {process_str}: {n} diagrams, {} hel combos",
            combos.len()
        );

        let prop_sig = |ast: &DiagramEval| -> String {
            let names: Vec<String> = ast
                .propagator_particles()
                .map(|id| model.particle(id).name.clone())
                .collect();
            names.join("+")
        };
        for (i, ast) in asts.iter().enumerate() {
            println!(
                "  diagram {i:3}  fermi_sign={:+}  [{}]",
                ast.fermi_sign,
                prop_sig(ast)
            );
        }
        // VG_PROBE_DUMP=3,17: print the full rooted trees of selected diagrams.
        if let Ok(dump) = std::env::var("VG_PROBE_DUMP") {
            for tok in dump.split(',') {
                let i: usize = tok.trim().parse().unwrap();
                println!("--- diagram {i} tree:\n{}", asts[i]);
            }
        }

        // amps[d][hel] via the production forward pass, one diagram at a time.
        let amps: Vec<Vec<C<f64>>> = asts
            .iter()
            .map(|d| {
                let symbolic = lower::lower(std::slice::from_ref(d));
                let f = Folded::build(&symbolic);
                let (cc, cf_pool) = f.pools::<f64>(&evaluated);
                let mut res = Vec::new();
                combos
                    .iter()
                    .map(|hel| {
                        let env = EvalEnv {
                            consts_c: &cc,
                            consts_f: &cf_pool,
                            ext_legs: f.ext_legs(),
                            momenta: &p,
                            helicities: hel,
                            ward_leg: None,
                        };
                        match run_forward_slot(&f, &env, &mut res) {
                            WaveformSlot::Scalar(s) => s.value,
                            WaveformSlot::Empty => C::new(0.0, 0.0),
                            other => panic!("amplitude root is not a scalar: {other:?}"),
                        }
                    })
                    .collect()
            })
            .collect();

        // Full [diagram][helicity] dump for the MG-side per-diagram matcher
        // (validation/madgraph/compare_amps.py). The `#hel` header lists the
        // helicity combos column-by-column so the matcher evaluates MadGraph's
        // AMP() at exactly the same combos in the same order.
        {
            use std::fmt::Write as _;
            let mut s = String::from("#hel");
            for hel in combos {
                let cs: Vec<String> = hel.iter().map(|h| h.to_string()).collect();
                let _ = write!(s, "\t{}", cs.join(","));
            }
            s.push('\n');
            for (i, arow) in amps.iter().enumerate() {
                let _ = write!(s, "{}\t{}", i, prop_sig(&asts[i]));
                for a in arow {
                    let _ = write!(s, "\t{}\t{}", a.re, a.im);
                }
                s.push('\n');
            }
            let out = out_dir.join(format!("vibegraph_amps_{name}.txt"));
            std::fs::write(&out, s).unwrap();
            println!("wrote {}", out.display());
        }

        // Per-flow JAMP dump (multi-flow processes only): bind the whole
        // evaluator and read the Op::Flows root's children `J_f` per helicity,
        // matched flow-by-flow against MadGraph's JAMP() by compare_amps.py.
        // The flow order is the color basis's sorted-key order (matching MG's
        // JAMP order), so a permutation/phase surfacing here is a real finding.
        let n_flows = evaluator.n_flows();
        if n_flows > 1 {
            let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
            let mut js_scratch = bound.scratch_space();
            let mut jamps: Vec<Vec<C<f64>>> = vec![Vec::with_capacity(combos.len()); n_flows];
            for hel in combos {
                let per_flow = bound.run_flows(&p, hel, &mut js_scratch);
                assert_eq!(
                    per_flow.len(),
                    n_flows,
                    "run_flows returned wrong flow count"
                );
                for (f, jf) in per_flow.into_iter().enumerate() {
                    jamps[f].push(jf);
                }
            }
            use std::fmt::Write as _;
            let mut s = String::from("#hel");
            for hel in combos {
                let cs: Vec<String> = hel.iter().map(|h| h.to_string()).collect();
                let _ = write!(s, "\t{}", cs.join(","));
            }
            s.push('\n');
            for (f, jrow) in jamps.iter().enumerate() {
                let _ = write!(s, "{f}\tflow{f}");
                for j in jrow {
                    let _ = write!(s, "\t{}\t{}", j.re, j.im);
                }
                s.push('\n');
            }
            let out = out_dir.join(format!("vibegraph_jamps_{name}.txt"));
            std::fs::write(&out, s).unwrap();
            println!("wrote {} ({n_flows} flows)", out.display());
        }

        // The color-summed |M|² MadGraph reports. For a single flow this is the
        // scalar diagonal factor CF(1,1) times the coherent diagram sum; for
        // several flows it is the full CF-weighted contraction (eval_m2), which
        // the JAMP dump above lets one debug flow-by-flow.
        let cf = {
            let r = evaluator.cf_matrix()[0];
            *r.numer() as f64 / *r.denom() as f64
        };

        // |M|² with a per-diagram sign assignment (subset S of diagrams flipped);
        // the single-flow diagnostic — for one color flow, |M|² = CF(1,1)·Σ_hel
        // |Σ_d amp_d|², so a relative sign error between diagrams shows up as a
        // subset flip that collapses the residual.
        let m2_flipped = |flip: &dyn Fn(usize) -> bool| -> f64 {
            let mut m2 = 0.0;
            for h in 0..combos.len() {
                let mut tot = C::new(0.0, 0.0);
                for (d, arow) in amps.iter().enumerate() {
                    tot += if flip(d) { -arow[h] } else { arow[h] };
                }
                m2 += tot.norm_sqr();
            }
            cf * m2
        };

        let base = if n_flows == 1 {
            m2_flipped(&|_| false)
        } else {
            let bound = BoundAmplitude::<f64>::bind(&evaluator, &evaluated);
            let mut sc = bound.scratch_space();
            bound.eval_m2(&p, &mut sc)
        };
        println!("MG ref |M|²      = {m2_ref:.10e}");
        println!(
            "vibegraph |M|²   = {base:.10e}   rel_diff = {:.3e}   (n_flows={n_flows}, CF(1,1)={cf})",
            (base - m2_ref).abs() / m2_ref
        );

        // The coherent-sum sign-flip search assumes a single color flow (a scalar
        // CF); with several flows the CF-weighted JAMP dump is the diagnostic.
        if n_flows > 1 {
            println!(
                "multi-flow process: per-flow JAMP dump is the diagnostic \
                 (compare_amps.py {name}); skipping single-flow sign-flip search"
            );
        } else if n <= 16 {
            // Exhaustive subset flips (diagram 0 held fixed: global sign is irrelevant).
            let mut hits: Vec<(u32, f64)> = (0u32..(1 << (n - 1)))
                .map(|mask| {
                    let m2 = m2_flipped(&|d| d > 0 && (mask >> (d - 1)) & 1 == 1);
                    (mask, (m2 - m2_ref).abs() / m2_ref)
                })
                .collect();
            hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            println!("best sign assignments (flipped diagram set -> rel_diff):");
            for (mask, rel) in hits.iter().take(4) {
                let flipped: Vec<usize> = (1..n).filter(|d| (mask >> (d - 1)) & 1 == 1).collect();
                println!("  flip {flipped:?} -> rel_diff {rel:.3e}");
            }
        } else {
            // One flip per propagator-signature class.
            let sigs: Vec<String> = asts.iter().map(prop_sig).collect();
            let uniq: std::collections::BTreeSet<&String> = sigs.iter().collect();
            println!("per-class sign flips ({} classes):", uniq.len());
            for sig in uniq {
                let m2 = m2_flipped(&|d| &sigs[d] == sig);
                println!(
                    "  flip [{sig}] -> rel_diff {:.3e}",
                    (m2 - m2_ref).abs() / m2_ref
                );
            }
        }
    }

    /// Cross-check the production scalar bilinear (`scalar_bilinear_current` × scalar
    /// leg) against the `iosxxx` reference routine.
    ///
    /// FFS1 (`ProjM`) is the left bilinear `ψ̄ P_L ψ`; FFS3 (`ProjP`) the right one.
    /// Each is multiplied by the off-shell scalar leg (the `Mul` the rooted FFS tree
    /// carries). `iosxxx` uses gc=[g,0] (left) for FFS1 and gc=[0,g] (right) for FFS3.
    #[test]
    fn test_eval_proj_amp_vs_iosxxx() {
        use crate::helas::vertex::iosxxx;
        use num_complex::Complex64;

        let mass = 0.511e-3_f64;
        let p_fi = LorentzVector::from_pxpypzmass(30.0, 0.0, 40.0, mass);
        let p_fo = LorentzVector::from_pxpypzmass(-20.0, 10.0, -30.0, mass);
        let p_s = -(p_fi + p_fo);
        let s_wf = ScalarWf {
            value: Complex64::new(0.7, -0.3),
            momentum: p_s,
        };
        let g = Complex64::new(1.0, 0.0);

        let hels = [SpinorHelicity::Down, SpinorHelicity::Up];
        for (hel1, hel2) in iproduct!(hels, hels) {
            for charge in [Charge::Particle, Charge::Antiparticle] {
                let fo = OutDiracWf::from_momentum(p_fo, mass, hel1, charge);
                let fi = InDiracWf::from_momentum(p_fi, mass, hel2, charge);

                let left_ref = iosxxx(&fo, &fi, &s_wf, [g, Complex64::new(0.0, 0.0)]);
                let right_ref = iosxxx(&fo, &fi, &s_wf, [Complex64::new(0.0, 0.0), g]);

                // leg1 = fi (column / ket), leg2 = fo (row / bra), leg3 = scalar.
                let fi_slot = WaveformSlot::FermionIn(fi);
                let fo_slot = WaveformSlot::FermionOut(fo);
                let s_slot = WaveformSlot::Scalar(s_wf);

                // FFS1: left bilinear × s
                let WaveformSlot::Scalar(got1) = mul_apply([
                    scalar_bilinear_current(&fi_slot, &fo_slot, Chirality::Left),
                    s_slot,
                ]) else {
                    panic!("FFS1 did not produce a scalar");
                };
                let diff1 = (got1.value - left_ref).norm();
                assert!(
                    diff1 < 1e-10,
                    "left bilinear vs iosxxx diff={diff1} (hel {hel1},{hel2}, {charge:?})"
                );

                // FFS3: right bilinear × s
                let WaveformSlot::Scalar(got3) = mul_apply([
                    scalar_bilinear_current(&fi_slot, &fo_slot, Chirality::Right),
                    s_slot,
                ]) else {
                    panic!("FFS3 did not produce a scalar");
                };
                let diff3 = (got3.value - right_ref).norm();
                assert!(
                    diff3 < 1e-10,
                    "right bilinear vs iosxxx diff={diff3} (hel {hel1},{hel2}, {charge:?})"
                );
            }
        }
    }

    /// Full-amplitude Ward identity for a 2→3 process with a final-state photon:
    /// `e+ e- > mu+ mu- a`. Replacing the external photon's polarisation ε^μ with
    /// its 4-momentum k^μ must make the *coherent sum over all diagrams* vanish
    /// (U(1) gauge invariance / current conservation). Unlike the single-current
    /// unit Ward tests, this exercises the multi-vertex paths the uux continuum
    /// depends on but 2→2 ee→μμ never hits:
    ///   - a fermion propagator chaining two vertices on one line (FSR: the muon
    ///     line absorbs the s-channel boson, propagates, then radiates the photon),
    ///   - an off-shell γ/Z (internal `VectorWf`, −i/q²) absorbed by a fermion line
    ///     via `GammaIout`/`GammaJout`.
    ///
    /// If the relative phases/signs between continuum diagrams are wrong (the
    /// diagnosed bug), this sum will NOT cancel.
    ///
    /// Largest U(1) Ward residual `|Σ_diagrams M(ε_γ→k_γ)| / max|M|`, maximised
    /// over all helicity configurations, for `proc` at momenta `p` with the photon
    /// on `ward_leg` replaced by its 4-momentum. Lepton masses are zeroed so the
    /// hand-built massless momenta are exactly on-shell (else the spinors fail the
    /// Dirac equation and Ward picks up an O(m²/s) artifact). Returns ~0 (machine
    /// precision) iff the coherent sum over diagrams gauge-cancels correctly.
    fn ward_max_ratio(proc: &str, p: &[LorentzVector<f64>], ward_leg: usize) -> f64 {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model_card(
            model.clone(),
            &"Block MASS\n 11 0.0\n 13 0.0\n 15 0.0\n"
                .parse::<ParamCard>()
                .unwrap(),
        );
        let opts = ParsingOptions::default();
        let card = parse_proc_card(proc, &opts).unwrap();
        let sets = generate_from_proc_card(&card, &model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);

        let mut scratch = bound.scratch_space();
        let global_scale = eval
            .helicities()
            .iter()
            .map(|hel| bound.eval_amplitude(p, hel, &mut scratch).norm())
            .fold(0.0_f64, f64::max)
            .max(1e-30);

        eval.helicities()
            .iter()
            .map(|hel| {
                bound
                    .eval_amplitude_ward(p, hel, ward_leg, &mut scratch)
                    .norm()
                    / global_scale
            })
            .fold(0.0_f64, f64::max)
    }

    /// Full-amplitude Ward identity for a 2→3 process with a final-state photon:
    /// `e+ e- > mu+ mu- a`. Replacing the external photon's polarisation ε^μ with
    /// its 4-momentum k^μ must make the *coherent sum over all diagrams* vanish
    /// (U(1) gauge invariance). Exercises the multi-vertex paths the uux continuum
    /// depends on but 2→2 ee→μμ never hits: a fermion propagator chaining two
    /// vertices on one line, and an off-shell γ/Z (internal `VectorWf`, −i/q²)
    /// absorbed by a fermion line via the off-shell-current nodes.
    #[test]
    fn test_ward_identity_full_amplitude_eemumua() {
        // Massless on-shell momenta in the e+e- CM frame, √s = 100; photon = leg 4.
        let s40 = 5.0 * 40.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),   // e+ (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),  // e- (leg 1)
            LorentzVector::new(30.0, 30.0, 0.0, 0.0),   // mu+ (leg 2)
            LorentzVector::new(35.0, -15.0, s40, 0.0),  // mu- (leg 3)
            LorentzVector::new(35.0, -15.0, -s40, 0.0), // a   (leg 4)
        ];
        let ratio = ward_max_ratio("generate e+ e- > mu+ mu- a", &p, 4);
        assert!(
            ratio < 1e-9,
            "2→3 Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// Quark-line counterpart of the 2→3 Ward test: `u u~ > mu+ mu- a`. The photon
    /// radiates off the (massless) initial-state up-quark line or the final muon
    /// line; the up-type quark FFV couplings (and the quark off-shell current) must
    /// gauge-cancel just like the leptonic case. This is the quark-continuum path
    /// the uux 2→6 process depends on.
    #[test]
    fn test_ward_identity_full_amplitude_uumumua() {
        let s40 = 5.0 * 40.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),   // u   (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),  // u~  (leg 1)
            LorentzVector::new(30.0, 30.0, 0.0, 0.0),   // mu+ (leg 2)
            LorentzVector::new(35.0, -15.0, s40, 0.0),  // mu- (leg 3)
            LorentzVector::new(35.0, -15.0, -s40, 0.0), // a   (leg 4)
        ];
        let ratio = ward_max_ratio("generate u u~ > mu+ mu- a", &p, 4);
        assert!(
            ratio < 1e-9,
            "u u~ Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// 2→5 Ward identity with THREE fermion lines: `e+ e- > mu+ mu- ta+ ta- a`.
    /// With three lepton lines joined by internal bosons, the boson-tree forces at
    /// least one fermion line to absorb TWO *internal* (off-shell, −i/q²) bosons in
    /// series — the exact path the uux 2→6 continuum needs but the 2→3/2→4 photon
    /// tests (one internal boson + external photons) never exercise. The photon
    /// (leg 6) is Ward-substituted.
    ///
    /// Regression guard for the FFS off-shell *scalar* (Higgs) current momentum bug:
    /// it used `fo.p + fi.p`, while the analogous off-shell vector current
    /// `GammaVout` uses `fo.p − fi.p` (the HELAS jioxxx convention). The sum is
    /// harmless at the amplitude sink (momentum unused there) but non-conserving when
    /// the scalar is an off-shell Higgs current feeding a VVS vertex — which only
    /// happens with ≥3 fermion lines. See `probe_2to5_momentum`.
    #[test]
    fn test_ward_identity_full_amplitude_eemumutata_a() {
        let r3 = 3.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),  // e+  (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0), // e-  (leg 1)
            LorentzVector::new(20.0, 20.0, 0.0, 0.0),  // mu+ (leg 2)
            LorentzVector::new(20.0, -20.0, 0.0, 0.0), // mu- (leg 3)
            LorentzVector::new(20.0, 0.0, 20.0, 0.0),  // ta+ (leg 4)
            LorentzVector::new(20.0, 0.0, -10.0, 10.0 * r3), // ta- (leg 5)
            LorentzVector::new(20.0, 0.0, -10.0, -10.0 * r3), // a  (leg 6)
        ];
        let ratio = ward_max_ratio("generate e+ e- > mu+ mu- ta+ ta- a", &p, 6);
        assert!(
            ratio < 1e-9,
            "2→5 Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// Full-amplitude Ward identity for a 2→4 process with TWO final-state photons:
    /// `e+ e- > mu+ mu- a a`. Beyond the 2→3 test this exercises a fermion line
    /// with THREE attachments (s-channel boson + two photons) → a *chained*
    /// off-shell fermion current (two fermion propagators in series), and
    /// `GammaVout` built from off-shell currents — the longer chains the 2→6 uux
    /// continuum needs. Ward-substituting one photon must still cancel the sum.
    #[test]
    fn test_ward_identity_full_amplitude_eemumuaa() {
        // Equal-energy massless tetrahedral final state (Σp⃗=0, ΣE=√s=100), e+e- on z.
        let c = 25.0 / 3.0_f64.sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),  // e+  (leg 0)
            LorentzVector::new(50.0, 0.0, 0.0, -50.0), // e-  (leg 1)
            LorentzVector::new(25.0, c, c, c),         // mu+ (leg 2)
            LorentzVector::new(25.0, c, -c, -c),       // mu- (leg 3)
            LorentzVector::new(25.0, -c, c, -c),       // a   (leg 4)
            LorentzVector::new(25.0, -c, -c, c),       // a   (leg 5)
        ];
        let ratio = ward_max_ratio("generate e+ e- > mu+ mu- a a", &p, 4);
        assert!(
            ratio < 1e-9,
            "2→4 Ward identity violated: max |k·M|/scale = {ratio:.3e}"
        );
    }

    /// The unified `Ast<Sym>` round-trips through its s-expression `Display`/`FromStr`
    /// (the egglog boundary): re-rendering the parsed tree reproduces the original
    /// string exactly.
    #[test]
    fn test_sexpr_roundtrip_eemumu() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::helas::eval::Sym;

        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &model).unwrap();
        let diagrams = compile_diagram_ast(&sets[0], &model).unwrap();

        let ast = lower::lower(&diagrams);
        let rendered = ast.to_string();
        let reparsed: Ast<Sym> = rendered.parse().expect("s-expr should re-parse");
        // `Display` expands the shared (DAG) currents into a tree, so the reparsed arena
        // has at least as many nodes; the rendered string is the stable invariant
        // (re-merging shared subterms is the future hash-consing/egglog pass).
        assert_eq!(
            rendered,
            reparsed.to_string(),
            "s-expr round-trip changed the tree"
        );
        assert!(reparsed.len() >= ast.len());
    }

    /// The whole-amplitude AST (one `Add` over all diagrams) reproduces the explicit
    /// coherent sum over per-diagram amplitudes, for every helicity of e+e-→μ+μ-.
    /// Guards the final diagram-sum `Add` and the symmetry/Fermi-sign folding.
    #[test]
    fn test_whole_amplitude_equals_diagram_sum_eemumu() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model_card(
            model.clone(),
            &"Block MASS\n 11 0.0\n 13 0.0\n"
                .parse::<ParamCard>()
                .unwrap(),
        );
        let opts = ParsingOptions::default();
        let card = parse_proc_card("generate e+ e- > mu+ mu-", &opts).unwrap();
        let sets = generate_from_proc_card(&card, &model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let diagrams = compile_diagram_ast(&sets[0], &model).unwrap();

        let st = 0.6_f64;
        let ct = (1.0 - st * st).sqrt();
        let p = [
            LorentzVector::new(50.0, 0.0, 0.0, 50.0),
            LorentzVector::new(50.0, 0.0, 0.0, -50.0),
            LorentzVector::new(50.0, 50.0 * st, 0.0, 50.0 * ct),
            LorentzVector::new(50.0, -50.0 * st, 0.0, -50.0 * ct),
        ];

        let mut scratch = bound.scratch_space();
        for hel in eval.helicities() {
            let whole = bound.eval_amplitude(&p, hel, &mut scratch);
            let parts = diagrams
                .iter()
                .map(|d| eval_single_diagram(d, &p, hel, &evaluated))
                .fold(C::new(0.0, 0.0), |a, b| a + b);
            assert!(
                (whole - parts).norm() <= 1e-12 * (whole.norm() + 1e-30),
                "whole-amplitude AST disagrees with per-diagram sum for hel {hel:?}: \
                 whole={whole:.6e} parts={parts:.6e}"
            );
        }
    }

    /// Cross-check the Z current from the *outgoing* mu-pair against MadGraph's W11
    /// intermediate wavefunction (hardcoded from the MG_EVAL_WFUNCS probe at the
    /// ee→μμττ CSV point-0 momenta), for helicities 38 and 42.
    ///
    /// The test sets up FFV2·GC_50 ⊕ FFV4·GC_59 with OUTGOING mu+ (Antiparticle,
    /// incoming=false) and mu- (Particle, incoming=false), then propagates through
    /// the Z. The current must equal MG's W-array bit-for-bit (VG = MG, no
    /// convention factor) — the wavefunction-level anchor tying the production
    /// vector chain to MadGraph.
    #[test]
    #[cfg(feature = "extended-validation")]
    fn test_z_current_outgoing_mupair_vs_mg() {
        use num_complex::Complex64;

        let model = sm_model(SMRestrict::Default);
        // Use MadGraph's massless-tau param card so couplings match the probe.
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let card_path = std::path::Path::new(&manifest).join(
            "../validation/madgraph/output/ee_to_mumu_tata_qcd0/Cards/param_card_masslesstau.dat",
        );
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| s.parse::<ParamCard>().ok())
            .expect("param_card_masslesstau.dat not found — run `pixi run -e madgraph build-diagrams` first");
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        let mu_id = model.particle_id("mu-").unwrap();
        let amu_id = model.particle_id("mu+").unwrap();
        let z_id = model.particle_id("Z").unwrap();

        // CSV point-0 momenta: mu+ (outgoing antiparticle) and mu- (outgoing particle).
        // MG Fortran leg ordering: P(0,3)=mu+, P(0,4)=mu- (leg indices 3,4 in Fortran = 2,3 in Python).
        let p_mup = LorentzVector::new(
            130.98844490914234,
            -106.66561232781022,
            -0.9379201403415187,
            -76.02328690775641,
        );
        let p_mum = LorentzVector::new(
            167.2530959714149,
            134.2336665209957,
            -62.607066356179416,
            -77.68703963098595,
        );

        // OUTGOING mu+ = Antiparticle; OUTGOING mu- = Particle.
        // leg_idx is the index into the momenta/helicities slice passed to eval, so 0/1.
        let leg_mup = ExtLegInfo {
            leg_idx: 0,
            id: amu_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: false,
        };
        let leg_mum = ExtLegInfo {
            leg_idx: 1,
            id: mu_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: false,
        };

        // FFV2·GC_50 + FFV4·GC_59 = SM ℓ̄ℓZ vertex (same as test_eval_ffv2_4_3 but
        // rooted at the Z (vector output leg = leg index 2 in the vertex).
        let vertex_info = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(&model, ffv2_id, &no_color(), gc50, Some(2), &[]).unwrap(),
                VertexTerm::from_ufo(&model, ffv4_id, &no_color(), gc59, Some(2), &[]).unwrap(),
            ],
        };

        let mz = evaluated.mass(z_id);

        let current_diagram = DiagramEval::from_nodes(
            2,
            vec![
                EvalNode::External(leg_mup.clone()),
                EvalNode::External(leg_mum.clone()),
                EvalNode::OffShellCurrent {
                    info: vertex_info.clone(),
                    adjoint: None,
                    children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                },
                EvalNode::Propagate {
                    info: PropInfo {
                        id: z_id,
                        t_channel: false,
                    },
                    adjoint: None,
                    child: EvalNodeId::new(2),
                },
            ],
        );

        // MG W11 values (indices [2..6], i.e. the 4 eps components) from MG_EVAL_WFUNCS,
        // measured at CSV point 0 via the probe_wfuncs.py script.
        // Convention check (contrast with test_eval_ffv2_4_3 incoming case): VG = MG here.
        // Hel 38: (e+:+1, e-:-1, mu+:-1, mu-:+1, ta+:+1, ta-:-1)
        //   → mu+ nhel=-1, mu- nhel=+1 → Down/Up
        // Hel 42: (e+:+1, e-:-1, mu+:+1, mu-:-1, ta+:+1, ta-:-1)
        //   → mu+ nhel=+1, mu- nhel=-1 → Up/Down
        let mg_w11 = [
            // hel 38: [eps_t, eps_x, eps_y, eps_z] from MG probe
            [
                Complex64::new(4.20870284e-04, 3.03971367e-04),
                Complex64::new(9.71186197e-05, -1.51285696e-04),
                Complex64::new(1.55174433e-04, -7.61571054e-04),
                Complex64::new(-8.63339445e-04, -3.02084576e-04),
            ],
            // hel 42: [eps_t, eps_x, eps_y, eps_z]
            [
                Complex64::new(-5.22725529e-04, 3.84361483e-04),
                Complex64::new(-1.22982451e-04, -1.88027964e-04),
                Complex64::new(-2.02039917e-04, -9.50088237e-04),
                Complex64::new(1.07570329e-03, -3.86719509e-04),
            ],
        ];

        let test_cases = [
            // hel 38: mu+ Down, mu- Up
            (SpinorHelicity::Down, SpinorHelicity::Up, mg_w11[0], "hel38"),
            // hel 42: mu+ Up, mu- Down
            (SpinorHelicity::Up, SpinorHelicity::Down, mg_w11[1], "hel42"),
        ];

        for (hel_mup, hel_mum, mg_eps, label) in test_cases {
            let WaveformSlot::Vector(got) = eval_single_diagram_slot(
                &current_diagram,
                &[p_mup, p_mum],
                &[hel_mup.sign(), hel_mum.sign()],
                &evaluated,
            ) else {
                panic!("Z current ({label}) must evaluate to a vector");
            };

            // The outgoing mu-pair convention is VG = MG (no i factor).
            for mu in 0..4 {
                let vg = got.eps.component(mu);
                let expected = mg_eps[mu];
                let diff = (vg - expected).norm();
                assert!(
                    diff < 5e-10 * mz as f64,
                    "Z current from outgoing mu-pair ({label}, μ={mu}): \
                     VG={vg:.4e} vs MG={expected:.4e}, diff={diff:.2e}"
                );
            }
        }
    }

    /// Pin the **e-line Z absorption** (chiral off-shell electron) against MadGraph.
    ///
    /// Controlled experiment on the e+-spine (MadGraph AMP(18) vs AMP(22), CSV point 0):
    /// the off-shell electron is built two ways that differ ONLY in the μ-side boson —
    ///   γ-path: e⁺ absorbs γ[μ] = `FFV1_1(e⁺, FFV1P0_3(μ-,μ+))`   (→ AMP(18))
    ///   Z-path: e⁺ absorbs Z[μ] = `FFV2_4_1(e⁺, FFV2_4_3(μ-,μ+))` (→ AMP(22))
    /// The γ path (chirality-blind, L=R) pins the rooting/adjoint/propagator machinery;
    /// the Z path adds the chiral (FFV2/FFV4) physics. The hand-built μ-pair current
    /// binds (μ⁻ bra, μ⁺ ket) at slots (mu+, mu-) — a reversed traversal — so the
    /// vertex is given the per-leg flows and the rooting conjugates its projector.
    ///
    /// Against MadGraph's actual off-shell electron (`probe_wfuncs.py`, slots 6=γ,
    /// 7=Z), both paths must equal MG exactly at both helicities. (Historically
    /// the Z path was 0.6403 × MG at the flipped-μ helicity — the per-Z continuum
    /// bug, fixed by the adjoint/crossing-aware chiral projector.)
    #[test]
    #[cfg(feature = "extended-validation")]
    fn test_espine_eline_z_absorption_ratio_vs_mg() {
        use num_complex::Complex64;

        let model = sm_model(SMRestrict::Default);
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let card_path = std::path::Path::new(&manifest).join(
            "../validation/madgraph/output/ee_to_mumu_tata_qcd0/Cards/param_card_masslesstau.dat",
        );
        let card = std::fs::read_to_string(&card_path)
            .ok()
            .and_then(|s| s.parse::<ParamCard>().ok())
            .expect("param_card_masslesstau.dat not found — run `pixi run -e madgraph build-diagrams` first");
        let evaluated = EvaluatedModel::from_model_card(model.clone(), &card);

        let gc3 = model.coupling_id("GC_3").unwrap();
        let gc50 = model.coupling_id("GC_50").unwrap();
        let gc59 = model.coupling_id("GC_59").unwrap();
        let ffv1_id = model.lorentz_id("FFV1").unwrap();
        let ffv2_id = model.lorentz_id("FFV2").unwrap();
        let ffv4_id = model.lorentz_id("FFV4").unwrap();
        let ep_id = model.particle_id("e+").unwrap();
        let em_id = model.particle_id("e-").unwrap();
        let mu_id = model.particle_id("mu-").unwrap();
        let amu_id = model.particle_id("mu+").unwrap();
        let a_id = model.particle_id("a").unwrap();
        let z_id = model.particle_id("Z").unwrap();

        let p_ep = LorentzVector::new(250.0, 0.0, 0.0, 250.0);
        let p_mup = LorentzVector::new(
            130.98844490914234,
            -106.66561232781022,
            -0.9379201403415187,
            -76.02328690775641,
        );
        let p_mum = LorentzVector::new(
            167.2530959714149,
            134.2336665209957,
            -62.607066356179416,
            -77.68703963098595,
        );

        let leg_mum = ExtLegInfo {
            leg_idx: 0,
            id: mu_id,
            spin: 2,
            charge: Charge::Particle,
            incoming: false,
        };
        let leg_mup = ExtLegInfo {
            leg_idx: 1,
            id: amu_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: false,
        };
        let leg_ep = ExtLegInfo {
            leg_idx: 2,
            id: ep_id,
            spin: 2,
            charge: Charge::Antiparticle,
            incoming: true,
        };
        let ep_flow = leg_ep.adjoint().unwrap();

        // Build the off-shell-electron sub-diagram for a given boson (γ via FFV1/GC_3,
        // or Z via FFV2⊕FFV4/GC_50,GC_59): μ-pair → boson current → e⁺ absorbs it.
        let make_diagram = |current_vertex: VertexInfo, absorb_vertex: VertexInfo, boson| {
            DiagramEval::from_nodes(
                3,
                vec![
                    EvalNode::External(leg_mum.clone()),
                    EvalNode::External(leg_mup.clone()),
                    EvalNode::OffShellCurrent {
                        info: current_vertex,
                        adjoint: None,
                        children: vec![EvalNodeId::new(0), EvalNodeId::new(1)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo {
                            id: boson,
                            t_channel: false,
                        },
                        adjoint: None,
                        child: EvalNodeId::new(2),
                    },
                    EvalNode::External(leg_ep.clone()),
                    EvalNode::OffShellCurrent {
                        info: absorb_vertex,
                        adjoint: Some(ep_flow),
                        children: vec![EvalNodeId::new(4), EvalNodeId::new(3)],
                    },
                    EvalNode::Propagate {
                        info: PropInfo {
                            id: em_id,
                            t_channel: false,
                        },
                        adjoint: Some(ep_flow),
                        child: EvalNodeId::new(5),
                    },
                ],
            )
        };

        // The μ-pair current binds (μ⁻ bra, μ⁺ ket) at slots (mu+, mu-) — the line
        // reads against the UFO slots (bra at the column), so the rooting needs the
        // per-leg flows to conjugate the chiral projector (uncrossed reversal).
        let mu_flows = [
            lf(leg_mum.adjoint().unwrap()),
            lf(leg_mup.adjoint().unwrap()),
            None,
        ];
        let gamma_current = VertexInfo {
            terms: vec![VertexTerm::from_ufo(
                &model,
                ffv1_id,
                &no_color(),
                gc3,
                Some(2),
                &mu_flows,
            )
            .unwrap()],
        };
        let gamma_absorb = VertexInfo {
            terms: vec![VertexTerm::from_ufo(
                &model,
                ffv1_id,
                &no_color(),
                gc3,
                Some(1),
                &[lf(ep_flow), lf(ep_flow), None],
            )
            .unwrap()],
        };
        let z_current = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(&model, ffv2_id, &no_color(), gc50, Some(2), &mu_flows)
                    .unwrap(),
                VertexTerm::from_ufo(&model, ffv4_id, &no_color(), gc59, Some(2), &mu_flows)
                    .unwrap(),
            ],
        };
        let z_absorb = VertexInfo {
            terms: vec![
                VertexTerm::from_ufo(
                    &model,
                    ffv2_id,
                    &no_color(),
                    gc50,
                    Some(1),
                    &[lf(ep_flow), lf(ep_flow), None],
                )
                .unwrap(),
                VertexTerm::from_ufo(
                    &model,
                    ffv4_id,
                    &no_color(),
                    gc59,
                    Some(1),
                    &[lf(ep_flow), lf(ep_flow), None],
                )
                .unwrap(),
            ],
        };

        let gamma_diagram = make_diagram(gamma_current, gamma_absorb, a_id);
        let z_diagram = make_diagram(z_current, z_absorb, z_id);

        let off_shell_e = |diagram: &DiagramEval, hmum, hmup| -> [Complex64; 4] {
            match eval_single_diagram_slot(
                diagram,
                &[p_mum, p_mup, p_ep],
                &[hmum, hmup, 1], // e+ helicity +1 (Up) for both hel 38 and 42
                &evaluated,
            ) {
                WaveformSlot::FermionOut(f) => core::array::from_fn(|k| f.spinor.component(k)),
                WaveformSlot::FermionIn(f) => core::array::from_fn(|k| f.spinor.component(k)),
                other => panic!("off-shell e must be a fermion, got {other:?}"),
            }
        };

        // r = eZ / eγ at the dominant component (cancels the orientation convention).
        let ratio = |ez: [Complex64; 4], eg: [Complex64; 4]| -> Complex64 {
            let k = (0..4)
                .max_by(|&a, &b| eg[a].norm().total_cmp(&eg[b].norm()))
                .unwrap();
            ez[k] / eg[k]
        };

        // MadGraph off-shell electron (probe_wfuncs.py slots 6=γ, 7=Z; physical [2..6]).
        let mg_ratio = |ez: [Complex64; 4], eg: [Complex64; 4]| ratio(ez, eg);
        let mg = [
            // (label, mu-, mu+ helicity codes, eγ[2..6], eZ[2..6])
            (
                "hel38",
                1_i32,
                -1_i32,
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(-2.108086e-05, -4.566556e-06),
                    Complex64::new(3.215450e-05, 1.827798e-05),
                ],
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(8.735431e-06, 1.853315e-06),
                    Complex64::new(-1.334406e-05, -7.510224e-06),
                ],
            ),
            (
                "hel42",
                -1_i32,
                1_i32,
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(1.989177e-05, -1.598997e-05),
                    Complex64::new(2.804911e-05, -1.685689e-05),
                ],
                [
                    Complex64::ZERO,
                    Complex64::ZERO,
                    Complex64::new(1.025656e-05, -8.316966e-06),
                    Complex64::new(1.447519e-05, -8.783500e-06),
                ],
            ),
        ];

        for (label, hmum, hmup, mg_eg, mg_ez) in mg {
            let vg_eg = off_shell_e(&gamma_diagram, hmum, hmup);
            let vg_ez = off_shell_e(&z_diagram, hmum, hmup);
            let r_vg = ratio(vg_ez, vg_eg);
            let r_mg = mg_ratio(mg_ez, mg_eg);
            eprintln!(
                "\n{label}: e-line off-shell electron, Z/γ ratio  VG={:+.5}{:+.5}i  MG={:+.5}{:+.5}i",
                r_vg.re, r_vg.im, r_mg.re, r_mg.im
            );

            // Photon absorption is chirality-blind (γ couples L=R), so it pins the
            // rooting/adjoint/propagator/momentum machinery: VG's γ-path off-shell electron
            // must equal MadGraph's EXACTLY (the fermion chain carries the Feynman
            // propagator −i, in phase with ALOHA). (The Z-path carries the chiral
            // physics and is the localiser — printed above.)
            let kmax = (0..4)
                .max_by(|&a, &b| mg_eg[a].norm().total_cmp(&mg_eg[b].norm()))
                .unwrap();
            let scale = mg_eg[kmax].norm();
            for k in 0..4 {
                let diff = (vg_eg[k] - mg_eg[k]).norm();
                assert!(
                    diff < 1e-6 * scale,
                    "{label} γ-path off-shell e [{k}]: VG={:.4e} vs MG={:.4e}, diff={diff:.2e}",
                    vg_eg[k],
                    mg_eg[k]
                );
            }

            // Z path: identical machinery, only the chiral (FFV2/FFV4) vertex differs.
            // With the adjoint-corrected chiral projector the off-shell electron equals
            // MG exactly at BOTH helicities (the historical per-Z 0.6403 at the
            // flipped-μ helicity is gone).
            let kz = (0..4)
                .max_by(|&a, &b| mg_ez[a].norm().total_cmp(&mg_ez[b].norm()))
                .unwrap();
            let zfac = vg_ez[kz] / mg_ez[kz];
            eprintln!(
                "  Z-path VG/MG = {:+.4}{:+.4}i   (expected 1)",
                zfac.re, zfac.im
            );
            assert!(
                (zfac.re - 1.0).abs() < 2e-3 && zfac.im.abs() < 2e-3,
                "{label} Z-path off-shell e: VG/MG={zfac:.4}, expected 1"
            );
        }
    }

    /// Every process of the MG-validated suite evaluates end-to-end in the default
    /// suite (no MG reference data): each helicity amplitude is finite (per-flow
    /// JAMPs for multi-flow processes, since those root on `Op::Flows` rather than a
    /// scalar) and the helicity-summed |M|² is finite and positive. Catches
    /// compile-pipeline and evaluator panics/NaNs on the full op mix, including the
    /// 2→6 processes; value-level validation is `validate_helas_mg`. RAMBO kinematics
    /// are massless (unphysical for the massive-external processes) — irrelevant for
    /// a finiteness check.
    #[test]
    fn mg_suite_forward_eval_is_finite() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        use super::super::compile::MG_VALIDATED_PROCESSES;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massless;

        let model = sm_model(SMRestrict::Default);
        let evaluated = crate::ufo::EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x0F0F_5EED);
        let sqrt_s = 500.0;
        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
                let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
                let mut scratch = amp.scratch_space();
                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massless(sqrt_s, eval.n_ext() - 2, &mut rng));
                for hel in eval.helicities() {
                    if eval.n_flows() == 1 {
                        let a = amp.eval_amplitude(&p, hel, &mut scratch);
                        assert!(
                            a.re.is_finite() && a.im.is_finite(),
                            "[{process}] non-finite amplitude at hel {hel:?}: {a:?}"
                        );
                    } else {
                        // `eval_amplitude` requires a scalar amplitude root; a
                        // multi-flow process roots on `Op::Flows` instead, so check
                        // each per-flow JAMP directly.
                        let jamps = amp.run_flows(&p, hel, &mut scratch);
                        for (i, j) in jamps.iter().enumerate() {
                            assert!(
                                j.re.is_finite() && j.im.is_finite(),
                                "[{process}] non-finite JAMP[{i}] at hel {hel:?}: {j:?}"
                            );
                        }
                    }
                }
                let m2 = amp.eval_m2(&p, &mut scratch);
                assert!(m2.is_finite() && m2 > 0.0, "[{process}] bad |M|²: {m2:?}");
            }
        }
    }

    /// Each extracted lane of the SIMD-batched [`eval_m2_lanes`] is **bit-identical**
    /// (equal f64 bits) to the scalar [`eval_m2`](BoundAmplitude::eval_m2) at the same
    /// point, for every process in the MG-validated suite, across three kinematically
    /// homogeneous batch regimes: partonic-CM z-beams, generic off-axis momenta, and
    /// threshold-adjacent (near-rest final-state) z-beams. Homogeneity keeps every
    /// data-dependent branch in the external-wavefunction builders lane-uniform (see
    /// the [`lanes`](super::lanes) contract), so the elementwise lane arithmetic
    /// reproduces the scalar float sequence with no reassociation. This pins the
    /// lane-uniformity claim: a mixed-branch batch would silently apply one branch to
    /// all lanes and break the assertion.
    #[test]
    fn eval_m2_lanes_bit_identical_to_scalar() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        use super::super::compile::MG_VALIDATED_PROCESSES;
        use super::eval_m2_lanes;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massless;

        // Two lanes suffice to pin bit-identity: `transpose_points`/`eval_m2_lanes`
        // are generic over the width, so N=2 exercises the full pack/eval/unpack path.
        // (`lanes4_lanes8_pack_unpack_bit_identical` covers the wider widths on one
        // small process.)
        const N: usize = 2;

        let model = sm_model(SMRestrict::Default);
        let evaluated = crate::ufo::EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x51D_1DE0);

        // One fully-random timelike, off-axis 4-momentum: E is strictly larger than
        // |p⃗| (so `pp = |p⃗|`), and a generic direction keeps every leg off the
        // z-axis. Physical on-shell-ness is irrelevant — the same momenta feed both
        // the scalar and lane paths — but the off-axis, timelike form drives the
        // "generic moving" branch of every wavefunction builder on all lanes.
        let random_mom = |rng: &mut StdRng| {
            let px = rng.random_range(-100.0..100.0);
            let py = rng.random_range(-100.0..100.0);
            let pz = rng.random_range(-100.0..100.0);
            let p3 = (px * px + py * py + pz * pz).sqrt();
            let e = p3 + rng.random_range(1.0..100.0);
            LorentzVector::new(e, px, py, pz)
        };

        for process in MG_VALIDATED_PROCESSES {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            assert!(!sets.is_empty(), "no diagrams for '{process}'");
            for set in &sets {
                let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
                let n_ext = eval.n_ext();
                let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
                let lane_amp = amp.broadcast_lanes::<N>();
                let mut scratch = amp.scratch_space();
                let mut lane_scratch = lane_amp.scratch_space();

                // z-beam partonic CM at √s, threshold-adjacent at a small √s (slow
                // finals), and fully generic off-axis momenta.
                let z_beam = |rng: &mut StdRng, sqrt_s: f64| {
                    let mut p = vec![
                        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                        LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                    ];
                    p.extend(rambo_massless(sqrt_s, n_ext - 2, rng));
                    p
                };

                let regimes: [Box<dyn Fn(&mut StdRng) -> Vec<LorentzVector<f64>>>; 3] = [
                    Box::new(|rng: &mut StdRng| z_beam(rng, 500.0)),
                    Box::new(|rng: &mut StdRng| z_beam(rng, 12.0)),
                    Box::new(|rng: &mut StdRng| (0..n_ext).map(|_| random_mom(rng)).collect()),
                ];

                // The 2→6 arenas are heavy in debug; the smaller processes (which
                // include every external-wavefunction branch class — massless/massive
                // vectors, massive fermions) carry the off-axis and threshold regimes,
                // so the 8-leg processes only run the production z-beam regime.
                let n_regimes = if n_ext >= 8 { 1 } else { regimes.len() };
                for (r, make_point) in regimes.iter().enumerate().take(n_regimes) {
                    let batch: [Vec<LorentzVector<f64>>; N] =
                        std::array::from_fn(|_| make_point(&mut rng));
                    let point_refs: [&[LorentzVector<f64>]; N] =
                        std::array::from_fn(|k| batch[k].as_slice());

                    let scalar: [f64; N] =
                        std::array::from_fn(|k| amp.eval_m2(&batch[k], &mut scratch));
                    let lanes = eval_m2_lanes(&lane_amp, &point_refs, &mut lane_scratch);

                    for k in 0..N {
                        assert_eq!(
                            scalar[k].to_bits(),
                            lanes[k].to_bits(),
                            "[{process}] regime {r} lane {k}: scalar {} vs lane {} not bit-identical",
                            scalar[k],
                            lanes[k]
                        );
                    }
                }
            }
        }
    }

    /// The lane pack/eval/unpack path is bit-identical to scalar at the wider widths
    /// `N ∈ {4, 8}` too, on one light process — the width-4/8 counterpart to the
    /// full-suite N=2 gate above (which pins every process at N=2).
    #[test]
    fn lanes4_lanes8_pack_unpack_bit_identical() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        use super::eval_m2_lanes;
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massless;

        let model = sm_model(SMRestrict::Default);
        let evaluated = crate::ufo::EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0xC0FFEE_11);

        let pc = parse_proc_card("generate e+ e- > mu+ mu- a", &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let n_ext = eval.n_ext();
        let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);

        let point = |rng: &mut StdRng| {
            let mut p = vec![
                LorentzVector::new(250.0, 0.0, 0.0, 250.0),
                LorentzVector::new(250.0, 0.0, 0.0, -250.0),
            ];
            p.extend(rambo_massless(500.0, n_ext - 2, rng));
            p
        };

        fn check<const M: usize>(
            amp: &BoundAmplitude<'_, f64>,
            batch: &[Vec<LorentzVector<f64>>; M],
        ) where
            LaneLen<M>: IntoArrayLength,
            LaneField<M>: Real,
        {
            let lane_amp = amp.broadcast_lanes::<M>();
            let mut lscratch = lane_amp.scratch_space();
            let mut sscratch = amp.scratch_space();
            let refs: [&[LorentzVector<f64>]; M] = std::array::from_fn(|k| batch[k].as_slice());
            let lanes = eval_m2_lanes(&lane_amp, &refs, &mut lscratch);
            for k in 0..M {
                let scalar = amp.eval_m2(&batch[k], &mut sscratch);
                assert_eq!(scalar.to_bits(), lanes[k].to_bits(), "N={M} lane {k}");
            }
        }

        let batch4: [Vec<LorentzVector<f64>>; 4] = std::array::from_fn(|_| point(&mut rng));
        check::<4>(&amp, &batch4);
        let batch8: [Vec<LorentzVector<f64>>; 8] = std::array::from_fn(|_| point(&mut rng));
        check::<8>(&amp, &batch8);
    }

    /// The helicity-expanded `eval_m2` equals the per-combination sum through the
    /// unexpanded program, **bit-for-bit**: the expansion copies each node's arithmetic
    /// verbatim (same kernels, same operand order), so every combination's amplitude —
    /// and the identically-ordered CF-weighted sum over combinations — is the same
    /// float. Covers single-flow (colorless + massive-external) and multi-flow
    /// (NCOLOR=2 and NCOLOR=6) processes.
    #[test]
    fn expanded_eval_m2_matches_per_helicity_sum() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massless;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x5A5A_11A5);
        let sqrt_s = 400.0;

        for process in [
            "e+ e- > mu+ mu-",
            "e+ e- > w+ w-",
            "e+ e- > ta+ ta- h",
            "u u~ > u u~",
            "g g > g g",
        ] {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            for set in &sets {
                let eval = AmplitudeEvaluator::compile(set, &model).unwrap();
                let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
                let mut scratch = amp.scratch_space();

                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massless(sqrt_s, eval.n_ext() - 2, &mut rng));

                let expanded = amp.eval_m2(&p, &mut scratch);

                // The same sum through the unexpanded program, one combination at a
                // time, replicating eval_m2's accumulation order exactly.
                let n = eval.n_flows();
                let reference = if n == 1 {
                    let mut hel_sum = 0.0f64;
                    for hel in eval.helicities() {
                        let m = amp.eval_amplitude(&p, hel, &mut scratch);
                        hel_sum += m.norm_sqr();
                    }
                    hel_sum * amp.cf[0]
                } else {
                    let mut total = 0.0f64;
                    for hel in eval.helicities() {
                        let jamps = amp.run_flows(&p, hel, &mut scratch);
                        for i in 0..n {
                            let mut ztemp = C::new(0.0, 0.0);
                            for (j, jamp_j) in jamps.iter().enumerate() {
                                ztemp += jamp_j.scale(amp.cf[j * n + i]);
                            }
                            total += (ztemp * jamps[i].conj()).re;
                        }
                    }
                    total
                };

                assert_eq!(
                    expanded, reference,
                    "[{process}] expanded eval_m2 diverged from the per-helicity sum"
                );
            }
        }
    }

    /// Diagnostic (`--ignored --nocapture`): per-combination relative contribution
    /// spectrum, to see the gap between contributing combinations and the
    /// floating-point residues of identically-zero ones.
    #[test]
    #[ignore]
    fn helicity_contribution_spectrum() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massive;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0x5BEC7);
        let sqrt_s = 500.0;

        for process in [
            "g g > g g",
            "g g > t t~",
            "e+ e- > w+ w-",
            "e+ e- > z h",
            "e+ e- > mu+ mu- ta+ ta- QCD=0",
        ] {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            let amp = BoundAmplitude::<f64>::bind(&eval, &evaluated);
            let mut scratch = amp.scratch_space();
            let m_out: Vec<f64> = eval.external_particles()[eval.n_in()..]
                .iter()
                .map(|&pid| evaluated.mass(pid))
                .collect();
            let mut p = vec![
                LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
            ];
            p.extend(rambo_massive(sqrt_s, &m_out, &mut rng));

            let n = eval.n_flows();
            let mut ts: Vec<f64> = Vec::new();
            for hel in eval.helicities() {
                let t = if n == 1 {
                    amp.eval_amplitude(&p, hel, &mut scratch).norm_sqr() * amp.cf[0]
                } else {
                    let jamps = amp.run_flows(&p, hel, &mut scratch);
                    let mut total = 0.0f64;
                    for i in 0..n {
                        let mut ztemp = C::new(0.0, 0.0);
                        for (j, jamp_j) in jamps.iter().enumerate() {
                            ztemp += jamp_j.scale(amp.cf[j * n + i]);
                        }
                        total += (ztemp * jamps[i].conj()).re;
                    }
                    total
                };
                ts.push(t);
            }
            let ans: f64 = ts.iter().sum();
            let mut rel: Vec<f64> = ts.iter().map(|t| t / ans).collect();
            rel.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let shown: Vec<String> = rel.iter().map(|r| format!("{r:.2e}")).collect();
            println!("{process}: ncomb={} rel spectrum = {shown:?}", rel.len());

            // z-boosted variant of the same point
            let boosted: Vec<LorentzVector<f64>> =
                p.iter().map(|q| q.boost([0.0, 0.0, 0.43])).collect();
            let mut ts2: Vec<f64> = Vec::new();
            for hel in eval.helicities() {
                let t = if n == 1 {
                    amp.eval_amplitude(&boosted, hel, &mut scratch).norm_sqr() * amp.cf[0]
                } else {
                    let jamps = amp.run_flows(&boosted, hel, &mut scratch);
                    let mut total = 0.0f64;
                    for i in 0..n {
                        let mut ztemp = C::new(0.0, 0.0);
                        for (j, jamp_j) in jamps.iter().enumerate() {
                            ztemp += jamp_j.scale(amp.cf[j * n + i]);
                        }
                        total += (ztemp * jamps[i].conj()).re;
                    }
                    total
                };
                ts2.push(t);
            }
            let ans2: f64 = ts2.iter().sum();
            let mut rel2: Vec<f64> = ts2.iter().map(|t| t / ans2).collect();
            rel2.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let shown2: Vec<String> = rel2.iter().map(|r| format!("{r:.2e}")).collect();
            println!("  z-boosted: {shown2:?}");
        }
    }

    /// `prune_zero_helicities` keeps exactly the combination counts MadGraph's own
    /// helicity filter bakes into its helicity-recycled sources (the `NCOMB` of each
    /// process's generated `matrix1_optim.f`), and the pruned `eval_m2` equals the
    /// unpruned one **bit-for-bit** on fresh phase-space points — every dropped
    /// combination contributed exactly `+0.0` to the helicity sum.
    #[test]
    fn prune_zero_helicities_matches_madgraph_filter_bitwise() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massive;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0xF117E5);
        let sqrt_s = 500.0;

        // (process, combinations, survivors): survivor counts pinned against
        // MadGraph's generated helicity-recycled sources.
        for (process, n_all, n_good) in [
            ("e+ e- > mu+ mu-", 16, 4),
            ("u u~ > mu+ mu-", 16, 4),
            ("e+ e- > e+ e-", 16, 6),
            ("e+ e- > mu+ mu- a", 32, 8),
            ("e+ e- > t t~", 16, 8),
            ("e+ e- > w+ w-", 36, 16),
            ("e+ e- > z h", 12, 6),
            ("e+ e- > ta+ ta- h", 16, 8),
            ("e+ e- > mu+ mu- ta+ ta- QCD=0", 64, 16),
            ("u u~ > c c~ e+ e- mu+ mu- QCD=0", 256, 16),
            ("b b~ > c c~ e+ e- mu+ mu- QCD=0", 256, 32),
            ("u u~ > u u~", 16, 6),
            ("g g > t t~", 16, 12),
            ("g g > g g", 16, 6),
        ] {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            let eval_full = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            let mut eval_pruned = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            assert_eq!(eval_full.helicities().len(), n_all, "[{process}] combos");

            let dropped = eval_pruned.prune_zero_helicities(&evaluated);
            assert_eq!(
                eval_pruned.helicities().len(),
                n_good,
                "[{process}] survivors (dropped {dropped})"
            );
            assert_eq!(dropped, n_all - n_good, "[{process}] dropped");

            let full = BoundAmplitude::<f64>::bind(&eval_full, &evaluated);
            let pruned = BoundAmplitude::<f64>::bind(&eval_pruned, &evaluated);
            let mut scratch_full = full.scratch_space();
            let mut scratch_pruned = pruned.scratch_space();
            let m_out: Vec<f64> = eval_full.external_particles()[eval_full.n_in()..]
                .iter()
                .map(|&pid| evaluated.mass(pid))
                .collect();
            for _ in 0..8 {
                let mut p = vec![
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
                    LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
                ];
                p.extend(rambo_massive(sqrt_s, &m_out, &mut rng));
                let a = full.eval_m2(&p, &mut scratch_full);
                let b = pruned.eval_m2(&p, &mut scratch_pruned);
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "[{process}] pruned eval_m2 diverged: {a:e} vs {b:e}"
                );
            }
        }
    }

    /// A pruned evaluator's `eval_m2` requires partonic-CM momenta with beams along
    /// ±z (`prune_zero_helicities` docs): `g g > t t~` drops same-helicity-gluon /
    /// opposite-helicity-top combinations that vanish by J_z conservation about the
    /// beam axis only in that frame. Feeding it a z-boosted point (physically valid
    /// kinematics, just the wrong frame for a pruned evaluator) must be caught
    /// rather than silently under-counting the sum. Without the frame guard in
    /// `eval_m2` this call would return a low-by-~3e-3 result instead of panicking.
    #[test]
    #[should_panic(expected = "partonic-CM kinematics")]
    fn eval_m2_pruned_rejects_boosted_frame() {
        use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
        use crate::phasespace::rambo_massive;
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let opts = ParsingOptions::default();
        let mut rng = StdRng::seed_from_u64(0xB005_1E5);
        let sqrt_s = 500.0;

        let pc = parse_proc_card("generate g g > t t~", &opts).unwrap();
        let sets = generate_from_proc_card(&pc, &model).unwrap();
        let mut eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
        let dropped = eval.prune_zero_helicities(&evaluated);
        assert!(
            dropped > 0,
            "expected g g > t t~ to prune some combinations"
        );
        assert!(eval.is_pruned());

        let bound = BoundAmplitude::<f64>::bind(&eval, &evaluated);
        let mut scratch = bound.scratch_space();
        let m_out: Vec<f64> = eval.external_particles()[eval.n_in()..]
            .iter()
            .map(|&pid| evaluated.mass(pid))
            .collect();
        let mut p = vec![
            LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, sqrt_s / 2.0),
            LorentzVector::new(sqrt_s / 2.0, 0.0, 0.0, -sqrt_s / 2.0),
        ];
        p.extend(rambo_massive(sqrt_s, &m_out, &mut rng));

        // Boost the whole event along the beam axis: still on-shell, momentum-
        // conserving kinematics, but no longer the partonic-CM frame the pruning
        // survey assumed.
        let boosted: Vec<LorentzVector<f64>> =
            p.iter().map(|q| q.boost([0.0, 0.0, 0.43])).collect();
        bound.eval_m2(&boosted, &mut scratch);
    }
}

#[cfg(test)]
mod hel_expand_stats {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::sm::{sm_model, SMRestrict};

    /// Size/shape probe of the helicity expansion (`--ignored --nocapture` to read the
    /// numbers): the expanded arena must be strictly smaller than combinations × base
    /// nodes (hash-consing shares something on every process), and the liveness
    /// allocator must keep each result arena well below one slot per node.
    #[test]
    #[ignore]
    fn expansion_shares_nodes_and_bounds_arenas() {
        let model = sm_model(SMRestrict::Default);
        let opts = ParsingOptions::default();
        for process in [
            "e+ e- > mu+ mu-",
            "g g > g g",
            "e+ e- > mu+ mu- ta+ ta- QCD=0",
            "u u~ > c c~ e+ e- mu+ mu- QCD=0",
        ] {
            let pc = parse_proc_card(&format!("generate {process}"), &opts).unwrap();
            let sets = generate_from_proc_card(&pc, &model).unwrap();
            let eval = AmplitudeEvaluator::compile(&sets[0], &model).unwrap();
            let base = eval.folded();
            let n_combos = eval.helicities().len();
            let t0 = std::time::Instant::now();
            let hel = eval.folded_hel();
            let dt = t0.elapsed();
            println!(
                "{process}: combos={n_combos} base_nodes={} expanded_nodes={} (naive {}), \
                 base_arenas={:?} hel_arenas={:?}, expand_time={dt:?}",
                base.ast.len(),
                hel.ast.len(),
                base.ast.len() * n_combos,
                base.program().arena_sizes,
                hel.program().arena_sizes,
            );
            assert!(hel.ast.len() < n_combos * base.ast.len());
            let peak: u32 = hel.program().arena_sizes.iter().sum();
            assert!(
                (peak as usize) < hel.ast.len() / 2,
                "[{process}] liveness allocation degenerated: peak {peak} live slots \
                 for {} nodes",
                hel.ast.len()
            );
        }
    }
}
