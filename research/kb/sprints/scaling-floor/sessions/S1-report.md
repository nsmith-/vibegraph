---
type: Session Report
title: "S1 report: a floor that scales with neval reaches the floor-bound channels and does not change the bbx ladder"
description: "ChannelFloor::ScalesWithBudget (512 accepted points up to neval 600 000, proportional above) is implemented beside today's floor, default unchanged and bit-identical. On bbx_to_ccx_emmm_qcd0, 20 paired seeds, it doubles every floor-bound channel's points at 1.2M (0 of 615 channels share counts with 600k) for 1.48x the points and 1.24x the wall time, and leaves the ladder where today's floor leaves it: seed sd 2.74e-9 -> 3.41e-9 (ratio 1.245; 0.901 without seed 55) against 2.74e-9 -> 3.34e-9 (1.220; 0.905), chi2/dof 1.70 against 1.43. The floor-held channels carry about 2% of the quoted variance; the excess is in alpha-share channels (613's seed-55 point), which no floor rule touches. Channel 551's quoted variance falls x0.39 (geometric mean) for x2 points but rises on 6 of 20 seeds."
status: draft
tags: [multichannel, allocation, coverage-floor, two-to-six, heavy-tail, budget-ladder, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-27/28, one long job at a time; two container restarts (before 23:35Z and 23:57Z on 10-27) killed runs, finished seeds kept, missing seeds rerun"
  commit: "ladder: probe binary built from a2b31a5 + this session's code diff (identical to commit 30a8b3b's code, diff sha256 58a0c39a…) + the uncommitted VG_TRACE instrumentation; validate pair on 30a8b3b (tree 46b7a965) and on 30a8b3b with the default flipped (throwaway)"
  command: "see Method; TSVs, traces, logs and scripts under the session scratchpad, prefixed sf-S1-"
sources:
  - {id: brief, resource: "S1.md", title: "S1 brief"}
  - {id: p1, resource: "../../floor-bound-neyman/sessions/P1-report.md", title: "P1 report, the scaling-floor proposal"}
  - {id: c1, resource: "../../floor-bound-neyman/sessions/C1-report.md", title: "C1 report, the plain-mean combine"}
  - {id: budget, resource: "../../../../../vibegraph-lib/src/budget.rs", title: "ChannelFloor, floor_for_acceptance, integrate_channels_with_floor"}
---
## Verdict

- **The rule works as designed and does not fix the ladder.** At 1.2M every
  floor-bound channel draws twice its 600k points (0 of 615 channels share a
  count history between the rungs, against 563–580 under today's floor), and
  the per-iteration spend doubles with the budget (1.74M → 3.50M points over
  iterations 1–7). The per-seed sd still rises from 600k to 1.2M, ×1.245,
  against ×1.220 under today's floor; χ²/dof reads 0.82 → 1.70 (today's: 0.82 → 1.43).
  Without seed 55 the two arms read ×0.901 and ×0.905. The item's `closes_when`
  is not met, and this record says why.
- **Why: the floor-held channels are not where the variance is.** At equal
  `neval` the two arms draw the same α-share channels (613 included, bit for
  bit), so their paired difference isolates what the floor rule changed. It is
  −1.7e-10 pb per seed with sd 9.3e-10, against a seed sd of 3.3e-9 (rung
  correlation +0.96). Split by channel, the ~569 channels the floor holds at 1.2M
  quote 5.7e-10 of error under today's floor and 2.4e-10 under the scaled one;
  the other ~46 quote 3.6e-9. The floor-held share of the quoted variance is
  2.4 % today and 0.4 % scaled. The ladder's excess is seed 55's 1.18e-8 pb
  point in channel 613, an α-share channel that doubles its points under both
  rules and is drawn identically by both.
- **At equal spend it loses.** The scaled arm spends 1.48× the points and 1.24×
  the wall time of today's floor at 1.2M for the same sd: sd² × points per
  iteration 3.82e-11 against 2.47e-11, sd² × wall 2.16e-14 against 1.67e-14.
  The 800k scaled rung that would have matched today's 1.2M spend point for point
  was dropped (see Step 3).
- **Falsifier: not met at every seed.** Channel 551's quoted variance falls on 14
  of 20 seeds when its points double (geometric mean ×0.39), and rises on six,
  by up to ×4.9 (seed 20260719). Its terms are heavy-tailed at the per-seed
  level, so a doubled sample reaches further into the tail on some seeds.
- **Default path bit-identical**, and the scaled rule equals today's at 600k:
  shown on seeded runs and traces (Step 2).
- **D1: a switch moves no banked cell** (Step 5): every gate runs at a budget at
  or below 600 000, where the rule is today's floor.
- **Recommendation: do not switch the default.** The rule stays available
  (`ChannelFloor::ScalesWithBudget`, `VG_2TO6_FLOOR=scaled`). The ladder needs
  work on the α-share channels' tails (Found 1).

## Step 1: design and offline sizing

Today's floor is `⌈512/acceptance⌉` drawn points, capped at 4 × 512, the same at
every `neval`. Candidates, each an accepted-point floor `A(neval)`, drawn floor
`min(⌈A/acceptance⌉, 4A)`:

| candidate | A(neval) | spend | channel with acceptance < 1/4 gets | depends on measured quantities? |
|---|---|---|---|---|
| today | 512 | `Σ max(shareⱼ, floorⱼ)`, above `neval` whenever floors bind | 2048 drawn | acceptance (as today) |
| ref R (chosen, R = 600 000) | `max(512, ⌈512·neval/R⌉)` | as today; grows ∝ `neval` past R | `4·A(neval)` drawn | acceptance only; `A` from `neval` |
| ref R = 300 000 | same with R = 300k | as today, ×1.67 at 600k | `4·A` | acceptance only |
| frac f | `max(512, ⌈f·neval/n_ch⌉)` | floors take ~f/acceptance of `neval` whatever the channel count | `4·A` | acceptance only |
| carve f | as frac, α shares taken from `(1−f)·neval` | closest to `neval`; still above it once acceptance corrects the floors | `4·A` | acceptance only |

No candidate keeps the total at `neval`: the acceptance correction and the 512
minimum already put today's 600k iteration at 1.74M points. Every candidate reads
its count from the channel's measured acceptance of earlier iterations, as today;
none reads a variance. The counts therefore change between runs with the
outcomes, and with them the stream positions (`first_point` is the channel's
cumulative draw), which is safe since #75 (the plain mean does not weight by
count) and since every count is set before its points are drawn.

