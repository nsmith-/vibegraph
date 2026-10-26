---
type: Session Report
title: "V2 report: the unequal-beam Drell-Yan row prepared for banking"
description: "The 6500 x 4000 card, the script entries, the planned manifest row, validate_hadronic reading the card's beams, the not-yet-banked paths in both gates and the collator, and the banking request. MadGraph was not run; the row's cells are planned."
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-21}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-21}
---
Worktree `/home/user/wt/S`, branch `sprintd-unequal-beams`, from `08833b3`.
Nothing in `vegas.rs`, `budget.rs` or the phase-space maps was touched; no
backlog item was edited; MadGraph was not run. The environment has no PDF
sets, no `validation/madgraph/output` and no `research/refs/mg5amcnlo`
content, so the banked layer was not run; the new banked-layer tests were
compiled with `--features extended-validation` and the not-yet-banked paths
were run.

## Corrections to the brief

1. **The collator needed a change the brief did not list.**
   `validation-report` fails a banked-tier cell that no gate wrote, unless the
   row is `bundled = false`. A `planned` row had no such exemption (`ProcessStatus::Planned`
   was parsed and listed, but never consulted when resolving a cell), so
   `pixi run validate` would have exited nonzero on the two empty cells. Fix:
   `validation-report/src/main.rs:358-370`, a planned row with no measurement
   renders `⏳ planned: no reference run yet` with no problem; a measurement, once
   banked, is rendered from as for any row.
2. **The card has three differing lines, not two.** The brief says `ebeam2` plus
   "the header line naming the beams". The card names the beams in two header
   lines (line 2, `at 13 TeV`, and line 11, `# Collider: 13 TeV proton-proton`);
   changing only one leaves a false claim in the other. Both are changed; the
   test requires exactly two differing `#` lines and exactly one differing
   value line.
3. **V1 §4's manifest row is verbatim, but the `samples` row writer cannot use
   `variant`.** The report joins a cell to its manifest row by key, so the new
   gates write under the row key `pp_to_ll_beams_6500_4000`, with no variant
   (V1 §4 said variant `beams_6500_4000`; that would have named the file
   `pp_to_ll_beams_6500_4000__beams_6500_4000.json` for no reason). The run
   name stays `beams_6500_4000` / `dy13_beams_6500_4000`.
4. The brief's gate list omits the feature-gated tests: `cargo clippy
   --workspace --all-targets` does not compile `validate_hadronic` and
   `validate_samples_proton` (`required-features`). I also ran
   `cargo clippy --workspace --tests --all-features -- -D warnings`.

## Changes

| file:line | change |
|---|---|
| `validation/madgraph/dy_beams_6500_4000_run_card.dat:2,11,16` | new card: byte copy of `dy13_default_run_card.dat`, `4000.0 = ebeam2` and the two header lines |
| `validation/madgraph/gen_hadronic_sigma.sh:4-5,119,122,139-143` | third `run_one beams_6500_4000 dy_beams_6500_4000_run_card.dat`, passed to the JSON writer, key `beams_6500_4000`; header comment |
| `validation/madgraph/bank_seeded_sigma.sh:16-19,172,192,413` | `dy13_beams_6500_4000` in `@hadronic`, the `dy13` family map and the card map; usage comment |
| `validation/manifest.toml:521-534` (key at 522) | V1 §4's row verbatim, `status = "planned"`, placed after `pp_to_ll` |
| `vibegraph-lib/tests/common/manifest.rs:46,106-117` | reads `status`; `planned_rows()` |
| `vibegraph-lib/tests/validate_hadronic.rs:64` (const `BEAMS` removed); sites `449, 464, 781, 1190, 1299, 2754, 2938, 4077, 4423` | every `ProtonIntegrand` built from a row's card takes `ProtonBeams::from_run_card(rc)` |
| `vibegraph-lib/tests/validate_hadronic.rs:533-555` | `check_dy_run` reports `NOT YET BANKED` and returns when there is no reference and the row is planned; panics when there is no reference and the row is not planned; row key `pp_to_ll_beams_6500_4000` for the new run |
| `vibegraph-lib/tests/validate_hadronic.rs:660-675` | `sigma_unequal_beams_vs_mg` |
| `vibegraph-lib/src/coupling/scales.rs:826-832` | `collider_energy` doc: the mass cancels only at equal beams |
| `vibegraph-cli/tests/validate_samples_proton.rs:704-716` | third `DY_ROWS` entry (run `dy13_beams_6500_4000`, events `run_beams_6500_4000`, key `pp_to_ll_beams_6500_4000`) |
| `vibegraph-cli/tests/validate_samples_proton.rs:738` | `generated_unequal_beam_drell_yan_events_agree_with_madgraphs_banked_ones` |
| `vibegraph-cli/tests/validate_samples_proton.rs:1321` | `not_yet_banked`, used by the new test, the binned-spectrum loop and the headroom probe |
| `vibegraph-lib/tests/scales_run_cards.rs` | new card in the fixed-scale test (3 cards); `the_unequal_beam_card_differs_from_the_default_card_in_the_beams_alone`; `run_card_beams_reproduce_the_constant_the_equal_beam_gates_used` |
| `validation-report/src/main.rs:365,500` | planned row with no cell waits; test `a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing` |

The equal-beam oracles at `SQRT_S_HAD` (`dilepton_lab`, `tau_min`, the `shat`
of the weight-tail probe) are untouched.

## Why the current cards stay bit-identical

The old constant was `ProtonBeams::new(13000/2, 13000/2)`. The card path is
`ProtonBeams::new(rc.ebeam1, rc.ebeam2)`. They are the same two doubles when the
card says `6500.0`.

- Committed cards, measured: `run_card_beams_reproduce_the_constant_the_equal_beam_gates_used`
  compares `energies()` and `sqrt_s()` of both `dy13` cards against the old
  constant by `to_bits`. It passes (below), and fails when the card is changed to
  an unequal one (M4) or `from_run_card` reads one beam twice (M5).
- The banked output cards of the other rows (`pp_to_llj*`, `pp_to_bb*`,
  `pp_to_jj`, scalefact2) are not in this checkout. V1's sweep of every
  committed input showed every `ebeam1`/`ebeam2` pair equal, and scripts that set
  no beam energy inherit MadGraph's 6500 / 6500. That is an argument, not a
  measurement on those cards here: CI's banked job is the measurement, and a moved
  number on any proton row there would be this change's bug.
- Blind spot: neither covers a banked card with a non-6500 equal energy; there is
  none.

## Gate

```
$ cargo fmt --all --check
fmt exit=0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 18.82s
clippy exit=0
$ cargo clippy --workspace --tests --all-features -- -D warnings
    Finished `dev` profile [optimized] target(s) in 6.04s
