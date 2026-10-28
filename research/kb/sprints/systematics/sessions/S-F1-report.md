---
type: Session Report
title: "S-F1 report: systematics weights for unmatched events"
description: "The typed request at artifact version 15, YAML metadata with the member cascade, lo_weight over the <mgrwt> record, MadGraph's header and weight bytes, the generation path; every one of 17.4M banked weights within one unit of MadGraph's eighth digit, the header byte-identical from three entry points, the controls as the design predicts."
status: draft
tags: [systematics, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
TBD
Worktree `/home/user/wt/S`, branch `sprinte-systematics`, from `43bbc244`.
Implements design §8's S-F1 scope of
[the systematics weights](../../../events/systematics-weights.md), with Found
F1 and F2 of the [S-D1 report](S-D1-report.md). Logs are in the session's
scratchpad (`S-F1-*.log`); the commands and their output tails are below.

## What was implemented

All in `vibegraph-lib/src/systematics/` unless said otherwise.

- **The typed request** (`request.rs`): `VariationRequest` with `RequestSource`,
  `DynScale`, `Axis`, `PdfEntry`/`PdfRef`/`MemberRef`, MadGraph's defaults,
  three readers (`from_card_arguments`, `from_sys_fields`, `from_command_line`
  through `shlex`), and `split_card_list`, a port of `banner.py`'s list reader
  (quoted runs kept whole and glued to the token before them, outer quotes
  stripped). Every argument-level refusal of design §1.3 is a `RequestError`.
- **The card side** (`runcard.rs`, `runcard/classes.rs`): `RunCard::written`
  (a `BTreeSet` of canonical names, `auto` on a list field not counted)
  replaces `pdlabel_written`; `RunCard::default()` and `decay_default()` count
  `systematics_arguments` as written (MadGraph's own card writes it).
  `split()` and `split_decay()` now return `Result` and read the request:
  program dispatch (`none`/`systematics`/`auto`; `syscalc`, capitalised and
  unknown programs refused), entry point by `written`, `use_syst` resolved off
  on fixed-energy beams unless written (written `T` refused), and a matched
  card (`ickkw = 1`) asking for weights refused. `FIELD_CLASSES`: `use_syst`,
  `systematics_program`, `systematics_arguments`, `sys_scalefact`,
  `sys_alpsfact`, `sys_pdf` are `Consumed`; `sys_matchscale` and
  `sys_scalecorrelation` stay `IgnoredBenign` with a SysCalc-only reason;
  `B_SYST` and `warn_unwritten_systematics` are gone.
- **Artifact version 15** (`artifact.rs`): `FORMAT_VERSION = 15`,
  `OLDEST_READABLE_VERSION = 15`, the comment saying why 14 cannot be
  upgraded; `GenerateSettings::systematics` is `{enabled, request:
  Option<VariationRequest>, card_fields}`.
- **F2, YAML metadata** (`pdf/grid.rs`): `.info` files and member headers are
  parsed with `yaml_serde` (the YAML organisation's maintained `serde_yaml`);
  `SetInfo` keeps the raw map and gains `description()` and `error_type()`
  (lowercased, `unknown` when absent). The `AlphaS_*` entries are read where
  present and left unset where not, since some sets give them only per member.
- **F1, the member cascade** (`pdf/mod.rs`): `PdfSet::member` parses the
  member header and lays it over the `.info` (`member_meta`), so the member
  carries its own `AlphaSInfo` (`PdfMember::alpha_s_info`) and its own
  `ForcePositive`. Production call sites (`integrate`, `generate`) now build the
  run's coupling from the member's metadata.
- **Formatters** (`format.rs`): Python `str(float)` (`py_float`), Fortran
  `Ew.d`/`Iw`, the `%+.7e` weight spelling (`wgt_text`), and
  `as_written_e15_8`.
- **The record** (`record.rs`): `LoRecord`, `PdfChain`, `RecordKinematics`
  (`RecordKinematics::of(&LheEvent)`), the `unwgt.f` writer and a
  `parse_lo_weight` parser; `as_written()` rounds the record to its printed
  digits, which is what the weights are evaluated on.
- **`lo_weight`** (`weight.rs`): `get_lo_wgt` term by term over a
  `VariationPdf` trait, chain and emission terms included (gated only through
  S-F2), the four dynamical scales with quirk L's negative radicand and a
  non-positive scale as errors.
- **The header** (`header.rs`): `expand` (`get_all_fct`, slot identity for
  quirk J, the nominal re-inserted before `lhaid + 1`) and `banner_lines`
  (`write_banner`'s state machine, quirk H's closing tags, the unreachable
  second PDF-group branch omitted).
- **Resolution and the writer** (`plan.rs`): `resolve` opens each `--pdf`
  entry's set through the tiers (pins and indices for ids, by name otherwise;
  `pdf/select.rs` gained `open_set_for_lhaid`/`open_set_by_name`), asks for
  every download a round of lookups needs in one question (O7), refuses the
  quirk-A list, loads every distinct member in parallel with its own
  `GridAlphaS` (a non-`ipol` type refused naming set and member), checks the
  run's own `.info`/`.dat` digests against the central read, and records one
  `SystematicsSet` per set (O8). `Writer` holds the variations, ids and header
  and gives an event its weights.
- **Generation** (`generate.rs`, `generate/source.rs`, `lhef/emit.rs`,
  `lhef/write.rs`): an unmatched proton event under `use_syst` carries a
  `LoRecord` (`RecordFill`: `n_qcd` of the configuration's diagram, `μR`, one
  chain entry per beam, `f₁f₂` as `totfact`, beam order from the drawn
  ordering); the emitter writes `<mgrwt>` then one `<rwgt>` with the
  systematics weights first and the reweighting hypotheses after, computing
  the ratios at write time in parallel chunks of 1024; `<initrwgt>` holds the
  systematics groups then the `mg_reweighting` group; `<vibegraphInputs>`
  gains one `<pdf use="systematics">` element per set. Refused before any
  event: a matched run, a reweight id that is also a systematics id (F3), and
  `--dyn=1` on a one-particle final state (O3, also at integrate). A decay
  with `use_syst = T` warns and writes nothing, as MadGraph bypasses it.
- **CLI** (`vibegraph-cli/src/generate.rs`, `integrate.rs`):
  `generate --systematics '<arguments>'` (refused on a decay and on an
  artifact without `use_syst`), resolution through the run's tiers and
  consent, and integrate's split error surfaced.
- **Docs**: `docs/src/cli/reference.md` regenerated
  (`scripts/gen-cli-docs.sh`), `docs/src/cli/overview.md` and
  `docs/src/guide/11-unweighting-events.md` gain the systematics weights.

## Deviations from the design, and choices made inside it

1. **`n_qcd` of an unmatched event** (§2.1, quirk D, O4): filled from the
   configuration the event is labelled in (`ProtonSelection::config`), the
   configuration `generate` already draws an event's colour flow in. Channels
   here are merged per map, so "the channel that drew the event" has no single
   diagram; the labelled configuration is the nearest one-diagram notion. The
   design's parenthesis, MadEvent's `nqcd(igraphs(1))`, is the clustering's
   graph under a dynamical scale, which neither choice reproduces: Found FA.
   Where no configuration carried weight, the group's highest order.
2. **Weights are evaluated on the record and `XWGTUP` as written**
   (`LoRecord::as_written`, `%+.7e` of `XWGTUP`), so a file's weights are what
   MadGraph's module computes reading the same file; the momenta are the
   record's unrounded ones (eleven printed digits, below the eighth-digit
   tolerance).
3. **O3 is wired at both integrate and generate** (after the multiplicity
   groups are built, which is where the written final state is known), and
   pinned only at the unit level: no `2 → 1` proton process reaches that
   point in this build (Found FB).
4. **The `setscales.f` control** is implemented as the production `lo_weight`
   on the event's momenta boosted to the partonic centre of mass and scaled by
   the card's `scalefact` (every dynamical scale is homogeneous of degree one
   in the momenta). It detects on exactly the five runs §6.2 lists and is blind
   on the other seven, but its magnitudes differ from S-D1's prototype
   (`pp_to_bb_qcd2` 2.1e4 against 1.7e3, `pp_to_ll_scalefact2` 1.6 against
   0.88): the two controls are not the same mutation, only the same class.
