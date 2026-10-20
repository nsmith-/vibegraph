---
type: Session Report
title: "V1 report: the σ seed calibrations re-recorded from one current run"
description: "Every seed figure in validate_sigma.rs, validate_hadronic.rs, the hadronic manifest notes and a new census page is re-recorded against refdata-10's seed records under both default flips; the drift attribution to the note-34 draw commits is falsified and replaced by a bisection (e99b05c, 7664ff9, b6a0b88, and c2c5407 after the census); ddx_to_epemg is no longer thin, gux_to_epemux sits at 2.1x and four times the budget does not buy it margin."
status: draft
agent: validation-dev (Opus)
tags: [sigma-gate, calibration, seed-sweep, budget-ladder, refdata, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-19}
sources:
  - {id: brief, resource: "V1.md", title: "V1 brief"}
  - {id: census, resource: "../../../validation/seed-headroom-census-2026-10.md", title: "Seed-sweep headroom of the σ gates (2026-10)"}
---
Worktree `/home/user/wt/sigma-calibration-rerecord`, branch
`sigma-calibration-rerecord`, verified before the first action (toplevel and
branch). Host: the 4-core cloud container, one long job at a time, every cargo
invocation with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`. Logs are in the session scratchpad
(`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/`)
under `rerecord-V1-`; every probe log's first line records `HEAD` and
`HEAD^{tree}`. The local clone was shallow: `git fetch origin --unshallow` and
`git fetch origin pull/6/head` were needed for the attribution.

## Commits

| commit | tree | what |
|---|---|---|
| `508e094` | `2f6d3f1` | `probe_llj_parton_seed_stability` reads the gate's seed-record reference, combines each rung under the seed policy and times it |
| `3cfd7b8` | `3f9a26f` | the resonant, capstone and non-SM seed probes read the gate's reference |
| `6ef026f` | `15eb85b` | the calibration comments in `validate_sigma.rs` and `validate_hadronic.rs` and the hadronic manifest notes re-recorded |
| `cc1fd3b` | `340683c` | the bb and re-carded ladder docs and notes; the 2026-10 census page; the 2026-09 page marked superseded |
| (this report) | | |

No constant, tolerance, budget or gate code changed. The two probe commits
change only `#[ignore]` probes' reference and printing; the other two change
comments, report-cell note strings and manifest notes.

## 1. Baseline

```
$ pixi run --skip-deps validate        # HEAD 5782665, tree e1fe8ae, 16:52-18:16 UTC
cargo test: 65 binaries, 1924 passed, 0 failed, 95 ignored
64 rows × 4 categories = 256 cells: 203 measured in the layers this run drove (198 ✅, 5 ⚠️, 13 ⏳, 40 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell passed
exit 0
```

Log `rerecord-V1-baseline-validate.log`; report kept as
`rerecord-V1-baseline-report/` (`report.json`, per-cell JSON).

## 2. Drift attribution

