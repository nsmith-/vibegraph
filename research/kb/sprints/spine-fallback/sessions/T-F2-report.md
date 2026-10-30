---
type: Session Report
title: "T-F2 report: the γγ → ℓℓ corner on DY"
description: "In progress. The campaign plan, written before the long runs."
status: draft
tags: [phase-space, spine, vbf, cms, overweight, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
---
The session brief is [T-F2](T-F2.md).

## Campaign plan (written before launch)

**Harness.** A weight-ranked dump, committed on `sprinte-spine-fallback`
(`7263d4c8`) and merged into `/home/user/wt/TV2`. `generate` keeps its eight
heaviest overweight trials and, at debug level, splits each one's weight into
the grid's Jacobian per coordinate, the channel's α, the integrand `f`, the
mixture density `g` and the channels carrying it, and the partonic momenta.
It draws nothing extra, so a seed's sample is the same as before. The
before and after measurements both use this harness:

```
RUST_LOG=info,vibegraph::generate=debug vibegraph --no-tui -y generate <R1 grid> \
  --seed 7 --nevents 600 -o <tag>.lhe --force
```

Driver `T-F2-gen.sh`: one summary line per run in `T-F2-results-gen.txt`,
resumable by tag. Integrations use T-V2's `T-V2-run.sh` command line
(`--fixed-budget --neval 10000 --niter <K>`, `-j 4`).

**Costs** (measured by T-V2 on this host, idle): DY R1 integrate ≈ 400 s,
DY R4 ≈ 1150 s, generate 600 events ≈ 24 min on one thread. P-V2 shares the
cores for the first hour or two, so the first phases run slower.

| phase | runs | estimate |
|---|---|---|
| A. before: generate on T-V2's R1 grids, seeds 1–4, 4 jobs of 1 thread | 4 | 0.5–0.8 h |
| B. hermetic diagnosis and the fix, bias checks on `/home/user/wt/T` | – | 2 h, mostly dev |
| C. after: integrate R1 seeds 1–4, then generate as in A | 4 + 4 | 1 h |
| D. paired R1 → R4 climb: R4 seeds 1–12 (R1 read off iteration 3) | 12 | 4 h |
| E. gates, banked re-gates of changed rows | – | 1.5 h |
| total | | ≈ 9–9.5 h |

- R4's first three iterations are R1's bit for bit (T-V2), so phase D gives
  both rungs per seed from one run.
- The "before" climb is T-V2's seeds 1–12, same command and budget.
- If phase D runs late, it stops at 12 seeds and τ (brief §5) is not run.
