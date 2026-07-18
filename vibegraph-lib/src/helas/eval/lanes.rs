//! SIMD lane-batched evaluation of `eval_m2`.
//!
//! The evaluator and repr layers are generic over the scalar field `F: Real`.
//! Choosing `F = NumericArray<f64, N>` (an `N`-wide elementwise SIMD array) runs
//! one `eval_m2` pass over `N` phase-space points at once: every elementwise
//! floating-point op (`+ - * /`, `sqrt`, `min`, `max`, `abs`, `signum`) executes
//! the identical scalar operation independently per lane, so each extracted lane
//! is bit-identical to the scalar `eval_m2` at the same point.
//!
//! # Lane-uniformity contract
//!
//! The one way this breaks is a *data-dependent branch on `F`*. Elementwise
//! comparisons on `NumericArray` (`==`, `<`, `>=`, `is_sign_positive`) reduce to a
//! single `bool` (lexicographic over the whole pack), so an `if predicate(F)`
//! evaluates ONE branch for the entire lane pack. If two lanes want different
//! branches the wrong formula is applied to one of them and bit-identity is lost.
//!
//! Every such branch on the `eval_m2` hot path lives in the external-wavefunction
//! builders (`vxxxxx`, `weyl_ixxxxx`) and the vector propagator, and each predicate
//! is *lane-uniform by construction* when all `N` points in a batch share process
//! topology and partonic-CM kinematics (beams exactly along ±z, external masses are
//! card constants broadcast identically across lanes):
//!
//! - **mass forks** (`vmass == 0`, `mass != 0`): the mass is a broadcast card
//!   constant — identical on every lane — so the predicate is uniform.
//! - **on-axis forks** (`pt == 0`, `pp3 > 0`, `px==0 && py==0 && pz<0`): test
//!   whether a leg lies on the z-axis. A given external leg is either a beam
//!   (always on ±z, for every lane) or a produced leg (off-axis for every lane
//!   except the measure-zero exactly-collinear configuration); leg identity is the
//!   same across a batch, so the predicate is uniform.
//! - **at-rest forks** (`pp == 0`): a produced particle exactly at rest — a
//!   measure-zero threshold coordinate. Threshold-*adjacent* points take the
//!   moving branch uniformly (the predicate is an exact `== 0`); only an
//!   exactly-at-rest lane mixed with a moving lane would diverge, which RAMBO /
//!   phase-space sampling does not produce.
//!
//! Callers must therefore batch kinematically-homogeneous points. The bit-identity
//! gate (`eval_m2_lanes_bit_identical`) pins this against the scalar path across all
//! MG-validated processes on random, z-beam, and threshold-adjacent batches.

use numeric_array::generic_array::typenum::Const;
use numeric_array::generic_array::{ConstArrayLength, GenericArray, IntoArrayLength};
use numeric_array::NumericArray;

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;

/// The scalar field for an `N`-wide lane pack: `N` phase-space points fed through
/// one `eval_m2` pass. `N` is a plain `usize` const generic; the
/// `Const<N>: IntoArrayLength` bound bridges it to numeric-array's typenum length.
pub type LaneField<const N: usize> = NumericArray<f64, ConstArrayLength<N>>;

/// Pack one f64 per lane into a single lane value.
#[inline]
pub(super) fn pack<const N: usize>(lanes: [f64; N]) -> LaneField<N>
where
    Const<N>: IntoArrayLength,
{
    NumericArray::new(GenericArray::from_array(lanes))
}

/// Transpose `N` scalar phase-space points — each a slice of the external momenta
/// in `eval_m2` order — into one structure-of-arrays momentum list whose scalar is
/// an `N`-wide lane pack. Every point must carry the same number of legs.
pub(super) fn transpose_points<const N: usize>(
    points: &[&[LorentzVector<f64>]; N],
) -> Vec<LorentzVector<LaneField<N>>>
where
    Const<N>: IntoArrayLength,
    LaneField<N>: Real,
{
    let n_ext = points[0].len();
    (0..n_ext)
        .map(|leg| {
            LorentzVector::new(
                pack(std::array::from_fn(|k| points[k][leg].e())),
                pack(std::array::from_fn(|k| points[k][leg].px())),
                pack(std::array::from_fn(|k| points[k][leg].py())),
                pack(std::array::from_fn(|k| points[k][leg].pz())),
            )
        })
        .collect()
}

/// Extract every lane of a lane-packed result into a plain `[f64; N]`.
#[inline]
pub(super) fn unpack<const N: usize>(lanes: LaneField<N>) -> [f64; N]
where
    Const<N>: IntoArrayLength,
{
    std::array::from_fn(|k| lanes[k])
}
