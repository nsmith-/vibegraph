//! Continuation of `x·f(x, Q²)` outside the tabulated grid.
//!
//! # Why there is one at all
//!
//! A grid stops where the fit stopped, not where the physics does. NNPDF23's
//! densities are tabulated to `Q = 10 TeV`, and a per-event factorisation scale
//! on a 13 TeV collider crosses that, so a reader that refuses out-of-grid
//! points cannot run a dynamical scale to completion. LHAPDF's answer is an
//! `Extrapolator` hierarchy selected per set, and since MadGraph reads its
//! densities through LHAPDF, the continuation it picks is part of the reference
//! a cross section has to match — not a local choice.
//!
//! # Which continuation, and what pins each line of it
//!
//! `GridPDF::_loadExtrapolator` takes the `Extrapolator` key off the set's
//! `Info`, which falls back through the set level to `lhapdf.conf`. Neither
//! fetched set names one, and both LHAPDF 6.5.3's and the installed 6.5.6's
//! `lhapdf.conf` carry `Extrapolator: continuation`, so `mkExtrapolator`
//! (`Factories.cc`) builds a `ContinuationExtrapolator`. The oracle dumps the
//! resolved name alongside its values, so this is a checked fact rather than a
//! reading of a config file that a rebuild could change.
//!
//! Each element below is read off `ContinuationExtrapolator::extrapolateXQ2`:
//!
//! - **The grid edges are the *flattened* ones.** The extrapolator reads
//!   `knotarray().xs(0)`, `xs(1)`, `xs(nx-1)` and `q2s(0)`, `q2s(nq-2)`,
//!   `q2s(nq-1)` — one x axis shared by every subgrid, and a Q² axis that is the
//!   bands' knots concatenated. The Q² pair the upper continuation runs through
//!   is therefore the *last band's* top two knots, never a per-band pair.
//! - **Four branches**, one per out-of-range quadrant, and they compose: past
//!   both upper boundaries the Q² continuation runs at each of the two lowest x
//!   knots and the x continuation runs between those two results.
//! - **The straight line is in the logarithm of the coordinate**, and in the
//!   logarithm of the value too when both endpoints exceed `1e-3` — which keeps
//!   a positive density positive however far it is continued, while a value that
//!   is small or of either sign is continued linearly instead
//!   (`_extrapolateLinear`).
//! - **Below the Q² floor** the continuation is a power law
//!   `f(q2Min)·(Q²/Q²ₘᵢₙ)^γ` whose exponent interpolates between the anomalous
//!   dimension `dlog f / dlog Q²` measured at the floor (from a 1% forward
//!   difference, clamped below at `-2.5`) and `1`, so the densities vanish as
//!   `Q² → 0`. A density too small to measure a gradient from takes exponent `1`
//!   directly.
//!
//! # What is refused
//!
//! Only points at which LHAPDF has no continuation either:
//!
//! - **`x` above the last x knot.** `ContinuationExtrapolator` raises
//!   `RangeError` there — it extends the grid below its first x knot and past
//!   both Q² ends, and deliberately not above `xMax`. For a set whose grid runs
//!   to `x = 1` this coincides with an unphysical momentum fraction.
//! - **A point that is not a point**: a non-finite `x` or `Q²`, an `x` at or
//!   below zero (the continuation is a straight line in `ln x`), or a negative
//!   `Q²`. `Q² = 0` is *not* refused: the power law is defined there and returns
//!   exactly zero.

use thiserror::Error;

use crate::helas::repr::Real;

use super::interp::{Bicubic2D, GridEdges, OutOfRange};

/// Endpoint values above which `_extrapolateLinear` runs the straight line
/// through `ln y` instead of `y`.
const LOG_LINEAR_FLOOR: f64 = 1e-3;

/// The forward step, as a fraction of `Q²ₘᵢₙ`, that the low-`Q²` anomalous
/// dimension is measured over, and the point it is measured at. Both spellings
/// are kept because LHAPDF writes both: `1.01*q2Min` for the step and a
/// separate `0.01` for the divisor.
const ANOM_STEP: f64 = 0.01;
const ANOM_STEP_POINT: f64 = 1.01;

