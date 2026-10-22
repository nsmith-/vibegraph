---
type: Session Report
title: "V1 report: the 1->n rule moves no decay the suite computes, and MadGraph's banked bounds agree"
description: "Every top-level decay the suite computes keeps its diagram set under the 1->n rule (only F1's own z > u u~ d d~ moves, 4 -> 17, with no MadGraph reference); the search's bound matches MadGraph's recorded WEIGHTED<=N on every census card that generates; the final-state-gluon removal changes an outcome on SMEFTsim (e+ e- > z g g, pinned as an ignored test); the missing-coupling_orders.py fallback changes nothing in reach."
status: draft
tags: [diagrams, coupling-orders, weighted, madgraph-parity, sprint-d, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: 2e6c2ca, resource: "https://github.com/nsmith-/vibegraph/commit/2e6c2ca", title: "test(diagrams): pin MadGraph's gluon-free search trial as an ignored case"}
---
Host: the shared 4-core cloud container (load 11-14). Worktree `/home/user/wt/E`,
branch `sprintd-weighted-cap`, verified before any action (toplevel
`/home/user/wt/E`, branch `sprintd-weighted-cap`, HEAD `5339576`). MadGraph read
at `b7687064b9a013317ca164aa1395bc9c0e39ae1e` (raw.githubusercontent).

## Method

Diagnosis by instrumentation, nothing of it committed. A four-line `eprintln!`
patch on `generate_undecayed` printed, per call, the process, `n_in`,
`is_decay_chain`, whether the search ran, its start, its cap and the bound it
stopped at (`found=` inside the loop, `atcap=` after it). Two scratch test files
ran every card of the three census references that bank MadGraph's bound
(`decay_chain_census.json`, `polarization_census.json`,
`schannel_census.json`, 118 cards), every top-level decay the suite computes, the
banked chain cards and the Found 1/2 candidates through it. The patch was then
reverted with `git apply -R` and the file touched; the probes were moved out of
the tree. Probe sources and logs:
`/tmp/claude-0/-home-user-vibegraph/5fc875eb-a86f-52ce-bbc1-f353ce6c8fb0/scratchpad/wcapE-V1-probes/`
and `.../scratchpad/wcapE-V1-*.log`.

One fact carries the old-versus-new comparisons: for a fixed process, the
diagram set at `WEIGHTED<=w` is the set of diagrams with WEIGHTED at most `w`,
so the sets at two bounds are nested, and **equal counts mean equal sets**.
The "old" column below is the pre-F1 loop emulated with explicit bounds: the
first `w` from `(n-2)*min(hierarchy)` to `(n-2)*max(hierarchy)` with any
diagram, i.e. what the old code returned.

## 1. Decays under the 1->n rule

No decay the suite computes carries explicit coupling orders, and none moves.
Every top-level decay was generated at HEAD (no search: all orders) and with the
old loop (`probe1b.log`, `DECAY [...] new=... old=...`):

