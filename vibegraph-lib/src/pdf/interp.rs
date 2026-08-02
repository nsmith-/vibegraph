//! Local log-bicubic PDF interpolation in `(ln x, ln Q²)`.
//!
//! This replicates LHAPDF6's `LogBicubicInterpolator` (the `lhagrid1` default
//! interpolator, `logcubic`): a *local* cubic Hermite in each of `ln x` and
//! `ln Q²`, with knot derivatives estimated by finite differences of the
//! `x·f` values. It is deliberately not a scipy-style global B-spline: a global
//! spline is a different algorithm and does not reproduce LHAPDF (and hence the
//! MadGraph cross section it feeds) off-knot.
//!
//! The x-direction cubic is precomputed once per member as four polynomial
//! coefficients `[a, b, c, d]` per `(x-interval, Q²-knot, flavor)`, matching
//! LHAPDF's `KnotArray::coeff` layout. The Q²-direction cubic is assembled at
//! evaluation time from the x-interpolated values on the four surrounding Q²
//! knots (Hermite with central/one-sided finite-difference slopes).
//!
//! Coefficient and knot tables are held in `f64`; evaluation is generic over
//! [`Real`] and casts the tables into `F` per point, so the same tables serve
//! any scalar field. Index lookup uses the `f64` value of the query point.

use crate::helas::repr::Real;

use super::grid::SubGrid;
use super::normalize_flavor_pdg;

/// Raised when an evaluation point lies inside the grid's overall extent but in
/// none of its subgrids — a gap between bands, which a well-formed `lhagrid1`
/// member does not have. Points *outside* the extent are not this: they are
/// continued by [`super::extrap`].
#[derive(Debug, Clone, PartialEq)]
pub struct OutOfRange {
    pub x: f64,
    pub q2: f64,
    pub x_min: f64,
    pub x_max: f64,
    pub q2_min: f64,
    pub q2_max: f64,
}

impl std::fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PDF evaluation point (x={}, Q²={}) falls in no subgrid of a member \
             whose support is x∈[{}, {}], Q²∈[{}, {}]",
            self.x, self.q2, self.x_min, self.x_max, self.q2_min, self.q2_max
        )
    }
}

impl std::error::Error for OutOfRange {}

/// The grid knots an out-of-range continuation is built from, in LHAPDF's
/// flattened layout: one x axis shared by every subgrid, and a Q² axis that is
/// the bands' knots concatenated in order. `q2_max1` is therefore the
/// second-to-last entry of that concatenation — the *last* band's penultimate
/// knot — and not a per-band quantity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridEdges {
    pub x_min: f64,
    /// The second x knot: the continuation below `x_min` is the line through
    /// the first two.
    pub x_min1: f64,
    pub x_max: f64,
    pub q2_min: f64,
    /// The second-to-last knot of the flattened Q² axis.
    pub q2_max1: f64,
    pub q2_max: f64,
}

/// A minimal seam over the 2D interpolation backend, so the evaluation
/// algorithm can be swapped without touching the [`super::PdfMember`] API.
/// The single implementation is the LHAPDF-matching [`LogBicubic`].
pub trait Bicubic2D {
    /// `x·f(x, Q²)` for PDG code `pdg` (0 aliases the gluon 21). Callers pass
    /// in-range points; a point in no subgrid is an [`OutOfRange`].
    fn xfx_q2<F: Real>(&self, pdg: i32, x: F, q2: F) -> Result<F, OutOfRange>;

    /// The grid's own edge knots, which is what a continuation past them needs.
    fn edges(&self) -> GridEdges;

    /// Whether the grid carries `pdg` at all (0 aliases the gluon 21). An absent
    /// flavor is exactly zero everywhere rather than a value to continue.
    fn has_flavor(&self, pdg: i32) -> bool;
}

/// The precomputed log-bicubic coefficient tables for one member, one subgrid
/// per Q² band.
#[derive(Debug, Clone)]
pub struct LogBicubic {
    subgrids: Vec<LogBicubicSubgrid>,
    edges: GridEdges,
}