$ cargo test --workspace
test exit=0
42 `test result` lines; summed: passed 1607, failed 0, ignored 18
$ python3 scripts/kb.py lint
879 concepts, 292 backlog items; 0 errors, 4 warnings
```

The four warnings are the `state: blocked` items without `blocked_by` that V1
reported. Logs: `…/scratchpad/ubS-V2-gate.log`, `-gatesum.log`.

Not-yet-banked paths, run with `--features extended-validation`:

```
$ cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic sigma_unequal_beams -- --nocapture
[beams_6500_4000] NOT YET BANKED (the manifest marks row pp_to_ll_beams_6500_4000 planned and neither a seed record nor hadronic_sigma_reference.json carries a reference, so no cell is written)
test sigma_unequal_beams_vs_mg ... ok
$ cargo test -p vibegraph --features extended-validation --test validate_samples_proton unequal_beam -- --nocapture
[dy13_beams_6500_4000] NOT YET BANKED (the manifest marks row pp_to_ll_beams_6500_4000 planned and this checkout has no run for it, so no cell is written)
test generated_unequal_beam_drell_yan_events_agree_with_madgraphs_banked_ones ... ok
$ cargo test -p vibegraph-validation-report
test tests::a_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing ... ok
test result: ok. 12 passed; 0 failed
```

## Mutations

Driver `…/scratchpad/ubS-V2-mut.sh`, log `…/scratchpad/ubS-V2-mut.log`; each
mutation applied, the named test run, the file restored (`restored` printed).

| mutation | test | result |
|---|---|---|
| M1 card `ebeam2 = 4001` | card-diff, beams | both **FAILED** (`scales_run_cards.rs:172`, `:207`) |
| M2 card `nevents` 100000 | card-diff | **FAILED** (`:172`) |
| M3 card header lines restored to `13 TeV` (a 3rd/4th differing line removed) | card-diff | **FAILED** (`:171`, the header count) |
| M4 card `ebeam2 = 6500` | card-diff, beams | both **FAILED** |
| M5 `from_run_card` reads `ebeam1` twice | beams | **FAILED** (`:207`) |
| M6 manifest row not `planned` | `sigma_unequal_beams_vs_mg` | **FAILED** (`validate_hadronic.rs:541`, no reference and not planned) |
| M7 manifest row not `planned` | samples test | **FAILED** (`validate_samples_proton.rs:109`, the missing banked events) |
| M8 samples row key `pp_to_ll` | samples test | **FAILED** (same line: the declaration is by row key) |
| M9 `DY_BEAMS_ROW` misspelt | `sigma_unequal_beams_vs_mg` | **FAILED** (`:541`) |
| M10 collator planned branch disabled | collator test | **FAILED** (`main.rs:517`) |

Not mutated, because they need the banked layer: the nine `from_run_card(rc)`
sites in `validate_hadronic.rs` (a site left at a constant cannot be seen by
any hermetic test), and the `not_yet_banked` call in the binned-spectrum loop,
which in this checkout is behind the first two rows' own missing runs.
Both are compiled (`--all-features` clippy) and read by CI's banked job; the
unequal-beam σ test is the one that would integrate at the wrong energy if a site
were missed, loudly against a 10.2 TeV reference.

## Banking request (for the M3)

Host: macOS, pinned MadGraph 3.7.1 (`mg5_pinned.sh`), the `madgraph` pixi
environment, PDF set 247000 under `validation/pdf` (`pixi run fetch-pdf`).

```
pixi run -e madgraph generate-hadronic-sigma
pixi run -e madgraph bank-seeded-sigma dy13_beams_6500_4000
```

1. `generate-hadronic-sigma` reruns nothing it already has (`run_one` skips a
   process directory with `bin/generate_events`) and generates
   `output/dy13_beams_6500_4000` from the card; it banks the run's events in
   `output/dy13_beams_6500_4000/Events/run_beams_6500_4000/` and rewrites
   `hadronic_sigma_reference.json` with a third key `beams_6500_4000`
   (`run_card = dy_beams_6500_4000_run_card.dat`, `sigma_pb`, `sigma_err_pb`).
   Check `default` and `mmll_60_120` in the JSON are unchanged to the printed
   digit (their directories exist and are read back, not re-run).
2. `bank-seeded-sigma dy13_beams_6500_4000` banks 5 seeds at the budget and 2
   at 4x (`hard_survey = 2`) under `output/seeded_sigma/dy13_beams_6500_4000/`,
   and prints `SCREEN`, `LADDER`, `SURVEY`, `SCATTER`. Expected: the generator line
   `generate p p > e+ e- (gen_hadronic_sigma.sh); run card
   dy_beams_6500_4000_run_card.dat`; `SURVEY` not INERT; `SCREEN` and `SCATTER` not
   FLAGGED. A FLAGGED line means the budget ladder first, as for any row.
3. The σ is the measurement; no value is predicted here.
4. Then, in the PR that lands the bundle: run
   `cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic sigma_unequal_beams -- --nocapture`
   and
   `cargo test -p vibegraph --features extended-validation --test validate_samples_proton unequal_beam -- --nocapture`
   (the line `NOT YET BANKED` is replaced by `GATE vibegraph σ = …` and the
   samples table); record pull, rel, chi2/dof, the seed record and the minimum KS /
   chi-squared p-values in the two `<…>` placeholders of the manifest cells; run
   the known-wrong control once (the energy-sum frames) and record that it fails;
   remove `status = "planned"` from the row. If the bundle is not re-cut in the
   same PR, add `bundled = false` as well: it is the declaration the existing
   gates read as awaiting-bundle (`unbundled_rows`).

## Found

1. **`validate_hadronic` and the samples gate have two different escape
   hatches for an absent run** (`bundled = false` → `AWAITING BUNDLE`, and now
   `planned` → `NOT YET BANKED`), implemented in four places
   (`dyn_run_present`, `run_present`, `not_yet_banked`, `check_dy_run`) with the
   collator branches. Close: one helper in `common/manifest.rs` that returns the
   row's declared state, used by all.
2. **`validation/madgraph/time_stages.py:222` keeps its own row-to-card map** for
   the `dy13` rows and does not know `dy13_beams_6500_4000`. Close: add the entry
   when the row is banked, or read the card from the manifest.
3. **The binned-spectrum test (`the_drell_yan_mass_spectrum_is_binned_against_madgraph`)
   iterates over all of `DY_ROWS`, so the unequal-beam row will be judged on the
   `m_ll` spectrum too** with the 20 GeV threshold the pt cut implies. That is
   the same threshold at 10.2 TeV, but the bins' `SPECTRUM_MIN_SHARE` judgement was
   calibrated at 13 TeV. Close: read the first banked reading and note it.
4. **The `lpp = 0` fixed-beam guard** (V1 Found 5) is still open.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`, one `Assisted-by` line, not pushed:

- (this commit) test(beams): prepare the 6500 x 4000 Drell-Yan banking row
