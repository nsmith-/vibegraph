---
type: Procedure
title: Sprint lifecycle
description: "A sprint as a folder of linked concepts: open with a draft-PR claim, survey, design and approval, sessions with reports, validation, and a close-out that promotes, records and deletes closed items."
status: draft
tags: [sprint, workflow, process, knowledge-bundle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n42-lifecycle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L135-L214", title: "Note 42 §4: sprint lifecycle under the bundle"}
  - {id: n42-readers, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L354-L362", title: "Note 42 §7.3: who reads what"}
  - {id: n42-trial, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L504-L512", title: "Note 42 §8: Trial"}
  - {id: n42-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L514-L535", title: "Note 42 §9: decisions 10 and 11"}
  - {id: n42-risks, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L537-L555", title: "Note 42 §10: risks"}
  - {id: n38-rhythm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1162-L1166", title: "Note 38 §5: feature work beside open evaluator PRs"}
---
A sprint is a **folder of linked concepts**, not one plan file that grows to
thousands of lines. Lasting knowledge is promoted into the topic folders at
close-out. The procedure below is how a sprint runs.

Under [one PR per backlog item](../decisions/pr-per-backlog-item.md), most work
is a single item's PR. A manager runs feature, validation and performance
sessions on it in turn, and later hygiene sessions as well. The same pieces
then apply **per PR**: a draft-PR claim, short session briefs, reports with a
Found section, and a close-out that deletes the item. The first work after the
knowledge-bundle migration is one dedicated
[hygiene sprint](../backlog/hygiene/hygiene-sprint.md). It runs in the full
sprint shape below and is that shape's test on live work.[^n42-trial]

## The folder

```
kb/sprints/<name>/
  index.md                reading order: overview → decisions → sessions
  log.md                  dated approvals, amendments, re-scopes
  sprint.md               type: Sprint — goal, exit criteria, scope, session graph
  audit.md                type: Audit — snapshot of today's surface
  decisions/D1-….md       type: Design Decision — one per decision
  sessions/G1.md          type: Session Brief — scope, agent type, dependencies, gate
  sessions/G1-report.md   type: Session Report — written by the dev agent
  closeout.md             type: Sprint Record — what was banked, census delta
```

`sprint.md` stays a readable overview, with the session list and decision
summaries inline, so the plan can still be reviewed in one file. That is the
guard against fragmentation, together with `index.md` reading orders.[^n42-risks]

## Steps

1. **Open.** The manager dispatches a sprint agent with the backlog items in
   scope.
   - Its first task is the plan of work: the folder, with `sprint.md` at
     `status: draft` and `active: true`, and the session briefs.
   - It opens a **draft PR** carrying the plan, whose description lists each
     claimed item on its own `Backlog: <slug>` line. The open draft PR *is* the
     claim ([backlog items](backlog-items.md)).
   - Before writing anything new it reads the topic's existing concepts.
     Concepts still at `draft` are open questions. Measurements recorded at a
     commit that predates heavy churn in the area are re-measurement candidates
     ([measurement provenance](measurement-provenance.md)).
   - It pushes to the draft PR only at infrequent checkpoints, typically after
     the manager has checked a session's report, so intermediate CI runs stay
     few.
2. **Survey.** Reading upstream code produces or updates a reusable
   `Codebase Survey` under `references/codebases/`, so the next sprint starts
   from it instead of re-reading MadGraph. Findings specific to this sprint go
   in `audit.md`.
3. **Design and approval.** The design edits topic concepts in place at
   `status: draft`, with one `Design Decision` concept per decision. Approval
   is recorded as data in three places:
   - `verified: [{by: human:nsmith-, at: …}]` on the concept;
   - its `status` flipped to `stable`;
   - an **Approval** entry in the sprint's `log.md`.
4. **Sessions.** A `Session Brief` is short. It holds the scope and links to
   the binding design concepts, the decisions, its gate and the items it closes
   (`closes: [...]`).
   - The dispatch names the brief (or, outside a sprint, one backlog item). The
     dev agent reads outward from its links; it never reads the whole backlog.
   - The agent writes its report as `sessions/<id>-report.md`, with
     `generated.by` set to itself and `sources` set to its commit hashes. That is
     the only bundle file a dev agent writes. New work it finds goes in the
     report's **Found** section.
   - A design deviation stops the session. The manager edits the design concept
     in place and adds a dated line to `log.md`.
   - Scoping rules are in [session scoping](session-scoping-rules.md), and
     dispatch mechanics in [agent dispatch](agent-dispatch-and-worktrees.md).
5. **Validation.** Briefs link to `Validation Gate` concepts. If gates become
   Attested Computations (optional), a gate's receipt is a recorded
   `validation-report` run, checked by the manifest collator.
6. **Close-out.** This is the only point where session results reach shared
   concepts, so parallel sessions never conflict over them.
   - **Promote.** Changed topic concepts go to `stable` after review. New
     lessons go into methodology concepts. Superseded concepts become
     `deprecated` and link to their replacement.
   - **Measurements** become `Measurement` concepts with a `measured:` block,
     taken after the sprint's last code change where possible.
   - **Record.** `closeout.md` holds what was banked and the census change.
     `landed_in` is filled for the sprint's measurements once the PR merges.
   - **Backlog.** Delete the files of items the sprint closed. Drop the
     `Backlog:` lines of items it leaves open, which releases the claim. File
     the reports' Found entries as new items.
   - **Bookkeeping.** Set `active: false` on `sprint.md`, add an entry to the
     root `log.md`, regenerate the indexes, run `kb-lint`, and mark the PR
     ready for review.

## Trust tiers on reports

"Subagent reports are evidence, not truth" (`AGENTS.md`) is encoded in the
report's frontmatter:

| Tier | Meaning | Mark |
|---|---|---|
| unverified | the agent's own claim | `generated.by` only |
| machine-confirmed | the manager spot-checked it | `verified: [{by: claude-code/…}]` |
| human-reviewed | a person checked it | `verified: [{by: human:…}]` |

What counts as a spot check, with the cases that taught it, is in
[agent dispatch](agent-dispatch-and-worktrees.md).

## Parallel PRs on a hot module

When several PRs are open at once, a session stays out of modules that open
PRs are reworking. The evaluator (`helas/eval`) is the usual one, because
performance work concentrates there. When a session must touch such a module,
it lands after those PRs or rebases onto them.[^n38-rhythm]

## Risks the procedure guards

- **Skipped promotion at close-out.** Topic concepts silently drift behind the
  sprint folders. Close-out discipline is the guard. A lint check that flags a
  `stable` sprint still linking `draft` design concepts is possible, but
  `kb-lint` does not have it yet.
- **Frontmatter overhead per sprint.** A `new-sprint` scaffold script and the
  generated indexes keep it small.

Validation sessions in a sprint follow
[expose, don't fix](expose-dont-fix.md). The older three-session chain is
described in [design, implement, review](design-implement-review-chains.md).

[^n42-lifecycle]: Note 42 §4.
[^n42-trial]: Note 42 §8 "Trial" and §9 decision 10.
[^n42-risks]: Note 42 §10.
[^n38-rhythm]: Note 38 §5, the working-rhythm decision of 2026-09-25.
