---
type: Validation Gate
title: LHAPDF PDF oracle
description: "gen_oracle.cpp against MadGraph's own LHAPDF 6.5.6 over knot, off-knot, seam, tail and corner categories, an analytic bilinear oracle, and validate_pdf_grid's per-category bars."
status: draft
tags: [pdf, lhapdf, oracle, interpolation, alpha-s]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-11, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L40-L62", title: "Note 18 §1.1 — why the oracle is LHAPDF, not a spline"}
  - {id: n18-3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L352-L378", title: "Note 18 §3 — validation regime"}
  - {id: n18-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 H1/H2 — oracle backend, scirs2 rejected, accept bars"}
  - {id: n31-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/31-perf-sprint-3-plan.md#L431-L485", title: "Note 31 §2.4 — the real accept bars, LHAPDF's operation order, the absolute screen"}
  - {id: code-gen, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/validation/pdf/gen_oracle.cpp#L1-L30", title: "validation/pdf/gen_oracle.cpp"}
  - {id: code-gate, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/validate_pdf_grid.rs", title: "vibegraph-lib/tests/validate_pdf_grid.rs"}
  - {id: code-interp, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/pdf/interp.rs#L655-L700", title: "pdf/interp.rs — bilinear_in_log_is_reproduced_exactly"}
---

# LHAPDF PDF oracle

MadGraph evaluates parton densities through the LHAPDF C++ library
(`pdlabel = lhapdf`), so LHAPDF's own numbers are the reference this crate's
in-house interpolator must match — not a generic interpolation of the same grid.
A scipy-style global `RectBivariateSpline` over `(ln x, ln Q²)` is a different
algorithm from LHAPDF6's local log-bicubic and diverged from it by up to ~120 %
off-knot on the pinned set; only the knot values are common to both[^n18-11].
Why the interpolator is in-house is
[the in-house PDF interpolation decision](../scales-pdf/pdf-interpolation-in-house.md);
the algorithm is [LHAPDF interpolation](../scales-pdf/lhapdf-interpolation.md).

## The generator

`validation/pdf/gen_oracle.cpp` is a standalone C++ program against the LHAPDF
6.5.6 that MadGraph's pixi environment builds — the same library MadGraph runs,
so there is no separate version pin to drift. It loads a member through
`GridPDF`, reads knot coordinates and raw grid values straight out of the
`KnotArray` for the `knot` category, and evaluates `xfxQ2` for the interpolated
categories, writing JSON at `%.17g` (round-trip exact)[^n18-5]:

> The on-knot ("knot") category reads raw grid values straight out of LHAPDF's
> KnotArray, which is the ground truth the pure-parser gate checks; the
> interpolated categories (off_knot, seam, x_to_one_tail, corner) are evaluated
> through LHAPDF's xfxQ2 and banked for the interpolation gate.

It handles single- and multi-subgrid sets: LHAPDF flattens all Q² bands into one
`KnotArray` with each seam knot duplicated, and the generator recovers the bands
and emits seam points on both sides of and exactly at every internal boundary
(for a single-subgrid set `seam` falls back to the global Q edges). It also dumps
the set's `AlphaS_Ipol` coupling across and past its table, and an `extrapolated`
block: the set's own resolved `Extrapolator` past every grid boundary, with and
without the `ForcePositive` clamp, since nothing but the library can say what a
set does out there.

Two committed dumps, two grid shapes:

| file | set | shape |
|---|---|---|
| `validation/pdf/oracle.json` | NNPDF23_lo_as_0130_qed (lhaid 247000, the pinned set) | one rectangular Q² subgrid, `nx = 100`, `nq = 50`, x ∈ [1e-9, 1], Q ∈ [1, 10⁴] GeV |
| `validation/pdf/oracle_multigrid.json` | NNPDF31_lo_as_0130 | two Q² subgrids joined at one seam (Q = 4.92 GeV) |

The pinned set is [the pinned PDF set](../scales-pdf/pinned-pdf-set.md).

## The gate: `validate_pdf_grid`

Banked layer, `extended-validation`: the dumps are committed, but the gate needs
the two fetched PDF sets. Its bars, per kind of check:

| check | bar | measured |
|---|---|---|
| knot values (parser) and subgrid structure | `REL_TOL = 1e-12` (locates knots; parsers may differ in a last bit) | exact (0.0) |
| interpolation: `off_knot`, `corner`, `x_to_one_tail`, multigrid `off_knot` and `seam` | ≤ 1e-9 relative | `off_knot` 1.32e-15; `x_to_one_tail` 1.95e-11 (from a ~1e-19 antitop value) |
| on-knot interpolation reproduces the node | atol + rtol (a pure relative bar is meaningless against an exact zero at x = 1) | 2.7e-20 absolute |
| multigrid continuity across seams | — | the walk lands each probe in a band whose interpolant matches |
| `αs` from the set's table, and past both ends | `ALPHA_S_REL_TOL = 1e-14` | above the table LHAPDF freezes rather than extrapolates, and so does this crate |
| continuation past every boundary | `EXTRAP_REL_TOL = 1e-11` relative plus `EXTRAP_ABS_TOL = 1e-30` | — |
| continuation, conditioned | `EXTRAP_CONDITIONED_TOL = 1e-14` after dividing by the point's condition number | 8.93e-16 (NNPDF23), 6.34e-16 (NNPDF31) |

The interpolation bar is loose because this crate's arithmetic mirrors LHAPDF's
per-point operation order: agreement is at `log`-rounding level, and the bar
leaves room for platform libm differences. The `1e-12` constant only locates
knots; the real bars are 1e-9 (interpolation), 1e-11 (flat continuation) and
1e-14 (conditioned continuation, the tightest and the one a reassociation would
break)[^n31-p1]. The absolute screen `1e-30` sits four orders below one ulp of
the `ForcePositive` floor (1e-10, the smallest magnitude LHAPDF treats as a
density), so only pure-residue and exact-zero probes see a different bar.

The continuation tests pin branch selection against LHAPDF's values, not against
the source they were read from: for every `above_q2max` probe both candidate
continuations are built from this crate's edge readings, and LHAPDF's number must
be the one its endpoint values select (184 NNPDF23 and 124 NNPDF31 probes sit where
the candidates visibly differ). `the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp`
and `the_clamp_level_is_the_one_lhapdf_resolved` pin the clamp. Out-of-grid
evaluation continues through `ContinuationExtrapolator` (`pdf/extrap.rs`); a
typed `OutOfRange` is the interpolator's own error, and only points LHAPDF itself
has no reading for are refused. See
[out-of-grid PDFs and ForcePositive](../scales-pdf/pdf-extrapolation-and-force-positive.md)
and [α_s sources](../scales-pdf/alpha-s-sources.md).

**Operation order is load-bearing.** `cubic_hermite` keeps LHAPDF's operation
order: at `t = 1` the Hermite basis weights are exact in binary, so that order
returns the knot value bit-for-bit, while a Horner/FMA chain reaches it only
through cancellation (on-knot reproduction degrades 2.7e-20 → 2.7e-12). That is a
stability reason independent of any oracle, recorded in the function's doc
comment; Horner+FMA is used in `cubic_x` only[^n31-p1].

## The analytic oracle

`pdf::interp`'s `bilinear_in_log_is_reproduced_exactly` (default suite): data
bilinear in `(ln x, ln Q²)` is reproduced exactly, because a local Hermite with
finite-difference slopes is exact for functions linear in each log coordinate. It
catches an x/Q² transpose or a linear-in-x-versus-ln-x bug that the LHAPDF
comparison alone cannot isolate. `grid.rs`'s `parses_multiple_subgrids` pins the
parser on a synthetic two-band fixture.

## Blind spots

- An oracle of `x·f` values cannot see a mislabelled flavour whose grid values
  happen to coincide, nor a convention shared by both libraries.
- The seam gate confirms each probe lands in a band whose interpolant matches
  LHAPDF, not that a particular internal structure was traversed.
- Two sets only. A set with a different interpolator or extrapolator name in its
  `.info` fails here by design rather than silently redefining the reference.

[^n18-11]: Note 18 §1.1.
[^n18-5]: Note 18 §5 H1/H2. Its "single-subgrid only" generator limit and "out-of-grid → OutOfRange" non-goal are both superseded: the generator handles multi-subgrid sets, and continuation is implemented.
[^n31-p1]: Note 31 §2.4 and its P1b follow-up.