5. **A set named twice in different spellings** (`errorset` and `247000@0`)
   is opened and listed once per spelling; slots stay distinct either way, as
   quirk J needs.
6. **The line reader** (`runcard.rs` `split_line`) now splits at the last
   `=`, as `banner.py` does: without it no card's `systematics_arguments` line
   was ever read (Found FC). Not in the design; required for the entry-point
   rule to be reachable.

## Corrections to the brief

- The brief's gate list says "the banked layer, `bash validation/validate.sh`,
  run detached", and §8 says the same. `validate.sh` runs every
  extended-validation target under `release-debug`, so on this shared host it
  is the longest step by far; it was run detached with an untracked
  `.cargo/config.toml` dropping `release-debug` debug info, as the
  extended-validation skill prescribes.
- The extended-validation skill's map for "the LHEF writer/reader" also names
  `pixi run -e pythia validate-pythia`, which needs Pythia 8 and is outside
  `validate.sh`: not run (no Pythia on this host).

## Gate 1: every weight of the twelve banked runs (§6.1)

`cargo test -p vibegraph-lib --features extended-validation --test
validate_systematics -- --nocapture` (dev profile; the banked layer reruns it
under `release-debug`). Production path end to end: `load_for_lhaid(247000)`
from `validation/pdf`, `resolve`, `Writer::new`, `Writer::weights` on the
record and particles parsed from each banked event; every `<wgt>` compared as
printed (`%+.7e`), the id order asserted on every event.