/// Densities smaller than this in magnitude at the `Q²` floor carry no usable
/// gradient, and take exponent `1` rather than a ratio of two tiny numbers.
const ANOM_VALUE_FLOOR: f64 = 1e-5;

/// The steepest fall the low-`Q²` power law is allowed to continue.
const ANOM_MIN: f64 = -2.5;

/// An `(x, Q²)` at which no `x·f` reading is defined — neither by interpolation
/// nor by continuation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PdfPointError {
    #[error(
        "PDF evaluation point (x={x}, Q²={q2}) is not a point a density can be read at: x must \
         be a finite number greater than zero and Q² a finite number at or above zero"
    )]
    Unphysical { x: f64, q2: f64 },

    #[error(
        "PDF evaluation point (x={x}, Q²={q2}) lies above the grid's last x knot {x_max}; the \
         continuation extends the grid below its first x knot and past both ends of Q², and has \
         nothing to continue into above x_max"
    )]
    AboveXMax { x: f64, q2: f64, x_max: f64 },

    #[error(transparent)]
    OutOfSupport(#[from] OutOfRange),
}

/// A seam over the out-of-grid continuation, matching LHAPDF's `Extrapolator`
/// hierarchy. Every continuation is assembled from in-range readings taken at
/// the grid's own edge, so an implementation is handed the interpolator the
/// in-range path uses rather than the raw knots.
pub trait Extrapolate2D {
    /// `x·f(x, Q²)` at a point outside `interp`'s support (`pdg` 0 aliases the
    /// gluon 21).
    fn xfx_q2<F: Real, I: Bicubic2D>(
        &self,
        interp: &I,
        pdg: i32,
        x: F,
        q2: F,
    ) -> Result<F, PdfPointError>;
}

/// LHAPDF's `continuation` extrapolator, the one both fetched sets resolve to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Continuation;

