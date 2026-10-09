---
type: Procedure
title: Scoping a sprint session
description: "One deliverable per session; oracle before engine; inert plumbing first; new cells informational first; stop-rules for others' bugs; hand-off notes that carry watch items."
status: draft
tags: [sessions, scoping, process, agents]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n16-debrief, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L497-L554", title: "Note 16 §6: colour-flow sprint debrief"}
  - {id: n28-rules, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L65-L86", title: "Note 28 §2: session-scoping ground rules"}
  - {id: n28-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L239-L253", title: "Note 28 §5 Z: a close-out session that does nothing else"}
  - {id: n35-rules, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L215-L239", title: "Note 35 §2: session-scoping ground rules"}
---
How to cut a body of work into dev-agent sessions. These rules were sized
against what has gone wrong in earlier sprints. They apply both to a sprint's
sessions and to the session types run in turn on a single item's PR
([one PR per backlog item](../decisions/pr-per-backlog-item.md)). Dispatch
mechanics are in [agent dispatch](agent-dispatch-and-worktrees.md). The physics
lessons that come from the same debriefs (blind spots, convention claims, the
known-wrong row, checking a fix arithmetically) are in
[oracle blind spots and non-vacuity](../validation/oracle-blind-spots-and-non-vacuity.md)
and `AGENTS.md` "Physics Validation".

## The rules

1. **One deliverable per session.** A session that banks references does
   nothing else. A close-out session does close-out only.
   - *Why:* one session died of context bloat after combining a 28-run
     re-bank with hygiene and close-out.
   - The close-out that followed was scoped to the reference re-cut, the
     manifest flips, the re-rendered report and the bookkeeping, and "nothing
     else rides on it".[^n28-rules][^n28-z]
2. **Oracle before engine, banked by another session.** The session that
   produces the reference dump or specification lands, and is reviewed, before
   the session that must match it starts. The session that builds an engine
   neither builds the oracle it is judged by nor banks the reference.
   - *Why:* several sessions grew past their design when an engine and its
     oracle were built in the same breath.[^n28-rules][^n35-rules]
   - For a high-stakes item the same separation can extend to design and
     review, in [design, implement, review](design-implement-review-chains.md).
3. **Land plumbing provably inert first.** Split a change into a structural
   step that leaves every gate byte-identical, then the behavioural step.
   - In the colour-flow sprint, the AST plumbing landed with the net
     byte-identical before and after. When the evaluator change then ran into
     trouble, the suspect diff was minimal.
   - Splitting pure-Rust work from MadGraph-environment work paid for itself
     the same way.[^n16-debrief]
4. **Land new cells informational first.** A new comparison is added as an
   informational cell, expected red, before the engine session that should flip
   it.
   - A session's gate is "the cells its brief names go green **and** every
     previously enforced cell is unmoved" (`pixi run --skip-deps validate`).
   - A new kernel that no row exercises shows as a red op census, not a
     passing suite.[^n35-rules]
   - In a validation session, a newly exposed failure stays informational and
     goes to the report's Found section for the manager to file
     ([expose, don't fix](expose-dont-fix.md)).
5. **A stop-rule for bugs that are not the session's.** Fix only if the fix is
   unambiguous; otherwise pin the finding, localise it, and report.
   - The first process to exercise a code path is as likely to find a
     pre-existing defect as one of the session's own. `gg_to_gg`'s failure in
     the colour-flow sprint decomposed into one of each.
   - The explicit stop-rule kept that sprint out of Lorentz-layer surgery, and
     the precise localisation made the deferred fix cheap to pick up.[^n16-debrief]
   - A design deviation also stops the session, and the manager amends the
     design ([sprint lifecycle](sprint-lifecycle.md)).
6. **Hand-off notes are load-bearing.** Each report carries a "for downstream
   sessions" section, and watch items travel in it.
   - In the colour-flow sprint, the T-transpose observation passed through four
     sessions as a watch item before it unravelled the real bug.
   - The section costs little and pays repeatedly.[^n16-debrief] Under the
     bundle, new work goes in the report's **Found** section, which the manager
     files ([backlog items](backlog-items.md)).
7. **Resume with reconciliation.** An interrupted session resumes from its
   transcript with a mandatory `git status`/`git log` check of what actually
   landed. That worked cleanly every time it was needed.[^n16-debrief]
   Re-verify the working directory on resume too: resume is when worktree
   isolation leaks.
8. **Name the cheap-model relief valves per session.** A brief says which
   deterministic bulk work (sweeps, fixture regeneration, mechanical extraction)
   the session may hand to narrow sub-agents. It never hands over judgement or a
   gate's meaning.[^n28-rules]
9. **Hermetic where possible.**
   - Pure representation work is unit-tested in the hermetic layer.
   - Work that needs the MadGraph submodule registers its tests through
     `required-features` in the banked layer, never as a runtime skip.
   - Reference tables, once banked, make their cells hermetic.[^n35-rules]

The validation layers these rules refer to are in
[validation layers](../validation/validation-layers.md).

## A close-out checklist item worth keeping

After any environment change, check `pixi.lock` against `pixi.toml` with a
locked install before closing. A close-out once tripped CI on exactly that.[^n28-z]

[^n16-debrief]: Note 16 §6, sprint debrief.
[^n28-rules]: Note 28 §2.
[^n28-z]: Note 28 §5, Z close-out.
[^n35-rules]: Note 35 §2.