/// One subgrid's precomputed x-direction cubic coefficients plus the log-knot
/// coordinates needed for the Q²-direction Hermite assembly.
#[derive(Debug, Clone)]
struct LogBicubicSubgrid {
    flavors: Vec<i32>,
    logxs: Vec<f64>,
    logq2s: Vec<f64>,
    x_min: f64,
    x_max: f64,
    q2_min: f64,
    q2_max: f64,
    nq: usize,
    nf: usize,
    /// x-direction cubic coefficients, shape `(nx-1, nq, nf, 4)` row-major:
    /// `coeffs[(((ix*nq + iq)*nf + ifl)*4) + k]`, `k` in `{a, b, c, d}` with the
    /// cubic `a·t³ + b·t² + c·t + d`.
    coeffs: Vec<f64>,
    /// Raw `x·f` grid values, shape `(nx, nq, nf)` row-major. Used only by the
    /// bilinear fallback for degenerate (two-Q²-knot) bands.
    xf: Vec<f64>,
}

impl LogBicubic {
    /// Precompute the log-bicubic coefficients for every subgrid of a member.
    pub fn build(subgrids: &[SubGrid]) -> Self {
        LogBicubic {
            subgrids: subgrids.iter().map(LogBicubicSubgrid::build).collect(),
            edges: edges_of(subgrids),
        }
    }
}

/// The flattened edge knots, assembled exactly as LHAPDF's `KnotArray` holds
/// them: the x axis off the first band (every band shares it) and the Q² axis as
/// the bands' knots concatenated. A grid too small to name an edge reports NaN
/// there, which fails every comparison and so leaves no point in range.
fn edges_of(subgrids: &[SubGrid]) -> GridEdges {
    let xs: &[f64] = subgrids.first().map(|sg| sg.x.as_slice()).unwrap_or(&[]);
    let flat_q2: Vec<f64> = subgrids
        .iter()
        .flat_map(|sg| sg.q2.iter().copied())
        .collect();
    let nq = flat_q2.len();
    GridEdges {
        x_min: xs.first().copied().unwrap_or(f64::NAN),
        x_min1: xs.get(1).copied().unwrap_or(f64::NAN),
        x_max: xs.last().copied().unwrap_or(f64::NAN),
        q2_min: flat_q2.first().copied().unwrap_or(f64::NAN),
        q2_max1: if nq >= 2 { flat_q2[nq - 2] } else { f64::NAN },
        q2_max: flat_q2.last().copied().unwrap_or(f64::NAN),
    }
}

impl Bicubic2D for LogBicubic {
    fn xfx_q2<F: Real>(&self, pdg: i32, x: F, q2: F) -> Result<F, OutOfRange> {
        let pdg = normalize_flavor_pdg(pdg);
        let xf = x.to_f64().unwrap();
        let q2f = q2.to_f64().unwrap();

        // Subgrid walk: first band whose (x, Q²) support contains the point.
        for sg in &self.subgrids {
            if xf >= sg.x_min && xf <= sg.x_max && q2f >= sg.q2_min && q2f <= sg.q2_max {
                let Some(ifl) = sg.flavors.iter().position(|&f| f == pdg) else {
                    // Flavor absent from the grid: LHAPDF returns exactly zero.
                    return Ok(F::zero());
                };
                return Ok(sg.eval(ifl, x, q2, xf, q2f));
            }
        }

        // Point in no band: report the union extent for a useful message.
        let x_min = self
            .subgrids
            .iter()
            .map(|s| s.x_min)
            .fold(f64::INFINITY, f64::min);
        let x_max = self
            .subgrids
            .iter()
            .map(|s| s.x_max)
            .fold(f64::NEG_INFINITY, f64::max);
        let q2_min = self
            .subgrids
            .iter()
            .map(|s| s.q2_min)
            .fold(f64::INFINITY, f64::min);
        let q2_max = self
            .subgrids
            .iter()
            .map(|s| s.q2_max)
            .fold(f64::NEG_INFINITY, f64::max);
        Err(OutOfRange {
            x: xf,
            q2: q2f,
            x_min,
            x_max,
            q2_min,
            q2_max,
        })
    }

    fn edges(&self) -> GridEdges {
        self.edges
    }

