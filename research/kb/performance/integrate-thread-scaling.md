---
type: Measurement
title: integrate thread scaling
description: "Serial/parallel fit of integrate at -j 16 (Amdahl, not stalled threads), the serial floor's parts, and the digest-identical speedup measurements."
status: draft
tags: [performance, parallelism, amdahl, integrate, determinism]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n31-j, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1195-L1241", title: "Note 31 §6.7 (the -j column)"}
  - {id: n32-amdahl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L77-L113", title: "Note 32 §1.1 (why -j 16 yields only 4.7–5.4×: Amdahl)"}
  - {id: n32-outcomes, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1 (S3: survey parallelised)"}
  - {id: n32-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L638-L748", title: "Note 32 §5.3 (close-out -j 16 measurement)"}
  - {id: n32-followups, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L749-L793", title: "Note 32 §5.4 (standing follow-ups)"}
  - {id: proton-survey, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/proton.rs#L2832-L2900", title: "proton.rs adapt_alphas and survey_variance"}
  - {id: cli-parallel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/parallel.rs#L20-L75", title: "vibegraph-cli parallel.rs (-j)"}
measured:
  - {commit: b0e08d3, host: "Apple M3 Max (load average 6–19 during the sweep)", command: "vibegraph integrate <cards> --fixed-budget --neval 120000 --niter 12 -j {1,16}, min of 5 rounds"}
  - {host: "Apple M3 Max (quiet, load 3.5–3.9)", command: "vibegraph integrate <cards> --neval 120000 --niter 12 -j {1,16}, min of 4 rounds"}
---

# `integrate` thread scaling

`vibegraph integrate -j N` sizes the rayon pool; results do not depend on it.
How the integrator stays bit-identical across thread counts (per-chunk
substreams addressed by point index, partials reduced in chunk order) is in
[RNG substreams and parallel determinism](../phase-space/rng-substreams-and-parallel-determinism.md).
This concept records how far it scales and why.[^cli-parallel]

## The method: a two-term fit

Fit `T₁ = S + P` and `T₁₆ = S + P/16` to the `-j 1` and `-j 16` walls of one
card and budget. `S` is the serial floor, `P/S` sets the `-j → ∞` ceiling
`T₁ / S`, and `S / T₁₆` is how much of a parallel run is spent waiting on serial
work. When the parallel term scales cleanly and the speedup is still poor, the
cause is Amdahl, not stalled or contending threads.[^n32-amdahl]

A fitted ceiling is a `-j → ∞` limit, not a `-j 16` prediction. On the M3 Max
`-j 16` also maps onto 12 performance and 4 efficiency cores, so even perfectly
parallel work cannot reach 16×.

## Measurements

All on the M3 Max, CLI-driven, `--neval 120000 --niter 12` fixed budget (since
the CLI default became `--target-rel`, `--fixed-budget` must be passed), minimum
over rounds. **Every run of a card wrote the same artifact digest at every
thread count**: `-j 16` is the same number computed faster, checked at the CLI
rather than inferred from a unit test.[^n31-j][^n32-closeout]

| card | `-j 1` | `-j 16` | speedup | when |
|---|--:|--:|--:|---|
| `dy13_default` | 2.0446 s | 0.2357 s | **8.68×** | survey parallel, load 6–19 |
| `pp_to_llj` | 11.0434 s | 1.1645 s | **9.48×** | survey parallel, load 6–19 |
| `dy13_default` | 2.077 s | 0.442 s | 4.70× | survey serial, quiet host |
| `pp_to_llj` | 12.088 s | 2.256 s | 5.36× | survey serial, quiet host |

The last two rows are kept because the comparison is the point: they are the
same cards before the α-adaptation survey was parallelised. An independent
reading after the change, also on a loaded host, gave dy13 0.225 s / 9.16× and
llj 1.443 s / 8.78×; the two post-change readings agree within the noise of a
host neither reading could quiet, and the dy13/llj ordering flips between them,
which is that noise, not an effect (llj's ~11 s `-j 1` run averages over more
contention than dy13's ~2 s).[^n32-closeout] The post-change speedups sit well
below the fitted ceilings, as expected of a `-j 16` reading on a loaded hybrid
CPU.

## What the serial floor is made of

Applied to the survey-serial rows, the fit put ~70–75% of a `-j 16` run in the
serial floor (dy13 S = 0.33 s, llj S = 1.60 s) while the parallel region scaled
cleanly. The parts, in measured size:[^n32-amdahl][^n32-outcomes]

- **The α-adaptation survey.** `survey_variance` visits every channel's density
  per point, O(n_survey × n_channels), and `adapt_alphas` runs several surveys
  back to back, budget-independently. Re-measured on a fixed-budget `-j 16` wall
  it was 41–52% of the run (llj 1.04 of 2.51 s, dy13 0.24 of 0.46 s). It now runs
  its point loop in one rayon region over the deterministic chunking, asserted
  bit-identical at `-j {1, 4, 16}`.[^proton-survey] This was the largest term,
  and the speedup roughly doubled when it went.
- **One-time setup**: model parse and intern, diagram enumeration, evaluator
  compile, PDF grid parse, artifact serialisation. Most of dy13's floor; its
  whole run is under half a second.
- **Barriers in the adapt phase.** Each VEGAS/α iteration is a barrier, and
  per-channel grids have a 512-point chunk floor, so small channels parallelise
  poorly within an iteration. Fewer, larger iterations at fixed total budget
  shorten the sequential critical path; that was measured but adopted nowhere,
  and any batch shape needs a ≥5-seed sweep before use.
- **Seed-level adaptation in multi-seed gates.** The σ gates in
  `validate_hadronic.rs` call `adapt_alphas` once per seed inside their own seed
  loop, so a multi-seed gate adapts its seeds one after another; only the point
  loop inside one call is parallel.[^n32-followups]

Further parallel axes (helicity strata, flavour-group × beam-ordering strata, a
shorter adapt phase before a frozen parallel pass) are open as
[stratified-parallel-axes-unbuilt](../backlog/performance/stratified-parallel-axes-unbuilt.md).

## Reading these numbers

- `-j` walls are wall-time numbers and move no CPU-denominated ratio against
  MadGraph; those keep their CPU-time construction
  ([integration against MadGraph](../performance/integration-vs-madgraph.md)).
- Time the run from outside the interpreter that launches it: a `-j 16` dy13 run
  is under half a second, and two interpreter start-ups inside the timed region
  are a tens-of-per-cent error.[^n31-j]
- Thread scaling needs more than 4 cores; the Cascade Lake and Emerald Rapids VMs
  cannot measure it ([benchmark hosts](../performance/benchmark-hosts.md)).
- The validation layer's per-row times are taken single-threaded by protocol
  ([validation-layer timings](../performance/validation-layer-timings.md)).

[^n31-j]: Note 31 §6.7.
[^n32-amdahl]: Note 32 §1.1, the two-term fit and the floor's decomposition (its ~27% survey share and its `proton.rs:1920` sequential-loop reference describe the tree before the survey was parallelised).
[^n32-outcomes]: Note 32 §5.1, S3 outcome and the 41–52% re-measurement.
[^n32-closeout]: Note 32 §5.3, close-out `-j 16` table.
[^n32-followups]: Note 32 §5.4, seed-level `adapt_alphas` and the batch-shape candidate.
[^proton-survey]: `ProtonIntegrand::adapt_alphas` and `survey_variance` (`into_par_iter`) in `proton.rs`.
[^cli-parallel]: `-j` / `--parallel` in `vibegraph-cli/src/parallel.rs`.
