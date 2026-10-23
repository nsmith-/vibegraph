---
type: Session Report
title: "P1 report: stop_scale no longer inflates on the 2 → 6 rows"
description: "Re-measured under pooled_scale on uux/bbx_to_ccx_emmm_qcd0, five seeds each: no kept channel-iteration holds fewer than 49 accepted points, scaled_rel/achieved_rel is ×1.01–×1.12, a 1 % target fires at min_iters on every seed. No fix; the item closes on the measurement. A 0.2 % target is limited by the quoted error itself, not by the factor."
status: draft
tags: [budget, stop-rule, target-rel, two-to-six, re-measure, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-17, shared with two sibling sessions running heavy jobs (load 5–7)"
  commit: "d2a3661 plus throwaway, uncommitted per-iteration trace instrumentation (see Method)"
  command: "see each section; logs and traces under the session scratchpad, prefixed twosix-P1-"
sources:
  - {id: brief, resource: "P1.md", title: "P1 brief"}
  - {id: item, resource: "../../../backlog/performance/stop-scale-inflated-by-few-accepted-iterations.md", title: "stop-scale-inflated-by-few-accepted-iterations"}
  - {id: budget, resource: "../../../../../vibegraph-lib/src/budget.rs", title: "ChannelHistory::stop_scale / pooled_scale"}
---
## Verdict

The mechanism does not bite under the current sampler, so there is no fix.
`stop-scale-inflated-by-few-accepted-iterations` closes on this measurement
(brief step 1: "If the mechanism no longer bites (the factor stays O(1)), stop
here and report").

On both 2 → 6 rows, five seeds each, at the gate's configuration
(600 000 × 8, α survey 30 000 × 6, `ByAlpha`, unweighted combination):

- **No kept channel-iteration holds 1–3 accepted points.** None holds zero, and
  none holds ten or fewer. The smallest per-seed minima are 49–78 (uux) and
  51–72 (bbx). The note-34 premise (iterations of one to three accepted points,
  5–14 accepted/channel/iteration after the floor) no longer describes these
  rows.
- **`scaled_rel/achieved_rel` is ×1.03–×1.12 (uux) and ×1.01–×1.11 (bbx)** at
  8 iterations. At 12 iterations it is ×1.02–×1.11 and ×1.00–×1.12. The note-34
  figure was ×3.4–×758. The largest per-channel `pooled_scale` on any seed is
  4.77, and the channels that carry the widening hold hundreds to tens of
  thousands of accepted points an iteration, so the remaining factor is not
  driven by few-accepted iterations.
- **A 1 % target fires at `min_iters` (iteration 6) on all ten seeds.**
- **A 0.2 % target fires on 5/5 bbx seeds (iterations 6–11) and on 1/5 uux
  seeds by iteration 12.** Where it does not fire, the *quoted* error is above
  0.2 % (2.26e-3–3.44e-3 at 12 iterations), and the factor contributes only
  ×1.02–×1.11. What keeps it from firing is the budget, not `stop_scale`.

## Method

`ChannelHistory` keeps no per-iteration accepted count, so the measurement used
throwaway instrumentation that is **not committed**. It is saved as
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/twosix-P1-trace-instrumentation.patch`:

- an env-gated (`VG_TRACE`) dump in `integrate_channels` of every channel's
  `(integral, variance, neval, accepted, α)` per iteration, with a kept flag;
- a probe `probe_tmp_stop_scale_trace` that runs `integrate_with` (the gate's
  path: survey and integration both seeded by the seed, `Budget::Fixed`,
  `ByAlpha`) for `VG_ROW`/`VG_NEVAL`/`VG_NITER`/`VG_SEEDS`.

An offline replay
(`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/twosix-P1-scripts/analyze.py`
and `summary.py`) re-implements `combine_kept`, `pooled_scale` and
`scaled_rel` and evaluates them at every iteration count. It reads the stop
decision of a `Budget::Target` at `min_iters = 6` (the CLI default) off the
same draws. With `ByAlpha` the allocation does not depend on the budget kind,
so a Target run draws the same points.

**Replay checked against the run's own output.** The replay's σ and quoted
error at the last iteration equal each run's `RESULT` line. Examples: uux seed
11 at 12 iterations, replay 7.17467e-07 against `RESULT … sigma 7.174666e-7`;
bbx seed 33, 1.68980e-06 against `1.689797e-6`; uux seed 20260719 at 8, quoted
rel 2.3898e-03 against `err 1.713e-9 / 7.169021e-7`.

Seeds 11, 22, 33 and 44 of uux and all bbx seeds ran 12 iterations. The
8-iteration figures are read from their first 8, which `Budget::Fixed` draws
identically whatever `niter` is: `niter` enters only `stop()` and the progress
total. That identity is argued from the code, not measured. Seed 20260719 of
uux ran 8 iterations (it was the cost calibration).

Commands (in `/home/user/wt/two-to-six-gate`, binary built with
`cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma --no-run`
under an untracked `.cargo/config.toml` setting `profile.release-debug.debug = 0`,
since removed):

```
VG_TRACE=$SP/twosix-P1-trace-uux-cal.txt VG_ROW=uux_to_ccx_emmm_qcd0 VG_NEVAL=600000 VG_NITER=8  VG_SEEDS=20260719 setsid -w $B --ignored --nocapture --exact probe_tmp_stop_scale_trace
VG_TRACE=$SP/twosix-P1-trace-uux.txt     VG_ROW=uux_to_ccx_emmm_qcd0 VG_NEVAL=600000 VG_NITER=12 VG_SEEDS=11,22,33,44 …
VG_TRACE=$SP/twosix-P1-trace-bbx.txt     VG_ROW=bbx_to_ccx_emmm_qcd0 VG_NEVAL=600000 VG_NITER=12 VG_SEEDS=20260719,11,22 …   (then 33,44)
python3 analyze.py <trace> 0 8          # per-seed accepted counts, factors, firing
python3 summary.py <ref> <ref_err> <traces…>   # seed table, target stops
```

Wall per seed: uux 883 s (8 iterations), 952–1110 s (12); bbx 1249–2165 s
(12), on a contended host.

## Measurements

### Accepted points per kept channel-iteration (first 8 iterations)

`awk '$4==1 && $3<8 …' <trace>`:

| row | seeds | kept channel-iterations | 0 | 1–3 | ≤ 10 | < 100 | < 200 | < 512 | min |
|---|---|---|---|---|---|---|---|---|---|
| uux | 5 | 17 370 | 0 | 0 | 0 | 57 | 1 887 | 8 142 | 49 |
| bbx | 5 | 18 450 | 0 | 0 | 0 | 60 | 1 989 | 9 170 | 51 |

Per seed, `analyze.py … 0 8`: uux minima 49 / 76 / 66 / 78 / 64 (seeds
20260719 / 11 / 22 / 33 / 44), median 522–549; bbx minima 56 / 67 / 62 / 51 /
72, median 489–556.

### Stop factor and quoted error (`summary.py`)

At 8 iterations (the gate's configuration):

| row | seed | σ (pb) | rel vs MG | quoted rel | scaled rel | factor |
|---|---|---|---|---|---|---|
| uux | 11 | 7.18071e-07 | +5.31e-03 | 4.062e-03 | 4.563e-03 | ×1.12 |
| uux | 22 | 7.16429e-07 | +3.01e-03 | 4.717e-03 | 4.857e-03 | ×1.03 |
| uux | 33 | 7.17194e-07 | +4.08e-03 | 3.163e-03 | 3.249e-03 | ×1.03 |
| uux | 44 | 7.16269e-07 | +2.78e-03 | 2.115e-03 | 2.361e-03 | ×1.12 |
| uux | 20260719 | 7.16902e-07 | +3.67e-03 | 2.390e-03 | 2.548e-03 | ×1.07 |
| bbx | 11 | 1.68625e-06 | +2.23e-03 | 2.469e-03 | 2.547e-03 | ×1.03 |
| bbx | 22 | 1.68806e-06 | +3.31e-03 | 2.273e-03 | 2.302e-03 | ×1.01 |
| bbx | 33 | 1.69040e-06 | +4.69e-03 | 2.730e-03 | 2.822e-03 | ×1.03 |
| bbx | 44 | 1.68568e-06 | +1.89e-03 | 1.416e-03 | 1.576e-03 | ×1.11 |
| bbx | 20260719 | 1.69108e-06 | +5.10e-03 | 2.179e-03 | 2.220e-03 | ×1.02 |

Seed summaries (unweighted mean, seed sd, χ²/dof of the seeds against their
quoted errors):

| row | iterations | seeds | mean rel vs MG | pull | seed sd/σ | mean quoted rel | χ²/dof |
|---|---|---|---|---|---|---|---|
| uux | 8 | 5 | +3.77e-03 | +1.23 | 1.00e-03 | 3.29e-03 | 0.10 |
| uux | 12 | 4 | +2.55e-03 | +0.80 | 1.86e-03 | 2.57e-03 | 0.36 |
| bbx | 8 | 5 | +3.44e-03 | +1.16 | 1.43e-03 | 2.21e-03 | 0.55 |
| bbx | 12 | 5 | +3.61e-03 | +1.20 | 1.69e-03 | 1.92e-03 | 0.66 |

MG references: uux 7.1428e-07 ± 2.167e-09 pb, bbx 1.6825e-06 ± 4.8978e-09 pb
(`validation/madgraph/sigma_reference.json`).

### Where a target would stop (`summary.py`, `min_iters` 6)

- **1 %:** every seed stops at iteration 6. uux rel +0.08 % to +0.52 %; bbx
  +0.06 % to +0.60 %.
- **0.2 %, bbx:** seeds 33 and 44 stop at 6, 20260719 at 9, 22 at 10, 11 at 11.
  Quoted rel at the stop is 1.56e-3–1.95e-3, and rel vs MG +0.06 % to +0.47 %.
- **0.2 %, uux:** seed 44 stops at 12 (quoted 1.815e-3, scaled 1.993e-3).
  Seeds 11, 22 and 33 are not met by 12, with quoted 2.78e-3 / 3.44e-3 /
  2.26e-3. Seed 20260719 is not met by 8 (it ran only 8).

## Reading

- The few-accepted-iteration mechanism is gone on these rows. Every kept
  channel-iteration carries ≥ 49 accepted points, and the stop factor is a
  ≤ 12 % widening. Re-checking against the item's closing condition: "if it
  persists, stop_scale qualifies few-accepted-point iterations". It does not
  persist, so no qualification was written. A minimum-accepted qualification at
  any threshold up to 48 would change nothing on these runs, since no kept
  iteration falls under it.
- The note-34 numbers (S3 close-out: 5–14 accepted/channel/iteration after the
  floor) were taken before PRs #6, #7 and #14 changed the channel set, the maps
  and the stop rule. Which of those moved the minimum from ~5 to ~50 was not
  attributed. It is not needed for the verdict, which is about the current tree.
- About half of all kept channel-iterations (8 142/17 370 and 9 170/18 450) hold
  fewer than 512 accepted points: the acceptance cap
  (`MAX_FLOOR_ACCEPTANCE_SCALE` = 4) binds on them. That is the documented,
  reported behaviour of the cap, not this item.
- **The quoted error is, if anything, pessimistic on uux.** The seed spread is
  1.0e-3 against a mean quoted 3.3e-3 at 8 iterations (χ²/dof 0.10), and 1.9e-3
  against 2.6e-3 at 12 (0.36). On bbx it is 0.55–0.66. A χ²/dof this far below 1
  is a warning by AGENTS.md's rule, not a comfort. Five seeds cannot calibrate it
  (two to five times understated spreads have been seen at five seeds), and it is
  V1's to read at 20+ seeds.
- **Heavy-tail arrivals are visible in the running quoted error, and a target
  can stop just before one.** bbx seed 20260719's quoted rel runs
  1.95e-3 → 3.37e-3 between iterations 9 and 10, and a 0.2 % target stops it at
  9. On uux seed 33 it runs 2.25e-3 → 3.51e-3 between 6 and 7. This is the
  2 → 6 heavy tail that `two-to-six-integrals-not-enforced` owns, not
  `stop_scale`. `pooled_scale` is ×1.00–×1.02 at those iterations, because the
  tail point inflates the iteration's own σᵢ along with its integral.

## Validation

No production code changed: the instrumentation was reverted
(`git diff --stat` empty before the report commit). The only commit adds this
report, so no gate was re-run. Nothing was banked and no MadGraph or MadEvent
run was made.

## Brief corrections

- The brief's premise, "as measured, iterations holding 1–3 accepted points",
  and the target figure "×3.4–×758 after the floor" are stale for the current
  tree. Measured minima are 49–78 and factors ×1.00–×1.12.
- The host line ("~28 GB free disk, shared with one sibling") was superseded by
  the dispatch message (13 GB, two siblings). The work used 1.2 GB of `target/`.

## For V1 (the 2 → 6 budget ladder)

- `stop_scale` will not distort a ladder or a target run on these rows: treat it
  as ×1.0–×1.1.
- At 600 000 × 8 both five-seed means sit +0.34 % (bbx) to +0.38 % (uux) above
  the MadEvent banks (pull +1.2 each), with the seed spread 0.10–0.14 %. This is
  the same sign on both rows and both iteration counts, and matches the
  look-elsewhere flag on the references that the 20-seed MadEvent confirmation
  addresses.
- A 12-iteration run costs ~1 000 s (uux) and ~1 250–2 150 s (bbx) a seed on
  this 4-core host under load. Twenty seeds of one rung of one row is 6–12 h
  here.
- The trace patch above is the cheap way to get per-channel, per-iteration
  accepted counts and running errors from a ladder run if V1 needs them.

## Found

1. **The reported χ²/dof of a wide multichannel run is the per-iteration-own-σ
   statistic the stop rule abandoned.** The run's `RESULT` lines report χ²/dof
   of 207 188, 312 072, 1 256, 3 736 and 10 076 (uux) and 611, 1 885, 4 711,
   850 and 56 840 (bbx), while the seeds scatter at χ²/dof 0.10–0.66.
   `combine_channels` (`vibegraph-lib/src/hadronic.rs`) averages each channel's
   `combine_kept` χ²/dof, formed against each iteration's own σᵢ. The module
   documentation of `budget.rs` explains why that statistic is meaningless under
   the Pareto-tail weights. Anything that prints or reads the total's
   `chi2_per_dof` on these rows (the σ probes, the gate's own report line)
   carries a number that cannot be read. To close it: report the pooled
   `emp/quoted` statistic, or the quantile of per-channel `pooled_scale`, in
   place of or beside it, and check which consumers read the field.
2. **Per-iteration accepted counts are not observable without patching.**
   `ConvergenceReport` carries `min_channel_accepted` and per-channel totals, but
   not per-channel, per-kept-iteration counts, `pooled_scale` values or the
   running `scaled_rel`. Every re-measurement of this kind needs the throwaway
   patch. To close it: a reported, test-only or debug-level per-channel stop
   record (kept `(I, σ², n, accepted)` and the factor), so the next audit is a
   probe run, not a patch.
