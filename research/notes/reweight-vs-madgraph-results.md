# Reweighting cost: vibegraph against MadGraph's reweight module — results

**Status: measurement record, 2026-10-05.** Tree: `claude/fervent-hopper-ytjddf` at
`f6b1936` (PR #13). One sitting on one host, both sides timed by
`validation/madgraph/bench_reweight.py` (`pixi run -e madgraph bench-reweight`) on
the four rows of `gen_reweight_oracle.py`, whose weights agree with MadGraph's event
by event to the files' printed precision. This is a quick comparison, not a
calibrated one: read it as an order of magnitude.

| | |
|---|---|
| host | 4-vCPU cloud VM, Intel Xeon @ 2.80 GHz |
| vibegraph | `--profile release-debug` (debug info off), `generate` from each row's banked grid |
| MadGraph | 3.7.1, the pinned `research/refs/mg5amcnlo` checkout, Python 3.11, gfortran |

**Headline.** Per event and hypothesis, MadGraph's reweight loop costs 720–800 µs on
every row; vibegraph's exact path costs 5–14 µs, **50–150× less**. The polynomial
path widens that to 100–320× on the rows that use it. Counting MadGraph's 6–10 s of
setup per run, reweighting a banked sample takes MadGraph 12–18 s and vibegraph
10–160 ms.

## 1. What is measured

- **vibegraph:** the wall time of `generate` with the row's reweight card minus the
  same run without it — same grid, seed and event count — median of 5 runs,
  interleaved. It includes building the plan (compiling every hypothesis's
  amplitude, choosing nodes) and the per-event audit. Measured at the row's own
  event count and at 20,000 events; at the row's own count the difference is within
  the generation's own noise (§3), so the per-event figures below are the
  20,000-event ones.
- **MadGraph:** its reweight module on the row's own events (vibegraph's, with
  MadGraph's banner added) with `change helicity False`, one hypothesis, under
  cProfile. *Setup* is `create_standalone_directory` + `compile` (process
  generation, the standalone output, the Fortran build); *loop* is
  `launch_actual_reweighting`, the pass over the events it makes once per launch.
  All hypotheses are estimated as setup + H × loop: a multi-launch card through
  `ReweightInterface.import_command_file` writes one mislabelled weight in this
  version (3.7.1), so it cannot be timed directly.

## 2. Results

Per event, all hypotheses of the row (µs):

| row | events | H | vibegraph exact | vibegraph polynomial (K) | MadGraph H × loop | ratio, exact | ratio, polynomial |
|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_tth_ymt` | 1000 | 4 | 20.5 | 9.0 (2) | 2880 | 140× | 320× |
| `ee_ttx_smeft` | 1000 | 11 | 157 | 56 (4) | 8070 | 51× | 144× |
| `pp_llj` | 2000 | 3 | 14–24 | — | 2150 | 90–150× | — |
| `tata_tth_grid` | 1000 | 9 | 109 | 73 (6) | 7220 | 66× | 99× |

The banked sample, end to end:

| row | vibegraph reweighting | MadGraph setup | MadGraph loop per hypothesis | MadGraph, all H (est.) |
|---|--:|--:|--:|--:|
| `ee_tth_ymt` | 14–18 ms | 8.0 s | 0.72 s | 10.8 s |
| `ee_ttx_smeft` | 85 ms polynomial, 163 ms exact | 9.8 s | 0.73 s | 17.9 s |
| `pp_llj` | 30–50 ms | 7.4 s | 1.43 s | 11.7 s |
| `tata_tth_grid` | 54 ms polynomial, 109 ms exact | 6.4 s | 0.80 s | 13.6 s |

The polynomial column is `--reweight-couplings` with the row's couplings. Without
the flag, `ee_ttx_smeft` and `tata_tth_grid` run exact (automatic grouping joins
only launches that move one parameter) and `ee_tth_ymt` takes the same K = 2 plan.

## 3. Reading it

- **MadGraph's cost is almost all overhead.** Its loop makes two Fortran
  matrix-element calls per event (`calculate_matrix_element`, the original and the
  new point), and `calculate_weight` is about 70% of the loop; the rest is reading
  and rewriting the event file in Python. That is why its per-event cost is flat at
  720–800 µs across processes whose amplitudes differ by 20×.
- **The two loops do not do the same work.** vibegraph reweights momenta it holds
  inside `generate`; it reads and writes no file for it. Part of the gap is that, and
  reweighting a stored `.lhe` (not implemented) would pay a parse.
- **The polynomial path gains less than its evaluation count.** 12 → 4 evaluations
  on `ee_ttx_smeft` buys 2.8×, 10 → 6 on `tata_tth_grid` 1.5×: each node is still a
  full amplitude evaluation, and the K×K Gram contraction per helicity combination
  adds a fixed cost.
- **Noise.** At the banked sample size the reweighting is a few percent of
  `generate`, so differences of identical plans scatter by tens of µs per event
  (`pp_llj`'s exact and default plans are the same and read 5 and 25 µs at 2000
  events, 24 and 14 at 20,000). The `pp_llj` row is quoted as the range.
- **What is not compared.** MadGraph's setup is a one-off per process and would
  amortise over larger samples; vibegraph's plan-building cost is inside its
  figures. Neither side was pinned to a core count: `generate` runs a second thread
  beside its main loop (its CPU time exceeds its wall time), while MadGraph's
  reweight loop is single-threaded.
