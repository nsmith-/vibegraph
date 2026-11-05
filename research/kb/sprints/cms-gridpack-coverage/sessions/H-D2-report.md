---
type: Session Report
title: "H-D2 report: a subprocess reached by two decay assignments is two subprocesses, summed incoherently as MadGraph does"
description: "D3 implemented. The in-line DuplicateSubprocess refusal is gone; each (core, decay assignment) is its own DiagramSet with its own forced lines. The decay-chain census now banks g g / e+ e- / p p > z j j, z > j j and matches MadGraph's 6 / 6 / 30 doubly reached subprocesses with per-matrix-element diagram counts. σ of the twelve two-assignment subprocesses of g g > z j j, z > j j: 112.04 ± 0.06 pb (vibegraph) against 111.70 ± 0.18 pb (MadEvent), +0.30%, the same offset as a single-assignment calibration card (+0.32%), so the ratio agrees to −0.02%. No card that reaches the duplicate runs end to end through the CLI: every such card stops at GroupCutsDiffer (D4)."
status: draft
tags: [decay-chains, madgraph-parity, cms, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-04}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: hd1, resource: "H-D1-report.md", title: "H-D1 report: MadGraph sums the two decay assignments incoherently"}
  - {id: mg-combine, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/helas_objects.py#L5427", title: "MadGraph helas_objects.py combine_decay_chain_processes"}
---
Session H-D2 of sprint H, item `decay-chain-duplicate-subprocess-vs-madgraph`,
implementing decision D3 (`origin/sprinth-plan`,
`decisions/D3-decay-duplicates-match-madgraph.md`). Worktree
`/home/user/wt-decay-duplicate`, branch `sprinth-decay-duplicate` from `f7ed6b0`,
verified before any action:

```
$ cd /home/user/wt-decay-duplicate && git rev-parse --show-toplevel && git branch --show-current
/home/user/wt-decay-duplicate
sprinth-decay-duplicate
```

MadGraph is the pinned 3.7.1 checkout, run from H-D1's scratch copy
(`scratchpad/hd1/mg5`). Scratch artefacts (not committed) are under
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/H-D2/`
and `scratchpad/H-D2_*.log`; the σ harness is `H-D2/hd2_scratch_sigma.rs`, an
integration test that was in `vibegraph-lib/tests/` while it ran and was removed
before the commit.

## 1. What changed

- **`diagrams/chain.rs`**: the refusal at `:203`–`:220` is gone. Every
  (core, assignment) pair is its own `DiagramSet`, with that assignment's forced
  lines, so its own Breit–Wigner windows and, under `cut_decays = F`, its own cut
  legs. A `debug_assert!` pins that no (core, assignment) key is stitched twice:
  `assignments` keeps an unordered combination once (as `red_decay_chains`), and the
  core subprocesses are already distinct by content, so a duplicate content is
  always two different assignments. The top-level line logs how many subprocesses
  two assignments reach. The module docs say why this departs from the identical-
  particle rule (every pairing in one set).
- **The refusal message that named one line twice** no longer exists: that message
  came only from the in-line check, which is removed. `DiagramError::DuplicateSubprocess`
  is now raised only by the cross-line check (`diagrams/mod.rs:433`, unchanged), whose
  message names two different lines.
- **`onshell.rs`**: the `$` group-marking check looked each member up in a
  `BTreeMap` keyed by its ordered (incoming, outgoing) PDG codes, built before
  `content_ordered` reorders legs. With two assignments of one content, a reordered
  member's key can equal the *other* assignment's enumerated key, and the check then
  compares against the wrong marking; a reordered member not present under its new
  key was skipped silently. The map (`subprocess_markings`, `SubprocessKey`,
  `set_key`) is removed. Each member's marking is now read off its own diagrams,
  `FlavorGroup::member_diagram_set(i)`, whose legs are in the representative's
  positions. `group_vetoes` and `check_group_members` lose the `markings` argument;
  callers in `multiplicity.rs` and `tests/validate_hadronic.rs` follow.
- **Census**: `dump_decay_chain_census.py` gains `gg_zjj_jj`, `ee_zjj_noz_jj`,
  `pp_zjj_jj`; `decay_chain_census.json` is regenerated (pure insertions: the 24
  existing cases are byte-identical, §3). `tests/decay_chain_census.rs` now also
  compares, per card, how many particle contents more than one entry covers.
- **Tests**: `tests/diagrams.rs` `a_subprocess_from_two_decay_assignments_is_two_subprocesses`
  and `two_assignments_on_two_lines_are_refused`; `proton.rs`
  `each_decay_assignment_of_one_subprocess_is_a_member`; `onshell.rs`
  `group_members_must_share_the_marking` rewritten for the new API.

### Audit of what assumed one diagram set per flavour content

| path | finding |
|---|---|
| `derive_flavor_groups` partition | By probe trace, never by content. Two assignments join one group only if their `|M|²` agree pointwise in their own leg orders; each is a member with its own luminosity term. No dedupe anywhere (`luminosity_rows`, `symmetry_weighted_luminosity_rows`, `member_slots` are per member). Measured in §4. |
| degenerate-groups check | `DegenerateGroups` fires on two groups within `GROUP_SEPARATION_MIN`. On the gg card both assignments of every content land in one group (traces equal to 1e-12, §4), and the three groups are separated: derivation passes on the gg, two-assignment-only and pp cards (§5). Nothing to change. |
| event process ids | `provenance.process` is the line's `@N`; both assignments of a line share it, as both of MadGraph's matrix elements carry the line's process number. Correct as is. |
| resonance records | Built per set (fixed beams, `generate.rs:436`) or per group from the representative's diagrams with `member_line_pdg` per member (`group_resonances`). A member of the same group has the same forced-line positions as the representative by construction of the group. Correct as is. |
| decay-chain census | Keys are (sorted initial, in-place final); two assignments put the decay products at the decaying particle's position, so their keys differ on both sides (`banked` asserts no key twice and held for all 42). |
| `HashMap`/`BTreeMap` keyed by flavour | `diagrams/mod.rs` `owner: HashMap<FlavourKey, …>` is the cross-line check and skips same-line entries (unchanged, as briefed). `onshell.rs` markings: fixed (above). Nothing else in `proton.rs`, `generate.rs`, `lhef/`, `reweight/`, `multiplicity.rs` or the CLI keys by content (grep of `HashMap<`/`BTreeMap<`/`HashSet<` and of content-equality lookups). |

### Equal ME tags (H-D1's two `@1` lines)

H-D1 §1–2: when two combinations carry equal ME tags MadGraph appends the second's
processes to the first matrix element, so the subprocess is two PD terms of one
`|M|²`; with different tags it is two matrix elements. Both are
`|M_a|²Θ_a + |M_b|²Θ_b`. So the tag is MadGraph bookkeeping, and what vibegraph must
reproduce is the incoherent sum, not the tag:

- **Within one line** vibegraph has two sets either way. Their flavour grouping is
  the analogue of the tag: where the two `|M|²` agree (a flavour symmetry), they are
  one group with two luminosity terms, MadGraph's "one ME, two PD terms"; where they
  differ, two groups. Pinned by `each_decay_assignment_of_one_subprocess_is_a_member`
  (both assignments of each of the 6 gg contents are members, each in the group whose
  `|M|²` is its own at fresh points, and the groups' unit-density luminosity terms sum
  to 16, one per subprocess).
- **Across two lines** (H-D1's `@1` + `@1` card, the only equal-tag case measured) the
  refusal at `diagrams/mod.rs:433` stays, as briefed. This departs from MadGraph,
  which sums them; the departure is pinned by `two_assignments_on_two_lines_are_refused`
  (§6) and filed in Found.

## 2. Gates

Commands run with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0` and `-j 2` (4 shared cores, load average 12–13).

Dev profile (opt-level 2, debug assertions on) throughout, including the banked
gates, which `pixi run validate` would run under `release-debug`. pixi is not
installed. Logs: `scratchpad/H-D2_gates.log`, `H-D2_banked.log`,
`H-D2_banked_full.log`, `H-D2_banked_diagrams.log`.

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | `GATE fmt 0` |
| clippy | `cargo clippy -j 2 --workspace --all-targets -- -D warnings` | `Finished`, `GATE clippy 0` |
| clippy, extended-validation targets | `cargo clippy -j 2 --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` (as `validate.sh` lints) | `GATE clippy-ev 0` |
| hermetic suite | `cargo test -j 2 --workspace` | `GATE test 0`; 47 `test result: ok` lines, summed 1807 passed, 0 failed, 25 ignored |
| diagram enumeration (banked) | `cargo test -p vibegraph-lib --features extended-validation --test validate_madgraph_diagrams -- --nocapture` | `test result: ok. 63 passed; 0 failed` |
| `$` vetoes through the binary (banked; `onshell.rs` changed) | `cargo test -p vibegraph --features … --test cli_onshell_veto` | `test result: ok. 4 passed; 0 failed` |
| decay-chain σ (banked) | `… --test cli_decay_chain` | `test result: ok. 1 passed; 0 failed` |
| decay-chain events (banked) | `… --test cli_decay_chain_events` | `test result: ok. 7 passed; 0 failed` |

The first attempt at the diagram gate passed `--features vibegraph/extended-validation`
to `-p vibegraph-lib`, which cargo rejects (*"the package 'vibegraph-lib' does not
contain this feature"*). The rerun with `--features extended-validation` is the row
above.

Enforced cells keep their status: all of these pass. The σ rows read as they did,
for example from `cli_onshell_veto`:

```
[pp_z] p p > e+ e- $ z (MadGraph 3.7.1): here 3.038105e2 ± 1.3e-1, MadEvent 3.038540e2 ± 2.7e-1, rel -1.433e-4, pull -0.14, χ²/dof 0.19
[t_bev_w] t > b e+ ve $ w+ (MadGraph 3.7.1): here 1.434664e-3 ± 1.2e-6, MadEvent 1.435080e-3 ± 7.4e-7, rel -2.900e-4, pull -0.30, χ²/dof 1.23
[ee_z_pole] e+ e- > mu+ mu- $ z (MadGraph 3.7.1): here 1.076789e1 ± 2.0e-3, MadEvent 1.076720e1 ± 2.1e-3, rel +6.412e-5, pull +0.24, χ²/dof 1.01
```

and from `cli_decay_chain`:

```
pp_ttx_lep  … ours/MG − 1 = -3.563e-4 ± 1.2e-3 | pull -0.29
ttx_wb      … ours/MG − 1 = +2.439e-4 ± 3.7e-4 | pull +0.66
zz_ee       … ours/MG − 1 = +2.549e-3 ± 7.9e-4 | pull +3.24
zz_emu      … ours/MG − 1 = +8.594e-4 ± 7.5e-4 | pull +1.15
```

These values were not rerun on `main` for a side-by-side comparison. They are
unchanged **by construction** for a card without a duplicate: there the removed check
never fired, and the marking check only decides a refusal. That is an argument, not
a measurement.

Not run: the rest of `validate.sh`'s banked layer (amplitudes, colour, hadronic σ,
samples, MLM). Neither amplitude evaluation nor phase space was touched, and on 4
cores shared with three sessions the full layer did not fit. `validate_hadronic`'s
`mlm_mixed_sigma_row` only changed its call to `group_vetoes`. It compiles and lints
(clippy-ev row) but was not run.

## 3. Structural census against MadGraph

The census input lives in the repository (`validation/madgraph/decay_chain_census.json`,
tracked), not in the refdata bundle, so **no M3 request** is needed: it is pure
Python on the pinned checkout and was regenerated here.

Determinism first: the old corpus regenerated byte-identically.

```
$ cp validation/madgraph/decay_chain_census.json …/H-D2_census_before.json
$ python3 validation/madgraph/dump_decay_chain_census.py
wrote …/decay_chain_census.json: 24 cards, 2 refused by MadGraph
real 0m4.232s
$ cmp …/H-D2_census_before.json validation/madgraph/decay_chain_census.json && echo IDENTICAL
IDENTICAL
```

With the three cards added (`H-D2_census_dump2.log`):

```
gg_zjj_jj: 6 matrix elements, 48 diagrams
ee_zjj_noz_jj: 6 matrix elements, 24 diagrams
pp_zjj_jj: 86 matrix elements, 504 diagrams
wrote …/decay_chain_census.json: 27 cards, 2 refused by MadGraph
```

`git diff --stat` of the JSON: `10977 +++++` and no deletions. MadGraph's side of the
new cases, counted from the JSON (contents keyed by sorted initial and sorted final):

```
gg_zjj_jj 6 distinct 10 twice 6 more 0 inplace dup 0
ee_zjj_noz_jj 6 distinct 10 twice 6 more 0 inplace dup 0
pp_zjj_jj 86 distinct 226 twice 30 more 0 inplace dup 0
```

That is H-D1's `output madevent` census (226 distinct, 30 twice, 86 matrix elements =
6 + 12 + 4 + 64) from the pure-Python path. The gate
(`cargo test -p vibegraph-lib --test decay_chain_census -- --nocapture`,
`H-D2_t1.log`):

