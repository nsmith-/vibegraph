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
    /// current's variance at runtime (a covariant `MetricVout` current
    /// forms its longitudinal term differently and is raised back), so no separate
    /// lowered-storage opcode is needed.
    Propagate,
    /// Product: ≤1 non-scalar child sets the output type, the rest are scalar
    /// factors. Subsumes scalar×wf scaling (coupling·coeff·current) and the Lorentz
    /// tensor product. Lowering emits arity 2 (multi-factor products reduce as
    /// balanced binary trees for egg); the evaluator itself accepts any arity.
    Mul,
    /// Sum (Lorentz-term, vertex-term, and diagram sums). Lowering emits arity 2
    /// (balanced binary trees); the evaluator itself accepts any arity.
    Add,
    /// Variadic root over per-color-flow JAMPs `(Flows jamp_0 jamp_1 … jamp_{n-1})`,
    /// one child per color-basis element. Modeled on [`Op::PMomOut`]: no leaf, any
    /// number of children.
    Flows,
    /// Variadic root over per-helicity-combination amplitude roots
    /// `(Hels root_0 root_1 … root_{n-1})`, one child per helicity combination (each
    /// child the combination's scalar amplitude, or its `Flows` node for a multi-flow
    /// color basis). Emitted only by the helicity expansion of a folded arena
    /// (`Folded::expand_helicities`); like [`Op::Flows`], it computes nothing itself —
    /// the helicity-summed |M|² reads its children's scalars out of the arena.
    Hels,
    /// Variadic root bundling the amplitude with the per-configuration diagram
    /// amplitudes: `(Configs <amplitude root> A_0 A_1 … A_{k-1})`. Child 0 is the
    /// amplitude root proper (a single JAMP scalar, or the [`Op::Flows`] node for a
    /// multi-flow colour basis); the rest are the colour-stripped amplitudes of the
    /// diagrams MadGraph gives an integration configuration, in configuration order.
    /// Squared and summed over helicities they are MadGraph's `AMP2`, which is what a
    /// per-event configuration draw reads. Like [`Op::Flows`] it computes nothing —
    /// keeping the wires under the root is what keeps their slots live to the end of
    /// the pass, so the values are read out of the arena afterwards.
    Configs,
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
    /// metric with one free index → off-shell vector current.
    MetricVout,
    /// full scalar bilinear ψ̄ δ ψ.
    IdentityAmp,
    /// γ⁵ on a continuing fermion current. Preserves the input's adjoint: γ⁵ is
    /// diagonal in the Weyl basis, so the same weighting acts on a ket from the
    /// left and on a bra from the right.
    Gamma5,
    /// pseudoscalar bilinear ψ̄ γ⁵ ψ.
    Gamma5Amp,
    /// three vectors → off-shell vector current `E^σ = ε^{μνρσ} a_μ b_ν c_ρ`, the
    /// free index last. Operands are the ε arguments with the output slot removed;
    /// the antisymmetry sign of moving that slot to the end is absorbed by the
    /// rooting, which swaps two operands when it is negative.
    EpsilonVout,
    /// four vectors → scalar `ε^{μνρσ} a_μ b_ν c_ρ d_σ`, operands in ε argument
    /// order.
    EpsilonAmp,
    // ── tensor-tensor (cyclic four-fermion) primitives ──
    //
    // A four-fermion structure whose two fermion lines are joined by *two* summed
    // Lorentz indices closes a cycle in the index graph, so no rooted tree can
    // evaluate it by contracting one index at a time. The cycle is cut at one line:
    // that line's `γ^α γ^β` chain is evaluated once, as a Clifford element
    // ([`Op::FierzOut`]), and the element is then applied to the other line — as an
    // operator on its continuing spinor ([`Op::MultivectorIout`] /
    // [`Op::MultivectorOout`]) or as the pairing that closes it into the amplitude
    // ([`Op::FierzPair`]).
    /// two fermions → the cut line's `γ^α γ^β` chain as a Clifford element, in the
    /// index order the *other* line reads it: `4s − 2·(½ t^{μν} σ_{μν})`, where `s`
    /// and `t^{μν}` are the grade-0 and grade-2 bilinears of the pair
    /// (`SpinorRepr::fierz_coefficients`). Children: `[bra, ket]` — the two ends of
    /// the cut line with everything but the two shared gammas already applied.
    /// `γ^αγ^β = g^{αβ} − i σ^{αβ}` is what puts the chain in those two grades, and
    /// `γ_α γ_β g^{αβ} = 4`, `γ_α γ_β t^{αβ} = −i σ_{αβ} t^{αβ}` are the two
    /// contractions the coefficients carry.
    FierzOut,
    /// [`Op::FierzOut`] with the two lines reading the shared indices in opposite
    /// orders (`γ^αγ^β` against `γ_βγ_α`): `4s + 2·(½ t^{μν} σ_{μν})`. Only the
    /// grade-2 sign differs, the whole content of the two index orders.
    FierzOutRev,
    /// Clifford element + flow-in fermion → flow-in fermion current `M ψ`.
    /// Children: `[m, f]`.
    MultivectorIout,
    /// Clifford element + flow-out fermion → flow-out fermion current `ψ̄ M`.
    /// Children: `[m, f]`.
    MultivectorOout,
    /// Clifford element + two fermions → the scalar `ψ̄ M ψ`, evaluated as the
    /// grade-diagonal Fierz pairing `⟨fierz_coefficients(ψ̄, ψ), M⟩`. Children:
    /// `[m, i, j]` with the fermion pair in the vertex's slot order.
    FierzPair,
    // ── literal `Sigma` primitives ──
    //
    // ALOHA's `Sigma` is `½ σ^{μν}`, half the textbook `(i/2)[γ^μ, γ^ν]`
    // ([`kernel::sigma_half`](super::kernel)); every kernel below carries that half.
    // A `Sigma` whose two Lorentz indices are both contracted is a Clifford element
    // ([`Op::SigmaMv`]) and reaches the surviving fermion line through the same
    // [`Op::MultivectorIout`]/[`Op::MultivectorOout`] the tensor⊗tensor contact uses;
    // one free Lorentz index makes it a vector producer instead.
    /// two fermions + a vector → the off-shell vector current `(ψ̄ Σ^{μν} ψ) v_ν`,
    /// the free index on `Sigma`'s *first* Lorentz slot. Children: `[i, j, v]` with
    /// the fermion pair in the vertex's slot order (row first). Reading the pair
    /// against the vertex's defined adjoint conjugates the structure as
    /// `C σ^{μνT} C⁻¹ = −σ^{μν}`, so a reversed line takes a relative −1, exactly as
    /// [`Op::GammaVout`] does.
    SigmaVout,
    /// [`Op::SigmaVout`] with the free index on `Sigma`'s *second* Lorentz slot,
    /// `(ψ̄ Σ^{νμ} ψ) v_ν` — its negative, `σ` being antisymmetric.
    SigmaVoutRev,
    /// two vectors → the Clifford element `Σ^{μν} a_μ b_ν`, grade 2 with coefficient
    /// `½ (a ∧ b)`. Children: `[a, b]` in `Sigma`'s own Lorentz slot order.
    SigmaMv,
    /// two fermions → the cut line of a `Sigma ⊗ Sigma` contact as a Clifford
    /// element: pure grade 2 with coefficient `½ t^{μν}`, where `t` is the pair's
    /// tensor bilinear. Children: `[bra, ket]` — the cut line's two ends with
    /// everything but the shared `Sigma` already applied. It is [`Op::FierzOut`] with
    /// the two gammas already contracted, so the `g^{αβ}` term (and with it the whole
    /// grade-0 part) is gone.
    SigmaOut,
    /// [`Op::SigmaOut`] with the two lines reading the shared indices in opposite
    /// orders: the grade-2 coefficient with the other sign, which is also what a line
    /// read against the vertex's own adjoint takes.
    SigmaOutRev,
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
    /// exact color coefficient (leaf → `consts_f` when real, `consts_c` when it
    /// carries a factor of `i`; the `Nc` power is already evaluated to a rational
    /// by the time this leaf is built).
    CoeffRat,
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
            Op::External | Op::Coupling | Op::Mass | Op::Width | Op::Coeff | Op::CoeffRat
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
    /// `Op::CoeffRat` payload: an exact color coefficient `± i^{imag}·num/den`
    /// (`Nc` already evaluated into the rational by the caller).
    Rational { num: i64, den: i64, imag: bool },
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

