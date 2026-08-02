//! `αs(Q)` read from a PDF set's own tabulation, the source LHAPDF's
//! `alphasPDF(Q)` reads when a set carries `AlphaS_Type: ipol`.
//!
//! # Why the grid and not the beta function
//!
//! A PDF set is fitted at a particular strong coupling, and the parton densities
//! and that coupling are one object: using the set's densities with a coupling
//! evolved from somewhere else is a mismatch, not a refinement. MadGraph makes the
//! same call at link time — `pdlabel = lhapdf` links
//! `Source/alfas_functions_lhapdf.f`, whose `ALPHAS(Q)` is a one-line forward to
//! `alphasPDF(Q)` — so for such a run the grid *is* the reference, and
//! [`RunningAlphaS`](crate::coupling::alphas::RunningAlphaS)'s beta-function solve
//! is not applicable however well it is implemented.
//!
//! # The algorithm, and what pins each choice
//!
//! This reproduces LHAPDF 6's `AlphaS_Ipol::alphasQ2`, the routine that call
//! reaches: `LHAGlue.cc`'s `alphasPDF(nset, Q)` forwards to `PDF::alphasQ(Q)`,
//! which is `alphasQ2(q*q)` on the set's `AlphaS` object (`PDF.h`), and for
//! `AlphaS_Type: ipol` that object is `AlphaS_Ipol`. Each element below is the
//! implementation's, not a fit to it:
//!
//! - **Subgrids** (`AlphaS_Ipol::_setup_grids`). The knot table is cut wherever a
//!   `Q²` value repeats — a flavour threshold, where `αs` is tabulated twice with
//!   different values — and each piece is interpolated on its own. The pieces are
//!   keyed by their first `Q²`, so a piece whose first knot repeats an earlier
//!   piece's replaces it, which is what makes a leading duplicate harmless.
//! - **Interpolation variable** `ln Q²`, natural log (`AlphaSArray::_syncq2s`).
//! - **Cubic Hermite** in that variable (`AlphaS_Ipol::_interpolateCubic`), with
//!   endpoint slopes taken as finite differences of the tabulated values:
//!   central inside a subgrid, forward at its first knot and backward at its
//!   last (`AlphaSArray::ddlogq_{forward,central,backward}`). Hermite with
//!   central differences reproduces a quadratic in `ln Q²` exactly and a
//!   one-sided difference does not, which is how the interior and the edge
//!   intervals are told apart in the tests below.
//! - **Above the last knot**, `αs` is frozen at the last tabulated value — a flat
//!   continuation, not an extrapolation of the trend. This is the one line
//!   `if (q2 > _q2s.back()) return _as.back();`, and it is what makes a table that
//!   stops at `Q = 10 TeV` usable at a 13 TeV collider.
//! - **Below the first knot**, a power law in `Q²` whose exponent is the gradient
//!   of the first tabulated interval in the `log₁₀`–`log₁₀` plane, so the reading
//!   is a straight line there.
//!
//! # What is refused
//!
//! Only inputs for which LHAPDF's own reading is undefined: a scale that is not
//! positive and finite, and a table whose subgrids are too short for the cubic.
//! A central slope at the upper end of an interval reads one knot beyond it, so
//! on a two-knot subgrid `AlphaS_Ipol` indexes off the end of its own arrays. A
//! scale outside the tabulated range is *not* refused — both continuations above
//! are part of the algorithm.

use thiserror::Error;

use super::grid::AlphaSInfo;

/// The `AlphaS_Type` whose knots this interpolator reads. Any other type means
/// LHAPDF derives `αs` from something else (a `Λ_QCD` value, or an ODE solve), and
/// reading the table anyway would produce a plausible number from the wrong source.
pub const TABULATED_TYPE: &str = "ipol";

/// The magnitude at which `AlphaS_Ipol::_interpolateCubic` stops reporting its own
/// result. A coupling this large is not physics, and the value that replaces it is
/// deliberately impossible to mistake for one.
const RUNAWAY_ALPHA_S: f64 = 2.0;