| run | weights | equal strings | 1 unit | > 1 unit | equal |
|---|---|---|---|---|---|
| `pp_to_bb` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_bb_fixed` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_bb_iht` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_bb_qcd2` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_jj` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_jj_ht` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_ll` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_ll_qcd0` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_ll_scalefact2` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_llj` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_llj_dyn` | 1450000 | 1450000 | 0 | 0 | 100.00000% |
| `pp_to_llj_fixed` | 1450000 | 1450000 | 0 | 0 | 100.00000% |

**17 400 000 of 17 400 000 weights print MadGraph's string exactly**; ids
1–145 in order on all 120 000 events. Worst relative deviation before
rounding, per weight class (all inside the half unit of the eighth digit):

```
pp_to_bb             dyn 4.984e-8, muf 3.854e-8, mur 1.937e-8, mur_muf 4.973e-8, nominal 2.220e-16, pdf 4.917e-8 
pp_to_bb_fixed       dyn 4.806e-8, muf 2.926e-8, mur 2.344e-8, mur_muf 3.613e-8, nominal 2.220e-16, pdf 4.826e-8 
pp_to_bb_iht         dyn 4.333e-8, muf 1.261e-8, mur 1.265e-8, mur_muf 1.491e-8, nominal 2.220e-16, pdf 4.655e-8 
pp_to_bb_qcd2        dyn 4.979e-8, muf 4.324e-8, mur 1.938e-8, mur_muf 4.960e-8, nominal 2.220e-16, pdf 4.904e-8 
pp_to_jj             dyn 4.961e-8, muf 1.146e-8, mur 9.644e-9, mur_muf 4.977e-8, nominal 2.220e-16, pdf 4.908e-8 
pp_to_jj_ht          dyn 4.968e-8, muf 2.946e-8, mur 2.806e-8, mur_muf 3.501e-8, nominal 2.220e-16, pdf 4.894e-8 
pp_to_ll             dyn 4.915e-8, muf 4.858e-8, mur 2.220e-16, mur_muf 4.858e-8, nominal 2.220e-16, pdf 4.405e-8 
pp_to_ll_qcd0        dyn 4.915e-8, muf 4.858e-8, mur 2.220e-16, mur_muf 4.858e-8, nominal 2.220e-16, pdf 4.405e-8 
pp_to_ll_scalefact2  dyn 4.950e-8, muf 3.938e-8, mur 2.220e-16, mur_muf 3.938e-8, nominal 2.220e-16, pdf 3.588e-8 
pp_to_llj            dyn 3.680e-8, muf 1.393e-8, mur 1.118e-8, mur_muf 1.632e-8, nominal 2.220e-16, pdf 4.643e-8 
pp_to_llj_dyn        dyn 2.498e-8, muf 1.521e-8, mur 1.338e-8, mur_muf 1.663e-8, nominal 2.220e-16, pdf 3.364e-8 
pp_to_llj_fixed      dyn 3.010e-8, muf 1.450e-8, mur 1.044e-8, mur_muf 1.651e-8, nominal 2.220e-16, pdf 2.749e-8 
```

The `mur` class is `2.2e-16` on the three `ll` runs: `n_qcd = 0`, so a
μR-only variation is the nominal.

