//! The unified node language for the evaluation AST.
//!
//! A node is a dataless [`Op`] tag plus a typed leaf payload (`Node<T>`); children
//! are stored separately by the [`Ast`](super::ast::Ast) arena. The same `Op` set is
//! shared by two leaf flavors:
//! - [`Sym`] — model ids ([`CouplingId`]/[`ParticleId`]/coeff/leg), the egglog /
//!   structure-optimization domain (`Ast<Sym>`).
//! - [`Const`] — deduped pool indices, the constant-folded eval domain (`Ast<Const>`),
//!   resolved against the `C<F>`/`F` pools built by [`super::fold`].
//!
//! The `Op` disambiguates what a leaf means (e.g. `Op::Mass` vs `Op::Width` both carry a
//! `ParticleId`), so the leaf enums stay small.

use std::fmt;

use crate::helas::repr::numbers::Charge;
use crate::ufo::couplings::CouplingId;
use crate::ufo::particles::ParticleId;

/// Index of a node within an [`Ast`](super::ast::Ast) arena.
pub type NodeId = u32;

/// Opcode tag — carries no data of its own (operands are arena children, constants are
/// the node's leaf payload).
///
/// The s-expression head token is the variant name verbatim; the token ↔ op round-trip
/// ([`name`](Op::name)/[`from_name`](Op::from_name)) and the full variant list are
/// derived by `strum` from the enum itself.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
    strum::VariantNames,
)]
pub enum Op {
    // ── inputs / structural / algebraic ──
    /// External wavefunction input. Leaf: `{leg_idx, spin, charge}`; child: `[Mass]`.
    External,
    /// Propagator. Children: `[current, Mass, Width]`. Dispatches on the input
    /// current's variance at runtime (a covariant `MetricVout`/`LowerVout` current
    /// forms its longitudinal term differently and is raised back), so no separate
    /// lowered-storage opcode is needed.
    Propagate,
    /// n-ary product: ≤1 non-scalar child sets the output type, the rest are scalar
    /// factors. Subsumes scalar×wf scaling (coupling·coeff·current) and the Lorentz
    /// tensor product.
    Mul,
    /// n-ary sum (Lorentz-term, vertex-term, and diagram sums).
    Add,
    // ── Lorentz primitives (semantics mirror the old `LorentzEvalNode`) ──
    /// 2 fermions → off-shell vector current.
    GammaVout,
    /// vector + flow-in fermion → flow-in fermion current.
    GammaIout,
    /// vector + flow-out fermion → flow-out fermion current.
    GammaOout,
    /// left chiral projection of a continuing fermion current.
    ProjM,
    /// right chiral projection of a continuing fermion current.
    ProjP,
    /// left chiral scalar bilinear ψ̄ P_L ψ.
    ProjMAmp,
    /// right chiral scalar bilinear ψ̄ P_R ψ.
    ProjPAmp,
    /// contract two vectors → scalar.
    Metric,
    /// contract two vectors → scalar, times the vertex's −i (pure-metric
    /// structure rooted as an amplitude; see `LorentzEvalNode::MetricNegI`).
    MetricNegI,
    /// metric with one free index → off-shell vector current.
    MetricVout,
    /// `MetricVout` without the −i vertex factor (index lowering only); the
    /// vector-output transform of P-carrying structures (VVV).
    LowerVout,
    /// full scalar bilinear ψ̄ δ ψ.
    IdentityAmp,
    // ── fused chiral FFV kernels ──
    // A vertex whose structures form a chiral pair (`Gamma·ProjM` / `Gamma·ProjP`
    // variants of one contraction shape) is fused at lowering into a single node:
    // the per-chirality effective couplings `g_L`/`g_R` (each a scalar sub-graph
    // `Σ coupling·coeff`) become operands, and the kernel evaluates
    // `g_L·(left-handed term) + g_R·(right-handed term)` directly — no `Mul`/`Add`
    // scaffolding, and no structurally-zero chiral half.
    /// fused chiral [`GammaVout`](Op::GammaVout): 2 fermions + `g_L` + `g_R` →
    /// off-shell vector current `g_L J_L^μ + g_R J_R^μ`. Children:
    /// `[f_i, f_j, gL, gR]` with the projector on the `f_j` position.
    FfvVout,
    /// fused chiral [`GammaIout`](Op::GammaIout): vector + flow-in fermion +
    /// `g_L` + `g_R` → flow-in fermion current `ε̸(g_L ψ_L ⊕ g_R ψ_R)`.
    /// Children: `[v, f, gL, gR]`.
    FfvIout,
    /// fused chiral [`GammaOout`](Op::GammaOout): vector + flow-out fermion +
    /// `g_L` + `g_R` → flow-out fermion current. Children: `[v, f, gL, gR]`.
    FfvOout,
    /// 4-momentum of the single child input, as a vector.
    PMom,
    /// 4-momentum of the vertex's *output* leg, as a vector: −Σ (input momenta).
    /// Children: all input currents of the vertex (ALOHA reads the output `P` off
    /// the negated stored sum, e.g. `VVV1P0_1`: `P1 = −(V2+V3)`).
    PMomOut,
    // ── constant leaves ──
    /// complex coupling constant (leaf id → `consts_c`).
    Coupling,
    /// particle mass (leaf id → `consts_f`).
    Mass,
    /// particle width (leaf id → `consts_f`).
    Width,
    /// real Lorentz-structure coefficient (leaf → `consts_f`).
    Coeff,
}

