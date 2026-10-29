---
type: Session Report
title: "S-F2 report: systematics weights for MLM events, FA and FE"
description: "<mgrwt> on matched events from rewgt, the matched refusal lifted, n_qcd from the clustering graph, <MGRunCard> under use_syst; MadGraph's own systematics.py (with LHAPDF) runs on vibegraph's files and agrees on every weight, vibegraph reproduces MadGraph's module on all 5.88M weights of the four banked MLM runs, the dump-based record check written but not runnable here, and an event-file replay of the banked records in its place."
status: draft
tags: [systematics, mlm, report, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
Worktree `/home/user/wt/S`, branch `sprinte-systematics`, from `7290ed09`.
Implements the [S-F2 brief](S-F2.md): design §8's S-F2 scope of
[the systematics weights](../../../events/systematics-weights.md), with Found
FA and FE of the [S-F1 report](S-F1-report.md). Logs are in the session's
scratchpad (`S-F2-*`); commands and output tails are below.

**The brief's gate could not be run as written.** The dumped MLM runs are not
in refdata-11 (see "Corrections to the brief"), so §6.3's record check is
implemented in `validate_mlm_dumps` and *not run*. In its place this session
measured three things the brief did not ask for, each recorded below:
MadGraph's own `systematics.py` (with a real LHAPDF 6.5.4 built here) on
vibegraph's files and on the banked MLM files, and a replay of every banked
MLM `<mgrwt>` record from the event files alone.

## What was implemented

- **The record from `rewgt`** (`systematics/record.rs`): `RecordInputs` and
  `LoRecord::fill`, the one filler for matched and unmatched events, per
  clustering beam: each chain starts at the incoming parton (`idup(j,
  IPSEL)`, `xbk(ib(j))`, `√q2fact(j)` of the last `setclscales` call) and
  gains one entry per *density ratio* `rewgt` took on that beam (flavour of
  the line entering, its fraction recomputed as `x/z` as `reweight.f:1719-1723`
  does, `√q2now`); one emission scale per `αs` ratio in vertex order;
  `<totfact> = rewgt × f₁f₂ × asref^n_qcd` (`asref = 0` unmatched); each chain
  written under its physical beam (`unwgt.f`'s `flip`).
- **`rewgt`** (`coupling/cluster/rewgt.rs`): `PdfStep::x_entering`.
- **Clustering outputs** (`coupling/scales.rs`): `EventScales::graph` (the last
  clustering's first graph, matched or not); `MatchedRecord` carries the
  second call's `q2fact` and `μR`.
- **Generation** (`proton.rs`, `generate/source.rs`, `generate.rs`): each
  term's `RewgtHistory` is kept beside its record (moved, not cloned); the
  drawn member's `Rewgt` is recomputed once per accepted event
  (`ProtonSelection::rewgt`); `RecordFill::record` fills matched and unmatched
  events; `<mgrwt>` on every proton event under `use_syst`, matched included,
  also under `systematics_program = none`.
- **The matched refusal lifted** at integrate (`runcard.rs`) and generate
  (`generate.rs`).
- **FA**: `n_qcd = nqcd(graph)` wherever the scale was clustered, the
  labelled configuration's otherwise.
- **FE**: `<MGRunCard>` in the header of every sample under `use_syst`.
- **Docs**: `docs/src/guide/11-unweighting-events.md`; the `use_syst` field
  class reason (`runcard/classes.rs`).
- **Tests**: unit tests per rule (below); `validate_mlm_dumps` gains
  `compare_mgrwt` (the §6.3 record check, with two controls); new banked
  target `validate_mlm_records` (the event-file replay); `validate_systematics`
  gains the ignored `madgraphs_module_on_other_files` and a `NoMlmTerms`
  control; `cli_generate_proton` gains `a_matched_sample_carries_its_mgrwt_record`,
  `matched_systematics_weights_round_trip_and_alps_moves_only_emitting_events`,
  an `<MGRunCard>` assertion in the unmatched round trip, and two `<mgrwt>`
  fractions in `matched_sample_record_fractions_against_madevent`.

## Corrections to the brief

1. **"the dumped MLM runs that refdata-11 carries"** is wrong. refdata-11
   carries the four runs' banked event files (`Events/run_01`), whose
   `<mgrwt>` is the oracle, but neither the per-event dumps
   (`validation/madgraph/output/ktdump/dumps/*.jsonl.gz`, outside the bundle
   by design: `pixi.toml`'s `validate-mlm-dumps` comment, `mlm_dump_manifest.json`)
   nor the process directories' `configs.inc`, `config_nqcd.inc`,
   `nexternal.inc` and `config_subproc_map.inc` that the replay reads. This
   container has no `gfortran`, and a regenerated dump must reproduce the M3's
   event file byte for byte. So the §6.3 record check (`compare_mgrwt` in
   `validate_mlm_dumps`) is written and compiles, but was **not run**; it runs
   under `pixi run validate-mlm-dumps` on the M3.
2. **FA's premise does not hold in MadEvent.** `cluster.f:360` (`filmap`)
   builds the clustering's graph table only from configurations with the same
   `nqcd` as the job's channel, so `nqcd(igraphs(1)) = nqcd(iconfig)` on every
   event; this crate's table does the same (`graph.rs:273`). The S-D1
   evidence is statistics, not the rule: `pp_to_bb_qcd2` (dynamical) has σ =
   3.76e8 pb against `pp_to_bb_fixed`'s 2.14e6 pb (banked `results.dat`), so
   the electroweak channels' ~2.8e3 pb is 7e-6 of the dynamical sample, 0.07
   events in 10 000, against 13 in the fixed one. FA was implemented as asked
   (it ties `n_qcd` to the configuration the scale was clustered in, which is
   MadEvent's one channel), but the census cannot tell the two rules apart:
   the discriminating evidence is the unit test.
3. **"Show the census {2: 10000}"**: shown (below), with the caveat above.

## FA: the `<rscale>` census

`S-F2-samples.sh`: `vibegraph integrate` of `generate p p > b b~ QCD=2` on the
banked `pp_to_bb_qcd2` card (`--fixed-budget --neval 100000 --niter 6`), then
10 000 events:

```
rscale census (vibegraph pp_to_bb_qcd2):  10000 2
banked:                                   10000 2
```

The test that fails under the old rule:
`generate::source::tests::the_qcd_power_is_the_clustered_graphs_wherever_the_scale_was_clustered`
(an event clustered in the QCD configuration, labelled in the electroweak one:
`n_qcd` 2; with no clustering, the label's 0). Mutation M5 below.

## FE: MadGraph's `systematics.py` on vibegraph's files

LHAPDF 6.5.4 built from source in the scratchpad (the tarball's `lhapdf.cpp`
does not compile against Python 3.12; regenerated with Cython 3.0.12), and
MadGraph 3.7.1's module from `research/refs/mg5amcnlo` called through
`call_systematics` (`S-F2-mg/run_systematics.py`). vibegraph's weighted files
had their `<initrwgt>` and `<rwgt>` stripped, MadGraph's module was run on the
rest, and every weight compared by id as printed (`S-F2-mg/compare.py`,
`S-F2-fe.log`):

| vibegraph file | MadGraph arguments | weights | equal | 1 unit | beyond | `<initrwgt>` |
|---|---|---|---|---|---|---|
| `pp_to_llj_fixed` card, 500 unmatched events | `--from_card=internal` (reads our `<MGRunCard>`) | 72 500 | 72 492 | 8 | 0 | identical |
| `pp_to_llj_mlm` card, 1000 matched events, `--systematics` default + `--alps=0.5,1,2` | the same arguments | 147 000 | 146 973 | 27 | 0 | identical |
| `pp_to_llj_mlm` card, 1000 matched events, `systematics_program = none` | `--from_card=internal` | 145 000 written, rc 0 | | | | |

Worst relative deviation before rounding 2.97e-8 and 4.04e-8; the `<mgrwt>`
blocks pass through MadGraph's module unchanged (1000/1000, 500/500).

## MadGraph's module on the banked MLM runs

The same module, over the four banked MLM event files (no weights banked),
with `--mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset --alps=0.5,1,2`
(`S-F2-mgbanked.sh`); then this crate's `Writer` on MadGraph's records:

```
SYSTEMATICS_FILES=<the four files> SYSTEMATICS_ARGS='--mur=0.5,1,2 --muf=0.5,1,2 --pdf=errorset --alps=0.5,1,2' \
  target/debug/deps/validate_systematics-… --ignored --nocapture madgraphs_module_on_other_files
pp_to_llj_mlm.lhe        weights 1470000 exact 1470000 one-unit 0 beyond 0 (100.00000% exact); worst alps 2.489e-8, dyn 3.930e-8, …, pdf 4.194e-8
  --alps: 8877 of 10000 events give it something to vary, and it moves every --alps weight on 8877 of them; it leaves every one at XWGTUP on 1123 of the other 1123
  control MLM terms dropped: DETECTED (max 2.610e0)
pp_to_ll_0j2j_mlm.lhe    weights 1470000 exact 1470000 …; --alps: 3527 / moves 3527 / still 6473 of 6473; control DETECTED (max 2.721e0)
pp_to_ttx_0j1j_mlm.lhe   weights 1470000 exact 1470000 …; --alps: 5253 / moves 5253 / still 4747 of 4747; control DETECTED (max 1.438e1)
pp_to_llj_xqcut_only.lhe weights 1470000 exact 1470000 …; --alps: 0 / still 10000 of 10000; control blind (max 4.682e-8)
test result: ok. 1 passed; … finished in 6.65s
```

**5 880 000 of 5 880 000 weights print MadGraph's string**, the emission and
chain terms included (S-F1's `lo_weight` terms "gated only through S-F2"), and
§6.3's `--alps` expectation holds strictly in both directions. This is not the
M3 CMS-arguments gate (S-V1's): one set, default arguments.

## The record: `validate_mlm_records` (event-file replay)

Each banked event's record rebuilt with the production clustering on this
crate's own forests of its subprocess, the channel searched (configurations
reproducing `SCALUP`, `AQCDUP` and `pt_clust`), the beam order and `xbk` read
off MadEvent's record (the written energies are `xbk·ebeam` only to ~2e-7), and
each field compared at one unit of `E15.8` (`<totfact>` at 1e-6):

| field | `llj_mlm` | `llj_xqcut_only` | `ttx_0j1j_mlm` | `ll_0j2j_mlm` |
|---|---|---|---|---|
| compared / no candidate | 9984 / 16 | 10000 / 0 | 9947 / 53 | 9905 / 95 |
| `<rscale>` power (gated) | 9984 | 10000 | 9947 | 9905 |
| `<rscale>` scale | 9984 | 10000 | 9947 | 9901 |
| `<asrwt>` | 9966 | 10000 | 9888 | 9884 |
| beam 1 codes / x / q | 9984 / 9700 / 9967 | 10000 ×3 | 9842 / 9746 / 9917 | 9895 / 9767 / 9881 |
| beam 2 codes / x / q | 9984 / 9682 / 9967 | 10000 ×3 | 9838 / 9762 / 9022 | 9896 / 9765 / 9881 |
| `<totfact>` (worst) | 9984 (1.6e-7) | 10000 (1.9e-7) | 7523 (1.0) | 9891 (0.97) |

Gated: the power everywhere, every field on the unmatched row. The rest is
informational, because the replay is not yet told apart from its own inputs:
the `llj_mlm` `asrwt`/`x` misses are two or three units of the eighth digit
(e.g. `0.33467570E+02` against `0.33467573E+02`); one `0j2j` miss is a tie of
two chain scales (`0.13921602E+03` twice, a ratio taken here and not by
MadEvent); codes differ on 10 `0j2j` and about 105 `ttx` events, and the
one inspected records the group representative's `u` codes for a written `c`
event (Found FI); and on `ttx_0j1j` 9% of events seed beam 2's chain at the core scale here
and at beam 1's emission scale in MadEvent (Found FG). `<totfact>` on
`llj_mlm` agrees on every event: the matched factor's rule holds.

## Gates

(Filled from `S-F2-banked/status.txt` below.)

## Mutations

`S-F2-mut/run.sh` applies batches by exact string replacement, runs
`cargo test -p vibegraph-lib --lib -- systematics::record coupling::cluster::rewgt
coupling::scales generate::source runcard`, restores (`restore check: diff stat
unchanged`). Unmutated: `test result: ok. 102 passed; 0 failed`.

| mutation | failing tests |
|---|---|
| M1 chain records `x` after `z` | `record::…a_matched_record_is_what_rewgt_left_in_its_buffers`, `…an_exchanged_event_writes_each_chain_under_its_physical_beam` |
| M5 FA reverted (`selection.config`) | `generate::source::tests::the_qcd_power_is_the_clustered_graphs_wherever_the_scale_was_clustered` |
| M7 unmatched clustering drops its graph | `coupling::scales::tests::xqcut_clusters_a_fully_fixed_card` |
| M8 matched refusal reinstated | `runcard::tests::a_matched_card_asks_for_weights_as_any_card_does` |
| batch M1 M5 M7 M8 | `FAILED. 97 passed; 5 failed` |
| M2 every chain step recorded, not only ratios | `record::…a_matched_record_is_what_rewgt_left_in_its_buffers` |
| M6 `x_entering` not divided by `z` | `rewgt::tests::a_single_isr_vertex_carries_one_alpha_s_and_one_density_ratio` |
| batch M2 M6 | `FAILED. 100 passed; 2 failed` |
| M3 chain under its clustering beam, not its label | `record::…an_exchanged_event_writes_each_chain_under_its_physical_beam` (`101 passed; 1 failed`) |
| M4 `<totfact>` without `asref^n` | `record::…a_matched_record_is_what_rewgt_left_in_its_buffers` (`101 passed; 1 failed`) |

FE has no unit mutation; the unmatched round trip asserts `<MGRunCard>`
(`use_syst`, `lhaid`, `lpp1`) and MadGraph's module reads it (above).

## Found

- **FF. The §6.3 record check needs the M3.** `compare_mgrwt` in
  `validate_mlm_dumps` was not run (no dumps, no `configs.inc` in the
  bundle). Closes with `pixi run validate-mlm-dumps` on the M3, every
  `mgrwt:` field at one unit and `<totfact>` at 1e-7, both controls printed.
- **FG. `pp_to_ttx_0j1j_mlm`: beam 2's chain seed scale differs on 9% of the
  replayed events** (9022/9947): MadEvent seeds a single-entry gluon chain at
  the other beam's emission scale, the replay at the core's (e.g. `0.30365287E+02`
  against `0.24361550E+03`), and `<totfact>` follows (7523/9947). The dumps
  manifest records the production second call agreeing with MadEvent's
  `q2fact` on every `ttx` event with MadEvent's forests, so this is either the
  replay's channel choice or this crate's own `g g > t t~ g` forests walking
  `jfirst` differently. Closes with the dump check (FF) on the row; if it
  passes, with this crate's forests compared to `configs.inc` for that
  subprocess.
- **FH. `generate` refuses the `pp_to_ttx_0j1j_mlm` card**: `ERROR the
  resonance 2 of g u > t t~ u has no flavour in the subprocess with outgoing
  [6, -6, 1] that shares its matrix element` (`group_resonances`,
  `generate.rs:1214`; integrate succeeds, σ = 1098.3 ± 2.9 pb at a smoke
  budget). Code untouched here; not checked on `main`. Closes with a matched
  `t t~ j` sample generated and a test.
- **FI. MadEvent's record can name the group representative's flavours**: on
  an inspected `pp_to_ll_0j2j_mlm` event the written event is `c c̄ → e⁺e⁻ c c̄` and
  `<pdfrwt>` says `2`/`-2` (`<totfact>` 43.8 against 6.3 for the written
  flavours), so MadGraph's weights there are computed for a `u` density. This
  crate writes the drawn member's codes. The other code misses of the replay
  (10 on `0j2j`, about 105 on `ttx`) are not inspected. Closes with a census
  of record-against-written codes over the banked runs and a line in the
  design's quirk table.
- **FJ. `ickkw = 0`, `xqcut > 0` and every scale fixed**: `setclscales`
  returns at `reweight.f:1102` before setting `n_qcd` and `s_scale`, so
  MadEvent writes the previous event's values; this crate writes the
  clustering graph's power and the fixed scale. Unbanked; a design-table line.
- **FK. Knowledge bundle**: `events/mlm-matched-event-record.md:44` says
  `<mgrwt>` is not written; the design's §2.5 quirk D and §6.3/§8 presume the
  dumps are banked and that `n_qcd` can differ from the channel's under
  clustering (correction 2); the run-card concept's `use_syst` entry gains
  `<MGRunCard>`. Close-out session.
- **FL. Banking MadGraph's module output for the MLM runs**: the four weighted
  files above are scratch; banked, `madgraphs_module_on_other_files` would gate
  the MLM terms in CI without the M3's CMS files.
