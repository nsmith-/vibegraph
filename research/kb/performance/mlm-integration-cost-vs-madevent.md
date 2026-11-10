---
type: Measurement
title: "MLM integration cost against MadEvent: where pp_to_ll_0j2j_mlm's gap goes"
description: "At v0.2.0, pp_to_ll_0j2j_mlm takes 2.4× MadEvent's CPU to equal accuracy (glibc build); the two-jet part's per-point cost (485 µs against 104 µs) explains it, from 64 flavour groups each clustered and evaluated twice per point."
status: draft
tags: [performance, madgraph-comparison, mlm, integration, profiling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
measured:
  - {commit: 2c80e447, host: "Intel Xeon (Ice Lake), 4-vCPU KVM guest, AlmaLinux 10.2", command: "glibc cargo build --release (and release-debug for profiling); integrate pp_to_ll_0j2j_mlm --target-rel 0.001 --seed 20260719 -j 4 --log-file trace.log; fixed-budget A/B --allocate neyman --neval 120000 --niter 20, seeds 20260719/20/21; eu-stack sampling at about 5 Hz, -j 1; a per-part timer around MultiplicitySum::value_in_channel in an uncommitted scratch build", data: "CERN host: /home/ncsmith/bench-v0.2.0/results/mlm_diag/ (main_glibc_j4_target, mg_parts.txt, prof_j1, ab, redelta.md, instrumentation.diff); tools in tools/mlm_diag/"}
---

# MLM integration cost against MadEvent

`pp_to_ll_0j2j_mlm` (`p p > e+ e-` with 0–2 jets, MLM-matched) is the one row of the v0.2.0
[integration comparison](integration-vs-madgraph.md) where vibegraph is clearly slower than
MadEvent. The diagnosis below runs at `main` 2c80e447, whose MLM, VEGAS and allocation
code is unchanged since v0.2.0. A glibc build there reproduces the v0.2.0 run bit for bit
(seed 20260719: 40 iterations, 5 834 881 evaluations, σ 1064.5 pb). A rerun of MadEvent
reproduces its banked run exactly (1067.2 ± 2.43 pb, 577 113 points, `<cumulated_time>`
70.0 s).

## The headline, split

The first published figure was about 4.0× (musl assets, δ widened by the printed χ²/dof).
It splits into three factors:

| factor | size | kind |
|---|--:|---|
| δ widened by the printed χ²/dof | ×1.4–1.5 | measurement definition ([protocol §5](end-to-end-timing-protocol.md)) |
| musl allocator | ×1.15 | release build ([backlog](../backlog/performance/musl-release-assets-anti-scale-with-threads.md)) |
| real gap, glibc build, consistency-scaled δ | **×2.40** | performance |

- **The printed χ²/dof is the wrong statistic for δ.** It reads 1.49–1.82 on this row. The
  iteration scatter of σ is 0.83–1.05 (0.82 from the glibc run's full trace), and six seeds
  span 1064.0–1065.4 pb against a quoted ±1.0 pb.
- **The musl share is allocator time.** The musl assets spend 22–40 s of system time
  single-threaded on this row; the glibc build spends 0.28 s.

## The 2.40×, by merged part

Per part: S is the per-point sd (error × √n), c the CPU per point. The per-part timer was
read in the VEGAS phase at `-j 1`. MadEvent's figures are summed over its G directories.

| part | ours S | MadEvent S | ours c | MadEvent c | ours share of points | MadEvent share |
|---|--:|--:|--:|--:|--:|--:|
| @0 | 805 | 512 | 12.8 µs | 50.8 µs | 59% | 17% |
| @1 | 557 | 409 | 50.8 µs | 68.8 µs | 15% | 11% |
| @2 | 673 | 508 | 485 µs | 104 µs | 26% | 72% |

The model is cost × variance = (Σ n·c)(Σ S²/n), with optimum (Σ S√c)². It reproduces the
measured 2.40 as a product:

| factor | ratio | notes |
|---|--:|---|
| intrinsic S and c, each side at its own optimum | ×3.14 | all from @2: S√c is 14 825 against 5 183; with ours equal to MadEvent's there, the ratio is 0.97 |
| estimator | ×1.26 | the target mode's unweighted mean over iterations (deliberate: inverse-variance biases low under heavy tails) plus consistency scaling |
| allocation | ×0.70 | ours is 1.46 above its optimum, MadEvent 2.07 |
| survey and setup | ×0.86 | 1.15 against 1.33 |

Within @2, its cost per point (485 against 104 µs) is ×2.5 of the intrinsic 3.14, and its
variance (S 673 against 508) the remaining ×1.26. @0 has the worse S (805 against 512)
but costs a quarter as much per point, so it nets out. MadEvent's `<cumulated_time>`
(70.0 s) exceeds the sum of its G directories (52.6 s). The difference is presumably
survey jobs whose refines were superseded (inferred). The comparison keeps it, as it
should for time to result.

## Where a two-jet point's time goes

An eu-stack sampling profile (release-debug, `-j 1`, fixed budget; 933 VEGAS-phase
samples, about 89% of CPU in @2), inclusive shares:

| component | share |
|---|--:|
| `setclscales` / `cluster_history` (of which `kt::cluster` 25.6%, `walk_tree` BTreeMap lookups 10.7%) | 49.8% |
| matrix element: `eval_amp2` 17.6% + `eval_m2` 14.4% | 32% |
| `member_rewgt` | 8% |
| PDFs and αs | ~5% |
| phase-space map, channel densities, cuts | < 2% |

About 18% of samples are allocator leaves, mostly inside the clustering.

**Structure (from the code).** `per_group_sum` handles every flavour group of the part at
every point, 64 groups at @2. For each group it:
- draws a configuration ∝ AMP2, which is a full matrix-element pass (`eval_amp2`);
- clusters (`setclscales`, `kt::cluster`);
- computes the member reweights;
- moves αs and evaluates the matrix element again at the same momenta (`eval_m2`).

`walk_tree` rebuilds a 2ⁿ-mask PDG table (64 entries at @2) from `MergeTable.ipdgcl` per
group per point, though it depends only on the channel and graph. MadEvent also has 64 @2
matrix-element files (40 in `P2_qq_llqq` alone), but clusters once per point, in its own
P-directory's configuration.

## Levers, estimated

Estimates from the profile shares or code reading; none is implemented. Each has a backlog
item.

| lever | effect | gain on ours/MadEvent |
|---|---|--:|
| share matrix element and clustering across flavour copies (u≡c, d≡s: 64 @2 groups → 28 classes; 12 → 6 at @1) | @2 cost ×0.54 | ~1.6× |
| cache `walk_tree`'s PDG table per (group, configuration) | @2 ×0.89 | ~1.1× |
| one matrix-element pass per group, rescaling by the αs ratio | @2 ×0.86 | ~1.13× |
| allocation-free clustering scratch | @2 ×0.9 | ~1.08× |
| cost-aware part allocation, n ∝ S/√c (the optimum gives @2 about 10% of points, not 24–26%) | allocation excess 1.40–1.46 → 1 | ≤ 1.4×, ~1.2× if @2's true sd is 1.4× its quoted |

The first four together put @2's cost at about ×0.4 and the ratio near 1.2 (inferred; the
overlaps are not checked). The cost-aware split runs against the tail argument of
[mlm-at2-tail-unpriced-in-allocation](../backlog/performance/mlm-at2-tail-unpriced-in-allocation.md):
both effects are real, and the net is unmeasured.

## A/B on map and grid options

Fixed budget, `--allocate neyman --neval 120000 --niter 20`, cost × variance against the
default's 6.10e8 (sd 0.29e8, 3 seeds):

| option | cost × variance | reading |
|---|--:|---|
| `--map-tau breit-wigner` (3 seeds) | 5.47e8 (sd 0.34e8) | −10% as run; at each run's own optimum 3.07e8 against 4.34e8. The gain is in @2 (S 544 against 658) and @1 (457 against 505), not @0. Its survey also gives @2 about 1.4× more points per iteration, so a better map and better-fed adaptation are not separated |
| `--grid-bin-weight absolute` (1 seed) | 1.64× worse | not a lever |
| `--grid-max-bin-width none` (1 seed) | 1.52× worse | not a lever |
| split-angle `isotropic` / `windowed` (1 seed) | within noise | not @0's S gap |

@0's S is 1.4× MadEvent's under every option tried. MadEvent splits @0 into a γ channel
(31 pb) and a Z channel (632 pb); vibegraph has one merged channel ("4 (group, diagram)
channels share 1 map"). That is cheap today, because an @0 point costs a quarter of
MadEvent's.
