---
type: Session Brief
title: "F-C: model, diagram and reweight fixes"
description: "Fix R-C's 12 triaged findings and two found bugs in ufo, diagrams, onshell and reweight, and close five claimed items including asin/acos and make_anti."
status: draft
agent: feature-dev (Opus)
depends_on: [triage]
closes: [ufo-asin-acos-evaluate-as-acsc-asec, make-anti-negates-singlet-octet-colour, process-model-and-artifact-doc-comments-stale, reweight-forbidden-onshell-guard-is-dead, feyngraph-submodule-pin-differs-from-build]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-C.2, .3, .4, .5, .6, .7, .8, .9, .10, .16, .18 (the parse-error test only), .20; R-C Found 1 (`EvaluatedModel::recompute` on a restriction-locked parameter); R-C Found 2 (the `scan` substring refusal) (`sessions/R-C-report.md`).

**Claimed items closed here:** ufo-asin-acos-evaluate-as-acsc-asec, make-anti-negates-singlet-octet-colour, process-model-and-artifact-doc-comments-stale, reweight-forbidden-onshell-guard-is-dead, feyngraph-submodule-pin-differs-from-build.

**Notes:**
- **make-anti-negates-singlet-octet-colour:** list every reader of `Particle::color` first (the item says why). Run the banked colour and amplitude gates after the change.
- **R-C.4:** the table test goes with the asin/acos item's closing test.
- **feyngraph-submodule-pin-differs-from-build:** bumping the submodule is a gitlink change. The worktree has no submodule checkout, so update the gitlink with `git update-index --cacheinfo 160000,<sha>,research/refs/feyngraph` and `research/refs/README.md`, and say how you verified the sha.