**Brief correction.** The falsifier as written cannot run: at `f85718d^`
`probe_gate_row_seed_headroom` does not exist (it was added by the census,
`44e4e04`), and the 13 figures written in `e73b158` belong to SMEFTsim and
toy-model rows that `e73b158` itself added, so no tree before it can reproduce
them. I ran an equivalent: a throwaway probe (`rerecord-V1-stream-probe.rs`,
appended to each old tree's `validate_sigma.rs`, never committed) that prints
every requested gated row's per-seed σ to nine digits at that tree's own plan
budget, against that tree's committed `sigma_reference.json`, with refdata-10's
run cards and the SM UFO symlinked in, built in a detached throwaway worktree
(`/home/user/wt/rerecord-V1-old`, removed afterwards, target included). Driver:
`rerecord-V1-oldrun.sh <commit> <rows> [seeds]`; logs
`rerecord-V1-old-<commit>-plan.log`.

**At `f85718d^` (`c9993b4`, tree `6966f3e`), five seeds, 17 SM rows**, worst
\|rel\| against the single run, beside the 2026-09 census (`0538e2d`):

| row | `c9993b4` | census | | row | `c9993b4` | census |
|---|--:|--:|---|---|--:|--:|
| `gg_to_gg` | 1.972e-3 | 1.972e-3 | | `ee_to_ee` | 1.435e-3 | 1.435e-3 |
| `gg_to_ttx` | 6.683e-4 | 6.683e-4 | | `ee_to_mumu` | 1.250e-3 | 1.250e-3 |
| `uux_to_uux` | 1.043e-3 | 1.042e-3 | | `ee_to_ttx` | 9.217e-4 | 9.217e-4 |
| `uux_to_epemg` | 4.164e-3 | 4.164e-3 | | `ee_to_zh` | 1.059e-3 | 1.059e-3 |
| `ddx_to_epemg` | 6.296e-3 | 6.296e-3 | | `ee_to_wpwm` | 2.801e-3 | 2.801e-3 |
| `gu_to_epemu` | 1.841e-3 | 1.841e-3 | | `uux_to_mumu` | 2.029e-3 | 2.029e-3 |
| `gux_to_epemux` | 2.628e-3 | 2.628e-3 | | `ee_to_mumua` | 1.099e-2 | 1.099e-2 |
| `ud_to_epemud_qcd0` | **3.407e-3** | 3.954e-3 | | `ee_to_tatah` | **2.190e-3** | 2.162e-3 |
| `ee_to_mumu_tata_qcd0` | **7.222e-3** | 6.716e-3 | | | | |

So `f85718d^` already gives the census's numbers on 14 rows: the draw commits
the census blamed did not move them. Bracketing the other three, first-seed σ:
`f85718d` (tree `c9a7ac2`) and `df109b9` equal `c9993b4` on all rows tried;
`09cfb61` (tree `793bad1`), `b6a0b88` (`1d4b110`) and `e73b158` (`c09ba4b`)
equal each other and the census. **`09cfb61`, "denominate the channel floor in
accepted points" (merged as `b6a0b88`), moved `ee_to_mumu_tata_qcd0`,
`ee_to_tatah` and `ud_to_epemud_qcd0`.**

**`35ab3f1` (tree `d175005`), the era of the stale comments**, five seeds: it
reproduces its own comments' figures — `gg_to_gg` 1.438e-3 (recorded 1.4e-3),
`gg_to_ttx` 8.545e-4 (8.6e-4), `uux_to_uux` 1.070e-3 (1.1e-3), `uux_to_epemg`
3.930e-3 (3.9e-3), `ddx_to_epemg` 5.579e-3 (5.6e-3), `ud_to_epemud_qcd0`
2.644e-3 (2.6e-3), `ee_to_mumu_tata_qcd0` 4.567e-3 (4.5e-3) — so the
instrument reads the same statistic those comments did. `gu_to_epemu` and
`gux_to_epemux` read 1.160e-2 and 1.022e-2 there: their recorded 1.35e-3 and
1.51e-3 were written after chain B's AMP2 configuration draw (`2d60d1b`).

**Bisection of `35ab3f1..c9993b4`** (108 first-parent commits;
`rerecord-V1-bisect.sh`, first seed of nine signature rows; table from
`python3 -I rerecord-V1-cmp.py`, A = `35ab3f1`'s value, B = `c9993b4`'s,
lower case = a third value):

```
fp#  commit  ddx  tata  mumua  gg_gg  gg_ttx  gu  ud  uux_epemg  uux_uux
 22 e63f946   A    A     A      A      A      a   A    A          A
 28 45a7d62   A    A     A      A      A      a   A    A          A
 33 5724a34   A    A     A      A      A      a   A    A          A
 37 eb53fb7   A    A     A      A      A      a   A    A          A
 38 e99b05c   a    a     a      B      B      b   a    a          B
 44 152efb1   a    a     a      B      B      b   a    a          B
 66 e059092   a    a     a      B      B      b   a    a          B
 88 74dada9   a    a     a      B      B      b   a    a          B
 93 f4e2f0f   a    a     a      B      B      b   a    a          B
 98 4d03400   a    a     a      B      B      b   a    a          B
 99 443f6bc   a    a     a      B      B      b   a    a          B
103 a38ac3a   B    B     B      B      B      B   B    B          B
108 c9993b4   B    B     B      B      B      B   B    B          B
```

Between fp 99 and 103 only `7664ff9` touches source (`31d2e9a`, `5e22d06`,
`a38ac3a`: no file under `vibegraph-lib/src` or `vibegraph-cli/src`).

**Verdict.** The 2026-09 attribution is falsified. Which commit moved what:

| commit | change | moved |
|---|---|---|
| `e99b05c` (merge of `ea58ab9`, 2026-08-04) | unbiased VEGAS iteration combination | every row tried, `2 → 2` included (`gg_to_gg`, `gg_to_ttx`, `uux_to_uux` reach their census values here) |
| `7664ff9` (merge of `cd49f40`, 2026-08-06) | cut-implied timelike floors for the invariant draw | every row with ≥ 3 final-state particles, no `2 → 2` row |
| `b6a0b88` (`09cfb61`, 2026-08-07) | the channel floor in accepted points | `ee_to_mumu_tata_qcd0`, `ee_to_tatah`, `ud_to_epemud_qcd0` |

`f85718d` moved nothing (bit-identical to its parent). `f3d6e8b` lands at
fp 51 and `c48fc69` at fp 106, each between two bisection points whose values
agree on every row, so neither moved anything either.

**After the census.** The hygiene sprint's re-read (`ee_to_ee` 1.44e-3 →
3.24e-3, `gg_to_gg` 1.97e-3 → 2.66e-3, `ll_to_qqx_toy_tensor` 1.40e-3 →
2.23e-3) is `c2c5407`, "integrate on MadGraph's configuration channel set", a
later commit of the census's own PR #6. First-seed σ: `44e4e04` (tree
`0afbedd`) and `c2c5407^` (`b8ac237`, tree `31f079c`) agree on `ee_to_ee`
(155.6373237) and `gg_to_gg` (142771.4023), and `ll_to_qqx_toy_tensor` moves
1.8e-6 between them; `c2c5407` (tree `d1a23bb`) moves all three (155.5519905,
142860.4205, 8036.271694). `02e8b25` (tree `20d9c4d`) and `669f3fa` (tree
`c5d9180`) reproduce the October readings to the digit (worst 3.240e-3,
2.659e-3, 2.230e-3). What falsifies this: a tree between `c2c5407^` and
`c2c5407` that differs from both, which cannot exist (one commit).