Replayed on B1's ByAlpha trace (the acceptance history of today's counts, an
approximation for the other rules):

```
$ python3 -I sf-S1-scripts/sizing.py t26-B1-main-trace.txt      (sf-S1-sizing-B1.txt)
today's rule replayed against the trace's drawn counts: 0 mismatches
```

| candidate | rung | A | floor-bound ch., it. 1–7 | above floors | points / it. 1–7 | it. 0 points | identical count histories 600k vs 1.2M |
|---|---|---|---|---|---|---|---|
| today | 600k | 512 | 589.3 (576–599) | 0.316 | 1 736 706 | 887 683 | |
| today | 1.2M | 512 | 583.6 (574–596) | 0.488 | 2 323 099 | 1 481 064 | 563–580 (569.5) |
| ref 600k | 600k | 512 | 589.3 | 0.316 | 1 736 706 | 887 683 | |
| ref 600k | 1.2M | 1024 | 589.6 | 0.316 | 3 474 814 | 1 775 364 | 0 |
| ref 300k | 600k | 1024 | 597.6 | 0.180 | 2 898 181 | 1 186 048 | |
| ref 300k | 1.2M | 2048 | 597.7 | 0.180 | 5 800 039 | 2 372 095 | 0 |
| frac 0.5 | 600k | 512 | 589.3 | 0.316 | 1 736 706 | 887 683 | |
| frac 0.5 | 1.2M | 976 | 589.1 | 0.327 | 3 366 463 | 1 747 597 | 0 |
| frac 1.0 | 600k | 976 | 597.1 | 0.188 | 2 788 862 | 1 157 932 | |
| frac 1.0 | 1.2M | 1952 | 597.2 | 0.188 | 5 581 197 | 2 315 863 | 0 |
| carve 0.5 | 600k | 512 | 597.6 | 0.180 | 1 449 129 | 593 024 | |
| carve 0.5 | 1.2M | 976 | 597.2 | 0.188 | 2 790 637 | 1 157 932 | 0 |

