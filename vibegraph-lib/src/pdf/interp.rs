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
//! # Kernel shape
//!
//! Tables and evaluation are `f64` throughout, and the kernel is written in the
//! shape a lane-parallel (SIMT) implementation needs, so that the fast scalar
//! form and a future vector form are the same code:
//!
//! - **Index computation is separated from arithmetic.** Band selection, the two
//!   knot searches and the two interval fractions happen once per `(x, Q²)`;
//!   everything after them is straight-line floating point over a contiguous
//!   coefficient block.
//! - **The Q²-slope stencils are branch-free.** The forward/backward/central
//!   finite-difference cases differ only by a clamped neighbour index and two
//!   per-interval constants ([`QCell`]), both resolved when the member is built.
//!   The one case that is genuinely a different formula — the bilinear fallback
//!   for a band with two Q² knots — is a property of the band, decided at build
//!   time, not of the point.
//! - **The inner loop is flavor-major** over the cell's contiguous
//!   `(flavor, coefficient)` block, which is what [`LogBicubic::xfx_all`]
//!   evaluates in one pass, and the x-direction cubic in it is a Horner chain of
//!   fused multiply-adds. The Q²-direction Hermite deliberately is not: see
//!   [`cubic_hermite`].
//!
//! A batch-of-points variant is a mechanical extension of this shape: hold the
//! per-point indices in lanes, gather the four coefficient blocks, and run the
//! same arithmetic. It is not built here because no caller can feed it — the
//! hadronic integrand is a single-point closure — and an unused batched API
//! would be untested surface. Extrapolation stays a scalar fallback either way.

use super::grid::SubGrid;
use super::{flavor_slot, FlavorRow, FLAVOR_SLOTS};

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
    fn xfx_q2(&self, pdg: i32, x: f64, q2: f64) -> Result<f64, OutOfRange>;

    /// Every tabulated flavor's `x·f(x, Q²)` at one point, written into `out` by
    /// [`super::flavor_slot`]. Slots the grid does not carry are set to zero.
    /// This is the form the luminosity sums want: one point, all flavors.
    fn xfx_all(&self, x: f64, q2: f64, out: &mut FlavorRow) -> Result<(), OutOfRange>;

    /// The grid's own edge knots, which is what a continuation past them needs.
    fn edges(&self) -> GridEdges;

    /// Whether the grid carries `pdg` at all (0 aliases the gluon 21). An absent
    /// flavor is exactly zero everywhere rather than a value to continue.
    fn has_flavor(&self, pdg: i32) -> bool;

    /// The `(slot, pdg)` pairs some band of this grid carries, ascending in
    /// slot — the set an all-flavor reading is defined on.
    fn present_flavors(&self) -> &[(u8, i32)];
}

/// The precomputed log-bicubic coefficient tables for one member, one subgrid
/// per Q² band.
#[derive(Debug, Clone)]
pub struct LogBicubic {
    subgrids: Vec<LogBicubicSubgrid>,
    edges: GridEdges,
    /// `(slot, pdg)` for every flavor some band carries, ascending in slot.
    present: Vec<(u8, i32)>,
    /// Slot → whether some band carries that flavor, for an O(1)
    /// [`Bicubic2D::has_flavor`].
    has_slot: [bool; FLAVOR_SLOTS],
    /// Whether the bands are ordered so that a Q² binary search reproduces the
    /// first-in-range walk exactly: ascending, non-overlapping in Q², and all
    /// sharing one x support. A `lhagrid1` member is always like this; anything
    /// else falls back to the walk.
    bands_searchable: bool,
}

