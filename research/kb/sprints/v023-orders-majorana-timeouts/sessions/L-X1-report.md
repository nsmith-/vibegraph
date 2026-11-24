---
type: Session Report
title: "L-X1 report: TpTp is a card defect, bstar_tW is a Goldstone-flag spelling, the stitch assert refuses, the zero-weight refusal names its vertex"
description: "TpTp zero traced to KS10D(3,3)=0 in the gridpack's param card; bstar_tW traced to a GoldstoneBoson spelling vibegraph did not read; the stitch assert became a refusal; the b-quark sigma excess is deferred to a MadGraph 3.7.1 regeneration."
status: draft
tags: [validation, model-loading, goldstone, refusal, cms-gridpacks]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-22}
---
Branch `sprintl-leftovers` off main a0a6eb4. Session L-X1 (renamed from L-V1;
the brief is `sessions/L-X1.md`). Closed items, files deleted in the final
commit: `tptp-integrand-zero-on-every-point`, `bstar-tw-zero-effective-points`,
`stitch-set-sign-assert-panics`. Not closed: `sigma-reads-high-on-mlm-and-wbq-cards`.

## What changed

1. **Stitch assert** (`vibegraph-lib/src/diagrams/chain.rs`). The
   `assert_eq!` on a graph reached twice with opposite signs is a
   `Result::Err` carrying `opposite_signs_refusal`. No card reaches the
   branch, so the test pins the refusal text only.
2. **Zero-weight refusal names the vertex** (`vibegraph-lib/src/support.rs`
   `zero_coupling_subprocesses`, called from `vibegraph-cli/src/integrate.rs`).
   When the matrix element is zero on every point that passed the cuts, the
   message lists the subprocesses whose every diagram holds a vertex whose
   couplings all evaluate to exactly zero, with the vertex and coupling names.
3. **Goldstone flag, any spelling** (`vibegraph-lib/src/ufo/particles.rs`).
   A particle is a Goldstone when any of `goldstoneboson`, `GoldstoneBoson`,
   `goldstone` is true, as MadGraph's `import_ufo.py` L1249-1264 reads it.
4. **List or dict `propagator`** (same file, `propagator_ref`): read as its
   first (unitary-gauge) entry, as `import_ufo.py` does. Before, such a keyword
   was read as no override, silently.
5. Reconstructed campaign cards for 108 and 157 under `cards/`, for the
   validator to diff against the originals on the validator host.

## TpTp (cards 108, 510-512): a card defect that MadGraph shares

- Model `eVLQS012_v8_4FNS_UFO` from `eVLQ_v8.1.tar.gz` (CMS model store). The
  only `s0 b b~` vertex is `V_45` with couplings `GC_26 = -KP10D3x3` and
  `GC_48 = complex(0,1)*KS10D3x3`; both parameters default to 0
  (`parameters.py`, blocks `KP10D` and `KS10D`).
- The gridpack's customize card sets `set param_card ks10d3x3 3 1.0`, a
  name-plus-index form MadGraph did not apply (validator's reading of the
  gridpack run; the `ks10e 2 2 0.01` line was applied). The gridpack's own
  MadGraph σ is 0 (MG 2.9.18). Gridpack:
  `.../REANA/TpTpTo2T2STo2Mu2B_MS30_el8_amd64_gcc10_CMSSW_12_4_8_tarball.tar.xz`.
- **The backlog item's high priority rested on a false premise**: the survey
  had no MadGraph σ to compare with. MadGraph reads zero too.