## Gate 2: the header, byte for byte, from three entry points

Same test binary, `the_header_is_madgraphs_byte_for_byte_from_every_entry_point`.
For each run: the banked `<initrwgt>` block's sha256 is asserted
`eb55efbb97a2cabfb6dd59595d3534770ed4175971d0cc217de18320d893c7d4`, and the
block this crate writes is compared byte for byte from (1) the arguments
`--mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset`, (2) the banner's own
`<MGRunCard>` through `RunCard::parse` and `split` (`--from_card`, the line
present: asserted `RequestSource::Arguments`), (3) the same card with its
`systematics_arguments` line deleted (asserted `RequestSource::SysFields`, the
`mur,muf,alps,dyn` block). **36 of 36 identical**, e.g.

```
header pp_to_bb             arguments    sha256 eb55efbb97a2cabf identical=true
header pp_to_bb             --from_card  sha256 eb55efbb97a2cabf identical=true
header pp_to_bb             sys_*        sha256 eb55efbb97a2cabf identical=true
… (all twelve runs, three lines each, every one identical=true)
test the_header_is_madgraphs_byte_for_byte_from_every_entry_point ... ok
test every_banked_weight_is_reproduced_and_the_controls_detect ... ok
test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 64.07s
```

The source assertions were added after this dev-profile run (the first
`--from_card` reading silently took the `sys_*` path: Found FC); the banked
layer's run below carries them.

## Gate 3: the negative controls (§6.2)

Each control is the production `lo_weight` with one change, on all twelve
runs; "detected" means a maximum relative deviation above `1e-7`, the gate's
one-unit tolerance. The test asserts detected on every run for the first two
and exactly on the design's five for the third.

| run | μR ↔ μF factors | αs power `n_qcd + 1` | `setscales.f` scale |
|---|---|---|---|
| `pp_to_bb` | detected 4.219e4 | detected 1.062e1 | detected 2.100e4 |
| `pp_to_bb_fixed` | detected 3.594e0 | detected 5.299e-1 | detected 1.181e-2 |
| `pp_to_bb_iht` | detected 1.172e0 | detected 3.297e-1 | detected 6.053e-4 |
| `pp_to_bb_qcd2` | detected 4.219e4 | detected 1.740e1 | detected 2.100e4 |
| `pp_to_jj` | detected 3.225e0 | detected 5.070e-1 | blind 5.810e-8 |
| `pp_to_jj_ht` | detected 8.468e-1 | detected 3.494e-1 | blind 4.968e-8 |
| `pp_to_ll` | detected 3.883e0 | detected 8.512e-1 | blind 4.915e-8 |
| `pp_to_ll_qcd0` | detected 3.883e0 | detected 8.512e-1 | blind 4.915e-8 |
| `pp_to_ll_scalefact2` | detected 3.879e0 | detected 1.052e0 | detected 1.601e0 |
| `pp_to_llj` | detected 2.295e0 | detected 6.263e-1 | blind 4.643e-8 |
| `pp_to_llj_dyn` | detected 1.644e0 | detected 5.778e-1 | blind 3.364e-8 |
| `pp_to_llj_fixed` | detected 1.601e0 | detected 5.215e-1 | blind 3.010e-8 |

As designed: the first two detect on 12 of 12; the frame control detects on
`pp_to_bb`, `_bb_fixed`, `_bb_iht`, `_bb_qcd2`, `_ll_scalefact2` and is at
the gate's noise (≤ 5.8e-8) on the seven massless, `scalefact = 1` runs. Its
magnitudes are not S-D1's (deviation 4 above).

## Gate 4: the round trip on this crate's own events (§6.4)

`cli_generate_proton::the_systematics_weights_round_trip_through_the_written_file`
(banked layer): the banked `pp_to_llj_fixed` card integrated and 400 events
generated; the written `<initrwgt>` must equal MadGraph's banked block, the
file must record `<pdf use="systematics" set="NNPDF23_lo_as_0130_qed"
lhaid="247000"`, every weight re-evaluated from the written record and
particles must be within one unit of what was written, and per event: each
chain's `pdg` is the incoming parton's, its `x` the parton's `E/ebeam` (to
1e-8), its `q` `SCALUP` as written, `αs(μR)` off the set equals `AQCDUP` to
1e-7, and weight 45 is `XWGTUP`'s spelling. Result: see the banked layer
below.