#[derive(Debug, Error, PartialEq)]
pub enum GridAlphaSError {
    #[error(
        "PDF set declares AlphaS_Type = '{kind}'; only '{TABULATED_TYPE}' takes its alpha_s from \
         the tabulated knots, so reading them here would report the wrong source"
    )]
    UnsupportedType { kind: String },
    #[error("PDF set tabulates {qs} alpha_s scales against {vals} values")]
    LengthMismatch { qs: usize, vals: usize },
    #[error(
        "PDF set tabulates {n} alpha_s knots; the cubic reads three to bracket a scale and take \
         its slopes"
    )]
    TooFewKnots { n: usize },
    #[error("PDF set's alpha_s scale table falls at knot {index}: {q}")]
    NotSorted { index: usize, q: f64 },
    #[error("PDF set's alpha_s scale table starts at a non-positive scale {q}")]
    NonPositiveScale { q: f64 },
    #[error("PDF set tabulates a non-positive alpha_s value {value} at knot {index}")]
    NonPositiveValue { index: usize, value: f64 },
    #[error(
        "PDF set's alpha_s knots split into a subgrid of {n} at Q = {q}; the cubic reads three, \
         so LHAPDF's own interpolation would run off the end of it"
    )]
    SubgridTooSmall { q: f64, n: usize },
}

/// A scale at which no reading of the table is defined.
///
/// Both continuations outside the tabulated range are part of the algorithm, so
/// this is not about the range: it is a scale that is not a scale.
#[derive(Debug, Error, PartialEq)]
#[error("alpha_s requested at Q = {q}, which is not a positive finite scale")]
pub struct UnusableScale {
    pub q: f64,
}

/// One contiguous run of knots between flavour thresholds, with the cubic's
/// endpoint slopes for each of its intervals precomputed.
#[derive(Clone, Debug, PartialEq)]
struct Subgrid {
    q2s: Vec<f64>,
    ln_q2s: Vec<f64>,
    vals: Vec<f64>,
    intervals: Vec<Interval>,
}

/// The cubic's inputs for one knot interval, in the form the Hermite basis takes
/// them: the slopes already multiplied by the interval's width in `ln Q²`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Interval {
    d_ln_q2: f64,
    m_lo: f64,
    m_hi: f64,
}

impl Subgrid {
    fn new(q2s: Vec<f64>, vals: Vec<f64>) -> Result<Self, GridAlphaSError> {
        let n = q2s.len();
        if n < 3 {
            return Err(GridAlphaSError::SubgridTooSmall {
                q: q2s[0].sqrt(),
                n,
            });
        }
        let ln_q2s: Vec<f64> = q2s.iter().map(|q2| q2.ln()).collect();

        let forward = |i: usize| (vals[i + 1] - vals[i]) / (ln_q2s[i + 1] - ln_q2s[i]);
        let backward = |i: usize| (vals[i] - vals[i - 1]) / (ln_q2s[i] - ln_q2s[i - 1]);
        let central = |i: usize| 0.5 * (forward(i) + backward(i));

        let intervals = (0..n - 1)
            .map(|i| {
                // The lower slope is one-sided only at the subgrid's own first
                // knot, the upper only at its last; everywhere else both are
                // central. `i == 0` is tested first, so a three-knot subgrid —
                // where the first interval is also the last — takes the forward
                // slope below and a central one above.
                let (lo, hi) = if i == 0 {
                    (forward(0), central(1))
                } else if i == n - 2 {
                    (central(i), backward(n - 1))
                } else {
                    (central(i), central(i + 1))
                };
                let d_ln_q2 = ln_q2s[i + 1] - ln_q2s[i];
                Interval {
                    d_ln_q2,
                    m_lo: lo * d_ln_q2,
                    m_hi: hi * d_ln_q2,
                }
            })
            .collect();

        Ok(Subgrid {
            q2s,
            ln_q2s,
            vals,
            intervals,
        })
    }

    /// The index of the knot below `q2`, capped so the last knot selects the
    /// interval that ends on it rather than one that starts there.
    fn interval_below(&self, q2: f64) -> usize {
        let above = self.q2s.partition_point(|&k| k <= q2);
        above.clamp(1, self.q2s.len() - 1) - 1
    }

    fn eval(&self, q2: f64) -> f64 {
        let i = self.interval_below(q2);
        let interval = self.intervals[i];
        let t = (q2.ln() - self.ln_q2s[i]) / interval.d_ln_q2;
        let t2 = t * t;
        let t3 = t2 * t;
        let value = (2.0 * t3 - 3.0 * t2 + 1.0) * self.vals[i]
            + (t3 - 2.0 * t2 + t) * interval.m_lo
            + (-2.0 * t3 + 3.0 * t2) * self.vals[i + 1]
            + (t3 - t2) * interval.m_hi;
        if value.abs() < RUNAWAY_ALPHA_S {
            value
        } else {
            f64::MAX
        }
    }
}

