---
type: Measurement
title: "Integration against MadGraph: throughput and time to accuracy"
description: "Time to 0.1% on σ and points per CPU-second against MadEvent over 7 rows. x86 v0.2.1 (Ice Lake VM): 0.52× MadEvent's CPU at equal accuracy (0.47× at v3); M3 Max v0.2.0: 0.30×. The MLM row is the one where we are slower."
status: draft
tags: [performance, madgraph, integration, throughput, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n30-throughput, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/30-perf-baseline-timings.md#L352-L407", title: "Note 30 §5.3 (throughput on a denominator that means something)"}
  - {id: n31-throughput, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L1036-L1103", title: "Note 31 §6.4 (throughput recomputed)"}
  - {id: n32-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L865-L955", title: "Note 32 §7 (time to a target accuracy, protocol)"}
  - {id: n34-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L338-L415", title: "Note 34 §3 (time to accuracy, remeasured post-sprint)"}
  - {id: cl-summary, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L11-L37", title: "Cascade Lake comparison: headline"}
  - {id: cl-tta, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L100-L156", title: "Cascade Lake comparison §3 (time to 0.1% on σ)"}
  - {id: cl-throughput, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L157-L214", title: "Cascade Lake comparison §4 (integrand throughput)"}
  - {id: cl-uncovered, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/mg-comparison-cascade-lake-results.md#L215-L226", title: "Cascade Lake comparison §5 (not covered)"}
  - {id: integrate-cli, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/integrate.rs#L285-L335", title: "integrate CLI budget and convergence flags"}
measured:
  - {commit: e4a90128, tag: v0.2.1, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "taskset -c 2 <v0.2.1 release asset> integrate <row>.proc --run-card <row>.run --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1 --no-network --no-tui; -j 4 unpinned; MadEvent reused from the v0.2.0 runs below (byte-identical copy)", data: "CERN host: /home/ncsmith/bench-v0.2.0/results-v0.2.1/ (SUMMARY.md, redelta.md, phase2a, phase2b, jscale, TABLES_EXTRA.md; tools-v021/redelta.py)"}
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Apple M3 Max, macOS 15.8.2, interactive desktop (load 2.3–3.5)", command: "<v0.2.0 aarch64-apple-darwin release asset> integrate … --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1 (no affinity on macOS); -j 12; MadEvent time_stages.py at nb_core = 12; --log-file reruns of ud_to_epemud_qcd0 and pp_to_ll_0j2j_mlm for the iteration scatter", data: "M3 host: /Users/ncsmith/bench-v020/results/phase2, phase2b"}
  - {commit: 3cc7f9c2, tag: v0.2.0, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "taskset -c 2 <release asset> integrate <row>.proc --run-card <row>.run --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1 --no-network --no-tui, timed with os.wait4; MadEvent via time_stages.py --nb-core 4", data: "CERN host: /home/ncsmith/bench-v0.2.0/results/{phase2a,phase2b}/ (collated by tools/collate2a_fullprec.py, collate2a_addenda_fullprec.py, collate2b.py)"}
  - {host: "Apple M3 Max", command: "RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 per-row validate_sigma / validate_hadronic"}
  - {commit: 098c9e2, host: "Apple M3 Max", command: "vibegraph integrate <proc card> --run-card <run card> --target-rel 0.001 --seed {20260719,20260720,20260721} -j 1"}
  - {commit: cf8b2b7, pr: 8, landed_in: 6e15ad3, host: "Intel Xeon @ 2.80 GHz, Cascade Lake (family 6 model 85), 4-vCPU Firecracker VM", command: "taskset -c 2 vibegraph integrate … --target-rel 0.001 -j 1; per-row gates under RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1"}
---

# Integration against MadGraph: throughput and time to accuracy

Two end-to-end comparisons with MadEvent, each taken with both sides on one host
in one sitting. **Integrand throughput** prices a point. **Time to a target
accuracy** is what a user waits for: it also sees whether the points were worth
drawing. The per-point matrix-element comparison (`MATRIX1`) is a different
measurement, in [matrix element against MadGraph](../performance/matrix-element-vs-madgraph.md);
the protocol rules are in
[the end-to-end timing protocol](../performance/end-to-end-timing-protocol.md).

## Current: x86 at v0.2.1 and the M3 Max at v0.2.0

**Headline: over seven rows, vibegraph reaches the same accuracy in 0.52× MadEvent's CPU
on the x86 host at v0.2.1 (0.47× with the x86-64-v3 asset), and in 0.30× on the M3 Max
at v0.2.0 (geomeans).** Without the MLM row it is 0.41× / 0.37× and 0.23×. The MLM row
is the one where vibegraph is slower: 2.1× on x86, 1.6× on the M3
([MLM integration cost against MadEvent](mlm-integration-cost-vs-madevent.md)). Every σ
agrees with MadEvent's (largest |pull| 1.47). Integrand throughput is 3.9× MadEvent's
points per CPU-second on x86 (4.4× at v3) and 6.7× on the M3.

The M3 Max needs no v0.2.1 run: the musl allocator fix, the only performance change
between the releases, does not touch the macOS build. The evaluator is unchanged
between them ([matrix element against MadGraph](matrix-element-vs-madgraph.md)).

Setup common to both hosts:
- **Ours.** The release assets, `--target-rel 0.001`, seeds 20260719/20/21, timed with an
  `os.wait4` wrapper. Cards: `import model sm` plus MadGraph's default multiparticle
  labels plus the row's `generate` line, and each row's banked run card.
- **δ** is the consistency-scaled error the run stops on
  ([protocol §5](end-to-end-timing-protocol.md)), and MadEvent's `<cumulated_time>` is
  scaled by (δ_MG/δ)² to it.
- **The draws are identical** between v0.2.0 and v0.2.1 seed for seed (iterations,
  evaluations, σ): only the time changed.

Host differences:
- **x86.** The CERN Ice Lake VM ([benchmark hosts](benchmark-hosts.md)), `-j 1` pinned to
  one core. MadEvent was not rerun for v0.2.1: the ratios reuse the v0.2.0 MadEvent runs
  (nb_core = 4, `global_flag = -O`), a byte-identical copy from the same host four days
  older. Phases that ran both sides in one sitting are the other measurements.
- **M3 Max.** The user's interactive desktop, load average 2.3–3.5 during the single-core
  runs, with no core affinity on macOS. MadEvent ran there at nb_core = 12.

Time to accuracy, ours/MadEvent CPU at equal δ (mean over 3 seeds; below 1 is faster):

| row | x86 v0.2.1 plain | x86 v0.2.1 v3 | M3 v0.2.0 | x86 v0.2.0 plain (for reference) |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 0.12 (0.11–0.15) | 0.11 | 0.058 | 0.12 |
| `dy13_default` | 0.16 (0.15–0.18) | 0.15 | 0.078 | 0.16 |
| `gg_to_gg` | 0.76 (0.72–0.80) | 0.66 | 0.48 | 0.88 |
| `pp_to_jj` | 0.67 (0.64–0.69) | 0.60 | 0.42 | 0.89 |
| `pp_to_llj` | 0.88 (0.86–0.90) | 0.80 | 0.62 | 1.10 |
| `ud_to_epemud_qcd0` (2→4) | 0.56 (0.53–0.59) | 0.47 | 0.26 | 0.55 |
| `pp_to_ll_0j2j_mlm` (MLM) | 2.09 (2.00–2.21) | 2.12 | 1.56 | 2.68 |
| **geomean** | **0.52** | **0.47** | **0.30** | 0.59 |

- **What the v0.2.1 musl fix changed on x86:** the rows with real multichannel work gained
  15–25% (`gg_to_gg`, `pp_to_jj`, `pp_to_llj`, the MLM row, whose 22–40 s of single-thread
  system time is gone). The cheapest rows and `ud_to_epemud_qcd0` did not move.
- **The hosts differ by 1.7–2.1× on every row**, in the M3's favour. Most of it is
  MadEvent's side: its `<cumulated_time>` is largely per-job work that changes little
  between hosts (see "Why the ratios halve between hosts" below).

**Is δ honest?** Two checks per row, both passing:
- **Seed spread.** The sd of σ over the three seeds is 0.06–1.04 of the mean quoted error
  on x86 and 0.20–1.04 on the M3.
- **Iteration scatter.** On x86, reconstructed from the cumulative lines, it is 0.55–1.41.
  On the M3 it was read from `--log-file` traces for the two rows whose printed χ²/dof is
  high: `ud_to_epemud_qcd0` 1.04 and the MLM row 1.04, against printed 1.43 and 1.57. The
  MLM row's parts read 1.13 / 0.84 / 1.25 (@0/@1/@2).

**Equal accuracy is not time to 0.1% on the cheap rows.** `ee_to_mumu` and `gg_to_gg` stop
at the default budget's 6-iteration floor, already past the target. With a smaller,
non-default budget (`--min-iters 0 --neval 30000`) they reach 0.1% at:

| row | x86 v0.2.1 plain / v3 | M3 v0.2.0 |
|---|--:|--:|
| `ee_to_mumu` | 0.19 / 0.17 | 0.089–0.099 |
| `gg_to_gg` | 0.81 / 0.70 | 0.48–0.50 |

`--min-iters 0` alone, at the default `--neval`, still overshoots `ee_to_mumu` and puts
`gg_to_gg` at 1.06 / 0.93 on x86 (0.76–1.22 by seed): the default iteration size, not the
minimum count, is what overshoots.

Integrand throughput, points per CPU-second (ours: evaluations over single-core CPU;
MadEvent: `results.dat` points over `<cumulated_time>`; x86 mean over seeds, M3 seed
20260719):

| row | x86 plain | x86 v3 | M3 |
|---|--:|--:|--:|
| `ee_to_mumu` | 28× | 31× | 57× |
| `dy13_default` | 13× | 15× | 27× |
| `gg_to_gg` | 1.9× | 2.2× | 3.2× |
| `pp_to_jj` | 1.5× | 1.7× | 2.4× |
| `pp_to_llj` | 3.4× | 3.8× | 4.9× |
| `ud_to_epemud_qcd0` | 4.4× | 5.2× | 9.4× |
| `pp_to_ll_0j2j_mlm` | 0.88× | 0.92× | 1.19× |
| **geomean** | **3.9×** | **4.4×** | **6.7×** |

Wall clock at equal cores (seed 20260719), with each side at its own defaults: ours at
about 0.1% against MadEvent's integration wall at its own δ (0.04–0.33%), so not equal
accuracy.

| row | x86: ours `-j 4` plain / v3 | MadEvent nb_core 4, integrate / total | M3: ours `-j 12` | MadEvent nb_core 12, integrate / total |
|---|--:|--:|--:|--:|
| `ee_to_mumu` | 0.47 / 0.43 s | 4.5 / 8.8 s | 0.15 s | 5.7 / 12.2 s |
| `dy13_default` | 1.54 / 1.40 s | 27.2 / 71.8 s | 0.26 s | 35.3 / 64.3 s |
| `gg_to_gg` | 3.78 / 3.30 s | 3.3 / 8.4 s | 1.29 s | 4.4 / 9.4 s |
| `pp_to_jj` | 10.7 / 9.5 s | 28.7 / 44.9 s | 3.14 s | 24.5 / 38.6 s |
| `pp_to_llj` | 40.8 / 36.8 s | 18.9 / 36.2 s | 11.4 s | 13.7 / 28.2 s |
| `ud_to_epemud_qcd0` | 17.5 / 15.1 s | 5.1 / 12.3 s | 2.75 s | 5.1 / 11.2 s |
| `pp_to_ll_0j2j_mlm` | 217.6 / 212.2 s | 78.3 / 90.1 s | 55.9 s | 49.9 / 59.5 s |

On x86 the v0.2.1 release assets scale cleanly: their `-j 4` CPU matches `-j 1`, and
`pp_to_jj` at a fixed budget speeds up 3.7× from `-j 1` to `-j 4` on every build
([musl allocator thread scaling](musl-allocator-thread-scaling.md)).

## Per-point cost on the 2→6 rows (M3 Max)

The matrix element is not the whole integrand: the phase-space map, the cuts, the scale
draw and the multichannel density sum `Σⱼ αⱼgⱼ(p)` are in every point too. On the two
2→6 rows, at 579 and 615 channels:
- **a point that survives the cuts** costs 138 µs and 207 µs, of which the density sum
  is 36 and 38 µs (26% and 18%);
- **most points never get that far:** acceptance is 3.6% and 4.3%, and the density runs
  only after the cut and the matrix element, so the average drawn point costs 5.8 µs and
  9.0 µs.

Both were an order of magnitude higher before the density moved behind the cut and began
sharing subtree momenta across channels
([the channel contract](../phase-space/channel-contract.md)).
`probe_2to6_eval_cost` and `probe_2to6_density_decomposition` in
`vibegraph-lib/tests/validate_sigma.rs` re-measure them.

## x86 v0.2.0, Ice Lake VM (`3cc7f9c2`, 2026-11-09/10)

Kept as the record behind the v0.2.0 MLM diagnosis and the δ correction; the x86 figures
above supersede it.

**Headline: over seven rows, vibegraph reaches the same accuracy in 0.59× MadEvent's CPU
(geomean; 0.56× with the x86-64-v3 asset), or 0.46× / 0.43× without the MLM row, which is
about 2.7× slower.** Integrand throughput is 3.5× MadEvent's points per CPU-second (3.7× at
v3). Every σ agrees with MadEvent's (largest |pull| 1.46).

Setup ([benchmark hosts](benchmark-hosts.md)):
- **Ours.** Both musl release assets (plain and `-v3`), `--target-rel 0.001 -j 1` pinned to
  one core, seeds 20260719/20/21, binaries interleaved, timed with an `os.wait4` wrapper.
  Cards: `import model sm` plus the row's `generate` line; run cards are the banked ones.
- **MadEvent.** One `time_stages.py` launch per row at `nb_core = 4`, `global_flag = -O`
  set explicitly. CPU is `<cumulated_time>`, scaled by (δ_MG/δ)² to the δ each of our
  seeds reached.
- **δ is the consistency-scaled error the run prints and stops on**
  ([protocol §5](end-to-end-timing-protocol.md)). It looks honest on every row: the
  iteration-scatter χ²/dof is 0.55–1.41, and the seed sd of σ is 0.06–1.04 of the mean
  quoted error. The first collation also widened δ by √(printed χ²/dof). That statistic
  is a mean of per-channel χ²/dof and is inflated by heavy tails. It changed nothing beyond
  noise where it is near 1, but on `ud_to_epemud_qcd0` (1.40) and `pp_to_ll_0j2j_mlm`
  (1.5–1.8) it overstated δ. The "§5 χ²" column keeps those first readings.
- **The musl assets anti-scale with threads**, so every multi-core figure below is a glibc
  `cargo build --release` of the same commit, labelled as such
  ([musl allocator thread scaling](musl-allocator-thread-scaling.md); fixed after v0.2.0). The
  musl assets also carry 15–20% single-thread overhead against glibc, which every
  single-core ratio here includes.

Time to accuracy, ours/MadEvent CPU at equal δ (mean over 3 seeds, min–max in brackets;
below 1 is faster):

| row | plain | v3 | §5 χ² reading, plain / v3 | printed / scatter χ²/dof | pull range |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 0.12 (0.11–0.15) | 0.12 (0.10–0.14) | 0.13 / 0.12 | 0.94 / 1.41 | within 1.46 |
| `dy13_default` | 0.16 (0.15–0.18) | 0.15 (0.14–0.17) | 0.16 / 0.15 | 0.85 / 0.85 | |
| `gg_to_gg` | 0.88 (0.84–0.92) | 0.81 (0.78–0.86) | 0.82 / 0.76 | 0.68 / 0.55 | |
| `pp_to_jj` | 0.89 (0.85–0.92) | 0.85 (0.82–0.88) | 0.89 / 0.84 | 1.04 / 1.08 | |
| `pp_to_llj` | 1.10 (1.07–1.12) | 1.04 (1.01–1.05) | 1.08 / 1.02 | 1.09 / 1.02 | |
| `ud_to_epemud_qcd0` (2→4) | 0.55 (0.52–0.58) | 0.47 (0.44–0.49) | 0.73 / 0.62 | 1.40 / 0.88 | +0.35 to +0.46 |
| `pp_to_ll_0j2j_mlm` (MLM) | 2.68 (2.55–2.83) | 2.70 (2.56–2.83) | 4.03 / 3.94 | 1.61 / 1.05 | −0.68 to −1.22 |

The glibc build at `-j 4` (seed 20260719) reads 0.55 on `ud_to_epemud_qcd0` and 2.41 on
the MLM row; the iteration-scatter χ²/dof of its MLM run, from a full trace, is 0.82.

**Equal accuracy is not time to 0.1% on the cheap rows.** `ee_to_mumu` and `gg_to_gg` stop
at the default budget's 6-iteration floor, already past the target, so the ratio compares
equal accuracy at the δ actually reached. Measured time to 0.1% needs a smaller budget
(row means, plain / v3):

| row | `--min-iters 0` (default `--neval`) | `--min-iters 0 --neval 30000` (non-default) |
|---|--:|--:|
| `ee_to_mumu` | 0.22 / 0.21 (still overshoots) | 0.18 / 0.17 |
| `gg_to_gg` | 1.23 / 1.14 (0.94–1.42 by seed) | 0.94 / 0.88 |

So the default budget's floor, not the integrator, is what would put `gg_to_gg` above 1 on
time to 0.1%.

**The MLM row** is diagnosed in
[MLM integration cost against MadEvent](mlm-integration-cost-vs-madevent.md). The 2.4× on
the glibc build is almost all the two-jet part's cost per point: 485 µs against
MadEvent's 104 µs. Both sides cluster about as often per point (28 against ~33), but each
of our clusterings costs about 4× MadEvent's, and we evaluate every group's matrix element
twice (56 evaluations) where MadEvent evaluates the one term it selects. The single-threaded system time on this row (22–40 s) is the musl
allocator; the glibc build shows 0.28 s. The plain and v3 assets take different iteration
paths from one seed on this row (inferred: last-ulp differences flipping a decision at a
threshold).

Integrand throughput, points per CPU-second (ours: evaluations over single-core CPU, mean
over seeds; MadEvent: `results.dat` points over `<cumulated_time>`, whose survey-pass
question is open, below):

| row | ours plain | ours v3 | MadEvent | ours/MG plain | v3 |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 739 000 | 787 000 | 26 714 | 27.7× | 29.5× |
| `dy13_default` | 353 000 | 383 000 | 26 597 | 13.3× | 14.4× |
| `gg_to_gg` | 74 600 | 80 400 | 44 358 | 1.7× | 1.8× |
| `pp_to_jj` | 21 100 | 22 100 | 18 786 | 1.1× | 1.2× |
| `pp_to_llj` | 32 100 | 34 100 | 11 654 | 2.8× | 2.9× |
| `ud_to_epemud_qcd0` | 74 200 | 87 600 | 16 414 | 4.5× | 5.3× |
| `pp_to_ll_0j2j_mlm` | 5 670 | 5 920 | 8 244 | 0.69× | 0.72× |

The two MLM and 2→4 throughput entries are seed 20260719 only.

Wall clock at equal cores (4), glibc build at `-j 4` against MadEvent at `nb_core = 4`,
seed 20260719:

| row | ours wall | MadEvent integrate / total wall |
|---|--:|--:|
| `ee_to_mumu` | 0.43 s | 4.47 / 8.84 s |
| `dy13_default` | 2.18 s | 27.2 / 71.8 s |
| `gg_to_gg` | 3.92 s | 3.32 / 8.45 s |
| `pp_to_jj` | 11.15 s | 28.7 / 44.9 s |
| `pp_to_llj` | 43.24 s | 18.9 / 36.2 s |
| `ud_to_epemud_qcd0` | 16.5 s | 5.1 / 12.3 s |
| `pp_to_ll_0j2j_mlm` | 229.9 s | 78.3 / 90.1 s |

MadEvent's wall is at its own δ (0.04–0.33%), ours at about 0.1%, so these are not equal
accuracy; read them as what a user waits for with each code's defaults on four cores.

## Earlier hosts: Cascade Lake VM and M3 Max

Headline, geomeans:[^cl-summary]

| | Cascade Lake VM | M3 Max |
|---|--:|--:|
| per point, ours/MG `MATRIX1` cost, 19 processes | 0.87× | 0.87× |
| time to 0.1% on σ, MG/ours, 5 rows | **1.67×** | **3.84×** |
| integrand throughput, ours/MG, 26 rows | **3.97×** | **8.76×** |

The per-point ratio carries across hosts unchanged; both end-to-end ratios
halve. The evaluators are not the cause (below). Every figure here predates the
evaluator's constant collection and fused scaled sums (2026-10-05), which sped
the matrix element up by 1.01–1.55× in-process; none has been re-measured since.

### Integrand throughput

**Construction.** Our points are `seeds × neval × niter` VEGAS evaluations from
each row's report record, over its single-thread `duration_s` (per-row protocol,
`RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1`). MadGraph's are field 4 of
`SubProcesses/results.dat` (MadEvent's own count of points behind the banked
result) over that file's `<cumulated_time>`, the summed CPU seconds of its
Fortran jobs, which takes its job farm out of the comparison. `pp_to_ll` divides
by the `dy13_default` run.[^n30-throughput][^n31-throughput][^cl-throughput]

| row | M3 Max ours/MG | Cascade Lake ours/MG |
|---|--:|--:|
| `ee_to_mumu` | 37.3× | 15.7× |
| `ee_to_ee` | 24.1× | 9.7× |
| `ee_to_ttx` | 25.3× | 8.9× |
| `ee_to_wpwm` | 13.8× | 4.7× |
| `ee_to_zh` | 34.6× | 13.2× |
| `uux_to_mumu` | 29.9× | 10.8× |
| `uux_to_uux` | 3.9× | 1.9× |
| `gg_to_ttx` | 3.7× | 1.9× |
| `gg_to_gg` | 1.9× | 1.1× |
| `ee_to_mumua` | 10.7× | 5.2× |
| `ee_to_tatah` | 19.9× | 7.3× |
| `uux_to_epemg` | 2.5× | 1.7× |
| `ddx_to_epemg` | 2.7× | 1.6× |
| `gu_to_epemu` | 5.1× | 2.7× |
| `gux_to_epemux` | 5.3× | 2.7× |
| `ee_to_mumu_tata_qcd0` | 13.7× | 7.0× |
| `ud_to_epemud_qcd0` | 5.8× | 3.1× |
| `pp_to_ll` | 23.3× | 9.3× |
| `pp_to_bb` | 4.0× | 2.0× |
| `pp_to_bb_qcd2` | 3.5× | 1.5× |
| `pp_to_bb_fixed` | 12.5× | 4.1× |
| `pp_to_jj` | 4.6× | 1.7× |
| `pp_to_ll_scalefact2` | 12.9× | 6.1× |
| `pp_to_llj_fixed` | 9.1× | 3.8× |
| `pp_to_llj` | 8.2× | 4.2× |
| `pp_to_llj_dyn` | 6.8× | 3.3× |
| **geomean** | **8.76×** | **3.97×** |

The shape is the same on both hosts: the cheapest leptonic rows lead, and the
pure-gluon and colour-dense 2→2 rows sit at the floor. `gg_to_gg` is the floor
everywhere: no PDF work to win back and the densest colour algebra in the census.
Throughput is per point, so a budget change is invisible in it by construction.

### Time to 0.1% on σ

**Protocol.** Ours: `vibegraph integrate <proc card> --run-card <run card>
--target-rel 0.001 --seed {20260719,20260720,20260721} -j 1`, wall clock over the
whole process (model load, enumeration and evaluator compilation inside).
MadGraph: the same run's `<cumulated_time>` and σ ± err, scaled by 1/δ² to the
χ²-scaled δ each of our seeds reached. On the M3 Max the MadGraph side is the
banked run; on Cascade Lake it is that host's own run of the same
process.[^n32-tta][^cl-tta]

| row | channels | M3 Max: ours s | MG s @δ | MG/ours | Cascade Lake: ours s | MG s @δ | MG/ours |
|---|--:|--:|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 2 | 0.39 | 6.9 | **17.6×** | 1.10 | 7.9 | **7.2×** |
| `dy13_default` | 4 | 3.09 | 26.0 | **8.4×** | 9.43 | 33.1 | **3.5×** |
| `gg_to_gg` | 4 | 3.95 | 8.7 | **2.2×** | 12.07 | 11.2 | **0.93×** |
| `pp_to_jj` | 19 | 18.83 | 48.7 | **2.6×** | 53.40 | 60.6 | **1.13×** |
| `pp_to_llj` | 24 | 122.54 | 121.2 | **0.99×** | 362.26 | 178.3 | **0.49×** |
| **geomean** | | | | **3.84×** | | | **1.67×** |

M3 Max at `098c9e2`; `pp_to_llj` needs 140–156 iterations (16.8–18.7M
evaluations) and converges at the CLI's default `--max-iters 500`; its σ
(505.54–505.81 pb) sits +0.18–0.23% above the banked 504.63 ± 1.67, consistent
with MadGraph's own last-three combination reading 0.146% below its iterations
recombined by point count.[^n34-tta][^integrate-cli] On Cascade Lake the draws
are close to the M3 Max's (`pp_to_llj` 142/160/134 iterations, 17.44M against
17.64M evaluations; default x86-64 has no FMA, so `mul_add_fast` rounds twice
and the stop lands differently).[^cl-tta]

The ratio decomposes, and the decomposition closes arithmetically: time ratio ≈
throughput ratio ÷ points ratio. On `pp_to_llj` (M3 Max) 9.0× MadGraph's points
at ~8–9× its throughput gives ~1; on Cascade Lake 4.2× throughput over 8.9× the
points gives 0.47× against 0.49× measured. `pp_to_llj`'s remaining deficit is map
quality (9× the points for the same accuracy) times its χ²/dof ≈ 1.38, both
pointing at the cut-boundary concentration at the map's lower edge.[^n34-tta]
Seed spread follows the iteration count: 2% on `ee_to_mumu` and `gg_to_gg`, up
to 17% on `pp_to_llj` and 26% on `dy13_default`. How `--target-rel` decides to
stop is in [the convergence stop rule](../phase-space/convergence-stop-rule.md).

### Why the ratios halve between hosts

MadGraph's `MATRIX1` is 3.02× slower on Cascade Lake than on the M3 Max,
uniformly (2.75–3.33× per process), and ours slows by the same factor
(`pp_to_llj` 20.8 µs per integrand point against 6.9 µs). What does not slow 3×
is the rest of MadEvent's summed job CPU, which is mostly not matrix element: it
is only 1.1–1.6× above the M3 bank's, so MadEvent's total CPU-seconds rose only
1.10× (`ee_to_mumu`) to 1.47× (`pp_to_llj`). On `gg_to_gg`,
`MATRIX1` is 1.4 of 10.2 CPU-s on Cascade Lake and 0.44 of 7.1 CPU-s on the M3
Max; the remainder (process start-up, grid I/O, phase space, PDFs) grew only
1.3×.[^cl-summary][^cl-tta]

Two readings fit, and one sitting cannot tell them apart: Cascade Lake is simply
less than 3× slower on that kind of work; or the M3 Max bank's `cumulated_time`
was inflated, because madevent ran 16 concurrent jobs over 12 performance and 4
efficiency cores, and a job on an efficiency core, or waiting for a performance
core, adds CPU-seconds without adding work. Either way, every ratio built on
`<cumulated_time>` carries a host-dependent factor of order one, and only a
same-host ratio is reported. Open as
[madevent-cumulated-time-host-dependence](../backlog/performance/madevent-cumulated-time-host-dependence.md).

## Caveats on every MadGraph column

- Whether `results.dat`'s point count includes the survey pass as well as the
  refine passes was never established, so a systematic factor of order one sits
  on every MadGraph throughput entry; it cannot depend on process complexity, so
  the trend across rows survives it.[^n30-throughput] (Open in
  [timing-baseline-unmeasured-stages](../backlog/performance/timing-baseline-unmeasured-stages.md).)
- Our per-point work is not MadGraph's per-point work: ours carries the
  multichannel map, the VEGAS grid, the cuts and the live configuration draw;
  MadGraph's carries its own analogues. Throughput is integrand throughput, not a
  matrix-element ratio.
- Parallel scaling (`-j 16`) and unweighting efficiencies are M3 Max figures
  only; the 4-core VMs cannot measure the first, and the second is a ratio of
  trial counts independent of host. The 2→6 integrand costs and the banked-layer
  wall were not re-measured on Cascade Lake.[^cl-uncovered]
- Same-host only: the M3 Max and Cascade Lake columns are never divided into
  each other except to locate the denominator effect above.

[^n30-throughput]: Note 30 §5.3, the construction and its two caveats.
[^n31-throughput]: Note 31 §6.4, M3 Max throughput recomputed (the "now" column used here).
[^n32-tta]: Note 32 §7, time-to-accuracy protocol.
[^n34-tta]: Note 34 §3, M3 Max time to accuracy at `098c9e2`, including `pp_to_llj`'s convergence and the iteration-cap change.
[^cl-summary]: Cascade Lake comparison, headline and the `MATRIX1` slowdown.
[^cl-tta]: Cascade Lake comparison §3.
[^cl-throughput]: Cascade Lake comparison §4.
[^cl-uncovered]: Cascade Lake comparison §5.
[^integrate-cli]: `--max-iters` (default 500) and `--target-rel` in `vibegraph-cli/src/integrate.rs`.
