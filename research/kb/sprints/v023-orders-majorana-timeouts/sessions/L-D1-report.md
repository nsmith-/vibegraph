---
type: Session Report
title: "L-D1 report: card 6 has a CP-odd coupling-binding defect, worth percent, not 1.66x"
description: "The census of card 6 matches MadGraph 3.7.1 subprocess by subprocess, with and without `/ h` and `$$ w+ w-`. On g u > t b~ d mu+ mu- (t > w+ b, w+ > e+ ve), per diagram and per helicity, every amplitude matches to 1e-14 once the imaginary Wilson coefficients are zero. With them on, 23 of 75 diagrams disagree: the four-point t-b-W-photon/Z contacts and the self-conjugate t-t-g/photon/Z dipoles take the wrong coupling under this crate's all-incoming vertex binding. A two-part fix, checked on the dumps and on the charge-conjugate subprocess, gives 2e-14. Follow-up: with card 6's own cuts both sides apply ptj 0.1 to the spectator b~ and mmll 30 to the core pair only, and on this subprocess vibegraph reads 1.668x MadGraph 3.7.1 with the imaginary coefficients on and 1.011x with them zero, so this defect is the 1.66x."
status: draft
tags: [dim6top, cp-violation, complex-couplings, hermitian-conjugate, oracle, madgraph-parity, cms-gridpacks, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Worktree `/home/user/wt/sprintl-dim6top-sigma`, branch `sprintl-dim6top-sigma`, head
`570f1ea9` (main `c095a1cf` plus the brief). This was a diagnosis session. No source
change is committed: the only commit is this report. Every scratch artifact is under
`/home/user/wt/sprintl-dim6top-mg/` (listed at the end). The scratch probe
`vibegraph-lib/tests/ld1_probe.rs` was copied into the worktree for each run and then
deleted. The canonical copy is `harness/ld1_probe.rs`. The brief closes no item, so
this commit deletes no file.

## Summary

> **Superseded in part by the follow-up section at the end.** The σ-level
> conclusion below ("this defect does not explain 1.66×") is wrong: with card 6's
> own run card, the same subprocess reads 1.668× MadGraph with the imaginary
> coefficients on and 1.011× with them off. The resonant-point |M|² scan
> understated the effect because it does not sample where the σ lives.

- **`/ h` and `$$ w+ w-` both reach the diagram set.** The census is machine-checked
  on both sides for both `generate` lines, in three variants: as written, without
  `/ h`, and without `$$ w+ w-`. Every vibegraph set pairs with one MadGraph
  core × decay product, and the counts match in all three variants (0 mismatches).
  Dropping `/ h` raises the count from 7200 to 8032 diagrams. Dropping `$$` raises
  it to 19440 diagrams in 108 sets. The result record leaves `/ h` out of its echo
  of the process. That is a display defect only (Found 1).
- **Imaginary Wilson coefficients set to zero: bit-exact.** On the chosen subprocess
  every one of 75 diagrams pairs one to one with a MadGraph graph. Each has ratio −i
  (one global phase) and residual ≤ 6.7e-14. The JAMP agrees to 7.9e-15 and the
  helicity-summed |M|² to 1e-12, at 6 generic points and 4 resonant points (t, W
  and Z exactly on their poles), with card 6's widths. The charge-conjugate
  subprocess gives the same result (1.9e-14).
- **Card 6 as given (imaginary coefficients on): 23 of 75 diagrams disagree, in two
  classes.** Every other diagram, including the W-dipole (`ctW`, `ctWI`) diagrams,
  agrees to 1e-14.
  - (A) **Four-point t–b–W–γ/Z contacts (4 diagrams).** The magnitude is exact, but
    the phase is off by `−c/c*` = e^{−i·46.4°}, where `c` = GC_795 or GC_848. This
    crate binds `V_382 [t~ b a W+]` with GC_795. MadGraph evaluates its Hermitian
    partner `V_349 [b~ t a W-]` with GC_796 = conj(GC_795).
    `UFOModel::conjugate_couplings` makes that exchange only for three-particle FFV
    vertices.
  - (B) **Self-conjugate dipoles t t g, t t γ, t t Z (19 diagrams).** Here the shape
    is wrong (overlap down to 0.53). The σ^{μν}q_ν γ⁵ term (FFV9, carrying ctGI,
    ctWI and ctZI) has the wrong sign relative to the σ^{μν}q_ν term (FFV11).
    Under the all-incoming binding, conjugating the structure flips the γ⁵ term, and
    `conjugate_couplings` returns `None` for a self-conjugate vertex.
- **Candidate fix, checked arithmetically.** (A) multiplies a bound contact's value
  by `−c_hc/c`. (B) negates the FFV9 part. Together they bring every diagram of the
  fitted subprocess to −i within 8.3e-14 (JAMP 7.2e-15, |M|² ratio 1.000000 at
  every point). The charge-conjugate subprocess `g d > t~ b u mu+ mu-` was not
  used to build the rule, and the same rule gives 1.9e-14 there. Each part alone
  leaves a 4–15% JAMP error, so both are needed.
- **This defect does not explain 1.66×.** Over 300 resonant points per energy, the
  summed |M|² ratio vg/MG with card 6 is 0.9891 at √ŝ = 400 GeV and 1.0886 at
  800 GeV. Per point it ranges from 0.22 to 9.5. With the imaginary coefficients
  zero the ratio is 1.000000. A ~1.45× factor appears in every flavour group of
  the validator's per-group σ, so it must come from somewhere outside the
  amplitudes.
- **One internal σ check is consistent.** At one reduced seed, this crate's
  decay-chain σ divided by its own undecayed core σ is 0.1158, against an
  expected BR × window of about 0.116 (below). The σ-level gap is therefore not in
  the chain normalisation of this subprocess. The prime σ-level suspect is the run
  card's treatment of the spectator b~: with `MB = 0` and no b cut the 4-flavour
  integral is log-divergent, and this crate refuses it. I could not see the
  gridpack's run card. Long-run requests are below.

## Provenance and input hashes

The input files are in `/home/user/wt/sprintl-dim6top-mg/` and were never edited.
Command: `sha256sum <file>`.

| file | sha256 |
|---|---|
| `param_card_card6.dat` | `25e1a921809e00058eef4b9e11e6eebbf07bfb0ae5afb018016adef079480f6f` |
| `proc_card_card6_vg.dat` | `62385f4217f285426fcd30b0dd31ac760ab24078469e03288603f0d6d13722a2` |
| `MG_SETTINGS.md` | `7e35fe8c84a45b86b690822b58d88dc9d78b10b4d42f7a85bf63bb55ff2f3fee` |
| `restrict_model_condensed.txt` | `8b4c25f3f132b85c66d8731439766a33807fc2b2d7701be30863bb7ec9781ad6` |
| `dim6top_manifest_public.txt` | `f39ebc960bb0d494111b8467c85b82ea9d10784ecfbe1e18733ca4bdc33ede0a` |
| `dl/dim6top_LO_UFO.tar.gz` | `04ceebc7b8e2e6db7a1b3c3e4bfd222004f245d5539a883abf633860ec297849` |
| `models/dim6top_LO_UFO/couplings.py` | `0bd1f9cbae0c6c0410cdae39ba91f8f5e7b2e4605766e0876ac34011d5e38cad` |
| `models/dim6top_LO_UFO/vertices.py` | `65da9f20645d87cb57b9746ba849044efd38acde0b6fc067b9268032e1624522` |
| `models/dim6top_LO_UFO/lorentz.py` | `d625fa1832078dd3a91e6476901dbe74da9cdb54bcdb227be5a52b4ff06d69e2` |
| `models/dim6top_LO_UFO/parameters.py` | `a9ccf4bd4cadacfecfabff1011f1ff7eb40a1152c76772a32a0ff2ef4b848bb1` |
| `models/dim6top_LO_UFO/particles.py` | `c816e6b02fda9ec9524e631601f76567c0544a3825025a81cc352fad2f36d00a` |
| `models/dim6top_LO_UFO/configuration.py` | `81938a3b029991935b6669c9b0784783a4992abcaf5ce64a9e55e1608171b47d` |

The other UFO files match `dim6top_manifest_public.txt` line for line.

Derived inputs:

- **The restrict card.** The UFO ships no restrict card, so
  `ufo/dim6top_LO_UFO/restrict_restrict.dat` (`733cf127…`) was rebuilt by
  `harness/ld1_restrict_card.py` from `restrict_model_condensed.txt`. It lists all
  473 external parameters from `parameters.py` at the listed values, else 0.
  - It carries the distinct placeholders 0.1 to 0.8 on the 8 ctX operators,
    Lambda = 1000, MB = ymb = 4.7, MT = ymt = 172, and the condensed card's widths.
  - The same card restricts both sides. vibegraph reads `ufo/` (its `.py` files are
    byte-identical to `models/`). MadGraph reads `mg371/models/dim6top_LO_UFO`.
- **The MadGraph model copy.** MadGraph 3.7.1 (`mg371/VERSION`: 3.7.1, 2026-04-29)
  is a copy of the worktree's `research/refs/mg5amcnlo`. Its model is a copy of
  `models/dim6top_LO_UFO` converted to Python 3 by MadGraph's own
  `set auto_convert_model T`. Every amplitude file keeps its hash; only
  `object_library.py` and `write_param_card.py` change (`census/mg371_model_sha.txt`).
- **The evaluation cards.**
  - `sa/standalone/Cards/param_card_edited.dat` (`693f31d9…`) is the restricted
    model's own card with every card-6 entry set. `ld1_bank.py` asserts that each
    card-6 entry the restricted card carries is equal, and that no card-6 entry is
    missing.
  - The no-imaginary card is `param_card_edited_noI.dat` (`1592166e…`), with
    ctpI = ctWI = ctZI = ctGI = 0.
- **The couplings agree by name.** MadGraph's ModelReader and this crate agree on
  all 66 couplings of the restricted model to better than 1e-12, on both cards.
  Commands: `python harness/ld1_couplings_mg.py <card> tables/mg_couplings_*.json`,
  then `python3 harness/ld1_coupcmp.py tables/mg_couplings_w.json tables/vg_w.json`
  → `66 couplings, 0 differ above 1e-12`, and the same for the noI card. So the
  defect is not in coupling evaluation. This includes `Gstrong = 1`
  (`norm_chromo_gs = False`) and the conditional `QCD` order of GC_725 and GC_726.
- **MadGraph line.** MadGraph 3.7.1 refuses `FCNC=0` on the restricted model with
  "model order FCNC not valid for this model": the restriction removes every FCNC
  coupling, and with them the order. The MadGraph lines therefore drop `FCNC=0`.
  This crate accepts `FCNC=0` and enumerates the same diagrams. See Found 2.
- **The validator's run used the restricted model.** This crate cannot compile the
  unrestricted model for this subprocess. With `restrict_card: None`, `g u > …`
  enumerates 193 diagrams instead of 75, and compiling fails on the four-fermion
  `FFFF12` ("cyclic Lorentz structure … only two fermion lines joined by two summed
  Lorentz indices are evaluable"). Command: the probe on `tables/ld1_gu_unres.json`,
  `/home/user/wt/logs/ld1-probe-unres.log`. A run of the bundled model as a bare
  name applies its `restrict_default.dat`, which MG_SETTINGS says equals the
  gridpack's `restrict_model.dat`. Suspect 4 (the dropped restriction) is therefore
  not live (see Corrections).

## Census (both `generate` lines, both sides, by machine)

How each side was counted:

- **MadGraph.** `harness/ld1_census_mg.py` runs MadGraph 3.7.1's `MasterCmd` with the
  gridpack preamble (`cards/preamble.mg5`), then `import model
  dim6top_LO_UFO-restrict`, the gridpack's `define`s (`cards/defines.mg5`) and both
  lines. It walks `_curr_amps` and records each core amplitude's diagram count and
  each decay's (every decay amplitude has exactly 1 diagram).
- **vibegraph.** `vibegraph enumerate cards/card6_<v>.dat --ufo-dir ufo --svg none`
  (release build of `570f1ea9`). Its `diagrams.json` is paired with MadGraph's
  census by `harness/ld1_census_cmp.py`.

Commands: `/home/user/wt/logs/ld1-census-mg.sh`, `/home/user/wt/logs/ld1-census-vg.sh`;
outputs `census/mg_<v>.json`, `census/vg_<v>/diagrams.json`, `census/cmp.txt`.

```
== full: vibegraph 72 sets, 7200 diagrams; MadGraph 24 cores, 1800 core diagrams; mismatches 0
   (vg sets, vg diagrams, mg core) histogram: [((75, 75, False), 48), ((150, 75, True), 24)]
== noh: vibegraph 72 sets, 8032 diagrams; MadGraph 24 cores, 2008 core diagrams; mismatches 0
   (vg sets, vg diagrams, mg core) histogram: [((75, 75, False), 32), ((101, 101, False), 16), ((150, 75, True), 16), ((202, 101, True), 8)]
== nodd: vibegraph 108 sets, 19440 diagrams; MadGraph 36 cores, 4860 core diagrams; mismatches 0
   (vg sets, vg diagrams, mg core) histogram: [((135, 135, False), 72), ((270, 135, True), 36)]
```

How to read the counts:

- **The expected count.** A vibegraph set is one MadGraph core subprocess × one
  W-decay lepton flavour. The expected count is MadGraph's core count × 1 × 1. It
  is doubled where the W-decay lepton equals a core lepton: this crate closes a
  stitched set under the permutations of identical final-state particles and
  applies the identical-particle factor. MadGraph keeps the decay products on
  their own legs and divides by `identical_decay_chain_factor`, which here is 1.
  The difference is by design (`diagrams/chain.rs`). The two agree in σ up to the
  interference between the assignments.
- **`noh`.** Without `/ h`, the 101-diagram rows are the τ⁺τ⁻ cores, where the
  Higgs couples through `ymtau`.
- **`nodd`.** Without `$$ w+ w-`, the 135-diagram cores add the s-channel W
  diagrams and 12 more cores.

The card as written (`table_full.md`), one row per MadGraph core subprocess:

| MG core subprocess | MG core × decays | vg (W > e) | vg (W > mu) | vg (W > ta) |
|---|---|---|---|---|
| `g u > t b~ d e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g u > t b~ d mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g u > t b~ d ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g c > t b~ s e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g c > t b~ s mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g c > t b~ s ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g d~ > t b~ u~ e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g d~ > t b~ u~ mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g d~ > t b~ u~ ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g s~ > t b~ c~ e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g s~ > t b~ c~ mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g s~ > t b~ c~ ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g d > t~ b u e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g d > t~ b u mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g d > t~ b u ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g s > t~ b c e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g s > t~ b c mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g s > t~ b c ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g u~ > t~ b d~ e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g u~ > t~ b d~ mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g u~ > t~ b d~ ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |
| `g c~ > t~ b s~ e+ e-` | 75 × 1 × 1 | 150 | 75 | 75 |
| `g c~ > t~ b s~ mu+ mu-` | 75 × 1 × 1 | 75 | 150 | 75 |
| `g c~ > t~ b s~ ta+ ta-` | 75 × 1 × 1 | 75 | 75 | 150 |

The other two variants' per-row tables are `census/table_noh.md` and
`census/table_nodd.md` (0 rows missing in each).

Further facts the census settles:

- **No b-initiated subprocess.** None exists on either side, though `p` includes
  b: no flavour assignment of `b q > t b~ j l+ l-` conserves charge and baryon
  number.
- **The coupling orders.** On the chosen subprocess the order classes are
  {DIM6 1, QED 6}: 13, {DIM6 1, QCD 1, QED 5}: 36 and {QCD 1, QED 6}: 26.
  MadGraph's graphs pair one to one with these diagrams (below). So DIM6=1 selects
  the same diagrams on both sides (suspect 2), and the DIM6² pieces are in both
  |M|²: each side squares a sum of DIM6 ≤ 1 diagrams with no squared-order
  selection.
- **`default_unset_couplings 99`.** QED and QCD are unbounded on both sides. The
  census matching at 75 shows that no order-based pruning differs.

## Chosen subprocess and why

I chose `g u > t b~ d mu+ mu- $$ w+ w- / h DIM6=1, (t > w+ b DIM6=0, w+ > e+ ve)`,
whose vibegraph set is `g u > e+ ve b b~ d mu+ mu-` (75 diagrams, 1 colour flow,
512 helicities). Every subprocess of the card has the same 75-diagram structure, and
this is the valence-u (dominant) one. It has distinct lepton flavours, so no
permutation doubling enters the pairing, and both leptons are massless. As a control
outside the fit I used its charge conjugate from the `add process` line,
`g d > t~ b u mu+ mu-, (t~ > w- b~, w- > e- ve~)`.

## Banking (MadGraph 3.7.1 standalone)

**Generation.** I generated `sa/standalone` and `sa/madevent` (and `sa/cc_*` for the
conjugate) from `sa/gen_*.mg5`: the gridpack preamble, then `import model
dim6top_LO_UFO-restrict`, then the line above without `FCNC=0`. Command:
`/home/user/wt/logs/ld1-mgout.sh`. MadGraph reports "1 processes with 77 diagrams …
Generated helas calls for 1 subprocesses (75 diagrams)" (77 counts the 2 decay
diagrams).

**Banking script.** `harness/ld1_bank.py` (`ld1_bank_cc.py` for the conjugate) reuses
`gen_standalone_jamps`'s point generator, card editor, `read_matrix` and
`read_jamp_coefficients`. It patches `MATRIX` to publish `AMP()` and `JAMP()` and
writes, per point and helicity, the |M|², JAMP(1) and all 75 `AMP()`:

| table | card | MATRIX |
|---|---|---|
| `w` | card 6 | t-channel widths zeroed as MadEvent writes them |
| `w_sa` | card 6 | standalone (spacelike widths kept) |
| `noI_w` | card 6, imaginary coefficients 0 | t-channel-zeroed |

`ld1_bank_zw.py` adds `zw` and `noI_zw` (every width zero, generic points only).

**Points.** 6 RAMBO points at 1000 GeV (seed 6061), plus 4 resonant points (seed
6062). The resonant points are built as RAMBO `t b~ d Z` at the t and Z masses
followed by isotropic `t > W b`, `W > e+ ve` and `Z > mu+ mu-` decays, so the t, W
and Z are exactly on their poles.

**The width zeroing.** MadEvent zeroes 16 calls of the 2→7 matrix element; the
banking prints each one (`/home/user/wt/logs/ld1-bank2.log`). L-I1's
`tchannel_zeroed`, which I copied, misses part of them. Its call regex
`CALL [A-Z0-9]+_\d\(` skips the multi-structure HELAS routines (`FFV11_9_1`,
`FFV2_8_1`), and its line regex skips calls whose width argument sits on a
continuation line. On the first pass it zeroed 11 of the 16. Those five calls then
showed up as a 1.5e-3 residual on the diagrams with two production tops. That
residual disappeared at zero width (8e-15), and that is how the gap was found. The
fixed matcher folds continuations, matches `CALL [A-Z0-9]+(_\d+)+\(`, compares
whitespace-free, and compiles with `-ffixed-line-length-none`. Every number below
comes from the fixed harness. See Found 3: L-I1's harness has the same gap.

## The oracle

`harness/ld1_probe.rs` is a copy of L-I1's probe without the squared-order split.
It uses `generate_with`, then the production `AmplitudeEvaluator::compile` of the
full set and of each diagram alone, `BoundAmplitude::eval_amplitude` per helicity,
and the production `eval_m2`. It also dumps the evaluated couplings by name. Command:
`LD1_COUPLINGS=… harness/run_probe.sh <table>…`, which runs
`cargo test --release -p vibegraph-lib --test ld1_probe` under the heavy lock, with
logs `/home/user/wt/logs/ld1-probe*.log`.

`harness/ld1_fit.py` pairs diagrams with MadGraph graphs by normalised overlap over
every (point, helicity) entry, with each point scaled by its JAMP norm. It reports
each pair's complex ratio, ||r|−1| and residual, the JAMP fit, and the |M|² per
point. `harness/ld1_classsum.py` condenses the per-diagram table by class. Fits are
`tables/fit_<table>.txt`.

### The fit with the imaginary coefficients on, and with them zero

Command: `python3 harness/ld1_classsum.py tables/fit_w.txt tables/fit_noI_w.txt
tables/fit_gd_w.txt tables/fit_gd_noI_w.txt tables/fit_noI_zw.txt tables/fit_zw.txt`

```
tables/fit_w.txt
  contact FFVV (V_382 V_402 V_349 V_369): n=4, min overlap 1.000000000, worst ||r|-1| 2.4e-15, worst resid 7.1e-14, worst |r-(-i)| 7.9e-01
  self-conjugate dipole term #1 (V_189 V_49 V_51): n=19, min overlap 0.525712530, worst ||r|-1| 9.8e-01, worst resid 7.5e-01, worst |r-(-i)| 2.0e+00
  every other diagram: n=52, min overlap 1.000000000, worst ||r|-1| 8.2e-15, worst resid 6.4e-14, worst |r-(-i)| 0.0e+00
tables/fit_noI_w.txt
  contact FFVV (V_382 V_402 V_349 V_369): n=4, min overlap 1.000000000, worst ||r|-1| 2.0e-15, worst resid 6.7e-14, worst |r-(-i)| 0.0e+00
  self-conjugate dipole term #1 (V_189 V_49 V_51): n=19, min overlap 1.000000000, worst ||r|-1| 5.3e-15, worst resid 6.7e-14, worst |r-(-i)| 0.0e+00
  every other diagram: n=52, min overlap 1.000000000, worst ||r|-1| 8.4e-15, worst resid 6.5e-14, worst |r-(-i)| 0.0e+00
tables/fit_gd_w.txt
  contact FFVV (V_382 V_402 V_349 V_369): n=4, min overlap 1.000000000, worst ||r|-1| 4.0e-15, worst resid 2.1e-14, worst |r-(-i)| 7.9e-01
  self-conjugate dipole term #1 (V_189 V_49 V_51): n=19, min overlap 0.575123287, worst ||r|-1| 1.4e+00, worst resid 9.6e-01, worst |r-(-i)| 3.1e+00
  every other diagram: n=52, min overlap 1.000000000, worst ||r|-1| 6.0e-15, worst resid 4.4e-14, worst |r-(-i)| 0.0e+00
tables/fit_gd_noI_w.txt
  (every class: min overlap 1.000000000, worst resid ≤ 4.6e-14, worst |r-(-i)| 0.0e+00)
tables/fit_noI_zw.txt
  (every class: min overlap 1.000000000, worst resid ≤ 9.0e-15, worst |r-(-i)| 0.0e+00)
tables/fit_zw.txt
  (as fit_w: contacts at |r-(-i)| 7.9e-01, dipoles at overlap 0.628; the rest ≤ 8.4e-15)
```

The JAMP fit and |M|² per point (`tail -4 tables/fit_<t>.txt`):

```
w:     JAMP fit G = -0.071574637848885-0.944258126141131i, worst |vg - G mg| / max|mg| = 1.49e-01
       per point: sum_h CF|J_vg|^2 / sum_h MG m2: 0.820262837426 1.563781829134 1.059627114730 1.225550303598 0.949345397247 0.938241271748 0.956013423902 0.507943597670 0.905447663096 1.282088505729
noI_w: JAMP fit G = +0.000000000000044-1.000000000000001i, worst |vg - G mg| / max|mg| = 7.90e-15
       per point: sum_h CF|J_vg|^2 / sum_h MG m2: 1.000000000000 (all 10 points); eval_m2 / sum_h MG m2: 1.000000000000e+00 (all 10)
gd noI_w: JAMP fit G = +0.000000000000056-1.000000000000000i, worst 1.90e-14; |M|^2 ratio 1.000000000000 at all 10 points
```

**Discriminator verdict.** With the imaginary coefficients zero, the gap closes
completely, to rounding level, on both subprocesses, both at card-6 widths and at
zero width. The disagreement is therefore entirely in how complex couplings are
handled. Each of the 23 disagreeing diagrams contains exactly one vertex of the two
classes above, and the 52 others agree. Those 52 include 26 diagrams carrying the W
dipole (`V_38#1`, GC_768 = ctWI + i·ctW), a complex coupling on a vertex that
`conjugate_couplings` already maps.

### Which coupling each side reads (a non-Hermitian probe)

This step reads which couplings each side uses. It cannot say which choice is
physically right; for that see "What each oracle cannot see" below.

- **The probe.** `ufo_hack/` is a scratch UFO copy with chosen couplings scaled by
  distinct primes: GC_767 ×2, GC_768 ×3, GC_795 ×5, GC_796 ×7, GC_725 ×11, GC_726
  ×13, GC_848 ×17, GC_847 ×19. I dumped the probe on it (2 points) and divided by
  the unscaled dump with `python3 harness/ld1_hackcmp.py tables/vg_w.json
  tables/vg_hack.json`.
- **What this crate reads.**
  - The contact diagrams 0 and 2 scale ×5 (GC_795), and 1 and 3 scale ×17 (GC_848).
  - Every `V_38#1` diagram scales ×3 (GC_768, the coupling of the partner
    `V_46 [b~ t W-]`): the existing exchange at work.
  - The `V_189#1` diagrams mix ×11 and ×13 (ratio 12.2, residual 1).
- **What MadGraph calls.** Its standalone `matrix.f` calls `FFVV1_0(…,GC_796,…)`
  and `FFVV1_0(…,GC_847,…)` for those graphs (GC_796 and GC_847 belong to `V_349`
  and `V_369`). It also calls `FFV3_*(…,GC_768,…)` for the W dipole. In the
  conjugate subprocess this crate binds `V_349` and `V_369`, and MadGraph calls
  `FFVV3_0(…,GC_795)` and `FFVV3_0(…,GC_848)`.
- **The conclusion.** This crate binds the contact's all-incoming half and keeps
  its coupling; MadGraph evaluates the other half. This is the same situation the
  comment on `UFOModel::bound_coupling` describes for FFV vertices.

## Where the disagreement lives

Both classes have one code site, `vibegraph-lib/src/ufo/mod.rs`,
`UFOModel::conjugate_couplings` (lines 809–885 at `570f1ea9`). It feeds
`bound_coupling`, which `helas/eval/diagram_eval.rs:432`, `ufo/report.rs`,
`reweight/poly.rs` and `proton.rs` call.

- **(A) The early return at line 820.** The check
  `v.particles.len() != 3 || spin_count(2) != 2 || spin_count(3) != 1` returns
  `None` for every four-point vertex. So `V_382` and `V_402` (FFVV1/FFVV3) keep
  their own couplings where MadGraph reads their partners'. The arithmetic of the
  conjugation:
  - `γ⁰[γ^μ,γ^ν]†γ⁰ = [γ^ν,γ^μ] = −[γ^μ,γ^ν]`, and there is no momentum whose sign
    could absorb the minus, so the conjugate of `FFVV3 ([γ,γ]P_R)` is `−FFVV1`.
  - The partner's coupling on it is `GC_796` = conj(GC_795), so the bound value
    must be multiplied by `−c_hc/c` = `−c*/c`.
  - Measured: `r/(−i)` = 0.6899 − 0.7239i = e^{−i·46.37°}. Predicted: `−c/c*` =
    e^{i(180° − 2·66.81°)} conjugated by the vg/MG direction, which also gives
    e^{−i·46.37°}, where 66.81° = arg GC_795.
  - For a coupling written i·(real), which is every CP-even Hermitian term, `−c*/c`
    is 1. That is why no enforced row has seen this.
- **(B) The `own == anti` return at line 844.** For a self-conjugate vertex
  (`V_189 [t~ t g]`, `V_49 [t~ t a]`, `V_51 [t~ t Z]`) `conjugate_couplings`
  returns `None`. Under the same binding, conjugating the structure maps
  `σ^{μν}q_ν` (FFV11) to itself: `γ⁰σ†γ⁰ = σ`, and the reversed momentum gives a
  second −1. It maps `σ^{μν}q_ν γ⁵` (FFV9) to minus itself, because `γ⁰(σγ⁵)†γ⁰ =
  −σγ⁵` and the momentum gives another −1. That is the same flip
  `[γ^μ,p̸]P_R ↔ [γ^μ,p̸]P_L` that the W dipoles already get. The vertex is its own
  partner, so the exchange amounts to negating the FFV9 term's coupling (GC_726
  ctGI, GC_814 ∝ ctWI − cw·ctZI, GC_722 ctZI). The FFV11 couplings (i·ctG, i·ctW,
  i·ctZ) are unchanged. With every FFV9 coupling at zero (noI) the defect vanishes,
  as measured.

Neither class is a Lorentz-kernel defect. Each structure is evaluated correctly; the
error is which coupling it is multiplied by. All 66 couplings agree by name, and
every magnitude in class (A) is exact.

## Candidate fix, with its arithmetic check

Proposed rule: extend the term exchange to every vertex with two fermion slots and at
least one further boson, self-conjugate vertices included. Each term binds the
coupling that the Hermitian partner carries on that term's conjugate structure,
times the sign of the conjugation. Concretely:

1. **Lift the three-particle restriction.** Lift
   `len == 3 && one vector` to "two fermion slots, every term a chain". FFVV
   structures conjugate as `[γ^μ,γ^ν]P_χ → −[γ^μ,γ^ν]P_χ̄`. The normal-form match
   must be allowed up to an overall sign, and `bound_coupling` must return that
   sign with the `CouplingId`. Today `chain_normal_form` equality is exact, so a
   `−FFVV1` target would match nothing.
2. **Map self-conjugate vertices to themselves** term by term, again up to the
   sign: FFV11 → +FFV11, FFV9 → −FFV9, and vector currents → themselves with +1.
3. **Leave the remaining refusals as they are.** Scalar structures and FFVS stay
   excluded, as the comment there argues. Neither class here needs them, and `/ h`
   removes every FFVS vertex from this card.

The arithmetic check, `harness/ld1_fixcheck.py`:

- (A) multiplies each bound contact diagram by `−C[hc]/C[own]`, with `(V_382:
  GC_795→GC_796)`, `(V_402: GC_848→GC_847)` and, for the conjugate subprocess,
  `(V_349: GC_796→GC_795)` and `(V_369: GC_847→GC_848)`.
- (B) negates the FFV9 part of each `V_189/V_49/V_51 #1` diagram. That part is
  isolated as `dump(card 6) − dump(noI)`, since the FFV11 couplings do not depend
  on the imaginary coefficients. The corrected value is `2·noI − w`.

Command: `python3 harness/ld1_fixcheck.py {none,A,B,AB} [w noI_w {gu,gd}]`

```
--- gu (the subprocess the rule was read from), card-6 widths, 10 points
fix none: bijection False; worst |r_d - (-i)| 2.00e+00; worst per-diagram resid 7.47e-01
JAMP fit G = -0.071574637849-0.944258126141j; worst |J - G mg|/max|mg| 1.49e-01
fix A:    JAMP worst 4.53e-02;  |M|^2 ratios 0.98..1.48
fix B:    JAMP worst 1.45e-01;  |M|^2 ratios 0.39..1.14
fix AB: bijection True; worst |r_d - (-i)| 8.27e-14; worst per-diagram resid 7.47e-14
JAMP fit G = 0.000000000000-1.000000000000j; worst |J - G mg|/max|mg| 7.24e-15
per point sum_h CF|J|^2 / sum_h MG m2: 1.000000 (all 10)
--- gu, zero widths (zw / noI_zw), 6 points
fix AB: bijection True; worst |r_d - (-i)| 1.25e-14; worst per-diagram resid 8.37e-15; JAMP 7.93e-15
--- gd (charge-conjugate control, not used to build the rule)
fix none: JAMP worst 1.63e-01; |M|^2 ratios 0.80..1.62
fix A:    JAMP worst 8.04e-02
fix B:    JAMP worst 1.12e-01
fix AB: bijection True; worst |r_d - (-i)| 4.17e-14; worst per-diagram resid 4.44e-14
JAMP fit G = 0.000000000000-1.000000000000j; worst |J - G mg|/max|mg| 1.85e-14
per point sum_h CF|J|^2 / sum_h MG m2: 1.000000 (all 10)
```

**Controls the production fix must pass that this session did not run.**

- The enforced rows whose vertices it touches: the SMEFTsim W-dipole rows, the CKM
  rows, the MSSM `t t~ > H+ H-` and neutralino rows, and every row with a Z or γ
  vector current. For every one of those structures the rule above gives the same
  coupling and +1 as today. That is a prediction, not a measurement.
- A row that turns on a CP-odd neutral dipole, for example SMEFTsim `ctGIm` with a
  gluon-top line. If no committed table exercises that, the fix needs one (Found 4).

## What the σ impact is

**The |M|² scan.** `harness/ld1_m2scan.py <√ŝ> 300` banks MadGraph's
helicity-summed |M|² (t-channel-zeroed `MATRIX`) at 300 resonant points.
`LD1_M2ONLY=1 harness/run_probe.sh scan<√ŝ>_{w,noI_w}` dumps this crate's
`eval_m2` at the same points.

```
sqrt_s 400 w     : sum vg / sum MG = 0.989053; per-point ratio median 1.0108, min 0.2229, max 1.7962, 5%-95% [0.8030, 1.3800]
sqrt_s 400 noI_w : sum vg / sum MG = 1.000000; per-point ratio median 1.0000, min 1.0000, max 1.0000
sqrt_s 800 w     : sum vg / sum MG = 1.088581; per-point ratio median 1.0867, min 0.3308, max 9.4818, 5%-95% [0.6927, 2.5698]
sqrt_s 800 noI_w : sum vg / sum MG = 1.000000; per-point ratio median 1.0000, min 1.0000, max 1.0000
```

The resonant points are not weighted by the phase-space density, so this is an
estimate of the size, not a σ. The defect moves the subprocess by percent, with a
sign that depends on energy. It cannot produce a 1.45–1.66× factor. A factor that
size also appears in the validator's per-flavour-group seed-1 numbers
(MG_SETTINGS.md). Each lepton-flavour combination gives 0.0074 to 0.0082 pb, except
the poorly converged τ, ττ group at 0.0136 ± 0.0074 pb. Nine combinations at about
0.0078 pb make about 0.070 pb, against MadGraph's 0.0475.

**A reduced single-seed σ consistency check inside this crate.** Command:
`/home/user/wt/logs/ld1-sigma.sh`, log `/home/user/wt/logs/ld1-sigma.log`.

- **Setup.**
  - Runs: `vibegraph integrate sigma/{core,chain}.dat --ufo-dir ufo --param-card
    param_card_card6.dat --run-card sigma/run_card.dat --target-rel 0.01`.
  - Run card: MadGraph 3.7.1's default for this process with two edits:
    `ptb = 20` and `etab = 5` regulate the g → b b̄ collinear limit this crate
    otherwise refuses, and `lhapdf`/247000 replaces nn23lo1.
  - Seed: 20260719.
- **Results.**

  | run | σ | χ²/dof | iterations, evaluations |
  |---|---|---|---|
  | core `g u > t b~ d mu+ mu-` | 3.000 ± 0.026 fb | 31.2 | 7, 1.06M |
  | chain, the same with `(t > w+ b, w+ > e+ ve)` | 347.4 ± 3.3 ab | 1.45e6 | 107, 15.5M |

  The chain's χ²/dof comes from channels carrying zb. The five channels carrying
  96% of σ (channels 6, 10, 17, 21 and 23) have χ²/dof between 1.9 and 4.9 (`vibegraph inspect
  sigma/out_chain/grid.bin.zst`).
- **The ratio.** Chain/core = 0.1158.
- **The expectation.**
  - Γ(t → bW) at LO with card 6's inputs (mt = 172.5, MW = 79.824) is 1.486 GeV,
    and BR(W → eν) = 0.2226/2.085 = 0.1067.
  - That gives Γ(t → b e ν) = 0.1586 GeV. With the card's Γ_t = 1.3129, the
    effective BR is 0.1208.
  - Times the ±15 Γ window fraction of the top Breit–Wigner (about 0.958), that is
    about 0.116 before W off-shell effects of order a few %.
- **Verdict.** The decay-chain normalisation of this subprocess is consistent with
  its core σ. This is one seed at a 1% target: it would catch a 1.45× factor, not a
  2% one.

**What is left at the σ level** (in my judgement, most likely first):

1. **The gridpack run card.** I did not see it. With `MB = 0` (customizecards) and
   `maxjetflavor = 4`, nothing in MadGraph's default card cuts the spectator b̄ of
   `g q > t b~ q' l+ l-`. The 4-flavour integral then diverges logarithmically in
   the collinear limit. This crate refuses it ("…σ diverges logarithmically, and
   any cross section reported is set by how far the sampler reaches into it"). The
   validator's vibegraph run therefore had a regulator. Whether the gridpack's
   MadGraph σ had the same one, or any, decides whether the 1.66× is a comparison
   of two different cut sets, or of a regulated σ against an unregulated one.
2. **The MadGraph 2.9.18 reference itself.** Version 2.9.18 accepted `FCNC=0` on a
   model 3.7.1 refuses it for. Whether 2.9.18's survey σ for this card is
   reproducible with 3.7.1 is unmeasured.
3. **The τ, ττ flavour group's convergence on this crate's side** (± 55% at seed
   1). It inflates the seed spread, but it is not the bulk factor.

## What each oracle cannot see

- **Census.** Counts only. A diagram with the right topology and the wrong vertex
  or coupling passes, as the 23 here did.
- **Coupling oracle (66/66 by name).** It cannot see which vertex, or which half of
  a Hermitian pair, a diagram binds. That is the entire defect here.
- **Per-diagram × per-helicity fit.**
  - It is blind to a phase shared by all 75 diagrams; the global −i is absorbed.
  - With one colour flow it cannot see flow-ordering errors.
  - Its points are RAMBO plus four resonant points. A defect confined to a corner
    of phase space between them (for example a near-threshold kernel instability)
    would be missed.
  - It never evaluates `eval_m2`'s helicity-pruned path at card 6. The probe
    compiles without `prune_zero_helicities`.
- **Imaginary-coefficient discriminator.** It sees only defects that vanish at real
  coefficients. A defect present in both runs would show in both fits; none did.
- **Non-Hermitian scaling probe.** It reports which coupling each side reads, not
  which is physically right. That this crate should follow MadGraph's half is the
  project's convention: MadGraph is the oracle. It is not a field-theory result
  from this session. Whether MadGraph's binding gives the physical sign of
  CP-odd observables was not tested (Found 5).
- **Resonant |M|² scan.** It measures size, not σ. The points carry no phase-space
  weight and come from one subprocess at two energies.
- **Single-seed chain/core σ ratio.** It tests this crate's decay-chain
  normalisation against its own core σ. It is blind to any factor shared by both
  (cuts, PDFs, scales, flux) and to anything MadGraph does differently.
- **Arithmetic fix check.** It was done on dumps of two subprocesses. It says
  nothing about the enforced rows the production change would touch (listed above).

## Requests for the long-run host

Branch `sprintl-dim6top-sigma`, head = this report's commit (on `570f1ea9`).

1. **MadGraph 3.7.1 madevent σ of card 6.** Use the gridpack's own `run_card.dat`,
   the card-6 param card, the preamble in `MG_SETTINGS.md`, `import model
   dim6top_LO_UFO-restrict` (with the gridpack's `restrict_model.dat`, sha
   `5fa1e83c…`), and the proc card without `FCNC=0` (3.7.1 refuses it).
   - Seeds: 3.
   - Expected: either ≈ 0.0475 pb, which reproduces 2.9.18, or not, which puts the
     reference itself in question.
   - Also report the run card's `ptb`, `etab`, `ptj`, `maxjetflavor`, `cut_decays`,
     `mmll`, `drll` and `bwcutoff`.
2. **The same MadGraph σ with ctpI = ctWI = ctZI = ctGI = 0**, against this crate's
   card-6 σ at the same zeroed card, with 5 seeds at campaign budget. Expected: if
   vg/MG stays about 1.6 at zero imaginary coefficients, where the amplitudes are
   bit-exact, the bulk of the gap is σ-level (cuts, regulator, reference). If it
   drops to about 1, this report's size estimate is wrong.
3. **This crate's σ of card 6 with the validator's run card** at two b̄ regulators
   (for example `ptb` 0/`etab` 5 and `ptb` 20), 5 seeds each. Expected: a strong
   dependence on the regulator says the comparison depends on cuts, not on
   physics.

## Gates

- **No source change.** `git diff --stat 570f1ea9 HEAD -- vibegraph-lib vibegraph-cli`
  is empty. The scratch test file was removed (`ls
  vibegraph-lib/tests/ld1_probe.rs` → no such file).
- **Builds.** `cargo build --release -p vibegraph` finished
  (`/home/user/wt/logs/ld1-build.log`: "Finished `release` profile [optimized]
  target(s) in 4m 35s", `EXIT rc=0`). `cargo test --release -p vibegraph-lib --test
  ld1_probe` passed on every table (`test result: ok. 1 passed`, logs
  `ld1-probe*.log`). The exception is the unrestricted-model run, which fails to
  compile FFFF12 as described.
- **Not run.** The workspace `cargo test`, clippy, the hermetic-layer run and
  `validate_*`. This commit changes only `research/kb/`; no code path changes.
  `pixi run kb-lint` on this commit: "1463 concepts, 520 backlog items; 0 errors, 9 warnings". The 9 warnings are all on pre-existing backlog files.

## Files (all under `/home/user/wt/sprintl-dim6top-mg/`, none committed)

- **harness/**
  - `ld1_bank.py`: banking, with the width-matcher fixes.
  - `ld1_bank_cc.py`: charge-conjugate control.
  - `ld1_bank_zw.py`: zero-width tables.
  - `ld1_census_mg.py`, `ld1_census_cmp.py`, `ld1_census_table.py`: census.
  - `ld1_couplings_mg.py`, `ld1_coupcmp.py`: couplings by name.
  - `ld1_probe.rs`: this crate's dump; copied to `vibegraph-lib/tests/` only while
    running.
  - `run_probe.sh`.
  - `ld1_fit.py`, `ld1_classsum.py`: fit.
  - `ld1_hackcmp.py`: scaled-coupling probe.
  - `ld1_fixcheck.py`: arithmetic fix check.
  - `ld1_m2scan.py`: resonant |M|² scan.
  - `ld1_restrict_card.py`, `ld1_default_card.py` (unused: Python 2 UFO).
- **cards/**: `preamble.mg5`, `defines.mg5`, `card6_{full,noh,nodd}.dat`,
  `convert.mg5`.
- **census/**: `mg_*.json`, `vg_*/`, `cmp.txt`, `table_*.md`, `mg371_model_sha.txt`.
- **sa/**: `standalone`, `madevent`, `cc_standalone`, `cc_madevent`, `gen_*.mg5`.
- **tables/**:
  - Banked tables: `ld1_gu_{w,w_sa,noI_w,zw,noI_zw,hack,unres}.json` and
    `ld1_gd_{w,w_sa,noI_w}.json`.
  - Dumps: `vg_{w,noI_w,zw,noI_zw,hack}.json`, `vg_gd_{w,noI_w}.json` and
    `vg_scan*.json`.
  - Fits: `fit_*.txt`; couplings: `mg_couplings_{w,noI}.json`.
- **ufo/, ufo_hack/, ufo_full/**: UFO copies (restricted; scaled couplings;
  unrestricted).
- **sigma/**: `core.dat`, `chain.dat`, `run_card.dat`, `out_{core,chain}/`.
- **mg371/**: MadGraph 3.7.1, copied from the worktree submodule.

## Found

1. **The result record and echo drop `/ h`.** `vibegraph enumerate`/`integrate`
   print `g u > t b~ d mu+ mu- $$ w+ w- FCNC=0 DIM6=1` for a line with
   `$$ w+ w- / h`, while the diagram set honours `/ h` (census: 7200 diagrams
   with it, 8032 without). The forbidden-particle list is missing from
   `SupportedProcess::process_string` (or whatever renders the `process:` line),
   which makes a cosmetic defect look like a physics one. Close by echoing every
   restriction and pinning it with a proc-card round-trip test.
2. **An order the restricted model no longer has is accepted.** This crate accepts
   `FCNC=0` on `dim6top_LO_UFO-restrict`, where the restriction removed every FCNC
   coupling; MadGraph 3.7.1 refuses the line ("model order FCNC not valid for this
   model"). Parity says it should refuse, or at least warn, as MadGraph would. The
   CMS card itself ran on 2.9.18, so a refusal would break the card exactly as 3.7.1
   does. Decide the policy (parity with 3.7.1, or a named warning).
3. **L-I1's `tchannel_zeroed` misses part of MadEvent's zeroed widths.** In
   `/home/user/wt/sprintl-interference-sign-mg/harness/li1_bank.py` the regex
   `CALL [A-Z0-9]+_\d\(` skips multi-structure HELAS routines, and the per-line
   regex skips calls whose width argument is on a continuation line. Here that
   left 5 of 16 zeroed widths at their card value. Re-check L-I1's banked `w`
   tables and its Follow-up A (t-channel zero-width sets) for any `FFVn_m_k` call
   with a propagator width. The fix is in this session's `ld1_bank.py`.
4. **No enforced row carries a CP-odd coefficient on a four-point FFVV contact or a
   self-conjugate dipole.** The amplitude oracle never had imaginary Wilson
   coefficients on those vertex classes, which is why the defect survived.
   Close it with a committed standalone table, for example this subprocess at card
   6, or a SMEFTsim row with `ctGIm`/`ctWIm` ≠ 0 in a channel with a contact
   vertex. The table must fail on main and pass with the fix.
5. **Whether MadGraph's half-of-the-pair binding gives the physical CP-odd sign is
   untested here.** The project follows MadGraph by convention, and this report
   recommends the fix as parity. An analytic CP-odd observable would settle it
   independently, for example the sign of the top's normal polarisation, or of a
   T-odd triple product, in single top with Im ctW, against the dim6top note or
   Protos.
6. **The τ, ττ flavour group of card 6 converges poorly on this crate's side**
   (0.0136 ± 0.0074 pb at seed 1, against 0.008 pb for its neighbours). This is the
   same-flavour set with all three leptons massive τ's and 150 diagrams. Its
   share of the 5-seed spread (χ²/dof 1.85) is worth a look once the σ-level gap is
   understood.

## Corrections to the brief

- **The output directory.** The "MadGraph 3.5.7 … Write MadGraph outputs under
  `/home/user/wt/sprintl-dim6top-sigma-mg/`" paragraph conflicts with the dispatch,
  which names `/home/user/wt/sprintl-dim6top-mg/` and MadGraph 3.7.1. I used the
  latter, and MadGraph 3.7.1 copied from the worktree submodule. The
  main checkout's madgraph pixi env supplied only `python` and `gfortran`.
- **"vibegraph ran the unrestricted bundled model".** MG_SETTINGS says the bundled
  UFO's `restrict_default.dat` equals the gridpack's `restrict_model.dat`, and a
  bare model name applies it, as in MadGraph. Without the restriction this crate
  cannot compile the subprocess at all (FFFF12 refused, 193 diagrams). The
  validator's run was therefore restricted, and suspect 4 is not live. For the
  record, "the restriction only removes zero couplings" is right in effect but not
  in count: the unrestricted model has 118 more diagrams on this subprocess. I
  could not check their couplings for zero on this crate's side, because it
  refuses to compile them.
- **The UFO has no restrict card.** The `dim6top_LO_UFO-restrict` the proc card
  imports needs one, and I had to rebuild it from the condensed description (see
  Provenance). The banked parameters are card 6's, so the placeholders only decide
  which couplings survive.
- **MadGraph 3.7.1 cannot run the proc card verbatim** (`FCNC=0`, Found 2). The
  MadGraph lines here drop it.
- **The census expectation.** The brief expects the two censuses to be compared as
  counts. On same-flavour sets they differ by design (×2 here, closed under
  identical-particle permutations), so the comparison has to apply that rule. With
  it, the counts match exactly.

## Follow-up: card 6's own cuts, and the σ the defect produces

Branch `sprintl-dim6top-sigma`, starting at `be45128c`. `origin/main` was still
`c095a1cf`, already merged (`git merge-base --is-ancestor origin/main HEAD`). Inputs
from the validator:

- The run card's cut and scale lines. Active cuts: `ptj = 0.1` and `mmll = 30`.
  `maxjetflavor = 5`, `cut_decays = False`, `bwcutoff = 15`, `dynamical_scale_choice
  = -1`, lhaid 325300.
- A correction to this report's earlier premise. vibegraph does not refuse card 6:
  at `maxjetflavor = 5` a b is a jet, so `ptj = 0.1` regulates the spectator b~. My
  earlier refusal was at the MadGraph 3.7.1 default card, `maxjetflavor = 4` with
  `ptb = 0`, and my earlier reduced σ used `ptb 20`/`etab 5`. Neither was card 6's
  regulator.

**The run card used here.** `sigma/run_card_gp.dat` (sha `b66cbc27…`) is MadGraph
3.7.1's default card for the process with the validator's values set: `ebeam 6800`,
`maxjetflavor 5`, `ptj 0.1`, `ptb = ptl = 0`, `etaj = etab = etal = -1`, every ΔR 0,
`mmll 30`, `mmbb 0`, `cut_decays F`, `bwcutoff 15`, `dynamical_scale_choice -1`,
`scale = dsqrt_q2fact = 91.188`. One substitution: lhaid **247000**
(NNPDF23_lo_as_0130_qed) in place of 325300. Set 325300 is not on this container, and
`--no-network` was kept. The same set is read through LHAPDF 6.5.6 on both sides,
so the comparison is like for like. The PDF is not the variable under test.

### 1. mmll and cut_decays

What each side's code does:

- **MadEvent 3.7.1.** `Template/LO/SubProcesses/setcuts.f` sets `do_cuts(i) =
  .false.` for every `from_decay(i)` when `cut_decays` is F (lines 201–203).
  `from_decay` comes from configuration 1's forced Breit–Wigner lines
  (`check_decay`, lines 989–1000). `s_min(j,i) = mmll·|mmll|` is set only when
  `do_cuts(i).and.do_cuts(j)`, both legs are leptons, they have the same |PDG|, and
  the product of their PDG codes is negative (lines 391–399). So only the core
  l+l− pair is cut. A pair with the W-decay lepton is never cut, whatever its
  flavour.
- **vibegraph (`vibegraph-lib/src/cuts.rs`).**
  - The header documents the same rule (`setcuts.f:192`, `:217`, `:399`): a leg
    descending from a forced line loses its single-leg and pairwise cuts.
  - On same-flavour sets, where the W-decay e+ and a core e+ are identical, every
    pairing is kept coherently. Each pairing exempts its own decay products, and its
    indicator is weighted by its share of |M|². MadEvent instead keeps one pairing.
    That changes σ only by the interference between pairings.
- **2.9.18.** The pinned submodule copy has no git history, so 2.9.18's `cuts.f`
  and `setcuts.f` could not be diffed here.

**Measured per event.** I integrated and then generated 500 events, at the card-6
param card with the card-6 cuts, on the distinct-flavour set (`sigma/chain.dat`) and
on the same-flavour set `g u > t b~ d e+ e-, (t > w+ b, w+ > e+ ve)`
(`sigma/chain_sf.dat`). Command: `/home/user/wt/logs/ld1-cuts.sh`, log
`ld1-cuts.log`, then `python3 harness/ld1_lhe_cuts.py sigma/gp_<c>/events.lhe`.
The core and decay legs are told apart by LHE mother pointers (a leg descending from
the top is a decay product).

```
== chain
min core m(l+l-) 32.212 GeV; min over every OS same-flavour pair 32.212 GeV; events with some OS SF pair below 30: 0
core m(l+l-) histogram: [30,60):16 [60,80):15 [80,85):8 [85,88):27 [88,90):61 [90,92):213 [92,94):91 [94,97):33 [97,100):9 [100,120):20 [120,200):6 [200,10000):1
== chain_sf
min core m(l+l-) 30.069 GeV; min over every OS same-flavour pair 5.961 GeV; events with some OS SF pair below 30: 21
```

**mmll matches MadEvent.** The core pair is always above 30 GeV. A pair formed with
the W-decay e+ is not cut: 21 of 500 events have it below 30 GeV, as MadEvent's
`from_decay` exemption implies. There is no uncut γ* pole: the core m(ll) peaks at
the Z.

### 2. ptj on the spectator b~

What each side's code does:

- **MadEvent.** `setcuts.f:217–221`: a final leg is `is_a_j` if `|pdg| ≤
  min(maxjetflavor, 6)`, and `is_a_b` only if `maxjetflavor < |pdg| ≤ 5`. At
  `maxjetflavor = 5` the b~ is a jet and takes `etmin = ptj = 0.1`
  (`setcuts.f:272`). `ptb = 0` is never read, because no leg is `is_a_b`. The
  decay b is `from_decay` and stays uncut.
- **vibegraph.** `cuts.rs` uses the same class rule ("a jet if `|pdg| ≤
  min(maxjetflavor,6)`; a b if `maxjetflavor < |pdg| ≤ 5`").

**Measured per event.** I compared vibegraph's 500 events with 2000 MadGraph 3.7.1
events of the same subprocess at the same cards (below). Command: `python3
harness/ld1_lhe_cmp.py sigma/mg_chain_events.lhe 1.497 sigma/gp_chain/events.lhe
2.497`.

```
min pT(b~): A(MG) 0.1011  B(vg) 0.1024        [chain_sf, vg: 0.1001]
```

**ptj matches.** Both sides cut the core b~ at 0.1 GeV and nothing else. The σ is
spread across the collinear logarithm. In vibegraph's events 28% of σ has pT(b~) <
1 GeV and 51% has pT(b~) < 5 GeV (`ld1_lhe_cuts.py`), with the median at 4.8 GeV.
This regulator makes σ very sensitive to the treatment of pT(b~) → 0. Both sides
apply the same regulator.

### 3. The cuts match, so the σ was measured directly: MadGraph against vibegraph on one subprocess

**Both cuts match, so neither explains the excess.** The manager's step 3 allows a
reduced MadGraph σ on one subprocess if cheap. It was cheap: about 2 minutes. I ran
MadGraph 3.7.1 `output madevent` of the same subprocess (`sa/me_sigma`, a copy of
`sa/madevent`) with the same run card. The param card was the restricted model's
with card 6's values (`param_card_edited.dat`), or with imaginary coefficients zero
(`param_card_edited_noI.dat`). Settings: `run_mode 2`, `nb_core 3`, 2000 unweighted
events. Commands: `/home/user/wt/logs/ld1-mesigma.sh` and
`/home/user/wt/logs/ld1-noI.sh`, logs `ld1-mesigma-mg.log`, `ld1-mesigma-noI-mg.log`
and `ld1-noI.log`. vibegraph used `integrate --target-rel 0.02` with seed 20260719,
on `param_card_card6.dat` or on its noI copy `sigma/param_card_card6_noI.dat` (sha
`fe569590…`, with ctpI, ctWI, ctZI and ctGI set to 0, and Γ_t kept at 1.3129 on both
sides).

| `g u > e+ ve b b~ d mu+ mu-`, card-6 cuts | vibegraph | MadGraph 3.7.1 | vg/MG |
|---|---|---|---|
| card 6 (imaginary coefficients on) | 2.497 ± 0.032 fb (χ²/dof 387) | 1.497 ± 0.0083 fb | **1.668 ± 0.023** |
| ctXI = 0 | 1.497 ± 0.012 fb (χ²/dof 559) | 1.481 ± 0.010 fb | **1.011 ± 0.011** |
| same-flavour set `… e+ e-`, card 6 | 2.546 ± 0.044 fb | not run | |

**The defect is the whole of the gap.** This one subprocess reproduces the card's
1.655 ± 0.074 to within its error, and setting the imaginary coefficients to zero
removes it to 1%. That is the amplitude discriminator of the main section, now at σ
level. The PDF and the run card do not enter the conclusion; each side is compared
with itself, at the same cards. One consequence is worth stating: MadGraph's σ
barely moves with the CP-odd coefficients (1.497 against 1.481). That is what CP-odd
terms should do in a CP-even observable, which they enter only quadratically.
vibegraph's mis-bound CP-odd terms instead interfere as if they were CP-even, adding
+67%.

**The per-event distributions point the same way.** The comparison above
(`ld1_lhe_cmp.py`) gives these ratios of the per-bin partial cross sections:

| pT(b~) bin (GeV) | vg/MG, card 6 | vg/MG, ctXI = 0 |
|---|---|---|
| [0.1, 0.3) | 1.58 | 1.02 |
| [0.3, 1) | 1.38 | 0.87 |
| [1, 3) | 1.73 | 1.18 |
| [3, 10) | 1.47 | 1.10 |
| [10, 30) | 1.70 | 0.79 |
| [30, 100) | 1.55 | 1.22 |
| [100, ∞) | 3.18 | 0.80 |

The ctXI = 0 column is statistically flat. The noI file pair is
`sigma/mg_chain_noI_events.lhe` and `sigma/gp_chain_noI/events.lhe`, each bin about
±15% at 500 events. The scales agree at zero imaginary coefficients: median SCALUP
194 against 194, and median AQCDUP 0.124 against 0.123. With card 6, vibegraph's
median SCALUP is 243 against MadGraph's 195. That shift is kinematic: vibegraph's
spurious terms grow with energy.

**The |M|² on MadGraph's own events.** `harness/ld1_events_m2.py` evaluates the
helicity-summed |M|² of the t-channel-zeroed standalone `MATRIX` at the momenta of
MadGraph's 2000 unweighted events. This crate's production `eval_m2` was evaluated
at the same momenta (`LD1_M2ONLY=1 harness/run_probe.sh ev_w ev_noI_w`):

```
ev_w:     2000 MG unweighted events; mean of |M_vg|^2/|M_MG|^2 = 1.3192 +- 0.0623; median 1.0045; 5%-95% [0.867, 1.935]
ev_noI_w: 2000 MG unweighted events; mean of |M_vg|^2/|M_MG|^2 = 1.0000 +- 0.0000; median 1.0000; 5%-95% [1.000, 1.000]
```

The mean of this ratio over MadGraph's events estimates σ_vg/σ_MG. It is heavy
tailed and biased low, because it samples MadGraph's distribution and so
under-represents the regions where vibegraph is much larger. It is consistent with
the 1.67 measured directly. At zero imaginary coefficients it is 1 to every digit.

**Why the earlier |M|² scan said "percent".** Its points were RAMBO t b~ d Z at a
fixed √ŝ with isotropic decays. They almost never place the b~ collinear with the
beam, which carries most of card 6's σ (51% at pT(b~) < 5 GeV). They also miss the
high-energy tail, where the ratio is 3.2. The scan measured the defect where it is
small. The finer oracle was one cheap MadEvent σ, and I should have reached for it
first.

### What each oracle in this section cannot see

- **The per-event cut checks.** They show that each cut is applied as MadEvent
  applies it at 500 events. A cut that is wrong only in a region those events do
  not reach, or only in other subprocesses, would pass. Only one distinct-flavour
  and one same-flavour set were checked.
- **The σ comparison.** It is one subprocess and one seed per side. The vibegraph
  runs have χ²/dof 387 and 559 across iterations. The noI agreement at 1.011 ±
  0.011 says those runs are not grossly wrong, but it is not a seed sweep. It also
  uses lhaid 247000, not 325300.
- **What this does not settle.** That the fix restores every subprocess. The
  charge-conjugate amplitude control says it should, but no σ was run with the
  fix, since that needs production code.

### Requests for the long-run host (replacing the earlier list)

1. **Once the fix of the main section is implemented on a feature branch**, card 6
   at campaign budget, 5 seeds, with the gridpack's run card (lhaid 325300).
   Expected: vg/MG ≈ 1.00 against 0.04754 pb, within the seed spread. Gate it on the
   sweep's spread and χ²/dof, not on a single pull.
2. **A pre-fix control on main**: card 6 with ctpI = ctWI = ctZI = ctGI = 0 in both
   vibegraph and MadGraph 3.7.1 (the gridpack's σ cannot be reused, since it is at
   nonzero coefficients). 5 vibegraph seeds and 3 MadGraph seeds. Expected: ratio
   about 1.0, the whole-card version of the 1.011 here.

### Found (additions)

7. **A σ-level oracle for CP-odd EFT cards exists and costs minutes.** A one-subprocess
   MadEvent σ at the card's own run card, on and off the imaginary coefficients,
   located this gap more cheaply than any amplitude scan. Every CMS EFT card with
   imaginary Wilson coefficients (dim6top, SMEFTsim `*Im`) should get this pair
   before σ is compared. Other cards in the campaign may carry the same defect.
8. **vibegraph's chain integration zeroes many points as "degenerate" at
   `ptj = 0.1`.** 176361 of 1143312 were zeroed in the distinct-flavour chain run
   (`ld1-cuts.log`), while the same-flavour run zeroed 1 of 2.0M. At ctXI = 0 the σ
   still agrees with MadGraph, so this cost no σ here, but a 15% zeroed share is
   worth explaining. It is an efficiency or robustness question at near-collinear
   b~, not a σ defect shown here.

### Scratch files added (all under `/home/user/wt/sprintl-dim6top-mg/`)

- `sigma/`: `run_card_gp.dat`, `param_card_card6_noI.dat`, `chain_sf.dat`,
  `gp_chain/`, `gp_chain_sf/`, `gp_chain_noI/` (grids and 500-event LHE files),
  `mg_chain_events.lhe`, `mg_chain_noI_events.lhe`.
- `sa/me_sigma/`: the MadEvent run directory, with runs `run_ld1` and `run_ld1_noI`.
- `harness/`: `ld1_lhe_cuts.py`, `ld1_lhe_cmp.py`, `ld1_events_m2.py`.
- `tables/`: `ld1_gu_ev_{w,noI_w}.json` and `vg_gu_ev_{w,noI_w}.json`.

The scratch probe was again copied into `vibegraph-lib/tests/` for one run and
deleted afterwards (`ls vibegraph-lib/tests/ld1_probe.rs` → no such file).
