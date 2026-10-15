---
type: Backlog Item
title: acceptance.yml has no weekly schedule trigger
description: Nothing runs acceptance.yml on a timer, so a CERN repackaging of the pinned PDF archive is detected only by an ignored test nobody runs.
area: hygiene
state: open
priority: medium
closes_when: acceptance.yml carries a weekly schedule trigger and its first scheduled run is green.
blocked_by: []
opened: 2026-08-06
tags: [acceptance, ci, pdf, upstream-risk]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L520-L524", title: "TODO.md entry T039"}
---
The PDF pin is a checksum against an archive CERN could repackage at any time.
The only other detector is an `#[ignore]`d test that nothing runs on a timer. A
weekly `schedule` on `acceptance.yml` is a second detector, because the script
downloads the PDF grids on every run on purpose.

**2026-10-15:** the schedule is on (Mondays 06:17 UTC), after the first green
dispatch against `v0.1.0` (run 38771664781). A scheduled run has no release
tag, so `scripts/acceptance.sh` takes `releases/latest/download`, which skips
prereleases and so tests the newest `v*` binary. The job's `if:` lets every
non-`release` event through, so the schedule is not filtered out by the
`refdata-*` guard. What is left is to read the first scheduled run; when it is
green, this item is deleted. Risk background: [note 24](../../history/notes/24-user-distribution-and-proton-events-plan.md)
§U2 and its "Upstream repackaging is a live operational risk" finding.
