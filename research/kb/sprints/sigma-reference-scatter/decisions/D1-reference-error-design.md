---
type: Design Decision
title: "MadEvent σ references: independent seeds, recorded iterations, equal-weight recombination"
description: "Proposed: every gated MadEvent σ reference banks fresh-directory seeds at two budgets and keeps each channel's iterations; the gate reads the equal-weight recombination, with the measured seed scatter folded into its error."
status: draft
tags: [madevent-reference, sigma, seeds, iteration-combination, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: seed-policy, resource: "../../../validation/madevent-reference-seed-policy.md", title: "Seed policy for MadEvent references"}
  - {id: pr22, resource: "https://github.com/nsmith-/vibegraph/pull/22", title: "sigma-2to4-offset: the iteration-weighting bias on a fixed-beam 2 → 4"}
  - {id: pr26, resource: "https://github.com/nsmith-/vibegraph/pull/26", title: "sigma-ee-mumua: the bias that does not shrink with budget"}
  - {id: pr28, resource: "https://github.com/nsmith-/vibegraph/pull/28", title: "sigma-uux-wbwb-veto: the W-pair continuum deficit"}
---
**Awaiting the user's decision.** Nothing here is implemented.

## What this sprint measured

MadEvent combines each channel's VEGAS iterations with weights
mean²/variance (`dsample.f:1995-1997`). An iteration that misses a peak reads
both low and precise, so it gets extra weight. The seed policy recorded the
effect once, on `pp_to_llj` (−0.146 %). This sprint measured it on three more
rows, all pinned 3.7.1 with fresh directories:

| row | reported σ vs an adaptation-free estimate | equal-weight recombination vs the same estimate | shrinks with budget? |
|---|---|---|---|
| fixed-beam `u u~ > e+ e- g g` (#22) | −0.17 % (60 seeds) | +0.02 % | yes |
| `ee_to_mumua` (#26), banked 10k | −0.95 % | −0.55 % | no: −0.94 % at 100k, χ²/dof 3.6 |
| `ee_to_mumua` at 100k | −0.94 % | −0.19 % ± 0.34 % | — |
| `u u~ > w+ b w- b~ / t t~` (#28), fixed μ | −1.9 % (1 seed) | moves up 0.5–1.2 % | no clear trend |
| `gg_to_gg_cg` (#27): not this bias, a single draw | banked run +0.22 % above fresh seeds at 10× budget | — | — |

On every row this side agrees with the adaptation-free estimate within its
errors. The reported MadEvent value is the outlier, and its quoted error does
not cover the bias. Seeds sharing a directory hide the bias further, because
they inherit its grid (χ²/dof 0.3–0.8 on the MLM rows).

A single-run reference is one draw. `gg_to_gg_cg`'s bank (iseed 21) sits 2.1
seed sd above MadEvent's own converged value, and this side agrees with the
converged value to 6e-6 (#27). Every single-run fixed-beam row in
`sigma_reference.json` has the same exposure.

## Options

- **A (proposed). Recombine, and record the scatter.**
  - Every gated σ reference, the single-run fixed-beam rows of
    `sigma_reference.json` included (first `gg_to_gg_cg`, expected 2.7065e5),
    is banked from ≥ 5 fresh-directory seeds at the
    banked budget plus ≥ 2 at 4×. Each run keeps its `G*/results.dat`.
  - The bundle records, per seed, the reported σ and the equal-weight sum of
    each channel's iterations, and, per row, both values' seed χ²/dof and the
    budget shift.
  - The gate reads the equal-weight mean. Its error is the seed policy's
    max(quoted, spread/√n), widened to the measured budget shift where that
    is larger.
  - A row whose equal-weight mean still moves with budget (`ee_to_mumua` at
    10k) is re-banked at the budget where it stops moving.
  - Cost: a recombination script beside `madevent_seeds.sh`, the bundle
    schema, the σ gates' reader, and an M3 re-bank of every gated row.
- **B. Keep the reported σ and register per-row tolerances.** No re-bank,
  but each row carries a hand-sized allowance for a bias the reference does
  not quote, and a new row's allowance is unknown until someone measures it.
- **C. Raise the MadEvent budgets until the bias is negligible.** It fails on
  `ee_to_mumua`, which does not converge at 10× budget.

A non-adaptive estimator per row, the third opinion three sessions here
wrote, is the strongest reference but is process-specific code. It stays a
diagnostic, not a gate.

## What A cannot see

- A region that every MadEvent iteration misses. Equal weights do not help
  there, and only an independent estimator can find it.
- The `#28` continuum deficit may be partly that.

## Amendment after V1 (approved by the user, relayed 2026-10-15)

[V1](../sessions/V1-report.md) measured 20 fresh seeds on each llj MLM row and
5 + one 4× on five single-run hadronic rows. Two points bear on A as approved:

1. **Equal-weight recombination is not validated on every row.** It closed
   the offset against an adaptation-free estimate on the fixed-beam 2 → 4
   (#22) and on `ee_to_mumua` at 100k (#26). On `pp_to_llj_xqcut_only` it
   reads this side −0.26 % (−2.5σ). Each channel refines for a
   precision-dependent number of iterations (3 to 21), so an equal-weight mean
   carries an optional-stopping exposure no estimator here has measured.
   - **Proposed:** bank both values per seed, as A already says. The gate
     reads the equal-weight mean only on rows where an adaptation-free check
     validated it; elsewhere it reads the reported-σ seed mean, and the
     equal-weight difference is recorded but not added to any tolerance.
2. **Two runs at 4× cannot calibrate a budget shift** against a 1× spread from
   five seeds (the ladder rule asks for 20+ on both rungs).
   - **Proposed:** keep 5 + 2 × 4× for every row as a screen. A row whose 4×
     runs sit more than 2σ from its 1× mean gets a 20 + 20 ladder before it
     is banked.

### Rows and estimated M3 time

These are the 54 banked or long-tier σ rows in `validation/manifest.toml`.
Seeded generators already exist for the decay-chain and grammar rows. At
about 0.7× this host's per-run time, a 10k-event row costs about 10
single-run units under A, roughly 5 min:

| group | rows | M3 estimate |
|---|---|---|
| fixed-beam single-run rows (ee, uux, gg, toy, SMEFT) | ~40 | ~3.5 h |
| hadronic single-run rows (`pp_to_*`, all banked at iseed 21) | 9 | ~1 h |
| llj MLM rows (fresh seeds measured in V1; add 4×) | 3 | ~0.5 h |
| `ee_to_mumua` at 100k events, `m(μμ)` cells (#26) | 1 | ~2 h |
| `pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm` | 2 | ~2 h |
| long-tier 2 → 6 rows | 2 | not timed; probably several h |

The total is roughly 9 h plus the 2 → 6 rows, before any 20 + 20 ladders.
These are estimates from this host's timings and are not measured on the M3.

## Second amendment after the survey finding (approved by the user, relayed 2026-10-16)

PR #23's V4 found that MadEvent's default survey left `g q > t t~ q` 0.55 %
low, and that `hard_survey = 2` removes it. A cross-check on the other rows
diagnosed in this sprint (all pinned 3.7.1, fresh directories, compared with
an adaptation-free or vibegraph value):

| row | default, reported | `hard_survey = 2`, reported | `hard_survey = 2`, equal-weight | cost |
|---|---|---|---|---|
| ttx gq, unmatched (#23) | −0.58 % | **−0.03 %** | — | ~1× |
| `pp_to_llj_xqcut_only` (20 seeds) | +0.08 % (EW −0.26 % vs ours) | **−0.005 %** | −0.009 % | 1.09× |
| `ee_to_mumua` (39 seeds) | −0.95 % | −0.66 % (−14σ) | −0.26 %, heavy-tailed | ~1.05× |
| uux top-free (20 seeds) | −2.03 % | −0.68 % (−6σ), χ²/dof 4.4 | +0.21 %, heavy-tailed | ~0.3× |
| fixed-beam 2 → 4 (#22) | −0.12 % | **inert**: `mmjj > 0.01 · (ebeam1 + ebeam2)` already forces the harder survey (`madevent_interface.py:3526`) | — | — |

`hard_survey = 2` fixes the rows where MadEvent missed through its survey grid
or a long refine chain. It does not fix the rows with rare, very large
weights (`ee_to_mumua`'s G6/G8, uux's G8/G9). There the equal-weight value
is too heavy-tailed at 10k events to bank.

**Proposed:**
1. Bank every row with `hard_survey = 2`. The cost is negligible.
2. Record each run's effective `min_events`/`max_events` (`run_config.inc`).
   A card where `hard_survey` is inert is then visible rather than assumed.
3. A row whose channels scatter beyond their quoted errors under
   `hard_survey = 2` is banked as usual, but its σ cell stays `info` against
   MadEvent. If an adaptation-free estimate exists, that is recorded beside it.
   **The scatter rule** (third amendment): per channel, the seed χ² of the
   reported values about their unweighted mean at the banked budget, and its
   p-value on n − 1 dof. A row is flagged when the worst channel's p-value
   times the number of channels tested is below 1 % (a look-elsewhere
   correction), so a row whose quotes are right flags by chance at most 1 %
   of the time, however many channels it has. A 5-seed flag is confirmed by
   the same test on 20 seeds, and the cell goes `info` only if it confirms.
   The record also keeps, for information, the count of channels above
   χ²/dof 3 and the count expected there by chance. The screen lists any
   flagged or confirmed row back to the user.
   Today two rows are `info` against MadEvent on an attribution, whatever the
   statistic reads: `ee_to_mumua`, on #26's rare large weights in G6/G8, and
   the uux continuum (not a gated row). Both do scatter: over 39
   `hard_survey = 2` seeds `ee_to_mumua`'s worst channel, G8, is at χ²/dof 2.78
   on 38 dof (p = 1.6·10⁻⁷ after correcting for its 6 channels), and over 20
   seeds the uux continuum's G9 is at 12.08.
4. Two driver checks: a run that writes fewer events than asked fails, and so
   does a σ that is not a number. `generate_events` exits 0 on the
   `libgammaUPC.a` compile race.

Unchanged: fresh seeds, the 4× screen, and both values recorded per seed.

**The two cases named explicitly:**

- **(a) The top-free continuum `u u~ > w+ b w- b~ / t t~` (#28).** It is not
  a banked or gated row: no gate reads `uu_tt` or a top-free row, and #28
  closed by documenting the deficit. So it is not in this batch. If a row for
  it is ever added, it falls under point 3: banked with `hard_survey = 2`, its
  σ cell `info` against MadEvent (−0.68 % after `hard_survey = 2`, channels
  G8/G9 at χ²/dof 6–12), with #28's adaptation-free three-channel value
  recorded beside it. Its equal-weight value is not used: it is too
  heavy-tailed at 10k events.
- **(b) #22's fixed-beam `u u~ > e+ e- g g` card, where `hard_survey` is
  inert.** It is a diagnostic card, not a banked row, so there is nothing to
  bank. #22 stays closed on its attribution, MadEvent's iteration combination,
  which `hard_survey = 2` cannot change because MadEvent's harder survey
  already runs there. In general, a banked row whose card has
  `mmjj > 0.01 · (ebeam1 + ebeam2)` shows up in its recorded effective
  `min_events`/`max_events`, and the screen lists it back. Its σ cell then
  follows the first amendment: the equal-weight value only where an
  adaptation-free check validated it, otherwise the reported mean. It stays
  `info` if point 3's scatter rule flags it. No committed banked card is
  known to trip the condition; the hadronic threshold is 130 GeV.

The durable record of the survey bias is the concept
`validation/madevent-default-survey-bias.md` on PR #23's branch. It is linked
from `madgraph-defect-policy` as a MadGraph approximation, and listed as an
upstream-report candidate.

## Third amendment: the scatter rule (decided 2026-10-16)

The user's decision, relayed by the planner, in two steps. F2 measured that
the second amendment's "a channel above χ²/dof 3" fires by chance on rows with
many channels: with 4 dof a correctly quoted channel exceeds 3 with
probability 7·e⁻⁶ ≈ 1.7 %, so a 50-channel row would flag 58 % of the time.
The user first chose to compare the count above 3 with the count expected by
chance. Below about 58 channels that expected count is under one, so read
literally it fires exactly as often as the old rule (measured on 5-seed
subsets: uux 98 %, `ee_to_mumua` 49.5 %, llj 1.3 % under both). The user then
chose the look-elsewhere form now in point 3 (measured: uux 92 %,
`ee_to_mumua` 21 %, llj 0 %; at 20 seeds uux 100 %, `ee_to_mumua` 87 %,
llj 0 %). A χ²/dof cut is also not one significance at different seed
counts: `ee_to_mumua`'s 2.78 over 39 seeds is far past chance. It stays
`info` on #26's attribution either way. The rule reads quantities every seed
record already holds, so no re-bank follows.

## Fourth amendment: what a 20 + 20 ladder decides (decided 2026-10-16)

The user's decision, relayed by the planner ("Accept the ladder rule with
measured-spread σ"). A row the 5 + 2 screen holds gets 20 seeds at its banked
budget and 20 at 4×. Then:

- **σ** is the combined standard error of the two rung means, from each rung's
  measured seed-to-seed spread: σ = √(s₁ₓ²/n₁ₓ + s₄ₓ²/n₄ₓ), with s the sample
  standard deviation of the seeds' reported values. The per-run quoted errors
  do not enter, following the ladder rule in `AGENTS.md`: few-seed scatter
  and quoted errors have understated heavy-tailed rows' spread.
- **If |mean₄ₓ − mean₁ₓ| > 2σ**, the row is re-banked at the 4× budget: its
  record's budget rung becomes the 4× seeds.
- **Otherwise** it stays at 1×, now with 20 seeds.

The `SCREEN` line's pull uses the seed-policy error (max of quoted and
spread/√n) and is a screen, not this verdict. Until `seed-record` prints the
ladder verdict itself, it is computed from the record's per-seed values.
First row under it: `ll_to_qqx_toy_dipole` (5 + 2 screen: 4× mean pull −2.34).