The same replay on P1's Neyman trace (`sf-S1-sizing-P1.txt`) agrees to the
first decimal on every column.

**Chosen: ref 600k.** Reasons:
- Past R every channel's count is ∝ `neval`, so a doubled budget doubles every
  channel's points and the floors keep the share they have at R (0.316 above
  them at both rungs). That is the precondition for the 1/√2 the item asks for.
- At and below R it is today's floor exactly. Every gate integrates at ≤ 600 000
  (the 2 → 6 rows at 600k, MLM rows at 150–200k, the banked rows ≤ 200k), so the
  rule is the user's to switch on at no gate cost; it differs only on a budget
  raised past the gates'. Its 600k rung is the control's, which halves the
  ladder's cost.
- It does not depend on the channel count. `frac` sets every channel at
  `f·neval/n_ch`, which on a 6-channel process at 150k is 12 500 accepted
  points per channel: a uniform mixture blended into the α split everywhere,
  not a floor fix on wide splits.
- `ref 300k` and `frac 1.0` buy the same scaling at 1.67× the 600k gate's cost.
- `carve` cannot keep the spend at `neval` either, and starves the α shares
  that turn out to carry the variance (Step 3).

## Step 2: the rule, and bit-identity of the default path

`vibegraph-lib/src/budget.rs`: `ChannelFloor { Fixed (default), ScalesWithBudget }`,
`FLOOR_REFERENCE_NEVAL = 600_000`, `ChannelFloor::accepted_points(neval)` (integer
ceiling, no overflow), `floor_for_acceptance(accepted_floor, acceptance)` (the cap is
`4 × accepted_floor`), `integrate_channels_with_floor` (`integrate_channels` passes
`ChannelFloor::default()`), and the split report and warnings name the floor in
force. `hadronic.rs`: `FixedBeamIntegrand::adapt_grids_budget_with_floor`.
`probe_2to6_budget_ladder`: `VG_2TO6_FLOOR=fixed,scaled`, a `floor` TSV column after
`alloc` (an older TSV is refused), the floor in every progress and summary line,
and a value with a comma as a seed list (`11,` is one seed; a rung of one seed
prints no summary), so a restart can rerun single seeds.

Tests (`budget::tests`):
- `the_scaling_floor_is_the_fixed_one_up_to_the_reference`: `Fixed` is 512 at
  every budget; `ScalesWithBudget` is 512 at 0…600 000, 513 at 600 001, 1024 at
  1.2M, 5120 at 6M, monotone, never below 512 and within 1 of the proportional
  ask over 401 budgets, finite at `usize::MAX`; the correction and cap scale with
  it.
- `the_scaling_floor_raises_a_floored_channel_only_past_the_reference`: at
  `neval` 4000 the two rules give bit-identical terms, totals and per-channel
  points; at 1.2M the scaled rule's first iteration draws 1024 per minor channel,
  all 7 minors are floor-bound, and each gathers > 0.8 × 1024 accepted points an
  iteration, while the fixed rule at the same budget stays under 1.2 × 512.

```
$ pixi run cargo test -p vibegraph-lib --lib budget::
test budget::tests::the_scaling_floor_is_the_fixed_one_up_to_the_reference ... ok
test budget::tests::the_scaling_floor_raises_a_floored_channel_only_past_the_reference ... ok
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 1085 filtered out; finished in 0.30s
```

