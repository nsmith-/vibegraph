---
type: Backlog Item
title: acceptance.yml has no weekly schedule trigger
description: Nothing runs acceptance.yml on a timer, so a CERN repackaging of the pinned PDF archive is detected only by an ignored test nobody runs.
area: hygiene
state: blocked
priority: medium
closes_when: acceptance.yml carries a weekly schedule trigger and its first scheduled run is green.
blocked_by: [acceptance-yml-never-passed]
opened: 2026-08-06
tags: [acceptance, ci, pdf, upstream-risk]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L520-L524", title: "TODO.md entry T039"}
---
The PDF pin is a checksum against an archive CERN could repackage at any time.
The only detector today is an `#[ignore]`d test that nothing runs on a timer. A
weekly `schedule` on `acceptance.yml` would be a second detector, because the
script downloads the PDF grids on every run on purpose. It stays off while the job
cannot pass at all ([acceptance-yml-never-passed](acceptance-yml-never-passed.md)),
because until then it can only fail.

A scheduled run has no release tag, so `scripts/acceptance.sh` takes
`releases/latest/download`. That path skips prereleases, so it tests the newest
`v*` binary and not a `refdata-*` bundle. The comment in `acceptance.yml` still
gives the reason for leaving the schedule off as "until a first release exists".
That reason is out of date: `v0.1.0` exists, and the real blocker is that no run
has been green yet. Risk background: [note 24](../../24-user-distribution-and-proton-events-plan.md)
§U2 and its "Upstream repackaging is a live operational risk" finding.
