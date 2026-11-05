---
type: Session Report
title: "H-M2 report: massive incoming partons on the survey's own cards"
description: "Both survey cards with massive incoming partons now match LHAPDF MadGraph over five seeds a side: LL_HAHM (c c~ at MC = 1.42) at +3.3e-4, pull +0.66, once ptonium/etaonium, which MadGraph never reads, are lifted; ZprimeToTauTau (every quark massive, MLM 0-2 jets) at +1.2e-3 in total and -1.5e-3/+1.5e-3 on its massive no-jet parts. The seven explicit-ordering mirror rows, massive and massless, match too, all sitting about 1e-3 high together. The earlier +0.53% on LL_HAHM was the alpha_s of nn23lo1, measured per event."
status: draft
tags: [proton, kinematics, masses, madgraph-parity, cms, sprint-h, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---
Session H-M2 of [sprint H](../sprint.md), branch `sprinth-massive-incoming`,
worktree `/home/user/wt-massive-incoming`. Items:
`massive-incoming-partons-refused` and, after #96 merged,
`explicit-initial-state-ordering-counted-twice`.

Scratch paths are under
`$SP = /tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/H-M2`;
none is committed. MadGraph is the pinned submodule (3.7.1, `b7687064`), run
as `python3 /home/user/vibegraph/research/refs/mg5amcnlo/bin/mg5_aMC`. Two
MadGraph PDF configurations appear below:

- **LHAPDF** (the gating one): LHAPDF 6.5.4 at `/home/user/lhapdf`,
  `LD_LIBRARY_PATH=/home/user/lhapdf/lib`, `pdlabel = lhapdf`,
  `lhaid = 247000`, the set and αs vibegraph reads. Driver
  `$SP/H-M2-mglaunch.sh`. The python LHAPDF module is absent, so MadGraph's
  `update_dependent` cannot rewrite SMINPUTS 3 from the PDF ("Failed to access
  python version of LHAPDF"); the running αs inside MadEvent comes from the
  Fortran LHAPDF interface either way.
- **nn23lo1** (context only, before LHAPDF was installed): MadGraph's built-in
  table of the same set.

vibegraph is `target/release-debug/vibegraph` at the branch tip, run with
`LHAPDF_DATA_PATH=validation/pdf`, `lhaid = 247000`, default convergence
target (0.1%) unless stated. Statistics are `$SP/H-M2-stats.py`: unweighted
mean over seeds, sd, se = max(sd/√n, quoted/n-combined), χ²/dof of the seeds
about their mean, pull on the two se's. `$SP/H-M2-collect.sh` regenerates every
table from the logs.

The cards were copied out of the extraction tree into `$SP/hahm` and `$SP/zp`
before anything ran.

## 1. `etaonium = -1` and `ptonium = 0` are inert: every reader

Grep of the whole pinned tree for `ptonium|etaonium` (tests excluded):

| site | what it does |
|---|---|
| `madgraph/various/banner.py:4318`, `:4338` | `add_param("ptonium", 1.0, legacy=True)`, `add_param("etaonium", 0.6, legacy=True)` |
| `banner.py:2863-2865` (`add_param`) | `legacy` → `legacy_parameter[name] = value; include = False`: never written to `run_card.inc` |
| `banner.py:3565-3567` (`check_validity`) | a non-default value only logs "The parameter … is not supported anymore. This parameter will be ignored." |
| `Template/LO/Source/cuts.inc:4,5,10,12` | `PTONIUM`, `ETAONIUM` declared in the `/to_cuts/` common block; nothing assigns them |
| `Template/LO/SubProcesses/setcuts.f:307-312` | the only Fortran reader, the `QUARKONIUM` block, commented out |
| `is_a_onium` (`setcuts.f`, `cuts.f`, `myamp.f`, `dummy_fct.f`, `ptj_bias.f`) | declared, never assigned; copied into `is_a_onium_save` at `setcuts.f:720`, which nothing reads |
| `Template/MadWeight/src/setrun.f:111`, `:131` | MadWeight's own reader, not on the madevent path |

"Onium" is quarkonium: the commented block would have given `is_a_onium`
legs a pT minimum, an `smin` term and a rapidity maximum. The gridpack's
own log (MadGraph 2.9.18) prints the same "not supported anymore … ignored"
warning on every step, so the parameters were legacy there too; the 2.9.18
source is not in this container.

**Measured equality**, local MadGraph, card as shipped against the card with
MadGraph's defaults (`1 = ptonium`, `0.6 = etaonium`), seeds 11–15
(nn23lo1, `$SP/mg-hahm/H-M2-drive.sh`):

```
md5sum runcardinc_*.inc | awk '{print $1}' | sort | uniq -c
     10 5ce64dc6c04f1fb80ebc026ed9118fb1          (Source/run_card.inc, all ten runs)
cmp runcardinc_shipped_$S.inc runcardinc_defaults_$S.inc  -> identical, S = 11..15
Integrated weight, shipped = defaults, per seed:
  11 0.01394827  12 0.01394229  13 0.01394622  14 0.01393925  15 0.01393528
md5 of the LHE from the first <event>, shipped = defaults, per seed:
  11 5c33d1bf5b4d  12 6c9ed46ca40d  13 02a2f7585440  14 ccd44d29a2b3  15 1fb85fcdb88b
```

**vibegraph** with the lift, seed 1: the shipped card and the defaults card
give `σ = 0.014017 ± 0.000014 pb` each, and all 158 VEGAS iteration lines are
identical. That is the equivalence argument for running vibegraph on the card
as shipped: nothing in MadGraph reads the two values, and nothing in vibegraph
does after the lift.

### The lift (the only code change of this session)

- `cuts.rs`: `ptonium` and `etaonium` leave `UNIMPLEMENTED_CUTS`.
- `runcard/classes.rs`: both become `IgnoredBenign(B_ONIUM)`, whose reason
  cites the readers above.
- New test `cuts::tests::onium_cuts_are_accepted_at_any_value_and_change_nothing`:
  at (0, −1), (1, 0.6) and (500, 0.01), `detect_unimplemented` passes and
  `Cuts::compile` equals the compiled cuts of a card without them.

Mutation (restore the two names in `UNIMPLEMENTED_CUTS`, log
`$SP/H-M2-mutation-M0.log`):

```
$ cargo test -j 2 -p vibegraph-lib --lib -- onium_cuts
test cuts::tests::onium_cuts_are_accepted_at_any_value_and_change_nothing ... FAILED
panicked at vibegraph-lib/src/cuts.rs:2306:13:
0 = ptonium
-1 = etaonium
test result: FAILED. 0 passed; 1 failed      EXIT 101
```

Re-run on the merged tree (`074a42b8` plus the uncommitted lift; script
`$SP/H-M2-onium.sh`, log `$SP/H-M2-onium.log`), final source, then the same
mutation, then the source restored:

```
$ cargo test -j 2 --profile release-debug -p vibegraph-lib --lib -- onium_cuts every_run_card_field unbindable_value ignored_physics_fields
test cuts::tests::onium_cuts_are_accepted_at_any_value_and_change_nothing ... ok
test cuts::tests::unbindable_value_does_not_excuse_other_cuts ... ok
test runcard::classes::tests::every_run_card_field_is_classified ... ok
test runcard::classes::tests::ignored_physics_fields_are_refused ... ok
test cuts::tests::no_other_unimplemented_cut_reads_an_unbindable_value_as_default ... ok
test runcard::classes::tests::every_run_card_field_has_one_stage_and_fourteen_are_generate ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1240 filtered out
EXIT 0
== mutation: ptonium/etaonium restored to UNIMPLEMENTED_CUTS
$ cargo test -j 2 --profile release-debug -p vibegraph-lib --lib -- onium_cuts
test cuts::tests::onium_cuts_are_accepted_at_any_value_and_change_nothing ... FAILED
panicked at vibegraph-lib/src/cuts.rs:2538:13:
0 = ptonium
test result: FAILED. 0 passed; 1 failed
EXIT 101
```

## 2. LL_HAHM_MS_400_kappa_0p01_MZd_14p0_ctau_1000

`p p > h > zp zp`, `p = g u c d s` and antiquarks; MadGraph generates two
subprocesses, `g g > h > zp zp` (the effective `ggh` vertex) and
`c c~ > h > zp zp` (Yukawa `ymc = 1.42`, MC = 1.42), each with its mirror.
13.6 TeV, dynamical scale choice −1 (the scale is √ŝ ≈ 125 GeV on every
event), no cuts reach `zp`.

### The MadGraph side needed one model edit (brief correction)

The pinned MadGraph refuses the bundled UFO as shipped:

```
models/import_ufo.py:905  raise UFOImportError("SMINPUTS#3 parameter should be aS")
```

HAHM puts `Gf` at SMINPUTS 3 and `aS` at 4; 2.9.18 accepted that. For the
MadGraph side only, `$SP/mg-hahm/models/HAHM_variablesw_v3_UFO/parameters.py`
swaps the two `lhacode`s and `$SP/mg-hahm/param_card_mgswap.dat` swaps the two
SMINPUTS lines (diff shown in the scratch log). Every coupling references the
parameters by name, so the physics is unchanged. vibegraph ran the bundled
model and the shipped param card unmodified. A side note for anyone running
such a model with python LHAPDF present: `update_dependent`
(`madgraph/interface/common_run_interface.py:6985-7015`) writes the PDF's
αs(MZ) into SMINPUTS 3 whatever parameter sits there, which on the unmodified
HAHM card is `Gf`, and `v = 1/(2^¼√Gf)`.

### σ, 5 seeds per side, LHAPDF 247000 on both

```
MG  seeds 11-15: 0.01401835 0.01401238 0.0140163 0.01400933 0.01400536  (± 7.18e-6 6.11e-6 7.19e-6 7.22e-6 7.83e-6)
VG  seeds 1-5:   0.014017 0.014021 0.014009 0.014023 0.014015          (± 0.000014 each; merged tree)

MG  n=5 mean 0.0140123 sd 5.23e-06 (3.74e-04 rel) se(scatter) 2.34e-06 se(quoted) 3.19e-06 chi2/dof 0.49
VG  n=5 mean 0.014017  sd 5.48e-06 (3.91e-04 rel) se(scatter) 2.45e-06 se(quoted) 6.26e-06 chi2/dof 0.15
VG - MG: +4.656e-06 (+3.32e-04 rel), pull +0.66
```

The vibegraph seeds were first run before the merge of `074a42b8`
(#94, #96); all five were rerun on the merged tree (`$SP/vg-hahm/H-M2-vg_post2_{1..5}.log`)
and reproduce σ, error and every VEGAS iteration line of the earlier run:

```
for s in 1..5: diff <(grep -o 'iteration.*' H-M2-vg_shipped_$s.log | sed 's/ at [0-9]* ns.*//') \
                    <(grep -o 'iteration.*' H-M2-vg_post2_$s.log  | sed 's/ at [0-9]* ns.*//')
seed 1 pre σ = 0.014017 ± 0.000014 pb post σ = 0.014017 ± 0.000014 pb iterlines-identical: yes n=161
seed 2 pre σ = 0.014021 ± 0.000014 pb post σ = 0.014021 ± 0.000014 pb iterlines-identical: yes n=273
seed 3 pre σ = 0.014009 ± 0.000014 pb post σ = 0.014009 ± 0.000014 pb iterlines-identical: yes n=171
seed 4 pre σ = 0.014023 ± 0.000014 pb post σ = 0.014023 ± 0.000014 pb iterlines-identical: yes n=196
seed 5 pre σ = 0.014015 ± 0.000014 pb post σ = 0.014015 ± 0.000014 pb iterlines-identical: yes n=144
```

This passes the hadronic σ gate's rule (`|pull| < 3`, `rel < 0.005`).
vibegraph's seeds took 141–270 iterations at χ²/dof 1.7–5.8 to reach 0.1%
(Found 3), and its quoted errors are conservative (χ²/dof of the seeds 0.15).

Per part, MadGraph's P directories (LHAPDF, seeds 11–15) against
vibegraph's two decompositions (seeds 1–5):

```
MG P1_gg_zpzp   0.01374 0.01373 0.01374 0.01373 0.01372       (± 6.0e-6 – 7.8e-6)
VG gg           0.013739 0.013737 0.013732 0.013743 0.013742  (± 0.000008 – 0.000014)
MG P1_ccx_zpzp  0.0002793 0.0002824 0.0002783 0.0002793 0.0002804  (± 1.1e-6)
VG cc̄           0.000278 0.000285 0.000277 0.000280 0.000273       (± 0.000002 – 0.000012)
```

MadGraph prints its per-directory σ to four digits, so the per-part
comparison is at that precision: both parts agree, the cc̄ part to its
vibegraph error of 1–4%.

**Gridpack log, informational:** 0.01106 ± 5.7e-5 pb with lhaid 325500
(αs(MZ) = 0.118), against 0.130 here. gg → h → zp zp goes as αs², so the
factor of about (0.130/0.118)² ≈ 1.21 between them is expected; it is not a
comparison.

### The nn23lo1 run (context): the +0.53% was αs, measured per event

Before LHAPDF was installed, MadGraph with `nn23lo1` gave mean 0.0139423
(sd 3.75e-4, χ²/dof 0.50), and vibegraph sat +5.36e-3 above it. The per-event
AQCDUP of the two samples (`$SP/H-M2-asratio.py`: vibegraph's
(SCALUP, AQCDUP) interpolated in ln scale at each of MadGraph's 9995 event
scales):

```
nn23lo1 MadGraph:  scale 100-150 n=9995 ratio median 1.002444 min 1.002251 max 1.002567 (ratio^2 1.00489)
LHAPDF  MadGraph:  scale 100-150 n=9995 ratio median 1.000000 min 1.000000 max 1.000000 (ratio^2 1.00000)
```

MadGraph's `P1_gg_zpzp` 0.01367 × 1.00489 = 0.013737, which is vibegraph's gg
part (0.013732–0.013743 over five seeds); the cc̄ part carries no αs at this
order and agreed. The offset is nn23lo1's own αs running from 0.130 against
the set's interpolated αs grid, not the PDF and not the masses.

### What this card can and cannot see

The c mass enters only the cc̄ part, 2% of σ. MadGraph's massless-c control
(MC = 0, Yukawa unchanged, nn23lo1, same seeds) differs from the shipped card
by −7.2e-8 pb per seed, −5.2e-6 of σ and −2.6e-4 of the cc̄ part, which is
β − 1 = √(1 − 4m²/ŝ) − 1 = −2.6e-4 at ŝ = (125 GeV)². vibegraph's MC = 0
control: 0.014015 and 0.014011 (seeds 1, 2), inside its own seed scatter.
**The σ match on this card does not discriminate the massive-incoming
treatment from a zeroed mass**; a 5e-6 effect is three orders below any σ
gate. What it does establish: the card runs as shipped, the massive part sits
in its own decomposition, and the total agrees with MadGraph at 3e-4.

### The written LHE (10000 events per side, `$SP/H-M2-lhecheck.py`)

```
== MG (LHAPDF, seed 11)
(-4, 4) n=89  worst|m-m_pole| 6.20e-05  beam1 |E/xE-1| med 1.29e-04 at-xE 0.000  beam2 med 1.27e-04 at-xE 0.000  worst|Y-ln(x1/x2)/2| 4.86e-06
(4, -4) n=105 worst|m-m_pole| 6.20e-05  beam1 |E/xE-1| med 1.34e-04 at-xE 0.000  beam2 med 1.26e-04 at-xE 0.000  worst|Y-ln(x1/x2)/2| 5.18e-06
(21,21) n=9806 beam1 med 3.62e-08 max 1.98e-05 at-xE 0.929  beam2 med 3.73e-08 at-xE 0.929  worst|Y-ln(x1/x2)/2| 1.98e-05
== VG (seed 1, generate --seed 7)
(-4, 4) n=97  worst|m-m_pole| 5.59e-05  beam1 |E/xE-1| med 1.26e-04 at-xE 0.000  beam2 med 1.27e-04 at-xE 0.000  worst|Y-ln(x1/x2)/2| 2.12e-08
(4, -4) n=106 worst|m-m_pole| 8.21e-05  beam1 |E/xE-1| med 2.60e-04 at-xE 0.000  beam2 med 1.27e-04 at-xE 0.000  worst|Y-ln(x1/x2)/2| 2.45e-08
(21,21) n=9797 beam1 med 6.70e-09 max 4.91e-08 at-xE 1.000  beam2 med 6.75e-09 at-xE 1.000  worst|Y-ln(x1/x2)/2| 2.70e-08
```

The incoming c is on its shell to the printing precision on both sides, in
both orderings; with two equal masses neither beam is at `xE_b` and the
system rapidity is `½ln(x₁/x₂)`. **Brief correction:** this card has no
mixed massive/massless initial state (gg and cc̄ only), so "the massless parton
at x·E_beam" can be checked only on the gg events, where both sides have it
(MadGraph to its proton-mass term, as H-M1 recorded). `check-events` reads the
vibegraph file back clean. vibegraph writes no status-2 `h` line where
MadGraph does; that is the open item `plain-process-onwindow-resonance-records`.

## 3. ZprimeToTauTau_M130_xqcut20_b0p02_Mu0p0_Tau0p17_bs1

`p p > zp (j j), zp > ta+ ta-` at 0, 1 and 2 jets, MLM (`ickkw = 1`,
`xqcut = 20`), `p` and `j` including b, `maxjetflavor = 5`, every quark
massive (MD 5.04e-3, MU 2.55e-3, MS 0.101, MC 1.27, MB 4.7), the bundled
`zprime_UFO`. Both sides ran the card as shipped except `lhaid`
325300 → 247000 (`diff $SP/zp/run_card.dat $SP/vg-zp/run_card.dat` is that one
line); MadGraph additionally `gridpack = False`, `nevents = 10000` and `iseed`
per seed (`$SP/mg-zp/rc_lhapdf_base.dat`, driver `$SP/H-M2-mglaunch.sh`).
vibegraph ran with `--target-rel 0.005` (`$SP/vg-zp/H-M2-vgdrive.sh`) on the
merged tree; seed 1 was rerun after both merges, seeds 2–5 ran only on it.

The @0 subprocesses are the mass-sensitive comparison: `b s~`, `s b~` and
`b b~` into `zp`, massive on both incoming legs (unequal masses on the first
two), no αs at this order and no matching veto. MadGraph's @0 P directories
are `P0_bbx_zp_zp_taptam`, `P0_bsx_zp_zp_taptam` and `P0_sbx_zp_zp_taptam`.

Total σ, MadGraph seeds 11–15 against vibegraph seeds 1–5:

```
MG wall 689 1034 625 513 548 s; Integrated weight 10.59799 10.60532 10.60908 10.62920 10.63801
VG wall 1214 505 516 680 650 s; σ 10.595320 ± 0.038528, 10.665176 ± 0.044975, 10.607540 ± 0.044657,
                                  10.682125 ± 0.045758, 10.593822 ± 0.040245
MG n=5 mean 10.6159 sd 0.0169 (1.59e-03 rel) se(scatter) 0.00757 se(quoted) 0.0065 chi2/dof 1.29
VG n=5 mean 10.6288 sd 0.0417 (3.93e-03 rel) se(scatter) 0.0187 se(quoted) 0.0192 chi2/dof 0.94
VG - MG: +0.01288 (+1.21e-03 rel), pull +0.62
```

Per part (`python3 -I $SP/H-M2-zpparts.py`: MadGraph's per-P-directory σ from
`HTML/lhapdf_<seed>/results.html`, summed per multiplicity; vibegraph's
per-decomposition `σ` lines, its two @0 decompositions told apart by size):

```
MG seeds 5 VG seeds 5
P0     MG 5.4808 (sd 0.00327, chi2/dof 0.43)  VG 5.4764 (sd 0.0146, chi2/dof 1.91)  VG-MG -8.059e-04 rel, pull -0.64
        MG 5.485 5.476 5.482 5.481 5.48 | VG 5.4909 5.489 5.464 5.4586 5.4793
P0_sb  MG 4.2228 (sd 0.0013, chi2/dof 0.10)  VG 4.2165 (sd 0.014, chi2/dof 2.12)  VG-MG -1.481e-03 rel, pull -0.95
        MG 4.221 4.223 4.224 4.224 4.222 | VG 4.2304 4.2265 4.2071 4.197 4.2216
P0_bb  MG 1.258 (sd 0.00394, chi2/dof 3.28)  VG 1.2598 (sd 0.00243, chi2/dof 0.23)  VG-MG +1.461e-03 rel, pull +0.53
        MG 1.264 1.253 1.258 1.257 1.258 | VG 1.2605 1.2625 1.2569 1.2616 1.2577
P1     MG 3.3467 (sd 0.0103, chi2/dof 1.07)  VG 3.3537 (sd 0.0391, chi2/dof 1.83)  VG-MG +2.074e-03 rel, pull +0.38
        MG 3.3375 3.3397 3.3405 3.3586 3.3574 | VG 3.3387 3.3348 3.3594 3.4183 3.3171
P2     MG 1.7884 (sd 0.009, chi2/dof 0.77)  VG 1.7987 (sd 0.0281, chi2/dof 0.85)  VG-MG +5.752e-03 rel, pull +0.74
        MG 1.7757 1.7906 1.7864 1.7888 1.8008 | VG 1.7657 1.8413 1.7841 1.8052 1.7974
```

`P0_sb` is `b s~` + `s b~`, `P0_bb` is `b b~`. The total and every part,
both @0 parts included, meet `|pull| < 3` and `rel < 0.005`. This is the
survey card that closes `massive-incoming-partons-refused` by its
`closes_when` (σ on a survey card within the σ-gate tolerance, with the
incoming mass treated as MadGraph treats it), and the item's file is deleted
in this commit.

**What this card can and cannot see.** The @0 parts carry the massive
incoming kinematics (flux, the threshold `ŝ > (m₁ + m₂)²`, on-shell incoming
momenta) on both legs with no αs and no matching, which makes them the
cleanest σ in the survey for this feature. The mass effect itself is small at
`M_zp = 130` GeV, of order `m_b²/M² ≈ 1.3e-3`, the same size as the @0 seed
scatter (`sd/mean` 3.3e-3 on vibegraph's `P0_sb`). So the match pins that the
card runs as MadGraph runs it — massive partons in the proton, unequal
incoming masses, an MLM-matched 0–2-jet sample — and agrees to 1.5e-3; it does
not on its own separate the incoming-mass treatment from a zeroed mass. That
treatment is pinned per helicity in H-M1 and by §4's `b b~ > mu+ mu-` row
(`mmll = 20`, a +5.3% mass effect in H-M1), which matches at +9.9e-4.
vibegraph's 0.5% target leaves @2 at about 1.6% per seed, so the jet parts
gate more loosely than @0.

With `nn23lo1` (before LHAPDF was installed), MadGraph gave 10.65–10.69 pb
over seeds 11–15 (`$SP/mg-zp/H-M2-zp-sweep.log`); the jet multiplicities carry
αs, so that run is context only.


## 4. The massive single-ordering rows (#96's item)

#96 (`56bcf347`) gives the mirrored ordering luminosity only when the card
generated it. Its report measured the massless rows; the massive ones needed
this branch. sm (MB = 4.7), 13 TeV, LHAPDF 247000 both sides, fixed
μR = μF = 91.188, `ptl = 0`, `etal = −1`, `drll = 0`, `maxjetflavor = 4`,
`mmll = 20` for the `b b~` rows; cards `$SP/mir/out_*.mg5`, `vg_*.proc`,
`rc_bb.dat`, `rc_gb.dat`. MadGraph seeds 11–15, vibegraph seeds 1–5.

| row | card | MadGraph mean (sd, χ²/dof) | vibegraph mean (sd, χ²/dof) | VG − MG | pull |
|---|---|---|---|---|---|
| `bb_exp` | `generate b b~ > mu+ mu-` | 40.7964 (0.0265, 0.99) | 40.8369 (0.0212, 0.44) | +9.9e-4 | +2.15 |
| `bb_sym` | `+ add process b~ b > mu+ mu-` | 81.6204 (0.0445, 0.56) | 81.6788 (0.0339, 0.28) | +7.2e-4 | +1.55 |
| `gb_exp` | `generate g b > z b` | 497.202 (0.383, 1.16) | 497.696 (0.199, 0.62) | +9.9e-4 | +2.41 |
| `gb_add` | `generate g b > z b` + `add process b g > z b` | 994.526 (0.485, 0.50) | 995.599 (0.714, 2.00) | +1.08e-3 | +2.43 |
| `gb_sym` | `define p = g b`, `generate p p > z b` | 994.955 (0.887, 1.55) | 995.617 (0.631, 0.48) | +6.7e-4 | +1.18 |

The per-seed values, as `$SP/H-M2-collect.sh` and the `gb_add` lines below
print them (MadGraph: banner `Integrated weight` and the log's error;
vibegraph: the final `σ` line):

```
MG bb_exp 40.787 40.804 40.785 40.838 40.768      VG 40.817182 40.855077 40.841403 40.812387 40.858330
MG bb_sym 81.671 81.549 81.627 81.620 81.635      VG 81.656239 81.700566 81.707777 81.630283 81.699351
MG gb_exp 497.464 497.032 497.113 497.691 496.711 VG 497.634016 497.830497 497.968922 497.548210 497.497594
MG gb_add 994.876 994.127 994.597 995.085 993.943 VG 995.832421 996.022584 996.375044 995.180247 994.584598
MG gb_sym 995.542 994.318 993.981 996.146 994.786 VG 995.208708 995.326250 996.024616 996.515323 995.009481

gb_add:  MG n=5 mean 994.526 sd 0.485 (4.87e-04 rel) se(scatter) 0.217 se(quoted) 0.305 chi2/dof 0.50
         VG n=5 mean 995.599 sd 0.714 (7.17e-04 rel) se(scatter) 0.319 se(quoted) 0.226 chi2/dof 2.00
         VG - MG: +1.073 (+1.08e-03 rel), pull +2.43
```

`gb_add` is the brief's symmetric form for `g b` (an `add process` of the
swapped ordering); MadGraph writes it as two P directories, both unmirrored
(`$SP/mir/gb_add/SubProcesses/P{1_gb_zb,2_bg_zb}/mirrorprocs.inc`:
`DATA (MIRRORPROCS(I),I=1,1)/.FALSE./` in each), the same structure as `bb_sym`.
`gb_sym` (the `p p` form, one P directory with its mirror) was measured
before the restart and is kept as the mirrored-path row.

Both-orderings over single-ordering: `b b~` 2.00068 (MadGraph) and 2.00012
(vibegraph); `g b` (`p p` form) 2.00110 and 2.00048; `g b` (`add process`
form) 2.00025 and 2.00041. MadGraph builds `bb_sym` as two P
directories (`P1_bbx_ll`, `P2_bxb_ll`) and `gb_sym` as one with its mirror;
vibegraph's numbers on both are bit for bit the ones H-M1 recorded before #96,
as #96 intends for cards that generate both orderings.

Every event of a single-ordering card has the card's parton on beam 1, on
both sides (10000 events each):

```
== VG bb_exp  (5, -5) n=10000 worst|m-m_pole| 3.75e-05  worst|Y-ln(x1/x2)/2| 3.06e-07
== MG bb_exp  (5, -5) n=10000 worst|m-m_pole| 5.03e-05  worst|Y-ln(x1/x2)/2| 2.27e-04
== VG gb_exp  (21, 5) n=10000 beam1 at-xE 1.000 (to 4.9e-8)  b mass 4.75e-05
== MG gb_exp  (21, 5) n=10000 beam1 at-xE 0.935 (to 2.3e-5)  b mass 4.74e-05
```

**The massless rows, against LHAPDF MadGraph.** #96's own massless rows
(H-R1 §4) were measured against MadGraph with `nn23lo1`, not LHAPDF, so they
were re-measured here with the same settings as the massive rows (`rc_bb.dat`
for `u u~`, `rc_gb.dat` for `u g`; driver `$SP/mir/H-M2-massless.sh`).
MadGraph writes one P directory each (`P1_qq_ll`, `P1_qg_qz`), both
`DATA (MIRRORPROCS(I),I=1,1)/.FALSE./`.

```
### uu_exp  (generate u u~ > e+ e-)
MG 777.68 778.18 777.48 778.24 777.69   VG 778.706513 777.732686 778.940887 778.891593 778.141771
MG n=5 mean 777.854 sd 0.336 (4.32e-04 rel) se(scatter) 0.15 se(quoted) 0.274 chi2/dof 0.31
VG n=5 mean 778.483 sd 0.526 (6.75e-04 rel) se(scatter) 0.235 se(quoted) 0.28 chi2/dof 0.66
VG - MG: +0.6287 (+8.08e-04 rel), pull +1.60
### ug_exp  (generate u g > u z)
MG 2456.83 2460.12 2454.49 2454.34 2454.50   VG 2455.998055 2459.343906 2458.524251 2461.720429 2456.082813
MG n=5 mean 2456.06 sd 2.5 (1.02e-03 rel) se(scatter) 1.12 se(quoted) 1.84 chi2/dof 0.34
VG n=5 mean 2458.33 sd 2.4 (9.76e-04 rel) se(scatter) 1.07 se(quoted) 0.967 chi2/dof 1.23
VG - MG: +2.278 (+9.27e-04 rel), pull +1.07
```

vibegraph's `uu_exp` mean is H-R1's post-fix mean to the digit (778.483),
as it should be on an unchanged code path; the MadGraph mean moved from
778.108 (`nn23lo1`) to 777.854 (LHAPDF).

All seven rows, massive and massless, single-ordering and both-orderings,
meet the multi-seed rule row by row with LHAPDF on both sides
(`|pull| < 3`, `rel < 0.005`), and both-orderings/single sits at 2 within
1.1e-3 on both sides. The item's `closes_when` holds, and its file is
deleted in this commit.

**The rows are not individually noisy; they are coherently high.** All seven
sit +6.7e-4 to +1.08e-3 above MadGraph, pulls +1.07 to +2.43; the summed
pull 12.4/√7 ≈ 4.7 is not seed scatter. The mirror observable is untouched by
it (a factor of 2 against a 1e-3 offset). The offset does not depend on the
b mass (`u u~ > e+ e-` shows it), and `b b~`/`u u~ > l l` carry no αs, so it
sits in something every hadronic row shares — the PDF evaluation, the EW
inputs, or MadGraph's own integration — and is Found 2.

## 5. The mirror caveat

Both survey cards generate `p p`: MadGraph logs "added to mirror process" for
every unequal pair on both (`c~ c`, `s~ b`, `b~ s`, `b~ b`, …), so #96 changes
nothing for them. Measured: LL_HAHM seed 1 after merging #96 reproduces all
158 iteration lines and σ of the run before it.

## 6. What the merges of main changed

Two merges of `origin/main` landed during the session: `d019a671` (#88) and
`074a42b8` (#96, #94, #98).

- #88 touches UFO and SLHA loading, param-card classification, `proton.rs`,
  `budget.rs`, `vegas.rs`, `hadronic.rs` and the LHE writer, so it can reach
  every vibegraph number. Its conflict in `proton.rs` was resolved by keeping
  H-M1's `inverse_flux` and #88's point census; a point below the incoming
  threshold is counted with the cut rejections (`census.rejected()`), since it
  carries no weight before any cut, as MadEvent's `one_tree` rejects it.
- LL_HAHM seed 1, before #88, after #88, and after #96/#94: σ, error, all 158
  iterations, and the generated event block are identical.
- ZprimeToTauTau was rerun entirely after both merges.
- The `d019a671` message lacks the `Assisted-by` trailer (my error); it was
  not amended.

## Gates

All on the final source (the lift plus this report's KB edits), merged tree
`074a42b8`. Cargo ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2` (log `$SP/H-M2-gates.log`,
script `$SP/H-M2-gates.sh`):

```
$ cargo fmt --all --check                                  FMT EXIT 0
$ cargo clippy --workspace --all-targets -- -D warnings    Finished `dev` profile ... CLIPPY EXIT 0
$ cargo test --workspace                                   TEST EXIT 0
  (summed over test results: passed 1908 failed 0 ignored 25)
```

The banked layer, detached (`nohup setsid`, script `$SP/H-M2-validate.sh`, log
`$SP/H-M2-validate.log`), with `CARGO_TARGET_DIR=target/h-m2-validate` so that
its release-debug build could not replace the binary the σ sweeps above were
running (Found 6):

```
$ bash validation/validate.sh          (03:16:54 → 05:32:15 UTC)
clippy (extended-validation features): Finished `dev` profile in 5m 45s, exit 0
cargo test --profile release-debug ... : summed test results passed 2361 failed 0 ignored 110
[report] 77 rows x 4 categories: the measured cells are the declared cells
77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove
  (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell
passed, and every measurement agrees with the manifest about what it is.
validate exit 0
```

The five ⚠️ cells are the manifest's informational ones (`gg_to_gg` and
`gg_to_gg_cg` diagram counts, `ee_to_wpwm_cw` and `ee_to_zh_smeft` amplitude
tolerances, the `ee_to_mumua` sample KS), measured and deliberately not
enforced; none is new to this branch. `kb.py index` and `kb.py lint`:
`1110 concepts, 353 backlog items; 0 errors, 5 warnings` (the five warnings
are pre-existing `state: blocked` items).

## Found

1. **MadGraph 3.7.1 refuses UFO models with `aS` off SMINPUTS 3**
   (`import_ufo.py:905`), which CMS's HAHM gridpacks ship; and with python
   LHAPDF present its `update_dependent` writes αs into SMINPUTS 3 whatever
   parameter is there (`Gf` on HAHM). Any oracle on such a model needs the
   lhacode swap recorded in §2. Closes with a note in
   `validation/madgraph-defects` and the swap scripted where the survey runs
   MadGraph.
2. **vibegraph sits +7e-4 to +1.1e-3 above LHAPDF MadGraph on all seven
   rows of §4**, massless included (`u u~ > e+ e-` +8.1e-4), pulls +1.07 to
   +2.43, summed 12.4/√7 ≈ 4.7. Same PDF set and αs on both sides, and the
   `q q~ > l l` rows carry no αs, so it is neither the mass nor αs. Each row
   passes the gate alone. To close: compare `x f(x, 91.188)` from vibegraph's
   reader with LHAPDF 6.5.4 on an x grid at these kinematics (the
   `lhapdf-oracle` machinery), then the EW inputs and the per-event
   `<pdfrwt>` x and weights of `uu_exp`.
3. **Narrow s-channel resonance converges slowly**: `p p > h > zp zp`
   (Γ_h = 7.3 MeV at 125 GeV) needs 144–273 iteration lines at χ²/dof 1.7–5.8 to
   reach 0.1%, about 20M evaluations for a 2 → 2. To close: look at the
   channel's Breit–Wigner map against the resonance width.
4. **LL_HAHM's σ cannot detect the incoming-mass treatment** (5e-6 of σ, §2).
   Any banked row meant to gate the massive-incoming kinematics needs a card
   where the massive initial state is a large fraction and the mass effect is
   percent-level (H-M1's `b b~ > mu+ mu-` at `mmll = 20`: +5.3%).
5. **MLM with massive light quarks is not per-event checked**: the
   ZprimeToTauTau σ comparison per multiplicity is the only matched oracle
   here; the per-event scale and veto against MadGraph's own events needs the
   `validate-scales` machinery on a banked run.
6. **`validate.sh` rebuilds `target/release-debug/vibegraph` with the
   extended-validation features.** Any σ sweep that runs the release-debug
   binary while the banked layer runs in the same target directory changes
   build mid-sweep. This session ran the layer with
   `CARGO_TARGET_DIR=target/h-m2-validate` to avoid it. To close: a line in
   the `extended-validation` skill.
