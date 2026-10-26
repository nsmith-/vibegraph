---
type: Session Report
title: "F2 report: the PDF from the run card (H2, CI fix, CLI half, migration)"
description: "The banked failure on lhaid 230000 root-caused to a test helper reading pdlabel/lhaid instead of pdf_choice, with pp_to_ll and pp_to_ll_qcd0 found to be banked nn23lo1 proton runs; PDFSUP fixed for MadGraph-written fixed-energy cards; --pdf-set/--pdf-dir removed from integrate; <vibegraphInputs> written by generate; a rebanking request."
status: draft
tags: [pdf, lhapdf, run-card, ci, banked, lhef, network-consent, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
---
Worktree `/home/user/wt/H`, branch `sprintd-pdf-from-run-card`, from `cabe19e`
(brief) and `8a646f0` (the user's decisions). Every cargo command ran with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
The MadGraph reference bundle was fetched into this worktree's gitignored
`validation/madgraph/output` (cut 10, sha256 `dcc91e15…83e4`, equal to the
manifest pin), through `gh api …/releases/assets/647731618` and
`validation/fetch_refdata.sh` with `VIBEGRAPH_REFDATA_SOURCE` set and a
python-zstandard shim for the missing `zstd`; neither the bundle nor the shim
is committed. So, unlike F1, every banked suite below was measured here.

## 1. The CI failure on #60

### Root cause

Two causes, the second hidden behind the first.

**Cause 1 (what CI showed).** MadGraph 3.7.1 writes a fixed-energy run card
with the per-beam PDF template and no `pdlabel` line. The banked `ee_to_ee`
card, like every `ee_*` row and `var_sde1`:

```
  0	= lpp1 ! beam 1 type
  0	= lpp2 ! beam 2 type
     none    = pdlabel1     ! PDF type for beam #1
     none    = pdlabel2     ! PDF type for beam #2
  230000	= lhaid ! if pdlabel=lhapdf, this is the lhapdf number
```

(`banner.py` L4585–4610 set `pdlabelX = none` for `lpp = 0` and mark
`pdlabel1` user-set, so `PDLabelBlock` writes its `template_on`.) The parser
leaves per-beam labels inert on fixed-energy beams, so `pdlabel` keeps its
default, which F1 moved from `nn23lo1` to `lhapdf`, beside the card's
`lhaid = 230000`. `tests/common/pdfset.rs::set_alpha_s_info` read
`card.pdlabel`/`card.lhaid` directly, saw `lhapdf`/230000 and panicked;
`card.pdf_choice()` returns `PdfChoice::None` for that card. The tests read all
banked runs sorted, and `ee_to_ee` is the first such card after
`ddx_to_epemg`, which matches the CI log.

Reproduced here at `cabe19e`:

```
$ cargo test -p vibegraph-lib --features extended-validation --test validate_alphas --test validate_scales --test validate_mlm_dumps --no-fail-fast
validate_alphas: thread 'banked_run_logs_pin_the_alpha_s_source_rule' panicked at vibegraph-lib/tests/common/pdfset.rs:30:28:
  no PDF set registered for lhaid 230000            (same for banked_events_reproduce_aqcdup)
  test result: FAILED. 0 passed; 2 failed
validate_mlm_dumps: test result: ok. 0 passed; 0 failed; 3 ignored
validate_scales: banked_events_reproduce_aqcdup_from_the_computed_scale panicked at pdfset.rs:30:28, lhaid 230000
  test result: FAILED. 9 passed; 1 failed; 1 ignored
```

The `validate_scales` failure never showed in CI: cargo stops at the first
failing test target, and every `validate_*` target after `validate_alphas` went
unmeasured.

**Cause 2 (the design claim was wrong).** With the helper following
`pdf_choice` (a scratch patch, reverted), the next failure is:

```
thread 'banked_run_logs_pin_the_alpha_s_source_rule' panicked at vibegraph-lib/tests/validate_alphas.rs:188:29:
pp_to_ll: alpha_s source: run card sets pdlabel = nn23lo1, MadGraph's built-in parameterisation; ...
```

`pp_to_ll` and `pp_to_ll_qcd0` are banked proton runs carded `nn23lo1`/230000
(their own `Events/run_01`, not the `dy13_*` cards their σ and sample cells
use), and both are in the gated `AQCDUP` lists of `validate_alphas` and
`validate_scales`. The D1c correction ("the one proton `nn23lo1` comparison is
the ungated `uu_tt` row") was wrong. I stopped and reported; the user decided:
re-card both to `lhapdf`/247000 and rebank them, and keep CI green meanwhile by
reporting them explicitly.

The claim was also wrong the other way: `uu_tt` needs no rebank.
`onshell_uu_run_card.dat` says `lpp = 1`, `nn23lo1`, but every row that reads it
(`uu_tt`, `uu_tt_mg371`, `uu_full`; `gen_onshell_veto.sh:83-85`,
`bank_seeded_sigma.sh:416-418`, `cli_onshell_veto.rs`) overrides
`lpp1 = lpp2 = 0` at √s = 500, so the label is inert and nothing is refused.
CI's `cli_onshell_veto` on `9e413af` passed `uu_tt` under F1 (`ok`, pull +4.88,
informational). **The brief and the manager's decision both list `uu_tt` for
rebanking; it should be dropped from the request.**

### Fix

- `tests/common/pdfset.rs`: `set_alpha_s_info` follows `RunCard::pdf_choice`
  (the rule the densities follow), `None` for no density or a refused label,
  whose refusal then surfaces where the coupling is built.
- `tests/common/mod.rs`: `AWAITING_REBANK_RUNS` (`pp_to_ll`, `pp_to_ll_qcd0`,
  each with its reason) and `awaiting_rebank(name, card)`, which asserts the
  card is still refused for `nn23lo1` and prints
  `pp_to_ll: not compared, banked on pdlabel = nn23lo1 (lhaid 230000), which this crate refuses; awaiting re-card to pdlabel = lhapdf, lhaid = 247000 and a rebank`.
  A rebanked card stops being refused and fails the gate until the row is
  removed from the list: the rebank PR flips them back.
- `validate_alphas`: both tests skip and record awaiting rows, compare against
  `compared(…)` (the declared list less the awaiting rows), and assert the
  awaiting rows seen equal those declared and present.
- `validate_scales::banked_events_reproduce_aqcdup_from_the_computed_scale`:
  the same. Their scales are still replayed by
  `banked_events_reproduce_every_printed_scale`; no density or αs enters that.
- `validation/validate.sh`: `cargo test … --no-fail-fast`. The banked job's
  cargo invocation lives here, not in `.github/workflows/ci.yml`, which runs
  `pixi run --skip-deps validate`; the manager's note named `ci.yml`.
- **`PDFSUP`** (below).
- New hermetic target `vibegraph-lib/tests/pdfset_choice.rs`: the banked
  `ee_to_ee` PDF lines, verbatim, give `pdf_choice() == None` and
  `set_alpha_s_info == None`.

### `PDFSUP` on MadGraph-written fixed-energy cards

The same cards exposed a library regression: `RunCard::pdfsup()` matched
`"lhapdf" => lhaid` and returned 230000 for them, where MadEvent wrote
`2.500000e+02 … 0 0 247000 247000 -4 1` (banked `ee_to_ee` `<init>`).
MadGraph's rule, read at `b7687064`:

- the `<init>` line is `get_banner_init_information` (`banner.py:3824-3837`),
  called from `lhe_parser.py:1057` on `self.banner.run_card`;
- that card is `charge_card`'s `RunCard(self[tag], consistency=False)`
  (`banner.py:538`), so `check_validity` and `PDLabelBlock` never run and
  `pdlabel` is the card's own `pdlabel` line or the template's `nn23lo1`
  (`banner.py:4253`);
- `PDFSUP = get_pdf_id(pdlabel)` (`banner.py:3839-3853`): `lhaid` under
  `lhapdf`, 247000 for `nn23lo1`. (`setrun.f`'s `get_pdfup` writes `heprup`
  after setting `pdlabel = 'none'` on `lpp = 0`, which would give 0; the LHE
  file does not come from there.)

`RunCard` now records whether the card wrote a `pdlabel` line
(`pdlabel_written`, kept by `for_decay`), and `pdfsup` is: on proton beams the
`lhaid` `pdf_choice` reads; on fixed-energy beams `get_pdf_id` of the written
label, or of `nn23lo1` without one. The field is serialized with the card; the
artifact is version 14, unreleased on this branch.

### Mutations (the CI fix)

`mutate.py` applied one mutation, ran the command, and restored the file byte
for byte; `git diff --stat` was unchanged after the sweep.

| id | mutation | command | result |
|---|---|---|---|
| C1 | `set_alpha_s_info` reads `card.pdlabel`/`card.lhaid` directly | `cargo test -p vibegraph-lib --test pdfset_choice` | caught, exit 101 (`a_madgraph_fixed_energy_card_reads_no_set` panics at `pdfset.rs:39`, the old panic) |
| C2 | `pdfsup` loses the template-label arm | `cargo test -p vibegraph-lib --lib runcard::` | caught, 47 passed 1 failed (`a_madgraph_fixed_energy_card_reports_the_template_pdfsup`) |
| C3 | parse never records a `pdlabel` line | same | caught, 46 passed 2 failed |
| C4 | `for_decay` drops `pdlabel_written` | same | caught, 46 passed 2 failed |
| C5 | interim marking removed (`AWAITING_REBANK_RUNS = []`), on the bundle | `cargo test -p vibegraph-lib --features extended-validation --test validate_alphas` | caught, 0 passed 2 failed (panic at `validate_alphas.rs:228`, the `pp_to_ll` refusal) |
| C6 | the same | `cargo test … --test validate_scales -- banked_events_reproduce_aqcdup_from_the_computed_scale` | caught, 0 passed 1 failed (panic at `validate_scales.rs:1100`) |

## 2. The CLI half

**Surface.** Before: `vibegraph integrate … [--pdf-set <PDF_SET>] [--pdf-dir <PDF_DIR>]`.
After: neither exists; the card's `lhaid` names the set and member, and
`$LHAPDF_DATA_PATH` (else `$LHAPATH`), CVMFS, the cache and a consented
download find it. `generate` was already without them. `docs/src/cli/reference.md`
was regenerated with `scripts/gen-cli-docs.sh`.

- `vibegraph-cli/src/integrate.rs`: the two fields gone; `integrate_proton`
  calls `assets::pdf_tiers()` and `load_for_run_card`.
- `vibegraph-cli/src/assets.rs`: `pdf_tiers()` takes no directory.
- `vibegraph-lib/src/pdf/select.rs`: `load_named` and its test removed (its
  only caller was `--pdf-set`).

**Migration.**

- `cli_onshell_veto.rs:140,153`: every row's `integrate` runs with
  `pdf_data_path(validation/pdf)`; `pp_z` drops `--pdf-dir`
  (`heavy::drell_yan_with_the_z_window_vetoed_matches_madevent`).
- `cli_pdf_cache_fetched.rs`: the cache test reaches its renamed set through a
  `pdfsets.index` in the cache for `lhaid` 999990 (in no set of LHAPDF's index:
  past `DSSV14pol` 960000, below 1000000) with the `.info`'s `SetIndex` moved to
  match; `an_explicit_pdf_dir_still_takes_precedence` became
  `lhapdf_data_path_takes_precedence_over_the_cache` (same set in both; the
  cached copy has no `SetIndex`, so a run that read it would be refused).
- `cli_pdf_cache.rs`: `an_unpinned_missing_set_is_refused_without_reaching_the_network`
  reaches `NoSuchSet_v99` through a cache index at 999990 instead of
  `--pdf-set`; new `a_pdf_option_passed_to_integrate_is_an_argument_error`.
- Every other test in run inputs §7 already passed `LHAPDF_DATA_PATH`
  (`common::PdfDataPath`, F1/H1). `scripts/acceptance.sh`, the Pythia scripts and
  `gen_reweight_oracle.py` only unset or pop the retired variable, which is
  harmless, and pass no flag.
- `validation/manifest.toml`: the `cli-pdf-cache-fetched` and `pp_to_jj` rationales.
- Docs: `docs/src/cli/overview.md` (the departure paragraph, the data table),
  `docs/src/guide/10-hadronic.md`, `docs/src/guide/11-unweighting-events.md`.
- Concepts: `tooling/asset-resolution.md`, `scales-pdf/pinned-pdf-set.md`,
  `scales-pdf/alpha-s-sources.md`, `run-card/run-card-parser-and-defaults.md`,
  `run-card/field-classification.md`, `events/lhef-record-conventions.md`, and
  the corrected claims in `pipeline/integrate-generate-split.md` §4,
  `pipeline/run-inputs-in-the-artifact.md` §6 and the sprint log (D1c entry).

**The `nn23lo1` migration list** (split design §4): `runcard.rs` defaults and
`matching.rs:264` were F1's; the `pdfsup` doc is rewritten above;
`coupling/alphas.rs` has no label table since F1; `cache/pinned.rs:32` already
reads correctly; `cli_decay.rs:468` comment rewritten. The inline proton cards
of the listed test files moved to the grid's αs in F1; the banked suites below
measure whether any pin moved: none failed.

## 3. `<vibegraphInputs>` (C1, §3a central entry)

`lhef::write::InputsRead { pdf: Option<(PdfInput, Tier)>, grid_sha256 }` and
its `header_block()`; `GenerateAssets` gains `inputs`, written after
`<MGRunCard>` on both the hadronic and the fixed-energy/decay paths.
`IntegrateArtifact::read_hashed` digests the bytes it decodes, so the grid
digest and the grid come from one read; the PDF entry is `load_recorded`'s
`LoadedPdf::input` (the files just parsed and checked) and its tier.
Systematics entries are not written (none are evaluated).

On a written file (`cli_integrate::generate_records_the_pdf_files_and_the_grid_it_read`):

```
<vibegraphInputs>
  <pdf use="central" set="NNPDF23_lo_as_0130_qed" lhaid="247000" member="0" tier="lhapdf_data_path">
    <file name="NNPDF23_lo_as_0130_qed.info" sha256="…"/>
    <file name="NNPDF23_lo_as_0130_qed_0000.dat" sha256="…"/>
  </pdf>
  <grid sha256="…"/>
</vibegraphInputs>
```

compared as a string against the SHA-256 of the two files on disk and of
`grid.bin.zst`. Hermetic: `cli_generate::the_header_records_the_grid_it_was_generated_from`
(fixed-energy: grid only) and `lhef::write::tests::the_inputs_block_lists_the_central_member_and_the_grid`
(escaping, tier name, no-PDF form).

## 4. F1's Found 2 and 3

- **Found 3, done.** `DownloadRequest` and `network::Download` carry
  `archive`; the index request says `saved as: …/pdfsets.index`, a set archive
  still `unpacks to:`. Tested in `network::tests` and the two
  `cache::lhapdf` request tests.
- **Found 2, not done** (a consent-flow design choice, not small): listed below.

### Mutations (the CLI half)

| id | mutation | command | result |
|---|---|---|---|
| D1 | `integrate` accepts `--pdf-set` again | `cargo test -p vibegraph --test cli_pdf_cache a_pdf_option_passed_to_integrate` | caught (`cli_pdf_cache.rs:222`) |
| D2 | the cache is searched before `$LHAPDF_DATA_PATH` | `cargo test -p vibegraph --features extended-validation --test cli_pdf_cache_fetched lhapdf_data_path_takes_precedence` | caught (`cli_pdf_cache_fetched.rs:200`) |
| D3 | `generate` writes no PDF entry | `cargo test -p vibegraph --features extended-validation --test cli_integrate generate_records` | caught (`cli_integrate.rs:391`) |
| D4 | the `tier` attribute dropped | `cargo test -p vibegraph-lib --lib lhef::write::tests::the_inputs_block` | caught (`write.rs:540`) |
| D5 | the grid digest not of the bytes read | `cargo test -p vibegraph --test cli_generate the_header_records_the_grid` | caught (`cli_generate.rs:643`) |
| D6 | the prompt ignores `archive` | `cargo test -p vibegraph --bin vibegraph the_index_download_is_saved` | caught (`network.rs:326`) |
| D7 | the index request claims to be an archive | `cargo test -p vibegraph-lib --lib cache::lhapdf::tests::the_index_is_fetched` | caught (`lhapdf.rs:625`) |
| D8 | the hadronic path drops the block | as D3 | caught (`cli_integrate.rs:391`) |

All sixteen exit 101 on a test failure, not a compile error.

## 5. Rebanking request: `pp_to_ll`, `pp_to_ll_qcd0` (M3)

`uu_tt` is not in it (§1). No ROWS-driven driver covers these two rows:
`build.sh` (`pixi run -e madgraph build-diagrams`) regenerates every
`scripts/*.mg5` whose `output/<row>` is absent and skips the rest, and
`gen_amplitude_tables.py` takes table keys as positional arguments. Selection
is therefore by moving the two directories aside.

1. **Card change**, in `validation/madgraph/scripts/pp_to_ll.mg5` and
   `pp_to_ll_qcd0.mg5`, the launch block, as `pp_to_bb.mg5` has it:
   ```
   launch
     set ebeam1 6500
     set ebeam2 6500
     set pdlabel lhapdf
     set lhaid 247000
     done
   ```
   (Not committed here, so the committed generators keep matching the banked
   runs until the rebank lands.)
2. **Regenerate**, from a checkout of this branch with the pinned submodule:
   ```
   pixi run -e madgraph fetch-pdf
   mkdir -p ../retired-refdata-10 && mv validation/madgraph/output/pp_to_ll validation/madgraph/output/pp_to_ll_qcd0 ../retired-refdata-10/
   pixi run -e madgraph --skip-deps build-diagrams
   pixi run -e madgraph --skip-deps generate-amplitude-tables pp_to_ll_qcd0
   pixi run -e madgraph --skip-deps extract-diagrams
   pixi run -e madgraph --skip-deps extract-configs
   ```
   `build-diagrams` rebuilds only the two missing directories, and its
   `-lc++` handling already covers `pdlabel = lhapdf` links. The amplitude
   table `amplitudes/pp_to_ll_qcd0.json` draws its event points from
   `pp_to_ll_qcd0`'s banked `unweighted_events.lhe.gz`
   (`gen_amplitude_tables.py:318-323`, row `pp_to_ll_qcd0`), so it is
   regenerated; `uux_to_mumu.json` reads its own events and does not move.
3. **What changes.** Bundle (cut 11 = cut 10 plus these replaced, as cut 5 did
   for four rows): every file `assemble_bundle.sh` takes under
   `output/pp_to_ll/` and `output/pp_to_ll_qcd0/` (`Cards/*`,
   `Events/run_01/*`, `SubProcesses/results.dat`, the `P*/G*` results and
   `run_*_log.txt`, `build.log`). Committed: `validation/manifest.toml`
   `[refdata]` (`version`, `archive`, `url`, `sha256`, `size_bytes`, a cut-11
   paragraph), `validation/madgraph/amplitudes/pp_to_ll_qcd0.json`, and the two
   `.mg5` scripts. Expected byte-identical, to be checked: `diagrams.json`,
   `configs.json` (the run card does not enter enumeration),
   `couplings/pp_to_ll_qcd0.json`, `output/pp_to_ll_qcd0_amplitude.csv` (its
   points are the fixed `sqrt_s_list` grid at seed 5, not events).
4. **Expected outputs.**
   - `Cards/run_card.dat`: `lhapdf = pdlabel`, `247000 = lhaid`.
   - `<init>`: `2212 2212 6.500000e+03 6.500000e+03 0 0 247000 247000 -4 1`
     (PDFSUP unchanged from today's).
   - Run log: `New value of alpha_s from PDF lhapdf: 0.13000271085472234`, as
     every grid-sourced row prints today.
   - σ (`Integrated weight`, today 1686.34 pb on both): a small shift within
     MadGraph's own error; the re-carded `pp_to_ll_scalefact2` moved +0.0066%
     and `pp_to_llj` +0.21%.
   - With the rows removed from `AWAITING_REBANK_RUNS` and added to
     `GRID_ALPHA_S_RUNS` in `validate_alphas.rs` and `validate_scales.rs`:
     every one of their 10000 `AQCDUP` values within its printing budget from
     the grid, and the run-log test reproducing 0.13000271085472234 to
     `GRID_ALPHA_S_TOL`. `amplitude_oracle` on the new table green.
   - Leaving the rows in `AWAITING_REBANK_RUNS` after the rebank fails
     `awaiting_rebank`'s assertion by design.

## 6. Gates

On the final tree:

```
$ cargo fmt --all --check                                         exit 0
$ cargo clippy --workspace --all-targets -- -D warnings           Finished … exit 0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
                                                                  Finished … exit 0
$ cargo test --workspace --no-fail-fast                           exit 0
  42 targets "test result: ok"; lib unit tests 1084 passed, 12 ignored
$ python3 scripts/kb.py lint                                      0 errors, 4 warnings
  (the 4 pre-existing backlog `state: blocked` warnings)
```

The banked layer, with the bundle present (dev profile, which is optimized here):

```
$ cargo test --workspace --features vibegraph/extended-validation,vibegraph-lib/extended-validation --no-fail-fast -- --nocapture
```

| target | result |
|---|---|
| `validate_alphas` | ok, 2 passed: `AQCDUP: 520000 events across 52 runs within their printing budget, worst 0.999 of budget (in pp_to_llj_dyn)`; `alpha_s source rule pinned against 58 MadGraph run logs`; `pp_to_ll`, `pp_to_ll_qcd0` reported `not compared, … awaiting re-card …` |
| `validate_scales` | ok, 10 passed 1 ignored: `AQCDUP from the computed scale: 510000 events across 51 runs, worst 1.000 of budget (in uux_to_epemg)`; the two awaiting rows reported |
| `validate_mlm_dumps` | ok, 0 passed 3 ignored (they need MadEvent's instrumented dumps) |
| `validate_hadronic` | ok, 14 passed 26 ignored |
| `validate_sigma` | ok, 8 passed 33 ignored |
| `validate_samples` | ok, 6 passed 2 ignored |
| `validate_lhef` | ok, 3 passed |
| `validate_scale_couplings`, `validate_unweighting`, `validate_vegas`, `validate_pdf_grid`, `validate_madgraph_diagrams` | ok |
| `cli_generate_proton` | ok, 5 passed 2 ignored |
| `validate_samples_proton` | ok, 11 passed 6 ignored (391.7 s) |
| `cli_reweight_proton` | ok, 2 passed |
| `cli_decay_chain_events` | ok, 7 passed |
| `cli_decay_chain`, `cli_integrate` (5), `cli_pdf_cache_fetched` (3), `cli_pdf_cache` (10), `cli_onshell_veto` (4), `cli_generate` (17), `cli_decay` | ok |
| `sm_interned_blob`, `ufo` | **failed**: `SM UFO source not found at …/research/refs/mg5amcnlo/models/sm`. This worktree has no `mg5amcnlo` submodule; CI checks it out |

68 targets ok, 2 failed on the missing submodule only. Every proton suite that
compares against a banked MadGraph run passed, so no pin moved under the
`lhapdf`/247000 default (F1 Found 1 measured).

## 7. Found

- **Stale local index can prompt for the wrong set** (F1 Found 2, not done).
  `pdf/select.rs::try_table` locates each index's candidate through every tier,
  download included, so a stale `$LHAPDF_DATA_PATH` index whose candidate for a
  newer id is an older set not on disk asks to download that older set before
  the next index is read. Closing it: a first pass over the local indices that
  locates candidates without the download tier, and a download only for the
  current index's candidate (fetched with consent); the choice of which index
  counts as current is a consent-flow decision for the design.
- **`tooling/asset-resolution.md` describes functions F1 removed**
  (`assets::resolve_pdf_set_dir`, `Source::DataPath`) and the old three-step
  table; only the `--pdf-dir` cells were corrected here. Closing it: rewrite the
  PDF half against `cache::lhapdf::{Tiers, locate_set}`.
- **No ROWS-driven driver regenerates a single banked process directory.**
  `build.sh` rebuilds whatever is absent; rebanking a row means moving its
  directory aside by hand (§5). Closing it: `ROWS=` in `build.sh`, as
  `bank_seeded_sigma.sh` and the `gen_*` scripts have.
- **The reference bundle could not be fetched with `gh release download`**
  here (GraphQL is blocked in cloud sessions); `gh api -H 'Accept:
  application/octet-stream' repos/nsmith-/vibegraph/releases/assets/<id>`
  works, and `zstd` is absent. Closing it: `vg_download_release_asset` falls
  back to the REST asset endpoint; `fetch_common.sh` names the `zstd` need in
  its setup docs or accepts a Python fallback.
