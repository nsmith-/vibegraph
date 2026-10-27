---
type: Backlog Item
title: "Fetching the reference bundle fails on cloud hosts"
description: "fetch_common.sh needs a zstd binary, which cloud containers lack, and its gh release download route fails there because GraphQL is blocked; the REST asset endpoint works."
area: hygiene
state: open
priority: low
closes_when: "vg_download_release_asset falls back to the REST asset endpoint (gh api -H 'Accept: application/octet-stream' …/releases/assets/<id>) and the unpack works without a zstd binary or names the need in the setup docs."
blocked_by: []
opened: 2026-10-26
tags: [tooling, refdata, cloud, fetch]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f2, resource: "../../sprints/pdf-from-run-card/sessions/F2-report.md", title: "pdf-from-run-card F2 report, Found 4"}
---
`validation/fetch_common.sh` downloads with `gh release download` (near :125) and
unpacks with `zstd -dc` (:220-221), dying if `zstd` is not on `PATH`. In cloud
sessions both fail: `gh release download` goes through GraphQL, which the proxy
blocks, and the image has no `zstd`. Two sessions (pdf-from-run-card F2, the
SIGABRT probe) worked around it the same way: `gh api -H 'Accept:
application/octet-stream' repos/nsmith-/vibegraph/releases/assets/<id>`, then
`fetch_refdata.sh` with `VIBEGRAPH_REFDATA_SOURCE` and a python-zstandard shim
([F2](../../sprints/pdf-from-run-card/sessions/F2-report.md)).
