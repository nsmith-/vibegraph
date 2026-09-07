# B0 — seed-sweep headroom census of the banked layer

Session B0 of `banked-open-ends` (note 36 §3). Branch `b0-seed-headroom`, base
`0538e2d`. Every number below is from a command run in this session; the command
and its full output per item are in `/tmp/b0-evidence.log`.

## 0. How to read the "headroom ×" column

The census separates two kinds of enforced threshold, because the same ratio
means opposite things across them.

**(a) Tolerances.** `rel_tol`, `*_MAX_REL`, `SIGMA_REL_LIMIT`, `SIGMA_MAX_REL`.
These bound a *disagreement of measured size* — a quantity in physical units that
does not shrink on its own. Headroom is the ratio of the bound to the measured
value, and a ratio near one means the next change that touches the row fails the
gate. The `< 2×` rule applies here.

**(b) Standardised thresholds.** `PULL_LIMIT`, the hadronic rows' `|pull| < 3.0`,
`SHAPE_PULL_LIMIT`, `SPECTRUM_MAX_PULL`, both `P_FLOOR`s, and every
`*_MAX_CHI2_PER_DOF` / `SHAPE_CHI2_LIMIT` — a χ²/dof is `χ²_k / k`, whose null has
mean 1 whatever the physics.
These are *false-positive rates against a trial count*: the statistic is the
largest (or smallest) of `N` draws from a known null, and the threshold was
chosen so that crossing it is rare at that `N`. The largest of five draws of
`|N(0,1)|` has expectation `1.57`, exceeds `2.0` a fifth of the time and exceeds
`3.5` once in 430, so a `3.5σ` limit reading "2× above the worst of five seeds"
is the instrument working. The census's own per-row worst `|pull|` averages
`1.562` over the 31 gated rows against that `1.57` (median `1.49`, range
`0.51`–`3.56`), which is the check that the pulls are the standardised variable
they are treated as. **More room would mean the threshold had stopped
rejecting.** Applying
the `< 2×` rule here is a category error, and the note-34 remedy "form the
statistic over five seeds" can make an extremum-vs-floor statistic *worse*: more
seeds are more draws, so the expected minimum `p` falls and the flag rate rises.
These rows are marked `(b)` and their headroom is reported, not judged. The seed
count still matters to them, but through the degrees of freedom rather than the
margin: `χ²/dof < 4` is a `1.8 %` false-positive rate on the two degrees of
freedom a three-seed gate has and `0.30 %` on the four a five-seed gate has, which
is the argument for §7's `JJ_SEEDS` change and was `LLJ_SEEDS`'.

The `⚠️` marks a class-(a) statistic under `2×`.

## 1. `validate_sigma.rs` — 31 `Plan::Gate` rows

Every row's gate statistic is formed at the **single** seed `SEED = 20260719`;
both `|rel| ≤ rel_tol` and `|pull| ≤ PULL_LIMIT` are asserted on that one draw.
The five-seed column is `probe_gate_row_seed_headroom` (added this session):
seeds `[SEED, 11, 22, 33, 44]` at each row's own plan budget, 207 s for all 31.

`probe_gate_row_seed_headroom` reproduces the recorded manifest numbers where
they exist, so it is the same statistic and not a cousin of it.

### 1a. `rel_tol` — a tolerance, class (a)

Threshold and measured value are both `|σ_vg/σ_MG − 1|`; the measured column is
the worst of the five seeds.

