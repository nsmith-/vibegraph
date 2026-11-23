---
type: Session Report
title: "L-Z2 report: hygiene of Majorana fermion flow, the chirality exchange and the Goldstone filter"
description: "Findings, mutations, fixes and identity oracles of the hygiene pass over the Majorana/fermion-number-violation change (#169) and the Goldstone-spelling and list-propagator parser fix (#178)."
status: draft
tags: [hygiene, majorana, fermion-flow, goldstone, mutation-testing, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-fermion` off main 8b3af5ab (brief bbffe26c). Scope:
`git show a5bf8fca` (#169) and `git show c18042ed` (#178), their callers and
tests. This section was committed before any code edit.

## Findings

Ranked; *fix* marks what this pass fixes. Line numbers are at bbffe26c.
Mutation ids (M*n*) refer to the mutation table below; a finding marked
*suspected* is settled by its mutation.

| id | pt | site | class | what is wrong (evidence) | disposition |
|---|---|---|---|---|---|
| F1 | 2 | `diagrams/diagram.rs:690-716` | refusal no test reaches | The four-fermion refusals `ConvertError::MajoranaFermion { vertex }` and `FermionNumberViolation` have no test: #169 deleted the CLI refusal tests, and `grep -rn "MajoranaFermion\|FermionNumberViolation\|four-fermion vertex" --include=*.rs` finds only the definitions and the two `return Err` sites. Mutation M6 (`fermions <= 2` → `<= 99`, never refuse) is suspected to survive. | fix: a test per variant in `tests/majorana_flow.rs`, on the toy models with a four-fermion vertex appended |
| F2 | 2 | `diagrams/chain.rs:509-518` | refusal no test reaches | The decay-chain Majorana refusal has no test: `grep -rn "decay-chain diagram carries" --include=*.rs` returns only its own line. M7 (`if false && …`) suspected to survive. | fix: a test in `tests/majorana_flow.rs` |
| F3 | 2 | `ufo/lorentz.rs:743-783` | test blind to a branch | `charge_conjugation_parity_of_two_fermion_structures` has no `Sigma` structure (M4: `σ` conjugation `(-l,-r)` → `(l,r)`), and every chirality it places left of a gamma is in a combination symmetric under `c_L ↔ c_R`, so the left-of-gamma swap is unobserved (M5: drop the swap). Both suspected to survive. | fix: add `σ^{μν}` (odd) and a mixed left/right-of-gamma chirality case |
| F4 | 2 | `ufo/particles.rs:621-625` | test blind to a branch | The Goldstone-spelling test sets only `= True`; M2 (`== Some(true)` → `.is_some()`, a `= False` flag marks a Goldstone) suspected to survive. | fix: assert a `= False` flag in every spelling leaves the particle physical |
| F5 | 2 | `ufo/particles.rs:628-643` | test blind to a branch | The dict propagator case `{0: Prop.U, 1: Prop.F}` puts key 0 first, so taking the first value regardless of key (M3) is suspected to survive. | fix: add `{1: Prop.F, 0: Prop.U}` |
| F6 | 1 | `tests/toy_models.rs:850-853` | comment breaks AGENTS.md | `/// The parser at a0a6eb4 (vibegraph-lib/src/ufo/particles.rs lines 233-235) reads only goldstoneboson and goldstone, so the GoldstoneBoson case fails there` describes a past commit's code. | fix: drop the history paragraph |
| F7 | 1 | `diagrams/diagram.rs:626-629` | doc does not match body | `fermion_flow_summary` says each line prints "its propagator count"; the `format!("{}{}~{}{} [{}]", …)` at 641-648 prints ends and vertices only. | fix: doc to the body |
| F8 | 3 | `helas/eval/diagram_eval.rs:203-233` | production contract on a test-only item | `VertexTerm::from_ufo` is `#[cfg(test)]` and carries the production doc ("Generate a VertexTerm from a UFO vertex definition …"); the production `from_ufo_read` is documented as "[`from_ufo`](Self::from_ufo), with …", a link to an item absent from a non-test build. | fix: the contract on `from_ufo_read`, the test wrapper's doc pointing to it |
| F9 | 1/4 | `diagrams/diagram.rs:666-669, 699-702` | duplicate within the module | The Majorana predicate `p.spin.abs() == 2 && p.is_self_conjugate` is written twice in `diagram.rs` (`violates_fermion_flow`, `check_fermion_flow`); the same expression is in `ufo/bind.rs:252` and `ufo/report.rs:281`. | fix: `Particle::is_majorana` used at the two `diagram.rs` sites; bind/report to Found |
| F10 | 2 | `tests/standalone_jamps.rs:1667` | `catch_unwind` discards its payload | `probe_graph_ratios` prints "the probe panicked" without the panic message. | fix: print the payload |
| F11 | 1 | `ufo/lorentz.rs:122-246` | function past 100 lines | `charge_conjugation_parity` is 125 lines; the chain walk (bra → ket over the spinor operators' `i`/`j`) is a separable unit and matches the six spinor variants twice. | fix: extract the chain walk; identity oracle is the unit test plus the enumerate/σ oracles |
| F12 | 2 | `helas/eval/diagram_eval.rs:421-422` | hermetic blind spot | `conjugate_standalone_projectors` (the chirality exchange) is pinned only by the MSSM rows, which are `extended-validation`: the toy Majorana and diquark models carry `Identity(2,1)` scalar vertices, whose exchange is the identity. M9 (`swap = false`) is suspected to survive the hermetic suite. A hermetic chiral row needs a new MadGraph table. | Found (validation-dev) |
| F13 | 2 | `diagrams/diagram.rs:620-621` | no mutation reaches | `orient_fermion_lines` picks which end to flip; by Denner–Eck–Hahn–Küblbeck the amplitude does not depend on the choice, so M13 (`flip = b` always) is expected to be an equivalent mutant: no amplitude test can see it. | leave, reported as unreachable if M13 survives |
| F14 | 1 | `tests/standalone_jamps.rs:1675-1781` | function past 100 lines | `probe_one` is ~110 lines. It is an `#[ignore]`d diagnosis, read top to bottom; splitting it buys nothing. | leave |

Backlog check: `grep -rln "particles.rs\|/lorentz.rs\|/diagram.rs\|diagram_eval.rs\|majorana_flow\|toy_models.rs\|chain.rs" research/kb/backlog` lists 13 items. None is a finding of this pass: `hygiene/diagram-count-test-helper-duplicated`
names `toy_models.rs::diagram_count` (unchanged here), `hygiene/comment-leftovers-after-hygiene-sprint`
a history comment in `ufo/lorentz.rs` outside `charge_conjugation_parity`, and the others name
other functions of the same files.
