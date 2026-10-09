---
type: Backlog Item
title: acceptance.yml runs and fails on every refdata prerelease
description: acceptance.yml's release:published trigger fires on refdata-* prereleases, which carry no binary, so each one is a red run that 404s.
area: hygiene
state: open
priority: medium
closes_when: Publishing a refdata-* release no longer produces a failed acceptance.yml run (it is skipped or not triggered), while v* releases still trigger it.
blocked_by: []
opened: 2026-08-01
tags: [acceptance, ci, release, refdata]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L509-L519", title: "TODO.md entry T038"}
---
`.github/workflows/acceptance.yml` subscribes to `release: [published]`, and
`refdata-*` reference-data bundles are published as GitHub prereleases whose
only asset is `vibegraph-refdata-N.tar.zst`. Every one of them starts acceptance,
which passes the tag through to `scripts/acceptance.sh --tag refdata-N`. The run
then fails in about 10 s with `curl: (22) ... 404` and `ACCEPTANCE FAILED: cannot
download .../releases/download/refdata-9/vibegraph-x86_64-unknown-linux-musl`.
This has happened on refdata-2 through refdata-9, including refdata-8 (2026-09-27)
and refdata-9 (2026-10-04, run 37166504559), both after the repository went public.

These red runs look like the old private-repo 404, so they hide whether the
public-repo fix worked. Fix: skip prereleases, or tags not matching
`v*.*.*`, in the job's `if:` (`github.event.release.prerelease` is available
on the event).
