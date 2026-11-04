//! A bit-level oracle for the member interpolator: LHAPDF's log-bicubic over
//! *precomputed* x-cubic coefficient tables, against the production
//! [`LogBicubic`], which stores only the knots and builds each coefficient at
//! the point.
//!
//! The oracle is the precomputed-table form of the interpolator: `[a, b, c, d]`
//! per `(x interval, Q² knot, flavor)`, built once per member from LHAPDF's
//! `_ddx` slopes in `GridPDF::_computePolynomialCoefficients` order, then the
//! same Horner chain, Q² Hermite and bilinear fallback. The production form
//! builds each coefficient from the knots at the point. Both are meant to
//! perform the same IEEE operations on the same operands, so the comparison is
//! on `to_bits`, every value, with no tolerance.
//!
//! Each member is read at every knot of every band; at 40 x values on every
//! band's lower and upper Q² edge (every seam) and one ulp either side; at
//! every x knot and one ulp either side; at 3000 log-uniform points; and past
//! the grid (below `x_min`, below `Q²_min`, above `Q²_max`, the corners, and one
//! ulp past each edge). Every point is read as the all-flavor row and as 16
//! single PDG codes, including the gluon alias 0, the photon and an absent code.
//! Past the grid both sides run the production [`Continuation`] and positivity
//! clamp over their own interpolator, so those values compare the edge readings
//! the continuation is built from, not the continuation itself.
//!
//! The members are every member on disk in the LHAPDF tree named by
//! `SYSTEMATICS_CMS_PDFS`, all of NNPDF23 where `validation/pdf` holds it, and
//! three in-memory members carrying the two-knot (bilinear) bands no real set in
//! the CMS list has.
//!
//! What this cannot see: an error the table form shares, such as a wrong
//! `_ddx` expression or a Hermite order that departs from LHAPDF in both (the
//! LHAPDF probe values of `validate_pdf_grid` pin those), and anything in the
//! continuation or the clamp, which both sides run.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

use super::extrap::{self, Continuation, Extrapolate2D, PdfPointError};
use super::grid::SubGrid;
use super::interp::{Bicubic2D, GridEdges, OutOfRange};
use super::{flavor_slot, force_positive_clamp, FlavorRow, PdfMember, PdfSet, FLAVOR_SLOTS};
use crate::helas::repr::Real;

/// Marks a slot no band position corresponds to, in `flavor_of_slot`.
const ABSENT: usize = usize::MAX;

/// One member's interpolator over precomputed x-cubic coefficient tables.
struct TableBicubic {
    bands: Vec<TableBand>,
    edges: GridEdges,
    present: Vec<(u8, i32)>,
    has_slot: [bool; FLAVOR_SLOTS],
    bands_searchable: bool,
}

#[derive(Clone, Copy)]
struct QCell {
    dlogq: f64,
    inv_dlogq_lo: f64,
    inv_dlogq_hi: f64,
    half_lo: f64,
    half_hi: f64,
    iq_lo: usize,
    iq_hi: usize,
}

struct TableBand {
    slot_of_flavor: Vec<usize>,
    flavor_of_slot: [usize; FLAVOR_SLOTS],
    logxs: Vec<f64>,
    logq2s: Vec<f64>,
    x_min: f64,
    x_max: f64,
    q2_min: f64,
    q2_max: f64,
    nq: usize,
    nf: usize,
    /// Shape `(nx-1, nq, nf, 4)` row-major, `[a, b, c, d]` of `a·t³ + b·t² + c·t + d`.
    coeffs: Vec<f64>,
    /// Shape `(nx, nq, nf)` row-major, read by the bilinear fallback only.
    xf: Vec<f64>,
    qcells: Vec<QCell>,
    degenerate: bool,
}

struct Cell<'a> {
    ix: usize,
    iq: usize,
    tlogx: f64,
    tlogq: f64,
    q: &'a QCell,
}

