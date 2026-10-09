---
type: Backlog Item
title: NLO generation is not implemented
description: Process lines asking for NLO ([QCD], [real=QCD], the !a! photon tag) are parsed and refused; the generator is LO only.
area: feature
state: open
priority: low
closes_when: A [QCD] process line produces an NLO sigma that matches MadGraph5_aMC@NLO on a banked row.
blocked_by: []
opened: 2026-09-25
tags: [nlo, process-grammar]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L601-L602", title: "TODO.md entry T053"}
---
Sequenced after MLM, which has landed. The loop specification (`[QCD]`,
`[real=QCD]`) and the photon tag (`!a!`) are in the proc-card AST and refused
by `check_supported` as `Unsupported::LoopSpec` / `Unsupported::PhotonTag`
(`vibegraph-lib/src/diagrams/check.rs`).

[Note 38 §3.2](../../history/notes/38-process-grammar-sprint-plan.md) says where the
Born/real/virtual split would attach: the per-diagram provenance slot on the
diagram container that `helas` receives. No design beyond that exists; a
first step is a plan note scoping subtraction scheme, virtuals (an external
one-loop provider or not) and the reference to gate against.
