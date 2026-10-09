---
type: Measurement
title: "Reweighting cost against MadGraph's reweight module"
description: "Per event and hypothesis: MadGraph 720–800 µs (mostly Python I/O), vibegraph exact 5–14 µs, the polynomial path 100–320× cheaper; an order-of-magnitude comparison only."
status: draft
tags: [performance, reweighting, madgraph-comparison, benchmarks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  commit: f6b1936
  pr: 13
  host: "4-vCPU cloud VM, Intel Xeon @ 2.80 GHz"
  command: "pixi run -e madgraph bench-reweight (validation/madgraph/bench_reweight.py)"
sources:
  - {id: rw, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/reweight-vs-madgraph-results.md#L11-L97", title: "Reweighting cost: vibegraph against MadGraph's reweight module — results"}
---

# Reweighting cost against MadGraph's reweight module

Both sides timed in one sitting on one host by `validation/madgraph/bench_reweight.py`
(`pixi run -e madgraph bench-reweight`), on the four rows of `gen_reweight_oracle.py`, whose
weights agree with MadGraph's event by event to the files' printed precision. It is a quick
comparison, not a calibrated one: **read it as an order of magnitude.**[^rw]

**Headline.** Per event and hypothesis MadGraph's reweight loop costs 720–800 µs on every
row; vibegraph's exact path costs 5–14 µs, 50–150× less, and the polynomial path widens that
to 100–320× on the rows that use it. Counting MadGraph's 6–10 s of setup per run,
reweighting a banked sample takes MadGraph 12–18 s and vibegraph 10–160 ms.

## What is measured

- **vibegraph** (`--profile release-debug`, debug info off): wall time of `generate` with the
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

## Results

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
  pay a parse, is not implemented.
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
