---
type: Backlog Item
title: "A 917 MB debug build committed under vibegraph-cli/target stays in git history"
description: "d5e7c71f (#171) committed 2889 build files; untracking them stops the growth, but every clone still downloads the blobs until history is rewritten."
area: hygiene
state: needs-user
priority: high
closes_when: "Once sprint L's PRs have merged and before the v0.2.3 tag, main's history is rewritten without vibegraph-cli/target (git filter-repo), force-pushed by the user, open branches are rebased or recreated, and a fresh clone's pack size is recorded before and after."
blocked_by: []
opened: 2026-11-23
tags: [git, repository-size, release, v0.2.3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Sprint L's PR 8 squash commit d5e7c71f added a debug build of
`vibegraph-cli` (2889 files, about 917 MB uncompressed; the pack is 313 MiB)
because `.gitignore` ignored only the root `/target`. A follow-up PR untracks
the directory and ignores `target/` everywhere, but the blobs stay in history.

The user decided on 2026-11-23 to rewrite history once the sprint converges.
**Do it before the v0.2.3 tag**: a tag cut first points at a commit the
rewrite replaces, and `release/v0.2` is cut from that tag.

Every open PR branch and worktree has to be recreated on the rewritten main,
so the rewrite waits until no sprint branch is open. Only the user can
force-push main past its ruleset.