- Control, same card, same budget (`--fixed-budget --neval 20000 --niter 3`,
  built-in dev profile binary):
  - card as the campaign ran it (KS10D zero):
    `ERROR every one of the 60000 points ... has zero weight, though 29992 passed
    the cuts` (the item's symptom, reproduced);
  - with `BLOCK KS10D / 3 3 1.0`: `σ = 1.679 ± 0.043 nb`, 8643 of 20001 kept
    points carry weight (effective sample size 1421). This value has no
    MadGraph counterpart and is not a measurement of anything: the card's
    widths (Γ(tp)=0.1, Γ(s0)=0.01 GeV) do not match couplings of order one.
- The refusal now says, on the zero card, `in 10 subprocess(es), g g > mu+ mu-
  t b b~ t~, u u~ > mu+ mu- t b b~ t~ and 8 more, every diagram holds a vertex
  whose couplings all evaluate to zero: V_45 (GC_26, GC_48)`.
- Test: `vibegraph-cli/tests/cli_support_refusals.rs`
  `a_coupling_zero_at_the_parameters_in_force_is_refused` (toy model,
  `V_ltltvt (GC_LV)`). It fails without the naming. It cannot detect a vertex
  that is only partly zero or whose couplings are non-zero and cancel.

## bstar_tW (card 157): the b' model's Goldstone was never dropped

**The hypothesis in the brief's addendum (the Breit-Wigner tau map rule for a
wide b') is rejected.** The cause is a model-loading mismatch.

- `Bprime_no_b_mass/particles.py` marks `phi+` with `GoldstoneBoson = True`.
  vibegraph read `goldstoneboson` and `goldstone` only
  (`particles.rs:233-235` at a0a6eb4); MadGraph's importer accepts the third
  spelling (`research/refs/mg5amcnlo/models/import_ufo.py` L1249-1264) and drops
  the particle. `phi+` has mass `MW` and width `ZERO`, so its propagator has an
  undamped pole inside the W window, and its coupling to `c s~` grows with the
  charm Yukawa: a few points carry the weight.
- Minimal card (`p = g b`, `p p > t w- QAD=2 QCD=0 QED=0, t > b c s~, w- > e-
  ve~`, `--fixed-budget --neval 50000 --niter 3`):
  - before the fix, 6 of 6 seeds refuse: effective sample size 1.1, 1.3, 5.0,
    1.5, 1.8, 2.1;
  - the `u d~` row of the same card integrates: 31.41, 31.30, 31.27, 31.45 ab
    at four seeds, effective sample size 6900-8400;
  - charm mass scan with the Yukawa moved with it: 0.00255 integrates
    (31.51 ab); 0.2, 0.5, 1.0, 1.42 GeV fail (effective 19.7, 1.9, 3.2, 1.1);
    `mu`, `md`, `ms` set to 1.42 do not fail;
  - `--map-tau breit-wigner` (effective 1.8), `--map-tau inverse-square` (2.6),
    `--map-tau log` (1.1), `--map-split-angle isotropic` (1.9), `bwcutoff 5`
    (8.8) do not remove the failure;
  - `enumerate` of `t > b c s~`: 2 diagrams, one through `phi+`, before; 1 after.
- After the fix: `c s~` row 30.75 ± 0.32 ab, `e` row 30.72 ± 0.33 ab (before
  the fix the integrating `u d~` row read 31.3 ± 0.2 ab: **a 2% shift on a row
  that integrated, so pre-fix sigma on any model with this spelling was biased,
  not only inefficient**).
- Full card 157 (reconstructed cards, `--fixed-budget --neval 50000 --niter 3`,
  seed default): `σ = 2.18 ± 0.39 fb` against MadGraph 2.9.18's
  1.7755e-3 pb ± 2.8e-6 pb (1775.5 ab): +1.0 sigma. The effective sample size is
  31 at this budget, 11 above the 20 floor; per-subprocess errors reach 77%.
  **This is rough agreement, not a gate.** The pre-fix run refused with an
  effective 16.8 points at the same budget.
- The tau-map rule is not involved: `NARROW_TAU_SHARE` is
  `vibegraph-lib/src/phasespace/maps.rs:148` and `narrow_in_tau` is line 156;
  the log map integrates the b' pole (the 4-seed spread of the `u d~` row is
  0.3%).

### Tests (they fail on the old reading)

`vibegraph-lib/tests/toy_models.rs`:
`a_goldstone_flag_in_any_spelling_removes_the_scalar_from_the_diagrams` (neutral
scalar) and `a_charged_goldstone_flag_removes_its_diagrams_and_keeps_the_vectors`
(neutral fermion, charged vector and charged scalar, `lt nt > lt nt`); and
`vibegraph-lib/src/ufo/particles.rs` unit tests for the three spellings and the
list/dict propagator. Measured against the old reading (the particles.rs
`is_goldstone` list reduced to the two old spellings, then restored):

```
a_goldstone_flag_in_any_spelling_removes_the_scalar_from_the_diagrams ... FAILED
  GoldstoneBoson: 8 diagrams with the scalar flagged, 8 without
a_charged_goldstone_flag_removes_its_diagrams_and_keeps_the_vectors ... FAILED
  GoldstoneBoson: 2 diagrams with the scalar flagged, 2 without
```

