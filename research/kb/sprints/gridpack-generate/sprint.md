---
type: Sprint
title: "gridpack-generate"
description: "generate takes the integrate artifact plus generate-time inputs only: the run card split by stage, artifact version 13, a required --seed, and LHAPDF_DATA_PATH."
status: draft
active: true
tags: [generate, artifact, runcard, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---

## Goal and exit criteria

The H1 step of [integrate and generate split by data dependency](../../pipeline/integrate-generate-split.md),
with the choices the user decided on 2026-10-21 (§5 there).

Exit criteria:
- `generate <ARTIFACT> --seed S` works from the artifact alone.
- A proc card or `--run-card` passed to `generate` is an argument error, and
  a missing `--seed` is refused.
- Every run-card field has a stage.
- The artifact is version 13, and older versions are refused.
- The model is checked by digest at `generate`.
- `LHAPDF_DATA_PATH` replaces `VIBEGRAPH_PDF_DIR`, and the `validation/pdf`
  fallback is removed (C7, C8).

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):
- `generate-rereads-integrate-inputs`

Not in scope:
- the PDF taken from the run card (H2, `pdf-set-not-taken-from-run-card`);
- the parameter card (`cli-has-no-param-card-option`).

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | workspace tests | – |
| F2 | feature-dev (Opus) | F1 | workspace tests | `generate-rereads-integrate-inputs` |
| V1 | (folded into CI) | F2 | the banked layer runs the proton path, which this container cannot | – |
| HY1 | hygiene-dev | V1 | workspace tests | – |