```
gg_zjj_jj: 6 subprocesses from two decay assignments; [1, -1, 1, -1]: 8 (+24 permuted), …; [1, -1, 2, -2]: 8 (+0 permuted), … 
ee_zjj_noz_jj: 6 subprocesses from two decay assignments; [1, -1, 1, -1]: 4 (+12 permuted), …; [1, -1, 2, -2]: 4 (+0 permuted), …
pp_zjj_jj: 30 subprocesses from two decay assignments; [1, -1, -4, -4]: 8 (+0 permuted), …; [1, -1, -4, -3]: 4 (+0 permuted), …
27 cards: 24 stitched and matched (357 subprocesses, 42 of them reached by two decay assignments), 3 refused as MadGraph refuses or drops
test decay_chain_census_matches_madgraph ... ok
```

Every subprocess's MadGraph diagram count equals vibegraph's diagrams on MadGraph's
decay blocks, both assignments of all 42 included (e.g. gg: 8 and 8 per content,
H-D1's `ME1`/`ME2`). What it cannot see: anything past the diagram list per forced
pattern, and the cut semantics (§5).

## 4. Flavour grouping of the two assignments

`cargo test -p vibegraph-lib --lib -- each_decay_assignment group_members_must_share --nocapture`
(`H-D2_t2.log`):

```
([21, 21], [-4, -3, 3, 4]): assignments in groups [2, 2]
([21, 21], [-4, -2, 2, 4]): assignments in groups [1, 1]
([21, 21], [-4, -1, 1, 4]): assignments in groups [2, 2]
([21, 21], [-3, -2, 2, 3]): assignments in groups [2, 2]
([21, 21], [-3, -1, 1, 3]): assignments in groups [4, 4]
([21, 21], [-2, -1, 1, 2]): assignments in groups [2, 2]
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1205 filtered out
```

**Convention surprise, measured:** on `g g > z j j, z > j j` the two assignments of
every content have equal `|M|²` (each in its own leg order) to 1e-12 at fresh points,
not just u ↔ c (equal couplings) but also up ↔ down (`Z → uū` on the dd̄ core equals
`Z → dd̄` on the uū core). So each pair shares one group with two luminosity terms.
That is correct for σ, but it means a σ on this card cannot tell the incoherent sum
from "the first assignment counted twice" (H-D1 used the same symmetry). The σ row
below therefore carries a known-wrong control instead (one assignment per content),
and the census and the diagram test carry the per-assignment structure. In the pp
card the qq̄-initial pairs have 8 and 4 diagrams and cannot share a trace.

## 5. σ against MadEvent

### No duplicate card runs end to end through the CLI

Each attempt, through `target/debug/vibegraph integrate … --seed 1 --target-rel 0.01`:

| card | run card | outcome (log) |
|---|---|---|
| `g g > z j j, z > j j` | MadEvent's (lpp 1, 6500, lhaid 247000, ptj 20, etaj 5, drjj 0.4, scales at MZ, `cut_decays F`) | `INFO enumerate: 6 subprocesses … reached by more than one decay assignment …` then `ERROR … groups 0 and 1 compile to different cut filters` (`H-D2_vg_probe.log`) |
| same | same with `cut_decays T` | same `GroupCutsDiffer` (`H-D2_vg_probe_cdT.log`) |
| `p p > z j j, z > j j` | defaults | `30 subprocesses … reached by more than one decay assignment`, then `GroupCutsDiffer` (`H-D2_vg_pp.log`) |
| `e+ e- > z j j / z, z > j j` | H-D1's fixed-beam card | `6 subprocesses …`, then `fixed-energy beams require every subprocess to share the same external particle content` (`H-D2_vg_ee.log`) |

All three now pass enumeration (before: `DuplicateSubprocess`, §6). **No card can
avoid `GroupCutsDiffer` while reaching a duplicate**, so the brief's "use a card that
reaches the duplicate and does not hit `GroupCutsDiffer`" has no solution under the
current cut architecture. If a content is reached by (core C₁, decay D₂) and (C₂, D₁),
the line also generates (C₁, D₁) and (C₂, D₂), which are same-flavour states
(`u u~ u u~`). Their stitched sets carry both pairings, so their window list is a
union of two patterns (and under `cut_decays = F` they exempt no leg), while the
distinct-flavour sets carry one pattern. Their cut filters differ whatever
`cut_decays` is; this is what the `cut_decays T` row measures. Putting the two kinds
in different mass parts needs different masses, but then the two assignments of the
duplicate itself have different forced-leg positions after `content_ordered`, and
differ in turn.

### The σ row: the twelve two-assignment subprocesses at proton beams

The library integrand on the same sets the CLI enumerates. Take the single line
`g g > z j j, z > j j` and drop the four same-flavour sets, keeping the twelve
distinct-flavour ones (both assignments of each of the six contents). Then
`derive_flavor_groups` (3 groups, members `[2, 8, 2]`), `ProtonIntegrand`,
`use_run_card_scales`, `adapt_alphas(seed, 40000, 6, 0.5)` and `adapt_grids_budget`
(target 2e-3, neval 120k, Neyman), as `tests/decay_chain_ladder.rs` drives it. The
harness is `H-D2/hd2_scratch_sigma.rs`. MadEvent side: the twelve assignments as
twelve process lines (`H-D2_gg_dup_out.mg5`, `generate g g > z u u~, z > c c~` …
`add process g g > z s s~, z > d d~`; 12 P-directories, `NGRAPHS = 8` each), which
MadGraph sums. Same run card on both sides, except that vibegraph refuses
`pdlabel = nn23lo1` (*"vibegraph reads parton densities only from LHAPDF6 grids. Set
pdlabel = lhapdf and lhaid … 247000"*). LHAPDF is not installed for MadGraph here, so
MadEvent reads its built-in `nn23lo1` and vibegraph reads LHAPDF 247000. A
single-assignment calibration card measures that offset.

Commands: `H-D2/H-D2_launch_dup.sh` (MadEvent, `launch … set iseed $s … set nevents
5000`), and
`HD2_RUN_CARD=…/H-D2_rc_gg.dat cargo test -p vibegraph-lib --test hd2_scratch_sigma -- --ignored --nocapture`.

| row | side | per seed (pb) | unweighted mean ± sd/√5 | seed sd | χ²/dof of seeds against quoted errors |
|---|---|---|---|---|---|
| 12 two-assignment subprocesses | MadEvent, seeds 51–55 | 112.1 ± 0.53, 111.7 ± 0.50, 112.1 ± 0.48, 111.2 ± 0.45, 111.4 ± 0.45 | **111.70 ± 0.18** | 0.41 | 0.73 |
| | vibegraph, seeds 1–5 | 112.140 ± 0.180, 112.150 ± 0.187, 112.077 ± 0.196, 111.868 ± 0.214, 111.945 ± 0.214 | **112.036 ± 0.056** | 0.12 | 0.39 |
| known-wrong control: one assignment per content (the refusal turned into a silent dedupe) | vibegraph, seed 1 | 56.049 ± 0.092 | — | — | — |
| calibration: `g g > z u u~, z > c c~` (one assignment, no duplicate) | MadEvent, seeds 61–65 | 7.207, 7.159, 7.216, 7.167, 7.148 (± 0.032–0.046) | **7.1794 ± 0.0135** | 0.030 | 0.68 |
| | vibegraph, seeds 1–5 | 7.1991, 7.2171, 7.2109, 7.1997, 7.1850 (± 0.012–0.014) | **7.2024 ± 0.0055** | 0.012 | 0.98 |
| full card `g g > z j j, z > j j` (for D4) | MadEvent, seeds 41–45 | 148.8 ± 1.05, 149.9 ± 0.61, 149.7 ± 0.62, 150.6 ± 0.78, 149.5 ± 0.75 | 149.70 ± 0.29 | 0.65 | — |

Reading:

- Two-assignment row: vibegraph − MadEvent = +0.336 ± 0.190 pb, **+0.30%, 1.8σ**.
- Calibration row, same kinematics without a duplicate: +0.023 ± 0.015 pb, **+0.32%,
  1.6σ**. The offset is the same, so it belongs to the PDF path (`nn23lo1` against the
  247000 grid; `manifest.toml:151` records that those two are different densities)
  or to the integrators, not to duplicate handling.
- The ratio (two-assignment / single) is 15.5584 (MadEvent) against 15.5554
  (vibegraph): **−0.02%**.
- The known-wrong control reads 56.05 pb, half of MadEvent's 111.70. It is what the
  refusal would give as a silent dedupe, and the row sees it at 300σ.
- The vibegraph two-assignment seeds scatter less than their quoted errors
  (χ²/dof 0.39 over 5 seeds); the calibration row's is 0.98. Five seeds cannot
  calibrate this, so it is a caution and not a finding.

What this row cannot see: the per-assignment `cut_decays = F` exemption against
MadGraph's for the *same-flavour* sets, and any cross-group cut handling. Both wait
for D4, and the full-card MadEvent number above is banked locally for that comparison.
It also cannot tell the incoherent sum from one assignment counted twice on this card
(§4). The census and the diagram test pin that structure.

## 6. Tests that fail before the change, and mutations they catch

**Before (M0).** `chain.rs` from `HEAD` (`git show HEAD:…/chain.rs >
…/chain.rs`), everything else as committed (`H-D2_mut0.log`):

```
test decay_chain_census_matches_madgraph ... FAILED
unexpected enumeration error: subprocess 'g g > u u~ c c~' is produced by two process lines ('g g > z j j, z > j j' and 'g g > z j j, z > j j'); both would be added to the cross section, so the card is refused rather than counting it twice
test a_subprocess_from_two_decay_assignments_is_two_subprocesses ... FAILED
called `Result::unwrap()` on an `Err` value: "subprocess 'g g > u u~ c c~' is produced by two process lines ('g g > z j j, z > j j' and 'g g > z j j, z > j j'); …"
test result: FAILED. 21 passed; 1 failed
```

**Mutation M1: the refusal replaced by a silent dedupe** (keep the first set per
(sorted initial, sorted final)), inserted before the logging block
(`H-D2_mut1.log`, `--no-fail-fast`):

```
test decay_chain_census_matches_madgraph ... FAILED
  gg_zjj_jj: subprocesses differ:
  ee_zjj_noz_jj: subprocesses differ:
  pp_zjj_jj: subprocesses differ:
test a_subprocess_from_two_decay_assignments_is_two_subprocesses ... FAILED
  left: 10
 right: 16
EXIT 101
```

Both restored (`cmp` of `chain.rs` against the saved fixed copy: `restored`), then
the gates in §2 ran on the committed tree.

**What stays refused**: two process lines reaching one content. This is
`two_assignments_on_two_lines_are_refused`, on H-D1's equal-tag card
`generate e+ e- > z u u~, z > c c~ @1` / `add process e+ e- > z c c~, z > u u~ @1`.
It asserts the `DuplicateSubprocess` message naming `e+ e- > u u~ c c~` and both
lines. It passes on the committed tree (§2). MadGraph sums this card (H-D1 §2), so the
refusal is a recorded departure; see Found.

## 7. M3 requests

None for this session's oracles: the census is committed and pure Python, and the σ
rows ran on local MadEvent. H-D1 §7's two requests stand unchanged: the literal survey
cards 024/029/068, and a banked `Zto2Q`-like σ row for D4.

## 8. Corrections to the brief

- *"Use a card that reaches the duplicate and does not hit `GroupCutsDiffer`"*: no
  such card exists (§5). The σ row runs the twelve two-assignment subprocesses of the
  single-line enumeration through the library integrand.
- *"H-D1's e+e− and gg cards … check whether they pass end to end through the CLI
  now"*: they pass enumeration and then stop: fixed beams at mixed content, proton
  beams at `GroupCutsDiffer` (§5).
- *"Use pdlabel nn23lo1"*: for MadEvent, yes. vibegraph refuses `nn23lo1`, so the
  comparison is `nn23lo1` against LHAPDF 247000, calibrated by the single-assignment
  row (+0.32%).
- *"If the census gate's inputs must live in the banked refdata bundle"*: they do not;
  the JSON is tracked and is committed here.
- *"Your PR deletes the item's file once `closes_when` holds"*: the item's
  `closes_when` names cards 024, 029 and 068 measured on the pinned checkout. H-D1
  showed 024's stated form has no duplicate, and the literal cards are not in this
  repository (H-D1 §7.1, an M3 request). What this session closes is the mechanism:
  MadGraph measured, vibegraph matching it with census rows. The literal-card clause
  does not hold. The survey concept also links the item file
  (`validation/cms-gridpack-survey-2026-10.md:241`), so deleting it needs a kb edit
  outside this session's remit. **The item file is left in place** for the manager
  to close or narrow.

## Found

- **Cross-line duplicates of distinct assignments are refused where MadGraph sums
  them.** `generate e+ e- > z u u~, z > c c~ @1` + `add process e+ e- > z c c~, z >
  u u~ @1` is refused by `diagrams/mod.rs:433` with *"…would be added to the cross
  section, so the card is refused rather than counting it twice"*. The two lines are
  different assignments, and MadGraph sums them as two PD terms of one matrix element
  (H-D1 §2). Evidence: `two_assignments_on_two_lines_are_refused`. Closing it means
  telling two lines that give one content through different forced lines (keep both,
  as within a line) from one subprocess written twice (refuse), for example by the
  forced-line pattern of the two sets.
- **Every hadronic duplicate-assignment card needs per-group cut indicators (D4).**
  The same-flavour sets that any duplicate card also generates carry a union of
  window patterns, so their filters differ from the distinct-flavour sets' under
  either `cut_decays` (§5). Closing it is D4. The MadEvent full-card number
  149.70 ± 0.29 pb (`H-D2/H-D2_gg_zjj`, seeds 41–45, run card `H-D2_rc_gg.dat` with
  `pdlabel nn23lo1`) is the first row to compare once it lands, after the PDF
  calibration below.
- **The `nn23lo1` against LHAPDF 247000 offset is +0.32% on `g g > z u u~, z > c c~`
  at 13 TeV** (§5, 1.6σ on 5 + 5 seeds). Local MadEvent rows compared against
  vibegraph carry it until MadGraph can read LHAPDF here. Closing it takes installing
  LHAPDF for the local MadGraph, or banking such rows on the M3 with `pdlabel = lhapdf`.
- **Decay-chain process strings omit `/ X`.** The log of `e+ e- > z j j / z, z > j j`
  prints `'e+ e- > z j j, z > j j'` (`H-D2_vg_ee.log`). This is documented behaviour of
  `SupportedProcess`'s `Display` ("the forbidden particles are not printed"), but it
  makes two lines that differ only in `/` print identically in refusal messages.
  Closing it takes printing the forbidden list, or a note in the refusals that quote it.