    fn has_flavor(&self, pdg: i32) -> bool {
        let pdg = normalize_flavor_pdg(pdg);
        self.subgrids
            .iter()
            .any(|sg| sg.flavors.iter().any(|&f| f == pdg))
    }
}

/// Largest knot index `i` with `knots[i] <= value`, clamped to `len-2` so
/// `i` and `i+1` are always valid. Mirrors LHAPDF's `indexbelow`. Callers only
/// pass in-range values (`>= knots[0]`); a below-range value clamps to interval
/// 0 rather than underflowing.
fn index_below(value: f64, knots: &[f64]) -> usize {
    let n = knots.len();
    // upper_bound: first index strictly greater than value.
    let mut i = knots.partition_point(|&k| k <= value);
    if i == n {
        i -= 1;
    }
    i.saturating_sub(1)
}

impl LogBicubicSubgrid {
    fn build(sg: &SubGrid) -> Self {
        let nx = sg.nx();
        let nq = sg.nq();
        let nf = sg.nf();
        let logxs: Vec<f64> = sg.x.iter().map(|&v| v.ln()).collect();
        let logq2s: Vec<f64> = sg.q2.iter().map(|&v| v.ln()).collect();

        // x-direction cubic coefficients, computed in log-x space exactly as
        // LHAPDF's GridPDF::_computePolynomialCoefficients (logspace=true).
        let mut coeffs = vec![0.0; (nx - 1) * nq * nf * 4];
        for ix in 0..nx - 1 {
            let dlogx = logxs[ix + 1] - logxs[ix];
            for iq in 0..nq {
                for ifl in 0..nf {
                    let vl = sg.xf_at(ix, iq, ifl);
                    let vh = sg.xf_at(ix + 1, iq, ifl);
                    let vdl = ddx_logx(sg, &logxs, ix, iq, ifl) * dlogx;
                    let vdh = ddx_logx(sg, &logxs, ix + 1, iq, ifl) * dlogx;

                    let a = vdh + vdl - 2.0 * vh + 2.0 * vl;
                    let b = 3.0 * vh - 3.0 * vl - 2.0 * vdl - vdh;
                    let c = vdl;
                    let d = vl;

                    let base = ((ix * nq + iq) * nf + ifl) * 4;
                    coeffs[base] = a;
                    coeffs[base + 1] = b;
                    coeffs[base + 2] = c;
                    coeffs[base + 3] = d;
                }
            }
        }

        LogBicubicSubgrid {
            flavors: sg.flavors.clone(),
            logxs,
            logq2s,
            x_min: sg.x[0],
            x_max: sg.x[nx - 1],
            q2_min: sg.q2[0],
            q2_max: sg.q2[nq - 1],
            nq,
            nf,
            coeffs,
            xf: sg.xf.clone(),
        }
    }

    /// The four x-cubic coefficients `[a, b, c, d]` for interval `ix`, Q²-knot
    /// `iq`, flavor `ifl`.
    #[inline]
    fn coeff(&self, ix: usize, iq: usize, ifl: usize) -> &[f64] {
        let base = ((ix * self.nq + iq) * self.nf + ifl) * 4;
        &self.coeffs[base..base + 4]
    }

