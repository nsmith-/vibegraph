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
| F10 | 4 | `generate.rs:1550,1557`, `2062-2072`, `generate/source.rs:429` | missing alias | `Vec<Vec<Vec<Vec<LineCode>>>>` is spelled three times, twice under `#[allow(clippy::type_complexity)]`. *checked* | fix: one `pub(super) type MemberCodes` in `generate/source.rs` beside the field it types; both `allow`s go |
| F11 | 3 | `lhef/resonance.rs:401-411` | test-only scaffolding (brief's question) | `diagram_lines()` + `from_diagram_lines()` are `cfg(test)` and used only by two `generate::tests` that build a synthetic member (top line dropped; top relabelled to two codes). No production constructor can build a member lacking a line its representative has, and a real diagram set without that line (`/ t t~`) would change the fixture under test. *checked* | leave: the pair is the narrowest shape for a synthetic member; `cfg(test)` keeps it out of production |
| F12 | 3 | `lhef/resonance.rs:612` `pub(crate) mod tests` | test-only visibility | Made `pub(crate)` for `top_pair`; the same pattern is in `cache/lhapdf.rs:345`, `ufo/card_classes.rs:568`. *checked* | leave: an established in-crate pattern |
| F13 | 1 | `widths.rs:220-230` `anti`, `233-239` `particle_with_code` | duplicated code across modules | `diagrams/diagram.rs:1019` `antiparticle` (name-based, `expect`s) and `diagrams/resolve.rs:110` `particle_by_pdg` (no physical filter) do the same lookups with different failure semantics. *checked* | Found: unifying them changes a panic/fallback contract outside this change |
| F14 | 3 | `widths.rs:122-125` | pub field beside derived state | `ComputedWidth::bound` is `madgraph_rounded(total)`, both `pub`. A serialized record built in one place (`two_body_width`); getters would add API for no reader. *checked* | leave |
| F15 | 4 | `ufo/report.rs:382` | knob | `describe_parameters` is now `describe_parameters_at(.., &[])` and its only callers are `report.rs`'s tests. A pre-existing public library entry point. *checked* | leave |
| F16 | 2 | `generate/source.rs:330-333` `matched_event_record`'s unresolved path; `generate.rs:1616` the widthless `Ambiguous` arm | blind sites | No test drives a matched (`ickkw = 1`) event of a member without a written line, or a widthless ambiguous line. Mutations M20, M21 settle whether any test reaches them. *suspected* | see the mutation table |

No comment in either diff breaks AGENTS.md's comment rules: `git show <sha> --
'*.rs' \| grep '^+' \| grep -nE '\bT[0-9]\b\|\bC[0-9]\b\|Phase [0-9]\|note
[0-9]+\|used to\|no longer\|sprint\|session\|backlog\|PR [0-9]'` is empty for
both. No function the change adds exceeds 100 lines (`two_body_width`, the
longest, is 95).
