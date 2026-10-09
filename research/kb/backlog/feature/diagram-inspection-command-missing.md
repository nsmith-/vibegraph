---
type: Backlog Item
title: No command shows which diagrams a process card selects
description: There is no vibegraph enumerate to draw and summarize a card's diagrams before integrating, nor a diagram artifact integrate can reuse.
area: feature
state: open
priority: medium
closes_when: vibegraph enumerate writes SVG drawings, a summary page (diagram counts per subprocess, coupling orders, flavour groups) and a binary artifact that integrate accepts in place of re-enumerating.
blocked_by: []
opened: 2026-09-06
tags: [cli, diagrams, user-request]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L633-L642", title: "TODO.md entry T057"}
---
A user request on PR #4. The CLI has `integrate`, `generate` and
`check-events` only (`vibegraph-cli/src/main.rs`). The workflow is
MadGraph's `display diagrams`: check that a card means the intended process
and nothing more before spending an integration on it.

The command takes a process card and reports every contributing diagram:
SVG drawings, a summary page (diagram counts per subprocess, coupling
orders, flavour groups), and a binary artifact `integrate` accepts in place
of re-enumerating. feyngraph's `drawing/` module is a candidate for the
drawings. The artifact half also delivers the "bundle the compiled program"
piece of the self-contained-artifact backlog item, since the enumerated
diagrams are that program's input.