What they cannot detect: a Goldstone marked some other way (the importer's
`model['gauge']` rule below), the coupling structure of a real model's Goldstone
vertices, and any change of sigma (they pin the enumeration).

### Measured: no banked model uses the spelling

```
$ grep -rIl "GoldstoneBoson" validation/madgraph/output research/refs/mg5amcnlo/models validation/ufo vibegraph-lib
research/refs/mg5amcnlo/models/import_ufo.py
research/refs/mg5amcnlo/models/usermod.py
vibegraph-lib/src/ufo/particles.rs
```

The first two are MadGraph's own code, the last is this change: no model file
in the worktree carries the spelling, and the interned Standard Model uses
`goldstoneboson`. The models fetched by `pixi`/`validation/fetch*.sh` were not
searched here. Validator census (70 bundled UFOs): 9 spell it
`GoldstoneBoson=True`; loaded by v022-1009: `Bprime_no_b_mass` 7 cards (156,
157, 875-879), `MSSM_SLHA2` 49, `NMSSMHET` 3, `loop_sm_twoscalar` 21; refused:
`heco_spinhalf_withZ0` 5, `loop_sm` all; `g2HDM`, `SM_alp`, `eVLQ` use
`goldstone=True` and are untouched.

### Measured: no banked model has a list or dict `propagator`

```
$ grep -rnE "propagator\s*=\s*[\[({]" $(find . -name particles.py -not -path "./target/*") \
    <scratchpad>/lv1-models/*/*/particles.py | wc -l
0           # 26 repository particles.py files plus the eVLQ and Bprime tarballs
```

The only matches anywhere are the `object_library.py` class body
(`self.propagator = {0: propagator, 1: propagator}`), which builds the dict from
a single entry. The change moves no banked sigma.

## Particle-keyword audit against MadGraph's importer

vibegraph `vibegraph-lib/src/ufo/particles.rs` against
`research/refs/mg5amcnlo/models/import_ufo.py` (`add_particle`, L1230-1330).

| keyword | vibegraph | MadGraph | match |
|---|---|---|---|
| goldstone flag | any of `goldstoneboson`, `GoldstoneBoson`, `goldstone` true (after this change; before, the first and last, first match winning) | L1249, 1251, 1261-1264: all three, any true | fixed |
| `propagator` list/dict | first entry after this change; before, none | L1292-1303: `value[0]` in unitary gauge | fixed |
| negative `spin` / `GhostNumber` | ghost when `GhostNumber != 0` | L1237: a tree-level model drops `spin < 0`; `GhostNumber` ignored (L1311) | differs; 681 particles in 30 local `particles.py` files, 0 disagree |
| model `gauge` list | not read; Goldstones always dropped | L1245: dropped when `unitary_gauge` and `0 in gauge`, or `1 not in gauge` | differs for a `gauge = [1]`-only model; every local model lists 0 |
| `propagating` | not read | L1285: `False` sets `line` to `None` | not measured here; I did not read the diagram-generation side |
| `charge`, `mass`, `width`, `spin`, `color`, `pdg_code`, `name`, `antiname` | read from the model | read from the model (L1273-1291) | match |
| quantum numbers (`LeptonNumber`, `Y`, ...) | ignored | L1313-1317: collected for a process-level conservation shortcut | match on diagram content |
| `line`, `texname`, `antitexname` | read, cosmetic | read, cosmetic | cosmetic, not changed |

The spin/ghost scan was a regex over `Particle(...)` blocks of the particles
files in `validation/ufo`, `research/refs/mg5amcnlo/models` and the fetched
tarballs, comparing `spin < 0` with `GhostNumber != 0`.

## Item 4: the b-quark sigma excess (188, 260, 967): not attributed

No attribution is made. The first step is a MadGraph 3.7.1 regeneration on the
exact cards, which the validator is running (967 first, then 260 and 188). 188's
reference is the gen_log value 13220 ± 19.05 pb by design. If 3.7.1 confirms
967's +1.3%, it goes to the Breit-Wigner window item. What to return from the
regeneration, for each card: `results.dat` and the log's cross section, the MG
version, and for 188 and 967 the LHE event files at the campaign cuts with the
per-event `SCALUP` / `AQCDUP`, the MLM merge sequence and the b-quark content;
then compare per event against vibegraph's per-event scales before any sigma
reading.

