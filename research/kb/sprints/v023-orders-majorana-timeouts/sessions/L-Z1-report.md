---
type: Session Report
title: "L-Z1 report: hygiene pass on squared orders and per-event-helicity reweighting"
description: "Findings, mutations and fixes for sprint L's PR 1 (#168, squared-order split) and PR 10 (#182, per-event-helicity reweighting), with the bit-for-bit identity oracle against main."
status: draft
tags: [hygiene, squared-orders, reweight, helicity, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-amp` off main 8b3af5ab. Scope: the squash commits
`4b132cc5` (#168) and `dfdf85d4` (#182), their direct callers and tests.

## Findings (written before any edit)

Ranked by consequence. Points: 1 maintainability, 2 non-vacuity, 3 visibility,
4 abstractions. *Fix* marks the ones fixed here.

| id | pt | site | what is wrong | evidence | disposition |
|---|---|---|---|---|---|
| F1 | 4 | `reweight/engine.rs:441-447`, `:527-533`; `:455-466`, `:538-555` | Second copy of the split read-out: the node quadratic form `uᵀRu` is the same eight lines in `Reweighter::ratios` and `BoundSub::ratios_in`, and the exact-slot memo loop is copied with only the per-slot evaluation changed. | `git show dfdf85d4 -- vibegraph-lib/src/reweight/engine.rs`: the `for (j, &uj) in u.iter().enumerate() { let row = &r[j * k..(j + 1) * k]; …` block appears twice, identical. | **fix**: one `quadratic_form(r, u)` and one slot-memo helper taking the evaluation as a closure. |
| F2 | 3/1 | `helas/eval/lower.rs:36-88` | The production lowering's contract (the JAMP formula, the `CoeffRat` rule, the configs bundle) is documented on `lower_flows`, which #168 made `#[cfg(test)]`; `lower.rs:38` says "the production path is [`lower_flows`]", false since `compile` calls `lower_split_flows`. Seven doc sites name `lower_flows` as what `compile` runs: `lower.rs:89,98,190`, `compile.rs:1460`, `eval/mod.rs:10`, `egraph.rs:332`, `run/probe_bridge.rs:6`. | `grep -rn lower_flows vibegraph-lib/src` gives one call, `compile.rs:1477`, inside `#[cfg(test)]`. | **fix**: delete the test-only wrapper, give its name and its doc to the one production function (`lower_flows(…, class_of)`), the one test caller passes `None`. |
| F3 | 1 | `helas/eval/lower.rs:151` | "reduces to today's color-free lowering": temporal framing (AGENTS.md comment rules). | `git show 4b132cc5 … \| grep today` | **fix** |
| F4 | 1 | `generate/audit.rs:166-168` | `finish`'s doc lists the subprocess and non-finite refusals only; #182 added three more it makes (refused helicity, identical particles under different mothers, helicity denominator mismatch). | quoted doc vs the `why` chain at `audit.rs:205-226`. | **fix** |
| F5 | 4 | `generate/audit.rs:89`, `:125` | The relative-deviation expression `(a - b).abs() / b.abs().max(f64::MIN_POSITIVE)` written twice in one function. | quoted lines. | **fix**: one helper. |
| F6 | 4 | `hadronic.rs:1684-1696` | `drawn_helicity` finds the drawn combination's index again by a linear `position` over `helicities()` after `select_helicity`, its only caller, already had it from `select_index`; the second lookup's `?` cannot fail. | `grep -rn "select_helicity(" vibegraph-lib/src`: one call, `hadronic.rs:1691`. | **fix**: `select_helicity` returns the index. |
| F7 | 3 | `diagrams/squared.rs:118-178`, `:396` | `SquaredOrders::{diagram_orders, accepts, classes, selected_pairs, is_signed, is_signed_on}` and `annotate_sets` are `pub` with in-crate users only (`compile.rs`, `generate.rs`, `artifact.rs`, `diagrams/artifact.rs`); two `impl SquaredOrders` blocks for no reason. | `grep -rn` of each name over `vibegraph-lib vibegraph-cli`: no hit in `tests/` or the CLI. | **fix**: `pub(crate)` (private where only the module reads it), one impl block. |
| F8 | 1 | `diagrams/squared.rs:349-358` | `filter` builds a throwaway `SquaredRequest { constraints: constraints.clone() }` only to read `constraints[k]` back out of it. | quoted lines. | **fix** |
| F9 | 3 | `reweight/card.rs:110` | `ReweightCard::helicity` is a `pub(crate)` field beside the derived `helicity()` (default applied) and `stated_helicity()`; nothing outside `card.rs` reads the field, and a crate reader of it would skip MadGraph's default. | `grep -rn "\.helicity\b" vibegraph-lib/src` outside `card.rs`: only `options.helicity` / `plan.helicity`, other types. | **fix**: private. |
| F10 | 2 | `vibegraph-cli/tests/cli_reweight_helicity.rs:153-159` | The contradicted-flag refusal asserts `"--reweight-helicity"` and `"change helicity"`, both substrings of `SUMMED_REMEDY`, which every per-event refusal prints: it does not assert which refusal fired. Mutation that survives: `helicity_word(flag)` → `helicity_word(line)` in `vibegraph-cli/src/generate.rs` (the refusal names the wrong flag). | mutation table below. | **fix**: assert the refusal's own head, `--reweight-helicity <flag>: … says \`change helicity <True\|False>\``. |
| F11 | 3 | `helas/eval/compile.rs:342-347` | `select_helicity`'s signed branch draws `∝ |T_c|`; no production path reaches it: `generate` refuses a signed selection (`generate.rs:427`) before any draw, and `select_helicity`'s only caller is the generation integrands. | `grep` above; mutation M11 below. | **leave**: a guard that keeps the draw defined (MadGraph's `DABS`) if the refusal is lifted; reported, no test can reach it through `generate`. |
| F12 | 2 | `reweight/engine.rs:748` | `ReweightError::HelicitySigned` is reachable only through `ReweightPlan::new` called directly (unit tests, `reweight_mg_oracle`): `generate` refuses a signed set first. | as F11. | **leave**: the plan is a public API with its own contract; the unit test reaches it. |
| F13 | 1 | `reweight/engine.rs:756-758` | `.expect("probed at the card's point, so at every point")` in `check_event_helicity`. | `contributing_helicities` returns `None` only on the evaluator's shape (`n_ext`, `n_in`), never on the point. | **leave**: an invariant, not a fallible path. |

Test non-vacuity (point 2) is in the mutation table: one mutation per test the
two PRs added, plus the split's runtime sites (M9-M11).
