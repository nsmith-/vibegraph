---
type: Caveat
title: Adapted VEGAS grids make a heavy weight tail
description: "Adapting on Σ(f·w)² starves low-acceptance bins, giving Pareto tails (Hill index ≤2) so w_max and file-to-file σ never settle; MadEvent's acceptance rescale is a partial cure."
status: draft
tags: [vegas, weight-tail, unweighting, mlm, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-fa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1617-L2005", title: "Note 41 M5 and F-A, the weight tail localised"}
  - {id: n41-m6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2702-L2915", title: "Note 41 M6, xqcut-aware phase space, the τ floor, and the target run"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2520", title: "Note 41 F-B, channel merging and the pooled stop"}
  - {id: vegas-rs, resource: "vibegraph-lib/src/vegas.rs#L976-L1042", title: "BlockIteration::hist accumulating (f·w)², refine_grid"}
  - {id: mg-dsample-grid, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/Source/dsample.f#L1890", title: "MadEvent dsample.f, grid adaptation on Σ|w|"}
  - {id: mg-dsample-rescale, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/Source/dsample.f#L2106-L2124", title: "MadEvent dsample.f, per-bin acceptance rescale capped at 10⁴"}
measured:
  - {commit: ef660f3, landed_in: 403cff8, pr: 14, command: "30 000 scan points per channel on nine channels of M5's seed-20260928 pp_to_ll_0j2j_mlm artifact, banked grid vs flat grid over the same map; env-gated instrumentation, never committed"}
---

## The caveat

A trained VEGAS grid buys most of a channel's variance reduction and pays for
it with a heavy upper tail in the per-point weight. On the tail's own scale
the variance barely exists, so anything read off the largest weights — a
channel's `w_max`, the overweight share of an event sample, a sample's σ, a
per-iteration χ² — fluctuates far more than its error bar says. The cause is
the grid, not the phase-space maps. This bounds how unweighting, sample σ and
the convergence stop may be read on any row with cut-heavy, floor-bound
channels, and most visibly on matched MLM rows.

## Where the tail comes from

On one `pp_to_ll_0j2j_mlm` artifact, 30 000 scan points per channel on nine
channels, with the banked grid and with a flat grid over the same map:[^n41-fa]

| | banked grid | flat grid |
|---|---|---|
| `@0` channels: per-point relative variance | 0.8–1.1 | 158–178 |
| `@0`: Hill tail index (top 20 of ~25k) | 1.3–1.7 | 5.3–6.5 |
| `@1` channels: per-point relative variance | 3.9–14.7 | 23.0–23.6 |
| `@1`: Hill index | 0.9–1.9 | 5.8–9.3 |
| `@1`: peak / mean | 118–433 | 77–87 |

Adaptation buys 1.6–200× in per-point variance and leaves a tail of index
≤ 2. On every channel dumped, the heaviest weights sit in single wide bins of
the first two coordinates: `@0`'s edge bins are 17× the average width (a large
lepton-pair rapidity, where `etal = 2.5` keeps only a narrow slice of decay
angles), `@1`'s 10–40× and 13–19×. The grid adapts on the per-bin sum of
`(f·w)²` (`refine_grid`, `vegas.rs`)[^vegas-rs], so a bin that mostly fails
the cuts is starved and widened, and the few points that pass in it carry its
width into their weight ([VEGAS integrator](vegas-integrator.md)).

## MadEvent's counter, and how far it goes

MadEvent adapts on `Σ|w|` per bin (`dsample.f:1890`)[^mg-dsample-grid] and
rescales each bin by `non_zero/inon_zero`, the inverse of the bin's own
acceptance, capped at 10⁴ (`dsample.f:2106-2124`):[^mg-dsample-rescale]

```fortran
grid(1,i,j) = grid(1,i,j)
&    *dble(min((real(non_zero)/real(inon_zero(i,j))), 10000.))
```

Adding that rescale to the histogram `adapt_blocks_iteration` returns, as a
probe only, moved `pp_to_llj_mlm`'s median Hill index from 1.8 to 2.2 and its
worst channel's from 1.0–1.2 to 1.5–1.6 (two seeds each, 150k × 10). It helps
and does not cure; the rest of the tail is not located. Any of the candidate
rules — the acceptance rescale, `Σ|w|` adaptation, a bound on bin width —
moves every integration, so each must be measured on the banked rows before
adoption: [VEGAS grid starves low-acceptance bins](../backlog/performance/vegas-grid-starves-low-acceptance-bins.md).

A higher but provably implied `τ` floor also fattens the tail: on
`pp_to_llj_fixed` the partition bound `√ŝ ≥ 50 + 20·n_j` raised the summed
`w_max` from 7.80e3 to 1.33e4 and cut the predicted unweighting efficiency
from 5.45% to 3.18%, with σ unmoved, and was reverted. Why is not
diagnosed:[^n41-m6]
[τ floor fattens the weight tail](../backlog/performance/tau-floor-fattens-weight-tail-undiagnosed.md).

## What the tail does downstream

- **`w_max` does not settle with scan budget.** Raising every channel's scan to
  ≥ 8 000 points cost 8× the CPU, removed 0.3% of σ from above `w_max`
  (2.65% → 2.34%), raised `Σ w_max` from 3.30e4 to 7.61e4 and halved the
  efficiency (3.21% → 1.40%); the largest `w/w_max` grew from 36.5 to 65.8. No
  scan floor is proposed. The truncation ladder cannot step off the top of a
  scan whose single largest weight exceeds 1% of its sum, which under an index
  near 2 is any scan below ~10⁴ points; the floor-bound channels scanned
  0.8–3.9k[^n41-fa]. How the maximum is chosen over such a tail, and the
  tail index measured on the `w_max` scans themselves, is
  [unweighting](../events/unweighting.md).
- **Overweights concentrate.** The excess above `w_max` was 2.65% of σ on
  M5's seed-20260928 sample, 0.9% of `@0`'s own σ (where the truncation rule
  asks for 1%), 4.6% of `@1`'s and 7.4% of `@2`'s; two `@1` channels carried
  45% of the sample's `Σr²`. Overweights are vetoed by the matching more often
  than other events, so counting events instead of weights moves the matched
  acceptance (0.6431 → 0.6513).
- **A file's σ scatters beyond its stated error.** A file declares
  `Σ w_max · Σ(event weights) / trials`, the sample's own estimate; at 10 000
  events and 3% efficiency its binomial error alone is ~1%, while `XERRUP`
  carries the integration's ±0.16%, about 7× smaller. MadEvent pins each
  channel's events to its integral, so its `@N` shares scatter well inside
  their binomial variance. The LHEF writer therefore normalises each
  integrated part's events to that part's integrated σ (`lhef/emit.rs`), which
  removes the excess without bias; overweights themselves survive it
  ([multi-process normalisation](../events/multi-process-normalisation.md)).
- **One spike holds a convergence stop.** A target run reads each channel's
  error widened by an iteration-consistency factor; a single point in one
  floor-bound two-jet channel carrying 0.004% of σ put that channel's χ²/dof
  at 5 675. The pooled factor (`budget.rs`, `pooled_scale`) together with
  merging channels that share a map now stops this row at 8–10 iterations,
  with σ inside the fixed-budget sweep. The tail is still there; its handling
  in the stop is [convergence stop rule](convergence-stop-rule.md)[^n41-fb].

The cost structure of the matched row this comes from is in
[mixed-multiplicity integrand](mixed-multiplicity-integrand.md) and
[MLM @2 excess decomposition](../validation/mlm-at2-excess-decomposition.md);
the per-channel arrangement that makes per-channel maxima possible at all is
[per-channel grids](per-channel-vegas-grids.md). Pricing the tail in the
budget allocation is open:
[MLM @2 tail unpriced in allocation](../backlog/performance/mlm-at2-tail-unpriced-in-allocation.md).

[^n41-fa]: Note 41 M5 and "F-A Landed: the weight tail".
[^vegas-rs]: `vibegraph-lib/src/vegas.rs`, `BlockIteration::hist` and `refine_grid`.
[^mg-dsample-grid]: MadEvent `dsample.f` line 1890, `grid(1, i, j) = grid(1, i, j) + abs(wgt)`.
[^mg-dsample-rescale]: MadEvent `dsample.f` at the pinned commit.
[^n41-m6]: Note 41 M6, "The τ floor, built and dropped".
[^n41-fb]: Note 41 F-B, the pooled stop after channel merging.
