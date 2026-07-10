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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// Every opcode, in declaration order. Keep in sync with the enum (the
    /// [`name`](Op::name)/[`from_name`](Op::from_name) matches are compiler-checked
    /// exhaustive; this list is pinned against them by `op_names_roundtrip`).
    pub const ALL: [Op; 22] = [
        Op::External,
        Op::Propagate,
        Op::Mul,
        Op::Add,
        Op::GammaVout,
        Op::GammaIout,
        Op::GammaOout,
        Op::ProjM,
        Op::ProjP,
        Op::ProjMAmp,
        Op::ProjPAmp,
        Op::Metric,
        Op::MetricNegI,
        Op::MetricVout,
        Op::LowerVout,
        Op::IdentityAmp,
        Op::PMom,
        Op::PMomOut,
        Op::Coupling,
        Op::Mass,
        Op::Width,
        Op::Coeff,
    ];

    /// The s-expression head token for this op.
    pub fn name(self) -> &'static str {
        match self {
            Op::External => "External",
            Op::Propagate => "Propagate",
            Op::Mul => "Mul",
            Op::Add => "Add",
            Op::GammaVout => "GammaVout",
            Op::GammaIout => "GammaIout",
            Op::GammaOout => "GammaOout",
            Op::ProjM => "ProjM",
            Op::ProjP => "ProjP",
            Op::ProjMAmp => "ProjMAmp",
            Op::ProjPAmp => "ProjPAmp",
            Op::Metric => "Metric",
            Op::MetricNegI => "MetricNegI",
            Op::MetricVout => "MetricVout",
            Op::LowerVout => "LowerVout",
            Op::IdentityAmp => "IdentityAmp",
            Op::PMom => "PMom",
            Op::PMomOut => "PMomOut",
            Op::Coupling => "Coupling",
            Op::Mass => "Mass",
            Op::Width => "Width",
            Op::Coeff => "Coeff",
        }
    }

    /// Parse an op from its s-expression head token.
    pub fn from_name(s: &str) -> Option<Op> {
        Some(match s {
            "External" => Op::External,
            "Propagate" => Op::Propagate,
            "Mul" => Op::Mul,
            "Add" => Op::Add,
            "GammaVout" => Op::GammaVout,
            "GammaIout" => Op::GammaIout,
            "GammaOout" => Op::GammaOout,
            "ProjM" => Op::ProjM,
            "ProjP" => Op::ProjP,
            "ProjMAmp" => Op::ProjMAmp,
            "ProjPAmp" => Op::ProjPAmp,
            "Metric" => Op::Metric,
            "MetricNegI" => Op::MetricNegI,
            "MetricVout" => Op::MetricVout,
            "LowerVout" => Op::LowerVout,
            "IdentityAmp" => Op::IdentityAmp,
            "PMom" => Op::PMom,
            "PMomOut" => Op::PMomOut,
            "Coupling" => Op::Coupling,
            "Mass" => Op::Mass,
            "Width" => Op::Width,
            "Coeff" => Op::Coeff,
            _ => return None,
        })
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

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Const {
    /// index into `consts_c` (complex pool) — `Op::Coupling`.
    Complex(u32),
    /// index into `consts_f` (real pool) — `Op::Mass` / `Op::Width` / `Op::Coeff`.
    Real(u32),
    /// `Op::External` payload (structural, never pooled).
    Ext {
        leg_idx: usize,
        spin: i32,
        charge: Charge,
        /// Whether this leg is an incoming external (see [`Sym::Ext`]).
        incoming: bool,
    },
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
            Const::Ext {
                leg_idx,
                spin,
                charge,
                incoming,
            } => {
                write!(
                    f,
                    "(ExtLegInfo {leg_idx} {spin} {} {})",
                    charge.sign(),
                    *incoming as i32
                )
            }
            Const::None => write!(f, "(None)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Op;

    /// `Op` ↔ s-expression head token is a bijection: every op round-trips through
    /// `name`/`from_name`, and no two ops share a name.
    #[test]
    fn op_names_roundtrip() {
        let mut seen = HashSet::new();
        for op in Op::ALL {
            assert_eq!(Op::from_name(op.name()), Some(op), "round-trip for {op:?}");
            assert!(
                seen.insert(op.name()),
                "duplicate s-expr name {}",
                op.name()
            );
        }
    }
}
