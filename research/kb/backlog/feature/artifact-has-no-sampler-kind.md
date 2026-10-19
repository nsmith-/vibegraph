---
type: Backlog Item
title: The integration artifact cannot say which sampler trained its channels
description: "ChannelGrid::grid is a bare bincode VegasGrid with no kind tag, so an artifact cannot carry another sampler's trained state and generate cannot refuse a kind it does not know."
area: feature
state: open
priority: medium
closes_when: "ChannelGrid carries a kind-tagged trained sampler state; a VEGAS-only artifact is written byte-identical at the version version_for gives it today (9–11 unbounded, 12 with a recorded grid rule); another kind writes a new FORMAT_VERSION that older builds refuse by number; generate rebuilds the sampler from the artifact's kind, with round-trip and refusal tests."
blocked_by: [sampler-hardwired-to-vegas-grid]
opened: 2026-10-17
tags: [artifact, sampler, serialisation, versioning, refactor]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
---
`artifact::ChannelGrid::grid` (`artifact.rs:283`) is a `VegasGrid`, encoded by
bincode, which is not self-describing: a reader has to know the type it is
decoding, and nothing in the file names it. A trained FOAM tree, tree ensemble
or flow cannot be banked, and a build that does not know a kind cannot refuse it
before misreading its bytes.

Resolve it with a sampler-kind enum around the trained state, following the
precedent `IntegrateArtifact::version_for` already sets: a writer records the
oldest version whose schema holds what it banks. A VEGAS-only artifact stays at
the version `version_for` gives it today (9–11 under the unbounded rule, 12
when it records the default bin-width bound) and today's bytes (bincode would otherwise add a variant tag),
and only an artifact holding another kind needs the new version, which
`read_from_path` refuses on an older build by number rather than by a decode
error. Versions 3–11 upgrade to the VEGAS kind. `generate` takes the kind from
the artifact, never from a flag, and keeps its `ndim` checks
(`generate.rs:957`, `:1502`) against whatever the kind reports. The VEGAS
`GridRule` is already in the wire format (top-level `grid_rule:
Option<GridRule>`, PR #48; `None` reserved for an artifact with no VEGAS grid);
other adaptation settings (damping) stay out, and recording a non-VEGAS kind's
config for provenance is optional and, if added, goes beside that field.

Note the name: `ChannelGrid::sampler: Option<ChannelSampler>` already exists and
records the phase-space map composition, so the new field needs another name.

Tests: VEGAS round-trip bytes equal to a pre-change artifact; uniform-kind
round-trip; an unknown kind and a kind/`ndim` mismatch refused with a message
naming them. Design: [importance-sampler-seam](../../phase-space/importance-sampler-seam.md).