impl TableBicubic {
    fn build(subgrids: &[SubGrid]) -> Self {
        let bands: Vec<TableBand> = subgrids.iter().map(TableBand::build).collect();
        let mut has_slot = [false; FLAVOR_SLOTS];
        let mut present: Vec<(u8, i32)> = Vec::new();
        for sg in subgrids {
            for &pdg in &sg.flavors {
                let slot = flavor_slot(pdg).expect("a parton flavor");
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
        TableBicubic {
            edges: edges_of(subgrids),
            bands,
            present,
            has_slot,
            bands_searchable,
        }
    }

    fn select_band(&self, x: f64, q2: f64) -> Option<&TableBand> {
        if !self.bands_searchable {
            return self
                .bands
                .iter()
                .find(|b| x >= b.x_min && x <= b.x_max && q2 >= b.q2_min && q2 <= b.q2_max);
        }
        let i = self.bands.partition_point(|b| b.q2_max < q2);
        let b = self.bands.get(i)?;
        (x >= b.x_min && x <= b.x_max && q2 >= b.q2_min).then_some(b)
    }

    fn out_of_range(&self, x: f64, q2: f64) -> OutOfRange {
        let mut e = OutOfRange {
            x,
            q2,
            x_min: f64::INFINITY,
            x_max: f64::NEG_INFINITY,
            q2_min: f64::INFINITY,
            q2_max: f64::NEG_INFINITY,
        };
        for b in &self.bands {
            e.x_min = e.x_min.min(b.x_min);
            e.x_max = e.x_max.max(b.x_max);
            e.q2_min = e.q2_min.min(b.q2_min);
            e.q2_max = e.q2_max.max(b.q2_max);
        }
        e
    }
}

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

impl Bicubic2D for TableBicubic {
    fn xfx_q2(&self, pdg: i32, x: f64, q2: f64) -> Result<f64, OutOfRange> {
        let Some(b) = self.select_band(x, q2) else {
            return Err(self.out_of_range(x, q2));
        };
        let Some(slot) = flavor_slot(pdg) else {
            return Ok(0.0);
        };
        let ifl = b.flavor_of_slot[slot];
        if ifl == ABSENT {
            return Ok(0.0);
        }
        Ok(b.eval_one(ifl, x, q2))
    }

    fn xfx_all(&self, x: f64, q2: f64, out: &mut FlavorRow) -> Result<(), OutOfRange> {
        let Some(b) = self.select_band(x, q2) else {
            return Err(self.out_of_range(x, q2));
        };
        *out = [0.0; FLAVOR_SLOTS];
        b.eval_all(x, q2, out);
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

fn index_below(value: f64, knots: &[f64]) -> usize {
    let n = knots.len();
    let mut i = knots.partition_point(|&k| k <= value);
    if i == n {
        i -= 1;
    }
    i.saturating_sub(1)
}

/// LHAPDF's `_ddx` in log-x at knot `ix`: one-sided at the first and last knot,
/// the mean of the two one-sided differences in between.
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

impl TableBand {
    fn build(sg: &SubGrid) -> Self {
        let nx = sg.nx();
        let nq = sg.nq();
        let nf = sg.nf();
        let logxs: Vec<f64> = sg.x.iter().map(|&v| v.ln()).collect();
        let logq2s: Vec<f64> = sg.q2.iter().map(|&v| v.ln()).collect();

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
            let slot = flavor_slot(pdg).expect("a parton flavor");
            slot_of_flavor[ifl] = slot;
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

        TableBand {
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

    fn locate(&self, x: f64, q2: f64) -> (Cell<'_>, f64, f64) {
        let logx = x.ln();
        let logq2 = q2.ln();
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

    fn coeff(&self, ix: usize, iq: usize, ifl: usize) -> &[f64] {
        let base = ((ix * self.nq + iq) * self.nf + ifl) * 4;
        &self.coeffs[base..base + 4]
    }

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

    fn eval_one(&self, ifl: usize, x: f64, q2: f64) -> f64 {
        let (cell, logx, logq2) = self.locate(x, q2);
        if self.degenerate {
            return self.bilinear(&cell, logx, logq2, ifl);
        }
        self.eval_at(&cell, ifl)
    }

    fn eval_all(&self, x: f64, q2: f64, out: &mut FlavorRow) {
        let (cell, logx, logq2) = self.locate(x, q2);
        for (ifl, &slot) in self.slot_of_flavor.iter().enumerate() {
            out[slot] = if self.degenerate {
                self.bilinear(&cell, logx, logq2, ifl)
            } else {
                self.eval_at(&cell, ifl)
            };
        }
    }

    fn bilinear(&self, cell: &Cell<'_>, logx: f64, logq2: f64, ifl: usize) -> f64 {
        let (ix, iq) = (cell.ix, cell.iq);
        let raw = |ix: usize, iq: usize| self.xf[(ix * self.nq + iq) * self.nf + ifl];
        let f_ql = interp_linear(
            logx,
            self.logxs[ix],
            self.logxs[ix + 1],
            raw(ix, iq),
            raw(ix + 1, iq),
        );
        let f_qh = interp_linear(
            logx,
            self.logxs[ix],
            self.logxs[ix + 1],
            raw(ix, iq + 1),
            raw(ix + 1, iq + 1),
        );
        interp_linear(logq2, self.logq2s[iq], self.logq2s[iq + 1], f_ql, f_qh)
    }
}

fn cubic_x(t: f64, coeffs: &[f64]) -> f64 {
    coeffs[0]
        .mul_add_fast(t, coeffs[1])
        .mul_add_fast(t, coeffs[2])
        .mul_add_fast(t, coeffs[3])
}

fn cubic_hermite(t: f64, vl: f64, vdl: f64, vh: f64, vdh: f64) -> f64 {
    let t2 = t * t;
    let t3 = t * t2;
    let p0 = (2.0 * t3 - 3.0 * t2 + 1.0) * vl;
    let m0 = (t3 - 2.0 * t2 + t) * vdl;
    let p1 = (-2.0 * t3 + 3.0 * t2) * vh;
    let m1 = (t3 - t2) * vdh;
    p0 + m0 + p1 + m1
}

fn interp_linear(x: f64, xl: f64, xh: f64, yl: f64, yh: f64) -> f64 {
    yl + (x - xl) / (xh - xl) * (yh - yl)
}

/// [`PdfMember::try_xfx_q2`]'s composition over the table interpolator: the
/// guards, the absent-flavor zero, interpolation or continuation, the clamp.
fn table_xfx_q2(
    t: &TableBicubic,
    force_positive: i32,
    pdg: i32,
    x: f64,
    q2: f64,
) -> Result<f64, PdfPointError> {
    if !(x > 0.0) || !x.is_finite() || !(q2 >= 0.0) || !q2.is_finite() {
        return Err(PdfPointError::Unphysical { x, q2 });
    }
    if !t.has_flavor(pdg) {
        return Ok(0.0);
    }
    let value = if extrap::in_grid_range(&t.edges(), x, q2) {
        t.xfx_q2(pdg, x, q2)?
    } else {
        Continuation.xfx_q2(t, pdg, x, q2)?
    };
    Ok(force_positive_clamp(force_positive, value))
}

/// [`PdfMember::try_xfx_all`]'s composition over the table interpolator.
fn table_xfx_all(
    t: &TableBicubic,
    force_positive: i32,
    x: f64,
    q2: f64,
    out: &mut FlavorRow,
) -> Result<(), PdfPointError> {
    if !(x > 0.0) || !x.is_finite() || !(q2 >= 0.0) || !q2.is_finite() {
        return Err(PdfPointError::Unphysical { x, q2 });
    }
    if extrap::in_grid_range(&t.edges(), x, q2) {
        t.xfx_all(x, q2, out)?;
    } else {
        *out = [0.0; FLAVOR_SLOTS];
        for &(slot, pdg) in t.present_flavors() {
            out[slot as usize] = Continuation.xfx_q2(t, pdg, x, q2)?;
        }
    }
    if force_positive != 0 {
        for &(slot, _) in t.present_flavors() {
            out[slot as usize] = force_positive_clamp(force_positive, out[slot as usize]);
        }
    }
    Ok(())
}

/// Every PDG code a single reading is taken for: the quarks, the gluon and its
/// alias 0, the photon, and 7, which no set carries.
const PDGS: [i32; 16] = [-6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 21, 22, 7];

/// Readings per point: the all-flavor row and each code in [`PDGS`].
const VALUES_PER_POINT: usize = FLAVOR_SLOTS + PDGS.len();

/// The members of CMS's `runcmsgrid.sh` systematics request (43 sets). A tree
/// holding fewer is a partial fetch, and the comparison over it would pass
/// without having seen the sets it is named for.
const CMS_MEMBERS: usize = 880;

const RANDOM_POINTS: usize = 3000;
const SEAM_XS: usize = 40;
const CONTINUATION_DRAWS: usize = 200;

/// Point categories, in the order [`Tally::points`] counts them.
const CATEGORIES: [&str; 4] = ["knot", "seam/edge±ulp", "random", "continuation"];

fn up(v: f64) -> f64 {
    f64::from_bits(v.to_bits() + 1)
}

fn down(v: f64) -> f64 {
    f64::from_bits(v.to_bits() - 1)
}

fn log_uniform(rng: &mut ChaCha8Rng, lo: f64, hi: f64) -> f64 {
    (lo.ln() + rng.random::<f64>() * (hi.ln() - lo.ln())).exp()
}

/// One member's points, each tagged with its index in [`CATEGORIES`].
fn points(sgs: &[SubGrid], seed: u64) -> Vec<(usize, f64, f64)> {
    let x = &sgs[0].x;
    let (xmin, xmax) = (x[0], x[x.len() - 1]);
    let q2min = sgs[0].q2[0];
    let last = &sgs[sgs.len() - 1].q2;
    let q2max = last[last.len() - 1];
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut p = Vec::new();
    for sg in sgs {
        for &xv in &sg.x {
            for &q2 in &sg.q2 {
                p.push((0, xv, q2));
            }
        }
    }
    let xs: Vec<f64> = (0..SEAM_XS)
        .map(|_| log_uniform(&mut rng, xmin, xmax))
        .collect();
    for sg in sgs {
        let (lo, hi) = (sg.q2[0], sg.q2[sg.q2.len() - 1]);
        for edge in [lo, hi] {
            for &xv in &xs {
                for q2 in [down(edge), edge, up(edge)] {
                    p.push((1, xv, q2));
                }
            }
        }
        let q2 = (lo.ln() + 0.37 * (hi.ln() - lo.ln())).exp();
        for &xv in &sg.x {
            for xx in [down(xv), xv, up(xv)] {
                if xx <= xmax {
                    p.push((1, xx, q2));
                }
            }
        }
    }
    for _ in 0..RANDOM_POINTS {
        let (xv, q2) = (
            log_uniform(&mut rng, xmin, xmax),
            log_uniform(&mut rng, q2min, q2max),
        );
        p.push((2, xv, q2));
    }
    for _ in 0..CONTINUATION_DRAWS {
        let xv = log_uniform(&mut rng, xmin, xmax);
        let q2 = log_uniform(&mut rng, q2min, q2max);
        let xlow = xmin * (-rng.random::<f64>() * 5.0).exp();
        let qlow = q2min * (-rng.random::<f64>() * 3.0).exp();
        let qhigh = q2max * (rng.random::<f64>() * 5.0).exp();
        p.extend([
            (3, xlow, q2),
            (3, xv, qlow),
            (3, xv, qhigh),
            (3, xlow, qlow),
            (3, xlow, qhigh),
        ]);
    }
    p.extend([
        (3, down(xmin), q2min),
        (3, xmin, down(q2min)),
        (3, xmax, up(q2max)),
    ]);
    p
}

/// What one or more members' comparisons found.
#[derive(Default, Clone)]
struct Tally {
    members: u64,
    points: [u64; 4],
    values: u64,
    nonzero: u64,
    differ: u64,
    examples: Vec<String>,
}

impl Tally {
    fn add(&mut self, o: &Tally) {
        self.members += o.members;
        for (a, b) in self.points.iter_mut().zip(o.points) {
            *a += b;
        }
        self.values += o.values;
        self.nonzero += o.nonzero;
        self.differ += o.differ;
        for e in &o.examples {
            if self.examples.len() < 10 {
                self.examples.push(e.clone());
            }
        }
    }
}

fn reading_bits(r: Result<f64, PdfPointError>) -> Result<u64, String> {
    r.map(f64::to_bits).map_err(|e| e.to_string())
}

/// Compare every reading of `member` with the table interpolator built from the
/// same subgrids.
fn compare(name: &str, id: u32, member: &PdfMember) -> Tally {
    let table = TableBicubic::build(member.subgrids());
    let fp = member.force_positive();
    let mut t = Tally {
        members: 1,
        ..Tally::default()
    };
    let diff = |t: &mut Tally, what: String| {
        t.differ += 1;
        if t.examples.len() < 10 {
            t.examples.push(format!("{name} member {id}: {what}"));
        }
    };
    for (cat, x, q2) in points(member.subgrids(), 0xB175 + u64::from(id)) {
        t.points[cat] += 1;
        let (mut ours, mut theirs): (FlavorRow, FlavorRow) =
            ([0.0; FLAVOR_SLOTS], [0.0; FLAVOR_SLOTS]);
        let a = member
            .try_xfx_all(x, q2, &mut ours)
            .map_err(|e| e.to_string());
        let b = table_xfx_all(&table, fp, x, q2, &mut theirs).map_err(|e| e.to_string());
        t.values += FLAVOR_SLOTS as u64;
        if a != b {
            diff(
                &mut t,
                format!("row at x={x:e} Q²={q2:e}: {a:?} against {b:?}"),
            );
        } else if a.is_ok() {
            for slot in 0..FLAVOR_SLOTS {
                if ours[slot] != 0.0 {
                    t.nonzero += 1;
                }
                if ours[slot].to_bits() != theirs[slot].to_bits() {
                    diff(
                        &mut t,
                        format!(
                            "row slot {slot} at x={x:e} Q²={q2:e}: {:e} against {:e}",
                            ours[slot], theirs[slot]
                        ),
                    );
                }
            }
        }
        for pdg in PDGS {
            let a = reading_bits(member.try_xfx_q2(pdg, x, q2));
            let b = reading_bits(table_xfx_q2(&table, fp, pdg, x, q2));
            t.values += 1;
            if matches!(a, Ok(v) if f64::from_bits(v) != 0.0) {
                t.nonzero += 1;
            }
            if a != b {
                let show = |r: &Result<u64, String>| r.clone().map(f64::from_bits);
                diff(
                    &mut t,
                    format!(
                        "pdg {pdg} at x={x:e} Q²={q2:e}: {:?} against {:?}",
                        show(&a),
                        show(&b)
                    ),
                );
            }
        }
    }
    t
}

/// Every `(set directory, set name, member)` on disk under an LHAPDF tree: each
/// directory holding `<name>.info`, and each `<name>_NNNN.dat` in it.
fn members_on_disk(tree: &Path) -> Vec<(PathBuf, String, u32)> {
    let mut out = Vec::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(tree)
        .unwrap_or_else(|e| panic!("{}: {e}", tree.display()))
        .map(|e| e.expect("a directory entry").path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        if !dir.join(format!("{name}.info")).is_file() {
            continue;
        }
        let prefix = format!("{name}_");
        let mut ids: Vec<u32> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .filter_map(|e| {
                let entry = e.unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
                let file = entry.file_name().to_string_lossy().into_owned();
                let digits = file.strip_prefix(&prefix)?.strip_suffix(".dat")?;
                if digits.len() != 4 {
                    return None;
                }
                digits.parse().ok()
            })
            .collect();
        ids.sort_unstable();
        out.extend(ids.into_iter().map(|id| (dir.clone(), name.clone(), id)));
    }
    out
}

fn geomspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    let (la, lb) = (a.ln(), b.ln());
    (0..n)
        .map(|i| (la + (lb - la) * i as f64 / (n - 1) as f64).exp())
        .collect()
}

/// In-memory members with the band shapes no real set in the CMS list has: a
/// two-knot (bilinear) band between Hermite bands, a lone two-knot band, and a
/// band of three Q² knots (both one-sided Q² stencils).
fn synthetic_members() -> Vec<(String, PdfMember)> {
    let x = geomspace(1e-7, 1.0, 37);
    let fl = vec![-5, -4, -3, -2, -1, 21, 1, 2, 3, 4, 5, 22];
    let band = |q2: Vec<f64>, k: f64| {
        let mut xf = Vec::new();
        for &xv in &x {
            for &q in &q2 {
                for &f in &fl {
                    let f = f64::from(f);
                    xf.push(
                        (1.0 - xv).powf(3.0 + 0.1 * f)
                            * xv.powf(-0.2 * k)
                            * (1.0 + 0.05 * q.ln()).sqrt()
                            * (2.0 + f / 7.0),
                    );
                }
            }
        }
        SubGrid {
            x: x.clone(),
            q2,
            flavors: fl.clone(),
            xf,
        }
    };
    vec![
        (
            "synthetic_mixed".into(),
            PdfMember::from_subgrids(vec![
                band(geomspace(1.0, 2.0, 6), 1.0),
                band(vec![2.0, 30.0], 1.1),
                band(geomspace(30.0, 1e8, 15), 0.9),
            ]),
        ),
        (
            "synthetic_two_knot".into(),
            PdfMember::from_subgrids(vec![band(vec![1.0, 1e6], 1.0)]),
        ),
        (
            "synthetic_three_knot".into(),
            PdfMember::from_subgrids(vec![band(vec![1.0, 1e3, 1e6], 0.7)]).with_force_positive(1),
        ),
    ]
}

/// Every member of the LHAPDF tree on `SYSTEMATICS_CMS_PDFS` (CMS's 880 members
/// of 43 sets), all of NNPDF23 where `validation/pdf` holds it, and the
/// in-memory bilinear members: every reading bit-identical to the table
/// interpolator's, over a counted, nonzero number of values.
#[test]
#[ignore = "needs an LHAPDF tree on SYSTEMATICS_CMS_PDFS"]
fn every_member_reads_as_its_coefficient_tables_did() {
    let tree = PathBuf::from(std::env::var("SYSTEMATICS_CMS_PDFS").expect("SYSTEMATICS_CMS_PDFS"));
    let mut list = members_on_disk(&tree);
    let from_tree = list.len();
    assert!(
        from_tree >= CMS_MEMBERS,
        "{from_tree} members on disk under {}, fewer than CMS's {CMS_MEMBERS}",
        tree.display()
    );
    let nnpdf23 = Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf");
    if nnpdf23.join("NNPDF23_lo_as_0130_qed").is_dir() {
        list.extend(
            members_on_disk(&nnpdf23)
                .into_iter()
                .filter(|(_, name, _)| name == "NNPDF23_lo_as_0130_qed"),
        );
    }
    println!(
        "{from_tree} members under {}, {} from validation/pdf",
        tree.display(),
        list.len() - from_tree
    );

    let mut per_set: BTreeMap<String, Tally> = BTreeMap::new();
    let synthetic = synthetic_members();
    for (name, member) in &synthetic {
        per_set
            .entry(name.clone())
            .or_default()
            .add(&compare(name, 0, member));
    }
    let loaded: Vec<(String, Tally)> = list
        .par_iter()
        .map(|(dir, name, id)| {
            let set = PdfSet::load(dir, name).unwrap_or_else(|e| panic!("{name}: {e}"));
            let member = set
                .member(*id)
                .unwrap_or_else(|e| panic!("{name} member {id}: {e}"));
            (name.clone(), compare(name, *id, &member))
        })
        .collect();
    for (name, t) in &loaded {
        per_set.entry(name.clone()).or_default().add(t);
    }

    let mut total = Tally::default();
    for (name, t) in &per_set {
        println!(
            "  {name:<40} members {:4} values {:10} nonzero {:10} differ {}",
            t.members, t.values, t.nonzero, t.differ
        );
        total.add(t);
    }
    println!(
        "members {}, points by category {}",
        total.members,
        CATEGORIES
            .iter()
            .zip(total.points)
            .map(|(c, n)| format!("{c} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "compared {} values ({} nonzero), differ {}",
        total.values, total.nonzero, total.differ
    );
    for e in &total.examples {
        println!("DIFF {e}");
    }

    let points: u64 = total.points.iter().sum();
    assert_eq!(
        total.members as usize,
        list.len() + synthetic.len(),
        "every member compared"
    );
    assert_eq!(
        total.values,
        points * VALUES_PER_POINT as u64,
        "every reading counted"
    );
    assert!(
        total.points.iter().all(|&n| n > 0),
        "a point category is empty: {:?}",
        total.points
    );
    assert!(
        total.nonzero * 2 > total.values,
        "fewer than half the readings are nonzero ({} of {}): the comparison is not exercising the grids",
        total.nonzero,
        total.values
    );
    assert_eq!(
        total.differ, 0,
        "readings differ from the coefficient tables"
    );
}
