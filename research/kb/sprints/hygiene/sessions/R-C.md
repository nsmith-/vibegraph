---
type: Session Brief
title: "R-C: hygiene review of models, diagrams and reweighting"
description: "Review ufo, diagrams, onshell and reweight on the four hygiene points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/src/ufo/`, `diagrams/`, `onshell.rs`, `reweight/` (about 16k lines) and their in-module tests.

**Already claimed, read and don't re-report:** ufo-asin-acos-evaluate-as-acsc-asec, make-anti-negates-singlet-octet-colour, process-model-and-artifact-doc-comments-stale, reweight-forbidden-onshell-guard-is-dead, feyngraph-submodule-pin-differs-from-build.

**Look especially at:** the UFO expression grammar's other function mappings (the asin/acos slip suggests a table worth checking entry by entry against Python's `cmath`); enum variants nothing constructs (the `OnShell::Forbidden` pattern); refusal paths with no test that triggers them.