| file · row | statistic | seeds forming | seeds calibrating | threshold | measured 5-seed value | headroom × |
|---|---|---|---|---|---|---|
| `ddx_to_epemg` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_llj_parton_seed_stability` | 0.0100 | 6.296e-3 | **1.6×** ⚠️ |
| `ee_to_ee` | `\|rel\| ≤ rel_tol` | 1 | **none recorded** | 0.0400 | 1.435e-3 | **27.9×** |
| `ee_to_mumu` | `\|rel\| ≤ rel_tol` | 1 | **none recorded** | 0.0200 | 1.250e-3 | **16.0×** |
| `ee_to_mumu_4f` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.328e-3 | **3.8×** |
| `ee_to_mumu_smlimit` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 6.044e-4 | **8.3×** |
| `ee_to_mumu_tata_qcd0` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_resonant_seed_stability` | 0.0200 | 6.716e-3 | **3.0×** |
| `ee_to_mumua` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_resonant_seed_stability` | 0.0300 | 1.099e-2 | **2.7×** |
| `ee_to_tatah` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_resonant_seed_stability` | 0.0200 | 2.162e-3 | **9.2×** |
| `ee_to_ttx` | `\|rel\| ≤ rel_tol` | 1 | **none recorded** | 0.0200 | 9.217e-4 | **21.7×** |
| `ee_to_ttx_dipole` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.591e-3 | **3.1×** |
| `ee_to_ttx_smeft` | `\|rel\| ≤ rel_tol` | 1 | 7 — `probe_smeft_capstone_seed_stability` | 0.0020 | 4.651e-4 | **4.3×** |
| `ee_to_ttx_smlimit` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.132e-3 | **4.4×** |
| `ee_to_wpwm` | `\|rel\| ≤ rel_tol` | 1 | **none recorded** | 0.0300 | 2.801e-3 | **10.7×** |
| `ee_to_wpwm_cw` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0080 | 3.067e-3 | **2.6×** |
| `ee_to_zh` | `\|rel\| ≤ rel_tol` | 1 | **none recorded** | 0.0200 | 1.059e-3 | **18.9×** |
| `ee_to_zh_smeft` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 4.399e-4 | **11.4×** |
| `gg_to_gg` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_qcd_seed_stability` | 0.0300 | 1.972e-3 | **15.2×** |
| `gg_to_ttx` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_qcd_seed_stability` | 0.0200 | 6.683e-4 | **29.9×** |
| `gg_to_ttx_smlimit` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 2.090e-3 | **2.4×** |
| `gg_to_ttx_smlimit_qcd2` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.692e-3 | **3.0×** |
| `gu_to_epemu` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_llj_parton_seed_stability` | 0.0050 | 1.841e-3 | **2.7×** |
| `gux_to_epemux` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_llj_parton_seed_stability` | 0.0050 | 2.628e-3 | **1.9×** ⚠️ |
| `ll_to_qqx_toy_dipole` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.009e-3 | **5.0×** |
| `ll_to_qqx_toy_tensor` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.399e-3 | **3.6×** |
| `ll_to_qqx_toy_yukawa` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 3.065e-4 | **16.3×** |
| `tata_to_ttx_tensor4f` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 1.448e-3 | **3.5×** |
| `ud_to_epemud_qcd0` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_resonant_seed_stability` | 0.0100 | 3.954e-3 | **2.5×** |
| `uux_to_epemg` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_llj_parton_seed_stability` | 0.0100 | 4.164e-3 | **2.4×** |
| `uux_to_mumu` | `\|rel\| ≤ rel_tol` | 1 | **none recorded** | 0.0200 | 2.029e-3 | **9.9×** |
| `uux_to_ttx_4f` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_non_sm_seed_stability` | 0.0050 | 6.684e-4 | **7.5×** |
| `uux_to_uux` | `\|rel\| ≤ rel_tol` | 1 | 5 — `probe_qcd_seed_stability` | 0.0200 | 1.042e-3 | **19.2×** |

### 1b. `PULL_LIMIT = 3.5` — class (b), reported not judged

Every gated row asserts this except `ee_to_mumua`, which
`PULL_REPORTED_NOT_ASSERTED` exempts. Measured column is the worst of five.

| file · row | statistic | seeds forming | seeds calibrating | threshold | measured 5-seed value | ratio |
|---|---|---|---|---|---|---|
| `ddx_to_epemg` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_llj_parton_seed_stability` | 3.5 | 2.65 | 1.3× |
| `ee_to_ee` | `\|pull\| ≤ PULL_LIMIT` | 1 | none recorded | 3.5 | 1.16 | 3.0× |
| `ee_to_mumu` | `\|pull\| ≤ PULL_LIMIT` | 1 | none recorded | 3.5 | 1.34 | 2.6× |
| `ee_to_mumu_4f` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.53 | 2.3× |
| `ee_to_mumu_smlimit` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 0.68 | 5.1× |
| `ee_to_mumu_tata_qcd0` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_resonant_seed_stability` | 3.5 | 2.33 | 1.5× |
| `ee_to_mumua` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_resonant_seed_stability` | 3.5 | 3.56 | 1.0× — **exempt**, see below |
| `ee_to_tatah` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_resonant_seed_stability` | 3.5 | 1.42 | 2.5× |
| `ee_to_ttx` | `\|pull\| ≤ PULL_LIMIT` | 1 | none recorded | 3.5 | 1.15 | 3.0× |
| `ee_to_ttx_dipole` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 2.19 | 1.6× |
| `ee_to_ttx_smeft` | `\|pull\| ≤ PULL_LIMIT` | 1 | 7 — `probe_smeft_capstone_seed_stability` | 3.5 | 1.22 | 2.9× |
| `ee_to_ttx_smlimit` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.58 | 2.2× |
| `ee_to_wpwm` | `\|pull\| ≤ PULL_LIMIT` | 1 | none recorded | 3.5 | 1.73 | 2.0× |
| `ee_to_wpwm_cw` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.87 | 1.9× |
| `ee_to_zh` | `\|pull\| ≤ PULL_LIMIT` | 1 | none recorded | 3.5 | 1.75 | 2.0× |
| `ee_to_zh_smeft` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.58 | 2.2× |
| `gg_to_gg` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_qcd_seed_stability` | 3.5 | 1.49 | 2.3× |
| `gg_to_ttx` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_qcd_seed_stability` | 3.5 | 0.51 | 6.8× |
| `gg_to_ttx_smlimit` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.47 | 2.4× |
| `gg_to_ttx_smlimit_qcd2` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.16 | 3.0× |
| `gu_to_epemu` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_llj_parton_seed_stability` | 3.5 | 0.84 | 4.2× |
| `gux_to_epemux` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_llj_parton_seed_stability` | 3.5 | 1.10 | 3.2× |
| `ll_to_qqx_toy_dipole` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.71 | 2.0× |
| `ll_to_qqx_toy_tensor` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.47 | 2.4× |
| `ll_to_qqx_toy_yukawa` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.00 | 3.5× |
| `tata_to_ttx_tensor4f` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 1.93 | 1.8× |
| `ud_to_epemud_qcd0` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_resonant_seed_stability` | 3.5 | 1.34 | 2.6× |
| `uux_to_epemg` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_llj_parton_seed_stability` | 3.5 | 1.66 | 2.1× |
| `uux_to_mumu` | `\|pull\| ≤ PULL_LIMIT` | 1 | none recorded | 3.5 | 1.93 | 1.8× |
| `uux_to_ttx_4f` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_non_sm_seed_stability` | 3.5 | 2.19 | 1.6× |
| `uux_to_uux` | `\|pull\| ≤ PULL_LIMIT` | 1 | 5 — `probe_qcd_seed_stability` | 3.5 | 0.87 | 4.0× |

