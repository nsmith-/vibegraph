//! # `repr` — Representation layer for HELAS/ALOHA helicity amplitudes
//!
//! This module provides the foundational generic traits and types needed to
//! implement HELAS (Helicity Amplitude Subroutines) and ALOHA (Automatic
//! Libraries Of Helicity Amplitudes) routines in a type-safe, basis-independent
//! way over an arbitrary real scalar field `F`.
//!
//! ## Geometric picture
//!
//! Each external or off-shell particle is a section of a vector bundle over
//! momentum space. The bundle decomposes as a product of a Lorentz bundle and a
//! gauge (color) bundle:
//!
//! | Field | Bundle | Lorentz rep | Color rep |
//! |-------|--------|-------------|-----------|
//! | Left-handed fermion | S_L ⊗ V_q | (½,0) | fund. SU(3) |
//! | Right-handed fermion | S_R ⊗ V_q | (0,½) | fund. SU(3) |
//! | Gauge boson | T\*M ⊗ ad(P_G) | (½,½) | adjoint |
//! | Scalar | triv ⊗ V_q | (0,0) | fund. SU(3) or singlet |
//!
//! Vertex factors such as γ^μ, σ^μν, and ε^μνρσ are *intertwiners*:
//! Spin(1,3)-equivariant linear maps between the fibers of the bundles
//! attached to each leg of a vertex. For example:
//!
//! ```text
//! γ^μ : S_L → T*M ⊗ S_R    (left-chiral spinor to vector ⊗ right-chiral spinor)
//! ```
//!
//! Each vertex intertwiner has multiple *orientations* depending on which legs
//! are incoming vs. outgoing: the same coupling constant appears in all
//! orientations, but the map between fibers changes because each orientation
//! contracts different leg bundles.
//!
//! ## Where the vertex factors live
//!
//! The vertex factors are methods on the [`lorentz`] representation types,
//! which is where each has a concrete basis to be written in.
//!
//! The `(j_L,j_R)` column reads a fermion bilinear `χ̄ Γ ψ` as a tensor product
//! of the chiral blocks it pairs: the bra factor first, then the ket factor,
//! each written as the representation it transforms in, and after the arrow the
//! irreducible piece of that product the vertex factor projects out. With the
//! left-chiral block `ψ_L` (components 0, 1) in `(½,0)` and the right-chiral
//! block `ψ_R` (components 2, 3) in `(0,½)`, conjugation exchanges the two, so
//! `χ_L†` is in `(0,½)` and `χ_R†` in `(½,0)`; `χ̄ = χ†γ⁰` holds `χ_R†` in its
//! components 0, 1 and `χ_L†` in 2, 3. An odd grade (`γ^μ`, `γ^μγ⁵`) pairs
//! blocks of the same chirality, an even grade (`1`, `σ^{μν}`, `γ⁵`) blocks of
//! opposite chirality.
//!
//! | Vertex factor | Map | `(j_L,j_R)` chain | Where |
//! |---------------|-----|-------------------|-------|
//! | `χ̄ P_L ψ`, `χ̄ P_R ψ` | S\* ⊗ S → ℂ | `(½,0)⊗(½,0)→(0,0)`, `(0,½)⊗(0,½)→(0,0)` | [`SpinorRepr::scalar_bilinear`](lorentz::SpinorRepr::scalar_bilinear) |
//! | `χ̄ γ^μ P_L ψ = χ_L† σ̄^μ ψ_L` | S\* ⊗ S → T\*M | `(0,½)⊗(½,0)→(½,½)` | [`SpinorRepr::left_current`](lorentz::SpinorRepr::left_current) |
//! | `χ̄ γ^μ P_R ψ = χ_R† σ^μ ψ_R` | S\* ⊗ S → T\*M | `(½,0)⊗(0,½)→(½,½)` | [`SpinorRepr::right_current`](lorentz::SpinorRepr::right_current) |
//! | `χ̄ γ^μ ψ` | S\* ⊗ S → T\*M | the two rows above, summed | [`SpinorRepr::vector_bilinear`](lorentz::SpinorRepr::vector_bilinear) |
//! | `χ̄ σ^{μν} ψ` | S\* ⊗ S → Λ²T\*M | `(½,0)⊗(½,0)→(1,0)` ⊕ `(0,½)⊗(0,½)→(0,1)` | [`SpinorRepr::tensor_bilinear`](lorentz::SpinorRepr::tensor_bilinear) |
//! | all sixteen `χ̄ Γ_A ψ` | S\* ⊗ S → Cl(1,3)⊗ℂ | every pairing above | [`SpinorRepr::fierz_coefficients`](lorentz::SpinorRepr::fierz_coefficients) |
//! | `ε^{μνρσ}` | (T\*M)³ → T\*M | `(½,½)⊗(½,½)⊗(½,½)→(½,½)` | [`epsilon_vector`](lorentz::epsilon_vector), [`epsilon4`](lorentz::epsilon4) |
//!
//! An arbitrary Clifford element — a γ-chain of any length, held as a
//! [`Multivector`](lorentz::Multivector) — acts on a spinor through
//! [`SpinorRepr::apply`](lorentz::SpinorRepr::apply), so a chain needs no
//! vertex factor of its own.
//!
//! ## Scalar primitives
//!
//! The [`Real`] trait and the [`C`] type alias are the atomic building blocks
//! used throughout every submodule. They are defined here so that all
//! submodules can import them from `super`.

