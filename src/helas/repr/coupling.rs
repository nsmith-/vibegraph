//! UFO vertex structures: color factor × Lorentz structure × coupling constant.
//!
//! ## Vertex decomposition
//!
//! Every UFO (Universal FeynRules Output) vertex decomposes as a sum of terms
//! of the form:
//!
//! ```text
//! Γ = coupling · ColorStructure ⊗ LorentzStructure
//! ```
//!
//! where:
//! - **coupling** — an overall complex number from the UFO parameter card
//! - **[`ColorStructure`]** — a Clebsch-Gordan tensor in color space
//!   (e.g. `δ_{ij}`, `T^a_{ij}`, `f^{abc}`)
//! - **[`LorentzStructure`]** — a Clebsch-Gordan tensor in Lorentz/spin space
//!   (e.g. `γ^μ`, `σ^{μν}`, `g^{μν}`)
//!
//! This decomposition mirrors how MadGraph5 stores vertices internally and
//! how ALOHA generates the corresponding helicity-amplitude routines.
//!
//! ## [`Vertex3`]
//!
//! A 3-point vertex is parametrised by the three external representation types
//! `R1, R2, R3` (Lorentz) and carries one `ColorStructure`, one
//! `LorentzStructure`, and one coupling constant `C<F>`.
//!
//! The generics `R1, R2, R3` are phantom types that encode the representations
//! at each leg at the type level, preventing the accidental connection of a
//! fermion leg to a vertex that expects a vector.
//!
//! ## [`GaugeVertex`]
//!
//! A color intertwiner: given the color fibers on legs 1 and 2, it produces
//! the color fiber on the outgoing leg 3 by contracting with the appropriate
//! Clebsch-Gordan coefficient tensor for the chosen `ColorStructure`.
//!
//! ## TODO
//! - Implement `GaugeVertex::apply` for each `ColorStructure` variant.
//! - Add 4-point vertex `Vertex4<F, R1, R2, R3, R4>`.
//! - Add `Vertex3::evaluate` method that contracts the full vertex including
//!   Lorentz structure with external wavefunctions.

use super::color::ColorRepr;
use super::{C, Real};

// ─────────────────────────────────────────────────────────────────────────────
// ColorStructure
// ─────────────────────────────────────────────────────────────────────────────

/// The color tensor structure at a vertex.
///
/// Each variant corresponds to one of the SU(3) Clebsch-Gordan tensors that
/// appear in the SM (and most BSM) UFO models. When combined with a
/// [`LorentzStructure`] and a coupling constant, these form a complete vertex.
///
/// ## Notation
/// - `i, j` — fundamental (triplet) indices
/// - `a, b, c` — adjoint (octet) indices
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorStructure {
    /// `δ_{ij}` — identity in the fundamental **3** representation.
    ///
    /// Appears at EW lepton-boson vertices and qq̄Z/γ vertices (color-neutral
    /// boson coupling to a quark pair with no color exchange).
    Delta,

    /// `T^a_{ij}` — SU(3) generator in the fundamental representation.
    ///
    /// The basic QCD quark–quark–gluon vertex color factor.
    /// `Tr[T^a T^b] = δ^{ab}/2`.
    Generator,

    /// `f^{abc}` — antisymmetric SU(3) structure constants.
    ///
    /// Appears at the triple-gluon vertex `g g g`.
    /// Defined by `[T^a, T^b] = i f^{abc} T^c`.
    StructureConstant,

    /// `d^{abc}` — symmetric SU(3) coefficients.
    ///
    /// Defined by `{T^a, T^b} = d^{abc} T^c + δ^{ab} I/3`.
    /// Appears in some BSM models and 4-fermion operators.
    SymmetricCoeff,

    /// `δ_{ab}` — identity in the adjoint **8** representation.
    ///
    /// Appears at gluon-scalar vertices where the scalar is a color adjoint.
    DeltaAdjoint,

    /// `ε_{ijk}` — Levi-Civita tensor in 3 dimensions.
    ///
    /// Appears in diquark-type vertices and some SUSY models.
    /// Completely antisymmetric: `ε_{123} = +1`.
    Epsilon3,
}