| Decay | Where the suite computes it | Top-level / chain | Old | New | MadGraph's reference |
|---|---|---|---|---|---|
| `t > b e+ ve` | `decay_amplitudes.json`, `decay_width_reference.json` (t_bev, t_bev_cuts), `onshell.rs`, `diagrams.rs` | top | 1 | 1 | SMATRIX table (decay_widths.rs, passes); schannel_census `decay_t_bepve`: orders `{}`, 1 diagram |
| `h > e+ e- mu+ mu-` | `decay_amplitudes.json`, `decay_width_reference.json` (h_4l, h_4l_cuts), `diagrams.rs`, `renumbering.rs` | top | 1 | 1 | SMATRIX; schannel `decay_h_zz`: `{}`, 1 |
| `z > e+ e- mu+ mu-` | `decay_amplitudes.json`, `diagrams.rs` | top | 8 | 8 | SMATRIX (8 interfering diagrams) |
| `t > b e+ ve a` | `decay_amplitudes.json`, `diagrams.rs` | top | 4 | 4 | SMATRIX |
| `t > w+ b` | `decay_width_reference.json` (t_wb), `sm_decay_widths.json`, `diagrams.rs` | top | 1 | 1 | decays.py width |
| `z > e+ e-` | `decay_width_reference.json` (z_ee), `hadronic.rs`, `diagrams.rs` | top | 1 | 1 | decays.py width |
| `h > b b~` | `cli_decay.rs` | top | 1 | 1 | decays.py width |
| 16 more two-body channels of `sm_decay_widths.json` (`h > ta- ta+`; `z >` d d~, s s~, b b~, u u~, c c~, e- e+, mu- mu+, ta- ta+, ve ve~, vm vm~, vt vt~; `w+ >` e+ ve, mu+ vm, ta+ vt, u d~, c s~) | `decay_widths.rs` | top | 1 each | 1 each | decays.py widths |
| `h > b b~ g`, `t > w+ b g`, `z > e+ e- a`, `w+ > j j`, `h > w+ e- ve~` | `diagrams.rs` (hermetic, own counts) | top | 2, 2, 2, 2, 1 | same | none (hermetic) |
| `t > w+{0} b` | `decay_widths.rs`, `check.rs`; polarization_census `decay_t_wp0_b` | top | 1 | 1 | polarization census: no WEIGHTED in MadGraph's process string |
| `t > b e+ ve $ w+` | `onshell_veto_reference.json` (t_bev_w), `check.rs` | top | 1 | 1 | MadEvent width (hermetic onshell-veto gate) |
| **`z > u u~ d d~`** | `weighted_search.rs` (F1's own test) | top | **4** | **17** | **none** -- see the banking request |
| `t{L} > w+ b` | `check.rs` | top | (refused: a polarized decay with orders is unsupported) | 1 | -- |
| chain decays `z > e+ e-`, `z > mu+ mu-`, `z > ta+ ta-`, `t > w+ b`, `t~ > w- b~`, `w+ > e+ ve`, `w- > e- ve~`, `t > b e+ ve`, `t~ > b~ e- ve~`, `w+ > j j`, `w- > j j`, `z > l+ l-` | decay_chain_census (24 cards), decay_chain_sigma, decay_chain_events | chain (search runs) | -- | bound 2 (two-body), 4 (three-body) | the bound MadGraph recorded, `Decay: ... WEIGHTED<=2` / `<=4`: identical on every one (section 2) |

The polarized-decay row and the 17/4 row are the only lines whose old and new
columns are not both counted from the same generation; the 17 is F1's own
assertion `auto == unbounded explicit orders`.

The 1->n rule itself is pinned by MadGraph's own record: schannel_census banks
the orders of each generated subprocess, and every top-level decay card there
(`decay_t_bepve`, `decay_t_bepve_no_t`, `decay_t_wp_bepve`, `decay_h_zz`,
`decay_h_zz_no_h`) carries `"orders": {}` -- no WEIGHTED bound -- where every
2->n card carries one and every chain decay in decay_chain_census carries one.
That is `diagram_generation.py:1978-1980` observed rather than read.

**Banked gates I cannot run here** (no `validation/madgraph/output`, no PDF
sets): `decay-chain-sigma` (`cli_decay_chain.rs`) and `decay-chain-events`
(`cli_decay_chain_events.rs`), both `extended-validation`. Their cards' diagram
sets are unchanged by F1, measured (`probe2.log`, `probe3.log`):

- every chain decay in them (`z > e+ e-`, `z > mu+ mu-`, `t > w+ b`, `t~ > w- b~`,
  `w+ > e+ ve`, `t > b e+ ve`, `t~ > b~ mu- vm~`) still runs the search, now
  starting at MadGraph's start, which equals F1's cap for each (2 or 4), and is
  empty one below it (`z > e+ e- WEIGHTED<=1`, `t > w+ b WEIGHTED<=1`,
  `w+ > e+ ve WEIGHTED<=1`, `t > b e+ ve WEIGHTED<=3`: all `NoDiagrams`), so the
  old loop reached the same bound;
- every core (`e+ e- > z z`, `e+ e- > t t~`, `p p > t t~`, `p p > z`,
  `e+ e- > mu+ mu- z $ z`, `p p > e+ e- / z`, `p p > mu+ mu- / a`) is likewise
  empty one below the new start, and its count at the new bound is the
  all-orders count except `p p > t t~` (7 against 15 unbounded), whose start (2)
  is the old start;
- full chain counts at HEAD: pp_ttx_lep 7, veto_chain 8, `p p > z, z > e+ e-` 4,
  zz_emu 2, ttx_nested 2.

So CI's banked job on this branch should show both decay-chain gates exactly as
on `main`: same diagrams, same matrix elements, same seeds.

## 2. The loop against MadGraph's own record

**What exists.** No committed file carries MadGraph's
`Trying coupling order WEIGHTED<=N` trace: the build logs live in
`validation/madgraph/output/<row>/build.log`, which is not committed and is
absent here (`grep -rl "Trying coupling order"` finds only this sprint's two
briefs). What *is* committed is the bound the loop returned, on 121 process
strings in four files: `decay_chain_census.json` (core and per-decay
`WEIGHTED<=N`), `polarization_census.json` (`WEIGHTED<=N @1`),
`schannel_census.json` (`orders.WEIGHTED` per subprocess, and the bound in each
`NoDiagramException` message) and `proc_grammar.json` (outcomes only). The bound
is the loop's output, `max(start, first w with a trial diagram)` or
`max(start, cap)`, so it checks start, step and cap together on each card,
though not each trial.

**What it confirms.** Script: MadGraph's bounds extracted per card
(`wcapE-V1-scripts/mg_bounds.py`), vibegraph's per call from the trace
(`vb_trace.py`), joined per card (`join.py`, output `wcapE-V1-join.txt`):

```
agree 109
('not_a_decay', [], ['4'], ...)          both refuse (two initial particles in a decay); no bound to compare
('g0_g_ttx', [], ['4'], ...)             both refuse; MadGraph's message bound is 2, see below
('ee_a0_z', [], ['4'], ...)              both refuse; MadGraph's message bound is 4 = ours
('weighted_le' / 'weighted_eq' / 'weighted_too_low' / 'weighted_pp' / 'weighted_with_qcd')
                                          explicit WEIGHTED: neither side searches
('five_flavour_forbidden_j', ['5'], ['6'], ...)   both refuse; see below
('five_flavour_smeftsim', ['4'], ['4', ...])      agrees (4, start 4, cap 210); a probe-ordering artefact in the join
```

So on every card where MadGraph generated, the bound this crate's search stopped
at equals MadGraph's: 2->2 at 2, 3 and 4; 2->3 at 5; 2->4 with required
s-channels at 6 and 8 (`ee_zh_mumubbx`, `ee_ttx_wpbwmbx`, `uux_ttx_wpbwmbx`: the
s-channel term of the start); chain cores and decays at 2, 3, 4; SMEFTsim at 4.
Of the refused cards, four carry a bound in MadGraph's message: `udx_wm_epve`,
`pp_wm_epve`, `udx_epve_no_wp` (4, ours 4), `ee_za_mumua` (8 from the s-channel
start term, above the cap 6; ours 8) and `ee_a0_z` (4, ours 4) agree. Two differ,
both refused on both sides:

- `five_flavour_forbidden_j` (`u u~ > e+ e- g / j`, sm-no_b_mass): MadGraph 5,
  ours 6. MadGraph's trial at its start 5 drops the final gluon
  (`u u~ > e+ e- / j` at `WEIGHTED<=4`, which has diagrams) and returns 5; ours
  trials the full process, finds nothing below the cap and generates at 6.
  This is F1's Found 1 observed in a committed reference.
- `g0_g_ttx` (`g{0} g > t t~`): MadGraph 2 (`NoDiagramException ... WEIGHTED=2`),
  ours 4. MadGraph's trial process is built from the legs' ids alone
  (`base_objects.Leg({'id': id, 'state': ...})`, :2025-2044), so it drops the
  polarization and finds `g g > t t~` at 2. Polarization never changes a
  diagram's orders, so this can only move the bound of a card that is refused
  anyway; it is not a diagram-set question (Found 3).

No banked trace confirms an individual step; the found bounds are consistent
with start, `+1` and cap on all 109 generating cards, and the s-channel start
term is exercised (and agrees) on the 8-bound rows.

## 3. F1's Found 1 and 2 as validation questions

**Found 1 (final-state gluon removal, :2040-2059): yes, in reach, on SMEFTsim.**
The removal applies when pdg 21 is in MadGraph's lowest hierarchy class
(`particles[0]`), which holds in the SM and in SMEFTsim (QCD and the `NPc*`
orders at hierarchy 1). In the SM it changes no diagram set: a gluon there
couples only through QCD at weight 1 (the reduced trial of an all-gluon final
state even succeeds trivially through the 2->0 identity), and the one
committed case where it moves a bound is a refusal on both sides (section 2).
SMEFTsim adds a gluon coupling outside the lowest class, the loop-induced
`h g g` at `SMHLOOP` (hierarchy 99), and there the outcome changes.
`e+ e- > z g g` on `restrict_SMlimit_massless`:

| Step | MadGraph (read at b7687064) | this crate, measured |
|---|---|---|
| start | e+, e-, z in class 2, g, g in class 1: drop 1, 1, sum 6 | `VBW start=6 cap=315` |
| trial at 6 | `e+ e- > z WEIGHTED<=4` (gluons removed, 6 - 2*1): 1 diagram, returns 6 | full `e+ e- > z g g WEIGHTED<=6`: `NoDiagrams` |
| result | generates `e+ e- > z g g WEIGHTED<=6`: no diagram, `NoDiagramException` | runs on to `found=107`: 1 diagram (`Z* -> Z h*, h* -> g g`) |

The same holds on `vg_cG` and `vg_cHG` (`probe2.log`, `GLUON ...`), and
`e+ e- > mu+ mu- g g` (1 diagram here) follows the same pattern. The failing
case is committed as
`weighted_search.rs::a_final_state_gluon_pair_without_a_qcd_vertex_is_refused`,
`#[ignore]` with the reason. It asserts both premises (`e+ e- > z WEIGHTED<=4`
has 1 diagram; `e+ e- > z g g WEIGHTED<=6` has none) before the outcome, so it
fails only on the outcome:

```
$ cargo test -p vibegraph-lib --test weighted_search -- --include-ignored
test a_final_state_gluon_pair_without_a_qcd_vertex_is_refused ... FAILED
  panicked at vibegraph-lib/tests/weighted_search.rs:166:18:
  expected NoDiagrams, got Ok(1)
test result: FAILED. 6 passed; 1 failed; 0 ignored
```

Mutation evidence, that the test passes when the search returns MadGraph's
bound for this card: `while w < cap` -> `while w < start` (the search generates
at its start and stops), applied, run, restored and touched:

```
test a_final_state_gluon_pair_without_a_qcd_vertex_is_refused ... ok
test a_two_order_vertex_is_found_by_the_search_below_the_cap ... FAILED
test a_two_order_vertex_is_found_above_the_largest_hierarchy ... FAILED
test result: FAILED. 5 passed; 2 failed
```

What would falsify it: a MadGraph 3.7.1 run of
`import model SMEFTsim_topU3l_MwScheme_UFO-SMlimit_massless` /
`generate e+ e- > z g g` that does not raise `NoDiagramException` at
`WEIGHTED=6`. That run is cheap and worth adding to the banking request's M3
session (below), since the reading here, while traced line by line, is not a
measurement.

**Found 2 (no `coupling_orders.py`): no model in reach.** All four UFOs under
`validation/ufo` ship the file, and the interned SM has its hierarchy. As a
check of what the rule would do, each toy was copied without the file
(this crate falls back to `{QCD: 1, QED: 2}`) and with an all-ones file (what
MadGraph's `get_order_hierarchy` gives a model whose orders are not exactly
{QCD, QED}, `base_objects.py:1379-1386`):

| Model | Processes | shipped / no file / all-ones |
|---|---|---|
| `vibegraph_toy_UFO` (orders all hierarchy 1) | 15, e.g. `lt~ lt > qt qt~` 8/8/8, `lt~ lt > lt~ lt st` 96/96/96 | all equal |
| `vibegraph_toy_color_UFO` | 3 | all equal |
| `vibegraph_toy_weighted_UFO` | `fw~ fw > fw~ fw` | 2 / **4** / 2 |
| | `fw~ fw > fw~ fw xw` | 8 / **16** / 8 |
| | 6 others | all equal |

So the fallback does change a diagram set, but only on a model built for the
purpose; no shipped model reaches it, and per the brief no test was built.

## 4. Banking request, final form (M3, MadGraph 3.7.1)

Run at the M3 in a checkout of this branch, reference bundle present.

**Scripts** (four new files in `validation/madgraph/scripts/`, one row each).
The first, in full; the others differ only in the header comment, the
`generate` line and the `output` name:

```
# MadGraph5 batch script: fw~ fw > xw xw
# in vibegraph_toy_weighted_UFO, no restrict card.
#
# Why this exact process:
#   Its only diagrams carry the QED+NP vertex fw~ fw xw twice (WEIGHTED 6),
#   above (n_ext - 2) times the model's largest hierarchy (4) and at
#   (n_ext - 2) times the largest interaction WEIGHTED per equivalent cubic
#   vertex (3): MadGraph's automatic WEIGHTED search finds them only because its
#   cap is the latter. No order bound on purpose: the row is the search.
#
# The model path is repo-relative; build.sh rewrites it to the staged copy.
# `display interactions` puts the interaction count (5) into build.log.
import model validation/ufo/vibegraph_toy_weighted_UFO
display interactions
generate fw~ fw > xw xw
output ffx_to_xwxw_toy_weighted -nojpeg
```

| Script | `generate` | `output` |
|---|---|---|
| `ffx_to_xwxw_toy_weighted.mg5` | `fw~ fw > xw xw` | `ffx_to_xwxw_toy_weighted` |
| `ffx_to_xwxwvw_toy_weighted.mg5` | `fw~ fw > xw xw vw` | `ffx_to_xwxwvw_toy_weighted` |
| `xwxw_to_xwxw_toy_weighted.mg5` | `xw xw > xw xw` | `xwxw_to_xwxw_toy_weighted` |
| `z_to_uuxddx_decay.mg5` (`import model sm`, no `display interactions`) | `z > u u~ d d~` | `z_to_uuxddx_decay` |

The fourth is included because section 1 found one decay whose new set has no
MadGraph reference. No `launch`: every existing script launches, so check after
`build.sh` that `extract_diagrams.py` found `matrix1_orig.f` or
`template_matrix1.f` in each new `P*` directory; if not, add the fixed-beam
block of the toy rows (`launch` / `set lpp1 0` / `set lpp2 0` / `set ebeam1 250`
/ `set ebeam2 250` / `set use_syst False` / `done`).

In the same session, outside the bank (no row): `generate e+ e- > z g g` on
`SMEFTsim_topU3l_MwScheme_UFO-SMlimit_massless`, keeping the log, to confirm or
falsify the ignored test of section 3.

**Extraction.** `python validation/madgraph/extract_diagrams.py` regenerates
`diagrams.json` (its keys follow the manifest). Expected `total_diagrams`: 2, 6,
4, 17. From each `output/<row>/build.log` keep the
`Trying coupling order WEIGHTED<=N` lines (`diagram_generation.py:2014`) in the
row's `diagrams.note`: expected `4, 5` / `6, 7` / none (the first trial, 4,
returns) / none (no search: the line is absent, and the generated process string
carries no `WEIGHTED`). For the first, the absence of a line for 6 is MadGraph
generating at the cap after its loop. `extract_interactions.py` should bank 5
interactions for the toy (`interactions.json`).

