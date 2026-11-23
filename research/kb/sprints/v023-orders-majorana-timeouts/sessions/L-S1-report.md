---
type: Session Report
title: "L-S1 report: parallel accept/reject on seed-placed blocks"
description: "generate -j N runs accept/reject over N threads on blocks of trials placed on the seed's streams by index, so the output is fixed by the seed alone; speed-up, phase breakdown, statistical gates, the ee_to_wpwm shape demotion and the 2-to-6 validator request."
status: draft
tags: [generate, unweighting, parallelism, determinism, samples-gate, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Branch `sprintl-generate` off main d5e7c71. Commits `4b356145` (the pass and
its tests), `39f98611` (the `ee_to_wpwm` KS demotion and its census probe) and
the final commit carrying this report. The item
`two-to-six-samples-serial-unweighting-cost` is **not** closed and its file is
kept: its `closes_when` asks for both 2→6 samples cells to be run and recorded,
and at four cores they cost about an hour, so they go to the validator host
(request below).

Transformation class: **statistical**. Integration is untouched: no stream,
estimator or artifact field of `integrate` changed, so every banked σ line is
bit for bit. Generation draws from new streams, so a seed's events differ from
v0.2.2's at every `-j`. They are bit-identical across `-j` on this host.

## What changed

**The pass** (`vibegraph-lib/src/unweight/blocks.rs`, `BlockedPass`). The
trial sequence is cut into blocks of `BLOCK_TRIALS = 1024`. Block `b` opens two
counter-based substreams of the generation seed, placed the way `integrate`
places a chunk of points (`vegas.rs` `adapt_blocks_iteration`: stream fixed,
word position = first point × draws per point × `WORDS_PER_DRAW`):

- the trial stream (stream 0) at draw `b·1024·stride`, with
  `stride = 2 + max_j grid_ndim(j) + event_draws`. A trial in channel `j`
  consumes exactly `2 + grid_ndim(j)` draws (channel, grid, acceptance), and an
  accepted one a further `event_draws` (4 fixed-beam, 5 proton) for its labels.
  So no block reaches into the next block's window; a `debug_assert` checks it.
- the trailing-uniform stream (`SCALE_DRAW_STREAM_BASE`) at draw
  `b·1024·scale_draw_ndim`, exactly `integrate`'s `first * scale_ndim`.

Blocks are computed in batches of `4 × threads` on the rayon pool. Each
accepted point's momenta, labels and record scales are built on the worker that
accepted it (`FixedBeamPrepared`, `ProtonPrepared` in `generate/source.rs`).
Points are handed out in block order, each block's in trial order. The serial
consumer keeps the record assembly, reweighting, `<mgrwt>` and the write.

**Statistics.** Every reported statistic (σ̂, pull, efficiency, overweight
rate and share, `ratio_max`, the heaviest trials) is over the trials up to and
including the one that produced the last event taken. That is every earlier
block in full, merged in block order (`UnweightStats::absorb`), plus the
current block as it stood after that trial: each accepted point carries a
snapshot. Blocks past that point are speculative and reach nothing. The
max-trials guard counts trials since the last event across block boundaries.
Overweights, truncation and per-channel selection are `Unweighter::trial_with`,
the same code the serial `trial` now calls.

**Why the output cannot depend on scheduling.** A block's content is a
function of (seed, block index, frozen maxima, integrand) only. Its draws come
from stream windows fixed by its index. `prepare` must be a function of its
arguments (the integrand's per-thread scratch already carries that guarantee
for `integrate`'s bit-for-bit-at-any-pool-size claim). `collect` on an indexed
parallel iterator preserves block order. The merge order is block order. The
batch width changes only how many speculative blocks exist, never which events
or statistics are taken.

**Block size, 1024 trials.** At the cheapest trials measured (`p p > t t~`,
about 20 µs a trial on four cores including the PDF), a block is about 20 ms
against microseconds of rayon dispatch, stream seek and stats merge. At a 2→6
trial (about 0.2 ms) a block is about 0.2 s, so a batch of 16 balances at the
barrier. The speculative tail measured 0.43–6.65% of trials (table below).

**Elsewhere.** The samples, unweighting and 2→6 cost tests drive the shipped
pass. `validate_samples` gains a long-tier driver
`the_long_tier_sample_rows_match_madgraph` (`pixi run validate-samples-2to6`)
and the census probe `probe_wpwm_sampler_census`. The headroom probe reports
each row's five-seed sample-σ spread and χ²/dof. The release note is
`docs/release-notes/v0.2.3.md`.

**Correction to the brief.** The brief's premise was "only the w_max scan is
parallel". That holds for accept/reject. Two other phases are as large or
larger and are not accept/reject (see the phase table): the scan itself
(already parallel, but as costly as the whole accept/reject phase on
`p p > t t~ j`) and the systematics-weight write (the dominant phase on
`p p > t t~`).

## Oracles

**Byte identity across `-j`, hermetic and banked layers.**

```
$ cargo test -p vibegraph --test cli_generate
test the_sample_does_not_depend_on_the_thread_count ... ok
test result: ok. 18 passed; 0 failed; ... finished in 3.60s
$ cargo test -p vibegraph --features extended-validation --test cli_generate_proton -- a_hadronic_sample_does_not_depend_on_the_thread_count
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 12.43s
```

`e+ e- > mu+ mu-` (multichannel, 20000 events): `-j 1/2/4/16` give the same
file bytes, and the same reported trials, overweight and sample-σ lines.
`p p > l+ l- j` (the banked `pp_to_llj_fixed` card, NNPDF23, several flavour
groups, 3000 events): `-j 1/2/4` give the same file bytes. In the release
timing runs, `cmp` found `-j1 == -j2 == -j4` on both speed-up cards.

**Unit tests** (`unweight::blocks::tests`, all pass):

- thread-count independence at 1/2/3/4 threads, over 3000 events;
- block tiling against an independent serial replay of the stream layout;
- the merged statistics equal to a single pass over the same trials (counts
  exact, sums at 1e-12, heaviest list identical);
- restart replay;
- unbiasedness of σ and of the channel shares;
- give-up on a pass that cannot accept.

`cargo test --workspace`: EXIT 0, 68 test binaries ok. `cargo clippy
--workspace --all-targets [--features vibegraph/extended-validation] -- -D
warnings`: clean.

**Variance of the estimator, blocked against serial** (throwaway toy, 400
seeds × 2000 events, two channels, a trailing uniform). Relative seed spread:
blocked 1.85%, serial (`Unweighter::next_event` on `ChaCha8Rng`) 1.91%,
binomial `√((1−ε)/n)` 1.94% at ε = 0.248. Means 1.4990 / 1.5017 against an
exact 1.5.

**Unweighting gate, five seeds** (`cargo test -p vibegraph-lib --profile
release-debug --features extended-validation --test validate_unweighting --
--nocapture`): rc=0, 18 s.

| row | σ(events) vs VEGAS | vs weighted ref | seed spread | worst shape χ²/dof, bin pull |
|---|---|---|---|---|
| ee_to_mumu | pull −0.66, rel −0.140% | −0.82, −0.147% | 0.55% | 0.58, 1.54 |
| uux_to_uux | +0.61, +0.097% | −0.45, −0.055% | 0.32% | 0.94, 2.09 |
| gg_to_ttx | −0.71, −0.108% | −0.10, −0.015% | 0.40% | 0.51, 1.66 |
| ee_to_tatah | **−2.50**, −0.350% | +1.00, +0.101% | 0.07% | 1.13, 1.59 |
| ee_to_mumua | +0.91, +0.274% | +0.86, +0.218% | 0.30% | 1.15, 2.34 |

The tatah pull's denominator is a five-seed spread of 0.07% against the
binomial 0.3%. A throwaway 20-seed run (not committed) put the per-seed spread
at 0.31%: pull −2.01 and rel −0.311% vs VEGAS, +1.16 and +0.140% vs the
weighted reference. That is the single-pass-vs-VEGAS offset the file already
recorded (−0.358% before). The documented readings were re-derived from this
run.

**Samples gate, three seeds, after the demotion below** (`... --test
validate_samples -- --nocapture unweighted_samples_agree_with_madgraphs_banked_ones`):
`test result: ok. 1 passed`, 192 s. Before the demotion: rc=101, failing only
on `ee_to_wpwm` and `ee_to_wpwm_cw` seed `0x5a4d0003` (KS `y(w+)` p 5.742e-5
and 8.025e-5). After it, both rows still print every KS column per seed and
list the sub-floor ones under "informational rows below the floor". Their χ²
(min p 1.977e-1 and 8.610e-2), beams and scales print `gate`.

**Samples, five seeds** (`probe_samples_p_floor_headroom`), rc=0, over the 40
gating rows:

- 3-seed min p 2.667e-3 (`uux_to_ttx_4f` y(t), seed 3);
- 5-seed min p 4.880e-4 (`ud_to_epemud_qcd0` m(e+,u), seed 5);
- sample σ vs MadGraph on every gating row: |rel| ≤ 0.64%, |pull| ≤ 2.03
  (`qqx_to_o8o8_toy_dcolor`; `ee_to_mumua`, informational, +1.57%).

The seed χ²/dof came out 0.05–1.7, mostly below 1, in that first run's error
model, which was the shipped `σ√Σw²/Σw`. It drops the `1/T` term and so
overstates the error (Found). The probe now uses the exact form. The
`P_FLOOR` documentation was re-derived from this run.

**`ee_to_wpwm` census** (`probe_wpwm_sampler_census`, committed). It uses the
same grids and maxima, 20 seeds `0x5a4d0001`… under each pass. "Serial" is
v0.2.2's single generator (`Unweighter::trial` on `ChaCha8Rng::seed_from_u64`);
"blocked" is this PR's pass.

| seed | serial y | serial cos | serial pt | blocked y | blocked cos | blocked pt |
|---|---|---|---|---|---|---|
| `0x5a4d0001` | 3.082e-2 | 3.082e-2 | 2.800e-2 | 9.915e-2 | 9.915e-2 | 9.332e-2 |
| `0x5a4d0002` | 1.123e-2 | 1.123e-2 | 1.057e-2 | 1.816e-2 | 1.816e-2 | 1.682e-2 |
| `0x5a4d0003` | 1.480e-4 | 1.480e-4 | 1.425e-4 | 5.742e-5 | 5.742e-5 | 6.163e-5 |
| `0x5a4d0004` | 5.698e-2 | 5.698e-2 | 5.102e-2 | 1.101e-2 | 1.101e-2 | 1.015e-2 |
| `0x5a4d0005` | 2.090e-1 | 2.090e-1 | 1.984e-1 | 1.459e-1 | 1.459e-1 | 1.378e-1 |
| `0x5a4d0006` | 5.800e-3 | 5.800e-3 | 5.448e-3 | 6.143e-3 | 6.143e-3 | 5.165e-3 |
| `0x5a4d0007` | 1.680e-2 | 1.680e-2 | 1.331e-2 | 4.921e-2 | 4.921e-2 | 3.503e-2 |
| `0x5a4d0008` | 5.510e-3 | 5.510e-3 | 5.347e-3 | 8.757e-4 | 8.757e-4 | 8.194e-4 |
| `0x5a4d0009` | 1.223e-1 | 1.223e-1 | 9.819e-2 | 7.467e-2 | 7.467e-2 | 5.872e-2 |
| `0x5a4d000a` | 7.239e-3 | 7.239e-3 | 7.419e-3 | 1.506e-2 | 1.506e-2 | 1.541e-2 |
| `0x5a4d000b` | 1.646e-3 | 1.646e-3 | 1.498e-3 | 1.136e-3 | 1.136e-3 | 9.995e-4 |
| `0x5a4d000c` | 1.528e-2 | 1.528e-2 | 1.532e-2 | 3.994e-2 | 3.994e-2 | 3.629e-2 |
| `0x5a4d000d` | 7.109e-4 | 7.109e-4 | 6.025e-4 | 3.445e-3 | 3.445e-3 | 2.971e-3 |
| `0x5a4d000e` | 3.359e-3 | 3.359e-3 | 2.982e-3 | 6.017e-3 | 6.017e-3 | 5.686e-3 |
| `0x5a4d000f` | 7.156e-2 | 7.156e-2 | 6.567e-2 | 1.338e-1 | 1.338e-1 | 1.036e-1 |
| `0x5a4d0010` | 7.501e-2 | 7.501e-2 | 5.659e-2 | 7.522e-2 | 7.522e-2 | 6.623e-2 |
| `0x5a4d0011` | 7.372e-4 | 7.372e-4 | 8.358e-4 | 6.091e-3 | 6.091e-3 | 6.598e-3 |
| `0x5a4d0012` | 5.427e-3 | 5.427e-3 | 5.121e-3 | 3.182e-3 | 3.182e-3 | 2.995e-3 |
| `0x5a4d0013` | 3.881e-2 | 3.881e-2 | 3.703e-2 | 4.587e-2 | 4.587e-2 | 4.284e-2 |
| `0x5a4d0014` | 1.394e-3 | 1.394e-3 | 1.186e-3 | 2.485e-3 | 2.485e-3 | 2.062e-3 |

| column | serial min / median / below 0.05 | blocked min / median / below 0.05 |
|---|---|---|
| y(w+) | 1.480e-4 / 9.237e-3 / 15 of 20 | 5.742e-5 / 1.304e-2 / 15 of 20 |
| cos(w+) | 1.480e-4 / 9.237e-3 / 15 of 20 | 5.742e-5 / 1.304e-2 / 15 of 20 |
| pt(w+) | 1.425e-4 / 8.995e-3 / 15 of 20 | 6.163e-5 / 1.278e-2 / 15 of 20 |
| min over KS columns | 1.425e-4 / 8.905e-3 / 16 of 20 | 5.742e-5 / 1.260e-2 / 17 of 20 |

Under agreement these p values would be uniform. Under both passes the same
non-uniformity appears, so the disagreement is in the sample's angular shape
against MadGraph's, not in the pass. The planner approved demoting the KS
columns of `ee_to_wpwm` and `ee_to_wpwm_cw` to informational
(`KS_INFO_ROWS`, manifest notes citing
`ee-to-wpwm-angular-shape-disagrees-with-madgraph-sample`). Their χ², beams
and scales still gate, and the seeds are unchanged. Fixed-seed references
re-derived for the new stream: the `P_FLOOR` readings in `validate_samples.rs`
and the σ/shape readings in `validate_unweighting.rs`. None was loosened.

### What each test cannot detect

- The `-j` byte-identity tests cannot see a pass that is deterministic but
  wrong: wrong block order taken consistently, a biased estimator. The
  statistical gates and the toy variance A/B cover that.
- They also cannot see dependence on a factor other than the pool size, such
  as the host's libm. Cross-host bit identity is not claimed.
- `the_output_is_independent_of_the_thread_count` runs at most two batches at
  four threads, so a bug that appears only after many batches is out of its
  reach. The 20000-event CLI test spans many more.
- The block-tiling test replays the same layout arithmetic the pass uses. It
  proves the pass equals its specification, not that the specification
  leaves the stream windows disjoint. The `debug_assert` on the per-block
  draw count covers that, in debug builds only.
- The merged-statistics test compares sums at 1e-12. It cannot see a
  reassociation smaller than that, and none matters at the estimator's
  percent-level error.
- The samples and unweighting gates are distribution and σ oracles. They are
  blind to per-event label errors that leave distributions intact, and at
  three or five seeds to a bias below about 0.3% in σ.
- The census probe compares two passes against one reference. It cannot say
  which of MadGraph's sample and this crate's integrand owns the `y(w+)`
  disagreement.

## Measurements

Host: this container, 4 cores, release build (fat LTO) of `4b356145` and of
main d5e7c71 (built in a separate worktree and target). Card: the banked
`pp_to_jj` run card (13 TeV, NNPDF23 lhaid 247000, `dynamical_scale_choice =
-1`, `use_syst = True`, 145 weights an event). One `integrate -j 4
--fixed-budget --neval 200000 --niter 6 --seed 20261122` per process, reused
by every `generate --seed 7`, buffer strategy. The load average was 0.8–1.6
(only these runs) through the whole table, 21:44–21:47. An earlier table
taken while a sibling's `cargo test --workspace` ran (load 6) is discarded.

| process | events | binary | -j | wall s | user+sys s | utilisation |
|---|---|---|---|---|---|---|
| p p > t t~ (ε 0.41) | 50000 | new | 1 | 12.71 | 12.64 | 0.99 |
| | | new | 2 | 7.31 | 12.66 | 0.87 |
| | | new | 4 | 4.54 | 12.54 | 0.69 |
| | | main | 4 | 6.05 | 12.04 | 0.50 |
| p p > t t~ j (ε 0.12, 30 configs) | 20000 | new | 1 | 26.57 | 26.45 | 1.00 |
| | | new | 2 | 14.67 | 27.15 | 0.93 |
| | | new | 4 | 7.98 | 27.34 | 0.86 |
| | | main | 4 | 16.55 | 26.84 | 0.41 |

`p p > t t~ j` is the heavier card for real per-point cost: several flavour
groups, the 16-diagram `g g > t t~ g`, a clustered per-event scale. The
earlier `p p > l+ l- j` has few diagrams.

**Phases**, from an instrumented copy of the binary (wall and CPU sampled
around the scan, around the emission and around every batch; not committed).
Each instrumented run wrote the same bytes as its timed run. Wall / CPU in
seconds:

| process | -j | setup | w_max scan | accept/reject | write (records, systematics, LHE) | speculative trials |
|---|---|---|---|---|---|---|
| t t~ | 1 | 0.53 / 0.50 | 2.63 / 2.58 | 2.56 / 2.50 | 7.51 / 7.49 | 0.43% |
| | 2 | 0.29 / 0.50 | 1.48 / 2.44 | 1.21 / 2.33 | 4.21 / 6.95 | 0.43% |
| | 4 | 0.16 / 0.52 | 1.07 / 2.58 | 0.70 / 2.66 | 2.69 / 6.80 | 6.65% |
| t t~ j | 1 | 0.58 / 0.56 | 11.91 / 11.89 | 11.75 / 11.70 | 2.73 / 2.72 | 1.99% |
| | 2 | 0.37 / 0.59 | 6.94 / 12.42 | 6.17 / 11.98 | 1.78 / 2.87 | 1.99% |
| | 4 | 0.23 / 0.67 | 3.55 / 12.44 | 3.12 / 11.78 | 1.20 / 2.95 | 1.99% |

Setup wall is the remainder of the total. The accept/reject phase alone
speeds up 3.7× (t t~) and 3.77× (t t~ j) at `-j 4`. Main's accept/reject+write
at `-j 4` on t t~ j is 16.55 − 0.58 − 3.55 = 12.4 s (setup and scan are the
same code) against 4.31 s here. Taking off the shared write leaves about
11.2 s against 3.12 s, so about 3.6× on the phase itself.

**The extra CPU at `-j 4`.** On the quiet box t t~ j costs +0.9 s of CPU at
`-j 4` (+3.4%): scan +0.55, write +0.23, accept/reject +0.08. The
speculative tail is 2% of a 12 s phase. The +21% in the first table was
contention, not the pass. Nothing cheap is left to take back by block size or
look-ahead.

**2→6 pricing** (`... --test validate_sigma -- --ignored --nocapture
probe_2to6_sample_cost`), 27 min, load 1.6–3.7 (own run only):

```
[uux_to_ccx_emmm_qcd0] grids 600000x8  530.3 s | w_max scan   81.1 s | acceptance 1.154e-2 (87 trials/event, 21.7% of the 400-trial budget) | 2000 events in 10.9 s on 4 threads, so 3 x 20000 events would take 326 s of accept/reject on top of 611 s of grids and scan
[bbx_to_ccx_emmm_qcd0] grids 600000x8  740.1 s | w_max scan  118.2 s | acceptance 2.835e-3 (353 trials/event, 88.2% of the 400-trial budget) | 2000 events in 69.4 s on 4 threads, so 3 x 20000 events would take 2082 s of accept/reject on top of 858 s of grids and scan
```

The two cells are about 16 + 49 minutes here, well past the 20-minute budget,
so they were not run.


### Four-core `generate` speedup table (manager, 2026-11-23)

Measured by the manager on the head 5b546464 (`vibegraph v0.2.2-14-g5b546464`),
release profile, after vg-planner asked for release-facing scaling evidence.
**This is a 4-core table.** The -j 8 measurement is vg-planner's, on the released
binary after the tag.

**Host.** Intel Xeon Processor @ 2.10 GHz, 4 cores (1 socket, 1 thread per
core), 15 GB. Nothing else ran during the timed runs: no agents were active,
and no build or test was running.

**Setup**, reproducible from the files in
[`pr13-speedup/`](../pr13-speedup/):

- **Card A, MLM-matched, three groups.** Model `sm` (built in), proc card
  `A_proc.dat`: `generate p p > e+ e- @0`, `add process p p > e+ e- j @1`,
  `add process p p > e+ e- j j @2`. Run card
  `validation/madgraph/pp_to_llj_mlm_run_card.dat`, unedited (13 TeV, lhaid
  247000, ickkw 1, xqcut 20).
  - Integrated once with `integrate -j 4 --seed 1`: σ = 1067.89 ± 1.05 pb, 232 s.
  - E = 10 000 events.
- **Card B, unmatched.** Model `sm`, `generate p p > t t~` (`B_proc.dat`). Run
  card `B_run.dat` is `validation/madgraph/pp_to_ttx_0j1j_mlm_run_card.dat`
  with exactly two edits: `0 = ickkw` and `0 = xqcut`.
  - Integrated once with `integrate -j 4 --seed 1`: σ = 513.26 ± 0.30 pb, 3.3 s.
  - E = 249 000 events.
- **Each run:** `vibegraph --no-network generate -j N <art>/grid.bin.zst --seed 7
  --nevents E --out <file>`, with `LHAPDF_DATA_PATH` pointing at the banked
  NNPDF23 set. Two repeats for each N. Times come from `wait4`.

| card | N | wall r1 (s) | wall r2 (s) | mean (s) | speedup vs -j 1 | user CPU (s) | peak RSS |
|---|---|---|---|---|---|---|---|
| A | 1 | 137.13 | 135.00 | 136.07 | 1.00 | 136.0 | 37 MB |
| A | 2 | 73.53 | 72.19 | 72.86 | 1.87 | 139.9 | 40 MB |
| A | 4 | 39.11 | 41.49 | 40.30 | **3.38** | 138.1 | 45 MB |
| B | 1 | 23.80 | 24.87 | 24.34 | 1.00 | 23.0 | 210 MB |
| B | 2 | 16.06 | 16.18 | 16.12 | 1.51 | 23.6 | 218 MB |
| B | 4 | 10.56 | 11.18 | 10.87 | **2.24** | 21.7 | 232 MB |

**Identical output.** The event file's sha256 is identical across all six runs
of each card: A `3b6467b394d7ea56…`, B `2c5a8ad8933c78dd…`.

**Where the time goes.** Stage spans come from the `progress` timestamps of
the r1 runs. This head predates the phase records.

| card, N | compile | weight scan | unweight | outside the stages |
|---|---|---|---|---|
| A, 1 | 0.2 s | 23.9 s | 111.4 s | ≈0.6 s |
| A, 4 | 0.2 s | 7.1 s | 30.8 s | ≈1.3 s |
| B, 1 | <0.01 s | 1.0 s | 16.6 s | ≈5.7 s |
| B, 4 | <0.01 s | 0.1 s | 5.9 s | ≈4.1 s |

- **Card A.** A scales with accept/reject: unweighting runs 3.6× faster and
  the scan 3.4× faster at -j 4.
- **Card B.** B's shortfall is a serial tail of about 4 s. The log goes quiet
  for 4 s after the last unweighting record, while the 225 MB Les Houches
  file is written. A cheap process like this one is bound by output, so the
  -j 8 number will depend more on the writer than on sampling.

**Re-checked after the merge with main.** Main dfdf85d4 includes PR 10, which
changes per-event helicity reweighting. On the merged head 0baf1703 (release,
`v0.2.2-23-g0baf1703`), the event sha256s are unchanged from the table above:
- card A at `-j 1`: `3b6467b394d7ea56…`;
- card A at `-j 4`: `3b6467b394d7ea56…`;
- card B at `-j 1`: `2c5a8ad8933c78dd…`.

Neither card uses a reweight card, so PR 10's changes do not reach these
event streams.

## Validator-host request

- Branch `sprintl-generate`, the final commit of this session.
- Command: `pixi run validate-samples-2to6`, then `pixi run validation-report`.
  The task is `cargo test -p vibegraph-lib --profile release-debug --features
  extended-validation --test validate_samples -- --ignored --nocapture
  the_long_tier_sample_rows_match_madgraph`.
- Seeds are the gate's: `GEN_SEEDS` 0x5a4d0001–3, integration `SEED`
  20260719, scan `0x5CA70FF0`.
- Cost on 4 cores: about 65 min. The `bbx` row's accept/reject is about
  35 min of it.
- Expected answer: both rows' `samples` cells written, with three seeds of
  20000 events each. Every KS and χ² p ≥ 1e-4. Incoming legs and scales agree
  at the printing tolerance. The test passes.
- Risk to watch: `bbx_to_ccx_emmm_qcd0` measured 353 trials an event against
  the 400-trial budget, so a seed may come up short ("produced N of 20000
  events"). That would be a budget finding, not a pass bug (see Found).

## Found

1. **`sample_estimate_error` overstates the sample-σ error at the
   efficiencies production runs at.**
   - The estimator is `σ̂ = W·Σwᵢ/T`. Per trial, `x = W·w` if accepted and `0`
     if not, so `Var(σ̂) = (E[x²] − E[x]²)/T`, estimated by
     `σ̂²·(Σwᵢ²/(Σwᵢ)² − 1/T)`.
   - The shipped form drops `1/T`, giving `σ̂·√Σwᵢ²/Σwᵢ`. For unit weights
     the ratio of the two is `1/√(1−ε)`: 1.41× at ε = 0.5 and 1.29× at
     ε = 0.41. The doc's "1–2% at the efficiencies these samples run at" holds
     only for ε ≲ 0.04.
   - Evidence, first run of the five-seed headroom probe (shipped form): seed
     χ²/dof 0.05–0.72 on all but two of the 42 rows then gating
     (`ud_to_epemud_qcd0` 1.71, `gu_to_epemu` 1.44).
   - Toy, 400 seeds × 2000 events at ε = 0.248: the measured relative spread
     is 1.85% blocked and 1.91% serial, against the binomial `√((1−ε)/n)` of
     1.94%. The shipped form would quote `1/√n` = 2.24%.
   - It reaches the file header's sample estimate, the reported pull, and
     `cli_generate_proton`'s `SAMPLE_PULL_MAX` calibration.
   - To close: carry `T` into the formula, re-derive the pull bounds that read
     it, and pin the result with a seed-sweep test.
2. **`bbx_to_ccx_emmm_qcd0` acceptance is 2.835e-3 (353 trials an event), down
   from the 2.244e-2 (45 trials an event) recorded in its manifest note.**
   - The pass does not set the maxima. The w_max scan and the truncation rule
     are unchanged code, and the unweighting gate's realised efficiencies match
     prediction (e.g. 0.432 against 0.438).
   - So this is the grids or the scan on this row at this tip, or the
     earlier host's measurement, against this one at 2000 events. Not
     explained.
   - To close: rerun the probe printing `σ/Σw_max` and the per-channel peak to
     w_max ratios on both tips. It also decides whether the 400-trial samples
     budget holds for this row.
3. **The w_max scan's fixed cost equals the accept/reject phase on
   `p p > t t~ j`.**
   - 11.9 s at `-j1` and 3.55 s at `-j4`, against 11.75 / 3.12 s for
     accept/reject, at `--scan-points share`: 200000 scan points for 20000
     events.
   - It is already parallel, so it is a budget question. The scan's own
     docs say the budget sets the quantile's precision, not convergence.
   - To close: measure the truncated w_max against scan budget on this card,
     and either size `share` by the requested event count or document the
     trade.
4. **The systematics write is the dominant phase on `p p > t t~`** (7.5 s of
   12.7 s at `-j1`, 2.7 s of 4.5 s at `-j4`, 145 weights an event).
   - It runs after the accept/reject pass, in the writer, and is partly
     parallel already (6.8 s CPU in 2.7 s wall).
   - Moving the weight computation into the blocked pass's workers would
     need the systematics evaluation to take a prepared event in place of
     the writer's record. That is not cheap, and it was not done here.
   - To close: profile `SystematicsWriter` per event; move the PDF and scale
     re-evaluation onto `prepare`.
5. **`ee_to_wpwm` and `ee_to_wpwm_cw` angular shape disagrees with MadGraph's
   banked sample.**
   - 15 of 20 seeds have the `y(w+)` KS p below 0.05 under both passes (table
     above). Medians are about 1e-2 and the minimum is 5.7e-5.
   - The manager is filing
     `ee-to-wpwm-angular-shape-disagrees-with-madgraph-sample`.
   - To close: per-event comparison of the `cos θ_W` spectrum against
     MadGraph's own σ in angular windows, as was done for `ee_to_mumua`'s
     `pt(a)`. That would show which side owns the shape.
6. **A restart replays the blocked pass from block 0, recomputing every
   block.**
   - The two-pass strategies restart on multi-process cards (`Buffer` with
     more than one process id), which doubles accept/reject there.
   - Caching the consumed blocks' prepared points would make the second pass
     free, at a memory cost of about one event record per event.
   - To close: measure on a multi-`@N` card and decide.
