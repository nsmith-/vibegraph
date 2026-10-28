---
type: Backlog Item
title: The integration artifact cannot say which sampler trained its channels
description: "ChannelGrid::grid is a bare bincode VegasGrid with no kind tag, so an artifact cannot carry another sampler's trained state and generate cannot refuse a kind it does not know."
area: feature
state: open
priority: medium
closes_when: "ChannelGrid carries a kind-tagged trained sampler state and its coordinate role list; the run records any per-role bin-width override; the artifact is written at the next version after 14, per the artifact-versioning concept; generate rebuilds the sampler from the artifact's kind and refuses an unknown kind, a kind/ndim mismatch and a role list this build lays out differently, naming the field; a VEGAS-only, window-free artifact replays to the same LHE at a fixed seed as before the change; round-trip and refusal tests."
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

What the artifact records and what `generate` refuses are settled in
[the joint design](../../phase-space/sampler-and-map-seams.md) (D7): per channel
the kind-tagged trained state and its coordinate role list (the layout the
grid was trained in); per run any per-role bin-width override, beside
`grid_rule`. The map windows record their own field in `MapChoices` when they
land. Adaptation settings other than
the VEGAS `GridRule` stay out.

**Versioning** follows [artifact versioning](../../pipeline/artifact-format-versioning.md)
as PRs #69 and #60 left it: every artifact is written at the current version
(14 after #60), and `version_for` is gone. This item takes the next free version
at merge. A version-14 file may upgrade to the VEGAS kind with its role list
not recorded, or the floor may rise; the item decides and says why. Byte identity of the file therefore no longer holds across
the step; the inertness check is replay: the same LHE from the same artifact
content and seed.

Note the name: `ChannelGrid::sampler: Option<ChannelSampler>` already exists and
records the phase-space map composition, so the new field needs another name.

Tests: VEGAS round-trip; uniform-kind round-trip; an unknown kind, a kind/`ndim`
mismatch and a changed role list refused with a message
naming them.