    /// Evaluate `x·f` at `(x, Q²)`. `xf`/`q2f` are the `f64` values of the
    /// (possibly non-`f64`) query point, used only for the knot index lookup.
    fn eval<F: Real>(&self, ifl: usize, x: F, q2: F, xf: f64, q2f: f64) -> F {
        // ln is monotone, so locating the interval on the log-knots is identical
        // to locating it on the raw knots; use the log-knots directly.
        let ix = index_below(xf.ln(), &self.logxs);
        let iq = index_below(q2f.ln(), &self.logq2s);

        let logx = x.ln();
        let logq2 = q2.ln();

        let tlogx = (logx - f(self.logxs[ix])) / f(self.logxs[ix + 1] - self.logxs[ix]);

        // x-interpolated values on the two bracketing Q² knots.
        let vl = cubic_x(tlogx, self.coeff(ix, iq, ifl));
        let vh = cubic_x(tlogx, self.coeff(ix, iq + 1, ifl));

        // Fallback to bilinear only when both Q² edges are grid edges (i.e. the
        // band has effectively two Q² knots) — LHAPDF's degenerate case.
        let q2_lower = iq == 0;
        let q2_upper = iq + 1 == self.nq - 1;
        if q2_lower && q2_upper {
            let f_ql = interp_linear(
                logx,
                f(self.logxs[ix]),
                f(self.logxs[ix + 1]),
                f(self.xf_raw(ix, iq, ifl)),
                f(self.xf_raw(ix + 1, iq, ifl)),
            );
            let f_qh = interp_linear(
                logx,
                f(self.logxs[ix]),
                f(self.logxs[ix + 1]),
                f(self.xf_raw(ix, iq + 1, ifl)),
                f(self.xf_raw(ix + 1, iq + 1, ifl)),
            );
            return interp_linear(
                logq2,
                f(self.logq2s[iq]),
                f(self.logq2s[iq + 1]),
                f_ql,
                f_qh,
            );
        }

        let dlogq_1 = f(self.logq2s[iq + 1] - self.logq2s[iq]);
        let tlogq = (logq2 - f(self.logq2s[iq])) / dlogq_1;

        let (vdl, vdh) = if q2_lower {
            // Forward difference at the lower Q² edge, central above.
            let vdl = vh - vl;
            let vhh = cubic_x(tlogx, self.coeff(ix, iq + 2, ifl));
            let dlogq_2 = f(1.0 / (self.logq2s[iq + 2] - self.logq2s[iq + 1]));
            let vdh = (vdl + (vhh - vh) * dlogq_1 * dlogq_2) * f(0.5);
            (vdl, vdh)
        } else if q2_upper {
            // Backward difference at the upper Q² edge, central below.
            let vdh = vh - vl;
            let vll = cubic_x(tlogx, self.coeff(ix, iq - 1, ifl));
            let dlogq_0 = f(1.0 / (self.logq2s[iq] - self.logq2s[iq - 1]));
            let vdl = (vdh + (vl - vll) * dlogq_1 * dlogq_0) * f(0.5);
            (vdl, vdh)
        } else {
            // Central differences on both sides.
            let vll = cubic_x(tlogx, self.coeff(ix, iq - 1, ifl));
            let dlogq_0 = f(1.0 / (self.logq2s[iq] - self.logq2s[iq - 1]));
            let vdl = ((vh - vl) + (vl - vll) * dlogq_1 * dlogq_0) * f(0.5);
            let vhh = cubic_x(tlogx, self.coeff(ix, iq + 2, ifl));
            let dlogq_2 = f(1.0 / (self.logq2s[iq + 2] - self.logq2s[iq + 1]));
            let vdh = ((vh - vl) + (vhh - vh) * dlogq_1 * dlogq_2) * f(0.5);
            (vdl, vdh)
        };

        cubic_hermite(tlogq, vl, vdl, vh, vdh)
    }

    /// Raw `x·f` at grid indices `(ix, iq)`, flavor `ifl`.
    #[inline]
    fn xf_raw(&self, ix: usize, iq: usize, ifl: usize) -> f64 {
        self.xf[(ix * self.nq + iq) * self.nf + ifl]
    }
}

/// Central/one-sided finite-difference slope of `x·f` in log-x at knot `ix`,
/// mirroring LHAPDF's `_ddx` (logspace).
fn ddx_logx(sg: &SubGrid, logxs: &[f64], ix: usize, iq: usize, ifl: usize) -> f64 {
    let nx = logxs.len();
    if ix == 0 {
        let del2 = logxs[1] - logxs[0];
        (sg.xf_at(1, iq, ifl) - sg.xf_at(0, iq, ifl)) / del2
    } else if ix == nx - 1 {
        let del1 = logxs[nx - 1] - logxs[nx - 2];
        (sg.xf_at(nx - 1, iq, ifl) - sg.xf_at(nx - 2, iq, ifl)) / del1
    } else {
        let del1 = logxs[ix] - logxs[ix - 1];
        let del2 = logxs[ix + 1] - logxs[ix];
        let lddx = (sg.xf_at(ix, iq, ifl) - sg.xf_at(ix - 1, iq, ifl)) / del1;
        let rddx = (sg.xf_at(ix + 1, iq, ifl) - sg.xf_at(ix, iq, ifl)) / del2;
        (lddx + rddx) / 2.0
    }
}