/// Per-Q²-interval constants of the Hermite slope stencil, one entry per
/// interval `iq` of a band.
///
/// The three finite-difference cases LHAPDF distinguishes — forward at the
/// band's lower Q² edge, backward at its upper edge, central in between —
/// collapse into a single expression
///
/// ```text
/// vdl = (vh - vl + (vl - vll)·dlogq·inv_dlogq_lo) · half_lo
/// vdh = (vh - vl + (vhh - vh)·dlogq·inv_dlogq_hi) · half_hi
/// ```
///
/// once the neighbour index is clamped onto the interval's own knot (so the
/// difference `vl - vll` or `vhh - vh` is exactly zero at an edge), the
/// reciprocal spacing there is zero, and the halving factor is one. That is the
/// same arithmetic in the same order as the branching form, with the edge terms
/// multiplied by zero instead of skipped.
#[derive(Debug, Clone, Copy)]
struct QCell {
    /// `logq2s[iq+1] - logq2s[iq]`.
    dlogq: f64,
    /// `1 / (logq2s[iq] - logq2s[iq-1])`, zero at the band's lower Q² edge.
    inv_dlogq_lo: f64,
    /// `1 / (logq2s[iq+2] - logq2s[iq+1])`, zero at the band's upper Q² edge.
    inv_dlogq_hi: f64,
    /// `0.5` for a central difference, `1.0` where the one-sided difference is
    /// the whole slope.
    half_lo: f64,
    half_hi: f64,
    /// `iq - 1`, clamped to `iq` at the lower edge.
    iq_lo: usize,
    /// `iq + 2`, clamped to `iq + 1` at the upper edge.
    iq_hi: usize,
}

/// One subgrid's precomputed x-direction cubic coefficients plus the log-knot
/// coordinates needed for the Q²-direction Hermite assembly.
#[derive(Debug, Clone)]
struct LogBicubicSubgrid {
    /// Flavor-list position → [`super::flavor_slot`], the scatter an all-flavor
    /// reading writes through.
    slot_of_flavor: Vec<usize>,
    /// Slot → flavor-list position, or `usize::MAX` for a flavor this band does
    /// not carry. Replaces the linear scan over the flavor list.
    flavor_of_slot: [usize; FLAVOR_SLOTS],
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
    /// Slope stencil constants, one per Q² interval.
    qcells: Vec<QCell>,
    /// A band with two Q² knots carries no third knot to take a finite
    /// difference against, and LHAPDF drops to bilinear there. It is a property
    /// of the band, so the choice is made here rather than per point.
    degenerate: bool,
}

impl LogBicubic {
    /// Precompute the log-bicubic coefficients for every subgrid of a member.
    pub fn build(subgrids: &[SubGrid]) -> Self {
        let bands: Vec<LogBicubicSubgrid> = subgrids.iter().map(LogBicubicSubgrid::build).collect();

        let mut has_slot = [false; FLAVOR_SLOTS];
        let mut present: Vec<(u8, i32)> = Vec::new();
        for sg in subgrids {
            for &pdg in &sg.flavors {
                let slot = flavor_slot(pdg).expect("checked when the band was built");
                if !has_slot[slot] {
                    has_slot[slot] = true;
                    present.push((slot as u8, super::normalize_flavor_pdg(pdg)));
                }
            }
        }
        present.sort_unstable();

        let bands_searchable = bands.windows(2).all(|w| {
            w[0].q2_max <= w[1].q2_min && w[0].x_min == w[1].x_min && w[0].x_max == w[1].x_max
        });

        LogBicubic {
            subgrids: bands,
            edges: edges_of(subgrids),
            present,
            has_slot,
            bands_searchable,
        }
    }

    /// The band a point reads from: the first whose `(x, Q²)` support contains
    /// it. On an ordered member (the only shape `lhagrid1` produces) this is a
    /// binary search on the bands' upper Q² edges, which lands a seam value in
    /// the lower band exactly as the walk does, because the seam *is* that
    /// band's upper edge.
    #[inline]
    fn select_band(&self, x: f64, q2: f64) -> Option<&LogBicubicSubgrid> {
        if !self.bands_searchable {
            return self
                .subgrids
                .iter()
                .find(|sg| x >= sg.x_min && x <= sg.x_max && q2 >= sg.q2_min && q2 <= sg.q2_max);
        }
        let i = self.subgrids.partition_point(|sg| sg.q2_max < q2);
        let sg = self.subgrids.get(i)?;
        (x >= sg.x_min && x <= sg.x_max && q2 >= sg.q2_min).then_some(sg)
    }

