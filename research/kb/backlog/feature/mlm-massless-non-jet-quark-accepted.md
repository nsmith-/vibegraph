---
type: Backlog Item
title: MLM accepts a massless quark outside the jet definition
description: At ickkw = 1 a massless quark above maxjetflavor in the hard process has no collinear regulator; MadGraph now refuses such a card, and vibegraph still runs it.
area: feature
state: open
priority: medium
closes_when: "At ickkw = 1, a card whose hard process has a massless quark with maxjetflavor < |pdg| <= 5 on an incoming or non-decay outgoing leg is refused with a named error, and refusal tests pin a massless production b (refused), a massive b (accepted) and a massless b only from a decay (accepted)."
blocked_by: []
opened: 2026-10-26
tags: [mlm, runcard, refusal, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: mg-456, resource: "https://github.com/mg5amcnlo/mg5amcnlo/issues/456", title: "mg5amcnlo#456, closed as an unsupported setup"}
  - {id: mg-pr463, resource: "https://github.com/mg5amcnlo/mg5amcnlo/commit/010a59f1c39376d4ebff123e503f888e04efdbc0", title: "mg5amcnlo PR #463 merge (3.5.18): the launch and runtime refusals"}
---
Under MLM matching only jets are regulated by `xqcut`. A massless quark that is
not a jet (a `b` at `maxjetflavor = 4` in a five-flavour model) has nothing
controlling its collinear region, so σ depends on whatever other cut happens
to apply, and the shower-side matching does not treat it as a jet either.

Upstream answered
[#456](https://github.com/mg5amcnlo/mg5amcnlo/issues/456) by refusing the
setup in the 3.5.18 development branch: a launch-time `InvalidRunCard` at
`ickkw = 1`, and a runtime stop when a subprocess groups flavour combinations
that disagree on which legs are jets. vibegraph still accepts the card. Its
`rewgt` matches MadEvent only on a run's first event there (module docs of
`coupling/cluster/rewgt.rs`), and no banked MLM row reaches it
([madgraph-defects](../../validation/madgraph-defects.md), the `rewgt` row).

The refusal belongs with the other matching refusals in
`runcard/matching.rs`, but needs the legs' model masses, so it sits where the
process and the card meet. Scope it narrower than MadGraph's launch check,
which also refuses a massless `b` that appears only as a decay product (read,
not run). That `b` is finite, and the merging cut skips decay products. A
massive `b`, as in `sm`, must stay accepted.

Detail: [report drafts, "Upstream response"](../../validation/madgraph-upstream-report-drafts.md#upstream-response).