/// Which constant pool a [`Const`] leaf indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstKind {
    /// index into `consts_c` (complex pool) — `Op::Coupling`.
    Complex,
    /// index into `consts_f` (real pool) — `Op::Mass` / `Op::Width` / `Op::Coeff`.
    Real,
    /// index into the folded external-leg table — `Op::External`.
    Ext,
    /// Non-leaf op: no payload.
    None,
}

/// Folded leaf payload: a deduped index into the `C<F>` / `F` constant pools tagged
/// with its [`ConstKind`], packed into a single `u32` (kind in the top 2 bits, index
/// in the low 30). This keeps [`Node<Const>`](Node) at 8 bytes — the opcode's own
/// alignment padding absorbs the kind, so the leaf costs no extra word beyond the
/// index. The `External` leg details live in the folded leg table (see
/// [`super::fold::ExtLeg`]).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Const(u32);

impl Const {
    const KIND_SHIFT: u32 = 30;
    const INDEX_MASK: u32 = (1 << Self::KIND_SHIFT) - 1;

    /// Non-leaf op: no payload.
    pub const NONE: Const = Const(0);

    fn packed(kind_bits: u32, index: u32) -> Const {
        assert!(
            index <= Self::INDEX_MASK,
            "constant-pool index {index} overflows the 30-bit Const payload"
        );
        Const((kind_bits << Self::KIND_SHIFT) | (index & Self::INDEX_MASK))
    }

