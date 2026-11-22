---
type: Session Report
title: "L-T1 report: streamed per-phase timing for the multijet timeout triage"
description: "integrate and generate stream a start and an end phase record, flushed as they happen, so a run killed at the cap names its phase; σ and grids are bit-for-bit against main; three local smoke runs were killed in enumeration, the amplitude probe and VEGAS."
status: draft
tags: [performance, mlm, multijet, cli, ldjson, timing, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Branch `sprintl-timeout-triage` off main b2434a0. No backlog item closes
here. The triage item's `closes_when` and body are amended to v022-1009.
The never-retried list is
[v022-1009-never-retried.tsv](../v022-1009-never-retried.tsv) (178 cards,
from `vg-validator` via the manager).

## What changed

**The instrument** (`vibegraph-cli/src/phase.rs`, wired into
`integrate.rs`, `generate.rs` and `main.rs`). `phase::start(name)` writes a
start record before the phase does any work. The guard's `end()` writes the
end record, and if the guard is dropped on an error return, the end record
says `outcome: failed`. Under `--output-ldjson` each record is a `phase`
object written through `emit_object`, which flushes every line. The record
is written whatever the visible level. In every mode the same record is also
a tracing event: the start at `debug`, the end at `info` with its wall time
and sizes. `--log-file` therefore holds both, and it writes each event
unbuffered. `elapsed_s` is measured on the monotonic clock from an origin
fixed at the top of `main`.

| phase | where | sizes on `end` |
|---|---|---|
| `input` | card or diagram artifact read, model loaded | — |
| `enumerate` | `enumerate_lines_in` + `process_facts` (default run card) or `generate_from_proc_card_in` | `subprocesses`, `diagrams` |
| `amplitude_probe` | `amplitude_probe::measure`: each set compiled and evaluated | `diagram_sets` |
| `pdf` | proton beams: `load_for_run_card` | — |
| `compile` | `multiplicity_groups_of` (proton) / `compile_subprocesses` + vetoes + cuts + pairing amplitudes (fixed energy) | `subprocesses`, `compiled_diagrams`, `helicities`, `colour_flows`; proton adds `parts`, `flavour_groups` |
| `channels` | maps, bound amplitudes, integrand construction | `channels` |
| `alpha_survey` | `adapt_alphas` (proton) / `use_multichannel` (fixed energy, which also builds the per-diagram maps) | `points_per_iteration`, `iterations` |
| `vegas` | `adapt_grids_budget`, warm-up included | `iterations`, `points` |
| `configurations` | per-configuration shares, channel records | — |
| `bank` | artifact assembled and written | — |

`generate` has `input`, `pdf` and `generate`. The library's `generate()`
contains the probe replay, the compile, the weight scan and the unweighting,
and its existing `progress` stages resolve those further. Splitting them
into phases would need a hook in the library. I left that out of scope
(see Found).

**Schema.** `phase` is a new `kind` under `schema_version` 1. The module doc
in `output.rs` now says that adding a kind keeps the version and that a
reader skips kinds it does not know. The docs section is its own
`### Phase timing` subsection in `docs/src/cli/overview.md`, so it does not
collide with PR 8's field additions. `cli_output_ldjson.rs`'s closed kind
list admits `phase`.

**Evaluator nodes** are not in `sizes`. `AmplitudeEvaluator` exposes no
node count, and the helicity-expanded arena is built lazily on first
evaluation. Diagrams, helicities and colour flows are the cheap proxies
(see Found).

**Item.** `multijet-mlm-cards-time-out-in-the-gridpack-campaign` now has a
title, description, `closes_when` and body for v022-1009: (a) to (d) as the
brief asks, with "one from each group of 'other'" added per the manager.

**Classification: order-preserving.** The instrument draws no random number
and reads or changes no input of a phase. It only adds writes to stdout and
the log. Gate: bit-for-bit.

## Oracles

**Phase stream tests**, `vibegraph-cli/tests/cli_phase_timing.rs` (default
suite):

```
$ cargo test -p vibegraph -j 2 --test cli_phase_timing
test a_check_run_stops_after_the_channels ... ok
test a_refusal_ends_its_phase_as_failed ... ok
test a_run_killed_in_a_phase_leaves_its_start_record_on_disk ... ok
test a_fixed_beam_integration_streams_its_phases_in_order ... ok
test a_proton_integration_streams_its_phases_in_order ... ok
test result: ok. 5 passed; 0 failed
```

- Order, fixed-beam: `e+ e- > mu+ mu-`. The records are start/end pairs in
  the exact 9-phase sequence, with `elapsed_s` non-decreasing across all of
  them and `wall_s` = end − start (1e-9). Every record precedes `result`.
  Sizes are pinned: 1 subprocess, 2 diagrams, 2 channels, 4 iterations. The
  test then runs `generate` from the artifact and checks `input`, `pdf`,
  `generate` with `events_written` 50.
- Order, proton: `p p > e+ e-` on `pp_to_llj_mlm_run_card.dat`, with the
  banked NNPDF set. The 10-phase sequence is checked, plus
  `subprocesses ≥ flavour_groups ≥ 1` and `channels ≥ 1`.
- Killed: an endless fixed-budget run with stdout redirected to a **file**,
  as a batch system does. The test polls the file for the `vegas` start,
  waits 200 ms, sends SIGKILL and confirms the exit was by signal 9. The last
  `phase` record on disk is the `vegas` start, no `result` or `refusal` was
  written, and the six phases before it are complete pairs.
- Failed: `e+ e- > e+ mu-`. The last record is an `end` with
  `outcome: failed`, written before the `refusal`. Observed directly (the
  `enumerate` phase):
  `{"event":"end","outcome":"failed","phase":"enumerate",...}` then
  `{"error_kind":"process","kind":"refusal","message":"failed to enumerate process: no diagrams for 'e+ e- > e+ mu-' ..."}`.
- `--check` stops after `channels` (5 pairs).

**Hermetic suite** on f809054 (dev profile):

```
$ cargo test --workspace -j 2   # lt1-hermetic.log
passed 2248 failed 0 ignored 26 (70 test binaries), rc=0
```

`cargo fmt --all --check` passes, and
`cargo clippy --workspace --all-targets -- -D warnings` returns rc=0 with no
warnings. `pixi run kb-lint` reports 0 errors (the 5 warnings were there
before this session, all on other items).

**Bit-for-bit, main against branch.** Main b2434a0 was built in a separate
detached worktree (`/home/user/wt/lt1-mainref`, its own target, removed
afterwards). The branch is f809054. Both were dev profile, same cards,
`--fixed-budget --neval 20000 --niter 5 --seed 1 -j 2 --output-ldjson`:

```
ee  main   sigma,err,bits=0.9018543676344345 0.0032574186962462275 3fecdbfdb0d7d43b grid_sha256=8f32a32446343b72f71edddadabef3123cd511545d61c00c7864326f16e800d5 phase_records=0
ee  branch sigma,err,bits=0.9018543676344345 0.0032574186962462275 3fecdbfdb0d7d43b grid_sha256=8f32a32446343b72f71edddadabef3123cd511545d61c00c7864326f16e800d5 phase_records=18
llj main   sigma,err,bits=270.3021332593776 1.615785707658493 4070e4d589af40f9 grid_sha256=73c3df1c13e8f26c8d7728437a41c01cbc439bdc74bfd34a8c76f3291b7f3364 phase_records=0
llj branch sigma,err,bits=270.3021332593776 1.615785707658493 4070e4d589af40f9 grid_sha256=73c3df1c13e8f26c8d7728437a41c01cbc439bdc74bfd34a8c76f3291b7f3364 phase_records=20
```

`ee` is fixed-beam `e+ e- > u u~ g` at 2 × 125 GeV. `llj` is
`p p > e+ e- j` on the banked MLM card `pp_to_llj_mlm_run_card.dat`
(ickkw 1, xqcut 20, lhaid 247000). σ, its error, and the whole artifact are
identical byte for byte.

## Local smoke runs

Profile: **dev, opt-level 2, no LTO**. Absolute phase times, and the
sampling share in particular, will differ from the validator's release
binary. Command, one card at a time:

```
timeout -s KILL 300 vibegraph integrate <proc> --run-card <run> --out <dir> \
  --neval 10000 --niter 3 --fixed-budget --seed 1 -j 2 --no-network \
  --output-ldjson --log-file <card>.log > <card>.ldjson
```

genproductions was not reachable from this session. A guessed
`raw.githubusercontent.com` path returned 404, and attaching the repository
was denied. I therefore wrote the cards myself in the campaign's shape:
`sm-no_b_mass` (5-flavour `p`), 2 × 6800 GeV, lhaid 247000 standing in for
the campaign's 325300, and MLM with ickkw 1, xqcut 10, ptj 10.

| card | last phase (killed at 300 s) | evidence from the stream |
|---|---|---|
| W(→ℓν) + 0–4 j MLM, both charges, 10 lines | `enumerate` (started 0.02 s, no end) | last progress: line 8 of 10 (ℓ⁻ν̄ + 3 j), 905 132 of 1 449 459 assignments; lines 9–10 (4 j) not reached |
| Z(→jj) + 0–4 j MLM, decay-chain form | `amplitude_probe` (started 106.37 s, no end) | `enumerate` ended at 106.35 s: 674 705 diagrams, 3725 subprocesses (Zto2Q-4Jets at 202a540 had 674 680); 194 s in the probe without finishing |
| ssWW VBS, `p p > w+ w+ j j QCD=0, w+ > ell+ vl` | `vegas` (started 50.54 s, no end) | enumerate 0.38 s (72 subprocesses, 6966 diagrams); amplitude_probe 12.54 s; compile 12.10 s (28 flavour groups in 3 parts, 2580 compiled diagrams); channels 0.05 s (430); alpha_survey 25.45 s; VEGAS warm-up 1: 221 631 points at 81.6 µs/eval; warm-up 2: 853 921 points at 211.8 µs/eval, killed in iteration 3 |
| WZ VBS to four quarks (`w+ > j j, z > j j`) | refused in `enumerate` (0.75 s) | `outcome: failed` then the refusal "decay chain ... identical particles between the decay and the rest of the process; which of them is on shell is ambiguous". My card's spelling, not the campaign's; replaced by the ssWW card |

So the instrument resolves the question. Three cards were killed in three
different phases, and each stream names its phase and the sizes behind it.
Two observations, both under the dev-profile caveat:
- The amplitude probe costs as much as the compile it duplicates: 12.5 s
  against 12.1 s on ssWW, and on Z+4j it alone takes more than 194 s.
- Enumeration alone exceeds 300 s on W+4j at `-j 2`.

## What each test cannot detect

- **Order tests**: a phase that is timed but placed wrongly, for example
  work that happens before a start record or after an end, so that it is
  billed to the neighbouring phase. They check order and pairing, not what
  falls inside each pair. The table above comes from reading the code. Two
  cases are untimed: the work in `integrate` between `bank`'s end and the
  `result` (the `result_object` build), and the PDF fetch inside `generate`,
  which is billed to `pdf`.
- **Kill test**: a kill inside a phase other than `vegas`, because earlier
  phases are too short to land a signal in on a test card. The flush path is
  the same `emit_object` for every phase, so the gap is one of timing, not
  of mechanism. It also cannot detect a buffered `stdout` that a pipe
  (rather than a file) would hide differently. The test uses a file because
  that is the campaign's case.
- **Failed-outcome test**: only one phase (`enumerate`) is exercised. A
  refusal that `return`s without dropping the guard is impossible in Rust,
  but a `process::exit` (the panic hook) writes no end record, and none is
  claimed.
- **Bit-for-bit**: two cards, one seed, `-j 2`. It cannot see a change that
  only appears on paths these cards do not take (decays, mixed multiplicity,
  `generate`). The instrument has no data dependence on any of them, which
  is an argument and not a measurement.
- **Smoke runs**: the cards are mine, not the campaign's. Model restriction,
  PDF, cuts and spellings differ, so the phases they name are evidence that
  the instrument works, not triage verdicts.

## Validator request (draft; the manager sends it)

- **Build**: branch `sprintl-timeout-triage` at the head this report is
  committed on, **release** build (`cargo build --release -p vibegraph`).
- **Command**: each card's own v022-1009 `integrate` command line, unchanged
  (seed 1, its budget flags), with `--output-ldjson --log-file <id>.log`
  added and stdout redirected to `<id>.ldjson`. Run under the espresso cap,
  `timeout -s KILL 1200` on 4 CPUs, one card at a time. One seed is enough:
  the phase a card dies in does not depend on the seed except within
  `vegas`. If cheap, wrap each run in `/usr/bin/time -v` for peak RSS.
- **Return per card**: the full `<id>.ldjson` stream, `<id>.log`, the exit
  status (137 = killed at the cap), wall seconds, and peak RSS if taken.
- **Expected answer**: per card, the last `phase` record (an unpaired
  `start` is the phase it was killed in) with the sizes of the phases that
  ended. Per family, the dominant phase.
- **Sample** (21 cards, at most 7 h at the cap):

| family | card_id | gridpack |
|---|---|---|
| W/Z/DY+4j | 58 | WtoLNu-4Jets_MLNu-120to200_madgraphMLM-pythia8 |
| W/Z/DY+4j | 65 | Zto2Q-4Jets_Bin-HT-800to1500_madgraphMLM-pythia8 |
| W/Z/DY+4j | 15 | DYBto2LB-4Jets_MLL-120_HT-2500_madgraphMLM-pythia8 |
| W/Z/DY+4j | 1009 | ZJetsToNuNu_HT200to400 |
| VBS | 133 | VBS_WPMZTo4Q_5f_LO_EWK |
| VBS | 14 | WWjj_SS_ewk |
| VBS | 744 | ZNuNuWJJjj_QCD_LO |
| VBS | 710 | VBS_WZ_PolarizLL_WtoLEP_ZtoHAD |
| VBF resonances | 290 | VBFWto2Q_madgraph-pythia8 |
| VBF resonances | 636 | VBFRadionTo2W_Inclusive_narrow_Par-M1800GeV |
| VBF resonances | 655 | VBFWprimeToWZ_Inclusive_narrow_Par-M1200GeV |
| GJ-4Jets | 32 | GJets-4Jets_dRGJ-0p2_HT-100to200_madgraphMLM-pythia8 |
| GJ-4Jets | 25 | GJ-4Jets_Bin-HT-1000-PTG-200_Par-dRGJ-0p25_madgraphMLM-pythia8 |
| TT-3Jets | 42 | TT-3Jets_Bin-HT-800to1500_madgraphMLM-pythia8 |
| TT-3Jets | 46 | TTto4Q-3Jets_madgraphMLM-pythia8 |
| aQGC | 196 | aQGC_WPJJWPJJjj_EWK_LO_NPle1 |
| aQGC | 195 | aQGC_ZlepZhadJJ_EWK_LO_SM_mjj100_pTj10 |
| other: QCD multijet MLM | 37 | QCD-5Jets_HT-40_madgraphMLM-pythia8 |
| other: older GJets | 992 | GJets_HT-200to400 |
| other: TT FCNC | 590 | TT-TbartoUbarZ-TtoLNu-kzut |
| other: VBS-like madspin/QCD | 722 | ZZ_TT_madspin |

The larger families get 3–4 cards and span their sub-shapes: lepton vs
hadronic V, b-enriched, polarised vs inclusive VBS, Radion vs W′.

## Brief corrections

- **Cap**: espresso is 20 min wall on 4 CPUs, not 600 s (the 600 s was
  202a540's). The manager corrected this.
- **Family counts**: the brief's counts (VBS 32, GJ 13, W/Z/DY ~40, VBF 20,
  TT 5) were preliminary. The list's counts are W/Z/DY+4j 56, VBS 47, VBF
  22, GJ 19, other 17, TT 10, aQGC 7.
- **genproductions over raw.githubusercontent.com** was not usable from
  this container (see Local smoke runs).
- **"Evaluator nodes when cheaply known"**: they are not exposed, see above.
- **"`generate` where it shares them"**: `generate` shares compilation and
  channel construction with `integrate` only inside the library, so its
  phases are coarse.
- The census the item linked (202a540, 37 timeouts) is superseded by
  v022-1009 for this triage. The item keeps the link as history.

## Found

- **Every diagram set is compiled twice per `integrate`, and again in
  `generate`.** `amplitude_probe::measure` compiles each set
  (`amplitude_probe.rs:262`), and `derive_flavor_groups` compiles each again
  (`proton.rs:1113`). `generate` compiles in `check_amplitude_probe` and
  again in `generate_sample`/`generate_proton_sample`. Evidence: ssWW
  `amplitude_probe` 12.54 s against `compile` 12.10 s, and Z(→jj)+4j spent
  more than 194 s in the probe. Fix: evaluate the probe on the evaluators
  the compile phase builds, or cache by set, and prove it with σ and probe
  values bit for bit.
- **The ssWW VBS warm-up grows from 221 631 to 853 921 points, and its
  ns/eval from 81.6 to 211.8 µs.** That is under `--neval 10000
  --fixed-budget`, with 430 channels at the per-channel floor. A 3-iteration
  campaign budget then spends minutes in two warm-ups. Fix: account for the
  warm-up schedule and the per-eval cost on VBS cards in the validator's
  timed runs. This may be the `vegas` dominant cause for the VBS family.
- **No evaluator node count is exposed.** `AmplitudeEvaluator` has no node
  or op count accessor, so `compile` sizes stop at diagrams, helicities and
  flows. Fix: a cheap accessor on the folded arena, added to the `compile`
  sizes.
- **`generate`'s phases are coarse.** Fix: a phase hook in
  `GenerateOptions` (like `on_scan`), so the probe replay, compile, scan and
  unweight are separate phases.
- **Peak memory is not recorded.** A card killed for memory and a card
  killed at the cap look alike in the stream. Fix: a `max_rss` field on end
  records (getrusage), or `/usr/bin/time -v` on the validator side as
  requested above.
- **Three v022-1009 "error" cards are model refusals at setup**, not
  timeouts: 443 HSCP-MCP_Par-S-0-M-700-Q-6, 444
  HSCP-MCP_Par-S-0-M-1800-Q-15 and 782 VLLD_tau_M100. Their gridpacks
  bundle no UFO model, so they are outside the timeout scope.
- **Enumeration alone exceeds 300 s on W(→ℓν)+0–4 j** (dev profile,
  `-j 2`). It was still on the 3-jet ℓ⁻ line when killed. If the
  validator's release runs agree, this is the W/Z/DY family's dominant
  cause.