impl Extrapolate2D for Continuation {
    fn xfx_q2<F: Real, I: Bicubic2D>(
        &self,
        interp: &I,
        pdg: i32,
        x: F,
        q2: F,
    ) -> Result<F, PdfPointError> {
        let e = interp.edges();
        let xv = x.to_f64().unwrap();
        let q2v = q2.to_f64().unwrap();

        // A flavor the grid does not carry is zero everywhere, in range and out
        // of it alike: LHAPDF returns before choosing between interpolation and
        // continuation at all.
        if !interp.has_flavor(pdg) {
            return Ok(F::zero());
        }

        let x_min = f::<F>(e.x_min);
        let x_min1 = f::<F>(e.x_min1);
        let q2_min = f::<F>(e.q2_min);
        let q2_max = f::<F>(e.q2_max);
        let q2_max1 = f::<F>(e.q2_max1);

        if xv < e.x_min && q2v >= e.q2_min && q2v <= e.q2_max {
            let f_min = interp.xfx_q2(pdg, x_min, q2)?;
            let f_min1 = interp.xfx_q2(pdg, x_min1, q2)?;
            Ok(extrapolate_linear(x, x_min, x_min1, f_min, f_min1))
        } else if xv >= e.x_min && xv <= e.x_max && q2v > e.q2_max {
            let f_max = interp.xfx_q2(pdg, x, q2_max)?;
            let f_max1 = interp.xfx_q2(pdg, x, q2_max1)?;
            Ok(extrapolate_linear(q2, q2_max, q2_max1, f_max, f_max1))
        } else if xv < e.x_min && q2v > e.q2_max {
            // The Q² continuation at each of the two lowest x knots, then the x
            // continuation between the two results.
            let at_x_min = extrapolate_linear(
                q2,
                q2_max,
                q2_max1,
                interp.xfx_q2(pdg, x_min, q2_max)?,
                interp.xfx_q2(pdg, x_min, q2_max1)?,
            );
            let at_x_min1 = extrapolate_linear(
                q2,
                q2_max,
                q2_max1,
                interp.xfx_q2(pdg, x_min1, q2_max)?,
                interp.xfx_q2(pdg, x_min1, q2_max1)?,
            );
            Ok(extrapolate_linear(x, x_min, x_min1, at_x_min, at_x_min1))
        } else if q2v < e.q2_min && xv <= e.x_max {
            let q2_step = f::<F>(ANOM_STEP_POINT * e.q2_min);
            let (at_floor, at_step) = if xv < e.x_min {
                // The two values the gradient is measured between are themselves
                // continued in x before the power law is built from them.
                let floor = extrapolate_linear(
                    x,
                    x_min,
                    x_min1,
                    interp.xfx_q2(pdg, x_min, q2_min)?,
                    interp.xfx_q2(pdg, x_min1, q2_min)?,
                );
                let step = extrapolate_linear(
                    x,
                    x_min,
                    x_min1,
                    interp.xfx_q2(pdg, x_min, q2_step)?,
                    interp.xfx_q2(pdg, x_min1, q2_step)?,
                );
                (floor, step)
            } else {
                (
                    interp.xfx_q2(pdg, x, q2_min)?,
                    interp.xfx_q2(pdg, x, q2_step)?,
                )
            };

            let anom = if at_floor.abs() >= f::<F>(ANOM_VALUE_FLOOR) {
                ((at_step - at_floor) / at_floor / f::<F>(ANOM_STEP)).max(f::<F>(ANOM_MIN))
            } else {
                F::one()
            };
            // The exponent runs from the measured anomalous dimension at the
            // floor to 1 far below it, so the density vanishes as Q² → 0.
            let ratio = q2 / q2_min;
            Ok(at_floor * ratio.powf(anom * q2 / q2_min + F::one() - ratio))
        } else if xv > e.x_max {
            Err(PdfPointError::AboveXMax {
                x: xv,
                q2: q2v,
                x_max: e.x_max,
            })
        } else {
            // Every point outside the support satisfies one of the branches
            // above, so this is the in-range point a caller should not have sent
            // here (LHAPDF's own `LogicError` arm).
            Err(OutOfRange {
                x: xv,
                q2: q2v,
                x_min: e.x_min,
                x_max: e.x_max,
                q2_min: e.q2_min,
                q2_max: e.q2_max,
            }
            .into())
        }
    }
}

/// LHAPDF's `_extrapolateLinear`: the straight line through `(ln xl, yl)` and
/// `(ln xh, yh)`, evaluated at `ln x`.
///
/// Two endpoints comfortably above zero are continued through `ln y` as well, so
/// the result cannot change sign however far it runs; anything else — a small
/// value, a negative one — is continued in `y` directly, where a linear
/// continuation is the only one defined. Note that `xl` and `xh` are the grid
/// edge and the knot *inside* it, so `x` sits outside the pair rather than
/// between them.
fn extrapolate_linear<F: Real>(x: F, xl: F, xh: F, yl: F, yh: F) -> F {
    let t = (x.ln() - xl.ln()) / (xh.ln() - xl.ln());
    if yl > f::<F>(LOG_LINEAR_FLOOR) && yh > f::<F>(LOG_LINEAR_FLOOR) {
        (yl.ln() + t * (yh.ln() - yl.ln())).exp()
    } else {
        yl + t * (yh - yl)
    }
}

/// Cast an `f64` constant into the scalar field `F`.
#[inline(always)]
fn f<F: Real>(v: f64) -> F {
    num_traits::cast(v).unwrap()
}

