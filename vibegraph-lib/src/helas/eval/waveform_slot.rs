//! Runtime wavefunction slot, representing a single particle's wavefunction in a computation.
use crate::helas::repr::{Real, C};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};
use crate::helas::LorentzVector;
use num_traits::Zero;
use std::ops::{Add, Mul};

/// A runtime wavefunction register (holds one particle's wavefunction).
///
/// Fermion slots carry their flow direction in the type: a column (ket, `u`/`v`)
/// current is [`WaveformSlot::FermionIn`] and a row (bra, `ū`/`v̄`) current is
/// [`WaveformSlot::FermionOut`]. An off-shell current produced by a `GammaIout`-style
/// node is flow-in; a `GammaJout`-style node is flow-out. Consumers request the flow
/// they need (see [`WaveformSlot::expect_fermion_in`] / [`WaveformSlot::expect_fermion_out`]),
/// applying the Dirac adjoint only when the topology genuinely needs the opposite flow.
#[derive(Clone, Debug, Copy)]
pub enum WaveformSlot<F: Real> {
    /// Ket (column) Dirac spinor or off-shell fermion current
    FermionIn(InDiracWf<F>),
    /// Bra (row) Dirac spinor or off-shell fermion current
    FermionOut(OutDiracWf<F>),
    /// 4-component polarization / off-shell vector current, index *up* (`ε^μ`).
    /// Every vector current is physical contravariant: external legs, momenta,
    /// vertex producers (`GammaVout`/`MetricVout`/`LowerVout`) and propagated
    /// currents alike.
    Vector(VectorWf<F>),
    /// Scalar amplitude + momentum
    Scalar(ScalarWf<F>),
    /// A bare real constant (mass / width / coefficient) with no momentum. Kept
    /// separate from `Scalar` so real coupling/coefficient chains multiply in `F`
    /// rather than paying the ~2× cost of `C<F>` multiplication.
    Real(F),
    /// Empty slot (not yet computed)
    Empty,
}

impl<F: Real> Add for WaveformSlot<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        use WaveformSlot::*;
        match (self, other) {
            (Empty, x) | (x, Empty) => x,
            // Scalars are summed by value without a momentum-equality check: the only
            // momentum-mismatched scalar sum is the final coherent sum over diagram
            // amplitudes, where the (conserved, ≈0) momentum is physically irrelevant.
            // Within a diagram, summed scalars share their inputs and so their momenta
            // already match bit-for-bit.
            (Scalar(s1), Scalar(s2)) => WaveformSlot::Scalar(ScalarWf {
                value: s1.value + s2.value,
                momentum: s1.momentum,
            }),
            (Vector(v1), Vector(v2)) => {
                assert_eq!(
                    v1.momentum, v2.momentum,
                    "Cannot add vector waveforms with different momenta"
                );
                WaveformSlot::Vector(VectorWf {
                    eps: v1.eps + v2.eps,
                    momentum: v1.momentum,
                })
            }
            (FermionIn(f1), FermionIn(f2)) => {
                assert_eq!(
                    f1.momentum, f2.momentum,
                    "Cannot add fermion waveforms with different momenta"
                );
                WaveformSlot::FermionIn(InDiracWf::from_spinor(f1.spinor + f2.spinor, f1.momentum))
            }
            (FermionOut(f1), FermionOut(f2)) => {
                assert_eq!(
                    f1.momentum, f2.momentum,
                    "Cannot add fermion waveforms with different momenta"
                );
                WaveformSlot::FermionOut(OutDiracWf::from_spinor(
                    f1.spinor + f2.spinor,
                    f1.momentum,
                ))
            }
            _ => panic!("Addition only implemented for matching waveform variants"),
        }
    }
}

impl<F> Mul<WaveformSlot<F>> for C<F>
where
    F: Real,
{
    type Output = WaveformSlot<F>;

    fn mul(self, rhs: WaveformSlot<F>) -> WaveformSlot<F> {
        use WaveformSlot::*;
        match rhs {
            Empty => Empty,
            // A bare real const scaled by a complex factor becomes a complex scalar
            // with no momentum.
            Real(r) => Scalar(ScalarWf {
                value: self * r,
                momentum: LorentzVector::zero(),
            }),
            Scalar(s) => Scalar(ScalarWf {
                value: self * s.value,
                momentum: s.momentum,
            }),
            Vector(v) => Vector(VectorWf {
                eps: v.eps * self,
                momentum: v.momentum,
            }),
            FermionIn(f) => FermionIn(InDiracWf::from_spinor(f.spinor * self, f.momentum)),
            FermionOut(f) => FermionOut(OutDiracWf::from_spinor(f.spinor * self, f.momentum)),
        }
    }
}

impl<F: Real> WaveformSlot<F> {
    pub fn momentum(&self) -> Option<LorentzVector<F>> {
        match self {
            WaveformSlot::FermionIn(f) => Some(f.momentum),
            WaveformSlot::FermionOut(f) => Some(f.momentum),
            WaveformSlot::Vector(v) => Some(v.momentum),
            WaveformSlot::Scalar(s) => Some(s.momentum),
            WaveformSlot::Real(_) => None,
            WaveformSlot::Empty => None,
        }
    }

    /// Extract a flow-in (column / ket) fermion, applying the Dirac adjoint if
    /// the slot holds a flow-out current (the topology asked for the opposite flow).
    pub fn expect_fermion_in(self) -> InDiracWf<F> {
        match self {
            WaveformSlot::FermionIn(f) => f,
            // A fermion line carries one flow throughout. With flow-typed externals
            // (`build_external_slot`) and flow-preserving currents, the flow a
            // consumer needs always matches the slot — a flow-out slot here means
            // the dispatch mis-assigned the flow, so panic instead of silently
            // applying a (physically wrong) mid-line Dirac adjoint.
            WaveformSlot::FermionOut(_) => {
                panic!("expect_fermion_in: slot is flow-OUT (fermion-flow mismatch)")
            }
            _ => panic!("expected a fermion waveform slot"),
        }
    }

    /// Extract a flow-out (row / bra) fermion, applying the Dirac adjoint if
    /// the slot holds a flow-in current (the topology asked for the opposite flow).
    pub fn expect_fermion_out(self) -> OutDiracWf<F> {
        match self {
            WaveformSlot::FermionOut(f) => f,
            // See expect_fermion_in: flow is an enforced invariant, not coerced.
            WaveformSlot::FermionIn(_) => {
                panic!("expect_fermion_out: slot is flow-IN (fermion-flow mismatch)")
            }
            _ => panic!("expected a fermion waveform slot"),
        }
    }
}