### 1c. What the σ census found

* **Six rows have no recorded seed calibration at all**: `uux_to_mumu`,
  `ee_to_mumu`, `ee_to_ttx`, `ee_to_zh`, `ee_to_wpwm`, `ee_to_ee`. No
  `probe_*_seed_stability` covers them and `plan_for`'s comments quote no sweep.
  Measured here for the first time, all six clear their `rel_tol` by `9.9×` to
  `27.9×` — the loosest bounds in the file on the smoothest integrands, which is
  why nothing had noticed. Their calibration is now a recorded measurement.
* **Two rows are class-(a) thin**: `ddx_to_epemg` at `1.6×` and
  `gux_to_epemux` at `1.9×`.
* **Every recorded calibration written before `e73b158` has drifted; every one
  written in it reproduces to the digit.** This is the census's sharpest finding
  and it splits cleanly on when the comment was written, not on what the row is.

  Written in `e73b158` (2026-09-07, `ufo-lorentz`), recorded vs measured worst
  `|rel|` over the same five seeds at the same budget — 13 of 13 identical:
  `6.0e-4 / 6.044e-4`, `2.1e-3 / 2.090e-3`, `1.7e-3 / 1.692e-3`,
  `1.1e-3 / 1.132e-3`, `1.6e-3 / 1.591e-3`, `4.4e-4 / 4.399e-4`,
  `1.3e-3 / 1.328e-3`, `6.7e-4 / 6.684e-4`, `1.4e-3 / 1.448e-3`,
  `1.0e-3 / 1.009e-3`, `1.4e-3 / 1.399e-3`, `3.1e-4 / 3.065e-4`,
  `3.1e-3 / 3.067e-3`.

  Written in `5cc41de` (2026-08-01) and `35ab3f1` — none reproduces:
  `gg_to_gg` `1.4e-3 → 1.972e-3`, `gg_to_ttx` `8.6e-4 → 6.683e-4`,
  `uux_to_uux` `1.1e-3 → 1.042e-3`, `uux_to_epemg` `3.9e-3 → 4.164e-3`,
  `ddx_to_epemg` `5.6e-3 → 6.296e-3`, `gu_to_epemu` `1.35e-3 → 1.841e-3`,
  `gux_to_epemux` `1.51e-3 → 2.628e-3`, `ud_to_epemud_qcd0`
  `2.6e-3 → 3.954e-3`, `ee_to_mumu_tata_qcd0` `4.5e-3 → 6.716e-3`, and
  `ee_to_mumua`'s gate-seed pull `2.83 → 3.56`.

  Same seeds, same budgets, same helper, so this is a *sampling stream* that
  moved and was never re-recorded. It brackets to `(5cc41de, e73b158]` — the
  window that contains the note-34 draw-performance commits, which are
  stream-touching by construction (`f85718d` builds each subsystem once per
  point rather than once per channel, `c48fc69` sweeps the channel densities
  once per surveyed point, `f3d6e8b` reuses filled arenas). Nothing has moved
  since `e73b158`. The falsifier for the attribution is one run of
  `probe_gate_row_seed_headroom` at `f85718d^`; not run here, out of scope.
  The `ℓℓj` rows' comments are corrected in this commit because two of them
  are on the thin list; the rest are left for whoever re-records them, with
  `probe_gate_row_seed_headroom` as the one command that does it.
