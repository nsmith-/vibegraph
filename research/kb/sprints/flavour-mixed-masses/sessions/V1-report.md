---
type: Session Report
title: "V1 report: the mixed-mass Drell-Yan row prepared for banking"
description: "The seeded-bank entry, the planned manifest row, the collator waiting on a planned row, the per-directory gate in validate_hadronic and the banking request for p p > l+ l- with l = e, ta. MadGraph was not run; the row's cells are planned."
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-22}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/L`, branch `sprintd-flavour-mixed-masses`, from
`0147349`. Nothing in `vegas.rs`, `budget.rs` or `phasespace/` was touched, no
backlog item was edited, and MadGraph was not run. This checkout has no
`validation/madgraph/output` and no `research/refs/mg5amcnlo` content, so the
reference-dependent half of the new test could not meet a real record (see
the test section for what was run in its place).

## 1. What changed

| file | lines | change |
|---|---|---|
| `validation/madgraph/bank_seeded_sigma.sh` | 19-21 | header text for the new kind |
| | 174 | `dy13_etau_default` in `@hadronic` |
| | 195 | `row_kind`: `dy13_etau_default) echo dy13etau ;;` |
| | 225 | `write_generation_script`: the `dy13etau` generation script (two `define` lines, `generate p p > l+ l-`, `output`) |
| | 420-423 | card map: the `dy13etau` case, run card `dy13_default_run_card.dat`, generator string as F1 wrote it |
| `validation/manifest.toml` | 522 | row `pp_to_ll_etau`, `status = "planned"`, text verbatim from F1 section 4 |
| `validation-report/src/main.rs` | 365 | `resolve_cell`: a `planned` row with no gate cell renders as `⏳ planned: no reference run yet` instead of a missing-cell problem |
| | 500 | test `a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing` |
| `vibegraph-lib/tests/common/manifest.rs` | 46, 110 | `status` read from the manifest; `planned_rows()` |
| `vibegraph-lib/tests/validate_hadronic.rs` | 631-882 | `sigma_mixed_outgoing_masses_vs_mg` and its helpers |

**Deviation from the brief: the collator change was not on this branch.**
`ProcessStatus::Planned` exists in `validation-report/src/manifest.rs`
(the manifest type and the rendered list of planned rows), but `resolve_cell`
did not read it, and `tests/common/manifest.rs` had no `planned_rows`. I
ported #67's V2 change (commit `1e142c3`) for those two files unchanged except
that the collator test names `pp_to_ll_etau`. `git apply --3way` applied both
hunks cleanly.

**Correction to the brief and to F1: the banking command.** The task reads the
row from the environment. `bank_seeded_sigma.sh`'s own usage text says `ROWS`
is required, and a positional argument prints that usage and exits 2:

```
$ bash validation/madgraph/bank_seeded_sigma.sh dy13_etau_default; echo "exit $?"
...usage text...
exit 2
```

F1's "Commands" block (`./bank_seeded_sigma.sh dy13_etau_default`) is the
positional form and does not work. #67's V2 report has the same defect
(`pixi run -e madgraph bank-seeded-sigma dy13_beams_6500_4000`); see Found.

**Dispatch evidence without MadGraph.** Running the script with the right form
reached the template step and wrote the generation script, then stopped on the
absent submodule. The file it wrote:

```
$ ROWS=dy13_etau_default bash validation/madgraph/bank_seeded_sigma.sh
>>> [dy13_etau_default] generating the template (dy13etau)
!!! no pinned MadGraph at .../research/refs/mg5amcnlo (git submodule update --init)
$ cat validation/madgraph/work/seeded_sigma/dy13_etau_default/generate.mg5
define l+ = e+ ta+
define l- = e- ta-
generate p p > l+ l-
output .../template -nojpeg
```

The work directory was removed afterwards.

## 2. The test

`sigma_mixed_outgoing_masses_vs_mg` (in the `extended-validation` test target)
does, in order:

1. Runs the card through `multiplicity_groups` and asserts two parts, each
   group of a part carrying that part's masses, `(0, 0)` then `(mta, mta)`.
   This needs no reference and runs everywhere.
2. Reads the seed record `output/seeded_sigma/dy13_etau_default`. If absent
   and the manifest row is `planned`, it prints `NOT YET BANKED` and passes.
   If absent and the row is not planned it fails through
   `validation::require`.
3. Per seed at the budget, sums `by_channel` `value_pb` per `P` directory and
   requires exactly two directories, one `*_epem` and one `*_taptam`
   (`lepton_part_of`; any other final-state token panics and names the
   directory). Per-directory values are combined under the seed policy
   (`combine`). The record's `reference()` is the total, read from
   `results.dat`.
4. Integrates `MultiplicitySum` over the five `DY_SEEDS` at `DY_NEVAL` x
   `DY_NITER` as the CLI does (`multiplicity_maps`, on-shell vetoes, run-card
   scales, `ByAlpha`), takes `part_results`, and compares e part to the `e`
   directory, tau part to the `tau` directory, and the total to `results.dat`
   on |pull| < 3, |rel| < 0.005 and the seeds' chi2/dof < 4. The three
   comparisons are written as `info` cells (`pp_to_ll_etau__e_part`,
   `__ta_part`, `__total`), matching the manifest's `mode = "info"`. They do
   not assert; the gate flip is the landing PR's.

Per-seed it also prints the directories' sum next to `results.dat`. That line
is a diagnostic and not an assertion: whether the channel values sum to the
combined total at the precision printed is not something this checkout could
show, so the first banked run reads it.

Blind to: an error both sides share inside a part (the single-mass Drell-Yan
row covers the electron one), and a mis-sampled region of small measure
(seeds and chi2/dof guard that, not the pull). The per-directory comparison is
what sees one part drifting while the total stays right.

**Run locally.** Without a record:

```
$ cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic sigma_mixed -- --nocapture
[pp_to_ll_etau] NOT YET BANKED (the manifest marks this row planned and this checkout has no seed record for dy13_etau_default; the split into two parts is checked, no cell is written)
test sigma_mixed_outgoing_masses_vs_mg ... ok
```

The measurement body was exercised once against a **synthetic** record
(five seeds, two `P` directories named `P1_qq_epem` and `P2_qq_taptam`, values
set to F1's own numbers, deleted afterwards), only to run the code path end to
end. Its pulls (+1.85, -0.01, +1.33) say nothing about MadGraph: the reference
was made up. It does show that the part and total comparison, the per-directory
sums and the cell writing run: 24.9 s for five seeds in this profile.

## 3. Mutation evidence

Each mutation applied, the named test run, the file restored.

| mutation | test | result |
|---|---|---|
| row not `planned`, no record | `sigma_mixed_outgoing_masses_vs_mg` | **FAILED**: `validation.rs:32` "needs a banked seed record (dy13_etau_default)" |
| synthetic record, both directories named `*_epem` | same | **FAILED**: `validate_hadronic.rs:755` "two directories integrate the same lepton mass" |
| collator `planned` branch disabled (`if false && ...`) | `a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing` | **FAILED**: `main.rs:517`, left `("❌", 1)` right `("⏳", 0)` |
| row not `planned` | the same collator test, its second half | the ordinary row reads `("❌", 1)`: the missing cell is reported |

Restored, the three pass again. Not mutated: the card's masses assertion (a
massless-tau model would trip `mta > 1.0` first), the `lepton_part_of` match
arms, and the `MIXED_MASS_ROW` constant spelling (the same failure as the
first row would show).

## 4. Gate

Run in the worktree with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`, log `fmL-V1-gate.log`:

```
### fmt
exit 0
### clippy
    Finished `dev` profile [optimized] target(s) in 14.58s
### clippy ext   (--features vibegraph/extended-validation)
    Finished `dev` profile [optimized] target(s) in 28.99s
### test   (cargo test --workspace)   no FAILED, no panicked
### report (cargo test -p vibegraph-validation-report)
test result: ok. 45 passed; 0 failed
test result: ok. 20 passed; 0 failed
test result: ok. 12 passed; 0 failed
### kb   (python3 scripts/kb.py lint)
914 concepts, 296 backlog items; 0 errors, 4 warnings
```

The four warnings are existing `state: blocked` items without `blocked_by`.
The banked layer (`pixi run validate`) was not run: no reference bundle in
this checkout.

## Banking request (for the M3)

Host: one with the pinned MadGraph 3.7.1 submodule initialised
(`research/refs/mg5amcnlo`, b7687064), the `madgraph` pixi environment, and
the PDF set under `validation/pdf` (`pixi run fetch-pdf`). The task takes the
row from the environment; a positional argument exits 2.

```
ROWS=dy13_etau_default pixi run -e madgraph bank-seeded-sigma
ls validation/madgraph/output/seeded_sigma/dy13_etau_default/s*/
```

1. The run banks 5 seeds at the card's budget (20261101-5) and 2 at 4x
   (20261111-2), `hard_survey = 2`, under
   `output/seeded_sigma/dy13_etau_default/`, and prints `SCREEN`, `LADDER`,
   `SURVEY`, `SCATTER`. Expected: the generator line
   `define l+ = e+ ta+; define l- = e- ta-; generate p p > l+ l- ; run card
   dy13_default_run_card.dat`; `SURVEY` not INERT; `SCREEN` and `SCATTER` not
   FLAGGED. A FLAGGED line means the budget ladder first, as for any row.
