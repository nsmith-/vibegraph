---
type: Design Decision
title: MLM matching never applies to decays
description: "Matching or xqcut on a decay process is refused permanently: MadGraph's MLM acts on the production process, so matching inside a decay has no defined meaning."
decided: 2026-10-09
decided_by: human:nsmith-
verified: [{by: "human:nsmith-", at: 2026-10-09}]
status: stable
tags: [scope, mlm, decays]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: user, resource: "https://claude.ai/artifact/N5Rm9hTa3rpkQ3duboK5UB", title: "User decision on the MLM refusals, 2026-10-09"}
  - {id: n41, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md", title: "Note 41-mlm §4 M1"}
---
`ickkw = 1` or `xqcut > 0` on a 1→n decay process stays a hard error. MLM
matching, as MadGraph implements it, clusters the hard process's jets against
the incoming partons and their factorisation scales. A decay has no incoming
partons and no PDF factors, so "matching a decay" has no meaning to
reproduce. The refusal is part of the release scope's hard-error rule
([release-scope-lo-mlm](release-scope-lo-mlm.md)), not a missing feature.

Today the refusal is `FixedBeamMatching`, shared with fixed-beam cards, which
are a separate, open backlog item. When fixed-beam matching is built, the
decay case keeps its refusal under its own reason.
