---
type: Session Report
title: "K-G1 report: integrate's memory on the MLM γ + 4-jet card"
description: "integrate kept every kept iteration's per-channel configuration shares, 1068 channels × 24,297 shares × 16 B = 396 MiB an iteration on GJ-4Jets HT-40to100, and collected about 3.2 GB of share tiles per iteration before reducing any. Shares now fold in as each iteration arrives and tiles reduce in bounded waves, order-preserving: RSS is flat at about 1.86 GB, and σ and grid sha256 are bit for bit. The card's σ is exactly zero at LO (pta 200 against htjmax 100), so it runs to the point cap and refuses."
status: draft
tags: [performance, memory, mlm, multijet, vegas, configuration-shares, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Worktree `/home/user/wt/sprintk-mlm-memory`, branch `sprintk-mlm-memory`,
based on `main` at `f78fb60b`. Inputs: gridpack 244's cards in
`/home/user/wt/sprintk-mlm-memory-mg/gj244/`, copied into per-run directories.
The PDF set lhaid 325300 (NNPDF31_nnlo_as_0118_mc_hessian_pdfas) was already in
`/home/user/inputs/pdf/`. The host has 4 cores and 15 GB, shared with one
sibling. Every run of the card was capped at 8 GB under
`kg1-watchdog.py` (in `/home/user/wt/sprintk-mlm-memory-mg/`). The watchdog
double-forks, samples VmRSS/VmHWM summed over the process tree every 1 s,
kills by PID at 8000 MiB and writes a terminal marker on every exit path.
All card runs used `-j 2` and the release build of `vibegraph` only. The
"before" binary is `main` at `f78fb60b` and the "after" binary is this branch.

## What grows

**Measured first.** Before the fix, the card ran at a reduced budget,
`--neval 20000 --fixed-budget --niter 10 --seed 1`.
`kg1-slope.py` matches each `vegas: iteration k` log line to the RSS samples
since the previous one:

```
$ kg1-watchdog.py kg1-base-n10 8000 -- vibegraph-main-f78fb60b integrate proc_card_mg5.dat \
      --run-card run_card.dat --out out --seed 1 -j 2 --neval 20000 --fixed-budget --niter 10
$ kg1-slope.py kg1-base-n10
iter warm t_end_s max_rss_mib min_rss_mib samples
   1 w    212.6   1108      6 212
   2 w    213.6   1148   1148   1
   3 k    217.0   4354   1774   3
   4 k    221.2   4836   1751   5
   5 k    224.5   5039   3294   3
   6 k    228.4   5511   2522   4
   7 k    232.3   5960   3182   4
   8 k    235.5   6013   3370   3
   9 k    238.9   6816   4695   3
  10 k    242.2   7212   4455   3
slope of per-iteration max: 399.4 MiB/iter over iterations [4, 5, 6, 7, 8, 9, 10]
slope of per-iteration min: 420.1 MiB/iter
peak VmHWM: 7212 MiB
```

The shape has two parts:
- During the two warm-up iterations RSS stays flat at the setup level of about
  1.1 GB.
- From the first kept iteration on, RSS rises linearly at about 400 MiB per
  iteration. On top of that, each iteration carries a transient of about 3 GB:
  the gap between each iteration's max and min.

No step falls at an adaptation or survey boundary. The α survey runs before
iteration 1 and leaves the setup level unchanged. An earlier run at the same
budget with `--niter 40` hit the cap at iteration 12 (`kg1-base-n20k`,
marker `STOP: rss cap t=246.2s`).

The growth was already there at the smallest budget tried. The budget does
not set it, because 1061 of the 1068 channels sit at the 512-point floor
whatever `--neval` is.

**Heap profile.** I used heaptrack 1.5 (Ubuntu package). A whole run under
heaptrack spent over 14 minutes enumerating before I stopped it. So heaptrack
was attached with `heaptrack -p` to a release-debug build of the same commit
once setup had finished (`kg1-heap-attach`, `--niter 6`). Unmerged backtraces
(`heaptrack_print --merge-backtraces 0`), with line numbers from `f78fb60b`:

```
peak heap memory consumption: 5.07G
1.25G peak memory consumed over 4272 calls from
    at vibegraph-lib/src/budget.rs:481      self.kept_shares.push(out.shares.clone());
    at vibegraph-lib/src/budget.rs:868      c.record(out, plan.neval, keep);
    at vibegraph-lib/src/multiplicity.rs:802
415.19M peak memory consumed over 4272 calls from
    at vibegraph-lib/src/vegas.rs:1240      shares: ...collect()   (this iteration's BlockIteration::shares)
207.98M peak memory consumed over 2140 calls from   (and six more such entries, one per worker path)
    at vibegraph-lib/src/vegas.rs:1160      let mut tiles = Vec::with_capacity(...)
```

The allocation-size histogram (`--print-histogram`) has 79,328 allocations of
exactly 388,752 B. That size is 24,297 × 16 B, one `(f64, f64)` pair or two
`f64` per share.

What grows is `ChannelHistory::kept_shares` in `budget.rs`. For every kept
iteration and every channel, it held a clone of the iteration's
per-configuration share vector. The vector is the length of the integrand's
whole share layout: for `MultiplicitySum`, every part's configurations, not
only the channel's own part.

**Arithmetic.** 1068 channels × 24,297 shares × 16 B = 415,187,136 B =
**396.0 MiB per kept iteration**, against the measured 399 MiB (max) and
420 MiB (min) per iteration. The 4272 calls are 4 kept iterations × 1068
channels.

The transient is the share tiles. `adapt_blocks_iteration` collects every
chunk's tiles (2 × n_shares `f64` per 256 points) before reducing any. Each
channel at its 2048-point cap runs 8 chunks of 256 points at `-j 2`. That
gives 8544 tiles × 388,752 B = 3.17 GiB, which matches the roughly 3 GB gap
between each iteration's max and min.

The CERN observation fits this model: 43 kept iterations × 396 MiB plus the
transient and setup comes to about 21 GB, against the 23.9 GB seen by
iteration 45. That host used mimalloc on musl with 4 threads.

## What changed

The change is **order-preserving**. Every sum is formed in the order and with
the weights it had before, so nothing that reaches σ, the grids or the
configuration shares moves by a bit.

- `budget.rs`: `ChannelHistory` now folds each kept iteration's shares into
  `share_sums: Vec<(f64, f64)>`, holding `(Σ wᵢ Iᵢ,c, Σ wᵢ² σ²ᵢ,c)`, and
  `share_weight = Σ wᵢ`. The weights come from the run's
  `IterationCombination`, which is now fixed at construction.
  `combine_shares` divides at the end. These are the products and the
  left-to-right iteration order of the former two-pass loop. The total
  starts at `+0.0` where `Iterator::sum` starts at `-0.0`, and both give
  `w₁` after the first positive weight. Storage is n_shares × 16 B per
  channel, whatever the iteration count.
- `vegas.rs`: `adapt_blocks_iteration` runs its `(block, chunk)` tasks in
  waves of consecutive tasks. Each wave's results are folded into their
  blocks' sums (`BlockSums`) in task order before the next wave starts.
  Tasks are block-major and ascending in chunk order, so every block's
  tiles, values and histogram entries are added in the same point order as
  before. A wave holds at most `SHARE_WAVE_BYTES` = 256 MiB of tiles and at
  least one task per pool thread. With no share requested, as in warm-up
  iterations and share-free integrands, all tasks go in one wave, exactly
  as before.

A wave boundary is a rayon barrier. At 256 MiB and 388 KB per task, a wave
holds about 690 tasks. Smaller integrands run in a single wave.

## Oracles

**1. Peak RSS flat in the iteration count.** This is the same reduced run as
above, after the fix.

```
$ kg1-slope.py kg1-fix1-n10        # --niter 10
  3 k    217.7   1360   1186   3
  4 k    220.6   1856   1746   3
  ...
 10 k    237.7   1819   1666   3
slope of per-iteration max: -6.6 MiB/iter
peak VmHWM: 2179 MiB                        (before: 7212 MiB at the same budget)

$ kg1-slope.py kg1-fix1-n30        # --niter 30, 28 kept iterations
  4 k    220.1   1855   1699   3
 ...
 30 k    295.2   1857   1689   3
slope of per-iteration max: 2.0 MiB/iter over iterations [4 .. 30]
slope of per-iteration min: 1.4 MiB/iter
peak VmHWM: 2172 MiB
```

From the second kept iteration on, the per-iteration max sits between
1696 and 1857 MiB and does not drift. At the old slope, the n30 run would
have needed about 11.5 GB more by iteration 30.

This oracle cannot detect growth that is slower than the 1 s sampling noise
over 27 iterations, about 10 MiB per iteration. It also cannot detect
growth that appears only past the 30th iteration, or only on a card with
nonzero weight. The close-out below covers the long run, on the zero-weight
card only.

**2. Bit for bit against `main`.** These are fixed seed and fixed budget runs
(`--seed 20261119 --fixed-budget --neval 20000 --niter 6 -j 2`), with the
banked cards in `validation/madgraph/` and lhaid 247000 (`kg1-bits.sh`):

```
llj_mlm    base grid_sha256=ffb872ed0b1f2113affd8cf3b7941dceec7f01ecff9ab6019a7fe4f9464c8187 | σ = 268.4 ± 1.4 pb
llj_mlm    fix  grid_sha256=ffb872ed0b1f2113affd8cf3b7941dceec7f01ecff9ab6019a7fe4f9464c8187 | σ = 268.4 ± 1.4 pb
ll0j2j_mlm base grid_sha256=ec6c8e6616d848da37e9b2f14d4167853cba44ed3c701efdd038b49b3f557d10 | σ = 1.0676 ± 0.0064 nb
ll0j2j_mlm fix  grid_sha256=ec6c8e6616d848da37e9b2f14d4167853cba44ed3c701efdd038b49b3f557d10 | σ = 1.0676 ± 0.0064 nb
dy13       base grid_sha256=5f9ae4f1709925a7de38a46028b8a5f31116da031b25e90ae600cac484731d32 | σ = 940.8 ± 4.1 pb
dy13       fix  grid_sha256=5f9ae4f1709925a7de38a46028b8a5f31116da031b25e90ae600cac484731d32 | σ = 940.8 ± 4.1 pb
```

The cases are `pp_to_llj_mlm` (single-multiplicity MLM), `pp_to_ll_0j2j_mlm`
(a `MultiplicitySum` of three parts) and `dy13` (non-MLM). For each case the
full logs are identical apart from the `wrote <path>` line, including every
per-iteration estimate. The artifact banks the configuration shares
(`config_share`), so the sha256 covers them as well as σ and the grids.

These cards' share counts are small enough to run in one wave, so this oracle
exercises the streaming fold of the shares but not a wave boundary.

**3. Wave boundaries move no bit** (`vegas::tests::share_waves_move_no_bit`).
The test has five blocks, including one of zero points, and chunks of one
tile. It runs at wave budgets that hold one task per thread, five tasks, or
every task, on 1 and 3 threads. Each run's `Debug` rendering of every
`BlockIteration` is compared with the single-region run on one thread. The
test asserts that the small budgets really split the 18 tasks into waves.

It would catch:
- a block's chunks folded out of order;
- a block dropped or repeated at a wave boundary;
- an empty block lost.

It cannot detect anything about memory.

**4. A wave keeps to its budget** (`vegas::tests::a_share_wave_keeps_within_its_budget`).
The test checks the wave length:
- every task when no share is summed;
- one task per thread at a 1-byte budget;
- `wave × per-task bytes ≤ budget` at 10 tasks' worth and at
  `SHARE_WAVE_BYTES`.

It checks the sizing arithmetic, not the RSS.

**5. Shares fold without growing** (`budget::tests::shares_fold_in_as_iterations_arrive_without_growing`).
The test records 40 kept iterations of 5 shares, plus one warm-up iteration,
under both combination rules. It checks four things:
- each combined share is bit for bit the two-pass weighted mean (the former
  algorithm, written out in the test);
- the warm-up adds nothing;
- `share_sums` stays at 5 entries;
- the capacity is unchanged after every iteration.

It would fail if the history kept per-iteration shares in `share_sums` again,
or if it weighted or ordered an iteration differently. It cannot see memory
held anywhere other than the channel's share sums: a new per-iteration
`Vec` added elsewhere in `ChannelHistory` would pass it. Oracle 1 is what
covers that.

**Mutation check on oracle 3.** The first version of the test compared runs
of the patched code against each other at different wave budgets. It passed
under a mutation that makes `BlockSums` restart at every chunk (`if true`
for the reopen test), because both sides of the comparison carried the same
fault. The test's reference is now each block run alone as one chunk. Under
the same mutation:

```
$ cargo test -p vibegraph-lib --lib -- vegas::tests::share_waves     # mutated
test vegas::tests::share_waves_move_no_bit ... FAILED
$ cargo test -p vibegraph-lib --lib -- vegas::tests::share_waves vegas::tests::a_share_wave budget::tests::shares_fold   # restored
test budget::tests::shares_fold_in_as_iterations_arrive_without_growing ... ok
test vegas::tests::a_share_wave_keeps_within_its_budget ... ok
test vegas::tests::share_waves_move_no_bit ... ok
```

The older `block_shares_are_independent_of_the_chunking_and_sum_to_the_block`
also fails under that mutation.

**Not run: a multi-wave case on a real card.** The cases in oracle 2 fit in
one wave. A card with the GJ card's layout and a nonzero σ would cross about
13 wave boundaries per iteration. I started one (`htjmax` set to −1 in a copy
of the card) and stopped it after 10 minutes in the base binary's α survey,
to free the slot for the close-out. Oracle 3 is the evidence for wave
boundaries.

**Timing (indicative).** This is not a quiet host: the sibling was building.
ns/eval over the 8 kept iterations of `--niter 10` on the card, before vs
after:

```
before 1507 1907 1521 1734 1756 1474 1552 1483   (mean 1617)
after  1437 1290 1318 1244 1274 1297 1324 1280   (mean 1308)
```

The fixed run is no slower and appears faster, presumably from fewer page
faults on the multi-gigabyte transient. The `kg1-bits` runs on the banked
cards took the same wall time before and after: 3/3 s, 65/67 s and 0/0 s.

## The card's σ is zero at LO

The integration sees no point passing the cuts, and the reason is
kinematic:
- `pta = 200`: the photon has pT ≥ 200 GeV.
- Every other final-state particle is a parton of `p`/`j`, five flavours
  with `maxjetflavor = 5`. They are all jet-class legs, and `htj` sums all
  of them with no pT preselection (`cuts.f:874-913`, as `cuts.rs` documents).
- Transverse momentum balances, Σ p⃗T,partons = −p⃗T,γ. So
  htj = Σ|p⃗T,j| ≥ |Σ p⃗T,j| = pT,γ ≥ 200 GeV.
- `htjmax = 100` therefore rejects every point of every multiplicity. σ is
  identically zero at LO.

The runs agree:
- Every survey reports `survey mean 0.000e0`.
- Each multiplicity's σ is 0 with a ~1e-150 error.
- The run ends with the named refusal `every one of the … points the
  integration evaluated has zero weight, and none passed the cuts … no
  artifact was written`, exit status 1.

**Whether MadGraph's gridpack has a nonzero σ is not known here.** The
README the CERN host shipped records the cards, their sha256 and vibegraph's
memory, but no MadGraph cross section, `results.dat` or event count. The run
card has `gridpack = True` and `nevents = 3000`, which says nothing about σ.
MadGraph's own `htjmax` cut is the one quoted above, so it should also find
zero. If the gridpack carries events, the two readings disagree, which is a
finding (see Found).

## Close-out run

The full card at the host's settings, apart from the thread count:
`--neval 120000 --target-rel 0.01 --seed 1 -j 2`, with this branch's
release binary, detached under the 8 GB watchdog (`kg1-closeout`).

```
$ kg1-watchdog.py kg1-closeout 8000 -- vibegraph integrate proc_card_mg5.dat --run-card run_card.dat \
      --out out --seed 1 -j 2 --neval 120000 --target-rel 0.01
marker: FAILED rc=1 t=740.7s
  210.7  INFO vegas: 1068 channels over 4 to 13 coordinates, 597751 points per iteration, allocated by αⱼsⱼ (Neyman)
  215.1  INFO vegas: iteration 3: 0.000000e0 ± 4.874812e-153 (χ²/dof n/a), 2217168 points at 1414 ns/eval
  731.3  INFO vegas: iteration 181: 0.000000e0 ± 3.643605e-154 (χ²/dof 0.000), 2217168 points at 1282 ns/eval
  740.4  INFO support:  0 of 396873072 kept points carry weight (effective sample size 0); 0 of 399687991 integration points passed the cuts
  740.4  WARN 539427 of 399687991 integration points were zeroed as degenerate (...)
  740.6 ERROR every one of the 399687991 points the integration evaluated has zero weight, and none passed
        the cuts: the cuts reject every point the sampler reached, whether the run card's or the bwcutoff
        window a decay chain holds each of its resonances in; no artifact was written
```

The first kept iteration was read at 215 s: RSS 1831 MiB, the line above. I
extrapolated a flat run of about 180 iterations at about 3 s each.

The run completed in **740.7 s wall** with **peak RSS (VmHWM) 2140 MiB**,
against the 12 GB limit the item sets. The CERN host reached 23.9 GB by
iteration 45. The run did 181 iterations, the last to reach the 400M
`--max-points` cap. It ended in the card's named refusal, exit status 1, not
a panic.

RSS over the iterations, as the per-iteration max / min in MiB
(`kg1-slope.py`):

```
iter   1 w  1124      5      (setup → first warm-up)
iter   3 k  1332   1315
iter  20 k  1851   1748
iter  40 k  1861   1854
iter  60 k  1711   1660
iter  80 k  1746   1673
iter 100 k  1861   1804
iter 120 k  1860   1660
iter 140 k  1725   1682
iter 160 k  1860   1815
iter 180 k  1736   1691
iter 181 k  1860   1859
slope of per-iteration max over iterations 4-181: 0.1 MiB/iter; of the min: 0.0 MiB/iter
```

Of the 740.7 s, about 211 s is setup: enumeration, compilation and the α
survey. About 520 s is 181 VEGAS iterations over an integrand that is zero
at every point, and about 9 s is the end-of-run reporting (Found 1).

## Gates

All gates were run on the committed tree:

```
$ cargo fmt --all --check                                     → clean
$ cargo clippy --workspace --all-targets -j 2 -- -D warnings  → Finished, no warning
$ cargo test --workspace -j 2                                 → EXIT 0; 69 test binaries,
      2233 passed, 0 failed, 26 ignored (lib: 1424 passed, 15 ignored)
$ pixi run kb-lint                                            → 0 errors, 5 warnings (all on
      backlog items this branch does not touch)
```

I did not run `pixi run --skip-deps validate` locally. The sprint session
ruled that CI's banked job on the PR is the banked gate for this change.
Oracle 2 is the local evidence that σ, the grids and the configuration shares
do not move.

## Corrections to the brief

- "Growth that tracks the iteration count rather than the channel count"
  (the item) is half right. Each kept iteration adds channels × shares × 16 B,
  so the slope scales with both the channel count and the share count.
  Warm-up iterations add nothing.
- "Other GJ-4Jets cards held about 1.23 GB" does not mean they are immune.
  Any card with the same channel and share layout grows at the same rate per
  kept iteration. This card is distinctive because its σ is zero, so it
  never meets the 1% target and runs to the point cap. A card that
  converges in a few kept iterations stops before the growth shows.
- The brief's "the distinctive feature is the narrow HT window with pta 200"
  (from the README) is exactly the zero-σ cause. It is not a memory property.
- A heap profile of a whole run under heaptrack is impractical on this card,
  because enumeration and compilation under heaptrack take over 15 minutes.
  Attaching after setup (`heaptrack -p`) profiles the iterations, and those
  are what grows.

## Found

1. **A run whose cuts reject every point runs to the point cap before
   refusing.** On this card `integrate` evaluates about 400M points of
   identically zero weight (181 iterations of 2.2M) and then refuses. On the
   close-out that took about 520 s of the 741 s wall time at `-j 2`. On the
   CERN host it would have run the same 181 iterations at 4 threads.
   Nothing can change after
   the first kept iteration once no point has passed the cuts, so stopping
   there with the same refusal would save almost all of it. To close it:
   - stop at the first kept iteration where the census shows zero points
     passing the cuts over every channel;
   - test it on a card with the same pT-balance contradiction.
2. **Configuration shares are stored at the full layout's length per
   channel.** For `MultiplicitySum`, every channel's share vector and its
   tiles span every part's configurations, although a channel writes only its
   own part's slice. On this card that is 945 of the 1068 channels across
   24,297 shares, of which the 2→5 part is most. The remaining fixed cost is
   one such vector per channel in the history and one in the iteration's
   outcomes, about 400 MiB each here. It grows as channels × shares for
   larger processes. To close it:
   - give `ChannelIntegrand` a per-channel share range;
   - size `BlockIteration::shares` and the tiles to it;
   - have `config_share::summed` read through the offset;
   - check that the artifact sha256 is unchanged.
3. **MadGraph σ for gridpack 244 is unrecorded.** Whether MadGraph's gridpack
   for GJ-4Jets_dRGJ-0p25_PTG-200_HT-40to100 has a nonzero cross section is
   not known here. If the CERN host's gridpack holds events, MadGraph and
   this crate read the card differently, for example an `htj` that counts
   only some jets, and that would be a parity finding. To close it, read the
   gridpack's `results.dat` / `madevent` survey output on the CERN host.
4. **A release build warns `method resolve_mom is never used`**
   (`vibegraph-lib/src/helas/eval/analysis.rs:208`). The warning appears on
   `cargo build --release -p vibegraph` at `f78fb60b` and is untouched here.
   Clippy's dev build does not see it. To close it, gate the method on the
   cfg that uses it, or drop it.
5. **The worktree's `mg5amcnlo` submodule gitdir is missing.**
   `git diff --stat` with no path fails with `fatal: not a git repository:
   research/refs/mg5amcnlo/../../../.git/modules/...`, because the content was
   copied in without its `.git/modules` entry. Path-limited git commands
   work. To close it, have the worktree setup that copies the submodule
   content also register it, or pass `--ignore-submodules` in the dispatch
   recipe.
6. **The γ + 4-jet card zeroes 0.13% of its points as degenerate.** The
   close-out warns `539427 of 399687991 integration points were zeroed as
   degenerate`, and the warning's own text says that a share beyond a
   handful points at a map drawing outside its range. Here every point
   fails the cuts anyway, so it moves no number. The same maps on a card
   with weight would lose those points silently. To close it, take a
   per-channel census of the degenerate points on this layout, for example
   on the card with `htjmax` off, and find which map's draws they are.
