---
type: Backlog Item
title: "Neither CI nor the hygiene review refuses a commit of build output"
description: "A debug build tree reached main through PR 8's squash; no CI step checks tracked files against .gitignore or a size limit, and the hygiene review checklist does not ask."
area: hygiene
state: open
priority: medium
closes_when: "A CI job fails when a tracked file matches .gitignore (git ls-files -ci --exclude-standard) or a new file exceeds a size limit, a test commit shows it failing, and the hygiene review concept lists the check."
blocked_by: []
opened: 2026-11-23
tags: [ci, git, hygiene-review, repository-size]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: review, resource: "../../workflow/hygiene-review.md", title: "Hygiene review"}
---
PR 8 (#171) committed `vibegraph-cli/target/` (2889 files, about 917 MB), and
neither its hygiene review nor CI caught it. A cheap mechanical gate would
have: once `.gitignore` says `target/`, `git ls-files -ci --exclude-standard`
lists any tracked build output, and a per-file size cap catches the binaries
even where the ignore rule has a gap, as `/target` did here.
