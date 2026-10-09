---
type: Backlog Item
title: acceptance.yml has never passed against a binary release
description: The release acceptance workflow has never run green, and no run has targeted v0.1.0 since the repository went public on 2026-09-25.
area: hygiene
state: needs-user
priority: medium
closes_when: An acceptance.yml run against a published binary release (v0.1.0 or later) is green, and its log has been read to confirm the PDF download, consent prompt, cache and card-to-.lhe steps all ran.
blocked_by: []
opened: 2026-08-01
tags: [acceptance, ci, release, user-call]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L509-L519", title: "TODO.md entry T038"}
  - {id: todo-user, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L51-L55", title: "TODO.md \"Open, and the user's call\": repo public, first green acceptance run"}
---
`scripts/acceptance.sh` downloads the release binary unauthenticated by design,
so it reproduces on a clean VM with no checkout and no token; an authenticated
fallback was declined to keep it checkout-free. While the repository was
private this 404'd on `releases/download/...` (the `v0.1.0` dispatch on
2026-08-11, run 31512802217). The `v0.1.0` assets themselves are sound: the macOS
binary was checked by hand against the published `SHA256SUMS`.

The repository is public (`gh api repos/nsmith-/vibegraph` reports
`visibility: public`), but every acceptance run since then was triggered by a
`refdata-*` prerelease, which has no binary; see the item
[acceptance-yml-fails-on-refdata-releases](acceptance-yml-fails-on-refdata-releases.md).
**No run has yet tested a binary release while the repo is public**, so the gate is
unproven. The `release` event that `release.yml` raises with its own token does not
start a workflow, which is why `release.yml` dispatches acceptance explicitly.

Next step (needs `actions: write`): `gh workflow run acceptance.yml -f tag=v0.1.0`,
or cut the next `v*` release. Then read the first green run: it is the first
evidence this gate can pass at all. Background: [note 24](../../history/notes/24-user-distribution-and-proton-events-plan.md),
"Acceptance A: what it covers today, and what it cannot see".
