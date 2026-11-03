---
type: Backlog Item
title: "Validation fetches do not retry a reset connection, and pdfsets.index is unpinned"
description: "vg_download (validation/fetch_common.sh) gives up on the first curl error, and fetch_cms.sh takes pdfsets.index with no checksum."
area: hygiene
state: open
priority: low
closes_when: "vg_download retries transient curl failures, and the pdfsets.index the CMS fetch uses is pinned by sha256 or replaced by the .info files' SetIndex fields."
blocked_by: []
opened: 2026-11-03
tags: [fetch, network, pdf, reproducibility]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: mv1, resource: "../../sprints/cms-pdf-memory/sessions/M-V1-report.md", title: "cms-pdf-memory M-V1 report, Found"}
---
One run of `fetch_cms.sh` stopped on `curl: (35) Connection reset by peer`.
Rerunning it resumed, but a long-tier run would fail outright. `pdfsets.json`
pins every archive and `.info` file, but not the `pdfsets.index` the lookup
reads ids from.