pub mod color;
pub mod lorentz;
pub mod numbers;
pub(crate) mod vectorspace;

/// Blanket trait alias for the real floating-point scalar used throughout.
///
/// Requires:
///    - [`num_traits::Float`] (arithmetic, sqrt, etc.; supplies `Zero`/`One` via
///      its `Num` supertrait, so zero/one are the method-based `F::zero()`/`F::one()`)
///    - [`num_traits::FloatConst`] (for π, etc.)
///    - [`Copy`] these should be cheap to copy for intermediate values
///    - `'static` no non-static references, can be used in trait objects without lifetime parameters
///    - [`std::fmt::Debug`] for diagnostic output.
///    - [`Send`] + [`Sync`] a field element is plain numeric data, so anything
///      built out of it — a phase-space channel, a bound amplitude — can be read
///      from several threads at once. Stating it here is what lets a parallel
///      integrator share one channel map instead of copying it per thread.
///
/// The zero/one bounds are method-based (`Zero`/`One`, inherited through `Float`)
/// rather than the associated-const `ConstZero`/`ConstOne`: a SIMD lane type
/// needs only the method forms (`LaneField` builds its zero by splatting at run
/// time), so batching one `eval_m2` call over several phase-space points needs
/// only this weaker bound.
///
/// Both `f32` and `f64` implement this automatically.
pub trait Real:
    num_traits::Float + num_traits::FloatConst + Copy + 'static + std::fmt::Debug + Send + Sync
{
    /// `self * a + b` as the target computes it fastest: one hardware FMA
    /// (single rounding) where [`HARDWARE_FMA`] holds, a product and a sum
    /// (two roundings) otherwise. `Float::mul_add` always rounds once, which
    /// without FMA hardware means a software FMA per call, several times the
    /// cost of the two-instruction form.
    #[inline(always)]
    #[allow(clippy::disallowed_methods)] // the one sanctioned `Float::mul_add` call
    fn mul_add_fast(self, a: Self, b: Self) -> Self {
        if HARDWARE_FMA {
            self.mul_add(a, b)
        } else {
            self * a + b
        }
    }
}

/// Whether this build's target has a hardware fused multiply-add:
/// x86 with `fma` enabled (`x86-64-v3` and up) or aarch64. It decides what
/// [`Real::mul_add_fast`] computes, so results agree across targets to
/// rounding, and bit for bit only between builds that agree on this flag.
pub(crate) const HARDWARE_FMA: bool = cfg!(any(
    target_feature = "fma",
    all(target_arch = "aarch64", target_feature = "neon")
));
impl<
        F: num_traits::Float + num_traits::FloatConst + Copy + 'static + std::fmt::Debug + Send + Sync,
    > Real for F
{
}

/// Complex number over a [`Real`] scalar. Alias for [`num_complex::Complex`].
pub type C<F> = num_complex::Complex<F>;

/// Lift a real scalar to a complex number with zero imaginary part.
///
/// Convenience shorthand used heavily in wavefunction and vertex code.
#[inline(always)]
pub fn r<F: Real>(x: F) -> C<F> {
    C::from(x)
}

/// Lift a real scalar to a purely imaginary complex number: `i·x`.
///
/// Convenience shorthand for `C::new(0, x)`.
#[inline(always)]
pub(crate) fn ri<F: Real>(x: F) -> C<F> {
    C::i() * x
}
