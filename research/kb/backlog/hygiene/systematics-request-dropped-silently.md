---
type: Backlog Item
title: A run card requesting systematics loses them without a warning
description: "systematics_program, systematics_arguments and sys_* are classified IgnoredBenign, so a card that asks for variation weights gets none and nothing says so."
area: hygiene
state: open
priority: medium
closes_when: "A card with use_syst = True and systematics_program, systematics_arguments or a sys_* field off its default warns, naming the variation weights that will not be written, by a test; the warning goes when the weights are written."
blocked_by: []
opened: 2026-10-20
tags: [runcard, systematics, classification, warning, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`FIELD_CLASSES` marks `systematics_program`, `systematics_arguments` and
`sys_scalefact`, `sys_alpsfact`, `sys_matchscale`, `sys_pdf`,
`sys_scalecorrelation` as `IgnoredBenign(B_SYST)`
(`vibegraph-lib/src/runcard/classes.rs:371-377`), on the argument that they
only add `<rwgt>` weights. That holds for σ, but the user's output then lacks
weights the card asked for, silently.

User decision (2026-10-20): **warn, do not refuse**, until
[systematics-variation-weights-not-written](../feature/systematics-variation-weights-not-written.md)
lands. Frequency is in the
[CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md): every LO
card sets `use_syst`. The classification is described in
[field classification](../../run-card/field-classification.md).