## Validator requests

1. **Card 157, five seeds or more, full budget**: branch `sprintl-leftovers` at
   the pushed head, the reconstructed cards in
   `research/kb/sprints/v023-orders-majorana-timeouts/cards/157/` (diff them
   against the originals), expected sigma 1.7755e-3 pb ± 2.8e-6 pb (MG 2.9.18),
   read the seed spread, `χ²/dof` and the effective sample size, not the headline
   pull.
2. **Enumerate-diff impact check** of main against the fix on the released-card
   set (48 PASS, 1 FAIL): every card that loads `Bprime_no_b_mass`,
   `MSSM_SLHA2`, `NMSSMHET` or `loop_sm_twoscalar`.
3. The two sha256 values the cards quote for the bundled UFOs
   (`0f2b2b30...` for eVLQ, `3f142c21...` for Bprime) differ from the tarballs
   fetched from the store (`af794ddd...`, `fc052fa1...`); the card hashes may
   be of the unpacked directories. Worth one check.

## Gates

```
$ cargo fmt --all --check                                  # clean
$ cargo clippy --workspace --all-targets -j 3 -- -D warnings   # Finished, 0 errors
$ cargo test --workspace -j 3
69 test-result lines: passed=2246 failed=0 ignored=26
```

`pixi run validate` was not run (the brief forbids it locally; CI's banked job
runs it). The change touches model loading and diagram enumeration only for
models that spell the flag `GoldstoneBoson` or use a list propagator; the grep
above shows no banked model does.

## Found

- **Which campaign cards load a `GoldstoneBoson` model**, and how far each
  one's sigma moved: pre-fix sigma was biased on any such card, not only
  inefficient (the 2% on the integrating `u d~` row). `MSSM_SLHA2` is 49 cards,
  `loop_sm_twoscalar` 21, `NMSSMHET` 3, `Bprime_no_b_mass` 7. Closes when the
  enumerate diff and a sigma comparison on a sample of each family is recorded.
- **Which campaign models use list or dict `propagator` entries.** None of the
  26 repository and two fetched model sets does; the validator's census can
  extend this.
- **A model with `gauge = [1]` only** keeps Goldstones in MadGraph (`import_ufo.py`
  L1245) and vibegraph always drops them: a silent wrong answer, or a refusal
  needed. No local model has it.
- **`GhostNumber` against `spin < 0`** decides ghosts differently in the two
  programs; no local model separates them.
- **VLferm_UFO has a particle named `Gp` with no Goldstone flag** (G0 and G+ are
  flagged `goldstone`). MadGraph also treats it as physical, so it is not a
  spelling mismatch, but VLferm is the family with the unexplained +1-4%
  offset. What is `Gp`? Not investigated here.
- **Zero sigma prints about 130 digits** in the per-subprocess log lines
  (`σ = 0.000...0 ± 1e-130 zb`). A subprocess that is exactly zero should print
  `0`. Cosmetic.
- **The effective-sample-size gate is on the combined integral**, so a model
  with 162 subprocesses and a heavy-tailed minority is refused as a whole; the
  refusal does not name the subprocess that carries the weight. Naming the top
  subprocess by weight share would have shortened this diagnosis from hours to
  minutes.

## Manager addenda (after the session)

- **The card 157 reconstruction matches the gridpack (host-measured).** The validator diffed
  `cards/157/` at ff19928 against the staged campaign cards.
  - proc_card is identical, apart from the order of `QED=0 QCD=0` and the gridpack's trailing `output` line.
  - All 29 param_card entries and all 34 run_card entries set here have equal values in the gridpack's cards.
  - The gridpack's cards also carry zero masses and widths, the internal MW, QNUMBERS blocks and template-default run-card entries.
- **Enumerate diff (host-measured).**
  - The 49 released PASS/FAIL cards on MSSM_SLHA2, NMSSMHET and loop_sm_twoscalar have identical diagram sets on v0.2.2 and ff19928, so the fix moves no released verdict.
  - Bprime cards 156, 877 and 878 go from 396 to 198 diagrams; every removed diagram has a phi± propagator.