/// `αs(Q)` from a set's `AlphaS_Qs` / `AlphaS_Vals` knots.
#[derive(Clone, Debug, PartialEq)]
pub struct GridAlphaS {
    qs: Vec<f64>,
    q2s: Vec<f64>,
    vals: Vec<f64>,
    subgrids: Vec<Subgrid>,
    mz: f64,
}

impl GridAlphaS {
    /// Build the interpolator from a set's parsed `AlphaS_*` metadata.
    pub fn from_info(info: &AlphaSInfo) -> Result<Self, GridAlphaSError> {
        if info.kind != TABULATED_TYPE {
            return Err(GridAlphaSError::UnsupportedType {
                kind: info.kind.clone(),
            });
        }
        if info.qs.len() != info.vals.len() {
            return Err(GridAlphaSError::LengthMismatch {
                qs: info.qs.len(),
                vals: info.vals.len(),
            });
        }
        if info.qs.len() < 3 {
            return Err(GridAlphaSError::TooFewKnots { n: info.qs.len() });
        }
        if !(info.qs[0] > 0.0) {
            return Err(GridAlphaSError::NonPositiveScale { q: info.qs[0] });
        }
        for (i, w) in info.qs.windows(2).enumerate() {
            if !(w[1] >= w[0]) {
                return Err(GridAlphaSError::NotSorted {
                    index: i + 1,
                    q: w[1],
                });
            }
        }
        // The reading below the table is a power law through the first two
        // distinct knots, taken in the log-log plane, so a non-positive value
        // anywhere would surface there as a NaN rather than as a bad table.
        for (i, &v) in info.vals.iter().enumerate() {
            if !(v > 0.0) {
                return Err(GridAlphaSError::NonPositiveValue { index: i, value: v });
            }
        }

        let q2s: Vec<f64> = info.qs.iter().map(|q| q * q).collect();
        let subgrids = split_subgrids(&q2s, &info.vals)?;
        Ok(GridAlphaS {
            qs: info.qs.clone(),
            q2s,
            vals: info.vals.clone(),
            subgrids,
            mz: info.mz,
        })
    }

    /// Lowest and highest tabulated scale — the range inside which the reading
    /// interpolates rather than continues.
    pub fn q_range(&self) -> (f64, f64) {
        (self.qs[0], self.qs[self.qs.len() - 1])
    }

    /// The set's declared `AlphaS_MZ`. Six printed digits in the sets seen so far,
    /// so it is metadata about the table rather than a value to evaluate at `M_Z`:
    /// [`eval`](Self::eval) is the accurate route to `αs(M_Z)`.
    pub fn declared_mz_value(&self) -> f64 {
        self.mz
    }

    /// Number of tabulated knots.
    pub fn knots(&self) -> usize {
        self.qs.len()
    }

    /// `αs(q)`, for any positive finite scale.
    pub fn try_eval(&self, q: f64) -> Result<f64, UnusableScale> {
        // Written so a NaN scale fails rather than reaching a comparison that
        // would silently take the interpolating branch.
        if !(q > 0.0) || !q.is_finite() {
            return Err(UnusableScale { q });
        }
        Ok(self.at_q2(q * q))
    }

    /// Like [`try_eval`](Self::try_eval) but panics on a scale that is not a
    /// positive finite number.
    ///
    /// A cross section that has reached the point of evaluating a coupling has
    /// already committed to the kinematics that produced the scale, so a scale
    /// that is not a scale there is a bug in the caller rather than a condition to
    /// recover from — the same stance [`PdfMember::xfx_q2`](super::PdfMember::xfx_q2)
    /// takes on an out-of-grid `(x, Q²)`.
    pub fn eval(&self, q: f64) -> f64 {
        self.try_eval(q).unwrap_or_else(|e| panic!("{e}"))
    }

    fn at_q2(&self, q2: f64) -> f64 {
        if q2 < self.q2s[0] {
            return self.below_table(q2);
        }
        if q2 > self.q2s[self.q2s.len() - 1] {
            return self.vals[self.vals.len() - 1];
        }
        let subgrid = self
            .subgrids
            .iter()
            .rev()
            .find(|s| s.q2s[0] <= q2)
            .expect("the first subgrid starts at the table's first knot");
        subgrid.eval(q2)
    }