**Manifest rows** (full text; the three toy rows differ only in key, process,
script, `n_final` and the note's numbers):

```toml
[[process]]
key = "ffx_to_xwxw_toy_weighted"
process = "fw~ fw > xw xw"
model = "validation/ufo/vibegraph_toy_weighted_UFO"
script = "validation/madgraph/scripts/ffx_to_xwxw_toy_weighted.mg5"
class = "single-channel"
n_final = 2
bundled = false
rationale = "The automatic WEIGHTED search's cap: the only diagrams carry a QED+NP vertex twice (WEIGHTED 6), above (n_ext - 2) times the largest hierarchy (4), so a search capped there finds none; MadGraph's cap is (n_ext - 2) times the largest interaction WEIGHTED per equivalent cubic vertex, int(3.5) = 3, which reaches them."
artifacts = ["output/ffx_to_xwxw_toy_weighted", "output/ffx_to_xwxw_toy_weighted.json"]
[process.categories]
diagrams = { tier = "hermetic", mode = "gate", note = "2 diagrams enumerated against MadGraph's 2 (t and u fermion exchange). MadGraph's build log: Trying coupling order WEIGHTED<=4, <=5, then generation at the cap 6." }
amplitudes = { tier = "uncovered", note = "A search-cap row: the vertices' Lorentz structures are the toy's plainest and are reached by no new amplitude here." }
integrals = { tier = "uncovered", note = "Counts only; no MadEvent run." }
samples = { tier = "uncovered", note = "Counts only; no MadEvent run." }
```

```toml
[[process]]
key = "z_to_uuxddx_decay"
process = "z > u u~ d d~"
script = "validation/madgraph/scripts/z_to_uuxddx_decay.mg5"
class = "single-channel"
n_final = 4
bundled = false
rationale = "A top-level 1 -> n decay without orders runs no WEIGHTED search in MadGraph (find_optimal_process_orders returns the empty orders for a one-particle process that is not a decay-chain decay), so both orders are kept: the four QCD^2 QED gluon exchanges and the QED^3 photon, Z and W exchanges."
artifacts = ["output/z_to_uuxddx_decay", "output/z_to_uuxddx_decay.json"]
[process.categories]
diagrams = { tier = "hermetic", mode = "gate", note = "17 diagrams enumerated against MadGraph's 17; a search would have stopped at WEIGHTED<=4 with 4. MadGraph's build log carries no Trying coupling order line." }
amplitudes = { tier = "uncovered", note = "Counts only." }
integrals = { tier = "uncovered", note = "Counts only." }
samples = { tier = "uncovered", note = "Counts only." }
```

`class` is `single-channel` on all four: in the manifest it separates one
concrete subprocess from a multiparticle `p p` row, and each of these has one.
Drop `bundled = false` once the next bundle carries the four output
directories. The notes' counts are to be replaced by what the
run printed, not kept as written here.

**What each passing test then shows.** `validate_madgraph_diagrams` gains four
trials, each generating the row's `generate` line here and comparing the count
with MadGraph's `NGRAPHS`: the first two show the cap (a `(n-2)*max(hierarchy)`
cap fails both with `NoDiagrams`; F1's mutation M5); the third shows the start
(a `(n-2)*min(hierarchy)` start gives 1, not 4; M4); the fourth shows the 1->n
rule (searching a top-level decay gives 4, not 17; M7). Each is blind to which
diagram is which beyond the count, and to the per-trial path, which only the
`Trying` lines in the notes record.

## Gate

Final run on the committed tree (instrumentation reverted, probes removed),
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ cargo fmt --all --check
fmt_exit=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 1m 02s
clippy_exit=0

$ cargo test --workspace
41 binaries: 1548 passed, 0 failed, 19 ignored       (F1: 1548 / 0 / 18; +1 ignored, the new test)
  lib (vibegraph-lib)          test result: ok. 1013 passed; 0 failed; 12 ignored
  validate_madgraph_diagrams   test result: ok. 57 passed; 0 failed; 0 ignored
  weighted_search              test result: ok. 6 passed; 0 failed; 1 ignored
  decay_widths                 test result: ok. 5 passed; 0 failed; 1 ignored
  cli_decay                    test result: ok. 3 passed; 0 failed; 0 ignored
  decay_chain_census           test result: ok. 1 passed; 0 failed; 0 ignored
  schannel_census              test result: ok. 1 passed; 0 failed; 0 ignored
  polarization_census          test result: ok. 1 passed; 0 failed; 0 ignored
test_exit=0

$ python3 scripts/kb.py lint      (after python3 scripts/kb.py index)
833 concepts, 291 backlog items; 0 errors, 4 warnings   (the four pre-existing `state: blocked` warnings F1 also saw)
```

The diagram census is unmoved: 57 passed, as F1 left it.

## For the manager

- **The brief's premise holds and needs no correction**, with one precision:
  the "MadGraph's own banked logs" that item 2 asks for do not exist in the
  repository; the committed found-bounds are the oracle used instead, and they
  confirm the loop's output on 109 generating cards, not its individual
  trials.
- The SM decays the suite computes are all single-order, which is why the 1->n
  rule moves none of them; F1's `z > u u~ d d~` is the only mixed-order decay in
  reach and the only one without a MadGraph reference.
- The SMEFTsim `e+ e- > z g g` reading rests on MadGraph's code, not a run; the
  M3 confirmation is one `generate` line.

## Found

1. **MadGraph's search trial drops the final-state gluons; this crate's does
   not, and on SMEFTsim the outcome differs.** Evidence: section 3 (SMEFTsim
   SM limit, `e+ e- > z g g`: MadGraph refuses at `WEIGHTED<=6`, this crate
   returns 1 diagram at 107) and the committed reference
   `schannel_census.json` `five_flavour_forbidden_j` (MadGraph's refusal at 5
   against ours at 6). Close: reproduce the reduced trial
   (`diagram_generation.py:2040-2059`, with the trivial-identity behaviour of a
   final state left empty), un-ignore
   `a_final_state_gluon_pair_without_a_qcd_vertex_is_refused`, and bank the
   SMEFTsim run above as its reference. This is F1's Found 1, now with a
   failing test.
2. **The missing-`coupling_orders.py` fallback differs from MadGraph's, out of
   reach today.** Evidence: section 3's table (`vibegraph_toy_weighted_UFO`
   without the file: `fw~ fw > fw~ fw` 4 here, 2 under MadGraph's all-ones
   default). Close: MadGraph's rule in `ParsedModel::parse` (all orders 1, QED 2
   only when the orders are exactly {QCD, QED}) with a test on a temporary copy
   of that toy without the file. F1's Found 2, now measured.
3. **MadGraph's search trial drops polarizations (and keeps only ids).**
   Evidence: polarization_census `g0_g_ttx`, MadGraph `NoDiagramException ...
   WEIGHTED=2`, this crate refuses at 4. Polarization never changes a diagram's
   orders, so this can move only the bound of a card refused on both sides;
   it changes no diagram set and no refusal. Close: nothing needed beyond
   noting it wherever the search's divergences from MadGraph are listed; if
   refusal messages ever quote the bound, quote MadGraph's.

## 2026-10-20 follow-up: was the Found 1 disagreement there before F1?

**Yes. F1 did not introduce it.** I reproduced the pre-F1 loop with explicit
bounds, as for the decays: start `(n-2)*min(hierarchy)`, cap
`(n-2)*max(hierarchy)` inclusive, return the first bound with a diagram. The
probe was scratch (`.../scratchpad/wcapE-V1-probes/wcap_v1_probe4.rs`) and is
not committed. SMEFTsim's declared hierarchy runs from 1 (QCD, `NPc*`) to 99
(`NP`, `SMHLOOP`, ...), so for `e+ e- > z g g` (n = 5) the old start is 3 and
the old cap is 297.

| Restrict | `<=3` | `<=6` | `<=106` | `<=107` | `<=297` | old loop stops at | old result | new result |
|---|---|---|---|---|---|---|---|---|
| SMlimit_massless | none | none | none | 1 | 2 | 107 | 1 diagram | 1 diagram (found 107) |
| vg_cG | none | none | none | 1 | 2 | 107 | 1 diagram | 1 diagram |
| vg_cHG | none | none | 1 | 2 | 4 | first nonempty bound, at most 106 | 1 diagram | 1 diagram |

On vg_cHG the old stopping bound is not pinned exactly, but it does not need to
be. Both loops climb one step at a time from a start that has no diagrams (old
3, new 6, and `<=6` is empty), so each stops at the same first bound with a
diagram and keeps the same set. MadGraph refuses on all three (its gluon-free
trial returns 6). The disagreement is therefore older than F1, which neither
created nor fixed it.

`five_flavour_forbidden_j` (`u u~ > e+ e- g / j`, sm-no_b_mass, n = 5): the old
start is 3 and the old cap is 2 * 3 = 6, inclusive. The full process has no
diagram at `<=6` (the new loop generated there and refused), so the old loop
also refused, at 6. MadGraph refuses at 5. Both versions refuse; only the bound
in the message differs, before F1 and after it.

Because F1 introduced nothing here, Found 1 stays an existing gap rather than a
regression. For sizing anyway: reproducing the gluon-free trial is local to the
search loop in `generate_undecayed`. Each trial would generate the same expanded
process with final-state gluons (pdg 21, only when 21 is in the lowest class)
removed from every final leg's id set, at `w - nglue * hierarchy[QCD]`. A final
state left empty succeeds through MadGraph's identity vertex when the initial
pair is conjugate. That is tens of lines plus that edge case. The awkward parts
are two: the reduced trial is per flavour assignment where the loop now trials
the whole multiprocess, and a leg whose multiparticle mixes gluons with other
ids drops only its gluon assignments.

## 5. Banking request, corrected after the M3 build (supersedes §4's `Trying` lines)

Section 4 is superseded where it predicts the `Trying coupling order` lines and
the notes built on them. Its scripts, manifest rows' `rationale` text, `class`
choice, extraction steps and the test descriptions stand, except as changed
below. The cause is [V2's reading of the source](V2-report.md): MadGraph prints
the `Trying` line *before* it runs the trial (`diagram_generation.py:2014`,
the trial is at :2023-2109), so a trial that succeeds is still one of the
printed lines, and the loop runs a trial at every bound strictly below the cap
(:2013), so a row whose diagrams weigh less than the cap succeeds inside the
loop. §4 predicted the lines of the failed trials only, and for the second row
took the stop at 7 where the diagrams weigh 8.

**Expected `Trying coupling order WEIGHTED<=N` lines** (each line continues
`: WEIGTHED IS <hierarchy definition>`, MadGraph's constant text; the notes
below omit it):

| row | start | cap | lines | what happens after |
|---|---|---|---|---|
| `fw~ fw > xw xw` | 4 | 6 | 4, 5 | the loop leaves at the cap and generates at 6 (no line for 6); 2 diagrams |
| `fw~ fw > xw xw vw` | 6 | 9 | 6, 7, 8 | the trial at 8 succeeds (its diagrams weigh 3 + 3 + 2 = 8); 6 diagrams |
| `xw xw > xw xw` | 4 | 6 | 4 | the trial at 4 succeeds; 4 diagrams |
| `z > u u~ d d~` | -- | -- | none | one-particle process, no search (:1978-1980); 17 diagrams |

The first and fourth rows matched §4 and the M3 measurement; the second and
third are the corrected ones, and the M3 measured all four.

**Scripts.** The three toy scripts are as in §4. `z_to_uuxddx_decay.mg5` has no
`import model` line (the harness rejects it for the default SM, and the repo's
other default-SM scripts omit it):

```
# MadGraph5 batch script: z > u u~ d d~
# in the default SM, no restrict card.
#
# Why this exact process:
#   A top-level 1 -> n decay without orders runs no WEIGHTED search in MadGraph
#   (find_optimal_process_orders returns the empty orders for a one-particle
#   process that is not a decay-chain decay), so every order is kept: the four
#   QCD^2 QED gluon exchanges and the QED^3 photon, Z and W exchanges.
generate z > u u~ d d~
output z_to_uuxddx_decay -nojpeg
```

**Manifest `diagrams` notes**, exact text; the rest of each row is as in §4.

```toml
# ffx_to_xwxw_toy_weighted
diagrams = { tier = "hermetic", mode = "gate", note = "2 diagrams enumerated against MadGraph's 2 (t and u fermion exchange). MadGraph's build log: Trying coupling order WEIGHTED<=4, <=5; no trial at 6, the loop leaves at the cap 6 and generates there." }

# ffx_to_xwxwvw_toy_weighted
diagrams = { tier = "hermetic", mode = "gate", note = "6 diagrams enumerated against MadGraph's 6 (the three emissions in any order along the fermion line, each diagram WEIGHTED 8). MadGraph's build log: Trying coupling order WEIGHTED<=6, <=7, <=8; the trial at 8 succeeds, below the cap 9." }

# xwxw_to_xwxw_toy_weighted
diagrams = { tier = "hermetic", mode = "gate", note = "4 diagrams enumerated against MadGraph's 4 (the contact term and the s, t and u yw exchanges). MadGraph's build log: Trying coupling order WEIGHTED<=4 only; the first trial succeeds, below the cap 6." }

# z_to_uuxddx_decay
diagrams = { tier = "hermetic", mode = "gate", note = "17 diagrams enumerated against MadGraph's 17; a search would have stopped at WEIGHTED<=4 with 4. MadGraph's build log carries no Trying coupling order line." }
```

The three toy rows' `rationale` lines: the first as in §4; for the other two
use `fw~ fw > xw xw vw`: "The automatic WEIGHTED search stops inside its loop, above the
largest hierarchy and below the cap: the six diagrams weigh 3 + 3 + 2 = 8, the
search starts at 6, and MadGraph's cap (n_ext - 2) times int(3.5) = 9 is not
reached." and `xw xw > xw xw`: "The automatic WEIGHTED search starts at the sum of the
n_ext - 2 largest leg values, 4, not at the lowest WEIGHTED any diagram has (the
contact term, 2): the first trial already holds the four diagrams."

**Instruction for the M3 session.** Commit the banked tree as it stands, with
the four notes above verbatim (they agree with the logs the M3 measured: the
`Trying` lines 4, 5 / 6, 7, 8 / 4 / none and the counts 2, 6, 4, 17). Before
committing, check each `build.log` still shows exactly those `Trying` values and
no others; if a log differs, stop and report instead of editing the note to
match. Add the `z_to_uuxddx_decay.mg5` text above without `import model`; keep
`model` unset on that row. The `e+ e- > z g g` SMEFTsim run stays outside the
bank, and its recorded outcome (`NoDiagramException` at `WEIGHTED<=6`) is
the confirmation the ignored test of §3 needs.