## 3. Current runs

All on this branch; the gate code at every commit below equals `5782665`'s.

| command (all `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test <t> -- --ignored --nocapture --exact <probe>`) | HEAD / tree | wall | exit |
|---|---|---|---|
| `validate_sigma probe_gate_row_seed_headroom` | `508e094` / `2f6d3f1` | 970 s | 0 |
| `validate_sigma probe_llj_parton_seed_stability` | `508e094` / `2f6d3f1` | 210 s | 0 |
| `validate_hadronic probe_llj_fixed_budget_ladder` | `508e094` / `2f6d3f1` | 203 s | 0 |
| `validate_hadronic probe_llj_dyn_budget_ladder` | `508e094` / `2f6d3f1` | 937 s | 0 |
| `validate_hadronic probe_jj_budget_ladder` | `508e094` / `2f6d3f1` | 961 s | 0 |
| `validate_hadronic probe_llj_fixed_scatter_guard_calibration` (80 seeds) | `508e094` / `2f6d3f1` | 380 s | 0 |
| `validate_hadronic probe_llj_dyn_scatter_guard_calibration` (80 seeds) | `3cfd7b8` / `3f9a26f` | 1859 s | 0 |
| `validate_sigma probe_resonant_seed_stability` | `3cfd7b8` / `3f9a26f` | 150 s | 0 |
| `validate_sigma probe_smeft_capstone_seed_stability` | `3cfd7b8` / `3f9a26f` | 57 s | 0 |
| `validate_sigma probe_non_sm_seed_stability` | `3cfd7b8` / `3f9a26f` | 1658 s | 0 |
| `validate_hadronic sigma_llj_mlm_vs_madevent` | `3cfd7b8` / `3f9a26f` | 359 s | 0 |
| `validate_hadronic sigma_llj_xqcut_only_vs_madevent` | `3cfd7b8` / `3f9a26f` | 114 s | 0 |
| `validate_hadronic probe_bb_budget_ladder` | `6ef026f` / `15eb85b` | 108 s | 0 |
| `validate_hadronic probe_recarded_budget_ladder` | `6ef026f` / `15eb85b` | 1758 s | 0 |
| `vibegraph integrate <card> --run-card <row card> --fixed-budget --neval N --niter K --seed s -j 4 --no-tui`, seeds 20261101-20, six rows (P2's `drive.py`, `--arms base --no-scan`) | binary built from `5782665`, library source equal at `3cfd7b8` | 37 min | 0 |

Consistency checks between instruments: every row's
`probe_gate_row_seed_headroom` mean rel, mean pull and χ²/dof equal the
baseline report's cell (`ddx_to_epemg` +2.085e-3 / +2.29 / 0.27 in both); the
ladders' gate rungs equal the baseline cells (`pp_to_llj_fixed` 424.2691,
`pp_to_llj_dyn` 415.6533, `pp_to_jj` 6.797732e8, `pp_to_bb_fixed` 2.148213e6,
`pp_to_bb` 3.762756e8 pb); the 80-seed calibrations' first five seeds are the
gate's five.