**Transformation class: order-preserving on the default path.** `ChannelFloor::Fixed`
evaluates `accepted_floor = 512` and the same arithmetic as before. Evidence:

```
$ python3 -I sf-S1-scripts/check.py      (sf-S1-check.txt)
fixed 600000 20260719: this tree 1.692998322e-6 ± 3.232010e-9 pts 13058820 | C1 rerun 1.692998322e-6 ± 3.232010e-9 pts 13058820 EQUAL | B1-trace plain replay EQUAL
scaled 600000 20260719: … | fixed: TSV-EQUAL | 14-field trace (4920 lines) IDENTICAL
scaled 600000 11:       … | fixed: TSV-EQUAL | 14-field trace (4920 lines) IDENTICAL
```

- The scaled rule at 600k is today's rule draw for draw (σ, error, points,
  floor-bound count, watched channels and all 4920 trace lines with the floor
  and share columns, on two seeds).
- The default path reproduces C1's post-#75 rerun of seed 20260719 at 600k to
  every printed digit and point.
- The validate pair (Step 5) shows the default flip changes no cell: the
  rule is today's at every gate budget.

**Brief correction (control).** The brief offered a replay of B1's main-tree
trace under the plain mean as the control, if exact on two seeds per rung. It is
not exact on this tree: seed 20260719 matches at both rungs, but seed 11 reads
1.692990955e-6 ± 2.937474e-9 here against 1.692570819e-6 ± 2.925258e-9 replayed
(600k), and 1.694838626e-6 against 1.694514372e-6 (1.2M). The trace shows why:
the α vectors differ from B1's in the last digit on 507 of 615 channels (seed
11) and on a few even for seed 20260719, and VEGAS amplifies that into a 2.5e-4
difference on seed 11. It is not run order: seeds 20260719 and 11 run in one
process (`sf-S1-order.tsv`) reproduce this tree's single-seed values. `main` has
moved since B1's and C1's trees (#60, #67, #72, #74 touch PDFs, cuts, run cards
and `proton.rs`), and the survey arithmetic moved with it at the ulp. So the
control was rerun, 20 seeds × 2 rungs on this tree. Its summary differs from the
replay by under 1 % in sd (2.737e-9 / 3.339e-9 against 2.714e-9 / 3.330e-9).

## Step 3: the paired ladder

`bbx_to_ccx_emmm_qcd0`, multi sampler, ByAlpha, 8 iterations (2 warm-up), seeds
20260719, 11, 22 … 209. `fixed` is today's floor and `scaled` the new rule, both
on this tree; `scaled` at 600k is `fixed` at 600k (Step 2). Points per iteration
are over all 8 iterations; wall time is per seed on this host, one job at a time.

```
$ python3 -I sf-S1-scripts/ladder.py      (sf-S1-ladder.txt)
```

| arm | rung | mean (pb) | quoted err | spread/√n | per-seed sd | RMS quoted | χ²/dof | points / iteration | wall / seed (min–max) | pull vs MadEvent reported / equal-weight |
|---|---|---|---|---|---|---|--:|---|---|---|
| fixed | 600k | 1.690032e-6 | 6.954e-10 | 6.120e-10 | 2.737e-9 | 3.110e-9 | 0.82 | 1 630 578 | 1000 s (898–1274) | +2.47 / +1.39 |
| fixed | 1.2M | 1.691703e-6 | 8.189e-10 | 7.465e-10 | 3.339e-9 | 3.662e-9 | 1.43 | 2 217 845 | 1500 s (1242–1953) | +3.48 / +2.33 |
| scaled | 600k | = fixed 600k | | | 2.737e-9 | | 0.82 | 1 630 578 | | |
| scaled | 1.2M | 1.691528e-6 | 7.972e-10 | 7.622e-10 | 3.409e-9 | 3.565e-9 | 1.70 | 3 287 170 | 1857 s (1580–2152) | +3.35 / +2.22 |

