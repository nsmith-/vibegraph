---
type: Backlog Item
title: The library reweight guard on $ processes can never fire
description: "OnShell::Forbidden is never assigned, so ReweightPlan's refusal of a $ process is dead code; only the CLI's card-level check refuses, and a library caller is unguarded."
area: hygiene
state: open
priority: low
closes_when: "A library ReweightPlan built for a process with a $ veto is refused (test), and OnShell::Forbidden is either assigned where the veto applies or removed with its doc."
blocked_by: []
opened: 2026-10-09
tags: [reweight, onshell-veto, refusal, dead-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: engine, resource: "../../../../vibegraph-lib/src/reweight/engine.rs", title: "reweight/engine.rs ~:170-176, the only reader of OnShell::Forbidden"}
  - {id: diagram, resource: "../../../../vibegraph-lib/src/diagrams/diagram.rs", title: "diagrams/diagram.rs ~:98-100, OnShell::Forbidden and its doc"}
  - {id: cli, resource: "../../../../vibegraph-cli/src/generate.rs", title: "vibegraph-cli/src/generate.rs ~:1056, the live card-level refusal"}
---
`ReweightPlan` (`vibegraph-lib/src/reweight/engine.rs` ~:170-176) refuses a set
whose diagrams carry a propagator with `onshell == OnShell::Forbidden`. Nothing
assigns that variant: the `$` veto keeps its own per-subprocess bookkeeping
(`onshell.rs`). So the check never fires. The `OnShell` doc
(`diagrams/diagram.rs` ~:98-100) says `$` lines are `Forbidden`, which is also
untrue.

`vibegraph generate` is unaffected: it refuses a reweight card on a `$` process
itself (`vibegraph-cli/src/generate.rs` ~:1056, `forbidden_onshell`). A library
caller gets no refusal and reweights a vetoed process without the veto. The
item [reweight-forbidden-schannel-and-as-refused](../feature/reweight-forbidden-schannel-and-as-refused.md)
is about supporting the case. This one is about the library guard, which
should refuse until that support lands. Found by drafter D13 and confirmed by
verifier V13.