The re-recorded figures, by site (the census page carries the full tables):

- **`PULL_LIMIT`** (`validate_sigma.rs`): restated for the five-seed mean's
  pull. A simulation of the seed-policy pull (both sides five seeds, error
  `max(quoted, spread/√5)`, 2·10⁶ trials; `python3 -I rerecord-V1-pullsim.py`,
  output `rerecord-V1-pullsim.log`) gives sd 0.913 and `P(|pull| > 3.5)` 2.44e-4 when the quoted errors
  are right, 4.57e-3 when one side scatters at twice its quoted error. Measured
  over the 35 rows: mean pulls mean +0.384, sd 0.84, max \|pull\| 2.29
  (`ddx_to_epemg`), median 0.50. The old text's "expected maximum about 1.16"
  was the expectation of the largest of five *signed* normals (1.163), not of
  `|pull|` (1.571).
- **`HEADROOM_SEEDS`**: 35 gated rows (not 34), mean pull +0.38; the first
  seed's own pulls average +0.548.
- **`plan_for`**: every quoted worst \|rel\| and margin, from the headroom
  probe; the resonant, ud, capstone, non-SM, toy and `gg_to_gg_cg` paragraphs
  from their own probes. `ee_to_wpwm` and `ee_to_wpwm_cw`'s "converged −0.22%
  offset" was the single MadEvent run's: against the seed records they read
  +3.7e-4 and +2.9e-4 (11.1× and 13.5×).