Over iterations 1–7 the iteration is 1 736 706 points at 600k, 2 323 099 for
`fixed` at 1.2M and 3 503 143 for `scaled` at 1.2M (×2.02 the 600k iteration).
Wall times carry the ±15 % run-to-run noise P1 measured on this host.

Paired per-seed differences (20 seeds):

| pair | Δ mean | sd(Δ) | t | seed sd a → b | sd ratio (leave-one-out) | correlation |
|---|---|---|--:|---|---|--:|
| fixed 600k → 1.2M | +1.671e-9 | 4.495e-9 | +1.66 | 2.737e-9 → 3.339e-9 | 1.220 (0.905–1.411) | −0.09 |
| scaled 600k → 1.2M | +1.496e-9 | 4.699e-9 | +1.42 | 2.737e-9 → 3.409e-9 | 1.245 (0.901–1.394) | −0.16 |
| scaled − fixed, 1.2M | −1.746e-10 | 9.256e-10 | −0.84 | 3.339e-9 → 3.409e-9 | 1.021 (0.988–1.051) | +0.96 |

Without seed 55 (its 1.2M runs read 1.7014e-6 ± 1.2e-8 under both rules):

```
('fixed', 600000) without seed 55: sd 2.760e-09 chi2/dof 0.84
('fixed', 1200000) without seed 55: sd 2.499e-09 chi2/dof 1.24
('scaled', 1200000) without seed 55: sd 2.487e-09 chi2/dof 1.45
fixed sd ratio 1.2M/600k without 55: 0.905
scaled sd ratio 1.2M/600k without 55: 0.901
```

**Where the variance sits** (`decomp.py`; the floor-held set is the channels
whose every iteration under today's floor drew more than their α share, at that
rung, applied to both arms):

```
fixed    600000 n=20: floor-held channels 579 | RMS quoted 6.759e-10 | rest RMS quoted 3.036e-09
fixed   1200000 n=20: floor-held channels 569 | RMS quoted 5.686e-10 | rest RMS quoted 3.618e-09
scaled  1200000 n=20: floor-held channels 569 | RMS quoted 2.387e-10 | rest RMS quoted 3.557e-09
1.2M scaled vs fixed over 20 seeds: bit-identical channel histories per seed 16-31 of 615;
  paired Δ floor-held part mean -3.861e-11 sd 5.868e-10; Δ rest mean -1.360e-10 sd 8.269e-10
```

(The parts' across-seed sds are not read: the per-seed α survey moves σ between
the two sets.) The scaled rule cuts the floor-held channels' quoted error ×2.4, as
designed, but they are 2.4 % of the quoted variance at 1.2M. The 46 other channels
hold the rest, and they draw ∝ `neval` under both rules already.

**Equal spend.** The scaled arm at 1.2M spends 1.48× the points of today's for
the same seed sd. Per seed, sd² × points per iteration is 2.47e-11 (fixed 1.2M)
against 3.82e-11 (scaled 1.2M), and sd² × wall 1.67e-14 against 2.16e-14.

**The 800k scaled rung was dropped.** It was queued as the point-for-point
equal-spend arm (its iteration, 1.74M × 4/3 ≈ 2.32M points, matches `fixed` at
1.2M), 20 seeds and about 7.5 h. The decomposition above decides it without the
run. At equal spend, `scaled` at 800k gives the α-share channels 2/3 of
`fixed`-1.2M's points, and those channels carry ≥ 97 % of the quoted variance;
it moves the remaining 1/3 into floor-held channels that carry under 3 %. It
cannot come out ahead unless the α-share channels' variance stops falling with
points, and then neither rule helps.

## Step 4: the falsifier, channels 551 and 613

Per seed, the channel term's quoted variance (plain mean of its kept iterations)
and kept points, 600k → 1.2M:

```
ch551 fixed  600000->1200000: quoted var fell on 3/20 seeds; geo-mean points ×1.00, quoted var ×1.251; max var ratio 3.70  (12/20 seeds bit-identical)
ch551 scaled 600000->1200000: quoted var fell on 14/20 seeds; geo-mean points ×2.00, quoted var ×0.392; max var ratio 4.91
     20260719 ×4.91  11 ×0.52  22 ×3.17  33 ×0.06  44 ×0.07  55 ×0.10  66 ×0.06  77 ×0.93  88 ×1.05  99 ×1.45
     110 ×0.37  121 ×0.38  132 ×0.03  143 ×3.36  154 ×0.49  165 ×0.25  176 ×0.42  187 ×0.10  198 ×0.32  209 ×1.21
ch613 (both arms) 600000->1200000: quoted var fell on 17/20 seeds; geo-mean points ×2.00, quoted var ×1.001; max var ratio 20516.54 (seed 55)
```

| channel | arm | rung | mean kept points | RMS quoted error | across-seed sd | largest single-point share |
|---|---|---|---|---|---|---|
| 551 | fixed | 600k | 12 288 | 8.08e-11 | 2.27e-10 | 1.45e-10 |
| 551 | fixed | 1.2M | 12 288 | 1.02e-10 | 2.81e-10 | 2.06e-10 |
| 551 | scaled | 1.2M | 24 576 | 7.26e-11 | 3.28e-10 | 2.11e-10 |
| 613 | both | 600k | 26 182 | 1.06e-10 | 4.08e-9 | 6.78e-11 |
| 613 | both | 1.2M | 52 364 | 2.64e-9 | 4.79e-9 | 1.18e-8 (seed 55) |