- **The store models are not the gridpacks' bundled models.** I computed the validator's model-identity
  hash (sha256 over the sorted lines `relpath<TAB>sha256(file)\n`, *.py only) on the current CMS-store tarballs:
  - `Bprime_no_b_mass`: 48f991df0791 (store) against 3f142c212e44 (gridpack).
  - `eVLQS012_v8_4FNS_UFO`: 2d5c0dc08a52 (store) against 0f2b2b305ab1 (gridpack).

  The TpTp and bstar conclusions stand on the gridpacks' bundled models: the validator confirmed the zero coupling and ran the enumerate diff on them, and the 157 gate runs on them too. This session's local reduced-budget 157 run (2.18 ± 0.39 fb) used the store model, so it is indicative only. **Found:** the CMS-store tarball and a gridpack's bundled UFO can differ under the same model name, so a local reproduction of a campaign card should use the bundled copy.

## Gate evidence for PR 11 (#177, merged as 970be89d)

The validator ran these gates (vg-validator). The manager relayed them; they were not re-run here.

**Builds**
- The σ rows ran on 8a651703, which isolates PR 11's change.
- The HNL binding rows ran on ba190428. That build is the main merge 187f7ba0 plus the user's ruling (A): an unknown code inside a block the model defines is ignored with a warning.

**Command for every σ row**

```
vibegraph integrate <gpdb-staged gridpack cards + bundled UFO> --fixed-budget --neval 120000 \
  --niter 3 --parallel-diagrams --seed N -j 4 --no-network --output-ldjson
```

- The driver is `chirality_cards.py --campaign`.
- The one exception is the card-878 4× rung, which used `--neval 480000`.
- Each card ran on its gridpack's bundled UFO, confirmed by identity hash: Bprime_no_b_mass 3f142c212e44, SM_HeavyN_Gen3Mass_NLO bba6f1be6b0f.
- Outputs are on the validator host under `$B/`:
  - `pr11-157` and `pr11-157-ext`
  - `pr11-cards`
  - `pr11-bprime` and `pr11-bprime-ext`
  - `pr11-878-4x`
  - `pr11-parambind`
  - `hnl-pr11-ba190428`
  - `paramcard-pr11-8a651703`

### Card 157 (bstar_tW), 20 seeds

MG reference: 1.7755e-3 ± 2.8e-6 pb (MG 2.9.18). Each seed takes 11 s, and the ESS is only 345–764 per seed. Seeds 6–20 were added for that reason.

| seed | σ (pb) | err | ESS |
|---|---|---|---|
| 1 | 1.8292e-3 | 3.6% | 742 |
| 2 | 1.7100e-3 | 3.8% | 685 |
| 3 | 1.7942e-3 | 4.4% | 505 |
| 4 | 1.8337e-3 | 4.6% | 471 |
| 5 | 1.9053e-3 | 5.2% | 368 |

- **Seeds 1–5:** unweighted mean 1.8145e-3, sd 3.9%, χ²/dof 0.91, vg/MG 1.0219 ± 0.0180.
- **All 20 seeds:** unweighted mean 1.7878e-3, sd 3.25%, sem 0.73%, χ²/dof 0.64 over 19 dof.
- **Verdict:** **vg/MG = 1.0069 ± 0.0075, pull +0.92.** The per-seed pulls run from −1.42 to +1.32.
- The session's local 2.18 fb (store model, reduced budget) was off.

### Bprime cards 156, 877 and 878, 20 seeds each

The fix takes each card from 396 to 198 diagrams; every diagram it removes has a phi± propagator.

| card | 5-seed vg/MG | 20-seed mean (pb) | sd | χ²/dof | seeds below MG | ESS | vg/MG | pull |
|---|---|---|---|---|---|---|---|---|
| 156 | 1.0138 ± 0.0216 | 0.0029517 | 4.67% | 1.23 | 11/20 | 127–1101 | 0.9925 ± 0.0105 | −0.71 |
| 877 | 1.0051 ± 0.0133 | 0.05308 | 3.18% | 0.64 | 10/20 | 318–1085 | 1.0023 ± 0.0073 | +0.31 |
| 878 | 0.9660 ± 0.0096 | 0.42746 | 3.52% | 1.12 | 14/20 | 455–1421 | 0.9886 ± 0.0079 | −1.44 |