* **`PULL_REPORTED_NOT_ASSERTED` is not vacuous**: `ee_to_mumua`'s worst pull
  over five seeds is `3.56`, i.e. above `PULL_LIMIT`. Without the exemption the
  gate fails on a residual it has separately attributed to the reference. That is
  the pinned-convention test AGENTS.md asks for, and the census supplies it.
* **`ddx_to_epemg`'s pull cannot run away with budget.** Its worst pull is `2.65`
  at the gate budget and `2.62` at four times it, because the combined error is
  the reference's (`0.20 %`) and not this side's: the pull saturates at
  `rel / σ_MG,rel ≈ 2.2`. So the `1.3×` in table 1b is a bounded quantity, not a
  systematic on its way through the limit — the opposite of `ee_to_mumua`.

## 2. `validate_samples.rs` — `P_FLOOR`

One enforced statistic: the smallest `p` over 32 gating rows × `GEN_SEEDS` ×
7–21 columns. `probe_samples_p_floor_headroom` (added this session) repeats the
whole comparison over five generation seeds.

| file · row | statistic | seeds forming | seeds calibrating | threshold | measured 5-seed value | headroom × |
|---|---|---|---|---|---|---|
| `validate_samples.rs` · all 32 gating rows | min `p` over rows × seeds × columns | 3 | 3 (the same reading) | 1e-4 | `1.573e-4` (`ee_to_wpwm` `pt(w+)`, seed `0x5a4d0003`) | 1.57× (b) |

The three-seed minimum reproduces the recorded `1.573e-4` on the same row,
column and seed. **The five-seed minimum is identical** — the two extra seeds
read `4.765e-2` and `1.962e-2` on that row. Fifteen of the 32 rows lower their
own minimum; none goes below `5.483e-3` (`ee_to_mumu_tata_qcd0`). Runners-up at
five seeds: `ee_to_wpwm_cw` `7.791e-4` (7.8×), `qqx_to_o8o8_toy_dcolor`
`9.361e-4` (9.4×), `ddx_to_epemg` `1.846e-3` (18.5×), `uux_to_ttx_4f`
`2.035e-3` (20.4×).

