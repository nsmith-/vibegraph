---
type: Backlog Item
title: GitHub Pages source switch to GitHub Actions awaits confirmation
description: docs.yml's deploy job needs the Pages source set to GitHub Actions; run history suggests it already is, and the user should confirm.
area: hygiene
state: needs-user
priority: low
closes_when: The user confirms that Settings → Pages uses the "GitHub Actions" source (or switches it), and docs.yml's deploy job on main is green.
blocked_by: []
opened: 2026-09-07
tags: [docs, ci, pages, user-call]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo-user, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L56-L57", title: "TODO.md \"Open, and the user's call\": switch GitHub Pages to the Actions source"}
---
`.github/workflows/docs.yml` publishes the mdBook site with
`actions/deploy-pages`. That step needs the repository's Pages source set to
"GitHub Actions" one time (Settings → Pages); until then the deploy job fails.

Evidence that this is already done: all 11 `docs.yml` push runs on `main` from
2026-09-07 (run 34079692712) to 2026-10-06 (run 37409968712) concluded
`success`, and the `deploy` job was among them. The Pages settings endpoint
could not be read from the agent session, so this is inferred from the run
history and not read from the setting itself. If the user confirms, delete this item.