**878 at 4× budget, 5 seeds:**
- The seeds read 1.0070, 1.0039, 0.9766, 0.9921 and 0.9855, each ±0.9–1.1%, with ESS 7514–11467.
- The mean is 0.9930 ± 0.0059 (pull −1.19, χ²/dof 1.94).
- The 5-seed −3.5σ was few-seed scatter. Those five seeds had an sd of 2.19%, against 3.52% over all 20.

### DarkPhoton cards 78, 79, 418 and 419, untrimmed (MASS 1000022 present)

- **Binding:** each card gets exactly one warning, the same at enumerate, check and integrate. Card 78's warning, verbatim:
  `MASS 1000022 sets 13.33333, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored`
- **MZd:** each card's own value is used. The Zp leg mass in the integrate channel lines reads 40, 86, 20 and 70, matching the cards.
- **Decay table under 1000022:** none of the gridpack cards has one. A scratch copy of card 78's card with `DECAY 1000022 2.5e-1` and two BR lines binds with rc 0, giving these warnings:
  `the decay table (2 channels) below DECAY 1000022 is for a particle the model does not have, so the table is ignored`
  `DECAY 1000022 sets 0.25, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored`

**σ against gpdb MG.** The 1-seed column ran at `--target-rel 0.02`. The 5-seed column ran at campaign budget, and its σ is the 5-seed unweighted mean.

| card | MZd | 1-seed σ/MG | 5-seed σ (pb) | sd | χ²/dof | vg/MG | pull |
|---|---|---|---|---|---|---|---|
| 78 | 40 | 0.9974 | 0.21385 | 1.05% | 1.18 | 0.9869 ± 0.0052 | −2.54 |
| 79 | 86 | 0.9962 | 4.4206 | 0.75% | 0.52 | 0.9977 ± 0.0038 | −0.62 |
| 418 | 20 | 1.0023 | 0.21725 | 1.24% | 1.08 | 0.9938 ± 0.0065 | −0.95 |
| 419 | 70 | 1.0039 | 0.39617 | 1.13% | 1.13 | 0.9909 ± 0.0053 | −1.70 |

- Card 78's five seeds read 0.9922, 0.9864, 0.9983, 0.9704 and 0.9869. The earlier hand-trimmed runs gave 0.985 (MZd 40) and 0.971 (MZd 86).
- **Card 78 is still open: the 20-seed and 4×-budget check is running.** Its result goes below when it lands.

### HNL cards 84, 85 and 438–442 on ba190428

- **Binding:** all 7 cards bind with rc 0 and exactly 10 warnings each (6 YUKAWA, 4 CKMBLOCK). The command was `vibegraph parameters <proc card with only 'import model'> --param-card <gridpack card> --ufo-dir <bundled UFO>`, which binds the same way integrate does.
- **Before the fix:** 8a651703 refused all 7 cards at YUKAWA codes 1, 2, 3, 4, 11 and 13. Main 7b36722c refused them at CKMBLOCK.
- **Card 84's warnings,** without the card-path prefix:

```
ckmblock 1 sets 0.227591, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
ckmblock 2 sets 0.003508, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
ckmblock 3 sets 0.041539, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
ckmblock 4 sets 1.2, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
yukawa 1 sets 0.00504, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
yukawa 2 sets 0.00255, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
yukawa 3 sets 0.101, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
yukawa 4 sets 1.27, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
yukawa 11 sets 0.000511, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
yukawa 13 sets 0.10566, and the model has no parameter there; MadGraph reads only the model's own parameters, so the entry is ignored
```

- **σ is not measured.** All 7 proc cards are decay chains (`generate p p > w, (w > l hnl)`, plus `w j` and `w j j`). Enumerate refuses them in 0.1 s before binding the card. Card 84's refusal, verbatim:
  `decay chain 'p p > w, w > l hnl': in u d~ > e+ N1, a decay-chain diagram carries a Majorana fermion or a fermion-number-violating vertex; its lines are oriented per core and per decay, and orienting a line across the stitched propagator is not validated against MadGraph`
- This blocker is filed as `majorana-in-decay-chain-refused` (v0.3). In these cards the stitched line is the W, and N1 is external inside the decay.

### Per-seed rows