- **Module doc**: `gg_to_gg_cg` is integrated and gated; one row of the
  non-SM set (`wpwm_to_wpwmz_cw`) is not; the non-SM row count is nineteen
  (13 SMEFTsim rows with a banked σ plus 6 toy rows, from the manifest's
  `model` fields), not sixteen; the 2 → 6 rows are `Plan::Long`, not
  "not integrated" (the same doc's stale claim, fixed alongside).
- **`validate_hadronic.rs`**: `LLJ_NEVAL`, `LLJ_MAX_REL`, `LLJ_DYN_MAX_REL`,
  `LLJ_MAX_CHI2_PER_DOF`, `sigma_llj_dynamical_scale_vs_mg` (doc and cell
  note), the jj docs and cell note, `DY_MAX_REL`/`DY_MAX_CHI2_PER_DOF`, the bb
  docs and cell note, `RECARDED_ROWS`, `RECARDED_MAX_REL`/`_CHI2`. Each ladder
  rung is quoted beside its five seeds' standard deviation and quoted error,
  and the gate rung beside the 80-seed spread.
- **Manifest** integrals notes of `pp_to_llj_fixed` (the stale "chi2/dof
  1.86"; now 0.55 at the gate, 0.89 over 80 seeds), `pp_to_llj_dyn`,
  `pp_to_llj`, `pp_to_jj`, `pp_to_ll`, `pp_to_bb`, `pp_to_bb_qcd2`,
  `pp_to_bb_fixed`, `pp_to_ll_scalefact2`.

The ℓℓj rationale changed in substance, not only in digits: the 75k rung no
longer carries the largest scatter (χ²/dof 1.15 and 0.93 against ≤ 1.15
above); what still argues for 150k is that 75k reads highest on both rows
(+0.40% and +0.35% against +0.22% and +0.20% at 600k, 1.9 and 1.7 standard
errors apart).

## 4. Spreads beside calibrations

In the [census page](../../../validation/seed-headroom-census-2026-10.md),
"Spreads beside the calibrations". Summary (spread / quoted, one seed):

| row | gate quintet (or ten) | 20 or 80 seeds |
|---|--:|--:|
| `pp_to_llj_mlm` | 0.90 (10 seeds) | 0.72 (20) |
| `pp_to_llj_xqcut_only` | 1.24 | 0.72 (20) |
| `pp_to_llj` | 0.70 | 1.01 (20) |
| `pp_to_jj` | 1.01 | 1.07 (20) |
| `uux_to_epemg` | 0.77 | 1.14 (20) |
| `ee_to_mumua` | 1.27 | 1.05 (20) |
| `pp_to_llj_fixed` | 0.74 | 0.94 (80) |
| `pp_to_llj_dyn` | 0.72 | 0.95 (80) |

`pp_to_llj_mlm`'s quoted error is 39% above its 20-seed spread (P2 read 26%
under `K = 6` before #51's map flip), and `pp_to_llj_xqcut_only` shares it.
The 20-seed runs are the CLI path at seeds 20261101-20, not the gate harness's
seeds; the gate columns are the gates' own seeds.

## 5. Thin headroom (for D1)

`probe_llj_parton_seed_stability` at `508e094`, the gate's five seeds, against
the refdata-10 seed records (0.074% and 0.075% errors):

| row | rung | mean rel | mean pull | seed χ²/dof | worst \|rel\| | margin on `rel_tol` | wall, 5 seeds |
|---|---|--:|--:|--:|--:|--:|--:|
| `ddx_to_epemg` (0.01) | 60000×8 | +2.09e-3 | +2.29 | 0.27 | 2.80e-3 | **3.6×** | 13.3 s |
| | 240000×8 | +1.48e-3 | +1.69 | 3.15 | 2.66e-3 | 3.8× | 34.8 s |
| `gux_to_epemux` (0.005) | 60000×8 | +1.15e-3 | +1.24 | 1.05 | 2.43e-3 | **2.1×** | 19.0 s |
| | 240000×8 | +0.90e-3 | +1.08 | 1.93 | 2.27e-3 | 2.2× | 40.4 s |

- `ddx_to_epemg` is no longer thin: the 1.6× was against the single MadEvent
  run (0.6177), which sits 0.23% below the five-seed record (0.6191). What
  remains is its mean pull, +2.29 (1.5× inside `PULL_LIMIT`, class b), the
  largest of the 35; with `uux_to_epemg` at +1.85 (+2.65 at 4×) the pair reads
  about +0.2% high at both budgets.
- `gux_to_epemux` stays at about 2×, and 4× the points does not change that:
  the worst seed at 4× is a different one (+2.27e-3 on a +0.90e-3 mean).
  The cost of 4× is about 2.5× the row's wall time (19 → 40 s for five seeds
  on this host; the α survey is a fixed cost).
- No budget changed. The decision is the user's (D1); on these numbers
  `ddx_to_epemg` needs none and `gux_to_epemux` would not be bought margin by
  4×.

## 6. Gates

All on `cc1fd3b` (tree `340683c`), the last code commit; this report adds
only itself. Logs `rerecord-V1-hermetic.log`, `rerecord-V1-closing-validate.log`.

```
$ cargo fmt --all --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 20.97s
exit 0
$ cargo test --workspace
40 binaries: 1538 passed, 0 failed, 16 ignored
exit 0
$ pixi run --skip-deps kb-lint
797 concepts, 270 backlog items; 0 errors, 3 warnings      # the three pre-existing blocked-item warnings
$ pixi run --skip-deps validate        # 23:07-00:03 UTC; runs clippy with
                                       # --features vibegraph/extended-validation,vibegraph-lib/extended-validation first
    Finished `dev` profile [optimized] target(s) in 3.64s   # that clippy, no warning or error line in the log
cargo test: 65 binaries, 1924 passed, 0 failed, 95 ignored
64 rows × 4 categories = 256 cells: 203 measured in the layers this run drove (198 ✅, 5 ⚠️, 13 ⏳, 40 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell passed
exit 0
```

Baseline against closing, every cell file (`python3 -I rerecord-V1-cellcmp.py
rerecord-V1-baseline-report rerecord-V1-closing-report`, comparing every field
but notes and timings):

```
cells compared 209 moved (non-note, non-timing fields): []
note text changed: ['integrals/pp_to_bb_fixed.json', 'integrals/pp_to_jj.json', 'integrals/pp_to_llj.json', 'integrals/pp_to_llj_dyn.json']
report marks identical: True ok True True
```

No cell moved: every σ, error, pull, rel, χ²/dof and per-seed value is
bit-identical, and the only differences are the four cell notes this session
rewrote.

## Deviations and corrections to the brief

- **The falsifier** (§2): as written it cannot run at `f85718d^`; an
  equivalent per-seed stream probe was run there and at 22 other commits.
- **Probe changes committed.** Four `#[ignore]` probes now read the gate's
  seed-record reference instead of the single run (`508e094`, `3cfd7b8`):
  without that, the figures they print are not the statistic the tolerances
  bound, and the thin-pair re-measurement "against refdata-10" would have read
  `ddx_to_epemg` against its known-low single run.
- **More than the named sites.** The DY, bb, re-carded and jj ladder figures
  and their manifest notes were re-recorded too (two extra ladders run),
  because the drift item closes on "the hadronic manifest gate notes", and
  every one of them quoted a pre-refdata-10 reading. The MLM rows' manifest
  notes (measured on refdata-9) were not touched: see Found.
- **The 2026-09 census page** is marked superseded for the σ gates and kept;
  the new figures are a new Measurement page, since the old one is a dated
  record. Its samples, unweighting and `cli_generate_proton` sections were not
  re-measured.
- `probe_llj_dyn_scatter_guard_calibration` ran at `3cfd7b8` rather than
  `508e094` (chained after the commit); `validate_hadronic.rs` is identical at
  both.

## Banking request

None. Every comparison is against refdata-10's existing seed records.

## Found

1. **`pp_to_llj_fixed`'s gate reads 1.8× inside `LLJ_MAX_REL`.** Gate quintet
   +0.284% against 0.005 (baseline cell); 80 seeds read +0.231% (2.2×) and the
   600k rung +0.22%, so a real offset of about +0.2% against the seed record
   (0.21% error) plus a high quintet. Class (a) under 2×. Close: the same
   budget-or-accept decision as D1, or a diagnosis of the +0.2% that
   `pp_to_llj`, `pp_to_llj_dyn` and the two `q q̄ → e⁺e⁻ g` rows share.
2. **The `ℓℓj`-family rows all read about +0.2% high against their seed
   records.** `pp_to_llj` +0.20%, `pp_to_llj_dyn` +0.21% (80 seeds),
   `pp_to_llj_fixed` +0.23% (80), `uux_to_epemg` +0.19% / +0.23% (1× / 4×),
   `ddx_to_epemg` +0.21% / +0.15%, `pp_to_llj_mlm` +0.23%, `pp_to_llj_xqcut_only`
   +0.19%; references 0.07-0.22%. Not budget (flat ladders), shared across
   fixed and dynamical scales. Close: the M3 batch's 20+20 `uux`/`ddx` ladder
   already requested on #42, read against this list.
3. **`ee_to_ee`'s five seeds are under-dispersed**: χ²/dof 0.07, spread
   1.5e-4 of σ against 6.0e-4 quoted, `P(χ²₄ < 0.28) ≈ 0.009`. Close: a
   20-seed run at the gate budget to see whether the quoted error overstates
   the spread on the t-channel row (as on the matched `ℓℓj` rows) or the
   quintet is a draw.
4. **`gg_to_gg_cg` sits at exactly 2.0× on `rel_tol`** (worst seed 2.55e-3,
   scatter 0.12% a seed). New on the thin list beside `gux_to_epemux` (2.1×)
   and `ll_to_qqx_toy_tensor` (2.2×, seed χ²/dof 2.73).
5. **The MLM manifest notes quote refdata-9 readings.** `pp_to_llj_mlm` now
   reads +0.23% (pull +1.00, ten seeds, χ²/dof 0.81) against a five-seed
   seed record (268.136 ± 0.593) where the note quotes ten MadEvent seeds
   268.147 ± 0.289; `pp_to_llj_xqcut_only` +0.19% (pull +0.85, χ²/dof 1.52).
   `_alps2`, `0j2j` and `ttx_0j1j` were not run. Close: `pixi run
   validate-mlm-sigma` and a re-record of those five notes.
6. **The census's attribution method was a guess the evidence never pinned.**
   The 2026-09 page bracketed the drift by when comments were written and
   named three commits by kind; the bisection names three others. Close
   (process): any future "these commits moved the stream" claim carries a
   first-seed σ at the named commit and its parent.
7. **`probe_llj_seed_ensemble`'s forty-seed figures** quoted in
   `RECARDED_ROWS` (2-5× understatement) are from an earlier sampler and were
   not re-run (the 20-seed CLI run on this tree reads `pp_to_llj` at 1.01 of
   quoted). Close: re-run it, or drop the figure once the 20-seed reading is
   the record.
8. **Other concept pages quote 2026-09 census figures** (`gate-thresholds`,
   `seed-sweeps-and-budget-ladders`, `samples-gate`, `reference-row-rationale`
   link the old page). Close: point them at the 2026-10 page at close-out.