The two `info` rows are unchanged and remain below the floor by construction:
`ee_to_mumua` `8.308e-6` (`pt(a)`, the reference's own sample) and
`ud_to_epemud_qcd0` `0.0` (`ICOLUP`, the standing finding B4 takes).

## 3. `validate_hadronic.rs`

Five seed families. `LLJ_SEEDS` is already five (note 34's remedy, applied
there); `DY_SEEDS`, `BB_SEEDS`, `JJ_SEEDS` and `RECARDED_SEEDS` were three.
`probe_hadronic_seed_headroom` (added this session) runs every three-seed arm at
its own gate budget over the gate's three seeds plus two, and prints both
readings. Its three-seed column reproduces the manifest notes' recorded numbers
(e.g. `pp_to_jj` `6.813339e8 ± 5.496e5 pb`, pull `+1.58`, rel `+0.37 %`,
χ²/dof `0.80`) bit for bit, so the five-seed column is the same statistic.

The `χ²/dof < 4.0` bounds were all calibrated on five-seed budget ladders while
the gate formed them on three — two degrees of freedom against the bound's four.
`DY_MAX_CHI2_PER_DOF` is the exception with no independent calibration at all:
its doc comment's "Measured `0.74` and `1.19`" are the three-seed gate readings
themselves.

| file · row | statistic | seeds forming | seeds calibrating | threshold | measured 5-seed value | headroom × |
|---|---|---|---|---|---|---|
| `dy_default` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.01 | +5.890e-4 (3-seed +7.230e-4) | **17.0×** |
| `dy_default` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.80 (3-seed 1.06) | 5.0× (b) |
| `dy_default` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | +0.84 (3-seed +0.91) | 3.6× (b) |
| `dy_mmll_60_120` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.01 | -1.325e-4 (3-seed -1.970e-4) | **75.5×** |
| `dy_mmll_60_120` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.97 (3-seed 1.43) | 4.1× (b) |
| `dy_mmll_60_120` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | -0.20 (3-seed -0.27) | 14.7× (b) |
| `pp_to_bb_fixed` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.005 | +1.549e-3 (3-seed +1.749e-3) | **3.2×** |
| `pp_to_bb_fixed` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.63 (3-seed 0.02) | 6.3× (b) |
| `pp_to_bb_fixed` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | +0.81 (3-seed +0.89) | 3.7× (b) |
| `pp_to_jj` | `\|rel\| < MAX_REL` | 3 → **5** | 5 (ladder, recorded) | 0.005 | +3.329e-3 (3-seed +3.659e-3) | **1.5×** ⚠️ |
| `pp_to_jj` | `χ²/dof < MAX_CHI2` | 3 → **5** | 5 (ladder, recorded) | 4 | 1.40 (3-seed 0.80) | 2.9× (b) |
| `pp_to_jj` | `\|pull\| < 3.0` | 3 → **5** | 5 (ladder, recorded) | 3.0 | +1.47 (3-seed +1.58) | 2.0× (b) |
| `pp_to_bb` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.005 | +7.240e-4 (3-seed +5.218e-4) | **6.9×** |
| `pp_to_bb` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.55 (3-seed 0.81) | 7.3× (b) |
| `pp_to_bb` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | +0.92 (3-seed +0.62) | 3.3× (b) |
| `pp_to_bb_qcd2` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.005 | +2.692e-4 (3-seed +1.488e-4) | **18.6×** |
| `pp_to_bb_qcd2` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.23 (3-seed 0.00) | 17.4× (b) |
| `pp_to_bb_qcd2` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | +0.33 (3-seed +0.17) | 9.0× (b) |
| `pp_to_llj` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.005 | -2.605e-4 (3-seed +3.445e-4) | **19.2×** |
| `pp_to_llj` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.32 (3-seed 0.20) | 12.5× (b) |
| `pp_to_llj` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | -0.08 (3-seed +0.10) | 39.9× (b) |
| `pp_to_ll_scalefact2` | `\|rel\| < MAX_REL` | 3 | 5 (ladder, recorded) | 0.005 | -3.369e-4 (3-seed +2.083e-4) | **14.8×** |
| `pp_to_ll_scalefact2` | `χ²/dof < MAX_CHI2` | 3 | 5 (ladder, recorded) | 4 | 0.82 (3-seed 0.09) | 4.9× (b) |
| `pp_to_ll_scalefact2` | `\|pull\| < 3.0` | 3 | 5 (ladder, recorded) | 3.0 | -0.16 (3-seed +0.10) | 18.6× (b) |
| `llj_fixed` (`pp_to_llj_fixed`) | `\|rel\| < LLJ_MAX_REL` | 5 | 5 (ladder, recorded) | 0.005 | `-2e-4` | 25× |
| `llj_fixed` | `χ²/dof < LLJ_MAX_CHI2_PER_DOF` | 5 | 5 (ladder, recorded) | 4.0 | `1.77` | 2.3× (b) |
| `llj_fixed` | `\|pull\| < 3.0` | 5 | 5 (ladder, recorded) | 3.0 | `-0.07` | 43× (b) |
| `llj_dyn` (`pp_to_llj_dyn`) | `\|rel\| < LLJ_DYN_MAX_REL` | 5 | 5 (ladder, recorded) | 0.005 | `+1.4e-3` | 3.6× |
| `llj_dyn` | `χ²/dof < LLJ_MAX_CHI2_PER_DOF` | 5 | 5 (ladder, recorded `0.66`) | 4.0 | `2.17` | 1.8× (b) |
| `llj_dyn` | `\|pull\| < 3.0` | 5 | 5 (ladder, recorded) | 3.0 | `+0.43` | 7.0× (b) |

The two `ℓℓj` arms above are read from this session's `pixi run --skip-deps
validate`, since `LLJ_SEEDS` is already five and they are not in the probe.

**`pp_to_llj_dyn`'s χ²/dof has moved and no one re-recorded it.** Its manifest
note and `LLJ_DYN_MAX_REL`'s doc comment both say `+0.21 %` at `χ²/dof 0.66` over
five seeds at this budget; today the same five seeds at the same budget read
`+0.14 %` at `χ²/dof 2.17`. σ barely moved and the seed *scatter* tripled, which
is what a small per-point numerical change does to a VEGAS estimator. It joins
the four `ℓℓj` partonic σ rows in §1c — the same family, the same direction, the
same "recorded five-seed calibration no longer describes the tip". The most
likely author is a stream-touching change inside the window §1c brackets, and
the falsifier is one run of `sigma_llj_dynamical_scale_vs_mg` at each candidate's
parent. Not run here — out of scope — but worth a session, because at
`1.8×` this cell is the one a further reshuffle takes over the bound. Its
author is bracketed the same way as §1c's σ rows.

## 4. `validate_unweighting.rs`

Four enforced thresholds, none of which carried any recorded calibration before
this session. The statistic is formed over `GEN_SEEDS`; the shape histograms
pool every seed against one weighted reference pass.

| file · row | statistic | seeds forming (before → after) | seeds calibrating | threshold | measured 5-seed value | headroom × |
|---|---|---|---|---|---|---|
| `validate_unweighting.rs` · 5 rows | `SIGMA_PULL_LIMIT` — pull of the seed-mean σ vs VEGAS / weighted ref | 4 → **5** | 0 → **5** | 3.5 | `1.56` (`ee_to_tatah` vs VEGAS); was `1.98` at four seeds | 2.2× (b) — was 1.77× |
| `validate_unweighting.rs` · 5 rows | `SIGMA_REL_LIMIT` — worst \|rel\| of the same | 4 → **5** | 0 → **5** | 0.03 | `5.77e-3` (`ee_to_mumua` vs weighted ref) | 5.2× — was 4.9× |
| `validate_unweighting.rs` · 12 shape columns | `SHAPE_CHI2_LIMIT` — worst χ²/dof | 4 → **5** | 0 → **5** | 3.0 | `1.25` (`ee_to_tatah` `cos θ`) | 2.4× — was 2.2× |
| `validate_unweighting.rs` · 12 shape columns | `SHAPE_PULL_LIMIT` — worst bin pull over ~75 judged bins | 4 → **5** | 0 → **5** | 5.0 | `2.54` (`ee_to_tatah` `cos θ`) | 2.0× (b) — was 1.9× |

## 5. `vibegraph-cli/tests/cli_generate_proton.rs` — `SIGMA_MAX_REL`

| file · row | statistic | seeds forming | seeds calibrating | threshold | measured 5-seed value | headroom × |
|---|---|---|---|---|---|---|
| `cli_generate_proton.rs` · `pp_to_llj_fixed` | sample σ vs its own integration's, relative | **1** | 5 (recorded) → 5 (re-measured) | 0.015 | `4.276e-3` (worst of 5) | 3.5× |

Per seed: `{+0.355, −0.428, +0.009, −0.264, −0.396} %` — the gate's own seed
reads `+0.355 %` (`4.2×` inside). The recorded calibration
(`{−0.36, −0.14, −0.62, +0.47, +0.29} %`) has the same magnitude and different
signs, which is what a single-seed cell is: the side of the integration a
sample lands on is not a property of the seed.

The file's other assertions are deterministic and carry no seed statistic:
momentum balance `< 1e-9`, on-shellness `< 1e-8`, `AQCDUP` within `4e-7` of the
grid, `mean XWGTUP / XSECUP − 1 < 1e-6`, `≥ 24` flavour assignments, and the
`MEASURED_STOP_Q2` pin. They are outside this census by construction.

## 6. `vibegraph-cli/tests/validate_samples_proton.rs`

| file · row | statistic | seeds forming | seeds calibrating | threshold | measured 5-seed value | headroom × |
|---|---|---|---|---|---|---|
| `validate_samples_proton.rs` · all 10 gating rows | min `p` over rows × seeds × columns | 3 | 3 (the same reading) | 1e-4 | `1.439e-3` (`pp_to_jj` `χ² flavour`, seed `0x5a4d1003`) | 14.4× (b) |
| `validate_samples_proton.rs` · `dσ/dm_ll`, 2 DY cards | `SPECTRUM_MAX_PULL` — worst bin pull over ~25 judged bins | 3 | trial count, no sweep | 4.0 | see §7 | (b) |

The five-seed minimum equals the three-seed one; only `pp_to_llj_fixed`
(`7.865e-2 → 3.007e-2`) and `dy13_default` (`1.235e-2 → 7.812e-3`) lower their
own. The margin here is two orders larger than the fixed-beam file's, so nothing
in the proton `samples` layer is close to its floor.

`pp_to_jj`'s manifest `samples` note records "worst KS p 7.9e-2 (phi(j1)),
ICOLUP chi-squared p 0.105–0.263" and does not mention the `flavour` χ² at
`1.439e-3`, which is the row's — and the whole file's — actual minimum. The note
is incomplete rather than wrong; flagged, not edited (bookkeeping is the
close-out's).

## 7. The thin list, and what was done about it

Class-(a) statistics under `2×` over their five-seed value, with the remedy
applied. **No threshold was widened.**

| statistic | 5-seed value | threshold | before | after | remedy |
|---|---|---|---|---|---|
| `validate_hadronic.rs` `pp_to_jj` `rel` vs `JJ_MAX_REL` | `+3.329e-3` | 0.005 | 1.5×, statistic formed on 3 seeds | 1.5×, statistic formed on **5** | `JJ_SEEDS` 3 → 5 (the note-34 remedy, `LLJ_SEEDS`' precedent). The ratio does not move because the residual is a converged offset, not scatter — what moves is that the gate no longer reads a three-seed mean of it. `χ²/dof` goes from `0.80` on 2 dof to `1.40` on 4, matching the bound's own calibration. Recorded in `JJ_SEEDS`, `JJ_MAX_REL` and `JJ_MAX_CHI2_PER_DOF`'s doc comments and in the manifest note. |
| `validate_sigma.rs` `ddx_to_epemg` `\|rel\|` vs `rel_tol` | `6.296e-3` worst of 5, mean `+4.509e-3` | 0.01 | 1.6× against a *stale* recorded `5.6e-3` | 1.6× recorded | Measurement recorded in `plan_for`'s comment, with the diagnosis: a converged `+0.45 %` offset against a `0.20 %` reference error, flat to `+0.42 %` at four times the budget. The bound is not widened and the row is not demoted — it agrees inside twice the reference's own error, which is what `rel_tol` was set from. What buys margin here is points, not tolerance: `4×` reads worst `5.5e-3`. **Recommended to the manager, not done:** a budget decision belongs with the row's ladder. |
| `validate_sigma.rs` `gux_to_epemux` `\|rel\|` vs `rel_tol` | `2.628e-3` worst of 5, mean `−6.273e-4` | 0.005 | 1.9× against a stale recorded `1.51e-3` | 1.9× recorded | Same: measurement recorded. Here the margin is eaten by *scatter* and not an offset — one seed of five at `−2.63e-3` against four inside `8.7e-4`, and `4×` takes the worst to `1.37e-3` and the mean to `−1.4e-4`. Again bought back with points. |
| `validate_unweighting.rs` `SIGMA_PULL_LIMIT` | `1.98` at 4 seeds | 3.5 | **1.77×** | **2.2×** (`1.56`) | `GEN_SEEDS` 4 → 5. The σ comparison is a pull on a seed-mean whose error is estimated from the same seeds, so the fifth seed buys the denominator. Measured before and after; every relative distance moves by less than `0.14 %`. |
| `validate_unweighting.rs` `SHAPE_PULL_LIMIT` | `2.68` at 4 seeds | 5.0 | 1.9× | 2.0× (`2.54`) | Same change. This is a class-(b) threshold over ~75 bins where `≈2.7` is the expected maximum, so `2×` is the designed operating point and the improvement is incidental. Recorded rather than acted on further. |

Class-(b) statistics whose ratio is under `2×` and where the remedy menu does
**not** apply, with the reason recorded rather than a change made:

* `validate_samples.rs` `P_FLOOR` at `1.57×`. Adding seeds adds draws from the
  null, which lowers the expected minimum and *raises* the flag rate. The
  five-seed measurement is identical to the three-seed one and both are recorded
  in the constant's doc comment. `ee_to_wpwm`'s `pt(w+)` remains the column to
  watch, and the standing prescription (record, mark the row informational, file
  the disagreement — never move the floor) is already written there.
* `validate_sigma.rs` `PULL_LIMIT` on nine rows reading `1.3×`–`1.9×`. A `3.5σ`
  bound on the worst of five roughly standard-normal draws is *supposed* to read
  about `2×`; the census's per-row worst pulls run `0.51`–`2.65` (excluding
  `ee_to_mumua`) with a mean of `1.562` against a half-normal expectation of
  `1.57`, which is the
  distribution a correctly sized limit produces. `P(|pull| > 3.5)` for a single
  draw is `4.7e-4`, so the gate's own one-seed reading over 31 rows flags
  spuriously about once in 70 full runs.
  Recorded in `PULL_LIMIT`'s doc comment.
* the hadronic rows' `|pull| < 3.0`, same argument; the tightest is `pp_to_jj`
  at `2.0×` (`+1.47`).
* `validate_hadronic.rs` `pp_to_llj_dyn`'s `χ²/dof` at `1.8×` (`2.17` against
  `4.0`). Already five-seed, so there is no seed remedy to apply; `χ²_4/4 > 2.17`
  has probability `0.070`, so this is a `1.5σ` upward draw rather than a margin.
  It is on the follow-up list in §3 for a different reason — the recorded value
  was `0.66`.

## 8. For the sessions downstream

* **B1** (massive fixed beams) re-rolls three `Plan::Info` toy rows and moves
  four light-massive rows by `O(m²/ŝ)`. Of its controls, `ee_to_ttx_smlimit` sits
  at `4.4×` and `tata_to_ttx_tensor4f` at `3.5×` on `rel_tol`; both have room for
  the moves the plan predicts. `ll_to_qqx_toy_*` are at `5.0×`, `3.6×` and
  `16.3×`. None of B1's may-move set is on the thin list.
* **B3** (MadGraph's channel set) re-rolls nine σ rows. Their measured `rel_tol`
  headroom: `ee_to_ee` `27.9×`, `ud_to_epemud_qcd0` `2.5×`, `ee_to_ttx_smeft`
  `4.3×`, `ee_to_mumu_4f` `3.8×`, `ee_to_ttx_dipole` `3.1×`, plus the proton rows
  whose subprocesses merge — `pp_to_jj` at `1.5×` is the one to watch and is
  exactly the row §7 moved to five seeds first. `gg_to_gg_cg`, `bbx_to_h_identity`,
  `gg_to_h_cpeven` and `gg_to_h_cpodd` have no gated σ cell, so they are not in
  this census.
* **B6** promotes `gg_to_gg_cg` and adds a `SCALUP` column to `samples`. A new
  `samples` column is `1/7` to `1/21` more draws from the null per row, so it
  lowers the expected minimum `p` across the whole file; `ee_to_wpwm`'s `1.57×`
  is where that will show first. Worth pre-registering.
* **The re-recording job §1c opens is the one follow-up worth filing.** Ten
  recorded calibrations across `plan_for`'s QCD, resonant and spine comments no
  longer describe the tip, and this session corrected only the four `ℓℓj` ones
  because two of those were thin. Doing the rest is one command's output pasted
  into comments — but it should happen *after* B1 and B3, not before, since both
  re-roll streams and would stale the new numbers again. `probe_gate_row_seed_
  headroom` is the standing instrument: every gated row, five seeds, 207 s.
* **Run each session's gate suites one at a time on this host.** This session's
  `pixi run --skip-deps validate` was SIGKILLed by the OS in `validate_samples`
  with three sprint worktrees running heavy suites at once. Nothing was wrong
  with the code — the four remaining suites passed when re-run serially — but a
  killed run leaves a partially cleared report directory, and "the gate failed"
  and "the gate was killed" look the same in a log tail if only the exit code is
  read.

## 9. Gate results behind this census

Evidence file: `/tmp/b0-evidence.log` (command + full output per item).

| command | result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo clippy --workspace --all-targets --features extended-validation -- -D warnings` | exit 0 |
| `pixi run --skip-deps validate` | **SIGKILLed by the OS** partway through `validate_samples`, three sprint worktrees running heavy suites concurrently. Every suite it completed passed: `cli_generate_proton` 3 passed, `validate_samples_proton` 12 passed, `validate_hadronic` all σ arms passing including `[jj] 5 seeds χ²/dof 1.40 pull +1.47`, plus the whole hermetic and diagram/amplitude/colour/scale layer. |
| the four remaining suites, re-run serially | `validate_samples` exit 0 (5 passed, 2 ignored, 170.55 s) · `validate_sigma` exit 0 (5 passed, 29 ignored, 39.14 s) · `validate_unweighting` exit 0 (1 passed, 13.32 s) · `validate_samples_proton` exit 0 |
| `pixi run --skip-deps validation-report` | exit 0 — "51 rows × 4 categories: the measured cells are the declared cells", 176 measured (166 ✅, 10 ⚠️, 4 ⏳, 24 uncovered), identical to the `e73b158` baseline note 36 §1 counted |

No `vibegraph integrate` child aborted with libmalloc's "pointer being freed was
not allocated" in any of the three `validate_samples_proton` invocations this
session (the probe's 10-row × 5-seed sweep, the killed run's, and the re-run's).
