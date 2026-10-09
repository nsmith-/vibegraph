---
type: Procedure
title: "Design, implement, review: three sessions per work item"
description: "A work item run as a design session (a pre-registered written section), an implementation session and a fresh-context review session; what it cost and caught, and when it is worth using."
status: draft
tags: [process, review, sessions, pre-registration]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n29-protocol, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L51-L87", title: "Note 29 §2: session protocol, design → implement → review"}
  - {id: n29-verdicts, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L6088-L6125", title: "Note 29 close-out: per-chain verdicts"}
  - {id: n29-observations, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L6137-L6158", title: "Note 29 close-out: protocol observations"}
---
A work item can run as a **chain of three separate agent sessions**: design,
implementation, then a review with fresh context. The manager then supervises
and reprioritises. It does not carry design context through implementation, and
it is not the only reviewer of work it also specified.[^n29-protocol]

**Where it is used.** The chain ran as the default for one validation sprint,
note 29's, covering seven chains and nine sessions. Later sprints did not adopt
it as the default:

- They wrote the design into the sprint plan itself, before any session ran.
- One session implemented each item.
- Pre-registration survived as its own tool
  ([pre-registered verdicts](../validation/pre-registered-verdicts.md)).

Under [one PR per backlog item](../decisions/pr-per-backlog-item.md), each
change is reviewed from several perspectives in turn (feature, validation,
performance, later hygiene). That is a different axis from this chain's
author/reviewer separation. The chain is a pattern a manager can choose for a
high-stakes item, typically one whose design premise is uncertain or whose
verdict is hard to adjudicate. It is not a standing rule.

## The three sessions

1. **Design** (agent type `claude`, strongest model, read-only intent). It
   writes exactly one thing, its design section, which contains:
   - the concrete change list, file by file;
   - the acceptance tests, by name;
   - the gates to run and the cells expected to move;
   - the risks, and an explicit claim of "what this provably cannot break".
   The manager reads the design before dispatching implementation. This is the
   reprioritisation point.
2. **Implementation** (a dev agent type). It executes exactly the design, with
   no scope invention, runs the gates the design names, and commits with command
   and output evidence in its report.
3. **Review** (agent type `claude`, strongest model, fresh context, pointed at
   the chain's worktree). It **verifies rather than trusts**:
   - it reads the full diff;
   - it re-runs the cheap gates itself;
   - it checks each acceptance criterion against recorded output;
   - it hunts specifically for the error class the implementation's own tests
     cannot see.
   The verdict is **merge**, **fix** (with named defects) or **escalate**. A fix
   verdict loops the implementer, continued in the same worktree where possible.
   A second fix loop escalates to the manager.

The manager merges chains into the integration branch in the planned order and
runs the full banked gate after each merge, not only at the end. Worktree and
dispatch mechanics are as in [agent dispatch](agent-dispatch-and-worktrees.md).

## What it caught, and what it cost

**The design step paid for itself on every chain, but not by being right.**
Three designs had a load-bearing premise falsified by the first measurement:

- one chain twice (the crossing class `u c~ > u c~`, then the
  reversal-degenerate self-pairing of `g g > g g`);
- one chain's veto premise (the veto existed, but panicked);
- one chain's 18-row reach list (nine fixed-beam rows compile no scale
  prescription).

Because each falsification hit a written, pre-registered claim, it produced an
amendment instead of an improvisation. A pre-registered decision table and a
may-move set turned the two highest-stakes adjudications into mechanical
checks. In one chain the live change's escalation diff landed byte-exactly on
the pre-registered five-cell set.[^n29-observations][^n29-verdicts]

**Fresh-context review found real defects on five of seven chains**, none of
which the implementer had caught:

- errors in a note's prose;
- a second silent skip;
- two misclassified run-card fields, found when a human read a reason string
  against MadGraph's `cluster.f`. That was the audit's own documented blind
  spot materialising, and the lesson adopted was per-field evidence strings
  wherever a block argument is not uniform;
- four gaps in a measurement record;
- a published stale manifest note.

All were cheap to fix in one loop, and no chain needed a second review loop.
Reviewers also re-derived what designs had only asserted: one independently
reproduced a 20k-event categorical result, and another derived the step a
design had taken on faith. That derivation stands in the record.[^n29-verdicts]

**Costs:**

- Sessions on cheaper models stalled "waiting for notification" three times
  and needed nudges. Strong-model sessions hit the ~600 s stream watchdog
  twice.
- The manager's brief after a context compaction lost reviewer wording it had
  deferred across a pause. The fix is to quote deferred text verbatim when
  putting it on hold.
- An implementer lost uncommitted work to its own `git reset`, and a
  design-session worktree was found reset. Briefs require committing early.

Worktree pre-provisioning had zero failures across the nine sessions.[^n29-observations]

## Related

[Session scoping](session-scoping-rules.md) carries the rules that outlived the
chain: one deliverable per session, and oracle before engine. The sprint's
per-chain record is its Sprint Record (`sprints/note-29-validation/closeout.md`).
The [sprint lifecycle](sprint-lifecycle.md) is the procedure work runs in.

[^n29-protocol]: Note 29 §2.
[^n29-verdicts]: Note 29 close-out, "Per-chain verdicts".
[^n29-observations]: Note 29 close-out, "Protocol observations".
