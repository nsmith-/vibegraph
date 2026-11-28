---
type: Backlog Item
title: "Activate the release/v* ruleset now that release/v0.2 is cut and dry-run"
description: "release/v0.2 is cut at the v0.2.3 tag and its non-publishing dispatch passed. The remaining step is the user's: activate the release/v* ruleset from #210's body."
area: hygiene
state: needs-user
priority: high
closes_when: "release/v0.2 exists at the v0.2.3 tag; `gh workflow run release.yml --ref release/v0.2 -f tag=v0.2.3` completes release-check, the self-test and all four build legs with publish skipped; the release/v* ruleset from #210's body is active (PR required, squash only, all-pass required, linear history, no deletion or force-push)."
blocked_by: []
opened: 2026-11-25
tags: [release, ci, ruleset, release-branches, needs-user]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: concept, resource: "../../tooling/release-branches.md", title: "Release branches and the backport flow"}
---
#210 closed `release-branch-ci-and-ruleset-missing` as far as it could go:
- CI runs on `release/v*`;
- the backport flow is documented in [release branches](../../tooling/release-branches.md);
- `publish` requires a genuine tag push.

A dispatch from a branch ref builds every leg and skips `publish`. Running one
from `release/v0.2` is the evidence that a tag pushed from a non-main branch
builds.

The ruleset settings mirror main's ruleset (id 22539060) and are given in #210's
body as UI steps and as a `gh api` payload. Activate the ruleset after the
branch is cut.

Done on 2026-11-28: `release/v0.2` is at e4102398 (the v0.2.3 tag's commit),
and the dispatch from it
([run 46099791575](https://github.com/nsmith-/vibegraph/actions/runs/46099791575))
passed release-check and all four build legs with publish skipped. Only the
ruleset remains.
