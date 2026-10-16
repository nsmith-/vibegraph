---
type: Session Report
title: "V4 report: MadEvent owns the unmatched g q > t t~ q deficit"
description: "A fixed-density estimate built on MadGraph's standalone SMATRIX and LHAPDF gives 137.928 ± 0.067 pb on the unmatched fixed-μ gq card. vibegraph reads 137.903 (−0.02 %, 0.3σ), MadEvent 137.12–137.18 (−0.55 %, 3.6σ), all of it in channel G2 (seed χ²/dof 7). hard_survey = 2 removes it (−0.03 %); splitting the symmetric configuration, sde_strategy = 2 and 10× the events do not. On the matched row, 20 hard_survey = 2 directories read @1 578.24 ± 0.52 against vibegraph's 578.78 (pull 0.88), with gg, gq and qq̄ each within 1.2σ. The cell can gate against a reference re-banked with that setting."
status: draft
tags: [mlm, sigma, ttx, madevent-reference, madgraph-defect, fixed-density-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources: []
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
This report is from the validation-dev agent. It is a diagnosis only. The
only file committed is this report.

**Card.** Every number is for V3's unmatched fixed-scale card: `p p > t t~ j`
with `j = u c d s u~ c~ d~ s~`, 13 TeV, `lhaid 247000`, `ickkw = 0`,
`xqcut = 0`, `ptj = 30`, `etaj = 5`, μR = μF = 173 fixed
(`ttx3/cards/fix_run_card.dat`), unless a row says "matched row". Only g q,
g q̄ and their mirrors contribute.

**MadEvent provenance.** Pinned 3.7.1 through `mg5_pinned.sh` under
`pixi run -e madgraph`, unpatched (`VG_MG5_ROOT` unset), `MGVersion.txt`
3.7.1 on every run, 2 cores, each seed in a fresh copy of a generated,
never-run directory. Variants change only what their row says: the run card
(`sde_strategy`, `hard_survey`, `nevents`) or, for "split", the generated
`symfact_orig.dat` and `symperms.inc` before compilation. The matched-row runs
copy V1's `ttx/ttx-pristine-fresh` with V3's driver (the banked row card,
`vector_size = 1`, `PDFSUP 247000`, first call on `PP`, all checked per run).

**Scratch.** Everything is under
`/tmp/claude-0/-home-user-vibegraph/e8f0bc03-01f3-5b41-b6ee-7a04856445c0/scratchpad/ttx4/`:

- `ttx4-fixed.py`: the fixed-density estimator and its volume check;
  `ttx4-chanor.py`: the same sampler with per-diagram |AMP|² for MadEvent's
  channel shares; `ttx4-ptcheck.py`: per-point checks of the standalone
  matrix elements;
- `sa/`: the MadGraph 3.7.1 standalone output for `g u > t t~ u`,
  `g u~ > t t~ u~`, `u g > t t~ u`, `u~ g > t t~ u~`, with `eval_pts` and
  `eval_amp2` drivers;
- results: `ttx4-sig-results.txt`, `ttx4-ch-results.txt`, `ttx4-vol.txt`,
  `ttx4-me-results.txt`; every MadEvent run's `results.dat`,
  `P*/G*/results.dat` and `P*/symfact.dat` in `me-keep/*.tgz`;
- analysis: `ttx4-chan4.py` (per channel across seeds), `ttx4-iter.py`
  (per group, reported against equal-weight, V3's rule);
- drivers: `ttx4-me-var.sh` (V3's variant driver with a patch hook),
  `ttx4-me-row.sh` (V3's row driver with a patch hook), `ttx4-split.sh`,
  `ttx4-*-queue*.sh`.

## Verdict

**MadEvent is the outlier. vibegraph is right on this card.**

1. The adaptation-free estimate is **137.928 ± 0.067 pb** (20 seeds in two
   independent runs). vibegraph reads 137.903 ± 0.060, **−0.018 % (0.28σ)**.
   MadEvent reads 137.124 ± 0.211 reported and 137.177 ± 0.197 equal-weight,
   **−0.58 % and −0.54 % (3.6σ)**.
2. **The deficit is all in MadEvent's channel `P1_gq_ttxq/G2`**: −0.78 pb
   (−0.92 %, 3.5σ) against the oracle's share for that channel. G1, G4 and
   G5 agree to 0.03 pb. G2 is configuration 2 with configuration 3 folded into
   it (`symfact 3 → −2`, t ↔ t̄): the incoming quark emits a t-channel gluon,
   and g g* → t t̄ through a t-channel top. It carries 61 % of σ. It is the
   only channel whose seeds scatter beyond their errors: χ²/dof 7.1 at 10k
   events, 7.5 and 6.0 on V3's dynamical-scale runs at 10k and 100k. Within a
   seed its 24 iterations agree with each other (χ²/dof 0.83). So every
   iteration of a seed shares one grid-level offset, which the quoted error
   does not contain.
3. **What removes it is a better survey.** `hard_survey = 2` (4× survey
   points, 3 more survey iterations, refine `min_events` 10 000) gives
   137.887 ± 0.125, **−0.03 % (0.3σ)**, χ²/dof 1.17, and G2 84.44 ± 0.11
   against the oracle's 84.58 ± 0.07. Three other changes do not remove it:
   - 10× the events (V3: −0.59 % at 100k, dynamical scale);
   - integrating configuration 3 as its own channel ("split"): −0.36 % at
     10k, −0.55 % at 100k, where G2 and G3 now each sit −0.7 % and −0.9 % low
     at χ²/dof 28 and 11;
   - `sde_strategy = 2`: −0.44 %.

   The grid the survey hands to refine under-samples part of G2's phase
   space, and refining that grid with more events does not repair it.
4. **On the matched row the same setting closes all of `@1`.** Twenty fresh
   `hard_survey = 2` directories read `@1` 578.24 ± 0.52 against vibegraph's
   578.78 ± 0.33 (+0.09 %, pull 0.88). gg, gq and qq̄ are each within 1.2σ.
   V3's gg residual was the same survey effect.
5. Per scope 3 there was nothing to compare per point: vibegraph is not the
   outlier. The vibegraph-drawn per-point check was not run (see Brief
   corrections).

## Table

Estimator × σ × error × seeds × χ²/dof. Errors are the seed rule
(max of rms-quoted/√n and sd/√n). "vs oracle" uses the combined oracle,
137.928 ± 0.067.

| estimator | σ (pb) | seeds | χ²/dof | vs oracle |
|---|---|---|---|---|
| **fixed-density oracle A** (`ttx4-fixed.py`, card top width) | 137.902 ± 0.090 | 10 × 4·10⁶ | 0.70 | |
| **fixed-density oracle B** (`ttx4-chanor.py`, t-channel top width 0) | 137.961 ± 0.100 | 10 × 4·10⁶ | 1.23 | A − B = −0.059 ± 0.134 |
| **oracle, A and B combined** | **137.928 ± 0.067** | 20 | | reference |
| vibegraph `vg-fixed`, 200k × 8 (V3) | 137.903 ± 0.060 | 5 | | −0.018 %, −0.28σ |
| MadEvent 10k, reported (V3) | 137.124 ± 0.211 | 10 | 4.67 | **−0.58 %, −3.63σ** |
| MadEvent 10k, equal-weight (V3) | 137.177 ± 0.197 | 10 | | **−0.54 %, −3.61σ** |
| MadEvent 10k, split, reported | 137.433 ± 0.139 | 10 | 1.63 | −0.36 %, −3.21σ |
| MadEvent 10k, split, equal-weight | 137.626 ± 0.149 | 10 | | −0.22 %, −1.84σ |
| MadEvent 100k, split, reported | 137.174 ± 0.136 | 5 | 8.31 | −0.55 %, −4.98σ |
| MadEvent 100k, split, equal-weight | 137.307 ± 0.179 | 5 | | −0.45 %, −3.25σ |
| MadEvent 10k, `sde_strategy = 2`, reported | 137.324 ± 0.198 | 10 | 2.68 | −0.44 %, −2.89σ |
| MadEvent 10k, `sde_strategy = 2`, equal-weight | 137.368 ± 0.197 | 10 | | −0.41 %, −2.69σ |
| **MadEvent 10k, `hard_survey = 2`, reported** | **137.887 ± 0.125** | 10 | 1.17 | **−0.03 %, −0.29σ** |
| MadEvent 10k, `hard_survey = 2`, equal-weight | 137.943 ± 0.208 | 10 | | +0.01 %, +0.07σ |

The seeds' χ²/dof is taken about their own mean with their quoted errors.
The oracle's own χ²/dof (0.70, 1.23) shows its quoted errors are honest, and
the error used is the seed spread where that is larger.

### Per channel

The oracle's channel shares weight each point's integrand by MadEvent's
single-diagram-enhancement factor `AMP2(c) / Σ AMP2` (`sde_strategy = 1`,
`tmin_for_channel = −1`, so `get_channel_cut = 1`). They use the
`CONFSUB` map of `P1_gq_ttxq`: configuration c is diagram c of `g u > t t~ u`
and of `g u~ > t t~ u~` in standalone numbering (MadEvent's `matrix2` swaps
diagrams 1 and 4, and `CONFSUB(2,·)` swaps them back). MadEvent errors are
sd/√n.

| channel | oracle B | MadEvent 10k | split 10k | split 100k | `hard_survey = 2` |
|---|---|---|---|---|---|
| G1 | 0.6972 ± 0.0003 | 0.7005 ± 0.0025 (+0.47 %, 1.3σ) | 0.6975 ± 0.0021 | 0.6979 ± 0.0021 | 0.6961 ± 0.0015 |
| G2 = configs 2 + 3 | **84.580 ± 0.067** | **83.802 ± 0.210 (−0.92 %, −3.5σ), χ²/dof 7.1** | 84.194 ± 0.086 (−0.46 %, −3.5σ) | 83.917 ± 0.166 (−0.78 %, −3.7σ) | **84.443 ± 0.109 (−0.16 %, −1.1σ), χ²/dof 1.07** |
| config 2 alone | 42.670 ± 0.034 | — | 42.456 ± 0.060 (χ²/dof 0.95) | 42.370 ± 0.138 (χ²/dof 28) | — |
| config 3 alone | 41.910 ± 0.035 | — | 41.738 ± 0.062 (χ²/dof 0.75) | 41.547 ± 0.091 (χ²/dof 11) | — |
| G4 | 8.5027 ± 0.0032 | 8.473 ± 0.014 (−0.35 %, −2.0σ) | 8.498 ± 0.019 | 8.496 ± 0.009 | 8.498 ± 0.014 |
| G5 | 44.181 ± 0.033 | 44.149 ± 0.039 (−0.07 %) | 44.043 ± 0.060 | 44.064 ± 0.063 (χ²/dof 8.0) | 44.251 ± 0.033 |

Configurations 2 and 3 are not equal (42.67 against 41.91, 1.8 %). |M|² of
g q → t t̄ q is not symmetric under t ↔ t̄ (the tree-level charge asymmetry
of the qg channel), so the share of each diagram differs. MadEvent's folding
samples both through the permutation, so it is unbiased in principle, and
splitting does not change the deficit at 100k. The folding is not the cause.

## How the oracle works, and what checks it

`ttx4-fixed.py` draws every point from one fixed density and never adapts:

- τ = x1 x2 from a mixture of uniform and a truncated exponential in
  ln(τ/τ0), τ0 = (2 m_t)²/s;
- the partonic rapidity uniformly;
- the jet's p_T (log-uniform 80 %, flat 20 %, on [30, p_T^max(ŝ)]);
- its CM rapidity uniformly on the kinematic range, φ uniformly;
- t t̄ isotropic in their rest frame.

The weight is f_a f_b |M|² / (2ŝ) × dΦ3 / density, with the cut
|η_j| < 5 in the lab. |M|² is MadGraph 3.7.1 standalone `SMATRIX` for the four
subprocesses (5 diagrams each, QCD only, so u = c = d = s), rescaled
(α_s(173)/0.118)³ with α_s(173) = 0.1180577518 from LHAPDF. The densities
are LHAPDF 247000 from `validation/pdf` (byte-identical to the env's copy).
4·10⁶ points per seed take 2.7 minutes on one core. The per-point weight has
coefficient of variation 4.1, and the largest weight is 7·10⁻⁵ of the sum: no
heavy tail.

Checks:

- **Phase-space volume** (`ttx4-vol.txt`, 4·10⁶ points each). Massless, no
  cut, against s/(256π³): 125.9815 ± 0.0208 against 125.9826 (−0.05σ).
  Massive tops against the recursive Φ2 ⊗ Φ2 quadrature, at √s = 400, 1000
  and 13000 GeV, with and without the 30 GeV p_T cut: pulls −1.17, −1.20,
  −0.01, +0.35. The Jacobian is right to ≤ 0.05 %, ten times below the
  effect.
- **Per point** (`ttx4-ptcheck.py`, 18 988 points):
  - `u g` standalone at (p_a, p_b) equals `g u` at (p_b, p_a) to 8·10⁻¹⁶,
    and likewise for ū. So the mirrored terms are right.
  - `eval_amp2` and `eval_pts` agree exactly.
  - The t-channel top width (standalone keeps Γ_t, MadEvent zeroes it)
    changes |M|² by −3.6·10⁻⁵ on average. A and B, which differ in this,
    agree to 0.06 ± 0.13 pb.
- **Seeds.** 10 per oracle, χ²/dof 0.70 and 1.23.

What the oracle shares with each side: the same PDF set, the same param card,
MadGraph's own matrix element (the same as MadEvent's, diagram for diagram)
and LHAPDF's α_s (the same as both). It shares no sampling and no adaptation
with either side. **What it cannot see:** anything common to all three, i.e.
a defect of the 3.7.1 matrix element itself, or of the PDF set or α_s. V2
already pinned vibegraph's per-point integrand to MadEvent's `DSIG1`. A
flux or conversion-constant error would also hide here, but it would have to
be the same in two independently written integrators.

## Scope 4: what @1 reads once its gq part is corrected

The matched row's gq group cannot be computed by this oracle, which has no
clustering and no `rewgt`. Two routes give the correction instead.

**(a) Transfer the unmatched deficit.** MadEvent's equal-weight gq is
−0.547 ± 0.152 % from the oracle on the unmatched card. Applying that fraction
to the matched row's gq (147.133, V3) adds 0.805 ± 0.223 pb. The gq group
then reads 147.938 ± 0.369 against vibegraph's 148.414 ± 0.073 (+0.32 %,
1.26σ). For `@1` (vibegraph 578.780 ± 0.327, V3):

| reference | uncorrected | gq corrected |
|---|---|---|
| bank, 575.717 ± 0.788 | +0.53 %, pull 3.59 | 576.522: +0.39 %, pull **2.56** |
| fresh equal-weight (V3), 576.164 ± 0.605 | +0.45 %, pull 3.80 | 576.969: +0.31 %, pull **2.50** |
| fresh reported (V3), 575.290 ± 0.520 | +0.61 %, pull 5.68 | 576.096: +0.47 %, pull 4.11 |

**(b) Measure it.** The matched row in 20 fresh pinned directories (seeds
20262101–20) with `hard_survey = 2` added to the banked row card, and nothing
else changed. `npts` per channel went from 7009 to 28009, so the setting took
effect. Reported σ, seed rule, against vibegraph's fixed build (V3's `fixed`
column, and V2's per-group CM runs):

| part | MadEvent `hard_survey = 2` (20 seeds) | χ²/dof | vibegraph | rel | pull |
|---|---|---|---|---|---|
| `@0` | 513.159 ± 0.113 | 0.95 | 513.219 ± 0.179 | +0.012 % | 0.29 |
| `@1` | **578.239 ± 0.520** | 0.79 | 578.780 ± 0.327 | **+0.094 %** | **0.88** |
| total | 1091.398 ± 0.532 | 0.82 | 1091.999 ± 0.421 | +0.055 % | 0.89 |
| `P1_gg_ttxg` | 411.142 ± 0.435 | 0.49 | 411.645 ± 0.25 | +0.12 % | 1.00 |
| `P1_gq_ttxq` | 148.219 ± 0.297 | 1.21 | 148.414 ± 0.073 | +0.13 % | 0.64 |
| `P1_qq_ttxg` | 18.878 ± 0.022 | 0.77 | 18.909 ± 0.011 | +0.16 % | 1.23 |

For comparison, the same driver without `hard_survey` (V3, 20 seeds) read
`@1` 575.290 ± 0.520 (pull 5.68). Two runs of 20 seeds each with the
symmetric configurations split instead read:

- gq split only: `@1` 575.683 ± 0.554 (pull 4.81), gq 147.624 ± 0.240
  (pull 3.16);
- every P1 directory split: `@1` 577.105 ± 0.550 (pull 2.62), gg 410.689 ±
  0.487 (pull 1.75), gq 147.524 ± 0.251 (pull 3.40).

So `hard_survey = 2` moves `@1` by +2.95 pb, and splitting does not do the
same.

**Reading.**

- **Against the current bank, the cell cannot gate.** It reads pull 3.59.
  Route (a) only reaches about 2.5σ, because it corrects gq and leaves gg's
  +0.36 %.
- **Against a `hard_survey = 2` reference, every part of the row agrees.**
  `@1` reads pull 0.88, `@0` 0.29 and the total 0.89. That includes gg: the
  gg residual V3 left at +0.36 % (2.3σ against equal-weight) is +0.12 %
  (1.0σ) here, so it was the same survey effect and not a generic multi-leg
  offset.
- **The cell can therefore gate (|pull| < 3) against a re-banked reference
  made with `hard_survey = 2`.** Twenty fresh directories give 578.24 ±
  0.52 on `@1`, the size of the current bank's error. It cannot gate against
  the current bank, with or without a transferred gq correction.
- **The equal-weight recombination stops mattering.** With
  `hard_survey = 2` the equal-weight values (`@1` 579.57 ± 1.04) carry
  larger errors than the reported ones, because the refine leaves fewer
  iterations per channel. The reported σ is the one to bank.

## Brief corrections

- **"vibegraph 118.494 / 137.903 at dynamical / fixed μ in V3".** Right.
  Only the fixed-μ value is compared here: the oracle has no dynamical scale.
  vibegraph matches the oracle at fixed μ, and V2 pinned its per-point
  dynamical-scale integrand to `DSIG1`. So for the dynamical card vibegraph
  itself is the reference.
- **Scope 3 needed no per-point run.** vibegraph agrees with the oracle to
  0.02 %, so there is no factor to find. The probe test was not written,
  and nothing on this side was touched (`git diff --stat` empty).
- **"MadEvent under-integrated specific channels and its seeds scattered at
  χ²/dof about 4, as here".** At the process level, yes (4.7). The
  per-channel scatter is larger and confined to G2 (7.1, and 28 and 11 for
  the split configurations at 100k).
- **The scope went past "stop there" in one respect.** Naming G2 did not say
  what reference could gate scope 4. So this session ran cheap MadEvent
  variants: split, `sde_strategy = 2` and `hard_survey = 2` on the
  unmatched card, and split and `hard_survey = 2` on the matched row. A run
  takes 15–100 s. Nothing was changed in MadGraph's source or in banked
  data. A second batch of 20 split-all row seeds was started and stopped
  after one unfinished seed, when `hard_survey = 2` made it moot. Its run
  directory was deleted.
- **"#28 ... as here".** The pattern matches #28 (one family of channels,
  seed χ²/dof ≫ 1, recombination closing only part of it). Whether #28's
  deficit also yields to `hard_survey = 2` was not tested (Found 5).

## Falsifiers

- A third, independent integrator that reproduces MadEvent's 137.1 on this
  card. It would have to disagree with both the oracle and vibegraph, which
  agree to 0.02 %.
- A MadEvent configuration with a better survey (`hard_survey = 2`) that is
  still low on G2 with more seeds. 10 seeds put it at −0.16 ± 0.15 % on G2.
- An oracle change in the density (k, mixture weights) that moves σ by more
  than its error. That would mean a support gap, which the volume checks and
  the absence of a heavy tail argue against.

## Banking request

**One request, for the user on the M3 host.** It is new banked reference
data, so this session did not produce it.

- **Row:** `pp_to_ttx_0j1j_mlm`.
- **Change:** add one line, `2 = hard_survey`, to
  `validation/madgraph/pp_to_ttx_0j1j_mlm_run_card.dat`. Nothing else
  changes: no patch, the pinned 3.7.1 and `VG_MG5_ROOT` unset.
- **Runs:** the row's samples-grade run and its cross-section seeds as
  `gen_mlm_references.sh` makes them, plus independent fresh directories for
  this row (`FRESH_ROWS` including `pp_to_ttx_0j1j_mlm`, the default 20
  `FRESH_SEEDS` 20261101–20). Something like
  `ROWS=pp_to_ttx_0j1j_mlm FRESH_ROWS=pp_to_ttx_0j1j_mlm` on that script.
  The reference should read the fresh directories, as `pp_to_ll_0j2j_mlm`'s
  does.
- **Expected outputs** (this session's 20 fresh directories, Linux, same
  card): `@0` 513.16 ± 0.11, `@1` 578.24 ± 0.52, total 1091.40 ± 0.53 pb, seed
  χ²/dof about 0.8. A macOS bank should agree within those errors.
- **Knock-on effects:**
  - `hard_survey` raises refine's `min_events` and `max_events`, so
    `run_01`'s event file changes. The MLM dumps that `validate_mlm_dumps`
    replays must be regenerated with it.
  - `<MGRunCard>` gains a field. `mg_run_card_matches_madevents_banner`
    compares only fields both sides carry, but that should be checked.
  - After the re-bank, `pixi run validate-mlm-sigma` on the fixed tree should
    read `@1` at pull about 1. The cell can then be promoted from `info` to
    `gate`, after the five-seed agreement its note asks for.

## Gates

None: no source change, and the only committed file is this report.
`git diff --stat` was empty before it was added. `python3 scripts/kb.py index`
and `python3 scripts/kb.py lint` were run before the commit.

## Found

1. **MadEvent 3.7.1 under-integrates `g q > t t~ q` by 0.55 % in channel G2
   at default survey settings, and more events do not repair it.** Evidence:
   the oracle and vibegraph agree to 0.02 %. MadEvent is −0.54 to −0.59 % at
   10k and 100k events, and G2 alone is −0.92 % with seed χ²/dof 7. To
   close: record it under
   [MadGraph defects](../../../validation/madgraph-defects.md), with
   `hard_survey = 2` as the setting that removes it (−0.03 % on this card).
   Then decide whether every banked MadEvent σ row should carry
   `hard_survey = 2`. #28's G7–G9 deficit looks like the same family, worth
   one `hard_survey = 2` run there.
2. **The `pp_to_ttx_0j1j_mlm` integrals cell's reference reads MadEvent's
   default survey.** It cannot gate against the current bank (pull 3.59).
   Against 20 `hard_survey = 2` directories it reads 0.88. To close: the
   banking request, then promote the cell.
3. **V3's gg residual was the same survey effect.** With `hard_survey = 2`,
   gg reads +0.12 % (1.0σ), down from +0.57 % reported and +0.36 %
   equal-weight. The generic-offset reading of V3's Found 2 does not hold for
   this row. To close: nothing beyond Found 1 and the re-bank.
5. **Every banked MadEvent σ row at default survey settings is suspect at the
   0.2–1 % level** where a channel's survey grid misses a region. #28's
   top-free G7–G9 (χ²/dof 8–19), the 2→4 offset item, and any row whose
   channels show seed χ²/dof ≫ 1 are candidates. To close: one
   `hard_survey = 2` run of 10 seeds per such row is cheap (15–40 s per seed
   here). It tells a survey effect from a real one.
4. **A per-event localisation of G2's missing region is still open.** The
   oracle can tally σ in slices of jet p_T, jet η and m_tt̄ for comparison
   with MadEvent's default-survey events. This would name the region the
   survey misses. It is not needed for the reference question.
