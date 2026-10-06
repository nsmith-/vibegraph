---
type: Backlog Item
title: Whether to rewrite AI co-author trailers in history is undecided
description: TODO.md asks whether to run scripts/rewrite-ai-trailers.py over history; a rewrite already ran once and the repository is now public.
area: hygiene
state: needs-user
priority: low
closes_when: The user decides whether history needs another trailer rewrite; the item closes once that decision is recorded and acted on.
blocked_by: []
opened: 2026-08-06
tags: [git-history, attribution, user-call]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo-user, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L58-L59", title: "TODO.md \"Open, and the user's call\": run scripts/rewrite-ai-trailers.py over history"}
---
`scripts/rewrite-ai-trailers.py` turns AI `Co-Authored-By:` trailers into
`Assisted-by: <harness>:<model>` (the convention in `AGENTS.md`). It rewrites
every commit hash and drops GPG signatures, so it is only safe before anyone else
has a clone.

Two facts change the question as `TODO.md` puts it:
- **A rewrite has already run.** Commit `27e9974` (2026-08-06) says "the
  attribution rewrite (Co-Authored-By -> Assisted-by) replayed all 687 commits"
  and remaps every recorded hash onto the new history. Reachable history has 245
  `Assisted-by` lines and one remaining `Co-authored-by: Claude` trailer
  (`f14ac4e`, 2026-08-06, on `main`). This was checked on a shallow clone, so
  older history was not inspected.
- **The repository has been public since 2026-09-25.** Another rewrite would now
  break every public clone and fork, which the script's own header says to avoid.

What the user needs to decide: leave history as it is (one stray trailer), or
force-push a second rewrite despite the public clones.
