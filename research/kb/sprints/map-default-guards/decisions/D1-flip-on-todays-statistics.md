---
type: Design Decision
title: "Flip the τ and soft-all defaults on today's guard statistics"
description: "Decided: V0 found both guards pass under the flipped defaults at the tip, with measured false-alarm rates at or below their Gaussian design, so neither statistic changes; both defaults flip together, the banked layer is re-run under them before the σ calibrations are re-recorded, and the jj composition excess common to both maps is filed on its own."
status: draft
decided: 2026-10-17
decided_by: human:nsmith-
tags: [maps, tau-rule, soft-all, samples-gate, scatter-guard]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: v0, resource: "../sessions/V0-report.md", title: "V0 report: the null distributions behind the jj flavour χ² and the ℓℓj scatter guard"}
---
**Approved by the user as written, relayed by the planner on 2026-10-17.**

## What V0 measured (commit `b23e246`, at today's tip)

Both backlog items quote note 37 §6.3, measured on `main@02e8b25`
(2026-09-23), 34 commits ago. At the tip:

| guard | then | now, flipped default | limit |
|---|---|---|---|
| `pp_to_jj` flavour χ², the gate's own run with τ flipped | p 2.8e-5 | min p **5.58e-2** (75.5/66, 87.5/68, 80.0/66) | 1e-4 |
| `pp_to_llj_dyn` gate quintet χ²/dof under soft-all | 4.17 | **1.96** | 4.0 |
| the same under today's default | 2.17 | 2.25 | 4.0 |

The calibrations are honest:
- **ℓℓj, 80 seeds per map.** χ²/dof is 0.93 (default) and 0.94 (soft-all), with no seed beyond 3σ. The low seeds' deficits are spread over all six channels, so no region is missed. The guard as it stands trips 0.07 % / 0.00 % of random quintets, against 0.30 % designed.
- **jj, both nulls nominal.** This side against itself reads per-seed χ²/dof 0.98–1.00; MadGraph half against half reads 0.99.

The flavour χ² against MadGraph reads high **by the same amount under both maps
and both grids** (per seed 1.19–1.23, pooled over 5 seeds 1.30–1.50). That is
not the map's doing, and it is close to one draw of the one banked 10k sample.

## Proposal

1. **No statistic changes.** The items asked to match each statistic to its
   calibration. At the tip both already pass under the flipped defaults, with
   measured false-alarm rates at or below design. The look-elsewhere and pooled
   alternatives also pass, but buy nothing here; the pooled jj statistic gains
   at most 1.5× sensitivity, because MadGraph's 10k sample is the binding size.
2. **Flip both defaults in `MapOptions::resolve`**: `tau` to MadEvent's rule
   (inverse-square where no whole-state resonance), and `split_angle` to
   `soft-all`, with the doc comment's numbers re-measured.
   - The τ rule reaches every hadronic row without a whole-state resonance, so
     V1 re-runs the banked layer (`pixi run --skip-deps validate`) under both
     flips.
   - A cell that leaves its limit becomes `info` with a Found entry; no limit
     moves.
   - The flip lands **before** `sigma-calibration-rerecord` (#45) re-records
     the hadronic calibrations, since it moves them.
3. **File the jj composition excess against MadGraph** as its own validation
   item. Its first step needs no banking: compare this side's per-directory σ
   with MadGraph's per-channel integrated σ already in each
   `P1_*/G*/run_01_log.txt`. A second banked MadGraph jj sample (a true
   two-run null) is requested only if that does not settle it.
4. **File V0's Found 1**: a generation seed reused across grids gives
   correlated samples (flavour χ²/dof 0.43 against 0.95), and llj seeds
   correlate across maps at ρ = 0.45. Seed-for-seed arm comparisons, note 37's
   "same-map seed controls" among them, are paired rather than independent and
   want an audit.

## What it cannot see

- `pp_to_llj_fixed` under soft-all is unmeasured; V1 measures it in the
  banked run.
- The τ grid-against-grid SPINUP null reads 1.25 against 1.10 under log (5
  dof, correlated readings), unconfirmed. V1 records it beside the flip.
