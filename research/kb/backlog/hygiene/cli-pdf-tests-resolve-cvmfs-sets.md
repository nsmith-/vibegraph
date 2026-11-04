---
type: Backlog Item
title: cli_first_run and cli_pdf_cache fail on hosts that mount /cvmfs
description: "The PDF resolution order puts the CVMFS LHAPDF area before the cache, so on a host with /cvmfs mounted these tests resolve sets from CVMFS instead of the cache tier they exercise."
area: hygiene
state: open
priority: medium
closes_when: "The resolution order lets a test disable the CVMFS tier (an env override or an injected path), cli_first_run and cli_pdf_cache use it, and both pass on a host with /cvmfs/sft.cern.ch mounted."
blocked_by: []
opened: 2026-11-04
tags: [tests, pdf, cache, cern-host]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: lhapdf, resource: "../../../../vibegraph-lib/src/cache/lhapdf.rs", title: "vibegraph-lib/src/cache/lhapdf.rs, CVMFS_LHAPDF_SETS"}
  - {id: first-run, resource: "../../../../vibegraph-cli/tests/cli_first_run.rs", title: "vibegraph-cli/tests/cli_first_run.rs"}
  - {id: pdf-cache, resource: "../../../../vibegraph-cli/tests/cli_pdf_cache.rs", title: "vibegraph-cli/tests/cli_pdf_cache.rs"}
---
On the CERN host, pristine main 58ab1865 fails 3 of 5 `cli_first_run` tests
and 6 of 12 `cli_pdf_cache` tests. The lookup order (LHAPDF data path, CVMFS,
cache, download) finds the CVMFS sets at
`/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current` first, so the tests never
reach the cache tier they assert on. Every CERN-host gate run carries these as
known failures, which hides a real regression in them. The tests need a way to
switch the CVMFS tier off.
