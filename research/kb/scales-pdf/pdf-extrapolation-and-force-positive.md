---
type: Algorithm
title: Out-of-grid PDFs and the ForcePositive clamp, as LHAPDF does them
description: "LHAPDF's ContinuationExtrapolator per quadrant with log-or-linear values and low-Q² power law, its refusals, and the ForcePositive clamp read from .info, applied after both."
status: draft
tags: [pdf, lhapdf, extrapolation, force-positive, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n28-k5a2-1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2915-L2944", title: "Note 28 §K5a2.1 (which continuation, and what pins each line)"}
  - {id: n28-k5a2-2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2945-L2965", title: "Note 28 §K5a2.2 (the oracle)"}
  - {id: n28-k5a2-3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2966-L2991", title: "Note 28 §K5a2.3 (the residual is one ulp of its conditioning)"}
  - {id: n28-k5a2-4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2992-L3008", title: "Note 28 §K5a2.4 (what is refused)"}
  - {id: n28-k5a2-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3009-L3032", title: "Note 28 §K5a2.5 (the ForcePositive finding)"}
  - {id: n29-e, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1522-L1544", title: "Note 29 chain E design (read before touching anything)"}
  - {id: n29-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1545-L1618", title: "Note 29 §E.1 (ForcePositive)"}
  - {id: n29-eb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1862-L1888", title: "Note 29 chain E (b) acceptance tests"}
  - {id: n29-ed, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1925-L1986", title: "Note 29 chain E (d) risks and blind spots"}
  - {id: extrap-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/pdf/extrap.rs#L1-L57", title: "pdf/extrap.rs module documentation"}
  - {id: pdf-mod, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/pdf/mod.rs#L147-L270", title: "PdfMember::try_xfx_q2 and force_positive_clamp"}
---

# Out-of-grid PDFs and the `ForcePositive` clamp

A PDF grid stops where the fit stopped. NNPDF23's densities are tabulated to
`Q = 10 TeV`, and a per-event factorisation scale on a 13 TeV collider crosses that, so a
reader that refused out-of-grid points could not run a dynamical scale to completion.
MadGraph reads its densities through LHAPDF, so LHAPDF's continuation is part of the
reference a cross section must match, not a local choice. vibegraph reproduces it in
`vibegraph-lib/src/pdf/extrap.rs`.[^extrap-rs] In-grid interpolation is
[scales-pdf/lhapdf-interpolation](lhapdf-interpolation.md).

## How one value is produced

`PdfMember::try_xfx_q2(pdg, x, q2)` (`pdf/mod.rs`) follows `PDF::xfxQ2`'s order
(`src/PDF.cc:49`, LHAPDF 6.5.3, identical in 6.5.6):[^pdf-mod][^n29-e1]

1. **A point that is not a point is refused** (`PdfPointError::Unphysical`): non-finite
   `x` or `Q²`, `x ≤ 0` (the small-x continuation is a straight line in `ln x`, so `x = 0`
   sends LHAPDF itself to `±inf`), or `Q² < 0`. **`Q² = 0` is not refused**: the power law
   below is defined there and returns exactly zero, as LHAPDF does.
2. **An absent flavour returns exactly `0.0`**, before either branch and before the clamp,
   so it never reads as the clamp's floor.
3. **In range → interpolator, out of range → extrapolator**, both edges inclusive
   (`GridPDF::_xfxQ2`, `KnotArray::inRangeX/inRangeQ2`).
4. **The `ForcePositive` clamp**, applied last, to both branches.

`try_xfx_all` produces the same numbers for every flavour at once, bit for bit; out of
range it falls back to the scalar continuation per present flavour.

## The continuation

Neither fetched set's `.info` names an `Extrapolator`, so
`GridPDF::_loadExtrapolator` falls through `PDFInfo → PDFSet → Config`, and
`lhapdf.conf` says `Extrapolator: continuation` (LHAPDF 6.5.3's source tree and the
installed 6.5.6 alike). `mkExtrapolator` (`Factories.cc:113`) builds a
`ContinuationExtrapolator`. The oracle dumps the resolved name, so a build configured
differently fails the gate instead of quietly redefining the reference.[^n28-k5a2-1]

Each line is read off `ContinuationExtrapolator::extrapolateXQ2`:

| element | rule |
|---|---|
| the edges it reads | `xs(0)`, `xs(1)`, `xs(nx−1)`, `q2s(0)`, `q2s(nq−2)`, `q2s(nq−1)` of the **flattened** knot array |
| above the Q² ceiling | a straight line in `ln Q²` through the last two flattened Q² knots |
| below the x floor | a straight line in `ln x` through the first two x knots |
| past both | the Q² line at each of the two lowest x knots, then the x line between them |
| value or its log | `ln y` when **both** endpoints exceed `1e-3`, `y` otherwise (`_extrapolateLinear`) |
| below the Q² floor | `f(Q²_min)·(Q²/Q²_min)^γ`, `γ = anom·Q²/Q²_min + 1 − Q²/Q²_min` |
| that `anom` | `dlog f / dlog Q²` from a `1.01×` forward difference, floored at `−2.5`; exactly `1` if `|f(Q²_min)| < 1e-5` |
| `x` above the last knot | `RangeError`, the one direction with no continuation |

Interpolating in the logarithm of the value keeps a positive density positive however far
it is continued; a small value, or one of either sign, is continued linearly instead and
can go negative.

**The flattened edges are the trap.** vibegraph stores its grids per band; LHAPDF's
`q2s(nq−2)` is the second-to-last entry of *all* bands concatenated. For a two-band set
that is the upper band's penultimate knot, not anything of the lower band. `edges_of`
builds the concatenation explicitly, and the multigrid oracle's `above_q2max` probes catch
a per-band reading.

## What is refused, and why each is undefined

- **`x` above the grid's last knot**: `ContinuationExtrapolator` raises `RangeError`. For
  both fetched sets `xMax = 1`, so this coincides with an unphysical momentum fraction, but
  it is the extrapolator's refusal, not the physical-range check's; a set whose grid
  stopped short of `x = 1` would separate them.
- **A point that is not a point**, as above.
- **A point in no subgrid while inside the overall extent**: a gap between bands, which a
  well-formed `lhagrid1` member does not have. `OutOfRange` remains as exactly that
  condition, the interpolator's own error, and nothing else.[^n28-k5a2-4]

## The `ForcePositive` clamp

LHAPDF's switch on the resolved level, applied after interpolation or continuation and
after the absent-flavour zero:[^n29-e1]

| level | effect |
|---|---|
| `0` | nothing |
| `1` | `if (xfx < 0) xfx = 0` |
| `2` | `if (xfx < 1e-10) xfx = 1e-10` |
| other | `LogicError` in LHAPDF; refused by the `.info` parser here (`GridError::InvalidValue`) |

The level is `info().get_entry_as<unsigned int>("ForcePositive", 0)` through
`PDFInfo → PDFSet → Config`. vibegraph reads the set's own `.info` with a default of `0`
(`SetInfo::force_positive`, `pdf/grid.rs`), and `PdfSet::member` applies it
(`PdfMember::with_force_positive`). That this reproduces LHAPDF's resolution is a
hypothesis, pinned by `the_clamp_level_is_the_one_lhapdf_resolved` against the oracle's
resolved value on both sets.

`force_positive_clamp` is written as explicit comparisons, **not** `f64::max`: `max`
returns the non-NaN operand and would silently clamp a NaN, where LHAPDF's
`if (xfx < 1e-10)` is false for NaN and passes it through. `PdfMember::from_subgrids`
(in-memory fixtures) carries level `0`.

**It matters out of grid.** In range the clamp bites only where a density is physically
negligible. Out of range a small-x continuation can run negative: on NNPDF31
(`ForcePositive: 2`) the clamp fires on 205 of 935 out-of-grid probes, and the largest
value it replaces with `1e-10` has magnitude 25.7.[^n28-k5a2-5]

**It changes no banked number.** The pinned set, `NNPDF23_lo_as_0130_qed`, has no
`ForcePositive` key, so its level is `0` and the clamp is the identity on every path a
gated row takes (0 of 1190 probes clamp; continued values down to `−1.1e-5` survive).
NNPDF31 is in the tree only as the multi-subgrid shape fixture and is named by no run
card ([scales-pdf/pinned-pdf-set](pinned-pdf-set.md)).[^n29-ed]

## The oracle and its tolerances

`validation/pdf/gen_oracle.cpp`'s `extrapolated` block probes every flavour in each
out-of-range quadrant: 1190 probes on NNPDF23 and 935 on NNPDF31. Each record carries two
values, `xf_raw` (`Extrapolator::extrapolateXQ2` called directly, no clamp) and `xf`
(`PDF::xfxQ2`, what MadGraph sees).[^n28-k5a2-2]

| category | NNPDF23 probes | worst rel | NNPDF31 probes | worst rel |
|---|---|---|---|---|
| `above_q2max` | 560 | 2.37e-13 | 440 | 3.77e-14 |
| `below_xmin` | 168 | 0 | 132 | 1.83e-15 |
| `below_xmin_above_q2max` | 56 | 9.81e-16 | 44 | 5.15e-14 |
| `below_q2min` | 350 | 3.75e-16 | 275 | 6.80e-15 |
| `below_q2min_below_xmin` | 56 | 0 | 44 | 3.17e-14 |

**The residual is one ulp of the conditioning.** A straight line evaluated far outside the
two points that define it is a difference of much larger numbers: at `x = 0.7`, four
decades above the Q² ceiling, the gluon's continuation has condition number 1.9e3, and one
ulp on its endpoints becomes 2.4e-13. So the gate makes two statements:[^n28-k5a2-3]

- the flat bound `EXTRAP_REL_TOL = 1e-11`, an order above the worst case and orders below
  what a branch or knot-pair confusion produces;
- the sharp one, `the_upper_continuation_misses_lhapdf_by_one_ulp_of_its_own_conditioning`:
  each point divided by its own condition number `(|y_lo(1−t)| + |t·y_hi|)/|result|` must
  be one ulp, bounded at `EXTRAP_CONDITIONED_TOL = 1e-14` (8.93e-16 worst on NNPDF23,
  6.34e-16 on NNPDF31). Reconstructing the endpoints from the crate's own interpolator is
  sound because those are independently gated against LHAPDF's at ≤4e-16.

**Branch selection** is pinned against LHAPDF's values, not against the source it was
read from (`the_branch_of_the_upper_continuation_is_the_one_the_endpoint_values_select`):
184 NNPDF23 and 124 NNPDF31 probes sit where the log and linear candidates visibly differ,
with 205 and 180 on the linear branch, so neither branch is covered only in name.

**Which member each gate reads.** Gates against `xf_raw` must read an **unclamped** member
(`load_unclamped_member`, level forced to 0), or they fail on the 205 clamped NNPDF31
probes:[^n29-e] `extrapolation_matches_lhapdf_past_every_grid_boundary` and the two above. Gates
against `xf` read the set's own level (`load_member`).
`the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp` reads both: where
the clamp fired, the clamped reading is bit-equal to `xf`; where it did not, clamped and
unclamped agree bit for bit; counts asserted (205 of 935, 0 of 1190, and > 0 so the test
cannot go vacuous).[^n29-eb] `an_in_grid_value_lhapdf_floors_is_floored_here_too` checks
in-range points where LHAPDF returns exactly `1e-10`. The interpolation gates keep their
`FORCE_POSITIVE_FLOOR = 1e-8` screen, which now absorbs only the band where the two
libraries' raw readings could land either side of `1e-10`; the nearest in-range value
above the floor sits 10.4% above it. 22 of the multigrid oracle's knot values are
negative, so a clamp misplaced into the interpolator or `xf_at` fails
`multigrid_on_knot_values_match_oracle_exactly` at once.

**Blind spots** ([validation/lhapdf-oracle](../validation/lhapdf-oracle.md)):[^n29-ed]

- The in-range clamp gate cannot see the clamp's *input*: the oracle carries no unclamped
  in-range value, so a point both libraries wrongly floor is invisible.
- `the_clamp_level_is_the_one_lhapdf_resolved` cannot see a level wrong identically on
  both sides. It pins the crate against LHAPDF's resolution, not against MadGraph's own
  PDF call.
- `an_absent_flavour_is_zero_and_not_the_floor` is a unit test on a fixture: the oracle
  probes only `gpdf.flavors()`, so the absent-flavour ordering is invisible to it.

Closing the first two would need a new `gen_oracle.cpp` block.

[^n28-k5a2-1]: Note 28 §K5a2.1, the continuation and what pins each line.
[^n28-k5a2-2]: Note 28 §K5a2.2, the extrapolation oracle.
[^n28-k5a2-3]: Note 28 §K5a2.3, the residual as one ulp of the conditioning; branch selection.
[^n28-k5a2-4]: Note 28 §K5a2.4, what is refused.
[^n28-k5a2-5]: Note 28 §K5a2.5, the measured `ForcePositive` gap on NNPDF31.
[^n29-e]: Note 29 chain E design, the two load-bearing facts.
[^n29-e1]: Note 29 §E.1, `ForcePositive` as LHAPDF applies it, and the member table.
[^n29-eb]: Note 29 chain E (b), acceptance tests and what each cannot detect.
[^n29-ed]: Note 29 chain E (d), why no banked number moves, and the blind spots.
[^extrap-rs]: `vibegraph-lib/src/pdf/extrap.rs` module documentation.
[^pdf-mod]: `PdfMember::try_xfx_q2` and `force_positive_clamp`, `vibegraph-lib/src/pdf/mod.rs`.
