---
type: Algorithm
title: "LHAPDF grids and the log-bicubic interpolation vibegraph replicates"
description: "LHAPDF6 .info/.dat layout, the local log-bicubic Hermite in (ln x, ln Q²), the subgrid walk and flavour aliasing, and the all-flavour f64 kernel (xfx_all) that evaluates it."
status: draft
tags: [pdf, lhapdf, interpolation, hadronic, kernel]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n18-11, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L40-L62", title: "Note 18 §1.1 (LHAPDF format; oracle backend switched to LHAPDF)"}
  - {id: n18-22, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L230-L245", title: "Note 18 §2.2 (PDF evaluation design)"}
  - {id: n18-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 decision records H1–H2 (oracle backend; in-house log-bicubic)"}
  - {id: n31-21, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L331-L390", title: "Note 31 §2.1–2.2 (subgrid structure; SIMT shape)"}
  - {id: n31-23, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L391-L485", title: "Note 31 §2.3–2.4 (the all-flavour kernel; Horner in x only)"}
  - {id: lhapdf-paper, resource: "https://arxiv.org/abs/1412.7420", title: "LHAPDF6: parton density access in the LHC precision era"}
---
# LHAPDF grids and the log-bicubic interpolation vibegraph replicates

MadGraph reads parton densities through the LHAPDF 6 C++ library, so a cross
section only matches MadGraph's if `x·f(x, Q²)` is LHAPDF's number, off the knots
as well as on them. vibegraph reads LHAPDF's `lhagrid1` files directly
(`vibegraph-lib/src/pdf/`) and reproduces LHAPDF's local log-bicubic
interpolator to libm-rounding level. Why it is an in-house port rather than a
generic spline crate is [pdf-interpolation-in-house](pdf-interpolation-in-house.md);
what happens outside the grid is
[pdf-extrapolation-and-force-positive](pdf-extrapolation-and-force-positive.md);
the gate is [lhapdf-oracle](../validation/lhapdf-oracle.md). The library is
described in [lhapdf6](https://arxiv.org/abs/1412.7420).

## The files

A set is a directory `<set>/` holding `<set>.info` and one `<set>_NNNN.dat` per
member. `PdfSet::load` / `pdf::grid` read both; only `Format: lhagrid1` is
accepted.

- **`.info`**: set metadata (flavours, `x` and `Q` ranges, member count, the
  extrapolator) and the `AlphaS_*` block (`AlphaS_MZ`, `AlphaS_OrderQCD`,
  `AlphaS_Type`, `AlphaS_Qs`, `AlphaS_Vals`, `AlphaS_Lambda4/5`). The coupling
  block is the set's own running, used when `pdlabel = lhapdf`
  ([alpha-s-sources](alpha-s-sources.md)).
- **member `.dat`**: a header, then one or more **subgrid blocks** separated by
  `---`. Each block has an `x`-knot line, a `Q`-knot line, a flavour line (PDG
  codes), then `nx × nq` rows of `nf` values of `x·f`. The parser checks the row
  count and row length per block (`GridError::ShapeMismatch`, `RowLength`).

Subgrids are **`Q²` bands sharing one `x` axis**, each a full `nx × nq × nf`
tensor; they partition the `Q²` range at flavour thresholds, with the seam knot
repeated as the top of one band and the bottom of the next. The pinned
`NNPDF23_lo_as_0130_qed` ([pinned-pdf-set](pinned-pdf-set.md)) has **one** band
(100 `x` × 50 `Q` knots, 14 flavours, `x ∈ [1e-9, 1]`, `Q ∈ [1, 10 000]` GeV);
`NNPDF31_lo_as_0130` has two (12 + 38 `Q²` knots, 11 flavours).[^n31-21]

**Flavours.** PDG `0` in a flavour list means the gluon `21`
(`normalize_flavor_pdg`). A flavour absent from the set evaluates to exactly 0.
A subgrid resolves a PDG code to its column with `flavor_index`; the all-flavour row
`FlavorRow` has a fixed slot per code (`flavor_slot`: six quarks, their
antiquarks, the gluon and the photon `22`, fourteen used of 16), the same for
every set, so a consumer resolves its beam flavours to slots once at setup.

## The interpolation (LHAPDF's `LogBicubicInterpolator`)

This is the `lhagrid1` default, `logcubic`: a **local** cubic Hermite in each of
`ln x` and `ln Q²`, with knot derivatives estimated by finite differences of the
`x·f` values. It is not a global B-spline: a scipy-style
`RectBivariateSpline` (cubic, interpolating) over the same knots misses LHAPDF
by up to 98.6 % relative at interior points of the pinned set (charm), and by at
least 3 % on every flavour.[^n18-5]

1. **Band selection.** The first band whose `(x, Q²)` support contains the point;
   a seam value lands in the lower band, because the seam is that band's upper
   edge. `LogBicubic::select_band` does it by binary search on the bands'
   upper `Q²` edges (`partition_point`), which is the same answer on any ordered
   member; an unordered one falls back to the linear walk.
   `subgrid_walk_selects_first_in_range_band` pins the seam semantics.
2. **Knot location.** `indexbelow`: `upper_bound − 1`, clamped so `i + 1` is
   valid, in `ln x` and `ln Q²`.
3. **`x` direction, precomputed.** Per member, for every
   `(x interval, Q² knot, flavour)` four coefficients `[a, b, c, d]` of the cubic
   in the interval fraction `t`, mirroring LHAPDF's
   `GridPDF::_computePolynomialCoefficients(logspace = true)`, `_ddx` and
   `KnotArray::coeff`. The interior derivative is the mean of the left and right
   difference quotients; the edges take one-sided ones.
4. **`Q²` direction, at evaluation.** A cubic Hermite (`cubic_hermite`) across
   the four `Q²` knots around the point, built from the `x`-interpolated values,
   with central slopes inside a band and forward/backward ones at its lower/upper
   edge. A band with only two `Q²` knots falls back to bilinear.

Tables and evaluation are `f64`. Because the arithmetic follows LHAPDF's
per-point operation order, agreement with the LHAPDF oracle is at the
`ln`-rounding level: worst `1.32e-15` relative off-knot, `1.34e-16` at seams,
`1.95e-11` in the `x → 1` tail (inflated only by a ~`1e-19` near-zero antitop
value), and on-knot reproduction at worst `|Δ| = 2.7e-20`. The first three
were measured when the port landed; the later Horner rewrite of `cubic_x` left
every category unchanged or moved it at ulp level (worst conditioned residual
`8.93e-16 → 1.08e-15`), and the gate's bar is `1e-9` relative on interpolated
points. Bars and categories are in [lhapdf-oracle](../validation/lhapdf-oracle.md).

**An algorithm-independent check.** Data bilinear in `(ln x, ln Q²)` is
reproduced exactly, since a local Hermite with finite-difference slopes is exact
for functions linear in each log coordinate. That catches an `x`/`Q²` transpose
or a linear-in-`x` coordinate bug that a comparison to LHAPDF alone cannot
isolate.

**Multi-band coverage.** `validation/pdf/gen_oracle.cpp` recovers the per-band
structure from LHAPDF's flattened `KnotArray` (a shared `x` axis, the bands'
`Q²` knots concatenated with each seam knot duplicated) and emits seam probes on
both sides of and exactly at every internal boundary; for a single-band set the
seam category falls back to the global `QMin`/`QMax` edges. A two-band set such
as NNPDF31 therefore exercises the walk against LHAPDF itself, beside the
synthetic two-band fixtures in the unit tests.

## The kernel: `xfx_all`

A hadronic phase-space point needs densities at exactly **two** `(x, Q²)`
points, one per beam, however many subprocesses, flavour groups and beam
orderings are summed over it (the mirror term swaps flavours, not points).
`PdfMember::xfx_all(x, q2, &mut FlavorRow)` therefore computes the guards, band
selection, both logarithms, both knot searches, the interval fractions and the
four `Q²` Hermite basis weights **once**, then evaluates every flavour's cubics
off the contiguous per-cell coefficient block (`coeffs` is
`(ix, iq, ifl, 4)` row-major). `xfx_q2` reads one flavour and is the test and
oracle form; the two agree bit for bit. The luminosity sums
([proton-integrand](../hadronic/proton-integrand.md)) consume per-beam rows.[^n31-23]

The kernel is written in the shape a lane-parallel (SIMT) implementation needs:
index computation separate from arithmetic; branch-free `Q²` slope stencils
(the forward/backward/central cases differ by a clamped neighbour index and two
per-interval constants resolved at build time; the bilinear two-knot case is a
property of the band, decided at build time); a flavour-major inner loop. A
batch-of-points variant is a mechanical extension, not built because the
hadronic integrand is a single-point closure and nothing could feed it.
Extrapolation stays a scalar fallback.

**Horner in `x` only.** `cubic_x` evaluates `((a·t + b)·t + c)·t + d` with
`mul_add_fast` (fused where the target has FMA); the stored monomial
coefficients are already the Horner coefficients. `cubic_hermite` keeps LHAPDF's
operation order, for two reasons recorded in its doc comment, independent of
any test:

- at `t = 1` (and `t = 0`) the four basis weights are exact in binary, so the
  sum returns the knot value to the bit; a Horner chain reaches it only through
  cancellation, and on-knot reproduction at a band's top `Q²` knot degrades from
  `2.7e-20` to `2.7e-12`;
- where `x·f` has died near `x = 1` the four inputs cancel by some thirty orders,
  leaving a `~1e-35` rounding residue that matches LHAPDF only because the
  operation order is LHAPDF's; Horner moves it by parts in `1e5`.

Do not revisit: `cubic_hermite` runs once per flavour against `cubic_x`'s four,
so the forgone gain is bounded. The kernel's measured speed-up and its share of
hadronic run time are performance measurements of commit `865828a` (merge
`c999c16`, 2026-08-04, note 31 §2.4), not part of this design.

## Caveats

- Out-of-grid points are not interpolated: points outside a member's overall
  extent go to the `extrap` seam (LHAPDF's own continuation); a point inside the
  extent but in no band is `OutOfRange`, which a well-formed `lhagrid1` member
  never produces.
- The oracle and the interpolator read the same LHAPDF build MadGraph links
  (6.5.6 in the `madgraph` pixi environment). A different LHAPDF version on
  MadGraph's side would need the oracle regenerated against it.

[^n18-5]: Note 18 §5, H1 and H2.
[^n31-21]: Note 31 §2.1.
[^n31-23]: Note 31 §2.3–2.4.