    /// A complex-pool (`consts_c`) index.
    pub fn complex(index: u32) -> Const {
        Const::packed(1, index)
    }

    /// A real-pool (`consts_f`) index.
    pub fn real(index: u32) -> Const {
        Const::packed(2, index)
    }

    /// A folded external-leg-table index.
    pub fn ext(index: u32) -> Const {
        Const::packed(3, index)
    }

    /// Which pool this leaf indexes.
    pub fn kind(self) -> ConstKind {
        match self.0 >> Self::KIND_SHIFT {
            1 => ConstKind::Complex,
            2 => ConstKind::Real,
            3 => ConstKind::Ext,
            _ => ConstKind::None,
        }
    }

    /// The pool index (meaningless for [`ConstKind::None`]).
    pub fn index(self) -> u32 {
        self.0 & Self::INDEX_MASK
    }
}

impl fmt::Debug for Const {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind() {
            ConstKind::Complex => write!(f, "Complex({})", self.index()),
            ConstKind::Real => write!(f, "Real({})", self.index()),
            ConstKind::Ext => write!(f, "Ext({})", self.index()),
            ConstKind::None => write!(f, "None"),
        }
    }
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
            Sym::Rational { num, den, imag } => {
                write!(f, "(Rational {num} {den} {})", *imag as i32)
            }
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
        match self.kind() {
            ConstKind::Complex => write!(f, "(Complex {})", self.index()),
            ConstKind::Real => write!(f, "(Real {})", self.index()),
            ConstKind::Ext => write!(f, "(Ext {})", self.index()),
            ConstKind::None => write!(f, "(None)"),
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