/// Cast an `f64` constant into the scalar field `F`.
#[inline(always)]
fn f<F: Real>(v: f64) -> F {
    num_traits::cast(v).unwrap()
}

/// Cubic from stored coefficients `[a, b, c, d]`: `a·t³ + b·t² + c·t + d`.
/// Operation order mirrors LHAPDF's `_interpolateCubic(t, coeffs)`.
#[inline]
fn cubic_x<F: Real>(t: F, coeffs: &[f64]) -> F {
    let t2 = t * t;
    let t3 = t2 * t;
    f::<F>(coeffs[0]) * t3 + f::<F>(coeffs[1]) * t2 + f::<F>(coeffs[2]) * t + f::<F>(coeffs[3])
}

/// Cubic Hermite on `[0,1]` from edge values `vl, vh` and edge slopes
/// `vdl, vdh` (already scaled by the interval width). Operation order mirrors
/// LHAPDF's `_interpolateCubic(t, vl, vdl, vh, vdh)`.
#[inline]
fn cubic_hermite<F: Real>(t: F, vl: F, vdl: F, vh: F, vdh: F) -> F {
    let t2 = t * t;
    let t3 = t * t2;
    let two = f::<F>(2.0);
    let three = f::<F>(3.0);
    let p0 = (two * t3 - three * t2 + F::one()) * vl;
    let m0 = (t3 - two * t2 + t) * vdl;
    let p1 = (-two * t3 + three * t2) * vh;
    let m1 = (t3 - t2) * vdh;
    p0 + m0 + p1 + m1
}