impl Op {
    /// The s-expression head token for this op (the `snake_case` variant name).
    pub fn name(self) -> &'static str {
        self.into()
    }

    /// Parse an op from its s-expression head token.
    pub fn from_name(s: &str) -> Option<Op> {
        use std::str::FromStr;
        Op::from_str(s).ok()
    }

    /// Whether this op carries a leaf payload token in the s-expression (a single
    /// id/coeff for the constant leaves, the `leg spin charge` triple for `External`).
    pub fn has_leaf_token(self) -> bool {
        matches!(
            self,
            Op::External | Op::Coupling | Op::Mass | Op::Width | Op::Coeff
        )
    }
}

/// A node: opcode tag + typed leaf payload. Children live in the arena's CSR table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node<T> {
    pub op: Op,
    pub leaf: T,
}

impl<T> Node<T> {
    pub fn new(op: Op, leaf: T) -> Self {
        Node { op, leaf }
    }
}

/// Symbolic leaf payload: model ids, kept independent of any parameter card.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sym {
    /// `Op::Coupling` payload.
    Coupling(CouplingId),
    /// `Op::Mass` / `Op::Width` payload.
    Particle(ParticleId),
    /// `Op::Coeff` payload.
    Coeff(f64),
    /// `Op::External` payload.
    Ext {
        leg_idx: usize,
        spin: i32,
        charge: Charge,
        /// Whether this leg is an incoming external (selects ket/bra flow and the
        /// `nsf` sign of the wavefunction). Baked in at compile time.
        incoming: bool,
    },
    /// Non-leaf op: no payload.
    None,
}

/// Folded leaf payload: deduped indices into the `C<F>` / `F` constant pools.
///
/// Kept to a single `u32` payload (8 bytes total) so folded nodes — and the stack
/// evaluator's instruction stream — stay small; the `External` leg details live in
/// the folded leg table (see [`super::fold::ExtLeg`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Const {
    /// index into `consts_c` (complex pool) — `Op::Coupling`.
    Complex(u32),
    /// index into `consts_f` (real pool) — `Op::Mass` / `Op::Width` / `Op::Coeff`.
    Real(u32),
    /// index into the folded external-leg table — `Op::External`.
    Ext(u32),
    /// Non-leaf op: no payload.
    None,
}

/// Decode a charge from its HELAS `nsf` sign (the s-expr encoding of [`Charge::sign`]).
pub(super) fn charge_from_sign(s: i32) -> Charge {
    if s < 0 {
        Charge::Antiparticle
    } else {
        Charge::Particle
    }
}

impl fmt::Display for Sym {
    /// Render only the payload (the enclosing [`Ast`](super::ast::Ast) emits the op head).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sym::Coupling(id) => write!(f, "(CouplingId {id})"),
            Sym::Particle(id) => write!(f, "(ParticleId {id})"),
            Sym::Coeff(c) => write!(f, "(Real {c:?})"),
            Sym::Ext {
                leg_idx,
                spin,
                charge,
                incoming,
            } => write!(
                f,
                "(ExtLegInfo {leg_idx} {spin} {} {})",
                charge.sign(),
                *incoming as i32
            ),
            Sym::None => Ok(()),
        }
    }
}

impl fmt::Display for Const {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Const::Complex(idx) => write!(f, "(Complex {idx})"),
            Const::Real(idx) => write!(f, "(Real {idx})"),
            Const::Ext(idx) => write!(f, "(Ext {idx})"),
            Const::None => write!(f, "(None)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Op;

    /// `Op` ↔ s-expression head token is a bijection: strum's `VariantArray` and
    /// `VariantNames` are index-aligned, every op round-trips through `name`/`from_name`,
    /// and no two ops share a name.
    #[test]
    fn op_names_roundtrip() {
        use strum::{VariantArray, VariantNames};

        let ops = <Op as VariantArray>::VARIANTS;
        let names = <Op as VariantNames>::VARIANTS;
        assert_eq!(ops.len(), names.len());

        let mut seen = HashSet::new();
        for (&op, &name) in ops.iter().zip(names) {
            assert_eq!(op.name(), name, "name for {op:?}");
            assert_eq!(Op::from_name(name), Some(op), "round-trip for {op:?}");
            assert!(seen.insert(name), "duplicate s-expr name {name}");
        }
    }
}
