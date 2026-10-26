---
type: Session Report
title: "F1 report: the 2 → 6 budget ladder is parameterised and runs at 20 seeds from one command"
description: "probe_2to6_budget_ladder reads rows, samplers, rungs, seeds, niter and a TSV path from VG_2TO6_* variables, prints pulls against the row's seed record and the single run, and a per-rung and adjacent-rung summary. Toy run shown."
status: draft
tags: [budget, ladder, two-to-six, probe, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-17}
sources:
  - {id: brief, resource: "F1.md", title: "F1 brief"}
  - {id: code, resource: "../../../../../vibegraph-lib/tests/validate_sigma.rs", title: "probe_2to6_budget_ladder"}
---
## Parameters

All environment variables; unset, the probe is today's (both rows, both
samplers, rungs 100k..800k flat and 300k/600k/1.2M multi, seeds `SEED,11,22,33,44`,
8 iterations). Documented in the probe's doc comment.

| variable | meaning |
|---|---|
| `VG_2TO6_ROWS` | comma list; full names or `uux` / `bbx` |
| `VG_2TO6_SAMPLERS` | `flat`, `multi` |
| `VG_2TO6_RUNGS` | `a,b,c` for every selected sampler, or `flat=a,b;multi=c,d` |
| `VG_2TO6_SEEDS` | one integer N: `SEED` then `11·k`, k = 1..N−1 (N = 5 is today's seeds); a comma list is verbatim |
| `VG_2TO6_NITER` | iterations (default 8) |
| `VG_2TO6_OUT` | TSV, appended, header if empty, one flushed line per seed: `row sampler neval niter seed sigma_pb err_pb wall_s` |

Output on stderr: a line per seed; per rung the unweighted mean, `max(quoted,
spread/√n)` with both parts, χ²/dof, pull against the row's refdata-10 seed
record (via `common::seed_record::reference`, as the gate reads it) and against
the `sigma_reference.json` single run; per adjacent rung pair, Δmean over
`√(s₁²/n₁ + s₂²/n₂)` (computed as `√(spread₁² + spread₂²)`, since `spread` is
sd/√n) labelled information.

## The command (20 seeds, 600k and 1.2M multichannel, both rows)

```
cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma --no-run
VG_2TO6_SAMPLERS=multi VG_2TO6_RUNGS=600000,1200000 VG_2TO6_SEEDS=20 \
  VG_2TO6_OUT=$PWD/ladder-2to6.tsv \
  nohup setsid cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_sigma -- --ignored --nocapture --exact probe_2to6_budget_ladder > ladder-2to6.log 2>&1 &
```

Outputs: `ladder-2to6.tsv` (per seed) and `ladder-2to6.log` (summaries).
Time, scaled from P1 (883 s a 600k×8 uux seed, ~1250–2150 s for 12 iterations
of bbx, loaded 4-core host): 20 seeds × (600k + 1.2M) × two rows is roughly 30 h
on that host; the M3 figure is not measured here. The Rung-1.2M cost of 2× the
600k one is an assumption (cost is linear in neval), not a measurement.

## Gate and toy run

- `cargo fmt --all --check`: clean (printed `FMT_OK`).
- `cargo clippy -p vibegraph-lib --features extended-validation --test validate_sigma -- -D warnings`: finished with no warnings (3 m).
- Release-debug test binary built (8 m).

Toy run (uux, multi, 300000, 2 seeds, 2 iterations), 197 s on a host at load ~10:

```
VG_2TO6_ROWS=uux VG_2TO6_SAMPLERS=multi VG_2TO6_RUNGS=300000 VG_2TO6_SEEDS=2 VG_2TO6_NITER=2 VG_2TO6_OUT=…/twosix-F1-toy.tsv \
  target/release-debug/deps/validate_sigma-6395ffa45aa89f81 --ignored --nocapture --exact probe_2to6_budget_ladder
```
```
[uux_to_ccx_emmm_qcd0] MadEvent reference from seeded_sigma/uux_to_ccx_emmm_qcd0/seed_record.json: 5 reported seeds at nevents 10000, 7.177100e-7 +- 1.084e-9 pb (quoted 1.084e-9, spread 9.035e-10, chi2/dof 0.74)
    multichannel 300000 x 2 seed 20260719: σ 7.158487e-7 ± 5.690e-9 pb | rel vs single run +2.20e-3 | 113 s
    multichannel 300000 x 2 seed         11: σ 7.194288e-7 ± 1.060e-8 pb | rel vs single run +7.21e-3 |  84 s
  multichannel 300000 x 2 (2 seeds, 197 s): σ = 7.176388e-7 ± 6.016e-9 pb (quoted 6.016e-9, spread/√n 1.790e-9) | χ²/dof 0.13 | pull vs seed record -0.01 | pull vs single run +0.53 | rel vs single run +4.70e-3
row	sampler	neval	niter	seed	sigma_pb	err_pb	wall_s
uux_to_ccx_emmm_qcd0	multi	300000	2	20260719	7.158486982e-7	5.689640e-9	113.2
uux_to_ccx_emmm_qcd0	multi	300000	2	11	7.194288253e-7	1.060134e-8	83.5
```

The toy has two seeds at two iterations: the numbers show the plumbing, not
agreement. The adjacent-rung line needs two rungs, so it was exercised
separately with flat RAMBO (uux, rungs `flat=2000,4000`, seeds 11,22,33, 1
iteration, 18 s): `flat RAMBO 2000 -> 4000: Δmean -2.072e-8 pb (-9.68e-2 rel)
over √(s₁²/n₁ + s₂²/n₂) = 3.564e-8 pb: -0.58σ (information)`; the same run's
pulls (−10 to −14) show the flat map missing the cross section, as expected.
The file-restart path (append to an existing TSV) was not exercised.

## Brief corrections

- None substantive. "SEED plus the next N − 1 from a documented scheme": the
  scheme chosen is 11·k, which reproduces today's five for N = 5; a single
  integer is therefore always a count, so a lone explicit seed cannot be given
  (one seed is not a ladder anyway; the probe needs ≥ 2).
- The seed record's reference is 5 seeds `reported` at nevents 10000 (the
  record is the refdata-10 one), not the 20-seed confirmation the ladder's
  D1 reading ultimately wants; the pull printed is against that 5-seed record.

## Found

1. **No per-rung D1 verdict against a 20-seed MadEvent record.** The ladder
   prints the pull against the 5-seed record; the D1 third-amendment reading
   needs the 20-seed MadEvent confirmation (the banking request owned by the
   sprint). Close by re-running the summary lines once that record is banked.
2. **The probe's reported `chi2_per_dof` is the `combine` seed χ², but
   `integrate_with` discards the run's own χ²/dof** (P1 Found 1 stands): nothing
   here reads it, so no change, noted so that it is not added without the pooled
   statistic.
