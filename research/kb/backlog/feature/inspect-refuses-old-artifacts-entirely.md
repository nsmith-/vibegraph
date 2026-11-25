---
type: Backlog Item
title: "inspect refuses an artifact from an older enumeration version instead of reading what it can"
description: "With ENUMERATION_VERSION 4, inspect refuses every v0.2.2 artifact, so a user cannot read the σ, card or provenance an old run recorded."
area: feature
state: open
priority: low
closes_when: "inspect reads an artifact of an older enumeration version in a read-only mode that reports its recorded σ, cards and provenance with a prominent warning that the artifact cannot be integrated or generated from. integrate and generate keep refusing it, and a test covers both sides."
blocked_by: []
opened: 2026-11-25
tags: [inspect, artifacts, versioning, ux]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Found by sprint L's release-prep PR (#211). The version bump is right for every
command that computes from the artifact. `inspect` only reports, though, and
users who kept v0.2.2 outputs lose the record of what those runs found.
