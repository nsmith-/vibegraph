---
type: Backlog Item
title: A model Co-authored-by trailer reached main through a cloud-agent PR
description: "ef84a12 (#15) carries a Co-authored-by: Claude trailer, which AGENTS.md forbids; cloud-agent PRs land without the user's handle, so the trailer slipped past review."
area: hygiene
state: needs-user
priority: low
closes_when: "The user has fixed the trailer on main (or recorded that it stays), and PRs opened by cloud agents carry only the Assisted-by trailer."
blocked_by: []
opened: 2026-10-09
tags: [git-history, attribution, user-call]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User comment on the taxonomy review, 2026-10-09"}
---
History was rewritten once (`27e9974`, 2026-08-06) to turn model
`Co-Authored-By:` trailers into `Assisted-by: <harness>:<model>`, the
convention in `AGENTS.md`. One trailer has reached `main` since: `ef84a12`
(#15) ends with `Co-authored-by: Claude`. The PR came from a cloud agent
acting without the user's GitHub handle, so it was not caught. A second
`Co-authored-by` sits on `f14ac4e` (2026-08-06), checked only on a shallow
clone.

The user will fix this later. Two parts:
- the trailer(s) already on `main`: a history rewrite (`scripts/rewrite-ai-trailers.py`) now that the repository is public, or leave them;
- stop it recurring: the cloud-agent PR path needs the same trailer rule as local sessions.