- **551, scaled.** Its points double on every seed and its quoted variance falls
  ×0.39 in geometric mean, faster than 1/2. But it rises on 6 of 20 seeds, by
  ×4.9, ×3.2 and ×3.4 on three of them. The falsifier as written ("must fall at
  every seed") fails. Not because the floor is not reaching the channel: the
  per-seed variance estimate of a tail-index-2 term is itself heavy-tailed, and
  twice the sample reaches further into the tail on some seeds. Its RMS quoted
  error falls only 8.1e-11 → 7.3e-11.
- **551, today's floor.** Twelve seeds draw identical histories at both rungs;
  where they differ the variance rises (×1.25 geometric), as B1 and P1 found.
- **613** is an α-share channel, never floor-bound. It draws bit-identical
  histories under both rules at each rung, and its seed-55 point (1.18e-8 pb,
  0.7 % of σ) is the ladder's largest single excursion under both.

## Step 5: what a default switch moves (D1)

`sf-S1-validate.sh`: `pixi run --skip-deps validate` on the committed tree
(`base`), then on the same tree with `#[default]` moved to
`ChannelFloor::ScalesWithBudget` (`flip`, reverted afterwards), detached, one at
a time, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

```
base: 30a8b3b, tree 46b7a965, clean, 2026-10-28T03:38:37Z – 04:55:57Z
flip: 30a8b3b + `#[default]` on ScalesWithBudget (diff sha256 a9a189a6…, 1 line moved), 04:55:57Z – 06:12:40Z
$ CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 pixi run --skip-deps validate
base: 74 rows × 4 categories = 296 cells: 216 measured in the layers this run drove (206 ✅, 5 ⚠️, 5 ❌, 13 ⏳, 67 — / uncovered). exit 101
flip: 74 rows × 4 categories = 296 cells: 216 measured in the layers this run drove (206 ✅, 5 ⚠️, 5 ❌, 13 ⏳, 67 — / uncovered). exit 101
test results summed: base 2086 passed, 14 failed, 98 ignored; flip 2086 passed, 14 failed, 98 ignored
the same 14 tests fail on both (sf-S1-failed-{base,flip}.txt)
$ python3 -I fbn-C1-scripts/cells.py sf-S1-report-base sf-S1-report-flip   (every cell file, durations and host stripped)
cells identical 217, moved 0
```

**No cell moved.** Every cell either run writes is identical under the switch,
as the design predicts: every gate's budget is at or below 600 000.

**Both runs fail, and not because of this change.** The 14 failing tests and 5 ❌
cells are missing reference data in this worktree. The copied bundle predates
#60/#67/#72/#74, which added rows and inputs:
- `pp_to_bb_iht` and `pp_to_jj_ht` run directories, absent;
- the `dy13_etau_default` seed record, absent;
- `dy13_beams_6500_4000/Events/run_beams_6500_4000/unweighted_events.lhe.gz`, absent;
- `bbx_to_tapta_ckm_no_b_mass`, absent (the scales gates);
- `pp_to_ll`'s banked run card still reads `pdlabel = nn23lo1` (the α_s-source
  gates).

Each failure names its missing input (`sf-S1-validate-base.log`). None is in
code this session touched, and the failing set is identical with and without the
switch. The banked layer was therefore not re-run green here. A run with the
current bundle (`pixi run fetch-refdata`) is the coordinator's call; see the brief
correction below.

**Which gates and budgets a switch touches.** None of today's: the switch changes
an allocation only where `neval > 600 000`.
- The 2 → 6 integrals cells run at 600 000 (`Plan::Long`): 512 = today's floor,
  bit-identical (Step 2).
- The mixed-row MLM gates run `--fixed-budget --allocate neyman --neval 200000`
  (`pp_to_ll_0j2j_mlm`, `pp_to_ttx_0j1j_mlm`) and the `llj` MLM rows 150 000: untouched.
- The banked σ and samples gates run at 12 000–200 000; the CLI's target mode
  defaults to `--neval 120000` per iteration: untouched.
- What it would change: any `integrate` run or probe at `neval > 600 000`, such
  as the 1.2M ladder rung (+48 % points there) and samples integrated at larger
  budgets. A user asking for a 1.2M iteration on a wide split would pay
  1.5–2× the points.
- 20-seed deciding pairs a switch would need: none to protect a gate. To justify
  the switch on its merits it would need a process where floor-held channels
  carry a material share of the variance at > 600k. bbx, the row the floor binds
  hardest on, is not one (2.4 %).

## Gates

On 30a8b3b plus the final doc paragraph on `ChannelFloor::ScalesWithBudget` and this
report (`sf-S1-gates.sh`, log `sf-S1-gates.log`), detached, with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ pixi run cargo fmt --all --check                                          --> exit 0
$ pixi run cargo clippy --workspace --all-targets -- -D warnings            --> exit 0
$ pixi run cargo clippy --workspace --all-targets \
    --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings   --> exit 0
$ pixi run cargo test --workspace                                           --> exit 0
  48 test-result lines: 1706 passed, 0 failed, 19 ignored
$ pixi run kb-index                                                         --> exit 0
$ pixi run --skip-deps kb-lint                                              --> exit 0
  956 concepts, 299 backlog items; 0 errors, 4 warnings
```

The four warnings are pre-existing `state: blocked` items this session did not
touch. The default path is bit-identical (Step 2). The validate pair (Step 5) is
a measurement, as the brief sets it, and it fails identically on both sides on
missing reference data.

## Method

- Scripts (`$SP/sf-S1-scripts/`): `sizing.py` (Step 1), `check.py` (Step 2),
  `ladder.py` (Steps 3–4, reads the TSVs and traces, asserts every trace replay
  equals its TSV σ to 1e-9 relative and its points exactly), `decomp.py` (Step 3).
- Probe binary: `sf-S1-build.sh` builds `validate_sigma` (`--profile release-debug
  --features extended-validation`, `debug=0`) from a2b31a5 + the code diff (sha256
  58a0c39a…, the code later committed as 30a8b3b) + `sf-S1-instrumentation.patch`
  (P1's `VG_TRACE` patch, its floor column reading the rule's `accepted_floor`),
  and copies it to `$SP/sf-S1-validate_sigma-instr` (sha256 d037f568…). The
  instrumentation was reverted before the commit.
- Drivers, launched with `nohup setsid`: `sf-S1-run.sh <tag> <floor> <rung>
  <seeds>` runs one process per seed (`VG_2TO6_SEEDS=<seed>,`), skips seeds
  already in the TSV and drops a partial trace of the seed it reruns;
  `sf-S1-ladder{,2,3,4}.sh` sequence the steps; `sf-S1-chain.sh` and
  `sf-S1-validate.sh` run the validate pair.
- Outputs: `sf-S1-check.tsv` / `-trace.txt` (today's floor, 20 seeds × 2 rungs),
  `sf-S1-scaled.tsv` / `-trace.txt` (scaled, 1.2M × 20 and 600k × 2),
  `sf-S1-order.tsv` (the run-order check), `sf-S1-report-{base,flip}` (validate
  cell files).
- Restarts: the container restarted twice (before 23:35Z and 23:57Z, 10-27)
  during the control's 1.2M rung. Each restart lost the seed in flight; the
  drivers reran exactly the seeds missing from the TSV. Every seed-run appears once
  in each TSV, and `ladder.py`'s loader keeps only complete 8-iteration runs.

## Brief corrections

- **The control replay** is not exact on this tree (Step 2). The control was
  rerun: 20 seeds × 2 rungs, about 14 h of the session's compute.
- **"Reference data is already copied in"**: the worktree's
  `validation/madgraph/output` predates main's newest rows, so
  `pixi run --skip-deps validate` fails 14 tests on missing inputs before and
  after any change of this session (Step 5).
- **The 800k equal-spend rung** was the session's own addition, not the brief's.
  It was dropped after the coordinator asked, on the decomposition in Step 3.

## Found

1. **The bbx ladder's excess sits in α-share channels, not floor-bound ones.**
   - Evidence: at 1.2M the floor-held ~569 channels quote 2.4 % of the variance
     (0.4 % under the scaled floor). Seed 55's single point in channel 613 (α
     share, 52 364 points) is 1.18e-8 pb of a 1.2e-8 quoted error. Without that
     seed both arms' sd ratio is 0.90 and χ²/dof 1.24 / 1.45.
   - Under the plain mean, channel 613's quoted variance does not fall with
     doubled points (geometric ×1.00 over 20 seeds), as P1 saw under the old
     combine.
   - To close it: measure the α-share channels' tail (613, 614, 590 per P1's
     shift list) on this ladder, e.g. the per-channel largest-point share and
     its growth with points. Then decide between a sampler change in those
     channels (grid or map resolution of the tail) and a stated heavy-tail
     error model. Allocation rules cannot reach it.
2. **The falsifier criterion is too strict for heavy-tailed channels.** "Quoted
   variance falls at every seed" fails on 6 of 20 seeds for a channel whose
   points doubled, because the per-seed variance estimate is itself
   heavy-tailed. To close it: restate the falsifier as a distribution statement
   (e.g. the geometric-mean ratio, or a quantile of the per-seed ratios, against
   1/2), measured here as ×0.39.
3. **ulp-level drift of the α survey between `main` and B1's tree.** The same
   seed's α vector differs in the last digit on 507 of 615 channels (seed 11),
   and the bbx 600k σ for seed 11 moves 2.5e-4 relative. This is benign per the
   ULP policy, but it voids trace replays across trees for every ladder session.
   To close it: banked ladder references should record the tree, and sessions
   should rerun the control rather than replay (as done here).
4. **`ChannelFloor` is reachable only from `FixedBeamIntegrand`.** The CLI
   (`integrate --fixed-budget`), `ProtonIntegrand` and `multiplicity.rs` use the
   default. If the user wants the rule usable outside the ladder probe, it needs
   a CLI flag and the other integrands' `_with_floor` entry points.
