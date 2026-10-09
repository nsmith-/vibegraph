---
type: Session Brief
title: "Z: hygiene sprint close-out"
description: "Close-out only: verify the exit criteria, delete closed items, file triaged and Found work, write closeout.md, and mark the PR ready."
status: draft
agent: manager
depends_on: [L]
closes: [hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Close-out only, per the [sprint lifecycle](../../../workflow/sprint-lifecycle.md)
step 6 and [session scoping](../../../workflow/session-scoping-rules.md) rule 1.

1. **Gates.** On the final commit, run and record each
   [exit-criterion](../sprint.md) gate, `pixi run --skip-deps validate`
   included, with every previously enforced cell unmoved. Check `pixi.lock`
   with a locked install if the environment changed.
2. **Backlog.**
   - Delete each closed item's file.
   - Drop the `Backlog:` line of each item left open, with the reason in
     `closeout.md`.
   - File every triaged *file* finding and every Found entry as a new item.
   - Re-derive the remaining `pub` surface from the final tree, grouped as
     V1 did (used by vibegraph-cli, test/bench-only, pub only through a pub
     signature). Use a per-item demote-and-compile run or rustdoc JSON
     rather than a name grep. Link it from lib-pub-api-surface-unaudited as
     the proposal.
3. **Promote.** D1–D4 are already `stable` (signed off 2026-10-09). Move the
   review protocol to `stable` once the user has reviewed it, folding in the
   reports' brief corrections. `workflow/hygiene-review.md` stays
   `draft` until the hygiene agent's own PR.
4. **Record.** Write `closeout.md` (`type: Sprint Record`): what was banked,
   the items closed, filed and released, and the `pub` counts before and after.
5. **Bookkeeping.**
   - Set `active: false` on `sprint.md`.
   - Add an entry to the root `log.md`.
   - Run `pixi run kb-index` and `pixi run kb-lint`.
   - Mark the PR ready for review.