## Python floats, refusals, and the rest of the unit layer

- `systematics::format::tests::py_float_matches_cpythons_str`: twenty values
  against `python3 -c 'print(str(x))'` (CPython 3), including the design's
  twelve (`0.5`, `1.0`, `2.0`, `0.25`, `0.3333333333333333`, `1e-05`,
  `0.0001`, `1000000000000000.0`, `1e+16`, `1.5e+16`,
  `1.2345678901234568e+17`, `0.30000000000000004`) and `1e+100`,
  `2.5e-300`, `-0.0`, `1.2345678901234568e+16`.
- One test per refusal-table row (design §1.3):

| row | test |
|---|---|
| `systematics_program = syscalc` | `runcard::tests::syscalc_is_refused_at_integrate` |
| unknown program | `runcard::tests::an_unknown_program_is_refused_at_integrate` |
| `Systematics` with capitals | `runcard::tests::a_capitalised_program_is_refused_at_integrate` |
| `--weight_format`, `--weight_info`, `--remove_wgts`, `--keep_wgts` | `systematics::request::tests::the_naming_and_removal_options_are_refused_as_unimplemented`; at integrate through a card, `runcard::tests::a_refused_argument_on_the_card_fails_at_integrate` |
| `--only_beam` … `--from_card`, any other option, a positional | `systematics::request::tests::options_outside_the_honoured_set_are_refused` |
| no `=`, two `=`, empty value | `systematics::request::tests::a_malformed_argument_is_refused` |
| factor not finite or ≤ 0 | `systematics::request::tests::a_factor_that_is_not_finite_and_positive_is_refused` |
| `--dyn` outside −1, 1–4 | `systematics::request::tests::a_dynamical_choice_without_a_label_is_refused` |
| `--together` name | `systematics::request::tests::an_unknown_together_axis_is_refused` |
| `--start_id` not an integer | `systematics::request::tests::a_start_id_that_is_not_an_integer_is_refused` |
| `id@N`, `sys_pdf` `id N` | `systematics::request::tests::a_member_by_id_and_offset_is_refused`; card: `runcard::tests::a_refused_sys_pdf_fails_at_integrate` |
| entry with whitespace, `&&` piece | `systematics::request::tests::a_pdf_entry_with_whitespace_is_refused` |
| `sys_pdf` starting with a count | `systematics::request::tests::a_sys_pdf_starting_with_a_count_is_refused` |
| `use_syst = T` written on fixed-energy beams | `runcard::tests::use_syst_on_fixed_energy_beams_is_refused_or_resolved_off`; CLI `cli_systematics::use_syst_written_on_fixed_energy_beams_is_refused_at_integrate` |
| two or more PDFs, nominal not first | `systematics::plan::tests::a_list_not_starting_with_the_nominal_is_refused` |
| set not found / consent refused | `systematics::plan::tests::missing_sets_are_asked_for_once_and_a_refusal_stops_the_run` |
| `AlphaS_Type` not `ipol` | `systematics::plan::tests::a_member_whose_coupling_is_not_tabulated_is_refused` |
| `ickkw = 1` with a request | `runcard::tests::a_matched_card_asking_for_weights_is_refused` |
| `generate --systematics`, `use_syst = F` | `cli_systematics::an_override_without_use_syst_is_refused_at_generate` |
| `generate --systematics` on a decay | `cli_systematics::an_override_on_a_decay_is_refused_at_generate` |
| (O3) `--dyn=1`, one-particle final state | `generate::tests::the_sum_et_scale_is_refused_on_a_one_particle_final_state` (unit level only: see Found FB) |
| (F3) reweight id colliding with a systematics id | `generate::tests::a_reweighting_id_colliding_with_a_systematics_id_is_refused` |

- F1/F2: `pdf::grid::tests::a_set_description_is_read_as_yaml` (a four-line
  folded double-quoted `SetDesc` and a single-quoted one with a doubled
  quote), `a_member_header_overrides_the_sets_coupling_and_clamp`,
  `pdf::tests::a_member_reads_its_own_coupling_and_clamp` (synthetic set on
  disk through `PdfSet::member`). On the real
  `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` (fetched to the scratchpad, 45 MB,
  not committed): the ignored
  `validate_systematics::the_pdfas_member_reads_its_own_coupling_and_clamp`:

  ```
  LHAPDF_DATA_PATH=<scratchpad>/S-F1-pdfs cargo test -p vibegraph-lib --features extended-validation \
    --test validate_systematics -- --ignored --nocapture the_pdfas
  NNPDF31_nnlo_as_0118_mc_hessian_pdfas: .info AlphaS_MZ 0.118 ForcePositive 0; member 0 AlphaS_MZ 0.118
    ForcePositive 0; member 102 AlphaS_MZ 0.12 ForcePositive 2 AlphaS_Vals[0] 0.3487554 (set's 0.33074891)
  test the_pdfas_member_reads_its_own_coupling_and_clamp ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.10s
  ```

