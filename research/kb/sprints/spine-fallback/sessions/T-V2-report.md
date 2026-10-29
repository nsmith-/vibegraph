---
type: Session Report
title: "T-V2 report: budget study and CMS VBF/EWK ladder (in progress)"
description: "Campaign plan with measured per-run wall times, written before the long runs were launched."
status: draft
tags: [phase-space, spine, vbf, cms, budget-ladder, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
The session brief is [T-V2](T-V2.md). This checkpoint holds the campaign plan
only; the results follow when the campaign ends.

## Campaign plan (written before launch)

**Build.** `/home/user/wt/TV2` at `3b0d9363` (#81 `a09a7806` merged with #82
`da56ab77`), `cargo build --profile release-debug -p vibegraph`. Model: a
scratchpad copy of `research/refs/mg5amcnlo/models/loop_sm` (submodule at
`b7687064`) plus the gridpacks' `restrict_ckm_no_b_mass.dat` (identical in all
three). The three `param_card.dat` are byte-identical. PDF 325300 was already
in the `~/.vibegraph` cache.

**Command per run** (driver `T-V2-run.sh`, one result line per finished run,
skips recorded tags):

```
vibegraph --no-tui -v -y integrate <card>/proc_card.dat --run-card <card>/run_card.dat \
  --param-card <card>/param_card.dat --ufo-dir T-V2-ufo --out <dir> --force \
  --seed <S> -j 4 --fixed-budget --neval <N> --niter <K>
```

**Measured pilots** (seed 1, `--neval 10000 --niter 4`, 4 threads, nothing
else running):

| card | channels (massless + τ part) | wall | σ (pb) | MG σ (pb) |
|---|---|---|---|---|
| DYto2L-2Jets-EWK_Bin-MLL-50 | 68 + 74 | 527 s | 12.03 ± 0.56 | 13.12 ± 0.031 |
| VBFto2L_Bin-MLL-50 | 68 + 68 | 533 s | 8.50 ± 0.75 | 7.574 ± 0.030 |
| VBFto2L_Bin-MLL-4to50 | 68 + 68 | 461 s | 104.7 ± 7.0 | 117.6 ± 0.44 |

Where the time goes (DY): 9 s enumeration and compilation (766 subprocesses);
130 s for the two α-surveys (6 × 10⁴ points each, the survey's minimum); then
about 99 s per VEGAS iteration. One iteration is about 8–9 × 10⁴ points,
because 139 of the 142 channels sit at the 512-point floor whatever `--neval`
is below that. The cost is about 1.1 ms per point at 4 threads, the same as
the CERN host measured.

**What this allows.** About 1.1 ms a point puts 10 hours at about 3 × 10⁷
points, all runs together. The brief's design (20 + 20 seeds on DY, then 5
seeds × 2+ rungs × 3 cards) needs about 21 hours at the cheapest well-formed
rungs. So the plan keeps the brief's priority order and drops the second
VBF rung:

- **Rungs.** `R1` = `--neval 10000 --niter 3`: the 512-point floor, two
  warm-up iterations, one measured. `R4` = `--niter 12`: 4× R1's VEGAS points
  (12 iterations against 3), ten measured. `--neval` stays at the floor
  because raising it only re-weights the few channels above the floor, and
  it raises the survey to 4 × 10⁴ points a part.
- **Estimated wall per run** (from the pilot, one iteration less or eight
  more): DY R1 ≈ 430 s, DY R4 ≈ 1320 s, VBF MLL-50 R1 ≈ 415 s, VBF MLL-4to50
  R1 ≈ 335 s.

| phase | runs | estimate |
|---|---|---|
| 1. budget study, DY R1, seeds 1–20 | 20 | 2.4 h |
| 2. ladder R1, VBF MLL-50 and VBF MLL-4to50, seeds 1–5 | 10 | 1.0 h |
| 3. budget study, DY R4, seeds 1–20 | 20 | 7.3 h |
| total | 50 | 10.7 h |

- **Not planned:** R4 on the two VBF cards (10 runs, about 3.6 h), and any
  rung above R4. DY's ladder is phases 1 and 3 (its first five seeds a rung,
  and all twenty).
- **Order.** Phase 2 goes before phase 3 so that every card has a reading
  early. The budget study is still complete before anything else is added.
- **Shared CPU.** The σ-level test is calibrated and gated on
  `/home/user/wt/T` while the campaign runs. That slows some runs; the
  recorded wall per run says by how much. It does not change any σ: a fixed
  budget at a fixed seed gives the same artifact at any thread count.
