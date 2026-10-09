---
type: Design Decision
title: "Validation sprints expose and record; follow-ups fix"
description: "Work that adds gates records a newly failing cell as informational with a backlog item and moves on; it fixes only regressions it caused, and never loosens a tolerance."
status: draft
tags: [validation, process, gates, scope]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n25-sessions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L501-L560", title: "Note 25 §8: sprint discipline — expose, don't fix"}
  - {id: n25-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L561-L579", title: "Note 25 §9: decisions (user, 2026-07-31)"}
  - {id: n27-charter, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L14-L24", title: "Note 27: the fixing sprint's charter and its scope control"}
  - {id: n36-rules, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L26-L32", title: "Note 36: the standing rules"}
---
**Decision (user, 2026-07-31).** Work whose job is to *expose* adds gates,
references and comparisons. When a gate it newly exposes fails, the session
does three things:[^n25-sessions][^n25-decisions]

1. Record the measurement as an **informational** (⚠️) cell.
2. File a debug or fix item in the backlog.
3. Move on.

It does not diagnose, tune, or touch physics or sampler code to make a new
cell green. A later session, whose job is to fix, takes those items.

A session fixes in-session only **regressions of pre-existing gates caused by
its own changes**. Those are its exit gate. And it **never loosens a
tolerance**. That rule is project-wide and predates this decision.

## Why

- **Scope control.** Exposure and diagnosis are separate jobs with separate
  briefs. The sprint that set this rule had the job of exposing and
  consolidating, and the backlog it generated went to a follow-up sprint
  chartered to diagnose and fix.[^n25-sessions][^n27-charter]
- **"Green" stays meaningful.** A cell goes green because the disagreement is
  **resolved**. Otherwise it stays ⚠️ with a note saying exactly what is
  unresolved and why.[^n27-charter] The informational cell is a recorded
  measurement, so a follow-up starts from a number rather than from a
  description.

The session's own exit gate is narrow and mechanical. The cells its brief
names are measured and reported, and every previously enforced cell is
unmoved. A session fails only if its machinery cannot produce the
measurements, not because what it measured disagrees.

## The rule in a fixing session

A session whose job is to fix still applies the rule in one direction. A
*new* finding exposed while fixing an old one is recorded as ⚠️ plus a backlog
item, not chased inside the session.[^n27-charter] A cell flips from
informational to gated only on a recorded measurement inside the reference's
own error.[^n36-rules] Related rules: a known-wrong informational comparison runs
while a fix is under construction, and new physics lands informational first
([session scoping](session-scoping-rules.md), `AGENTS.md` "Physics
Validation").

## Where it applies now

Under [one PR per backlog item](../decisions/pr-per-backlog-item.md), the rule
applies to a validation session on any item's PR. If its new gate exposes a
failure the item was not about, the failure becomes an informational cell and
a **Found** entry in the session report, which the manager files
([backlog items](backlog-items.md)). It does not widen the PR.

The layers and the informational/gate distinction are in
[validation layers](../validation/validation-layers.md). How each check is
judged for what it can and cannot see is in
[oracle blind spots and non-vacuity](../validation/oracle-blind-spots-and-non-vacuity.md).

[^n25-sessions]: Note 25 §8, "Sprint discipline — expose, don't fix", carried verbatim in every session prompt of that sprint.
[^n25-decisions]: Note 25 §9, the general directive and decisions 4 and 6.
[^n27-charter]: Note 27, charter of the fixing sprint that followed.
[^n36-rules]: Note 36, the standing rules.
