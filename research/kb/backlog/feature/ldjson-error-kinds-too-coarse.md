---
type: Backlog Item
title: "generate's library catch-all and the systematics request errors stream under the wrong ldjson error kind"
description: "GenerateError::Other carries output-file and artifact-disagreement errors, streamed as run; systematics request errors (NominalNotFirst, request-content Entry) stream as pdf."
area: feature
state: open
priority: low
closes_when: "generate's IO and artifact-disagreement errors are their own library variants mapped to io and artifact, and the systematics request errors to usage or card by origin, each pinned by a refusal test (one on a stale channel-map digest)."
blocked_by: []
opened: 2026-11-05
tags: [cli, ldjson, refusal, sprint-g]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
G-HY5 Found 1–2 ([report](../../sprints/v02-output-modes/sessions/G-HY5-report.md)): lib `generate.rs` ~761, 820–856, 1405; `plan.rs` ~370–400. Adding or splitting a kind bumps the ldjson `schema_version` by its rule.
