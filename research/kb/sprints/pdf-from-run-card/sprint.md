---
type: Sprint
title: "The run card picks the PDF set and member"
description: "integrate and generate take no PDF option: lhaid/pdlabel choose set and member, looked up through LHAPDF_DATA_PATH, CVMFS, the cache, then a consented download, and the artifact records them with a checksum."
status: draft
active: false
tags: [pdf, lhapdf, runcard, artifact, cli, cms, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, every clause pinned by a test.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `pdf-set-not-taken-from-run-card`

Part of sprint D, the CMS gridpack survey fixes (planner hand-off 2026-10-20,
amended the same day with the survey's missing features). The sibling PR
`param-card` (item `cli-has-no-param-card-option`) also adds run inputs to the
integrate artifact; D1 designs both PRs' artifact fields together, and the
user approves D1 before either implementation starts.

Hermetic tests never touch the real network: the download tier is tested
through an injected fetcher against a local file or server, never by setting
`VIBEGRAPH_FETCH_CONSENT` (planner, 2026-10-20).

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| D1 | feature-dev (Opus), design only | – | kb-lint; user approval | – |
| F1 | feature-dev (Opus) | D1 approved | hermetic, fmt, clippy, kb-lint | the item |
| V1 | validation-dev | F1 | banked PDF/σ gates (CI) | – |
| HY1 | hygiene-dev | V1 | hermetic, fmt, clippy | – |