These are the validator's rows: σ, relative error, ESS, and σ/MG against the gpdb MG value. The build is 8a651703, run with the campaign argv except where marked 4×. The card means recomputed from these rows match the summaries above.
- The validator's copy is `pr11_seed_rows.txt` in its session scratchpad.
- Card 157 seeds 1–5 are in its table above.
- Card 78's per-seed ratios are under its table above.

**Card 157, seeds 6–20**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 6 | 0.0017659 | 4.32% | 526 | 0.9946 |
| 7 | 0.0018042 | 3.77% | 686 | 1.0162 |
| 8 | 0.0017521 | 4.04% | 602 | 0.9868 |
| 9 | 0.001802 | 4.09% | 587 | 1.0149 |
| 10 | 0.0016879 | 3.67% | 727 | 0.9506 |
| 11 | 0.0017953 | 5.35% | 345 | 1.0111 |
| 12 | 0.0017915 | 4.11% | 581 | 1.0090 |
| 13 | 0.0018086 | 4.77% | 433 | 1.0186 |
| 14 | 0.0016986 | 3.57% | 764 | 0.9567 |
| 15 | 0.001682 | 4.36% | 517 | 0.9474 |
| 16 | 0.0017908 | 3.91% | 641 | 1.0086 |
| 17 | 0.0018137 | 3.59% | 754 | 1.0215 |
| 18 | 0.0017975 | 3.78% | 683 | 1.0124 |
| 19 | 0.0018682 | 4.74% | 439 | 1.0522 |
| 20 | 0.0018244 | 5.01% | 393 | 1.0276 |

**Card 156**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 0.0028853 | 4.51% | 484 | 0.9702 |
| 2 | 0.0032235 | 4.13% | 575 | 1.0839 |
| 3 | 0.0030903 | 8.86% | 127 | 1.0391 |
| 4 | 0.0028929 | 4.76% | 434 | 0.9727 |
| 5 | 0.0029827 | 4.59% | 468 | 1.0029 |
| 6 | 0.0030209 | 4.63% | 459 | 1.0158 |
| 7 | 0.0029723 | 4.46% | 493 | 0.9994 |
| 8 | 0.0027808 | 3.77% | 689 | 0.9351 |
| 9 | 0.0028746 | 4.57% | 472 | 0.9666 |
| 10 | 0.0029959 | 4.33% | 523 | 1.0074 |
| 11 | 0.0028953 | 5.72% | 303 | 0.9735 |
| 12 | 0.0028179 | 3.60% | 751 | 0.9475 |
| 13 | 0.003003 | 4.73% | 440 | 1.0098 |
| 14 | 0.0027842 | 4.81% | 426 | 0.9362 |
| 15 | 0.0030837 | 5.23% | 361 | 1.0369 |
| 16 | 0.0032464 | 4.71% | 443 | 1.0916 |
| 17 | 0.0028191 | 2.96% | 1101 | 0.9479 |
| 18 | 0.0027733 | 3.17% | 967 | 0.9325 |
| 19 | 0.0030214 | 6.92% | 207 | 1.0159 |
| 20 | 0.0028712 | 4.31% | 526 | 0.9654 |

**Card 877**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 0.054655 | 3.10% | 1005 | 1.0320 |
| 2 | 0.051813 | 2.98% | 1085 | 0.9783 |
| 3 | 0.053379 | 4.49% | 484 | 1.0079 |
| 4 | 0.05147 | 3.09% | 1014 | 0.9719 |
| 5 | 0.054833 | 4.33% | 522 | 1.0354 |
| 6 | 0.05115 | 3.39% | 845 | 0.9658 |
| 7 | 0.051878 | 3.37% | 855 | 0.9796 |
| 8 | 0.052689 | 3.96% | 625 | 0.9949 |
| 9 | 0.05083 | 3.68% | 723 | 0.9598 |
| 10 | 0.057527 | 5.58% | 318 | 1.0862 |
| 11 | 0.052167 | 3.41% | 835 | 0.9850 |
| 12 | 0.052267 | 3.91% | 640 | 0.9869 |
| 13 | 0.054028 | 3.17% | 966 | 1.0202 |
| 14 | 0.053101 | 4.18% | 559 | 1.0027 |
| 15 | 0.05214 | 3.88% | 651 | 0.9845 |
| 16 | 0.054092 | 3.75% | 695 | 1.0214 |
| 17 | 0.053367 | 3.15% | 975 | 1.0077 |
| 18 | 0.055409 | 4.02% | 608 | 1.0462 |
| 19 | 0.051131 | 3.06% | 1034 | 0.9655 |
| 20 | 0.053681 | 3.99% | 615 | 1.0136 |