// ─────────────────────────────────────────────────────────────────────────────
// LorentzStructure
// ─────────────────────────────────────────────────────────────────────────────

/// The Lorentz tensor structure at a vertex, as defined in UFO model files.
///
/// Each variant corresponds to a distinct combination of gamma matrices,
/// metric tensors, and momentum insertions. The names follow the conventions
/// used in ALOHA and MadGraph5 UFO vertex `lorentz` fields.
///
/// ## Standard Model coverage
/// The SM UFO model uses a subset of these: `ChiralFF`, `VVV`, `VVVV`, `FFS`,
/// `FFV` (= `VectorFF` + `AxialFF` combined via `gc[0]`, `gc[1]`), and `SSS`.
/// The others appear in SMEFT or BSM extensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LorentzStructure {
    /// `γ^μ P_L` — purely left-handed vector coupling to a fermion pair.
    ///
    /// Matrix element: `ψ̄ γ^μ P_L ψ`. Used for `W` boson couplings to
    /// left-handed fermion doublets.
    VectorFF,

    /// `γ^μ P_R` — purely right-handed vector coupling to a fermion pair.
    ///
    /// Matrix element: `ψ̄ γ^μ P_R ψ`.
    AxialFF,

    /// `γ^μ (g_L P_L + g_R P_R)` — general chiral vector coupling.
    ///
    /// The generic fermion–fermion–vector coupling with independent left
    /// and right coupling constants. Used for `Z`, `γ`, and `g` vertices.
    ChiralFF,

    /// `σ^{μν}` — antisymmetric tensor bilinear coupling.
    ///
    /// Tensor operator `i[γ^μ, γ^ν]/4`. Appears in dipole operators,
    /// anomalous magnetic moments, and SMEFT dimension-6 insertions.
    TensorFF,

    /// Scalar (Yukawa-type) coupling to a fermion pair.
    ///
    /// Matrix element: `ψ̄ ψ` (Dirac scalar, parity-even).
    ScalarFF,

    /// Pseudoscalar (Yukawa-type) coupling to a fermion pair.
    ///
    /// Matrix element: `ψ̄ γ^5 ψ` (parity-odd). Appears in CP-violating
    /// and axion-like-particle models.
    PseudoscalarFF,

    /// Three-vector (non-abelian gauge) vertex: `A^μ A^ν A^ρ`.
    ///
    /// The SM `g g g` and `W W V` (V = γ, Z) vertices. Involves three
    /// antisymmetrised metric contractions.
    VVV,

    /// Four-vector (quartic gauge) vertex: `A^μ A^ν A^ρ A^σ`.
    ///
    /// The SM `g g g g` and `W W V V` quartic couplings. The Lorentz
    /// structure involves products of two metric tensors summed over
    /// permutations.
    VVVV,

    /// Two-vector one-scalar vertex: `A^μ A^ν φ`.
    ///
    /// Appears in the Higgs sector (`H W^+ W^-`, `H Z Z`) and in
    /// scalar-extended BSM models.
    VVS,

    /// Three-scalar vertex: `φ₁ φ₂ φ₃`.
    ///
    /// Lorentz-trivial (no Lorentz indices). The complete vertex is just
    /// the coupling constant times the color structure.
    SSS,

    /// Four-scalar vertex: `φ₁ φ₂ φ₃ φ₄`.
    ///
    /// Appears in the Higgs quartic potential and scalar extensions.
    SSSS,

    /// Fermion–fermion–scalar vertex (generic Yukawa).
    ///
    /// Matrix element: `ψ̄ (g_S + g_P γ^5) ψ φ`. Covers both scalar and
    /// pseudoscalar Yukawa with separate coefficients.
    FFS,
}

// ─────────────────────────────────────────────────────────────────────────────
// Vertex3
// ─────────────────────────────────────────────────────────────────────────────

