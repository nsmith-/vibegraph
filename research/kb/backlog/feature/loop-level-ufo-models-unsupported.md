---
type: Backlog Item
title: Loop-level UFO models are unsupported
description: "Loop UFOs (`loop_sm`, NLO models) are outside the LO charter: their counterterm files and loop attributes are ignored, and `[QCD]` processes are refused."
area: feature
state: open
priority: low
closes_when: The project adopts NLO in its charter and a loop-UFO process is computed and gated against MadGraph5_aMC@NLO; until then this stays as a deliberate refusal.
blocked_by: []
opened: 2026-09-07
tags: [non-sm-ufo, descoped-v1, nlo, ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L926-L927", title: "TODO.md entry T080"}
---
The project's charter is leading order, so loop-level UFOs are out of scope.

The parser tolerates `loop_sm`'s attribute assignments: it skips
`.counterterm`/`.loop_particles` (`vibegraph-lib/src/ufo/particles.rs:147`).
`CT_vertices.py`, `CT_couplings.py` and `CT_parameters.py` are never read. The
process card refuses `[QCD]`/`[real=QCD]` (`Unsupported::LoopSpec`) and `!a!`
(`Unsupported::PhotonTag`) in `vibegraph-lib/src/diagrams/check.rs`.

[Note 04](../../history/notes/04-ufo-parsing-future.md) holds the parser history.

This is the largest scope extension in the list: virtual amplitudes,
renormalisation and subtraction. It is filed so the boundary stays visible, not
as planned work.
