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
//! # What this interpolator is, and what it is not
//!
//! LHAPDF's `ipol` interpolates the tabulated `(Q, αs)` knots with a cubic in
//! `log Q²`. This reads the same knots with a *linear* interpolant in `log Q²`,
//! which is exact at every knot and differs from the cubic between them — measured
//! at `~1.7e-4` relative mid-interval on `NNPDF23_lo_as_0130_qed`, whose knots are
//! spaced by a factor of `1.2` in `Q`.
//!
//! That is enough for a fixed renormalisation scale sitting on (or beside) a knot
//! and is *not* enough for a dynamical one. At `Q = 91.188`, one part in `4e4` of
//! the way into the knot interval `[91.1876, 109.8541]`, the linear reading lands
//! `1.0e-8` from the value MadGraph's LHAPDF call returns — five orders below any
//! cross-section tolerance. Anywhere else in that interval it would not.
//!
//! Since `log Q² = 2 log Q`, the factor of two cancels out of the interpolation
//! parameter: linear in `log Q²` and linear in `log Q` are the same interpolant.
//! The `log Q²` spelling is kept because it is the one LHAPDF's own `ipol` is
//! defined in.

use thiserror::Error;

use super::grid::AlphaSInfo;

/// The `AlphaS_Type` whose knots this interpolator reads. Any other type means
/// LHAPDF derives `αs` from something else (a `Λ_QCD` value, or an ODE solve), and
/// reading the table anyway would produce a plausible number from the wrong source.
pub const TABULATED_TYPE: &str = "ipol";

#[derive(Debug, Error, PartialEq)]
pub enum GridAlphaSError {
    #[error(
        "PDF set declares AlphaS_Type = '{kind}'; only '{TABULATED_TYPE}' takes its alpha_s from \
         the tabulated knots, so reading them here would report the wrong source"
    )]
    UnsupportedType { kind: String },
    #[error("PDF set tabulates {qs} alpha_s scales against {vals} values")]
    LengthMismatch { qs: usize, vals: usize },
    #[error("PDF set tabulates {n} alpha_s knots; at least two are needed to bracket a scale")]
    TooFewKnots { n: usize },
    #[error("PDF set's alpha_s scale table is not strictly increasing at knot {index}: {q}")]
    NotIncreasing { index: usize, q: f64 },
    #[error("PDF set's alpha_s scale table starts at a non-positive scale {q}")]
    NonPositiveScale { q: f64 },
}

/// A scale outside the set's tabulated `αs` range. Extrapolation is a deliberate
/// non-goal, as it is for the parton densities themselves.
#[derive(Debug, Error, PartialEq)]
#[error("alpha_s requested at Q = {q}, outside the set's tabulated range [{q_min}, {q_max}]")]
pub struct ScaleOutOfRange {
    pub q: f64,
    pub q_min: f64,
    pub q_max: f64,
}

/// `αs(Q)` from a set's `AlphaS_Qs` / `AlphaS_Vals` knots.
#[derive(Clone, Debug, PartialEq)]
pub struct GridAlphaS {
    qs: Vec<f64>,
    ln_q2: Vec<f64>,
    vals: Vec<f64>,
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
        if info.qs.len() < 2 {
            return Err(GridAlphaSError::TooFewKnots { n: info.qs.len() });
        }
        if !(info.qs[0] > 0.0) {
            return Err(GridAlphaSError::NonPositiveScale { q: info.qs[0] });
        }
        for (i, w) in info.qs.windows(2).enumerate() {
            if !(w[1] > w[0]) {
                return Err(GridAlphaSError::NotIncreasing {
                    index: i + 1,
                    q: w[1],
                });
            }
        }
        Ok(GridAlphaS {
            ln_q2: info.qs.iter().map(|q| 2.0 * q.ln()).collect(),
            qs: info.qs.clone(),
            vals: info.vals.clone(),
            mz: info.mz,
        })
    }

    /// Lowest and highest tabulated scale.
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

    /// `αs(q)`, linearly interpolated in `log Q²` between the bracketing knots and
    /// exactly equal to the tabulated value at a knot.
    pub fn try_eval(&self, q: f64) -> Result<f64, ScaleOutOfRange> {
        let (q_min, q_max) = self.q_range();
        // Written so a NaN scale fails rather than indexing on a comparison result.
        if !(q >= q_min && q <= q_max) {
            return Err(ScaleOutOfRange { q, q_min, q_max });
        }
        let upper = self.qs.partition_point(|&k| k <= q);
        let i = upper.clamp(1, self.qs.len() - 1) - 1;
        let t = (2.0 * q.ln() - self.ln_q2[i]) / (self.ln_q2[i + 1] - self.ln_q2[i]);
        Ok((1.0 - t) * self.vals[i] + t * self.vals[i + 1])
    }

    /// Like [`try_eval`](Self::try_eval) but panics on a scale outside the table.
    ///
    /// A cross section that has reached the point of evaluating a coupling has
    /// already committed to the kinematics that produced the scale, so an
    /// out-of-range scale there is a bug in the caller rather than a condition to
    /// recover from — the same stance [`PdfMember::xfx_q2`](super::PdfMember::xfx_q2)
    /// takes on an out-of-grid `(x, Q²)`.
    pub fn eval(&self, q: f64) -> f64 {
        self.try_eval(q).unwrap_or_else(|e| panic!("{e}"))
    }
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

    /// The interpolation variable is `log Q²`, not `Q`: the midpoint of a decade in
    /// `log Q` is `√10 ≈ 3.162`, not `5.5`, and the two readings differ by 20% of
    /// the interval here. A linear-in-`Q` interpolant would pass a knots-only test.
    #[test]
    fn interpolation_is_logarithmic_in_the_scale() {
        let a = two_decade_table();
        assert!((a.eval(10.0f64.sqrt()) - 0.4).abs() < 1e-12);
        let linear_in_q = 0.5 + (10.0f64.sqrt() - 1.0) / 9.0 * (0.3 - 0.5);
        assert!(
            (a.eval(10.0f64.sqrt()) - linear_in_q).abs() > 0.01,
            "the log and linear readings are indistinguishable here"
        );
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

    #[test]
    fn a_scale_outside_the_table_is_refused() {
        let a = two_decade_table();
        assert!(a.try_eval(0.9).is_err());
        assert!(a.try_eval(1001.0).is_err());
        assert!(a.try_eval(f64::NAN).is_err());
        assert!(a.try_eval(1.0).is_ok());
        assert!(a.try_eval(1000.0).is_ok());
    }

    #[test]
    fn a_table_that_cannot_be_read_is_refused_rather_than_guessed() {
        let mut analytic = info(vec![1.0, 10.0], vec![0.5, 0.3]);
        analytic.kind = "analytic".to_string();
        assert_eq!(
            GridAlphaS::from_info(&analytic),
            Err(GridAlphaSError::UnsupportedType {
                kind: "analytic".to_string()
            })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0, 10.0], vec![0.5])),
            Err(GridAlphaSError::LengthMismatch { qs: 2, vals: 1 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0], vec![0.5])),
            Err(GridAlphaSError::TooFewKnots { n: 1 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![0.0, 10.0], vec![0.5, 0.3])),
            Err(GridAlphaSError::NonPositiveScale { q: 0.0 })
        );
        assert_eq!(
            GridAlphaS::from_info(&info(vec![1.0, 10.0, 10.0], vec![0.5, 0.3, 0.2])),
            Err(GridAlphaSError::NotIncreasing { index: 2, q: 10.0 })
        );
    }
}