## Mutations

`S-F1-mut/run.sh` (scratchpad) applies a batch of mutations by exact string
replacement, runs `cargo test -p vibegraph-lib --lib -- runcard systematics
pdf::`, and restores the files (restore checked: no mutated text remains,
`git diff --stat` unchanged at 23 files). Two batches, each of four mutations
in disjoint code, so each failing test is attributed to the one mutation in
its code path. Unmutated: `test result: ok. 151 passed; 0 failed`.

| mutation | where | failing tests |
|---|---|---|
| M1 member header not laid over the `.info` | `pdf/grid.rs` `member_meta` | `pdf::grid::tests::a_member_header_overrides_the_sets_coupling_and_clamp`, `pdf::tests::a_member_reads_its_own_coupling_and_clamp`, `systematics::plan::tests::a_member_whose_coupling_is_not_tabulated_is_refused` |
| M2 Python's exponent threshold `decpt > 16` → `> 15` | `format.rs` | `systematics::format::tests::py_float_matches_cpythons_str` |
| M4 card line split at the first `=` | `runcard.rs` `split_line` | `runcard::tests::a_value_holding_an_equals_sign_reads_whole`, `…the_request_is_read_from_the_entry_point_madgraph_takes`, `…a_refused_argument_on_the_card_fails_at_integrate` |
| M6 quirk-A refusal off (`list_len >= 1`) | `plan.rs` | `systematics::plan::tests::a_list_not_starting_with_the_nominal_is_refused` |
| batch A total | | `test result: FAILED. 144 passed; 8 failed` |
| M3 nominal not re-inserted before `lhaid + 1` | `header.rs` `expand` | `systematics::header::tests::the_default_request_expands_to_145_weights`, `…a_second_slot_of_the_nominal_member_is_not_the_nominal`, `systematics::plan::tests::errorset_reads_the_runs_whole_set_with_the_nominal_first` |
| M5 `<rscale>` power not reduced by the emissions | `weight.rs` | `systematics::weight::tests::the_weight_is_get_lo_wgt_term_by_term` |
| M7 written `use_syst = T` on fixed-energy beams accepted | `runcard.rs` `split_as` | `runcard::tests::use_syst_on_fixed_energy_beams_is_refused_or_resolved_off` |
| M8 one question per download | `plan.rs` `open_sets` | `systematics::plan::tests::missing_sets_are_asked_for_once_and_a_refusal_stops_the_run` |
| batch B total | | `test result: FAILED. 146 passed; 6 failed` |

The per-event gate's three controls are the weight function's own mutation
evidence on MadGraph's data (Gate 3).

## Tests and pins that moved (Q3 and the new shape)

