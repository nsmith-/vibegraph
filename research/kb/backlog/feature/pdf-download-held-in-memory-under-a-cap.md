---
type: Backlog Item
title: "A PDF set download is held whole in memory under a 256 MiB cap"
description: "HttpFetch reads the whole archive into memory and refuses past 256 MiB, though a run reads one member; the largest sets CMS cards name have not all been measured against the cap."
area: feature
state: open
priority: low
closes_when: "Archives stream to disk (or the cap is set from measured sizes of the sets CMS cards name), with a test that a download larger than memory-sized buffers succeeds through the injected fetcher."
blocked_by: []
opened: 2026-10-26
tags: [pdf, lhapdf, download, cache]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: d1, resource: "../../sprints/pdf-from-run-card/sessions/D1-report.md", title: "pdf-from-run-card D1 report, Found 2"}
  - {id: log, resource: "../../sprints/pdf-from-run-card/log.md", title: "pdf-from-run-card log, CVMFS layout"}
---
`MAX_ARCHIVE_BYTES` is `256 * 1024 * 1024` in `vibegraph-cli/src/fetch.rs`, and
the archive is held in memory. Every set is downloaded whole, although a run
reads one member ([D1](../../sprints/pdf-from-run-card/sessions/D1-report.md)).

Measured on the CERN host's CVMFS copy ([log](../../sprints/pdf-from-run-card/log.md)):
`NNPDF31_nnlo_as_0118_mc_hessian_pdfas` (325300) is 44.9 MB and
`NNPDF23_lo_as_0130_qed` 27.6 MB. The other sets CMS cards name (325500, 306000,
325100) are unmeasured. CVMFS and `$LHAPDF_DATA_PATH` come before the download
tier, so a CERN worker rarely reaches it.
