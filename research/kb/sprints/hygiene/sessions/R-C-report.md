---
type: Session Report
title: "R-C report: models, diagrams and reweighting"
description: "20 findings on ufo, diagrams, onshell and reweight: an any-refusal grammar oracle, untested refusal variants, a silently dropped Lorentz divisor, silent defaults for required UFO fields, and duplicated massless/side/WEIGHTED logic."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Paths are under
`vibegraph-lib/src/` unless stated. Mutations were reasoned from the code, not
run.

**Backlog check:** about 20 filed items cite cluster paths. Every spot-checked
site still holds. About half of the grep hits were substring false positives
(`card.rs`, `sm.rs`, `lorentz.rs`).

## Findings

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-C.1 | 2 | `tests/proc_grammar_oracle.rs:133-139` | Where MadGraph refused, the oracle counts *any* refusal at any stage. Removing `ConstrainedOrdersBeyondTree` or the decay-constraint check is masked by a later refusal. | file (map MadGraph errors to refusal stage and variant) | suspected |
| R-C.2 | 2 | `diagrams/resolve.rs:31-69` | 10 of 15 `ResolveError` variants are named by no test. | fix here (one `matches!` test per variant) | checked |
| R-C.3 | 2 | `diagrams/check.rs:483,496,637-639` | `PhotonTag` and `ModelOption` refusals are reached by no test or corpus card. Deleting the `PhotonTag` push passes everything. | fix here | checked |
| R-C.4 | 2 | `ufo/expr.rs:114-131,231-265` | 12 of 16 `Func` arms have no value test. `sec/csc/asec/acsc` act on the complex argument where UFO uses `z.real`. `cot`, `theta_function`, `cond` and `reglog` are refused at parse. No other mis-mapping. | fix here (a table test, with the asin/acos item) | checked; SM indirect coverage suspected |
| R-C.5 | 2, latent bug | `ufo/lorentz.rs:459-470` | `div_terms` silently drops a divisor that is an operator or a parenthesised group (`Metric(1,2)/(2.)` loads ×1). No shipped model does this. | fix here (refuse, with a test) | checked |
| R-C.6 | 1 | `ufo/mod.rs:709-734` | `split_vertices_by_coupling_order`'s doc attaches to the `OrderGroup` alias above it. | fix here | checked |
| R-C.7 | 1 | `ufo/topo.rs:420` | A doc cites "§1.2 of the sprint note". | fix here | checked |
| R-C.8 | 1 | `ufo/parameters.rs:138-186` | `recompute` re-implements `dependents`' BFS (O(n²)), and `set_alpha_s` walks the graph twice. | fix here | checked |
| R-C.9 | 1 | `ufo/parameters.rs:265-276` | `extract_value_*` duplicate the `ast_util` kwarg helpers; one fallback can never fire. | fix here | checked |
| R-C.10 | 1 | `ufo/vertices.rs:66-104` | `_expected_prefix` is never checked, and non-attribute items are silently dropped. | fix here | checked |
| R-C.11 | 3 | `ufo/mod.rs:159-172,545`; `diagrams/diagram.rs:166-184` | `pub` fields with derived state beside them (`param_values`, `particles`/`vertices`, `Diagram` legs/props). Outside targets only read them. | file (accessors are a CLI API change) | checked |
| R-C.12 | 3 | `diagrams/mod.rs:196-219` | `ParsingOptions {}` is an empty vestigial `pub` parameter with 16 outside call sites. | file | checked |
| R-C.13 | 4 | `diagrams/resolve.rs:207` vs `ufo/particles.rs:92`, `diagrams/mod.rs:496`, `hadronic.rs:1022` | Two definitions of "massless": restriction-zeroed vs the literal `"ZERO"`. | file (`Particle::is_massless(&ParameterSet)`) | checked; reachability suspected |
| R-C.14 | 4 | `diagram.rs:136,642`; `schannel.rs`; `onshell.rs:132` | Four implementations of "which side of a line is final-state". | file | checked |
| R-C.15 | 4 | `diagrams/mod.rs:869`; `diagrams/chain.rs:222`; `helas/eval/stitching.rs:131` | Three `WEIGHTED` computations with different missing-order behaviour (ignore, 0, panic). | file | checked |
| R-C.16 | 4 | `ufo/{parameters,couplings,particles,lorentz,vertices,propagators,mod}.rs` | The UFO constructor AST walk is repeated seven times. | fix here (one `ast_util` iterator) | checked |
| R-C.17 | 1 | `diagrams/mod.rs:562-705,764-942`; chain, engine, topo and parse functions over 110 lines | `generate_sets_inner` (179 lines) and `generate_undecayed` (144); split points named. | file | checked |
| R-C.18 | 2 | `ufo/slha.rs:76-121` | `DECAY … Auto` and non-integer keys are silently ignored, and `SlhaError::Parse` is untested. | fix here (parse-error test); file (`Auto` policy) | checked |
| R-C.19 | 2 | `ufo/parameters.rs:217-233`; `couplings.rs:96`; `particles.rs:158`; `vertices.rs:65` | Required UFO fields silently default (typo'd nature becomes internal, a missing value becomes 0, pdg becomes 0, name becomes ""). Unnamed vertices overwrite each other. | file | checked by reading |
| R-C.20 | 3 | `diagrams/mod.rs:37-38` | The `pub(crate)` re-export of `pub` check types narrows nothing. | fix here | checked |

**Also seen:**
- a comment says "allowing error" where the code panics (`ufo/mod.rs:645`);
- an SLHA error is wrapped as `Io`;
- `complex(re, im)` drops the imaginary parts;
- a tautological clone-digest assert;
- `SMRestrict::ALL` could be derived with `strum`;
- speculative TODOs;
- "backlog" headings in `check.rs` docs;
- hand-rolled temp directories where `tempfile` is available;
- `name == antiname` where `is_self_conjugate` exists;
- an ambiguity-blind case-insensitive particle lookup.

**Cross-cluster patterns:**
- any-refusal oracles;
- silent defaults for required input fields;
- masslessness as a string compare;
- `WEIGHTED` per call site;
- hand-rolled temp directories (`artifact.rs`, `pdf/mod.rs`, `cache/*`, two
  test files).

## Method

| step | share | produced |
|---|---|---|
| backlog check, separating real hits from substring hits | 10% | — |
| scripted visibility census (133 `pub` items, 34 with no outside user, traced to their exposing signature) | 15% | R-C.11, R-C.12, R-C.20 |
| refusal coverage (variant constructions vs test mentions, 21 enums) | 25% | R-C.1–3 |
| grammar tables against `function_library.py` | 15% | R-C.4, R-C.5 |
| close reading | 30% | R-C.6–10, R-C.13–19 |

**Lightly read:** `parse.rs`, `chain.rs`, `alias.rs`, `selector.rs`, `poly.rs`.

**Dead ends:**
- a PEG prefix clash;
- shared helpers in the onshell tests, which hard-coded legs anchor;
- reweight moving a locked parameter, which `check_movable` guards.

## Found

1. **`EvaluatedModel::recompute` partly revives a restriction-locked
   parameter** (`ufo/mod.rs:644-662`). It writes the new value before
   `ParameterSet::recompute` declines, so couplings reading it directly change
   while their dependents stay at 0. Reweight is guarded; the `pub` method is
   not. The fix is unambiguous (return early on a zeroed parameter), so it is a
   stop-rule candidate.
2. **`reweight/card.rs:146` refuses any `set` line containing "scan"** in a
   name; MadGraph's syntax is the `scan:` value prefix.
3. **R-C.5 is a latent wrong-coefficient bug**, not only a test gap.

## Brief corrections

- **`git branch --show-current` is empty on a detached checkout.**
- **Bare-file-name backlog grep is noisy.** Extracting `(ufo|diagrams|reweight)/…\.rs`
  citations works.
- **The `mg5amcnlo` submodule is empty in review worktrees** (also R-E).
- **The cluster size was right** (15 840 lines).

## Manager check (2026-10-09)

Worktree clean.
- **R-C.5:** reproduced. `_ => lhs` is at `ufo/lorentz.rs:468`.
- **R-C.1:** reproduced. `Some(_) => refused += 1` is in the oracle's error
  branch.
- **Found 1:** `recompute` writes `param_values` before calling
  `params.recompute`, as described.
- **Found 2:** `content.contains("scan")` is at `reweight/card.rs:146`.
