---
type: Validation Methodology
title: Setting gate thresholds and reading headroom
description: "Tolerances sit at the algorithm's error scale with measured ulp headroom; tolerances and extremum-of-N statistics read headroom oppositely; margin is bought with points, never by widening."
status: draft
tags: [validation, tolerances, statistics, headroom, ulp]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n27-rule, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L11-L26", title: "Note 27 — never a loosened tolerance"}
  - {id: n28-k3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2021-L2064", title: "Note 28 K3.1 — a 1e-12 bound over an observed 0.0"}
  - {id: n28-k5a, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2795-L2817", title: "Note 28 K5a.2 — printing budgets and GRID_ALPHA_S_TOL"}
  - {id: n28-k5a2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2966-L2991", title: "Note 28 K5a2.3 — a flat bound plus a conditioned one-ulp bound"}
  - {id: n36a-read, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L16-L51", title: "Note 36a §0 — the two classes of threshold"}
  - {id: n36a-thin, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L326-L364", title: "Note 36a §7 — the thin list and what was done"}
  - {id: n37-cells, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L489-L528", title: "Note 37 §6.3 — a one-in-eight quintet, and a reference that reads high"}
  - {id: code-samples, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_samples.rs#L140-L177", title: "validate_samples.rs — P_FLOOR and its calibration"}
---

# Setting gate thresholds and reading headroom

The binding statements — ULP exactness is never the target; tolerances sit at
the algorithm's own error scale; an intolerable variation is a stability problem
to reformulate, not a comparison to relax — are in
[`AGENTS.md`](../../../AGENTS.md) ("ULP exactness is never the target",
"Samplers gate statistically"). This concept carries the working rules and the
cases.

## A cell goes green by resolution, never by loosening

A threshold is not moved to make a cell pass. A disagreement either is resolved
(and the cell goes green because the numbers now agree), or the cell stays
⚠️/`info` with a note saying exactly what is unresolved and why, and the finding
is filed[^n27-rule]. The workflow half of this — who fixes what, and when — is
[expose, don't fix](../workflow/expose-dont-fix.md). Every enforced threshold's
doc comment records the measurement it was set from, so a reading drifting
toward it is visible.

## Deterministic comparisons: algorithm scale, measured ulp headroom

Where both sides compute the same expressions on the same inputs, set the bound
well above last-ulp library noise and well below anything a wrong branch could
produce, and report the observed worst value rather than requiring it:

- The kT clustering replay asserts `1e-12` relative on every scale and merge
  measure. Observed worst over 90 000 dumped events: `0.0` — reported, not
  required, because a system libm may legitimately differ in the last place
  (`pow`, `cosh`, `log`)[^n28-k3].
- `αs(M_Z)` off the PDF grid matched MadGraph's 17-digit report to every bit,
  and `GRID_ALPHA_S_TOL` was set at `1e-14`: two orders above one `ln` call's
  arithmetic noise[^n28-k5a].
- When a gated quantity sits on a *printing* boundary (MadGraph writes `AQCDUP`
  to seven significant digits), the budget is the printing precision, and a run
  of events reading 0.999 of that budget is events piling against a rounding
  boundary, not a gate about to fail.

**Conditioning gets its own bound.** A flat relative bound cannot tell a
one-ulp residual on an ill-conditioned point from a real error. The PDF
continuation gate therefore makes two statements: a flat `EXTRAP_REL_TOL = 1e-11`
as the coarse net, and a sharp one dividing each point by its own condition
number `(|y_lo(1−t)| + |t·y_hi|)/|result|` and requiring the remainder to be
one ulp (`EXTRAP_CONDITIONED_TOL = 1e-14`; measured 8.93e-16 on NNPDF23)[^n28-k5a2].
The flat bound is `|Δ| ≤ EXTRAP_ABS_TOL + EXTRAP_REL_TOL·|want|` with an absolute
floor of `1e-30`, under the set's own `1e-10` positivity clamp, so dead corners
that are pure rounding residue are screened. Only the `above_q2max` branch is a
two-point line with a writable condition number; the other three continuation
categories have the flat bound alone (`validate_pdf_grid.rs`).
The amplitude oracle does the same per event point (`ULP_BUDGET = 10` times the
point's own one-ulp sensitivity; see [the amplitude oracle](amplitude-oracle.md)).

## Statistical thresholds: two classes that read headroom oppositely

The headroom census separates enforced thresholds into two classes, because the
same ratio of bound to reading means opposite things[^n36a-read]:

**(a) Tolerances** (`rel_tol`, `*_MAX_REL`, `SIGMA_REL_LIMIT`, `SIGMA_MAX_REL`)
bound a disagreement of physical size, which does not shrink by itself. Headroom
is bound ÷ measured value; under **2×** means the next change touching the row
may fail it.

**(b) Standardised thresholds** (`PULL_LIMIT`, `|pull| < 3.0`,
`SHAPE_PULL_LIMIT`, the `P_FLOOR`s, every `*_MAX_CHI2_PER_DOF`) are
false-positive rates against a trial count: the statistic is the largest (or
smallest) of `N` draws from a known null. The largest of five `|N(0,1)|` draws
has expectation 1.57, exceeds 2.0 a fifth of the time and 3.5 once in 430, so a
3.5σ limit reading "2× above the worst of five seeds" is the instrument working;
the census's per-row worst pulls averaged 1.562 over 31 gated rows. **More room
would mean the threshold had stopped rejecting.** The 2× rule does not apply, and
"form the statistic over more seeds" can make an extremum-vs-floor statistic
worse: more draws lower the expected minimum p and raise the flag rate. Seed
count matters to these through degrees of freedom instead: `χ²/dof < 4` is a
1.8 % false-positive rate on 2 dof and 0.30 % on 4.

## Buying margin

For a class-(a) statistic under 2×, the remedies are (in order) more seeds
forming the statistic, more points per seed, or recording the measurement with
its diagnosis — never a wider bound[^n36a-thin]:

- `pp_to_jj` `rel` vs `JJ_MAX_REL` stayed at 1.5×: the residual is a converged
  offset, not scatter, so `JJ_SEEDS` 3 → 5 changed what the gate reads (a
  five-seed mean, χ²/dof on 4 dof) but not the ratio.
- `ddx_to_epemg` (offset) and `gux_to_epemux` (one seed of five) were recorded
  with their diagnoses; four times the points buys them margin, and that budget
  decision belongs with the row's ladder ([budget alignment](budget-alignment-rule.md)).
- `validate_unweighting`'s σ pull went 1.77× → 2.2× by `GEN_SEEDS` 4 → 5, which
  buys the denominator of a pull on a seed mean.

Class-(b) readings under 2× are recorded, not acted on. `validate_samples`'
`P_FLOOR = 1e-4` reads 1.57× (`ee_to_wpwm` `pt(w+)`, `1.573e-4`) at three and
at five seeds alike; its doc comment carries the standing prescription — a
column below the floor is recorded, the row marked informational, the
disagreement filed, the floor not moved.

## A threshold firing is a measurement too

When a standardised guard fires, measure its calibration before reading it as a
defect. `pp_to_llj_dyn`'s five-seed scatter guard read χ²/dof 4.17 (limit 4.0)
under one map; forty seeds of the gate's own configuration read 0.91 under both
maps, with spread ÷ quoted error 0.91–0.95, and one of eight disjoint quintets
above 4.0 — the gate's five had drawn a one-in-eight quintet
(`probe_llj_dyn_scatter_guard_calibration`)[^n37-cells]. A
[seed sweep](seed-sweeps-and-budget-ladders.md) separates that from a real miss;
the current per-statistic readings are
[the seed headroom census](seed-headroom-census-2026-09.md). A failing samples
cell can also be the reference's fluctuation: two samples of this crate's own
under the two maps compared at χ² 40.5/47 on the weighted flavour composition,
twice as sensitive per category as the gate, while the reference's own flavour
χ² already read high.

[^n27-rule]: Note 27, charter. The rule predates it (note 25 §8) and is restated in every validation sprint since.
[^n28-k3]: Note 28 K3.1, over the nine runs dumped then; the manifest now pins eight (80 000 events).
[^n28-k5a]: Note 28 K5a.2.
[^n28-k5a2]: Note 28 K5a2.3.
[^n36a-read]: Note 36a §0.
[^n36a-thin]: Note 36a §7.
[^n37-cells]: Note 37 §6.3.
