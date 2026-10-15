---
type: Sprint
title: "Amplitude signs and conventions"
description: "Pin, diagnose and fix the amplitude-level sign and convention errors where |M|² itself may be wrong, each against MadGraph standalone per diagram, per helicity and per flow."
status: draft
active: true
tags: [amplitudes, sign-convention, standalone, madgraph-oracle, sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: method, resource: "../../validation/bit-exact-amplitude-debugging.md", title: "Localising an amplitude disagreement with a bit-exact oracle"}
  - {id: decision, resource: "../../decisions/pr-per-backlog-item.md", title: "Decision: one PR per backlog item, four session types"}
---
Correctness first: every item here is a place where the amplitude itself, not
its integration, may be wrong or is unpinned. The work runs under
[one PR per backlog item](../../decisions/pr-per-backlog-item.md), so the sprint
is seven draft PRs, each claiming its own items. This file is the overview that
reads them as one plan. It lands with PR G; the other PRs add only their own
session briefs and reports, so they never conflict over shared files.

## Goal and exit criteria

The sprint is done when every claimed item is closed (its file deleted by its
PR) or released with the reason recorded in `closeout.md`, and:

1. **Bit-exact oracle first.** Every diagnosis is made on per-diagram ×
   per-helicity (and per-flow) complex dumps against MadGraph standalone `AMP()`
   / `JAMP()`, with matched parameter provenance
   ([method](../../validation/bit-exact-amplitude-debugging.md)).
2. **Falsifier before fix.** Each sign rule changed in production code has a
   test that fails on the code before the change, and the candidate rule was
   verified arithmetically on the dumped amplitudes before it was written.
3. **New comparisons land informational first**, and flip to a gate only on a
   recorded measurement inside the gate's tolerance.
4. **Each new test's blind spot is stated** in its session report.
5. **Gates unmoved:** `cargo fmt --all --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace` and `pixi run --skip-deps validate` on each PR's
   final commit, with every previously enforced cell unchanged. A fix that moves
   |M|² on a process with a banked σ row is reported to the planner, because it
   changes what the parallel σ sprint measures.

## Scope

Backlog items claimed, one `Backlog: <slug>` line each in the draft PR that
claims it:

| PR | Items | Why bundled |
|---|---|---|
| G | `gg-ttxg-no-amplitude-gate` | alone; feeds `pp_to_ttx_0j1j_mlm`'s `@1`, which the σ sprint investigates |
| W | `wpwm-to-epem-neutrino-exchange-sign` | alone |
| P | `per-diagram-probes-bypass-production-runtime` | alone; the other PRs' per-diagram dumps rely on the probe |
| C | `standalone-identity-crossed-line-sign-unpinned`, `derivative-scalar-contact-sign-unoracled` | the same `build_child` sign arms in `root_lorentz.rs`, each pinned by a new standalone row |
| F | `wpwmz-cw-ow-five-vector-residual`, `wpwmz-cw-config-partition-uncompared` | the same row; the partition item says it belongs with the residual |
| J | `jioxxx-relative-chiral-sign-unpinned`, `jioxxx-reference-port-comparison-has-no-teeth` | the same test, `test_eval_jioxxx` |
| Q | `same-flavour-four-fermion-diagram-count-ungated` | alone |

Out of scope: the σ gates (`validate_sigma.rs`, `validate_hadronic`), phase
space, sampling, scales, MLM and the MadEvent references, which the parallel σ
sprint owns. New MadEvent σ or event references are banked only by the user on
the M3 host. Standalone tables (`validation/madgraph/standalone/`) are banked
from this sprint.

## Sessions

No feature sessions are planned, because no item adds a capability. A
performance session runs only on a PR whose fix touches the evaluator's runtime
path; a compile-time sign does not. Each PR ends with a hygiene session.

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| [G1](sessions/G1.md) | validation-dev (Opus) | — | standalone_jamps, amplitude_oracle, validate | gg-ttxg-no-amplitude-gate |
| W1 | validation-dev (Opus) | — | diagnosis only | — |
| W2 | validation-dev (Opus) | W1 checked | standalone_jamps, amplitude_oracle, validate | wpwm-to-epem-neutrino-exchange-sign |
| P1 | validation-dev (Opus) | — | hermetic | per-diagram-probes-bypass-production-runtime |
| C1 | validation-dev (Opus) | P1 | rows banked informational; diagnosis | — |
| C2 | validation-dev (Opus) | C1 checked | standalone_jamps, validate | both C items |
| F1 | validation-dev (Opus) | P1 | diagnosis only | — |
| F2 | validation-dev (Opus) | F1 checked | amplitude_oracle, config_groups, validate | both F items |
| J1 | validation-dev (Opus) | — | hermetic | both J items |
| Q1 | validation-dev (Opus) | — | standalone_jamps, validate | same-flavour-four-fermion-diagram-count-ungated |
| *-H | hygiene-dev (Sonnet or Opus, by scope) | the PR's last session | the PR's gate | — |

Each brief lives in its PR's branch under `sessions/`, and is written once its
dependency's report has been checked. The host (4 cores, 30 GB free disk)
takes two dev agents at a time, with their heavy runs serialised by the
manager.

## Approvals

A fix verified arithmetically on the dumps, and by the controls outside its fit,
goes to production code without a separate design approval. A change of
convention gets a `Design Decision` concept and the user's approval before it
lands. Examples are recording a diagram-count difference as a convention cell,
or choosing a rule the data leave open.
