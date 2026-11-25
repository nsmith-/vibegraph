---
type: Backlog Item
title: "vibegraph's quoted σ error disagrees with its seed scatter on multi-jet rows, in both directions"
description: "Seed χ²/dof reads 4.02 on HT 2→4 and 2.79 on fixed-energy 3 jets (error under-quoted), and 0.12 on p p > j j j (over-quoted ~3×); none was measured with the 20 seeds the policy asks."
area: validation
state: open
priority: medium
closes_when: "Each of the three rows has a 20-seed measurement at two budget rungs, the cause of any persistent mismatch is found (a mis-estimated error, or a seed that misses a region), and the error combination is fixed or the row's tolerance taken from the measured spread."
blocked_by: []
opened: 2026-11-07
tags: [sigma, seeds, error-estimate, vegas, multi-jet]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hg1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G1-report.md", title: "H-G1 report, Found 9"}
  - {id: hk1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-K1-report.md", title: "H-K1 report, Found: quoted error 3× the seed scatter on p p > j j j"}
  - {id: hq1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-Q1-report.md", title: "H-Q1 report, Found 5"}
---
Three sprint H sessions read seed χ²/dof far from 1, each on 3–5 seeds:

| row | flags | seed χ²/dof | reading |
|---|---|---|---|
| HT-binned 2→4 `u u > g g u u` + conjugate (H-G1) | `--target-rel 0.003` | 4.02 (5 seeds); `u u` alone 4.84 (3); 0.11 at `0.0015` (3) | quoted error ~2× low |
| fixed √ŝ = 500 GeV, 3 jets (H-Q1) | `--target-rel 0.0005` | 2.79 (5); hadronic runs 0.6–1.0 | quoted error low |
| `p p > j j j` (H-K1) | `--target-rel 0.003` | 0.12 (5): quoted 0.27%, scatter 0.096% | quoted error ~3× high |

An under-quoted error makes a σ gate pass on a confidently wrong value; an
over-quoted one hides real offsets from the pull (H-K1). Few-seed scatter has
understated heavy-tailed rows by 2–5× before, so the readings need the 20-seed
calibration the seed policy asks for before any cause is chosen. H-K1 suggests
looking at how per-iteration errors combine when many channels sit at the point
floor. The statistic itself is discussed in
[chi2-per-own-sigma-misreads-heavy-tails](chi2-per-own-sigma-misreads-heavy-tails.md).

**More cases (2026-11-17).** The CERN host's 1009-gridpack survey, stage 2 (5
seeds), found seed-scatter χ²/dof of 9.3 on ChargedHiggsGM VBF M2000
(gridpack 72) and 5.1 on QCD-4Jets HT-10to40 (gridpack 34).
TpTpTo2T2STo2Mu2G_MS10 (gridpack 508) hits the 500-iteration cap on some
seeds.

## A 20-seed calibration on a cancelling interference (614, 2026-11-25)

Sprint L measured VLQ card 614 (TprimeBToTH, `VLQ^2==2`, a signed interference
across a 20%-width pole) on main e6a4cba8, 20 seeds per rung:

| rung | mean (pb) | seed sd | mean quoted error | χ²/dof |
|---|---|---|---|---|
| 4× (neval 480000) | −2.984e-4 | 5.7% | 8.4% | 0.51 |
| 8× (neval 960000) | −2.997e-4 | 5.0% | 5.7% | 0.80 |

The rungs agree (−0.25σ). The quoted error overstates the scatter at 4×. The
seed spread shrinks far less than the 1/√2 that point count alone would give,
which points to grid adaptation rather than per-iteration statistics dominating
the scatter. A 5-seed read of the same 4× rung gave a 3.2% spread, 1.8× too
small, and a spurious "dip".