2. Each seed's kept files must show **two `P` directories**, one `*_epem`
   (`qq~ > e+e-`) and one `*_taptam` (`qq~ > ta+ta-`). One directory
   contradicts F1 section 1 (`find_process_classes` keys on `MTA` against
   `ZERO`) and stops the banking: report it, do not bank it. If the two
   directories are named otherwise than ending `epem` / `taptam`,
   `lepton_part_of` panics naming them; change its match arms to the names
   seen, which is a test-side edit.
3. Expected values (F1's pre-registration, vibegraph at 120000 x 12, five
   seeds; MadGraph's are the measurement):
   - the `e` directory's channel results summed: about 933 pb (vibegraph
     934.541 +- 0.616; the existing `dy13_default` record is 933.23 +- 0.48);
   - the `ta` directory: about 923.8 pb (vibegraph 923.778 +- 0.610);
   - `SubProcesses/results.dat` field 1: about 1858 pb (vibegraph 1858.318 +-
     0.867).
   Per-directory sums matter because the `ta` part is the new physics; a
   total alone hides two offsetting errors.
4. Then run the test:
   `cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic sigma_mixed -- --nocapture`
   (use `--profile release-debug` if the default test profile is slow here).
   The `NOT YET BANKED` line is replaced by one `directories {...} sum ... |
   results.dat ...` line per seed, the five `[pp_to_ll_etau seed ...]` lines,
   and three result lines: `e_part`, `ta_part`, `total`, each with pull, rel
   and chi2/dof. The directories' sum should equal `results.dat` for every
   seed; if it does not, that is a finding about what the channel values
   mean, not about the physics, and the comparison should then read the
   directory's own `results.dat` instead.
5. **Commit:** `validation/madgraph/output/seeded_sigma/dy13_etau_default/`
   where the bundle carries banked runs (the same artifacts path the row
   lists), and in the landing PR the manifest edit below. Do not commit
   `work/`.
6. **Fill the manifest cell** (`pp_to_ll_etau`, `integrals`): replace the
   pre-registered text with the measured readings, one per comparison: the
   five-seed mean +- error, pull, rel and chi2/dof for the e part, the ta
   part and the total, the seed record (seeds, `SCREEN`, `SCATTER`), and the
   date. Keep the known-wrong control sentence, and run the control once:
   integrate the ta part on the massless e channels (one part, masses 0) and
   record that it misses the ta directory by about -1.15% (10.7 pb). Flip
   `mode` from `info` to `gate` only when the readings agree under
   |pull| < 3, |rel| < 0.005, chi2/dof < 4; then remove `status = "planned"`.
   If the bundle is not re-cut in the same PR, add `bundled = false`: it is
   the declaration the existing gates read as awaiting-bundle. The `samples`
   cell stays `uncovered`.

## Found

1. **The unequal-beams banking request (#67, `V2-report.md`) uses the
   positional form** `pixi run -e madgraph bank-seeded-sigma
   dy13_beams_6500_4000`, which exits 2. The same defect is in F1's request.
   Close: write `ROWS=<row> pixi run -e madgraph bank-seeded-sigma` in the
   unequal-beams record before its M3 runs, or make the script accept a
   positional row (a script change, not made here).
2. **`validation/madgraph/time_stages.py` keeps its own row-to-card map for
   the `dy13` rows** (V2 found the same) and does not know
   `dy13_etau_default`. Close: read the card from the manifest, or add the
   entry when the row is banked.
3. **The `P` directory names are assumed, not observed.** `lepton_part_of`
   reads the final-state token of `P<N>_<initial>_<final>`. MadGraph 3.7.1's
   name for a grouped process is one this checkout could not generate. Close:
   the first banked run shows the names; fix the match arms if they differ.
4. **Planned-row handling is now in four places** (`planned_rows()`, the
   collator branch, and the two `NOT YET BANKED` paths of #67's tests once it
   lands) beside `bundled = false`'s `unbundled_rows()`. This is #67's Found
   item 1, now with a fifth reader. Close: one helper in
   `common/manifest.rs` returning a row's declared state.
5. **The gate compares the sum of `by_channel` `value_pb` to the part.** The
   record stores each channel's `results.dat` value; a P directory's own
   `results.dat` is not in the record. If the sums differ from it at the
   percent level the gate reads the wrong quantity. Close: the first banked
   run's diagnostic line answers it (step 4 above).

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`, one `Assisted-by` line, not
pushed:

- (this commit) test(validation): prepare the mixed-mass Drell-Yan banking row