| where | old | new |
|---|---|---|
| `runcard::tests::a_split_card_keeps_madgraphs_record_through_an_encoding` | card `1 = ickkw`, `30 = xqcut`, `45 = ptj` | the same plus `none = systematics_program` (a matched card asking for weights is refused) |
| `lhef::write::tests::mg_run_card_carries_what_set_mad_reads` | matched card with `True = use_syst` | the same plus `none = systematics_program` |
| `runcard::tests::split_hands_the_generate_fields_to_the_settings` | `settings.systematics.fields.len() == 7` | `card_fields.len() == 7`, and `request.mur == [0.5, 1.0]` from `sys_scalefact` |
| `artifact::tests::a_version_13_artifact_is_refused` | `found 13, oldest 14, expected 14` | `a_version_14_artifact_is_refused`: 13 and 14 refused, `oldest 15, expected 15` |
| `runcard::classes::tests` | three warning tests (`a_systematics_request_warns_once_and_a_default_card_does_not`, `systematics_fields_do_not_warn_when_use_syst_is_off`, `every_systematics_field_warns_on_its_own`) | removed with the warning; `the_systematics_request_fields_are_consumed` |
| `cli_reweight_proton` (banked) | event ids `["wz", "nominal", "aew"]` | 145 numeric systematics ids first (the `pp_to_llj_fixed` card asks for them), then `["wz", "nominal", "aew"]` |
| `docs/src/cli/reference.md` | — | `--systematics <ARGUMENTS>` (regenerated by `scripts/gen-cli-docs.sh`; `cli_reference_docs` failed before it) |
| `cli_onshell_veto` (banked), the `uu_*` rows | `onshell_uu_run_card.dat` (writes `True = use_syst`) with the row's `lpp1=0;lpp2=0` overrides | the same plus a `use_syst=False` override on fixed-energy rows: O5 refuses a written `use_syst = T` there (measured: `ERROR run card sets use_syst = 'True': fixed-energy beams carry no parton density to vary …`); MadEvent's cross section does not read it |
| `cli_hard_errors`, `validate_mlm_dumps`, `artifact`, `generate`, `lhef::write` tests | `RunCard::split()` a tuple | `.split().unwrap()` / `.expect(…)`, values unchanged |

Every other proton sample in the banked layer now carries `<mgrwt>` and 145
weights where its card asks for them (MadGraph's banked cards do); the banked
layer below says whether any other gate moved.

## Found

- **FA. `n_qcd` under a dynamical scale is the clustering's graph in MadEvent,
  not the labelled configuration.** `reweight.f:1437` sets `igraphs(1) =
  iconfig` only when `setclscales` did not run (fixed scales); with a
  dynamical scale `igraphs(1)` is the graph the clustering chose. S-D1's
  evidence (13 of 10 000 `pp_to_bb_fixed` events at `<rscale> 0`, every
  dynamical `pp_to_bb*` event at 2) fits that. This build fills `n_qcd` from
  the configuration the event is labelled in (`ProtonSelection::config`, drawn
  ∝ `AMP2`), which on a mixed-order process gives some EW-configuration events
  `n_qcd = 0` where MadEvent writes the QCD graph's 2, and so different
  `μR`-varied weights on those events. The per-event gate cannot see it (it
  reads MadGraph's record). Closes when the unmatched dynamical-scale fill
  takes the clustering's first graph (the scale source already clusters for
  `dynamical_scale_choice = -1`; `EventScales::unmatched` drops the graph) and a
  census of `<rscale>` integers on a regenerated `pp_to_bb` sample matches the
  banked `{2: 10000}`.
- **FB. A `2 → 1` proton process panics instead of being refused.**
  `vibegraph integrate` on `generate p p > z` with MadGraph's banked
  `pp_to_llj_fixed` card stops with `thread 'main' panicked at
  vibegraph-lib/src/phasespace/rambo.rs:52:5: RAMBO needs at least two
  final-state momenta` (backtrace `rambo` ← `proton::derive_flavor_groups` ←
  `multiplicity_groups` ← `integrate_proton`), measured in this session's
  banked-layer run. It predates this work, and it makes the O3 refusal
  unreachable on proton beams for now; when `2 → 1` is supported, MadGraph's
  default request (which holds `--dyn=1`) will be refused there by O3, so
  whether the default should drop choice 1 for such a process is a user
  decision. Closes with a refusal (or support) for one-particle final states
  at proton beams and an end-to-end test of O3.
- **FC. Run-card lines whose value holds `=` were dropped silently.**
  `split_line` split at the first `=`, so MadGraph's own
  `['--mur=0.5,1,2', …] = systematics_arguments` line read as a name with
  spaces and was skipped as structural; every card's list was the table's
  default. Fixed here (split at the last `=`, `banner.py:2902`), with a test;
  no other banked card line holds `=` in its value, but a card-reader audit of
  "skipped as structural" lines would say whether anything else was lost.
- **FD. The run-card concept's defaults list does not say `use_syst`
  resolves off on fixed-energy beams.** Design §5 lists it among Q3's moves;
  it is a knowledge-bundle edit, left for the close-out session.
- **FE. An unmatched sample carries no `<MGRunCard>`,** so MadGraph's own
  `systematics` command cannot be run on a vibegraph file (it reads the
  banner's run card for `lhaid`, `use_syst`, the beams). Closes by writing
  the record on every proton sample under `use_syst`, or by deciding it is not
  wanted.