/// Whether `(x, Q²)` lies inside the flattened grid extent, i.e. whether it is
/// the interpolator's point rather than the continuation's. This is LHAPDF's
/// `KnotArray::inRangeX` / `inRangeQ2` pair, with both edges inclusive.
pub fn in_grid_range(e: &GridEdges, x: f64, q2: f64) -> bool {
    x >= e.x_min && x <= e.x_max && q2 >= e.q2_min && q2 <= e.q2_max
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::grid::SubGrid;
    use crate::pdf::interp::LogBicubic;

    /// Geometrically spaced knots (log-uniform), as PDF grids use.
    fn geomspace(a: f64, b: f64, n: usize) -> Vec<f64> {
        let (la, lb) = (a.ln(), b.ln());
        (0..n)
            .map(|i| (la + (lb - la) * i as f64 / (n - 1) as f64).exp())
            .collect()
    }

    fn subgrid_from_fn(
        x: &[f64],
        q2: &[f64],
        flavors: &[i32],
        val: impl Fn(f64, f64) -> f64,
    ) -> SubGrid {
        let (nx, nq, nf) = (x.len(), q2.len(), flavors.len());
        let mut xf = vec![0.0; nx * nq * nf];
        for (ix, &xv) in x.iter().enumerate() {
            for (iq, &q2v) in q2.iter().enumerate() {
                for ifl in 0..nf {
                    xf[(ix * nq + iq) * nf + ifl] = val(xv.ln(), q2v.ln());
                }
            }
        }
        SubGrid {
            x: x.to_vec(),
            q2: q2.to_vec(),
            flavors: flavors.to_vec(),
            xf,
        }
    }

    const X_KNOTS: (f64, f64) = (1e-5, 1.0);
    const Q2_KNOTS: (f64, f64) = (1.0, 1e6);

    /// `x·f` linear in `(ln x, ln Q²)` and negative everywhere the tests probe,
    /// so `_extrapolateLinear` always takes its `y`-linear branch — the one a
    /// straight line is reproduced exactly by, at any distance past the grid.
    fn linear_grid() -> (LogBicubic, impl Fn(f64, f64) -> f64) {
        let val = |lx: f64, lq: f64| -(50.0 + 0.5 * lx - 0.25 * lq);
        let x = geomspace(X_KNOTS.0, X_KNOTS.1, 8);
        let q2 = geomspace(Q2_KNOTS.0, Q2_KNOTS.1, 8);
        (
            LogBicubic::build(&[subgrid_from_fn(&x, &q2, &[1], val)]),
            val,
        )
    }

    /// `x·f` a positive exponential of the same linear form, scaled so the
    /// endpoint values sit on a chosen side of the `1e-3` branch floor.
    fn exponential_grid(scale: f64) -> LogBicubic {
        let val = move |lx: f64, lq: f64| scale * (0.5 * lx - 0.25 * lq).exp();
        let x = geomspace(X_KNOTS.0, X_KNOTS.1, 8);
        let q2 = geomspace(Q2_KNOTS.0, Q2_KNOTS.1, 8);
        LogBicubic::build(&[subgrid_from_fn(&x, &q2, &[1], val)])
    }

    /// Above the Q² ceiling the continuation is a straight line through the top
    /// two Q² knots, in `ln Q²`. On data that is itself such a line, that line
    /// *is* the data, so the continued value is the analytic one however far
    /// past the grid it is asked for.
    ///
    /// What this pins is the coordinate and the knot pair: a continuation linear
    /// in `Q²` rather than `ln Q²`, or one running through the wrong two knots,
    /// misses the analytic value by far more than the bound a decade out.
    /// Blind spot: linear data agrees with both `_extrapolateLinear` branches
    /// only in the `y`-linear one, which is why the branch *selection* is pinned
    /// separately below rather than here.
    #[test]
    fn above_the_ceiling_the_continuation_is_a_line_in_log_q2() {
        let (interp, val) = linear_grid();
        for &q2 in &[1.000001e6, 2e6, 1e8, 1e12] {
            let got: f64 = Continuation
                .xfx_q2(&interp, 1, 0.01_f64, q2)
                .expect("above the ceiling is a continuation, not a refusal");
            let want = val(0.01_f64.ln(), q2.ln());
            assert!(
                (got - want).abs() <= 1e-9 * want.abs(),
                "Q²={q2}: got {got} want {want}"
            );
        }
    }

    /// Below the x floor the same straight line runs through the two lowest x
    /// knots, in `ln x`.
    #[test]
    fn below_the_x_floor_the_continuation_is_a_line_in_log_x() {
        let (interp, val) = linear_grid();
        for &x in &[9.9e-6, 1e-6, 1e-9] {
            let got: f64 = Continuation
                .xfx_q2(&interp, 1, x, 100.0_f64)
                .expect("below the x floor is a continuation, not a refusal");
            let want = val(x.ln(), 100.0_f64.ln());
            assert!(
                (got - want).abs() <= 1e-9 * want.abs(),
                "x={x}: got {got} want {want}"
            );
        }
    }

    /// Past both upper boundaries the two continuations compose: the Q² line is
    /// run at each of the two lowest x knots, then the x line between those two
    /// results. On data linear in both logs the composition is still analytic.
    ///
    /// The negative control is the whole point: a version that continued in only
    /// one coordinate would return the grid-edge value in the other, and the
    /// margin below is six orders above the agreement bound.
    #[test]
    fn past_both_upper_boundaries_the_two_continuations_compose() {
        let (interp, val) = linear_grid();
        let (x, q2) = (1e-9_f64, 1e12_f64);
        let got: f64 = Continuation
            .xfx_q2(&interp, 1, x, q2)
            .expect("continuation");
        let want = val(x.ln(), q2.ln());
        assert!(
            (got - want).abs() <= 1e-9 * want.abs(),
            "got {got} want {want}"
        );

        let edge_x = val(X_KNOTS.0.ln(), q2.ln());
        let edge_q2 = val(x.ln(), Q2_KNOTS.1.ln());
        assert!(
            (got - edge_x).abs() > 1e-3 * want.abs(),
            "the x continuation did not run: {got} vs the x_min edge {edge_x}"
        );
        assert!(
            (got - edge_q2).abs() > 1e-3 * want.abs(),
            "the Q² continuation did not run: {got} vs the Q²_max edge {edge_q2}"
        );
    }

    /// The two branches of `_extrapolateLinear` are picked by the endpoint
    /// values and by nothing else: two endpoints above `1e-3` are continued
    /// through `ln y`, anything else through `y`.
    ///
    /// Scaling the same exponential shape by `1e-6` moves it across that floor.
    /// A branch-blind continuation would then scale with it exactly; these two
    /// do not even agree in sign, because the log-linear branch cannot leave the
    /// positive half-line while the linear one crosses zero a little past the
    /// grid.
    #[test]
    fn the_log_linear_branch_is_selected_by_the_endpoint_values() {
        const SCALE: f64 = 1e-6;
        let big = exponential_grid(100.0);
        let small = exponential_grid(100.0 * SCALE);
        // On an x knot, so the endpoint values the continuation runs through are
        // the tabulated ones and the log-linear branch is exact on this grid.
        let (x, q2) = (X_KNOTS.0, 1e8_f64);

        let got_big: f64 = Continuation.xfx_q2(&big, 1, x, q2).unwrap();
        let got_small: f64 = Continuation.xfx_q2(&small, 1, x, q2).unwrap();
        let rescaled = got_small / SCALE;
        assert!(
            (got_big - rescaled).abs() > 0.5 * got_big.abs(),
            "both grids took the same branch: {got_big} vs {rescaled}"
        );
        assert!(
            got_big > 0.0,
            "the log-linear continuation went non-positive"
        );
        assert!(
            got_small < 0.0,
            "the linear continuation did not cross zero"
        );

        // And the log-linear branch is exact on a log-linear grid, which is what
        // says it is that branch rather than merely a different one.
        let want = 100.0 * (0.5 * x.ln() - 0.25 * q2.ln()).exp();
        assert!(
            (got_big - want).abs() <= 1e-7 * want,
            "got {got_big} want {want}"
        );
    }

    /// Below the Q² floor the reading is the power law, and both of its guards
    /// are guards: a gradient steeper than `-2.5` is clamped there, and a
    /// density too small to measure a gradient from takes exponent `1` outright
    /// whatever its shape.
    #[test]
    fn the_low_q2_power_law_clamps_its_exponent_and_its_gradient() {
        // The gradient the extrapolator measures is a 1% forward difference read
        // through the interpolator, so a grid too coarse in Q² reports a much
        // milder slope than its data has. This fixture's Q² knots are close
        // enough together that the reading reaches past the clamp, and the first
        // assertion is that precondition rather than an assumption of it.
        const X: f64 = 0.01;
        const Q2_FLOOR: f64 = 1.0;
        let x = geomspace(X_KNOTS.0, X_KNOTS.1, 8);
        let q2 = geomspace(Q2_FLOOR, 10.0, 12);
        let steep = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &[1], |_, lq| (-8.0 * lq).exp())]);

        let at_floor: f64 = steep.xfx_q2(1, X, Q2_FLOOR).unwrap();
        let at_step: f64 = steep.xfx_q2(1, X, ANOM_STEP_POINT * Q2_FLOOR).unwrap();
        let measured = (at_step - at_floor) / at_floor / ANOM_STEP;
        assert!(
            measured < ANOM_MIN,
            "the fixture's gradient {measured} never reaches the clamp"
        );

        let ratio = 0.5;
        let near: f64 = Continuation.xfx_q2(&steep, 1, X, ratio * Q2_FLOOR).unwrap();
        let clamped = at_floor * ratio.powf(ANOM_MIN * ratio + 1.0 - ratio);
        let unclamped = at_floor * ratio.powf(measured * ratio + 1.0 - ratio);
        assert!(
            (near - clamped).abs() <= 1e-12 * clamped.abs(),
            "the gradient is not clamped at {ANOM_MIN}: {near} vs {clamped}"
        );
        assert!(
            (near - unclamped).abs() > 0.1 * near.abs(),
            "the clamp is not observable here: clamped {clamped}, unclamped {unclamped}"
        );

        // Far below the floor the exponent tends to 1, so x·f vanishes linearly
        // in Q² however steep the grid was.
        let deep: f64 = Continuation.xfx_q2(&steep, 1, X, 1e-9 * Q2_FLOOR).unwrap();
        assert!(
            (deep / at_floor / 1e-9 - 1.0).abs() <= 1e-6,
            "the exponent does not tend to 1 far below the floor: {deep} / {at_floor}"
        );

        // The same shape scaled below the value floor takes exponent 1 at once,
        // which changes the reading's shape and not merely its size.
        let tiny = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &[1], |_, lq| {
            1e-9 * (-8.0 * lq).exp()
        })]);
        let tiny_at_floor: f64 = tiny.xfx_q2(1, X, Q2_FLOOR).unwrap();
        let tiny_near: f64 = Continuation.xfx_q2(&tiny, 1, X, ratio * Q2_FLOOR).unwrap();
        assert!(
            (tiny_near - tiny_at_floor * ratio).abs() <= 1e-12 * tiny_at_floor,
            "a sub-floor density did not take exponent 1: {tiny_near}"
        );
    }

    /// `Q² = 0` is the power law's own limit and reads as exactly zero; `x`
    /// above the last knot has no continuation at all and stays a refusal.
    #[test]
    fn the_remaining_refusals_are_the_ones_lhapdf_has_no_reading_for() {
        let (interp, _) = linear_grid();
        assert_eq!(
            Continuation.xfx_q2(&interp, 1, 0.01_f64, 0.0_f64).unwrap(),
            0.0
        );
        let err = Continuation
            .xfx_q2(&interp, 1, 1.5_f64, 100.0_f64)
            .unwrap_err();
        assert!(matches!(err, PdfPointError::AboveXMax { .. }), "{err:?}");
    }

    /// A flavor the grid does not carry is zero outside the grid exactly as it
    /// is inside it, rather than a continuation of nothing.
    #[test]
    fn an_absent_flavor_is_zero_outside_the_grid_too() {
        let (interp, _) = linear_grid();
        assert_eq!(
            Continuation.xfx_q2(&interp, 4, 1e-9_f64, 1e9_f64).unwrap(),
            0.0
        );
    }
}
