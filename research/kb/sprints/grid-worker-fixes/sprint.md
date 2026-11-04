---
type: Sprint
title: "Grid-worker fixes: PDF lookup without a home directory, and quark-jet ttx at generate"
description: "A PDF set on LHAPDF_DATA_PATH is found with no HOME, and generate writes events for p p > t t~ j with a quark-only jet."
status: draft
active: true
tags: [pdf, cache, grid-worker, generate, resonance, flavour-groups, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
## Goal and exit criteria

Part of sprint F (CMS production readiness). Done when both items'
`closes_when` hold, each pinned by a test that fails before the fix.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `pdf-tiers-need-a-home-directory`
- `generate-refuses-quark-only-jet-ttx`

Both are small `generate` failures on a production worker, so one PR reads
them as one change.

## Design

- **Home directory.** `pdf_tiers` (`vibegraph-cli/src/assets.rs`) resolves
  the cache root first and refuses when it is absent, before
  `$LHAPDF_DATA_PATH` is read. The cache root becomes required only when the
  cache or download tier is reached; its refusal then names that tier. Pinned
  by a CLI test with `HOME` and `VIBEGRAPH_HOME` unset and the set on
  `$LHAPDF_DATA_PATH`, and by one that still refuses when the set is
  elsewhere.
- **Quark-jet ttx.** The resonance mapping in a mixed-flavour group looks up
  the wrong member (`g u > t t~ u` against the subprocess with outgoing
  `[6, -6, 1]`). The session reproduces it on the `gq` card from
  `sigma-ttx-mlm-at1` V2, finds which member the lookup should use, and fixes
  it. Gates: generate writes events on that card; the per-subprocess event
  fractions agree with integrate's σ split; the resonance and phase-space
  gates the `extended-validation` skill names stay green.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| G-F1 | feature-dev (Sonnet) | – | CLI test, hermetic | `pdf-tiers-need-a-home-directory` |
| G-F2 | feature-dev (Opus) | – | failing-first test, extended validation | `generate-refuses-quark-only-jet-ttx` |
| G-V1 | validation-dev (Sonnet) | G-F2 | event fractions against integrate on the gq card | – |
| G-HY1 | hygiene-dev (Sonnet) | G-V1, G-F1 | hermetic, fmt, clippy | – |

No performance session: neither change touches a hot path.
