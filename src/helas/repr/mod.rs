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
//! contracts different leg bundles. The [`intertwiner`] module encodes each
//! orientation as a separate implementor of the [`intertwiner::Intertwiner`]
//! trait, parameterised by the basis type `B`.
//!
//! ## Module structure
//!
//! | Submodule | Contents |
//! |-----------|----------|
//! | [`lorentz`] | `LorentzRepr`, `SpinorRepr`, `VectorRepr`, `ScalarRepr`, `WeylBasis`, `DiracBasis` |
//! | [`color`] | `ColorRepr`, `SU3Fundamental`, `SU3Adjoint`, `ColorSinglet` |
//! | [`intertwiner`] | `Intertwiner`, `GammaL`, `GammaR`, `GammaV`, `SigmaTensor`, `Epsilon` |
//! | [`propagator`] | `Propagator`, `DiracPropagator`, `MasslessVectorPropagator`, `MassiveVectorPropagator`, `ScalarPropagator` |
//! | [`coupling`] | `ColorStructure`, `LorentzStructure`, `Vertex3`, `GaugeVertex` |
//!
//! ## Scalar primitives
//!
//! The [`Real`] trait and the [`C`] type alias are the atomic building blocks
//! used throughout every submodule. They are defined here so that all
//! submodules can import them from `super`.

pub mod color;
pub mod coupling;
pub mod intertwiner;
pub mod lorentz;
pub mod propagator;

pub use color::{ColorRepr, ColorSinglet, SU3Adjoint, SU3Fundamental};
pub use coupling::{ColorStructure, GaugeVertex, LorentzStructure, Vertex3};
pub use intertwiner::{Epsilon, Gamma, GammaL, GammaR, GammaV, Intertwiner, SigmaTensor};
pub use lorentz::{
    Charge, DiracBasis, FourMomentum, LorentzRepr, MinkowskiRep, ScalarField, ScalarRepr,
    SpinorHelicity, SpinorRepr, VectorRepr, WeylBasis,
};
pub use propagator::{
    DiracPropagator, MassiveVectorPropagator, MasslessVectorPropagator, Propagator,
    ScalarPropagator,
};

// ─────────────────────────────────────────────────────────────────────────────
// Scalar primitives — used throughout all submodules
// ─────────────────────────────────────────────────────────────────────────────

/// Blanket trait alias for the real floating-point scalar used throughout.
///
/// Requires [`num_traits::Float`] (arithmetic, sqrt, etc.), [`Copy`], `'static`
/// lifetime (needed for higher-kinded bounds), and [`std::fmt::Debug`] for
/// diagnostic output.
///
/// Both `f32` and `f64` implement this automatically.
pub trait Real: num_traits::Float + Copy + 'static + std::fmt::Debug {}
impl<F: num_traits::Float + Copy + 'static + std::fmt::Debug> Real for F {}

/// Complex number over a [`Real`] scalar. Alias for [`num_complex::Complex`].
pub type C<F> = num_complex::Complex<F>;

/// Lift a real scalar to a complex number with zero imaginary part.
///
/// Convenience shorthand used heavily in wavefunction and vertex code.
#[inline(always)]
pub fn r<F: Real>(x: F) -> C<F> {
    C::new(x, F::zero())
}

/// Lift a real scalar to a purely imaginary complex number: `i·x`.
///
/// Convenience shorthand for `C::new(0, x)`.
#[inline(always)]
pub fn ri<F: Real>(x: F) -> C<F> {
    C::new(F::zero(), x)
}
