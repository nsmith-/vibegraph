---
type: Backlog Item
title: generate re-reads the integrate inputs
description: "generate takes the proc card and run card again beside the artifact; it should take the artifact plus generate-time inputs only, with the run card split by stage."
area: feature
state: open
priority: high
closes_when: "`generate` refuses a proc card or `--run-card` as an argument error and requires `--seed`; a test asserts every `PARAM_DEFAULTS` name has a stage; the artifact (version 13) carries the proc card, the integrate settings and the generate settings."
blocked_by: []
opened: 2026-10-21
---
Today `vibegraph generate` re-parses the proc card and run card beside the
artifact, and compares them with what the grid was trained on. The user's
2026-10-21 decision is that information a workflow step introduces lives in
that step's artifact and is never requested again.

So `generate` takes the artifact plus generate-time inputs only (`--seed`,
`--nevents`, output, strategy, reweighting). The run card splits into an
integrate stage and a generate stage. The model is checked by digest at
`generate`.

The design, with field classification, structures, CLI and migration, is
[integrate and generate split by data dependency](../../pipeline/integrate-generate-split.md).
It also takes the `LHAPDF_DATA_PATH` environment tier and the removal of the
`validation/pdf` fallback (C7, C8 in
[run inputs in the artifact](../../pipeline/run-inputs-in-the-artifact.md)).
