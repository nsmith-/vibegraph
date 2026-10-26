---
type: Session Report
title: "V4 report: MadEvent's mirrored-ordering cut registered as a deviation"
description: "The defect catalogued and drafted upstream (fixed in 3.8.1, present in the 3.5.17 LTS, both read). The unequal-beam sigma gate runs a test-only emulation of MadEvent's cut, 717.575 +- 0.309 pb against the brief's 716.34 +- 0.36 (pull +2.60) and the seed record's 717.25 +- 0.145 (pull +0.95), with this crate's own 754.956 +- 0.352 reported. The samples row reports the columns the defect can move, gates the five it cannot, and gates the direct-ordering sub-sample on every column. Two hermetic tests; eleven mutations and two controls caught."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-22}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/S`, branch `sprintd-unequal-beams`, from `1395933`.
Nothing in `vegas.rs`, `budget.rs` or the phase-space maps was touched; no
backlog item was edited; nothing was pushed or posted. The user's decision
(2026-10-22) is outcome 3 of the
[defect policy](../../../validation/madgraph-defect-policy.md): a registered
deviation.

## What changed

**Defect catalogue.** A row and a section in
[madgraph-defects](../../../validation/madgraph-defects.md): the defect, its
lines at `b7687064`, the measured size (+5.26 % in σ at 6500 × 4000) and
"registered deviation (user, 2026-10-22)". The upstream draft is
`research/upstream/madgraph-mirrored-cut-unequal-beams/` (README, `repro.sh`,
`report-draft.md`), indexed in
[madgraph-upstream-report-drafts](../../../validation/madgraph-upstream-report-drafts.md).

**Upstream status, read from source.** 3.8.1 (`a5b17913`) is fixed: its
`PASSCUTS` (`cuts.f:24-98`) cuts each orientation in its own frame, the
mirrored one at `CM_RAP = ln(E₁/E₂) − CM_RAP` on the rotated momenta, and
`DSIG` zeroes an orientation whose `CUTS_ORIENT` failed. The drafts index
already names the change `e9576dbce`, "the mirror-cuts fix". 3.5.17
(`4492b3e3`) is affected by another path: it cuts the rotated momenta at
`CM_RAP = −CM_RAP` inside `DSIGPROC` (`:467`, `:472`), the same window. The
draft therefore asks for the fix on the LTS line. MadGraph is not installed
here, so `repro.sh` has not been run on any version. Its event parser was run
on a vibegraph stand-in, and it printed the expected table.

**The emulation** (`vibegraph-lib/src/proton.rs`, compiled only under
`cfg(any(test, feature = "extended-validation"))`):
- `ProtonIntegrand::emulate_madevent_mirrored_cut` (`:2143`), with the doc
  naming the defect and its lines;
- `cut_orderings_as_madevent` (`:2151`) cuts the mirrored term at
  `(x₁, x₂, q)` on the configuration MadEvent sampled it from: partons at
  `(x₂, x₁)`, outgoing `R q`, boosted at that configuration's `cm_rap`;
- `madevent_cut_luminosities` (`:2181`), called in `group_term` (`:2454`),
  zeroes the luminosity of an ordering whose cut failed.

The production path changes by one line, `shape`'s cut read as a
`let passes` (`:2103`). The emulation asserts that it is used on the
shared-scale sum and the cross section only (`:2271`, `:2711`). Nothing is
user-facing.

**The σ gate** (`vibegraph-lib/tests/validate_hadronic.rs`):
- `MirroredCut` (`:367`) is threaded through `adapted_integrand` (`:484`);
- `check_dy_run` picks `MadEvent` for the beams row (`:587`) and gates that
  σ;
- `DySetup::registered_deviation` (`:725`) measures this crate's own σ on the
  same five seeds and writes it, its pulls against MadGraph and against the
  emulation, and the defect, into the gated cell's `note`.

The note is not an `info` file because the collator requires every file of a
cell to carry the manifest's mode. `sigma_unequal_beams_vs_mg` (`:791`)
documents the deviation and the gate's blind spot.

**The samples gate** (`vibegraph-cli/tests/validate_samples_proton.rs`):
- `MirroredCutDeviation` and `MADEVENT_MIRRORED_CUT` (`:233`, `:245`),
  attached to the beams row only (`:1119`);
- `check_row` (`:741`), restructured around `judge` (`:948`),
  `print_comparison` (`:916`) and `seed_sample` (`:994`);
- the Drell-Yan spectrum, binned on the direct-ordering sub-sample for that
  row (`:1605`).

**Task 5.** `a_configuration_inside_the_cuts_only_in_the_partonic_frame_carries_no_weight`
(`proton.rs:6231`). At `x₁ = x₂` and 6500 × 4000, leptons at partonic
`y = ±2.4` pass the cuts in the CM frame and fail them in the lab frame (2.64).
`shape` must return 0. The same point between beams of equal energy at the same
`√s` must carry weight. A second test,
`the_emulated_madevent_mirrored_cut_reads_the_sampled_configuration` (`:6291`),
pins the emulation:
- at `x₁/x₂ = E₁/E₂` only the emulated mirrored term survives;
- at `x₁/x₂ = E₂/E₁` the emulation keeps the direct term alone;
- at equal beams the emulation is bit-identical over 5 000 production points.

**Manifest.** `pp_to_ll_beams_6500_4000`'s two notes state the deviation, its
size and how each cell treats it. The integrals note corrects "swapping the
beams leaves sigma unchanged": that holds for this crate (755.16 ± 0.36 at
4000 × 6500), not for MadGraph, predicted at about 792 pb there. The `<…>`
placeholders and `status = "planned"` are kept, and one placeholder each was
added for the reported own σ and for the reference counts.

## The emulated σ against the reference

The reference was temporarily added to a local `hadronic_sigma_reference.json`
and the file was restored after the run; it is not committed. Log
`…/scratchpad/ubS-V4-sigma-emulated.log`.

```
[beams_6500_4000] GATE vibegraph σ = 717.575 ± 0.309 pb (5 seeds, χ²/dof = 0.85) | MG σ = 716.340 ± 0.360 pb | pull = +2.60 | rel = +0.0017
[beams_6500_4000] INFO registered deviation (...): this crate's own sigma is 754.956 +- 0.352 pb (chi2/dof 0.51), rel +0.0539 (pull +76.7) against MadGraph's 716.340 +- 0.360 and rel +0.0521 (pull +79.8) against the emulation's 717.575 +- 0.309; info
test sigma_unequal_beams_vs_mg ... ok
```

The seed record's value, 717.25 ± 0.145, was put in the same file
temporarily (log `ubS-V4-sigma-emulated-seeded.log`):

```
[beams_6500_4000] GATE vibegraph σ = 717.575 ± 0.309 pb (5 seeds, χ²/dof = 0.85) | MG σ = 717.250 ± 0.145 pb | pull = +0.95 | rel = +0.0005
... rel +0.0526 (pull +99.0) against MadGraph's 717.250 +- 0.145 ...
```

The committed emulation reproduces V3's scratch emulation to the digit, and
this crate's own σ reproduces 754.956. **Against the brief's 716.34 ± 0.36 the
pull is +2.60**, inside `|pull| < 3` but with little room. That number is not
the seed record. `banked()` reads the seed record first, so once the M3 commits
its bank, 717.25 ± 0.145 is what gates: pull +0.95.

## Samples: what is reported and what is gated

MadGraph's banked events are not in this tree. The harness was therefore run
against vibegraph stand-ins placed temporarily at
`validation/madgraph/output/dy13_beams_6500_4000/`, which was removed
afterwards.

- **"emulated".** vibegraph generated 90 000 events at `etal = 3.0`. The
  stand-in keeps the direct events with both leptons at `|η| ≤ 2.5` and the
  mirrored ones with both leptons inside `[L − 2.5, L + 2.5]`, which is
  MadEvent's acceptance exactly. 67 794 events are kept, with σ 716.09 pb.
  This filter is an independent, generation-side emulation of the defect, and
  it agrees with the integrand one: 719.3 against 717.6 pb.
- **"plain".** 60 000 correct events.
- **"distorted".** The emulated stand-in with its direct events also cut at
  `|η| ≤ 2.0`.

Logs: `ubS-V4-samples-emu.log` and `ubS-V4-smut.log`. Running the spectrum test
needed a temporary `&DY_ROWS[2..]`, since the `pp_to_ll` runs are absent; the
edit was restored.

**Read off the reference first. These checks are gated, and are the
diagnosis's falsifier:**
- no direct event (quark on `+z`) has a lepton outside `|η| ≤ etal`;
- every mirrored event lies inside `ln(E₁/E₂) ± etal`;
- at least one mirrored event lies outside the cut.

On the emulated stand-in: 0 of 33 969 direct events, and 2827 of 33 825
mirrored events (8.4 %), all inside [−2.014, 2.986], with the largest `|η|`
2.9855. None of ours (3 × 20 000) leaves the cut.

**Reported (info) on the full sample**, because the admitted region can move
them:
- the leptons' `y` and `cos`, and `y(ll)`;
- `beam1/2 E` and `pz`;
- `pt(l±)`, `m(e+,e-)` and `cs_cos(ll)`;
- `flavour`, `ICOLUP` and `SPINUP`.

On the emulated stand-in the ones below the floor were `y(l±)`, `cos(l±)`,
`y(ll)`, beam `E` and `pz`, `ICOLUP` and `flavour` on every seed. These are the
brief's list, with `cos` and `y(ll)` added. `pt`, `m`, `cs_cos` and `SPINUP`
stayed above the floor here, but I have no argument that the defect cannot move
them, so they are reported rather than gated. This is a deliberate widening of
the brief's list.

**Gated on the full sample, with the argument that the defect cannot move
them:**
- `phi(l)/pi`: everything, the displaced window included, is invariant under
  rotations about the beam axis.
- `beam1 m` and `beam2 m`: constants of the record.
- `SCALUP` and `AQCDUP`: fixed by the card.

**Gated on every column: the direct-ordering sub-sample** (variant
`direct_ordering`). MadEvent cuts that ordering on its own lab frame, so the
defect cannot reach it on either side. This is a finer oracle than the brief
asked for. On the emulated stand-in the minimum KS p is 0.078 (`phi(l+)/pi`),
the minimum χ² p is 0.24 (`SPINUP`) and `ICOLUP` has a single category. The
absolute `dσ/dm_ll` spectrum is binned on this sub-sample, every bin within
1.65 combined errors. Without the restriction it fails (Ms2), since the whole
spectrum carries the 5 %.

**Generation.** Emulating the defect in generation would need a hook in the
shipped binary, which the brief rules out as user-facing. The sample is
compared unemulated.

## Mutations and controls

Drivers `…/scratchpad/ubS-V4-mut.sh` and `ubS-V4-smut.sh`. Each mutant was
restored and checked with `cmp`, and the unmutated tree passes at the end of
each driver.

| id | mutation | caught by |
|---|---|---|
| M1 | `shape` cuts on the partonic CM | task-5 test and emulation test FAIL; V3's exchange test passes (its documented blind spot) |
| M2 | `shape` applies no cut | task-5 test and emulation test FAIL |
| M3 | emulated mirrored cut = the lab cut | emulation test FAIL; σ gate FAIL, 754.956 against 716.34, pull +76.67 |
| M4 | emulation at `r = E₁/E₂` | emulation test FAIL; σ gate FAIL, 754.956: with a symmetric `|η|` cut this is the lab cut again |
| M5 | failed ordering's luminosity kept | emulation test FAIL; σ gate FAIL, 853.658, pull +215 |
| M6 | σ gate run unemulated | σ gate FAIL, pull +76.67 |
| C1 | correct reference (no defect) | samples FAIL: "mirrored events leave the window ln(E1/E2) +- etal" (5222 of 31 401) |
| C2 | reference's direct ordering distorted | samples FAIL on `direct_ordering` (y, pt, beam E/pz, cs_cos) |
| Ms1 | every full-sample column gated | samples FAIL on the reported columns: the info cells are not vacuous |
| Ms2 | sub-sample unfiltered | samples FAIL on `direct_ordering`; spectrum FAIL |
| Ms3 | sub-sample = mirrored ordering | samples FAIL; spectrum FAIL (pull +9.16 at 90-92 GeV) |
| Ms4 | our-side cut check tightened to `|η| > 2.0` | samples FAIL, 5340 of 20 000 events flagged |

## For the M3: committing the bank and filling the placeholders

1. Commit the bank as the V2 banking request says: the
   `output/dy13_beams_6500_4000` run (with `Events/run_beams_6500_4000/` and
   `Cards/run_card.dat`), its seed record under `output/seeded_sigma/`, and
   the bundle re-cut. Do **not** add a `beams_6500_4000` entry to
   `hadronic_sigma_reference.json`: `banked()` reads the seed record first,
   and the 716.34 ± 0.36 single run would gate at pull +2.60 rather than
   +0.95.
2. Run `cargo test -p vibegraph-lib --profile release-debug --features
   extended-validation --test validate_hadronic -- --exact
   sigma_unequal_beams_vs_mg --nocapture`. Copy the `GATE` line's pull, rel
   and χ²/dof and the seed record into the first integrals placeholder, and
   the `INFO` line's own σ, rel and pulls into the second. Expected values:
   717.575 ± 0.309, pull +0.95, rel +0.05 %, χ²/dof 0.85; own σ 754.956 ±
   0.352, rel +5.26 %.
3. Run `cargo test -p vibegraph --profile release-debug --features
   extended-validation --test validate_samples_proton --
   generated_unequal_beam_drell_yan_events_agree_with_madgraphs_banked_ones
   the_drell_yan_mass_spectrum_is_binned_against_madgraph --nocapture`.
   - Copy the "MadEvent mirrored-ordering cut on MadGraph's sample" line into
     the samples counts placeholder.
   - Copy the minimum KS and χ² p-values of the gated columns into the last
     placeholder, from `target/validation-report/samples/pp_to_ll_beams_6500_4000.json`
     and `…__direct_ordering.json` (`min_ks_p`, `min_chi2_p`, `min_beam_ks_p`).
   - The full row's `min_ks_p` is over every column, the reported ones
     included, so for the full sample quote the gated ones from `per_seed`
     (`phi`, `beam* m`, `SCALUP`, `AQCDUP`).
   - Prediction: a few to about ten per cent of MadGraph's mirrored events
     outside the cut, none above 2.986, and none among the direct events.
     About 8.4 % on the stand-in.
   - A failure of "the defect is not the one registered" or "the direct
     ordering is not the one MadEvent cuts on its own lab frame" falsifies the
     diagnosis. Do not demote it; report it.
4. Run the known-wrong energy-sum control the note asks for, then flip
   `status = "planned"` off.

## Gate

The tails are in the session's final message. The commands:
- `cargo fmt --all --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`, with and without
  `--features vibegraph/extended-validation`;
- `cargo test --workspace`;
- `python3 scripts/kb.py lint`.

`pixi run --skip-deps validate` was not run: there is no pixi here and no
`validation/madgraph/output`, and the change touches no amplitude, eval or
coverage-list code.