    /// The union extent of every band, for an out-of-range report.
    #[cold]
    fn out_of_range(&self, x: f64, q2: f64) -> OutOfRange {
        let mut e = OutOfRange {
            x,
            q2,
            x_min: f64::INFINITY,
            x_max: f64::NEG_INFINITY,
            q2_min: f64::INFINITY,
            q2_max: f64::NEG_INFINITY,
        };
        for sg in &self.subgrids {
            e.x_min = e.x_min.min(sg.x_min);
            e.x_max = e.x_max.max(sg.x_max);
            e.q2_min = e.q2_min.min(sg.q2_min);
            e.q2_max = e.q2_max.max(sg.q2_max);
        }
        e
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
    fn xfx_q2(&self, pdg: i32, x: f64, q2: f64) -> Result<f64, OutOfRange> {
        let Some(sg) = self.select_band(x, q2) else {
            return Err(self.out_of_range(x, q2));
        };
        // Flavor absent from the band: LHAPDF returns exactly zero.
        let Some(slot) = flavor_slot(pdg) else {
            return Ok(0.0);
        };
        let ifl = sg.flavor_of_slot[slot];
        if ifl == ABSENT {
            return Ok(0.0);
        }
        Ok(sg.eval_one(ifl, x, q2))
    }

    fn xfx_all(&self, x: f64, q2: f64, out: &mut FlavorRow) -> Result<(), OutOfRange> {
        let Some(sg) = self.select_band(x, q2) else {
            return Err(self.out_of_range(x, q2));
        };
        *out = [0.0; FLAVOR_SLOTS];
        sg.eval_all(x, q2, out);
        Ok(())
    }

    fn edges(&self) -> GridEdges {
        self.edges
    }

    fn has_flavor(&self, pdg: i32) -> bool {
        flavor_slot(pdg).is_some_and(|slot| self.has_slot[slot])
    }

    fn present_flavors(&self) -> &[(u8, i32)] {
        &self.present
    }
}

/// Marks a slot no band position corresponds to, in `flavor_of_slot`.
const ABSENT: usize = usize::MAX;

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

/// Where one point sits in one band: the interval indices and the two interval
/// fractions, computed once and shared by every flavor.
struct Cell<'a> {
    ix: usize,
    iq: usize,
    tlogx: f64,
    tlogq: f64,
    q: &'a QCell,
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

        let mut flavor_of_slot = [ABSENT; FLAVOR_SLOTS];
        let mut slot_of_flavor = vec![0usize; nf];
        for (ifl, &pdg) in sg.flavors.iter().enumerate() {
            let slot = flavor_slot(pdg).unwrap_or_else(|| {
                panic!(
                    "subgrid flavor list carries PDG code {pdg}, which is not a parton \
                     density this reader has a slot for (quarks -6..=6, gluon 21, photon 22)"
                )
            });
            slot_of_flavor[ifl] = slot;
            // First position wins, matching the scan over the flavor list a
            // duplicated code would have resolved to.
            if flavor_of_slot[slot] == ABSENT {
                flavor_of_slot[slot] = ifl;
            }
        }

        let qcells = (0..nq.saturating_sub(1))
            .map(|iq| {
                let lower = iq == 0;
                let upper = iq + 1 == nq - 1;
                QCell {
                    dlogq: logq2s[iq + 1] - logq2s[iq],
                    inv_dlogq_lo: if lower {
                        0.0
                    } else {
                        1.0 / (logq2s[iq] - logq2s[iq - 1])
                    },
                    inv_dlogq_hi: if upper {
                        0.0
                    } else {
                        1.0 / (logq2s[iq + 2] - logq2s[iq + 1])
                    },
                    half_lo: if lower { 1.0 } else { 0.5 },
                    half_hi: if upper { 1.0 } else { 0.5 },
                    iq_lo: if lower { iq } else { iq - 1 },
                    iq_hi: if upper { iq + 1 } else { iq + 2 },
                }
            })
            .collect();

