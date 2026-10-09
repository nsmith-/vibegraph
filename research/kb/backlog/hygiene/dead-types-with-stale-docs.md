---
type: Backlog Item
title: Unused types carry docs that describe uses they do not have
description: "Applicability::ProtonBeams classifies no field, Intertwiner2Leg/3Leg/4Leg have no implementors, and Vertex3/GaugeVertex have no users; their docs and a doc example name nonexistent types."
area: hygiene
state: open
priority: low
closes_when: "Each listed type is deleted, or given its user, and no doc names WeylBasis, MinkowskiRep or a fixed-energy pdlabel card as a reason for a type."
blocked_by: []
opened: 2026-10-09
tags: [dead-code, visibility, stale-comment]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: classes, resource: "../../../../vibegraph-lib/src/runcard/classes.rs", title: "runcard/classes.rs Applicability (~:44-51), test docs (~:552-564, ~:633-636)"}
  - {id: repr, resource: "../../../../vibegraph-lib/src/helas/repr/coupling.rs", title: "helas/repr/coupling.rs Vertex3 / GaugeVertex and its TODO sections"}
---
For the hygiene sprint's visibility and maintainability pass:

- **`Applicability::ProtonBeams`** (`vibegraph-lib/src/runcard/classes.rs`
  ~:47-51). No field is classified with it: `pdlabel1/2` moved to `Consumed`,
  and `ignored_physics_fields_are_refused` asserts `proton_only == 0`. Its doc,
  and the test doc at ~:633-636, still say the fixed-energy cards that set
  `pdlabel1/2` are why it exists.
- **`Intertwiner2Leg` / `3Leg` / `4Leg`** (`helas/repr/intertwiner.rs`). The
  module doc says "nothing implements them yet", and nothing does.
- **`Vertex3`, `GaugeVertex`** (`helas/repr/coupling.rs`). No users outside
  the file. The `Vertex3` doc example (~:201) names `WeylBasis` and
  `MinkowskiRep`, which do not exist, and the file has four `# TODO` sections
  (~:38, :208, :249, :277).

Found by Phase 2 drafters D9 and D12; confirmed by grep.