/// A 3-point Feynman vertex: `color × Lorentz × coupling`.
///
/// The type parameters `R1, R2, R3` are phantom types encoding the Lorentz
/// representation at each leg. These prevent connecting mismatched legs at
/// the type level (e.g. a fermion leg to a vector slot).
///
/// In a UFO model, a single physical vertex may correspond to multiple
/// `Vertex3` instances summed together (when the vertex has multiple color ×
/// Lorentz terms, as in the non-abelian gauge sector).
///
/// # Usage
/// ```rust,ignore
/// let v = Vertex3::<f64, WeylBasis, WeylBasis, MinkowskiRep>::new(
///     ColorStructure::Generator,
///     LorentzStructure::ChiralFF,
///     C::new(gs, 0.0),   // g_s coupling
/// );
/// ```
///
/// # TODO
/// Add an `evaluate(wf1, wf2, wf3) -> C<F>` method that contracts the three
/// wavefunctions with the full vertex tensor (color × Lorentz).
pub struct Vertex3<F: Real, R1, R2, R3> {
    /// Color tensor at this vertex.
    pub color: ColorStructure,
    /// Lorentz tensor at this vertex.
    pub lorentz: LorentzStructure,
    /// Overall complex coupling constant (from the UFO parameter card).
    pub coupling: C<F>,
    _phantom: std::marker::PhantomData<fn(R1, R2, R3)>,
}

impl<F: Real, R1, R2, R3> Vertex3<F, R1, R2, R3> {
    /// Construct a new vertex from its components.
    pub fn new(color: ColorStructure, lorentz: LorentzStructure, coupling: C<F>) -> Self {
        Self {
            color,
            lorentz,
            coupling,
            _phantom: std::marker::PhantomData,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GaugeVertex
// ─────────────────────────────────────────────────────────────────────────────

/// A color intertwiner: maps two incoming color fibers to one outgoing fiber.
///
/// Geometrically this is an element of
/// `Hom(C1::Color ⊗ C2::Color, Cout::Color)` — the Clebsch-Gordan map for
/// the chosen `ColorStructure`. Concretely, it contracts the two incoming
/// color indices with the appropriate SU(3) tensor (δ, T^a, f^abc, …) to
/// produce the outgoing color flow.
///
/// # Type parameters
/// - `C1`, `C2` — color representations of the two incoming legs
/// - `Cout` — color representation of the outgoing leg
///
/// # TODO
/// Implement `apply` for each `ColorStructure` variant:
/// - `Delta`: `(c1, c2) → c1 · c2` (dot product for fundamental indices)
/// - `Generator`: contract with `T^a_{ij}` Gell-Mann matrices (stored as const)
/// - `StructureConstant`: contract with `f^{abc}` (store as const 8×8×8 array)
/// - `SymmetricCoeff`: contract with `d^{abc}`
/// - `DeltaAdjoint`: `(c1, c2) → c1 · c2` in the adjoint
/// - `Epsilon3`: `ε_{ijk} c1^j c2^k`
pub struct GaugeVertex<F: Real, C1: ColorRepr<F>, C2: ColorRepr<F>, Cout: ColorRepr<F>> {
    /// Which color tensor is being contracted.
    pub structure: ColorStructure,
    _phantom: std::marker::PhantomData<fn(F, C1, C2) -> Cout>,
}

impl<F: Real, C1: ColorRepr<F>, C2: ColorRepr<F>, Cout: ColorRepr<F>> GaugeVertex<F, C1, C2, Cout> {
    /// Construct a new color intertwiner for the given `ColorStructure`.
    pub fn new(structure: ColorStructure) -> Self {
        Self {
            structure,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Apply the color intertwiner to two incoming color fibers.
    ///
    /// Contracts `c1` and `c2` with the Clebsch-Gordan tensor for
    /// `self.structure`, returning the color fiber on the outgoing leg.
    ///
    /// # TODO
    /// Dispatch on `self.structure` and implement each case using the
    /// explicit SU(3) generator / structure-constant tables.
    pub fn apply(&self, _c1: &C1::Color, _c2: &C2::Color) -> Cout::Color {
        todo!(
            "GaugeVertex::apply — dispatch on ColorStructure and contract with Clebsch-Gordan tensor"
        )
    }
}
