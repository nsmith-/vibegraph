---
type: Procedure
title: "Hygiene review protocol"
description: "How an R-session reviews one module cluster on four points, read-only, and the report shape triage and the lessons depend on."
status: draft
tags: [hygiene, review, protocol]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Every R-session brief links here. The brief names the cluster; this page
says how to review it.

## Ground rules

- **Read-only.** No edits, commits or builds in the worktree. `cargo` is not
  run: eight reviews run in parallel on one host
  ([agent dispatch](../../../workflow/agent-dispatch-and-worktrees.md), "One
  heavy suite per host"). `grep`, `git log -L`, `git blame`, LSP queries and
  reading code are the tools.
- **Every finding is checked against the code before it is reported**, with
  its `path:line`. "Probably unused" is not a finding. "`grep -rn Foo` over
  the workspace returns only its definition" is.
- **Check the backlog before reporting.** Start the session by listing every
  filed item, in any area, that names a path in the cluster:
  `grep -rlF -e <name> research/kb/backlog/` for each file and directory in
  the brief's cluster, then `pixi run backlog --item <slug>` on each hit. Items
  cite paths with different prefixes (`vibegraph-lib/src/helas/eval/op.rs`,
  `src/helas/eval/op.rs`, `helas/eval/op.rs`), so search for the shortest
  unambiguous tail, such as `eval/op.rs`, and for the bare file name. Filed
  items, claimed or not, are not findings. Report a filed item only when one of
  its sites is wrong or missing, or when the code shows it is already fixed;
  either of those *is* a finding. List the items you found this way at the
  top of the report, so triage can tell a rediscovery from a new finding.
- **AGENTS.md conventions are the standard,** especially the comment
  guidelines and "Physics Validation". A finding that cites a rule names it.

## The four points

1. **Maintainability.** Logic duplicated within the cluster; functions long
   enough that a reader loses the thread (give the length and what to split
   out); unclear module boundaries; comments that contradict the code, or
   narrate its history or a plan.
2. **Test non-vacuity.** Tests that compare nothing, or compare a value with
   itself; tolerances far wider than the algorithm's error; gates that
   soft-skip (a runtime return where the banked layer requires
   `validation::require`); asserts that cannot fail on the defect the test's
   name or doc claims to guard. For each, name the **mutation that would
   prove it vacuous**: the one-line code change the test should catch and,
   you claim, does not. The fix session runs it.
3. **Visibility.** V1 and V1b have demoted everything with no outside user. Report
   what remains `pub` that should not be (a module re-exporting an internal
   type, a `pub` field that breaks an invariant), and fields or methods whose
   visibility exceeds the type's.
4. **Reusable abstractions.** Patterns repeated across the cluster, or that
   you suspect recur outside it, that want one home. Name the instances.

## Report

Return the report as the final message. The manager records it as
`sessions/R-<x>-report.md` (`type: Session Report`). Sections:

- **Findings.** At most 30, ranked strongest first. Each finding gives:
  - an id (`R-<x>.<n>`), the point (1–4) and `path:line`;
  - the evidence: the command and its output, or the quoted lines;
  - a proposed disposition: *fix here*, *file* or *leave*, as defined in
    [D3](../decisions/D3-fix-small-file-large.md);
  - a confidence: *checked* (the evidence settles it) or *suspected* (says
    what would settle it).
- **Also seen.** One line each for anything below the cut.
- **Cross-cluster patterns.** Suspected recurrences outside the cluster, for
  triage to merge.
- **Method.** What you did on each point, in order, and roughly what share of
  the session it took. Which technique found the checked findings, and which
  produced leads that did not hold up. This is the lessons' raw material, so
  be candid about dead ends.
- **Found.** New work outside the four points, such as a latent bug, one
  entry each.
- **Brief corrections.** Where this protocol or the brief was wrong.

## Worktree & long-command discipline


- **First action**: `cd` to the absolute worktree path in your assignment, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current`. If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between completion
  notifications instead of idle-waiting. On resume after a stall, first check
  whether the interrupted command finished (`ps`, result artifacts) before
  re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