/// Linear interpolation `yl + (x-xl)/(xh-xl)·(yh-yl)`.
#[inline]
fn interp_linear<F: Real>(x: F, xl: F, xh: F, yl: F, yh: F) -> F {
    yl + (x - xl) / (xh - xl) * (yh - yl)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Geometrically spaced knots (log-uniform), as PDF grids use.
    fn geomspace(a: f64, b: f64, n: usize) -> Vec<f64> {
        let (la, lb) = (a.ln(), b.ln());
        (0..n)
            .map(|i| (la + (lb - la) * i as f64 / (n - 1) as f64).exp())
            .collect()
    }

    /// A subgrid whose `x·f` is a chosen function of `(ln x, ln Q²)` per flavor.
    fn subgrid_from_fn(
        x: &[f64],
        q2: &[f64],
        flavors: &[i32],
        val: impl Fn(f64, f64, i32) -> f64,
    ) -> SubGrid {
        let (nx, nq, nf) = (x.len(), q2.len(), flavors.len());
        let mut xf = vec![0.0; nx * nq * nf];
        for (ix, &xv) in x.iter().enumerate() {
            for (iq, &q2v) in q2.iter().enumerate() {
                for (ifl, &fl) in flavors.iter().enumerate() {
                    xf[(ix * nq + iq) * nf + ifl] = val(xv.ln(), q2v.ln(), fl);
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

    /// A local cubic Hermite with finite-difference derivatives is exact for
    /// data that is *linear* in each log coordinate (linear ⊂ cubic, and the
    /// FD slope of a linear function is exact). Pinning an analytic value at an
    /// off-knot point is an oracle independent of LHAPDF: it would fail if the
    /// coordinate were mistakenly linear-in-x (not ln x), if the x/Q² axes were
    /// transposed, or if the interval lookup were off by one.
    #[test]
    fn bilinear_in_log_is_reproduced_exactly() {
        let x = geomspace(1e-5, 1.0, 8);
        let q2 = geomspace(1.0, 1e6, 8);
        let flavors = vec![1, 2, 21];
        // x·f = a + b·ln x + c·ln Q², with a per-flavor offset.
        let val = |lx: f64, lq: f64, fl: i32| 3.0 + 0.5 * lx - 0.25 * lq + fl as f64;
        let interp = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &flavors, val)]);

        for &(xv, q2v) in &[(3.3e-4, 55.0), (0.11, 9.9e3), (7e-5, 2.0), (0.9, 8e5)] {
            for &fl in &[1, 2, 21] {
                let got: f64 = interp.xfx_q2(fl, xv, q2v).unwrap();
                let want = val(xv.ln(), q2v.ln(), fl);
                assert!(
                    (got - want).abs() <= 1e-12 * want.abs().max(1.0),
                    "fl={fl} x={xv} Q²={q2v}: got {got} want {want}"
                );
            }
        }
    }

    /// The subgrid walk selects the first band whose `(x, Q²)` support contains
    /// the point; a seam value lands in the lower band ("first in-range").
    #[test]
    fn subgrid_walk_selects_first_in_range_band() {
        let x = geomspace(1e-4, 1.0, 6);
        // Two adjacent Q² bands sharing the seam at Q²=100.
        let q2_lo = geomspace(1.0, 100.0, 5);
        let q2_hi = geomspace(100.0, 1e6, 5);
        let flavors = vec![1, 21];
        // Constant-but-distinct value per band, so the selected band is legible.
        let lo = subgrid_from_fn(&x, &q2_lo, &flavors, |_, _, _| 1.0);
        let hi = subgrid_from_fn(&x, &q2_hi, &flavors, |_, _, _| 2.0);
        let interp = LogBicubic::build(&[lo, hi]);

        // Interior of the lower band.
        assert!((interp.xfx_q2(1, 0.01_f64, 10.0_f64).unwrap() - 1.0).abs() < 1e-12);
        // Interior of the upper band.
        assert!((interp.xfx_q2(1, 0.01_f64, 1e4_f64).unwrap() - 2.0).abs() < 1e-12);
        // Exactly on the seam Q²=100: the lower band wins (first in-range).
        assert!((interp.xfx_q2(1, 0.01_f64, 100.0_f64).unwrap() - 1.0).abs() < 1e-12);
    }

    /// PDG 0 aliases the gluon 21, and an absent flavor evaluates to exactly 0.
    #[test]
    fn flavor_alias_and_absent_flavor() {
        let x = geomspace(1e-4, 1.0, 6);
        let q2 = geomspace(1.0, 1e6, 6);
        let flavors = vec![1, 21];
        let val = |lx: f64, lq: f64, fl: i32| 1.0 + 0.3 * lx - 0.1 * lq + 0.01 * fl as f64;
        let interp = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &flavors, val)]);

        let g21: f64 = interp.xfx_q2(21, 0.01_f64, 100.0_f64).unwrap();
        let g0: f64 = interp.xfx_q2(0, 0.01_f64, 100.0_f64).unwrap();
        assert_eq!(g21, g0, "PDG 0 must alias the gluon 21 bit-for-bit");

        // Charm (4) is not in the flavor list: exactly zero, no error.
        assert_eq!(interp.xfx_q2(4, 0.01_f64, 100.0_f64).unwrap(), 0.0);
    }

    /// The interpolator itself covers only the tabulated range: a point outside
    /// it is an error here, and it is [`super::super::extrap`] that turns such a
    /// point into a value.
    #[test]
    fn out_of_support_is_out_of_range_error() {
        let x = geomspace(1e-4, 1.0, 6);
        let q2 = geomspace(1.0, 1e6, 6);
        let interp = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &[1], |_, _, _| 1.0)]);
        assert!(
            interp.xfx_q2(1, 1e-6_f64, 100.0_f64).is_err(),
            "x below XMin"
        );
        assert!(
            interp.xfx_q2(1, 0.01_f64, 1e8_f64).is_err(),
            "Q² above QMax"
        );
    }

    /// `index_below` returns the interval whose low knot is `<= value`, clamped
    /// so `i+1` is always a valid knot.
    #[test]
    fn index_below_clamps_at_upper_edge() {
        let knots = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(index_below(0.5, &knots), 0); // below first
        assert_eq!(index_below(1.0, &knots), 0); // on first knot
        assert_eq!(index_below(2.5, &knots), 1);
        assert_eq!(index_below(3.0, &knots), 2); // on interior knot
        assert_eq!(index_below(4.0, &knots), 2); // on last knot -> clamp to len-2
        assert_eq!(index_below(9.0, &knots), 2); // above last -> clamp
    }
}
