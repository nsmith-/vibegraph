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
