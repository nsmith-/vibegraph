---
type: Session Report
title: "B1 report: one floor-bound channel carries bbx's seed spread, and the ladder's budget never reaches it"
description: "bbx_to_ccx_emmm_qcd0's 1.2M sd and χ²/dof come from single points in channel 551 (a b-exchange spine channel, α ~1e-4, floor-allocated). Neutralising that channel alone gives sd 6.19e-9 → 3.98e-9 and χ²/dof 1.05 / 1.03. 92 % of channels are bit-identical between the rungs, so their tail variance cannot shrink with neval. On main (#48, #51) the channel-551 tails are gone: 600k sd 2.69e-9, χ²/dof 0.79. A rarer single point remains, in channel 613 at 1.2M, so the rung reads sd 3.34e-9 and χ²/dof 1.44."
status: draft
tags: [budget, ladder, two-to-six, heavy-tail, multichannel, floor, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-22/23, no sibling jobs (load 3–4 from this session's own run)"
  commit: "sprint: bda18b1 (code = 784582d) plus throwaway, uncommitted trace instrumentation; main: local unpushed merge e5a09a8 = 784582d + origin/main c0e5907, same instrumentation"
  command: "see Method; logs, TSVs, traces and scripts under the session scratchpad, prefixed t26-B1-"
sources:
  - {id: brief, resource: "B1.md", title: "B1 brief"}
  - {id: p1, resource: "P1-report.md", title: "P1 report"}
  - {id: f1, resource: "F1-report.md", title: "F1 report"}
  - {id: budget, resource: "../../../../../vibegraph-lib/src/budget.rs", title: "integrate_channels, floor_for_acceptance"}
---
## Verdict

- **The spread does not grow with budget; one channel's single points
  dominate it at both rungs.** On `784582d`, channel 551 alone takes the
  1.2M rung from sd 3.98e-9 to 1.00e-8 and its χ²/dof from 1.03 to 2.97. At
  600k it takes the sd from 6.19e-9 to 7.50e-9 and the χ²/dof from 1.05 to 1.50.
  The test replaces channel 551's term by its across-seed median and leaves
  every other channel as drawn. With 551 neutralised the sd falls ×0.64 from
  600k to 1.2M, against 1/√2 = 0.71 expected.
- **Outlier seeds.** At 600k they are 88, 99 and 110 (+2.4, +2.3 and +1.4 seed
  sd). At 1.2M it is 154 (+3.9 seed sd), which carries 79 % of that rung's
  variance, and 88 stays high (+0.7). Seed 88 is high at both rungs because its
  channel-551 history is bit-identical at both. Seeds 99 and 110 drop at 1.2M,
  because their tail points were in channels whose share grows with neval. Seed
  154 is new at 1.2M: there channel 551's iteration-0 share (633) crosses the
  512 floor, so its draws differ from 600k.
- **The χ²/dof is carried by the quiet seeds, not by the outliers.** The seeds
  holding a tail point quote errors as large as their displacement, with
  (s − m)/err of +0.4 to +1.2. The χ² comes from the eleven seeds with the
  smallest quoted errors, which sit below the tail-raised mean:
  - 600k: their mean deficit is −4.0e-9 at mean quoted error 3.4e-9, giving
    1.20 of the 1.50.
  - 1.2M: the deficit is −4.5e-9 at 2.7e-9, giving 2.78 of the 2.97.

  Doubling neval shrinks the quiet seeds' errors (×0.79). Their deficit from the
  mean does not shrink, because the channel that sets it gets no more points.
- **Mechanism: the ladder's extra budget does not reach the channels that carry
  the tail.** Under `ByAlpha`, every channel with α·neval below its floor gets
  the floor at both rungs. That floor is 512 at iteration 0, then 512/acceptance
  capped at 2048. Its draws then depend only on its own stream and acceptance,
  so its history is bit-identical at 600k and 1.2M for a given seed. That holds
  for 551–585 of the 615 channels on every seed. Channel 551 is floor-bound at
  both rungs on 14 of 20 seeds. Its α ranges from 1.6e-7 to 2.1e-2 across seeds, so the 30k × 6
  α survey does not measure it. A single point in it moves σ by up to +1.1 %
  (seed 88) or +2.6 % (seed 154).

  This is the AGENTS.md "migrates between seeds instead of shrinking" signature
  in kind: the tail moves between seeds (99, 110 → 154) and does not shrink.
  Here that comes from the allocation never giving the tail more points, not
  from an error in an amplitude or a map.
- **Falsifier.** If channel 551's points are made to scale with neval, its
  across-seed variance must fall in proportion to its points. The other channels
  must leave the 1.2M sd near 4e-9 with χ²/dof near 1. A 1.2M rung that stays at
  sd ≈ 1e-8 with χ²/dof ≈ 3 under that allocation refutes the mechanism. A
  third rung under the current allocation is the cheaper half-test. It must
  leave channel 551's per-seed histories bit-identical to the 600k ones wherever
  α·neval stays under the floor, and the quiet seeds' χ² share must grow by
  about ×2 again.
- **On main** (`784582d` + `origin/main` `c0e5907`, which brings #48 and #51),
  the effect is about three times smaller but has the same signature:
  - 600k: sd 2.69e-9, χ²/dof 0.79.
  - 1.2M: sd 3.34e-9, χ²/dof 1.44.

  Channel 551's largest single point over all 40 main runs is 2.1e-10 pb,
  against 4.3e-8 on the sprint tree. Channel 564's is 4.2e-10, against 1.6e-8.

  The 1.2M rung's sd excess is again one seed: seed 55, one point of
  +1.2e-8 pb (0.7 % of σ) in channel 613, another b-exchange spine channel.
  Dropping that seed gives sd 2.49e-9 and χ²/dof 1.25. Channel 613 is
  α-scaled, not floor-bound, so this hit is not the shared-draw mechanism.

  The structural part persists. 563–580 of 615 channels are bit-identical
  between the rungs. Without seed 55 the sd still falls only ×0.93, against
  1/√2. Whether #48 or #51 removed the channel-551 tail is not attributed.

## Step 1: reproduction on `784582d`

20 seeds (20260719, 11, 22 … 209), multi, 8 iterations, bbx only, tree
`bda18b1` (code = `784582d`) plus the trace instrumentation (Method), which
changes no draw.

| rung | source | σ (pb) | quoted | spread/√n | per-seed sd | χ²/dof | s/seed |
|---|---|---|---|---|---|--:|--:|
| 600k | M3 (PR #47, 14:04 UTC) | 1.691150e-6 | 1.791e-9 | 1.663e-9 | 7.44e-9 | 1.47 | 91 |
| 600k | this host | 1.691138e-6 | 1.791e-9 | 1.676e-9 | 7.50e-9 | 1.50 | 462 |
| 1.2M | M3 | 1.692920e-6 | 2.482e-9 | 2.240e-9 | 1.00e-8 | 3.09 | 134 |
| 1.2M | this host | 1.693070e-6 | 2.483e-9 | 2.244e-9 | 1.00e-8 | 2.97 | 699 |

The run's own summary lines are from `t26-B1-sprint.log`:

```
multichannel    600000 x 8 (20 seeds, 9232 s): σ = 1.691138e-6 ± 1.791e-9 pb (quoted 1.791e-9, spread/√n 1.676e-9) | χ²/dof 1.50 | pull vs seed record +0.38 | pull vs single run +1.66 | rel vs single run +5.13e-3
multichannel   1200000 x 8 (20 seeds, 13975 s): σ = 1.693070e-6 ± 2.483e-9 pb (quoted 2.483e-9, spread/√n 2.244e-9) | χ²/dof 2.97 | pull vs seed record +0.88 | pull vs single run +1.92 | rel vs single run +6.28e-3
multichannel 600000 -> 1200000: Δmean +1.932e-9 pb (+1.14e-3 rel) over √(s₁²/n₁ + s₂²/n₂) = 2.801e-9 pb: +0.69σ (information)
```

**They match in statistics, not in bits.** The quoted errors agree to four
digits. The means differ by 1.2e-11 (600k) and 1.5e-10 (1.2M), and the
χ²/dof by 0.03 and 0.12. The M3's per-seed TSV is not on this host, so I
cannot say which seeds differ. On this host, seeds 20260719 and 11 at 600k
reproduce P1's 8-iteration replay on `d2a3661` digit for digit (1.69108e-6 and
1.68625e-6), and the replay of every seed from the trace matches its TSV line
to ≤ 3e-10 relative. The M3 differences are at the size a different libm
leaves after eight grid refinements of a heavy-tailed integrand. That is
plausible but not verified.

## Step 2: the outliers

`report.py t26-B1-sprint-trace.txt t26-B1-sprint.tsv 0`. Columns: σ, the seed's
quoted error, the pull against the rung mean in seed sd and in its own quoted
error, and its χ²/dof term (`((s − m)/err)²/(n − 1)`).

| seed | 600k σ | err | /sd | /err | χ² term | 1.2M σ | err | /sd | /err | χ² term |
|---|---|---|--:|--:|--:|---|---|--:|--:|--:|
| 20260719 | 1.691077e-6 | 3.69e-9 | −0.01 | −0.02 | 0.000 | 1.693548e-6 | 4.31e-9 | +0.05 | +0.11 | 0.001 |
| 11 | 1.686250e-6 | 4.16e-9 | −0.65 | −1.17 | 0.073 | 1.698335e-6 | 6.83e-9 | +0.52 | +0.77 | 0.031 |
| 22 | 1.688060e-6 | 3.84e-9 | −0.41 | −0.80 | 0.034 | 1.687214e-6 | 2.13e-9 | −0.58 | −2.76 | 0.400 |
| 33 | 1.690395e-6 | 4.62e-9 | −0.10 | −0.16 | 0.001 | 1.695638e-6 | 4.89e-9 | +0.26 | +0.53 | 0.015 |
| 44 | 1.685683e-6 | 2.39e-9 | −0.73 | −2.28 | 0.275 | 1.692762e-6 | 2.33e-9 | −0.03 | −0.13 | 0.001 |
| 55 | 1.684051e-6 | 3.77e-9 | −0.95 | −1.88 | 0.186 | 1.686769e-6 | 2.32e-9 | −0.63 | −2.72 | 0.388 |
| 66 | 1.689218e-6 | 3.98e-9 | −0.26 | −0.48 | 0.012 | 1.688602e-6 | 3.61e-9 | −0.45 | −1.24 | 0.081 |
| 77 | 1.686988e-6 | 3.53e-9 | −0.55 | −1.18 | 0.073 | 1.694113e-6 | 7.64e-9 | +0.10 | +0.14 | 0.001 |
| **88** | **1.708930e-6** | 1.92e-8 | **+2.37** | +0.93 | 0.045 | **1.700057e-6** | 1.80e-8 | +0.70 | +0.39 | 0.008 |
| **99** | **1.708465e-6** | 1.41e-8 | **+2.31** | +1.23 | 0.079 | 1.683893e-6 | 3.49e-9 | −0.91 | −2.63 | 0.364 |
| **110** | **1.701786e-6** | 1.67e-8 | **+1.42** | +0.64 | 0.021 | 1.693075e-6 | 5.91e-9 | +0.00 | +0.00 | 0.000 |
| 121 | 1.687042e-6 | 5.49e-9 | −0.55 | −0.75 | 0.029 | 1.691537e-6 | 3.57e-9 | −0.15 | −0.43 | 0.010 |
| 132 | 1.687322e-6 | 3.09e-9 | −0.51 | −1.24 | 0.080 | 1.688518e-6 | 2.60e-9 | −0.45 | −1.75 | 0.162 |
| 143 | 1.684024e-6 | 3.03e-9 | −0.95 | −2.35 | 0.290 | 1.692080e-6 | 6.81e-9 | −0.10 | −0.15 | 0.001 |
| **154** | 1.687766e-6 | 2.96e-9 | −0.45 | −1.14 | 0.068 | **1.731923e-6** | 4.26e-8 | **+3.87** | +0.91 | 0.044 |
| 165 | 1.687786e-6 | 3.66e-9 | −0.45 | −0.92 | 0.044 | 1.686415e-6 | 1.62e-9 | −0.66 | −4.10 | 0.885 |
| 176 | 1.688810e-6 | 4.00e-9 | −0.31 | −0.58 | 0.018 | 1.691372e-6 | 2.67e-9 | −0.17 | −0.64 | 0.021 |
| 187 | 1.699122e-6 | 1.04e-8 | +1.07 | +0.77 | 0.031 | 1.688215e-6 | 3.66e-9 | −0.48 | −1.33 | 0.092 |
| 198 | 1.686354e-6 | 3.01e-9 | −0.64 | −1.59 | 0.133 | 1.690690e-6 | 2.69e-9 | −0.24 | −0.89 | 0.041 |
| 209 | 1.693630e-6 | 1.09e-8 | +0.33 | +0.23 | 0.003 | 1.686639e-6 | 2.25e-9 | −0.64 | −2.86 | 0.430 |

- **What carries the sd.** The high seeds carry it: 88, 99 and 110 at 600k,
  and 154 at 1.2M. Seed 154 alone is (3.885e-8)²/19 = 7.9e-17 of the rung's
  variance of 1.01e-16. Dropping it gives sd 4.25e-9 and χ²/dof 1.56, and
  dropping 88 at 600k gives 6.39e-9 and 1.09 (`arith.py`).
- **What carries the χ²/dof.** The low seeds with small quoted errors carry
  it, not the outliers. At 1.2M these are 165 (0.885), 209, 22, 55 and 99
  (0.36–0.43 each). At 600k they are 143, 44 and 55.
- **Are the same seeds high at both rungs?** Only seed 88 (see step 3 for
  why). Seed-level correlation between the rungs is −0.08.

## Step 3: the channels

The across-seed per-channel variance is not usable for attribution. The α
vector is surveyed per seed, so channel terms trade σ between them from seed to
seed: summed per-channel across-seed variance is 1.9e-14 against a total of
6e-17. Attribution is done within each seed instead (`tails.py`). For each
channel, *shift* is its combined term minus the median of its own kept
iterations, which is how far its outlying iterations moved it. *wshare* is the
largest single point's contribution, w/(n·k).

Each cell gives: channel, shift (pb), single-point share (pb), pooled stop
scale, iteration of the top point, and accepted points per kept iteration.

| rung | seed | s − m | Σ shift | top channels |
|---|---|---|---|---|
| 600k | 88 | +1.81e-8 | +2.98e-8 | **551**: +1.8e-8, 1.8e-8, ×1.0, it 7, 253 acc. 362: +5.1e-9. 346: +3.4e-9 |
| 600k | 99 | +1.77e-8 | +2.80e-8 | 574: +8.4e-9, 9.6e-9, ×1.0, it 6, 13 776 acc. 572: +8.2e-9, 628 acc |
| 600k | 110 | +1.10e-8 | +2.26e-8 | **564**: +1.6e-8, 1.6e-8, ×1.0, it 3, 409 acc |
| 1.2M | 154 | +3.89e-8 | +4.63e-8 | **551**: +4.2e-8, 4.3e-8, ×1.0, it 7, 322 acc |
| 1.2M | 88 | +6.99e-9 | +2.34e-8 | **551**: +1.8e-8, identical to 600k |
| 1.2M | 11 | +5.27e-9 | +1.49e-8 | 604: +6.0e-9, 6.1e-9, ×1.0, it 7, 5 164 acc |
| typical (1.2M 176, 198, 66) | | −1.7e-9 to −4.5e-9 | +6.4e-9 to +8.3e-9 | no channel above +3.4e-9 |
| quiet (1.2M 165, 209) | | −6.7e-9, −6.4e-9 | +2.7e-9, +2.9e-9 | no channel above +4.9e-10 |

In every outlier the excess is one point: wshare ≈ shift. Its iteration
quotes the point in its own σᵢ, so the channel's pooled stop scale is ×1.0
(P1 saw the same).

**Channel 551** (`chan551.py`, every seed and rung; `probe_tmp_describe_channels`
for its map):

- Its map is a peripheral (spine) channel with one t-channel b exchange (pole
  m_b² = 22.09 GeV²) and four s-channel structures, Z, γ*, γ* and Z.
- α is 1.6e-7 to 2.1e-2 across the 20 seeds.
- On 14 of 20 seeds it is floor-bound at both rungs: 512 points in iteration 0
  and 2048 (the acceptance cap) after, with 236–407 accepted per kept
  iteration. Its history is then bit-identical between the rungs.
- Its usual term is 2e-11 to 6e-10 pb. In every seed the largest weight of an
  iteration is 450–2 000 times that iteration's mean weight, so even its quiet
  estimates are single-point estimates.
- Seed 88 (both rungs) and seed 154 (1.2M) drew a point worth 1.8e-8 and
  4.3e-8 pb: 1.1 % and 2.6 % of σ.

Channel 564, the 600k seed 110 outlier, has the same structure, with H, Z, Z
and Z on the spine. So do the smaller tail channels 369, 393, 604 and 608,
each with one or two t-channel b lines. Which region of the b-exchange channel
the tail points sit in (the γ* poles at the mmll cut, the t-channel
b-propagator, or a corner of a Z) is not measured: the trace does not carry
the point's kinematics (Found 2).

**Stop scale at both rungs** (`stopscale.py`; P1 had 600k over five seeds):

- 600k: scaled_rel/achieved_rel is ×1.00–×1.15 (median ×1.03).
- 1.2M: ×1.00–×1.35 (median ×1.04). The largest are seed 121 (×1.35) and seed
  22 (×1.22).
- The largest per-channel pooled scale on any seed is 5.80.
- The smallest accepted count in a kept channel-iteration is 47.
- On the outlier seeds (88, 99, 110 and 154) the factor is ×1.00, because the
  tail point inflates its iteration's own σᵢ.

The stop scale plays no part in this. P1's verdict stands at 1.2M and 20 seeds.

## Step 4: mechanism, checked on the dumps

1. **Common random numbers make most of the run budget-independent.** Each
   channel draws from its own stream `CHANNEL_STREAM_BASE + j` at its own
   generator position, with its own grid. Under `ByAlpha` its allocation is
   `max(α·neval, floor)`, and the floor depends only on its own acceptance
   (`allocate_iteration`, `floor_for_acceptance`). A channel with α·neval
   under 512 at both rungs therefore draws the same points at both, iteration
   by iteration. Measured (`tails.py`, cross-rung check): 551–585 of 615
   channels are bit-identical per seed (`(I, V, n)` of every iteration equal).
   They hold only 2.8e-9 to 2.4e-8 pb of σ. The across-seed sd of that part
   is 4.77e-9, mostly from seed 88's channel-551 point. That is as large as all
   of the 1.2M spread left once channel 551 is neutralised, and it is the same
   number at both rungs.
2. **The tail lives in such channels.** Channel 551's α (1e-4 typical) is below
   the floor at both rungs. Its per-point weight is spiky in every seed,
   and a hit is worth 1–3 % of σ. Single points of 1e-8 pb or more are rare:
   - 600k: seeds 88 (551) and 110 (564);
   - 1.2M: 88 (551, the same draw) and 154 (551).

   With that few of them, the per-rung sd counts hits. It is not a measure of a
   variance that changes with neval.
3. **The arithmetic** (`arith.py`):
   - Neutralise channel 551 (its term set to the 2.05e-10 pb across-seed
     median):
     - 600k: sd 7.50e-9 → 6.19e-9, χ²/dof 1.50 → 1.05.
     - 1.2M: sd 1.00e-8 → 3.98e-9, χ²/dof 2.97 → 1.03.
     - The ratio of the two sds is then 0.64, against 1/√2.
   - The quiet seeds' χ² share is 1.20 at 600k and 2.78 at 1.2M. That growth is
     the ratio of their errors squared, (3.36/2.66)² = 1.6, times the change in
     their deficit, (4.49/4.02)² = 1.25: 1.20 × 1.6 × 1.25 = 2.4 against the
     measured 2.78. The rest is seed 99's switch from high to quiet.
   - The neutralised mean moves by −1.1e-9 (600k) and −3.2e-9 (1.2M). That is
     the tail's contribution to the mean, and not a bias to remove: the
     points are legitimate weights of the integrand.
4. **What it is not:**
   - It is not the stop rule: ×1.00 on the outliers, ×1.04 median.
   - It is not starvation by accepted count: the minimum is 47, and channel 551
     holds 236–407 accepted per iteration.
   - It is not the VEGAS bin width either. The tail point appears in a late
     iteration (7, 7, 3) after the grid has refined, and its seed-level
     occurrence follows the channel's draws, not the rung. The step 5 run tests
     the bin-width bound directly.

**Fix (proposed, not implemented).** The variance has to be bought where it
lives:

- **(a)** Run under `BlockAllocation::Neyman`, which allocates by measured
  per-point sd. Channel 551's point sd is large relative to its α in every
  seed (its iterations are single-point estimates), so Neyman should move
  points to it. This is a one-line probe change (`integrate_with` takes no
  allocation today) and is the cheapest test of the falsifier above.
- **(b)** Make the floor scale with neval, so that a rung is a budget for
  every channel.
- **(c)** Raise the α survey's resolution for channels whose α is noise.
  Channel 551's α spans five decades over 20 seeds.

(a) is the natural next session. (c) alone cannot work while a hit is a
single point.

## Step 5: on current main

`/home/user/wt/t26-B1-main` was a detached worktree at a local, unpushed merge
`e5a09a8` (`784582d` + `origin/main` `c0e5907`, which brings #48 and #51). The
only conflict was `research/kb/sprints/index.md`, resolved to ours. Reference
data was symlinked from the sprint worktree, with the same instrumentation and
the same command. The 1.2M seeds 88 … 209 were killed by a container restart
after 28 of 40 runs. Their partial trace lines (one iteration of seed 88) were
dropped, and only those 12 seeds were rerun into the same TSV and trace, so
each seed-run appears exactly once (`uniq -d` empty). The probe's printed
1.2M summary therefore covers only the rerun 12 seeds. The full rung below is
the replay of all 20 seeds from the trace, which matches every TSV line to
≤ 3e-10 relative.

| rung | σ (pb) | quoted (combined) | spread/√n | per-seed sd | RMS quoted/seed | χ²/dof |
|---|---|---|---|---|---|--:|
| 600k | 1.689996e-6 | 6.94e-10 | 6.01e-10 | 2.69e-9 | 3.10e-9 | 0.79 |
| 1.2M | 1.691693e-6 | 8.20e-10 | 7.47e-10 | 3.34e-9 | 3.67e-9 | 1.44 |
| 1.2M without seed 55 | 1.691178e-6 | 5.90e-10 | | 2.49e-9 | | 1.25 |

The 600k line as the probe printed it:
`multichannel 600000 x 8 (20 seeds, 14373 s): σ = 1.689996e-6 ± 6.940e-10 pb (quoted 6.940e-10, spread/√n 6.013e-10) | χ²/dof 0.79 | pull vs seed record +0.02 | pull vs single run +1.52 | rel vs single run +4.46e-3`.

The 600k sd is 2.8 times smaller than on the sprint tree, and the 1.2M sd 3.0
times smaller. The mean at 600k sits on the 5-seed MadEvent record
(pull +0.02, against +0.38 on the sprint tree).

- **The channel-551 and 564 tails are gone.** Over all 40 main runs, the
  largest single-point share in channel 551 is 2.1e-10 pb (sprint 4.3e-8), and
  in 564 it is 4.2e-10 (sprint 1.6e-8). Channel 551 is still floor-bound, its
  α still spans 1.7e-6 to 1.4e-3 across seeds, and its iterations are still
  single-point estimates (mean largest/mean weight about 1 000, against
  1 500). The extreme points no longer occur in 40 runs. All three channels
  have the same composition on both trees (`probe_tmp_describe_channels` on
  each): one or two t-channel b lines with pole 22.09 GeV². Channel 613 has Z,
  Z and γ* on the spine.
- **A rarer tail remains.** Main seed 55 at 1.2M has a single point of
  1.18e-8 pb in channel 613, in iteration 3, with largest/mean weight 7 713.
  Across its other 39 runs that ratio is 41–3 543 (median 108), and their
  largest single-point share is 2.8e-10 pb. Channel 613 has α 7e-3 and is α-scaled
  (4 195 → 8 390 points an iteration), so this hit is the ordinary tail of a
  channel whose budget does grow. It is one seed. On the sprint tree, channel
  613's largest point over 40 runs was 3.3e-10.
- **The structural part persists.** 563–580 of 615 channels are bit-identical
  between the rungs on every seed. Without seed 55 the sd moves from 2.69e-9
  to 2.49e-9 (×0.93, against 1/√2 = 0.71). The quiet seeds' χ² share rises
  from 0.55 to 1.19, as their errors shrink (2.63e-9 → 1.87e-9) and their
  deficit does not (−1.66e-9 → −1.89e-9). The identical-channel part's
  across-seed sd is 1.74e-9 at both rungs.

  That part is anticorrelated with the rest (−0.64 / −0.34), because the
  per-seed α moves σ between channels. Its 1.74e-9 is therefore an upper
  reading of a budget-independent floor, not a measured one.
- **Stop scale:** ×1.01–×1.17 at 600k and ×1.00–×1.15 at 1.2M (median ×1.07
  both).
- **Cost:** about 720 s a 600k seed and 1 000–1 780 s a 1.2M seed, against 460
  and 700 s on the sprint tree, on the same host.

Which of #48 (bin-width bound) or #51 (map defaults) removed the channel-551
and 564 tails is not attributed (Found 5).

## Method

Instrumentation, not committed. The patch is
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/t26-B1-instrumentation.patch`.
It adds two things:
- a `max_weight` field on `BlockIteration`, the max |f·w| of the block's
  points, accumulated in the reduction loop;
- a `VG_TRACE`-gated per-channel, per-iteration line in `integrate_channels`:
  `seed neval_budget channel iteration keep I V n accepted alpha max_weight`.

Neither changes a draw or a sum. The first two seeds reproduce P1's digits,
and every seed's replay matches its TSV line. The patch also carries the
throwaway probe `probe_tmp_describe_channels`, which prints
`channel_samplers()[j]`. All of it was reverted before the commit
(`git diff --stat` empty).

Scratchpad `$SP = /tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad`:

```
# build (both trees)
CARGO_INCREMENTAL=0 pixi run cargo test --config 'profile.release-debug.debug=0' -p vibegraph-lib \
  --profile release-debug --features extended-validation --test validate_sigma --no-run
# run, detached (python double-fork + setsid), one tree at a time
$SP/t26-B1-run.sh sprint /home/user/wt/two-to-six-gate 600000,1200000 20
$SP/t26-B1-run.sh main   /home/user/wt/t26-B1-main     600000,1200000 20
#   = VG_TRACE=$SP/t26-B1-<tag>-trace.txt VG_2TO6_ROWS=bbx VG_2TO6_SAMPLERS=multi
#     VG_2TO6_RUNGS=600000,1200000 VG_2TO6_SEEDS=20 VG_2TO6_OUT=$SP/t26-B1-<tag>.tsv
#     <validate_sigma binary> --ignored --nocapture --exact probe_2to6_budget_ladder
# analysis
python3 -I $SP/t26-B1-scripts/report.py    $SP/t26-B1-<tag>-trace.txt $SP/t26-B1-<tag>.tsv 0
python3 -I $SP/t26-B1-scripts/tails.py     $SP/t26-B1-<tag>-trace.txt 3
python3 -I $SP/t26-B1-scripts/arith.py     $SP/t26-B1-<tag>-trace.txt
python3 -I $SP/t26-B1-scripts/stopscale.py $SP/t26-B1-<tag>-trace.txt
python3 -I $SP/t26-B1-scripts/chan551.py   $SP/t26-B1-<tag>-trace.txt 551
VG_CHANNELS=551,564,369,393,604,608,574,582 <binary> --ignored --nocapture --exact probe_tmp_describe_channels
```

Outputs are `t26-B1-{sprint,main}.{tsv,log}`, `-trace.txt`,
`t26-B1-sprint-{report,tails,arith,stopscale}.txt` and the same for main.
Sprint run: 14:46:42Z to 21:13:29Z (23 207 s). Main run: 21:13:35Z to the restart (28 seed-runs), then the 12 missing 1.2M seeds from 03:40Z to 08:29:41Z (17 322 s).

Nothing was added to the sprint's seeds: 20 a rung, the brief's set, was enough
for the diagnosis, and the falsifier needs a different allocation, not more
seeds.

## Validation

No `.rs` change is committed: the probe extension stayed throwaway, since the
diagnosis needed none of it in the tree. The commit adds this report and the
regenerated kb index only. `pixi run --skip-deps kb-lint` reports `774 concepts, 262 backlog items; 0 errors, 3 warnings`. All three warnings are pre-existing `state: blocked` backlog items.

## Brief corrections

- **"The seed spread grows with budget" is a reading of one seed.** The 1.2M
  sd of 1.00e-8 is seed 154's single point (79 % of the variance). Without it
  the rung reads 4.25e-9 against 600k's 6.39e-9 without its own worst seed. A
  20-seed sd ratio of 1.34 (variance ratio 1.81) is not significant even under
  a Gaussian F-test (p ≈ 0.1), and the per-seed values are heavy-tailed. The
  real finding is different and structural: the extra budget does not reach
  the channel that carries the variance.
- **The M3 used the RMS quoted error that the brief omits.** The M3's rows
  have it above the seed sd at both rungs: 8.0e-9 against 7.44e-9 at 600k, and
  1.11e-8 against 1.00e-8 at 1.2M. That rules out "errors underestimated across
  the board" before any run. The χ²/dof comes from uneven quoted errors, where
  seeds that hit the tail carry large ones.
- **"If the seeds are the M3's, the per-seed values should match its TSV."**
  The TSV is not on this host, and the statistics match to 1e-5 relative, not
  bit for bit (step 1).
- **The main-tree rung costs more than the sprint tree.** On this host a main seed costs about 1.5× (600k) to 2× (1.2M) a sprint seed, so a 20-seed, two-rung main ladder is about 12 h here, not the 8 h the sprint tree took.

## Found

1. **The ladder's rungs are not budgets for most channels.** On bbx, 551–585
   of 615 channels draw bit-identical points at 600k and 1.2M for the same
   seed, because their α·neval is under the floor at both. A rung-to-rung
   comparison therefore holds those channels' variance fixed and compares
   only the rest. The two rungs of one seed are also correlated, and the
   adjacent-rung line's `√(s₁²/n₁ + s₂²/n₂)` assumes they are independent.
   To close it:
   - state the fraction of the budget above the floors with each rung;
   - either draw the rungs on disjoint seeds or report the paired difference;
   - decide whether a rung should be a budget for every channel (fix (b)).
2. **The tail point cannot be located.** The trace gives the channel, the
   iteration and the weight, but not the phase-space point. Which region of
   the b-exchange spine channels (551, 564, 369, 393, 604 and 608) produces
   weights 450–2 000 times the iteration mean is not known. Neither is why
   `uux` (the same topologies, massless u exchange) shows no such spread
   (sd 2.91e-9 → 1.58e-9 on the M3). To close it: record the momenta and the
   channel densities of each channel's largest-weight point per iteration
   (test-only), then read the invariants: the γ* masses against mmll, t
   against m_b², and the Z windows.
3. **Neyman is the untested candidate fix.** `integrate_with` fixes
   `ByAlpha`. A ladder with `BlockAllocation::Neyman` on bbx, 20 seeds at 600k
   and 1.2M, is the falsifier of step 4 and the cheapest test of fix (a). To
   close it: expose the allocation in the probe and run it.
4. **The seed χ²/dof is the per-own-σ statistic one level up** (P1 Found 1 for
   iterations). With heavy-tailed seeds it reads the quiet seeds' small
   errors against a tail-raised mean: χ²/dof 2.97 here, from seeds whose errors
   are correct for what they drew. Anything gating on it (the ladder's
   summary, a seed-record χ²) inherits this. To close it: report a robust
   companion beside it, such as the median |pull| or χ² with the worst seed
   dropped, and the RMS quoted error against the seed sd.
5. **The removal of the channel-551 and 564 tails on main is not
   attributed.** The 40-run largest point fell from 4.3e-8 to 2.1e-10 pb on
   551, and from 1.6e-8 to 4.2e-10 on 564, between `784582d` and main. Main
   carries #48 (K = 6 bin-width bound) and #51 (soft-all maps, τ rule). To close
   it: rerun the bbx 600k rung, or just seeds 88 and 110, with each change
   alone. Seed 88's channel-551 point is reproducible on `784582d` and is the
   sharpest single test.
6. **Main's remaining spread still has a budget-independent part.** Without
   its one tail seed, the main 1.2M sd is ×0.93 the 600k one, not ×0.71. That
   is consistent with the floor-bound, shared-draw part of Found 1, but the
   anticorrelation that the per-seed α induces keeps it from being a
   measurement. To close it: a paired per-seed difference (1.2M − 600k), in
   which the identical channels cancel exactly, read against the two rungs'
   separate spreads.
