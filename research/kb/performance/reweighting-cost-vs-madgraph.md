---
type: Measurement
title: "Reweighting cost against MadGraph's reweight module"
description: "On a 45-hypothesis EFT operator basis at v0.2.0, MadGraph's real run is 151–335× slower than vibegraph's; per event and hypothesis MadGraph pays ~500 µs, mostly Python I/O."
status: draft
tags: [performance, reweighting, madgraph-comparison, benchmarks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "VIBEGRAPH_BIN=<release asset> pixi run -e madgraph python validation/madgraph/bench_reweight.py --work <oracle work> --out <dir>; basis8.py for the operator-basis row", data: "CERN host: /home/ncsmith/bench-v0.2.0/results/phase3/"}
  - {commit: f6b1936, pr: 13, host: "4-vCPU cloud VM, Intel Xeon @ 2.80 GHz", command: "pixi run -e madgraph bench-reweight (validation/madgraph/bench_reweight.py)"}
sources:
  - {id: rw, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/reweight-vs-madgraph-results.md#L11-L97", title: "Reweighting cost: vibegraph against MadGraph's reweight module — results"}
---

# Reweighting cost against MadGraph's reweight module

Both sides timed in one sitting on one host by `validation/madgraph/bench_reweight.py`
(`pixi run -e madgraph bench-reweight`), on the four rows of `gen_reweight_oracle.py`, whose
weights agree with MadGraph's event by event to the files' printed precision. It is a quick
comparison, not a calibrated one: **read it as an order of magnitude.**[^rw]

**Headline (v0.2.0).** On an EFT operator basis (8 SMEFTsim operators at NP=1, 45
hypotheses, 1000 events), MadGraph's real 45-launch run takes 71.0 s against vibegraph's
0.40–0.47 s exact and 0.21–0.29 s on the polynomial path: **151–335× slower**, or 134–298×
without MadGraph's one-off setup. MadGraph's loop costs about 490–550 µs per event and
hypothesis on every row; vibegraph's exact path covers all of a row's hypotheses in
15–120 µs per event.

The f6b1936 measurement below is kept for history: per event and hypothesis MadGraph cost
720–800 µs and vibegraph's exact path 5–14 µs, with MadGraph's multi-hypothesis cost
estimated as setup + H × one loop. The v0.2.0 basis row shows that estimate understates
MadGraph's real cost by 2.4×.

## v0.2.0, Ice Lake VM (`3cc7f9c2`, 2026-11-10)

Both release assets (musl, plain and x86-64-v3) through `bench_reweight.py` with
`VIBEGRAPH_BIN` set; logging shims confirmed each pass called only its own binary. The work
areas come from `gen_reweight_oracle.py --check`, whose check passed: all 27 hypotheses on
the four rows agree with MadGraph per event to ≤ 1e-7. Both sides at `change helicity False`.
MadGraph's side ran once (it does not depend on our binary). `generate` ran unpinned at its
default thread count, so the musl assets' thread contention
([musl allocator thread scaling](musl-allocator-thread-scaling.md); fixed after v0.2.0) may
inflate our figures; it was not separated.

Existing rows, cost per event (generate with the card minus without, median of 5; µs per
event at the row's sample size). Ours is over all H hypotheses together; MadGraph's is per
hypothesis:

| row | H | ours exact, plain / v3 | ours polynomial, plain / v3 | MG loop per hypothesis | MG setup + H × loop |
|---|--:|--:|--:|--:|--:|
| `ee_tth_ymt` (1000 events) | 4 | 22.5 / 18.6 | 15.8 / 18.1 | 536 | 4.3 + 4 × 0.54 = 6.4 s |
| `ee_ttx_smeft` (1000) | 11 | 120.9 / 89.3 | 71.0 / 54.9 | 492 | 7.8 + 11 × 0.49 = 13.2 s |
| `tata_tth_grid` (1000) | 9 | 64.9 / 61.2 | 53.2 / 42.5 | 538 | 4.5 + 9 × 0.54 = 9.3 s |
| `pp_llj` (2000) | 3 | not resolved | — | 550 | 5.7 + 3 × 1.10 = 9.0 s |

`pp_llj`'s difference is inside the generation's own noise (the v3 pass read negative).

**Operator basis.** `e+ e- > t t~ NP<=1` in SMEFTsim `topU3l` (massless restriction), with
eight operators: `ctWRe`, `ctBRe`, `cHt`, `cHQ1`, `cHQ3`, `cHWB` (at 0.5, as in
`ee_ttx_smeft`), `cQe`, `cte`. The 45 hypotheses are the SM, each operator at +1 and at −1,
and the 28 pairs at (+1, +1). The polynomial plan reads all 45 off K = 9 amplitude
evaluations per event. The card-point |M|² check against the integrand deviated by 0 over
1000 events.

| side | 1000 events | 20 000 events |
|---|--:|--:|
| ours exact, plain / v3 | 0.471 / 0.404 s | 7.44 / 6.51 s |
| ours polynomial, plain / v3 | 0.285 / 0.212 s | 3.49 / 3.24 s |
| MadGraph, real 45-launch card | 71.0 s (setup 7.8 s, 45 loops 21.9 s, ~41 s other per-launch overhead) | not run |
| MadGraph, setup + 45 × one loop (estimate) | 29.4 s | |

MadGraph's 45-launch output declares all 45 weights but carries one per event, the
multi-launch defect in [MadGraph defects](../validation/madgraph-defects.md): the timing is
of real work, but a correct MadGraph basis needs 45 single-launch runs, each paying setup
(about 7 minutes, inferred, not run).

## f6b1936 (PR #13), 4-vCPU cloud VM

### What is measured

- **vibegraph** (`--profile release-debug`, the build `bench-reweight` makes; the record says
  debug info was off for the run, though the profile itself sets `debug = 1`): wall time of `generate` with the
  row's reweight card minus the same run without it (same grid, seed and event count),
  median of 5 interleaved runs. It includes building the plan (compiling every hypothesis's
  amplitude, choosing nodes) and the per-event audit. At the row's own event count the
  difference is inside the generation's own noise, so per-event figures are from 20 000
  events.
- **MadGraph** 3.7.1 (the pinned checkout, Python 3.11, gfortran): its reweight module on
  the row's own events (vibegraph's, with MadGraph's banner added), `change helicity False`,
  one hypothesis, under cProfile. *Setup* is `create_standalone_directory` + `compile`;
  *loop* is `launch_actual_reweighting`, one pass over the events. All hypotheses are
  estimated as setup + H × loop, because a multi-launch card through
  `ReweightInterface.import_command_file` writes one mislabelled weight in 3.7.1 and cannot
  be timed directly (see [MadGraph defects](../validation/madgraph-defects.md)).

### Results

Per event, all hypotheses of the row (µs):

| row | events | H | vibegraph exact | vibegraph polynomial (K) | MadGraph H × loop | ratio, exact | ratio, polynomial |
|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_tth_ymt` | 1000 | 4 | 20.5 | 9.0 (2) | 2880 | 140× | 320× |
| `ee_ttx_smeft` | 1000 | 11 | 157 | 56 (4) | 8070 | 51× | 144× |
| `pp_llj` | 2000 | 3 | 14–24 | — | 2150 | 90–150× | — |
| `tata_tth_grid` | 1000 | 9 | 109 | 73 (6) | 7220 | 66× | 99× |

The banked sample end to end:

| row | vibegraph reweighting | MadGraph setup | MadGraph loop per hypothesis | MadGraph, all H (est.) |
|---|--:|--:|--:|--:|
| `ee_tth_ymt` | 14–18 ms | 8.0 s | 0.72 s | 10.8 s |
| `ee_ttx_smeft` | 85 ms polynomial, 163 ms exact | 9.8 s | 0.73 s | 17.9 s |
| `pp_llj` | 30–50 ms | 7.4 s | 1.43 s | 11.7 s |
| `tata_tth_grid` | 54 ms polynomial, 109 ms exact | 6.4 s | 0.80 s | 13.6 s |

The polynomial column is `--reweight-couplings` with the row's couplings. Without the flag,
`ee_ttx_smeft` and `tata_tth_grid` run exact (automatic grouping joins only launches that
move one parameter) and `ee_tth_ymt` takes the same K = 2 plan.

## Reading it

- **MadGraph's cost is almost all overhead.** Its loop makes two Fortran matrix-element
  calls per event (`calculate_matrix_element`, original and new point); `calculate_weight`
  is about 70% of the loop and the rest is reading and rewriting the event file in Python.
  That is why its per-event cost is flat at 720–800 µs across processes whose amplitudes
  differ by 20×.
- **The two loops do not do the same work.** vibegraph reweights momenta it holds inside
  `generate` and reads and writes no file for it. Reweighting a stored `.lhe`, which would
  pay a parse, is not implemented
  ([backlog](../backlog/feature/reweight-stored-lhe-unsupported.md)).
- **The polynomial path gains less than its evaluation count.** 12 → 4 evaluations on
  `ee_ttx_smeft` buy 2.8×, 10 → 6 on `tata_tth_grid` 1.5×: each node is still a full
  amplitude evaluation, and the K×K Gram contraction per helicity combination adds a fixed
  cost. A graded evaluator that computes coupling-free subtrees once is the open lever
  ([backlog](../backlog/performance/reweight-k-evaluations-per-event.md)).
- **Noise.** At the banked sample size reweighting is a few percent of `generate`, so
  identical plans scatter by tens of µs per event (`pp_llj`'s exact and default plans are the
  same and read 5 and 25 µs at 2000 events, 24 and 14 at 20 000); that row is quoted as a
  range.
- **Not compared.** MadGraph's setup is one-off per process and would amortise over larger
  samples; vibegraph's plan building is inside its figures. Neither side was pinned to a
  core count: `generate` runs a second thread beside its main loop (CPU time exceeds wall),
  while MadGraph's reweight loop is single-threaded.

[^rw]: `reweight-vs-madgraph-results.md`, at `f6b1936` (PR #13).