        LogBicubicSubgrid {
            slot_of_flavor,
            flavor_of_slot,
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
            qcells,
            degenerate: nq == 2,
        }
    }

    /// Locate `(x, Q²)` in this band. All the index work and both `ln` calls
    /// happen here, once per point however many flavors are read.
    #[inline]
    fn locate(&self, x: f64, q2: f64) -> (Cell<'_>, f64, f64) {
        let logx = x.ln();
        let logq2 = q2.ln();
        // ln is monotone, so locating the interval on the log-knots is identical
        // to locating it on the raw knots; use the log-knots directly.
        let ix = index_below(logx, &self.logxs);
        let iq = index_below(logq2, &self.logq2s);
        let q = &self.qcells[iq];
        let cell = Cell {
            ix,
            iq,
            tlogx: (logx - self.logxs[ix]) / (self.logxs[ix + 1] - self.logxs[ix]),
            tlogq: (logq2 - self.logq2s[iq]) / q.dlogq,
            q,
        };
        (cell, logx, logq2)
    }

    /// The four x-cubic coefficients `[a, b, c, d]` for interval `ix`, Q²-knot
    /// `iq`, flavor `ifl`.
    #[inline]
    fn coeff(&self, ix: usize, iq: usize, ifl: usize) -> &[f64] {
        let base = ((ix * self.nq + iq) * self.nf + ifl) * 4;
        &self.coeffs[base..base + 4]
    }

    /// One flavor's `x·f` in an already-located cell.
    #[inline]
    fn eval_at(&self, cell: &Cell<'_>, ifl: usize) -> f64 {
        let q = cell.q;
        let vl = cubic_x(cell.tlogx, self.coeff(cell.ix, cell.iq, ifl));
        let vh = cubic_x(cell.tlogx, self.coeff(cell.ix, cell.iq + 1, ifl));
        let vll = cubic_x(cell.tlogx, self.coeff(cell.ix, q.iq_lo, ifl));
        let vhh = cubic_x(cell.tlogx, self.coeff(cell.ix, q.iq_hi, ifl));
        let d = vh - vl;
        let vdl = (d + (vl - vll) * q.dlogq * q.inv_dlogq_lo) * q.half_lo;
        let vdh = (d + (vhh - vh) * q.dlogq * q.inv_dlogq_hi) * q.half_hi;
        cubic_hermite(cell.tlogq, vl, vdl, vh, vdh)
    }

    /// Evaluate `x·f` at `(x, Q²)` for one flavor position.
    fn eval_one(&self, ifl: usize, x: f64, q2: f64) -> f64 {
        let (cell, logx, logq2) = self.locate(x, q2);
        if self.degenerate {
            return self.bilinear(&cell, logx, logq2, ifl);
        }
        self.eval_at(&cell, ifl)
    }

    /// Evaluate `x·f` at `(x, Q²)` for every flavor this band carries, writing
    /// each into its slot of `out`.
    fn eval_all(&self, x: f64, q2: f64, out: &mut FlavorRow) {
        let (cell, logx, logq2) = self.locate(x, q2);
        if self.degenerate {
            for (ifl, &slot) in self.slot_of_flavor.iter().enumerate() {
                out[slot] = self.bilinear(&cell, logx, logq2, ifl);
            }
        } else {
            for (ifl, &slot) in self.slot_of_flavor.iter().enumerate() {
                out[slot] = self.eval_at(&cell, ifl);
            }
        }
    }

    /// LHAPDF's degenerate case: a band whose only two Q² knots are both grid
    /// edges has no finite difference to take, and falls back to bilinear on the
    /// raw values.
    fn bilinear(&self, cell: &Cell<'_>, logx: f64, logq2: f64, ifl: usize) -> f64 {
        let (ix, iq) = (cell.ix, cell.iq);
        let f_ql = interp_linear(
            logx,
            self.logxs[ix],
            self.logxs[ix + 1],
            self.xf_raw(ix, iq, ifl),
            self.xf_raw(ix + 1, iq, ifl),
        );
        let f_qh = interp_linear(
            logx,
            self.logxs[ix],
            self.logxs[ix + 1],
            self.xf_raw(ix, iq + 1, ifl),
            self.xf_raw(ix + 1, iq + 1, ifl),
        );
        interp_linear(logq2, self.logq2s[iq], self.logq2s[iq + 1], f_ql, f_qh)
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

/// Cubic from stored coefficients `[a, b, c, d]`: `a·t³ + b·t² + c·t + d`,
/// in Horner form over fused multiply-adds. The stored monomial coefficients
/// *are* the Horner coefficients, so this is an evaluation-order change and not
/// a different polynomial.
#[inline]
fn cubic_x(t: f64, coeffs: &[f64]) -> f64 {
    coeffs[0]
        .mul_add(t, coeffs[1])
        .mul_add(t, coeffs[2])
        .mul_add(t, coeffs[3])
}

/// Cubic Hermite on `[0,1]` from edge values `vl, vh` and edge slopes
/// `vdl, vdh` (already scaled by the interval width). Operation order mirrors
/// LHAPDF's `_interpolateCubic(t, vl, vdl, vh, vdh)`, and that is a constraint
/// rather than an accident, for two independent reasons.
///
/// **The interval's ends are exact.** At `t = 1` the four basis weights are
/// `2−3+1`, `1−2+1`, `−2+3` and `1−1`, each exact in binary, so the sum is `vh`
/// to the bit; likewise `vl` at `t = 0`. Collecting the basis into monomial
/// coefficients and running Horner reaches `vh` only through a cancellation
/// between them, which costs eight orders on node reproduction at a band's top
/// Q² knot (worst `|Δ|` over the LHAPDF on-knot probes: `2.7e-20` this way,
/// `2.7e-12` under Horner).
///
/// **The `x → 1` residue is not reconstructible.** Where `x·f` has died the four
/// inputs cancel by some thirty orders, leaving a rounding residue around
/// `1e-35` rather than a density; reproducing *LHAPDF's* residue is the only
/// sense in which the two agree at such a point, and Horner moves it by parts in
/// `1e5`. That is nothing absolutely, and the continuation comparison past the
/// grid boundaries screens it as such — but the two gates that reconstruct the
/// upper continuation from its endpoints compare relatively, and see it.
///
/// The x-direction cubic has no such cancellation and is evaluated by Horner.
#[inline]
fn cubic_hermite(t: f64, vl: f64, vdl: f64, vh: f64, vdh: f64) -> f64 {
    let t2 = t * t;
    let t3 = t * t2;
    let p0 = (2.0 * t3 - 3.0 * t2 + 1.0) * vl;
    let m0 = (t3 - 2.0 * t2 + t) * vdl;
    let p1 = (-2.0 * t3 + 3.0 * t2) * vh;
    let m1 = (t3 - t2) * vdh;
    p0 + m0 + p1 + m1
}

/// Linear interpolation `yl + (x-xl)/(xh-xl)·(yh-yl)`.
#[inline]
fn interp_linear(x: f64, xl: f64, xh: f64, yl: f64, yh: f64) -> f64 {
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
                let got = interp.xfx_q2(fl, xv, q2v).unwrap();
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
        assert!((interp.xfx_q2(1, 0.01, 10.0).unwrap() - 1.0).abs() < 1e-12);
        // Interior of the upper band.
        assert!((interp.xfx_q2(1, 0.01, 1e4).unwrap() - 2.0).abs() < 1e-12);
        // Exactly on the seam Q²=100: the lower band wins (first in-range).
        assert!((interp.xfx_q2(1, 0.01, 100.0).unwrap() - 1.0).abs() < 1e-12);
    }

    /// The Q² binary search over band edges and the first-in-range walk are the
    /// same selection, on both sides of every seam and at it — the property that
    /// lets the search replace the walk. Checked against the walk directly, so a
    /// future band layout the search cannot reproduce is caught here rather than
    /// in a cross section.
    #[test]
    fn the_band_search_agrees_with_the_walk_everywhere() {
        let x = geomspace(1e-4, 1.0, 6);
        let bands: Vec<SubGrid> = [(1.0, 100.0), (100.0, 1e4), (1e4, 1e6)]
            .iter()
            .enumerate()
            .map(|(i, &(lo, hi))| {
                let q2 = geomspace(lo, hi, 5);
                subgrid_from_fn(&x, &q2, &[1, 21], move |_, _, _| i as f64)
            })
            .collect();
        let interp = LogBicubic::build(&bands);
        assert!(interp.bands_searchable, "the fixture is an ordered member");

        let walk = |q2: f64| {
            interp.subgrids.iter().position(|sg| {
                0.01 >= sg.x_min && 0.01 <= sg.x_max && q2 >= sg.q2_min && q2 <= sg.q2_max
            })
        };
        for &q2 in &[
            0.5,
            1.0,
            1.0000001,
            50.0,
            99.999999,
            100.0,
            100.00001,
            5e3,
            1e4,
            1e4 + 1.0,
            5e5,
            1e6,
            1e6 + 1.0,
            1e9,
        ] {
            let searched = interp.select_band(0.01, q2).map(|sg| {
                interp
                    .subgrids
                    .iter()
                    .position(|s| std::ptr::eq(s, sg))
                    .unwrap()
            });
            assert_eq!(searched, walk(q2), "band selection differs at Q²={q2}");
        }
    }

    /// PDG 0 aliases the gluon 21, and an absent flavor evaluates to exactly 0.
    #[test]
    fn flavor_alias_and_absent_flavor() {
        let x = geomspace(1e-4, 1.0, 6);
        let q2 = geomspace(1.0, 1e6, 6);
        let flavors = vec![1, 21];
        let val = |lx: f64, lq: f64, fl: i32| 1.0 + 0.3 * lx - 0.1 * lq + 0.01 * fl as f64;
        let interp = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &flavors, val)]);

        let g21 = interp.xfx_q2(21, 0.01, 100.0).unwrap();
        let g0 = interp.xfx_q2(0, 0.01, 100.0).unwrap();
        assert_eq!(g21, g0, "PDG 0 must alias the gluon 21 bit-for-bit");

        // Charm (4) is not in the flavor list: exactly zero, no error.
        assert_eq!(interp.xfx_q2(4, 0.01, 100.0).unwrap(), 0.0);
    }

    /// The all-flavor reading is the single-flavor one, flavor by flavor, and
    /// bit-for-bit: the luminosity sums go through `xfx_all` while every oracle
    /// goes through `xfx_q2`, so a difference between them would be a gate the
    /// production path is not under.
    #[test]
    fn all_flavor_and_single_flavor_readings_are_identical() {
        let x = geomspace(1e-5, 1.0, 12);
        let flavors = vec![-2, -1, 1, 2, 21, 22];
        let val = |lx: f64, lq: f64, fl: i32| (0.3 * lx - 0.11 * lq + 0.07 * fl as f64).exp();
        // One ordinary band and one degenerate (two-Q²-knot) band, so the
        // bilinear fallback is covered as well as the Hermite.
        for q2 in [geomspace(1.0, 1e6, 9), geomspace(1.0, 1e6, 2)] {
            let interp = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &flavors, val)]);
            let mut row = [0.0; FLAVOR_SLOTS];
            for &(xv, q2v) in &[(3.3e-4, 55.0), (0.11, 9.9e3), (7e-5, 1.0), (0.99, 9e5)] {
                interp.xfx_all(xv, q2v, &mut row).unwrap();
                for &fl in &flavors {
                    let one = interp.xfx_q2(fl, xv, q2v).unwrap();
                    let all = row[flavor_slot(fl).unwrap()];
                    assert_eq!(
                        one.to_bits(),
                        all.to_bits(),
                        "fl={fl} x={xv} Q²={q2v}: xfx_q2 {one:e} vs xfx_all {all:e}"
                    );
                }
                // A flavor no band carries stays exactly zero in the row.
                assert_eq!(row[flavor_slot(5).unwrap()], 0.0);
            }
        }
    }

    /// The interpolator itself covers only the tabulated range: a point outside
    /// it is an error here, and it is [`super::super::extrap`] that turns such a
    /// point into a value.
    #[test]
    fn out_of_support_is_out_of_range_error() {
        let x = geomspace(1e-4, 1.0, 6);
        let q2 = geomspace(1.0, 1e6, 6);
        let interp = LogBicubic::build(&[subgrid_from_fn(&x, &q2, &[1], |_, _, _| 1.0)]);
        assert!(interp.xfx_q2(1, 1e-6, 100.0).is_err(), "x below XMin");
        assert!(interp.xfx_q2(1, 0.01, 1e8).is_err(), "Q² above QMax");
        let mut row = [0.0; FLAVOR_SLOTS];
        assert!(interp.xfx_all(1e-6, 100.0, &mut row).is_err());
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