    /// The power law below the first knot: a straight line through the first two
    /// distinct knots in the `log₁₀ αs` against `log₁₀ Q²` plane.
    fn below_table(&self, q2: f64) -> f64 {
        let next = (1..self.q2s.len())
            .find(|&i| self.q2s[i] != self.q2s[0])
            .expect("a subgrid of three knots has three distinct scales");
        let log_gradient =
            (self.vals[next] / self.vals[0]).log10() / (self.q2s[next] / self.q2s[0]).log10();
        self.vals[0] * (q2 / self.q2s[0]).powf(log_gradient)
    }
}

/// Cut the knot table at every repeated `Q²` and build a subgrid from each piece.
///
/// A repeat is a flavour threshold: `αs` is tabulated twice at the same scale, once
/// on each side, and interpolating across the pair would straddle a zero-width
/// interval. The pieces are indexed by their first `Q²` and a later piece with the
/// same first `Q²` replaces an earlier one, which is what LHAPDF's `map` insertion
/// does and what keeps a table whose *first* knot is itself a threshold readable.
fn split_subgrids(q2s: &[f64], vals: &[f64]) -> Result<Vec<Subgrid>, GridAlphaSError> {
    let mut pieces: Vec<(Vec<f64>, Vec<f64>)> = Vec::new();
    let mut current: (Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new());
    for (i, (&q2, &v)) in q2s.iter().zip(vals).enumerate() {
        if i > 0 && q2 == q2s[i - 1] {
            pieces.push(std::mem::take(&mut current));
        }
        current.0.push(q2);
        current.1.push(v);
    }
    pieces.push(current);
    // Shadowing is resolved before anything is built: a piece that never gets
    // indexed is never interpolated on, so its length is not a defect.
    pieces.dedup_by(|later, earlier| {
        if later.0[0] == earlier.0[0] {
            *earlier = std::mem::take(later);
            true
        } else {
            false
        }
    });
    pieces
        .into_iter()
        .map(|(piece_q2s, piece_vals)| Subgrid::new(piece_q2s, piece_vals))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(qs: Vec<f64>, vals: Vec<f64>) -> AlphaSInfo {
        AlphaSInfo {
            mz: 0.118,
            order_qcd: 0,
            kind: TABULATED_TYPE.to_string(),
            qs,
            vals,
            lambda4: 0.3,
            lambda5: 0.2,
        }
    }

    fn two_decade_table() -> GridAlphaS {
        GridAlphaS::from_info(&info(
            vec![1.0, 10.0, 100.0, 1000.0],
            vec![0.5, 0.3, 0.2, 0.15],
        ))
        .unwrap()
    }

    #[test]
    fn a_knot_returns_its_tabulated_value_exactly() {
        let a = two_decade_table();
        for (q, v) in [(1.0, 0.5), (10.0, 0.3), (100.0, 0.2), (1000.0, 0.15)] {
            assert_eq!(a.eval(q).to_bits(), f64::to_bits(v), "at Q = {q}");
        }
    }

    /// A table of `αs = A + B·ln Q² + C·(ln Q²)²` is reproduced exactly wherever
    /// the cubic takes both its slopes from central differences, because a central
    /// difference of a quadratic *is* its derivative and Hermite with exact
    /// endpoint derivatives is exact for anything cubic or below.
    ///
    /// The two edge intervals are the control: there one slope is a forward or
    /// backward difference, which misses the derivative by `C·Δ`, and the reading
    /// visibly departs from the quadratic. So this one fixture separates three
    /// claims at once — the interpolant is a cubic Hermite, its variable is
    /// `ln Q²`, and its slopes are central inside a subgrid and one-sided at its
    /// ends. A linear reading, a reading in `Q`, or central differences carried
    /// into the edges would each fail a different half of it.
    #[test]
    fn a_quadratic_in_log_q2_is_exact_inside_and_only_inside() {
        let quadratic = |ln_q2: f64| 0.3 - 0.02 * ln_q2 + 0.001 * ln_q2 * ln_q2;
        let qs: Vec<f64> = (0..8).map(|i| 2.0f64.powi(i)).collect();
        let vals: Vec<f64> = qs.iter().map(|q| quadratic(2.0 * q.ln())).collect();
        let a = GridAlphaS::from_info(&info(qs.clone(), vals)).unwrap();

        for i in 0..qs.len() - 1 {
            let q = (qs[i] * qs[i + 1]).sqrt();
            let want = quadratic(2.0 * q.ln());
            let error = (a.eval(q) - want).abs() / want;
            let interior = i > 0 && i + 2 < qs.len();
            if interior {
                assert!(error < 1e-14, "interval {i} at Q = {q}: {error:.2e}");
            } else {
                assert!(error > 1e-4, "edge interval {i} at Q = {q}: {error:.2e}");
            }
        }
    }

    /// Above the last knot the reading is flat, not a continued trend: the
    /// tabulated value comes back bit for bit however far past the table the
    /// scale sits. This is what lets a set whose table stops at 10 TeV be read at
    /// a 13 TeV collider at all.
    #[test]
    fn above_the_table_alpha_s_is_frozen_at_the_last_knot() {
        let a = two_decade_table();
        let last = 0.15;
        for q in [1000.0 * (1.0 + 1e-12), 1300.0, 1.0e6, 1.0e30] {
            assert_eq!(a.eval(q).to_bits(), f64::to_bits(last), "at Q = {q}");
        }
        // Freezing is a property of this end of the table and not a blanket
        // clamp on anything out of range: below the first knot the reading still
        // moves with the scale.
        assert_ne!(a.eval(0.5), a.eval(0.1));
    }

    /// Below the first knot the reading is a straight line in the log-log plane
    /// through the first two knots, so the gradient measured between any two
    /// probes below the table is the gradient of that first interval.
    #[test]
    fn below_the_table_alpha_s_follows_the_first_interval_in_the_log_log_plane() {
        let a = two_decade_table();
        let want = (0.3f64 / 0.5).log10() / (100.0f64 / 1.0).log10();
        let (lo, hi) = (0.01, 0.5);
        let got = (a.eval(hi) / a.eval(lo)).log10() / ((hi * hi) / (lo * lo)).log10();
        assert!((got - want).abs() < 1e-12, "{got} against {want}");
        // And it meets the table at the first knot rather than jumping there.
        assert!((a.eval(1.0 - 1e-12) - 0.5).abs() < 1e-12);
    }

    /// A repeated scale is a flavour threshold: the two knots carry different
    /// values, the interpolation never straddles them, and the repeated scale
    /// itself reads the *upper* subgrid.
    ///
    /// The fixture makes the failure visible rather than small: above the
    /// threshold the tabulated values are constant, which a Hermite cubic on that
    /// subgrid reproduces exactly, so any leakage of the lower subgrid's slope or
    /// value across the seam moves the reading off that constant.
    #[test]
    fn a_repeated_scale_cuts_the_table_into_subgrids() {
        let a = GridAlphaS::from_info(&info(
            vec![1.0, 2.0, 4.0, 4.0, 8.0, 16.0],
            vec![0.5, 0.4, 0.3, 0.2, 0.2, 0.2],
        ))
        .unwrap();
        // On the upper subgrid's own knots the Hermite basis collapses to the
        // tabulated value with nothing left to round; between them the two
        // surviving terms are rounded separately and can miss it by an ulp.
        for q in [4.0, 8.0, 16.0] {
            assert_eq!(a.eval(q).to_bits(), f64::to_bits(0.2), "at the knot {q}");
        }
        for q in [5.0, 12.0] {
            assert!((a.eval(q) - 0.2).abs() < 1e-15, "above the seam at {q}");
        }
        // The lower subgrid still ends on its own value, so the seam is a jump
        // and not a smoothing of the two.
        let below = a.eval(4.0 - 1e-9);
        assert!(
            (below - 0.3).abs() < 1e-8,
            "below the seam the lower subgrid reads {below}"
        );
    }

    /// `AlphaS_Ipol::_interpolateCubic` replaces its own result with the largest
    /// representable double once the magnitude reaches 2 — a coupling that size is
    /// not physics, and the substitute is meant to be unmistakable. Reproduced
    /// because a set that triggers it must not read differently here than it does
    /// through MadGraph.
    #[test]
    fn a_runaway_reading_is_reported_as_lhapdf_reports_it() {
        let a = GridAlphaS::from_info(&info(
            vec![1.0, 10.0, 100.0, 1000.0],
            vec![0.5, 2.5, 0.4, 0.3],
        ))
        .unwrap();
        assert_eq!(a.eval(10.0), f64::MAX);
        assert!(a.eval(1.0) < RUNAWAY_ALPHA_S);
    }

    #[test]
    fn a_scale_that_is_not_a_scale_is_refused() {
        let a = two_decade_table();
        assert_eq!(a.try_eval(0.0), Err(UnusableScale { q: 0.0 }));
        assert_eq!(a.try_eval(-1.0), Err(UnusableScale { q: -1.0 }));
        assert!(a.try_eval(f64::NAN).is_err());
        assert!(a.try_eval(f64::INFINITY).is_err());
        // Outside the tabulated range is not one of these: both continuations
        // are part of the algorithm.
        assert!(a.try_eval(0.9).is_ok());
        assert!(a.try_eval(1e9).is_ok());
    }

    #[test]
    fn a_table_that_cannot_be_read_is_refused_rather_than_guessed() {
        let mut analytic = info(vec![1.0, 10.0, 100.0], vec![0.5, 0.3, 0.2]);
        analytic.kind = "analytic".to_string();
        assert_eq!(
            GridAlphaS::from_info(&analytic),
            Err(GridAlphaSError::UnsupportedType {
                kind: "analytic".to_string()
            })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0, 10.0, 100.0], vec![0.5, 0.3])),
            Err(GridAlphaSError::LengthMismatch { qs: 3, vals: 2 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0, 10.0], vec![0.5, 0.3])),
            Err(GridAlphaSError::TooFewKnots { n: 2 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![0.0, 10.0, 100.0], vec![0.5, 0.3, 0.2])),
            Err(GridAlphaSError::NonPositiveScale { q: 0.0 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0, 100.0, 10.0], vec![0.5, 0.3, 0.2])),
            Err(GridAlphaSError::NotSorted { index: 2, q: 10.0 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0, 10.0, 100.0], vec![0.5, 0.0, 0.2])),
            Err(GridAlphaSError::NonPositiveValue {
                index: 1,
                value: 0.0
            })
        );
        // A threshold that leaves either side with fewer than three knots is
        // where LHAPDF's own cubic would index off the end of that side.
        assert_eq!(
            GridAlphaS::from_info(&info(
                vec![1.0, 10.0, 100.0, 100.0, 1000.0],
                vec![0.5, 0.3, 0.2, 0.19, 0.15]
            )),
            Err(GridAlphaSError::SubgridTooSmall { q: 100.0, n: 2 })
        );
    }

    /// A table whose very first knot repeats is readable: the one-knot piece it
    /// opens with is indexed by the same scale as the piece that follows and is
    /// replaced by it, so nothing ever interpolates on it.
    #[test]
    fn a_leading_repeated_scale_is_shadowed_rather_than_refused() {
        let a = GridAlphaS::from_info(&info(
            vec![1.0, 1.0, 10.0, 100.0, 1000.0],
            vec![0.6, 0.5, 0.3, 0.2, 0.15],
        ))
        .unwrap();
        assert_eq!(a.eval(1.0).to_bits(), f64::to_bits(0.5));
        assert_eq!(a.eval(10.0).to_bits(), f64::to_bits(0.3));
        // The shadowed value is not the one the power law below the table is
        // anchored on either: that gradient is taken to the first *distinct*
        // scale, exactly as `AlphaS_Ipol` skips repeated leading knots.
        let want = (0.3f64 / 0.6).log10() / (100.0f64 / 1.0).log10();
        let (lo, hi) = (0.01, 0.5);
        let got = (a.eval(hi) / a.eval(lo)).log10() / ((hi * hi) / (lo * lo)).log10();
        assert!((got - want).abs() < 1e-12, "{got} against {want}");
    }

    #[test]
    fn every_bracket_is_used_and_the_reading_is_monotone() {
        let a = two_decade_table();
        let mut previous = a.eval(1.0);
        for i in 1..=300 {
            let q = 10.0f64.powf(3.0 * f64::from(i) / 300.0);
            let v = a.eval(q);
            assert!(v <= previous, "not monotone at Q = {q}");
            assert!(v > 0.0 && v.is_finite());
            previous = v;
        }
    }
}
