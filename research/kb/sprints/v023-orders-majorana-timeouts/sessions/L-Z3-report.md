---
type: Session Report
title: "L-Z3 report: hygiene pass on Auto widths and per-member resonance records"
description: "Findings, mutations, fixes and identity oracles for the hygiene pass over sprint L's PR 9 (#179, Auto widths) and PR 15 (#188, per-member resonance codes)."
status: draft
tags: [hygiene, widths, resonance, generate, mutation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-widths` off main 8b3af5ab (brief commit 0a1698b7). Scope:
the code, tests and docs of e99f8ba2 (#179: `widths.rs`, `ufo/bind.rs`,
`ufo/report.rs`, `ufo/slha.rs`, the CLI's `bind_param_card`, `parameters`) and
5a1c33e8 (#188: `generate.rs` `group_resonances`/`member_line_codes`,
`generate/source.rs`, `lhef/resonance.rs` `MemberLines`).

No backlog item names a path of this scope: `grep -rln -e '/widths.rs' -e
'/bind.rs' -e '/report.rs' -e '/resonance.rs' -e 'generate/source.rs' -e
'/generate.rs' research/kb/backlog` returns nothing.

## What changed

- `widths.rs`: `METHOD`, `QCD_SCALE_FLOOR`, `madgraph_rounded` private (F6);
  `decaying_particle` is the one lookup of a `DECAY` line's particle (F7); a
  width's total is summed from `0.0`, so a stable particle's computed width is
  `0`, not `-0` (F18, a bug).
- `ufo/bind.rs`: `auto_width_params` uses `decaying_particle` (F7);
  `a_given_table_is_replaced_and_a_stable_particle_is_zero` also resolves the
  card text with a given table and rebinds it (F3), and pins the stable
  width's sign and its `DECAY` line (F18).
- `tests/auto_widths.rs`: new
  `a_coloured_parents_channel_below_the_qcd_scale_is_left_out` (F4).
- `lhef/resonance.rs`: `LineCode`, `MemberCode`, `MemberLines` (`new`,
  `resolve`) and `member_line_pdg` are `pub(crate)` (F5);
  `ResonanceLine::code()` replaces `line_code` (F8).
- `generate/source.rs`, `generate.rs`: `member_pdgs` → `member_codes`, `pdgs`
  → `codes` (F9); `MemberCodes` alias (F10); `event_record` and
  `matched_event_record` docs say code and colour and the no-code fallback
  (F17); `a_widthful_line_under_several_codes_the_legs_cannot_tell_apart_is_refused`
  also pins that a widthless line in that case holds no code (F16).

Tests added: `a_coloured_parents_channel_below_the_qcd_scale_is_left_out`.
None deleted. This pass closes no backlog item, so no item file is deleted.

## Findings (written before any edit)

Ranked by consequence. *checked* means the quoted evidence settles it;
*suspected* findings are settled by the mutation batch below.

| id | pt | site | class | evidence | disposition |
|---|---|---|---|---|---|
| F1 | 2 | `tests/auto_widths.rs:248-254`, `ufo/bind.rs` `an_auto_width_binds_like_the_number_it_computes` | vacuous test | The bound value's seven-digit rounding (`bound: madgraph_rounded(total)`, `widths.rs:522`) is checked only as `format!("{:.6e}", ours.bound) == format!("{:.6e}", printed)` and `(bound/total - 1).abs() < 5e-7`; an unrounded `bound` formats to the same string and sits inside that bound. Mutation M8 (`bound: total`) should survive. *suspected* | fix: compare `ours.bound == printed` exactly; assert `bound` is its own seven-digit rounding and differs from `total` |
| F2 | 2 | `widths.rs:149` `decay_table` | untested contract | The doc promises each branching ratio "to the seven digits MadGraph writes"; the one test reading the table compares it with a card typed from the same numbers (`{:e}`, full precision). Mutation M9 (drop `madgraph_rounded` on the ratio) should survive. *suspected* | fix: assert each ratio is its own seven-digit rounding |
| F3 | 2 | `widths.rs:632` `resolved_card_text` | untested branch | The given table below an `Auto` line is dropped (`dropping`), but every test calls it on `"DECAY 24 Auto\n"` with no table (`bind.rs` test, `cli_param_card.rs` `an_auto_width_integrates_as_the_card_it_banks`). Mutation M11 (`dropping = false`) should survive. *suspected* | fix: resolve and rebind a card with a given table |
| F4 | 2 | `widths.rs:505` below-`QCD_SCALE_FLOOR` rule | untested branch | No hermetic test has a coloured parent with an open channel under 0.1 GeV (SM default: `t > W+ b` only); the MadGraph comparison chains `channels` with `below_qcd_scale` (`auto_widths.rs:211`), so it is blind to which list a channel lands in. Mutation M14 (`color.abs() != 1` → `false`) should survive. *suspected* | fix: new test on `restrict_ckm`'s top (`t > W+ s`, `t > W+ d`) |
| F5 | 3 | `lhef/resonance.rs:343,361,383,389,422,512` | public item that should be crate-private | `LineCode`, `MemberCode`, `MemberLines` (`new`, `resolve`) and `member_line_pdg` are `pub` in the public `lhef::resonance`; `grep -rn 'member_line_pdg\|MemberLines\|LineCode\|MemberCode'` outside `lhef/resonance.rs`, `generate.rs`, `generate/source.rs` returns nothing (no CLI, test-target or `validation-report` user). *checked* | fix: `pub(crate)` |
| F6 | 3 | `widths.rs:74,78,394` | public item that should be private | `METHOD`, `QCD_SCALE_FLOOR`, `madgraph_rounded`: `grep -rn '\bMETHOD\b\|QCD_SCALE_FLOOR\|madgraph_rounded'` outside `widths.rs` returns nothing. *checked* | fix: private |
| F7 | 1 | `ufo/bind.rs:467-470` | duplicated code | `auto_width_params` re-spells `widths::particle_with_code` (`widths.rs:233-239`) plus `two_body_width`'s `pdg > 0` filter (`widths.rs:451-453`): `.find(\|p\| pdg > 0 && p.pdg_code == i64::from(pdg) && !p.is_unphysical())`. *checked* | fix: one `pub(crate)` lookup in `widths` used by both |
| F8 | 1/4 | `generate/source.rs:265-268` | duplicated code | `LineCode { pdg: lines[i].pdg, color: lines[i].color }` is `lhef/resonance.rs:463` `line_code(line)`. *checked* | fix: `ResonanceLine::code()`, used by both |
| F9 | 1 | `generate.rs:1133,1141`, `generate/source.rs:429,662`; `group_resonances`'s `pdgs`; `event_record`'s `pdgs` | stale name | The field and locals named `member_pdgs`/`pdgs` hold `LineCode`s (code and colour) since #188; the doc on `member_pdgs` already says "PDG code and colour". *checked* | fix: rename to `member_codes`/`codes` |
| F10 | 4 | `generate.rs:1550,1557`, `2062-2072`, `generate/source.rs:429` | missing alias | `Vec<Vec<Vec<Vec<LineCode>>>>` is spelled three times, twice under `#[allow(clippy::type_complexity)]`. *checked* | fix: one `pub(super) type MemberCodes` in `generate/source.rs` beside the field it types; `group_resonances`'s `allow` goes (the test helper's three-tuple still trips the lint and keeps its own) |
| F11 | 3 | `lhef/resonance.rs:401-411` | test-only scaffolding (brief's question) | `diagram_lines()` + `from_diagram_lines()` are `cfg(test)` and used only by two `generate::tests` that build a synthetic member (top line dropped; top relabelled to two codes). No production constructor can build a member lacking a line its representative has, and a real diagram set without that line (`/ t t~`) would change the fixture under test. *checked* | leave: the pair is the narrowest shape for a synthetic member; `cfg(test)` keeps it out of production |
| F12 | 3 | `lhef/resonance.rs:612` `pub(crate) mod tests` | test-only visibility | Made `pub(crate)` for `top_pair`; the same pattern is in `cache/lhapdf.rs:345`, `ufo/card_classes.rs:568`. *checked* | leave: an established in-crate pattern |
| F13 | 1 | `widths.rs:220-230` `anti`, `233-239` `particle_with_code` | duplicated code across modules | `diagrams/diagram.rs:1019` `antiparticle` (name-based, `expect`s) and `diagrams/resolve.rs:110` `particle_by_pdg` (no physical filter) do the same lookups with different failure semantics. *checked* | Found: unifying them changes a panic/fallback contract outside this change |
| F14 | 3 | `widths.rs:122-125` | pub field beside derived state | `ComputedWidth::bound` is `madgraph_rounded(total)`, both `pub`. A serialized record built in one place (`two_body_width`); getters would add API for no reader. *checked* | leave |
| F15 | 4 | `ufo/report.rs:382` | knob | `describe_parameters` is now `describe_parameters_at(.., &[])` and its only callers are `report.rs`'s tests. A pre-existing public library entry point. *checked* | leave |
| F18 | 1 (bug) | `widths.rs:515` | wrong value written | A particle with no open channel gets `total = -0.0`: `channels.iter().map(\|c\| c.width).sum()` over an empty list is `-0.0` (the float `Sum` starts from negative zero). Main's binary on `Block MASS 25 1.0` + `DECAY 25 Auto` (`vibegraph parameters … --param-card h_light.dat`) logs `WH = -0 GeV` and writes `"total": -0.0, "bound": -0.0` into `parameters.json`; the bind test's `assert_eq!((width.total, width.bound), (0.0, 0.0))` cannot see the sign. Found by the identity-oracle run, before any edit. *checked* | fix: sum from `0.0`; pin the sign. Not physics (`-0.0 == 0.0`; every non-empty total keeps its bits, since `-0.0 + x == x` exactly) |
| F17 | 1 | `generate/source.rs:235-239`, `298` | doc does not match body | `event_record`'s doc says each status-2 line carries "the PDG code `pdgs[config][line]`"; since #188 it carries a code and colour, and a line the member holds no code for (`LineCode::NONE`) writes the event without intermediates and counts it (`tally.unresolved`), which neither doc says. *checked* | fix: the docs say both |
| F16 | 2 | `generate/source.rs:330-333` `matched_event_record`'s unresolved path; `generate.rs:1616` the widthless `Ambiguous` arm | blind sites | No test drives a matched (`ickkw = 1`) event of a member without a written line, or a widthless ambiguous line. Mutations M20, M21 settle whether any test reaches them. *suspected* | see the mutation table |

No comment in either diff breaks AGENTS.md's comment rules: `git show <sha> --
'*.rs' \| grep '^+' \| grep -nE '\bT[0-9]\b\|\bC[0-9]\b\|Phase [0-9]\|note
[0-9]+\|used to\|no longer\|sprint\|session\|backlog\|PR [0-9]'` is empty for
both. No function the change adds exceeds 100 lines (`two_body_width`, the
longest, is 95).

## Final dispositions

| id | disposition | evidence |
|---|---|---|
| F1 | **not reproduced** | M8 (`bound: total`) fails `ufo::bind::tests::an_auto_width_binds_like_the_number_it_computes` (`WW: 2.0479100513132478+0i against 2.04791+0i`: the rebound card text carries seven digits) and `cli_param_card::an_auto_width_integrates_as_the_card_it_banks` (`auto.record() == typed.record()`). The round trip through the written card pins the rounding; the format-string comparison in `auto_widths.rs` is weak but not the only guard. Left alone. |
| F2 | **not reproduced** | M9 fails the same two tests (the rebound table carries `0.3333605` where the unrounded one has `0.33336046149070625`). Left alone. |
| F3 | **fixed** | M11 survived every test before (73 ok, 0 FAILED); after, it fails `a_given_table_is_replaced_and_a_stable_particle_is_zero` (the given ` 0.5 2 12 -11` rows survive into the resolved text). |
| F4 | **fixed** | M14 survived every test before; after, it fails the new `a_coloured_parents_channel_below_the_qcd_scale_is_left_out` (`below_qcd_scale` empty). Brief correction to my own finding: binding refuses `restrict_ckm`'s top (`t > W+ Z s (through t)`, `t > W+ Z d`), so the test reads `two_body_width` and `ComputedWidth::decay_table` rather than binding. |
| F5 | **fixed** | `pub(crate)`; workspace builds and clippy are green (Gates). |
| F6 | **fixed** | private. |
| F7 | **fixed** | `widths::decaying_particle` (`particle_with_code(..).filter(\|_\| pdg > 0)`), used by `two_body_width` and `bind::auto_width_params`. Same particle: both take the first physical particle with that code. |
| F8 | **fixed** | `ResonanceLine::code()` replaces `line_code` and the literal in `event_record`. |
| F9 | **fixed** | `member_codes`, `codes`. |
| F10 | **fixed** | `MemberCodes` alias; `group_resonances` loses its `allow`, the test helper `resolved_groups` keeps its own (its three-tuple still trips `type_complexity`; clippy said so when I removed it). |
| F11 | leave | as in the table. |
| F12 | leave | as in the table. |
| F13 | **Found** | below. |
| F14 | leave | as in the table. |
| F15 | leave | as in the table. |
| F16 | **fixed** (widthless arm), **Found** (matched path) | M21 survived before; after, it fails `a_widthful_line_under_several_codes_the_legs_cannot_tell_apart_is_refused` (`a line without a width under several codes does not refuse the run: Other("the resonance 22 of e+ e- > b e+ ve b~ mu- vm~ has no single flavour …")`). M20 (`matched_event_record` stops counting an unresolved event) survives before and after: no test drives a matched record of a member without a written line. |
| F17 | **fixed** | both docs now say code and colour, and the no-code fallback. |
| F18 | **fixed** (bug) | the fold from `0.0`; M27 (restore `Iterator::sum`) fails `a_given_table_is_replaced_and_a_stable_particle_is_zero` (`total: -0.0, bound: -0.0`), which also checks the resolved card starts `DECAY  25   0.000000e+00 `. Before the fix the same assertions fail on the unmodified code: M27 *is* main's code at that line. |

## Mutations

Driver: `/home/user/wt/lz3-oracle/mut/instrument.py` rewrites each site so it
behaves as the unmutated code unless `LZ3_MUT` names its id (one build per
batch instead of one per mutation, under a shared heavy lock), `revert`
restores the backed-up files byte for byte; `fast.sh` runs the prebuilt test
executables once per id. Per id, it runs the lib unit tests filtered to
`ufo::bind:: ufo::report:: lhef::resonance:: generate::`, all of
`auto_widths` with `--include-ignored` (the g2HDM row ran against
`VG_G2HDM_UFO=/home/user/wt/sprintl-auto-widths-mg/g2HDM_UFO`),
`ufo_input_refusals::evaluating_a_model_at_a_card_with_an_auto_width_is_refused`,
`cli_param_card`'s two Auto tests and `cli_generate_vanishing_group`. Results
in `/home/user/wt/lz3-oracle/mut/{pre,post}/summary.txt`, logs
`/home/user/wt/logs/lz3-mut-{pre,post}-<id>.log`. Baseline: 73 ok (pre), 74 ok
(post, one new test).

| id | mutation | before the fixes | after |
|---|---|---|---|
| M1 | `outgoing_symmetry_factor` → `1.0` | 5 FAILED incl. `identical_daughters_take_the_symmetry_factor`, `a_scalar_into_identical_scalars…`, g2HDM row | same |
| M2 | channel opens at `M = m1 + m2` | 10 FAILED incl. `a_channel_at_threshold_is_closed_and_opens_above_it` | 11 |
| M3 | a closed channel is not recorded | 7 FAILED incl. `a_closed_channel_is_listed_and_adds_nothing` | same |
| M4 | `initial_spin_color_average` → `1.0` | `massless_daughters_take_the_analytic_width`, `sm_auto_widths_match_madgraph_per_channel` | same |
| M5 | the `α_s` refusal never fires | `a_channel_running_with_alpha_s_is_refused` | same |
| M6 | a coupling reading another `Auto` width is not refused | `a_channel_reading_another_auto_width_is_refused` | same |
| M7 | `compute_width` binds a particle with three-body modes | 5 FAILED incl. `sm_top_and_w_bind_and_z_and_higgs_are_refused`, `cards_the_model_cannot_honour_are_refused` | same |
| M8 | `bound: total` (no rounding) | `an_auto_width_binds…`, `an_auto_width_integrates_as_the_card_it_banks` | same |
| M9 | branching ratio unrounded | the same two | + `a_given_table_is_replaced…` |
| M10 | a given table's rows are all accepted | `auto_widths_that_cannot_be_computed_are_refused_naming_why` | same |
| M11 | `resolved_card_text` keeps the given table (`dropping = false`) | **survives** | `a_given_table_is_replaced_and_a_stable_particle_is_zero` |
| M12 | the report carries no `computed_width` | `an_auto_width_is_reported_on_its_record` | same |
| M13 | `with_widths_set` keeps an empty table | `a_given_table_is_replaced_and_a_stable_particle_is_zero` | same |
| M14 | no channel is below the QCD floor | **survives** | `a_coloured_parents_channel_below_the_qcd_scale_is_left_out` |
| M15 | a massless emission off the parent's own line counts as a mode | 7 FAILED incl. `sm_top_and_w_bind…` | same |
| M16 | a member without the configuration's diagram never falls back to its other lines | `a_member_line_takes_its_own_diagrams_code` | same |
| M17 | `same_line` compares colour exactly, not up to conjugation | `a_live_groups_quark_line_carries_each_members_own_flavour` | same |
| M18 | `member_code` returns `LineCode::NONE` as a code | `a_member_code_is_read_only_where_the_member_has_one`, `a_member_without_a_widthful_line…` | same |
| M19 | a vanishing group resolves codes | `a_group_that_vanishes_at_the_card_resolves_no_resonance_codes` | same |
| M20 | `matched_event_record` does not count an unresolved event | **survives** | **survives** (Found) |
| M21 | a widthless ambiguous line refuses | **survives** | `a_widthful_line_under_several_codes_the_legs_cannot_tell_apart_is_refused` |
| M22 | the unresolved warning is dropped | `a_member_without_a_widthful_line_writes_no_intermediate_and_counts_it` | same |
| M24 | `from_model_card` evaluates an `Auto` card | `evaluating_a_model_at_a_card_with_an_auto_width_is_refused` | same |
| M26 | `integrate` banks the card text without the computed widths | `an_auto_width_integrates_as_the_card_it_banks` | same |
| M27 | the total is `Iterator::sum` again (`-0.0` for no channel) | (main's code; not a mutation there) | `a_given_table_is_replaced_and_a_stable_particle_is_zero` |

Every test the two PRs added fails under at least one mutation of the code it
guards, except as follows. `cli_generate_vanishing_group` failed under none:
its own header says it cannot see the flavour of a line the record never
writes, and M19 (the one mutation aimed at its path) is caught by the unit test
first, with the CLI test still passing, so the CLI test pins only that
generation is not blocked. M20's site is reached by no test.

What the new and tightened tests cannot detect: the floor test reads one
parent (the top) at the restriction's default CKM, so a floor applied to a
colourless parent would pass it only if such a parent had a channel below
0.1 GeV, which none in the SM has; the `-0.0` pin covers the binding path and
`resolved_card_text`, not `parameters.json` directly (the identity oracle
shows that file); the widthless-ambiguous check is synthetic (relabelled
lines), as the widthful one already was.

## Identity oracle

Chosen before the first edit. Main's binary was built from this branch at
0a1698b7 (main 8b3af5ab plus the brief; its build ran after the findings
commit 2f9b59d2, which `git describe` shows in `version`; neither commit
touches code) into `/home/user/wt/lz3-main-target`, copied to
`/home/user/wt/lz3-oracle/vibegraph-main`; the head binary is the fixed tree's
`target/debug/vibegraph`.

**Width reports.** `/home/user/wt/lz3-oracle/widths.sh <label> <binary>` runs
`vibegraph parameters <proc> --param-card <card> --ufo-dir … --out …` on
eleven rows: SM `DECAY 6/24/23/25 Auto`, `6`+`24`, `24` with a given table,
a light Higgs (`Block MASS 25 1.0`), the SM model-only report with `24`, the
`auto_widths.rs` toy model (`S` and `U`, written from the test's constants to
`/home/user/wt/lz3-oracle/widths/toy`) and g2HDM's
`auto_widths_g2hdm_m1000_param_card.dat` (H2, H3, H+). It keeps
`parameters.json`, `param_card.dat`, stdout, stderr and the exit code. 49 files
compared after removing the `version` field, wall-clock timings and the output
directory name:

```
$ diff <(norm main) <(norm head)   # per file, 49 files
DIFF sm_h_light/parameters.json
<         "total": -0.0,
<         "bound": -0.0,
>         "total": 0.0,
>         "bound": 0.0,
DIFF sm_h_light/stderr
<  INFO h_light.dat: DECAY 25 Auto: WH = -0 GeV, the tree-level two-body width over 0 open channels
>  INFO h_light.dat: DECAY 25 Auto: WH = 0 GeV, the tree-level two-body width over 0 open channels
```

That is F18 and nothing else: the t, W, toy `S`/`U` reports and the Z, H and
g2HDM refusal messages are byte-identical.

**Events.** `/home/user/wt/lz3-oracle/gen.sh integrate` integrated each row
once with main's binary (`--seed 11 --fixed-budget`; `pp_ttx_lep` 20000 × 2
with `decay_chain_pp13_run_card.dat`, `ttx_nested` 5000 × 1 with
`decay_chain_ee500_run_card.dat`, cards copied from L-G2's bit directory);
`gen1.sh` generated 500 events `--seed 7 -j 4` from each artifact with each
binary:

```
gen pp_ttx_lep main rc=0 sha=be3f59c2a1457820 events=500 status2=1990
gen ttx_nested main rc=0 sha=29ee83fa961dc52f events=500 status2=1500
gen pp_ttx_lep head rc=0 sha=be3f59c2a1457820 events=500 status2=1990
gen ttx_nested head rc=0 sha=29ee83fa961dc52f events=500 status2=1500
cmp pp_ttx_lep IDENTICAL
cmp ttx_nested IDENTICAL
```

The sha256 prefixes are also L-G2's base values for the same rows.

## Found

- **F13: three copies of a particle lookup by PDG code, and two of the
  antiparticle lookup.** `widths.rs` `particle_with_code` (physical only) and
  `anti` (by PDG code, falling back to the particle itself);
  `diagrams/resolve.rs:110` `particle_by_pdg` (any particle);
  `diagrams/diagram.rs:1019` `antiparticle` (by `antiname`, `expect`s it
  present). To close: one lookup each on `UFOModel`, with the failure
  behaviour chosen once (the `expect` is a panic on a malformed model).
- **F16: `matched_event_record`'s no-code path is reached by no test.** M20
  (stop counting an unresolved matched event) passes every test before and
  after this pass. To close: a matched (`ickkw = 1`) record built for a member
  whose codes hold `LineCode::NONE` on a clustered line, asserting no status-2
  line and the warning count, as `a_member_without_a_widthful_line…` does for
  the unmatched writer.

No long-run requests.

## Method

Rough shares: reading the two squash diffs against the four points, 30%; the
batched mutation runs, 35%; the identity oracle, 25%; fixes, 10%. The
env-gated instrumentation (every site behaves as the original unless
`LZ3_MUT` names it) made 25 mutations cost one build per batch, which mattered
under a heavy lock shared with two siblings; the per-mutation `cargo test`
driver queued behind the lock for every invocation and was abandoned after its
baseline row.

Leads that did not hold up: F1 and F2. Reading the assertions, the seven-digit
rounding looked unpinned; the round trip through the written card catches
both mutations. The lead that found a real bug (F18) was the identity oracle
itself: running main's binary on the light-Higgs card showed `-0`.

## Gates

On the fixed tree, driver `/home/user/wt/lz3-oracle/gates.sh` (results
`/home/user/wt/lz3-oracle/gates.txt`, logs `/home/user/wt/logs/lz3-gate-*.log`),
every cargo under `flock /home/user/wt/heavy.lock`:

| command | result |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | rc 0, `Finished` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | rc 0, `Finished` |
| `cargo test -p vibegraph --test cli_generate_vanishing_group` | `ok. 1 passed` |
| `cargo doc -p vibegraph-lib --no-deps --document-private-items` | rc 0, 13 warnings, none in a file this pass touches (`coupling/`, `helas/eval/`, `phasespace/channel.rs`, `diagrams/squared.rs`, `lhef/mod.rs`'s ambiguous `write` link) |
| `cargo doc -p vibegraph --no-deps --document-private-items` | rc 0, no warning |
| `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_lhef` (the LHEF writer the resonance records go through) | `ok. 3 passed`, 78.6 s |
| `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_decay_chain_events` (status-2 records against MadEvent's) | `ok. 7 passed`, 51.4 s |
| `python3 scripts/kb.py lint` | rc 0 |
| `cargo test --workspace` with `research/refs/mg5amcnlo`, `validation/madgraph/output`, `validation/pdf` moved aside (restored after; `ls` confirms) | rc 0: 2353 passed, 0 failed, 28 ignored |

`auto_widths` (all 11, the g2HDM row with `--include-ignored`) and the lib
unit tests of `ufo::bind`, `ufo::report`, `lhef::resonance`, `generate` ran as
the `BASE` row of the post-fix mutation batch: 74 ok, 0 failed.