**Card 878**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 0.42541 | 3.27% | 907 | 0.9838 |
| 2 | 0.41055 | 3.22% | 933 | 0.9495 |
| 3 | 0.42966 | 2.72% | 1291 | 0.9937 |
| 4 | 0.41018 | 3.06% | 1030 | 0.9486 |
| 5 | 0.41267 | 2.99% | 1077 | 0.9544 |
| 6 | 0.42315 | 4.32% | 526 | 0.9786 |
| 7 | 0.41834 | 3.07% | 1023 | 0.9675 |
| 8 | 0.424 | 3.49% | 798 | 0.9806 |
| 9 | 0.45096 | 4.65% | 455 | 1.0429 |
| 10 | 0.41199 | 2.60% | 1411 | 0.9528 |
| 11 | 0.41623 | 2.87% | 1166 | 0.9626 |
| 12 | 0.46455 | 3.53% | 783 | 1.0744 |
| 13 | 0.4382 | 4.11% | 581 | 1.0134 |
| 14 | 0.42708 | 3.15% | 975 | 0.9877 |
| 15 | 0.40938 | 2.60% | 1421 | 0.9468 |
| 16 | 0.44198 | 3.12% | 996 | 1.0222 |
| 17 | 0.43209 | 2.98% | 1091 | 0.9993 |
| 18 | 0.44111 | 3.57% | 763 | 1.0201 |
| 19 | 0.44059 | 3.40% | 843 | 1.0189 |
| 20 | 0.42099 | 3.06% | 1033 | 0.9736 |

**Card 878 at 4× (`--neval 480000`)**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 0.43542 | 0.94% | 10285 | 1.0070 |
| 2 | 0.43407 | 0.89% | 11467 | 1.0039 |
| 3 | 0.42228 | 0.90% | 11348 | 0.9766 |
| 4 | 0.42897 | 1.11% | 7514 | 0.9921 |
| 5 | 0.42612 | 1.00% | 9221 | 0.9855 |

At 4× budget the ESS is about 10× the 1× value (about 1000 → about 10000), which is more than linear in budget. That fits the adapted grid concentrating better at the larger neval.

**Card 79**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 4.4503 | 1.13% | 7217 | 1.0044 |
| 2 | 4.4307 | 1.37% | 5017 | 0.9999 |
| 3 | 4.3991 | 0.98% | 9406 | 0.9928 |
| 4 | 4.449 | 1.87% | 2754 | 1.0041 |
| 5 | 4.374 | 0.92% | 10433 | 0.9871 |

**Card 418**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 0.22149 | 1.59% | 3816 | 1.0132 |
| 2 | 0.2146 | 0.87% | 11637 | 0.9817 |
| 3 | 0.21796 | 1.17% | 6766 | 0.9971 |
| 4 | 0.21539 | 0.99% | 9183 | 0.9853 |
| 5 | 0.2168 | 1.05% | 8310 | 0.9918 |

**Card 419**

| seed | σ (pb) | err | ESS | σ/MG |
|---|---|---|---|---|
| 1 | 0.39901 | 2.79% | 1270 | 0.9980 |
| 2 | 0.3967 | 1.62% | 3646 | 0.9922 |
| 3 | 0.40169 | 1.08% | 7934 | 1.0047 |
| 4 | 0.39068 | 0.98% | 9350 | 0.9772 |
| 5 | 0.39276 | 0.99% | 9158 | 0.9824 |

### Card 188

Skipped: it waits on MG 3.7.1 (R1).

### Local gates (manager)

- **187f7ba0** (the main merge), run with the reference data hidden:
  `/tmp/claude-0/lp/verify-hidden.sh <worktree> pr11m` → FMT_OK, CLIPPY_RC=0, CLIPPY_EV_RC=0, TEST_RC=0, 2374 passed, 0 failed.
- **ba190428,** as reported by the session that made the change, with the same gates: 2375 passed, 0 failed.
- **CI on ba190428** (run 45378200789): every job green, including the full banked job.
