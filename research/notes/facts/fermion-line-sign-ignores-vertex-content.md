---
type: Caveat
title: The fermion-line reversal sign does not depend on vertex content
description: Note 35's rule gating the per-propagator −1 on a line's Dirac-matrix content was reverted on 2026-10-05; every uncrossed line takes one −1 per internal propagator.
tags: [non-sm-ufo, fermion-flow, sign-convention, supersedes-note-35]
status: draft
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L931-L934", title: "TODO.md entry T082"}
  - {id: todo-reweight, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L696-L700", title: "TODO.md reweight entry (Found on the way)"}
---
**Superseded.** [Note 35 §T3 and §10.1 rule 5](../35-ufo-lorentz-sprint-plan.md)
make the fermion-line reversal sign depend on the line's Dirac-matrix content.
Under that rule, a line of Yukawa-type vertices (`Identity`/`Gamma5`/projector)
took no per-propagator −1. Commit `7f523ad` (PR #13, reweight, 2026-10-05)
reverted the rule and removed `carries_dirac_matrix`.

**Current rule.** `Diagram::fermion_line_sign`
(`vibegraph-lib/src/diagrams/diagram.rs:570`) gives every uncrossed line one −1
per internal fermion propagator, whatever its vertices are. A crossed line takes
a single −1.

**Why note 35's rule looked right.** The exemption was pinned on the toy row
`qt qt~ > o8 o8`, where it cancelled a second bug: the operator-free scalar
vertices `SSS1`/`SSSS1` were missing their scalar-sink −1. After both fixes,
against MadGraph's per-diagram `AMP()`, `ta+ ta- > t t~ h h` goes from 24 wrong
signs of 96 to 0 and `ta+ ta- > t t~ h` (14 diagrams) has none, while
`qt qt~ > o8 o8` still agrees per flow to 2e-16. New standalone JAMP rows gate the rule per helicity: `tata_to_ttxh` (2e-15),
`tata_to_ttxhh` (2e-14) and `bbx_to_hh`.

**For a reader.** Note 35 §10.1 rule 5, and its account of the `o8 o8` bug, do
not describe the current convention. The toy row did expose a sign that no SM
process isolates. But it also pinned a rule that only looked right because two
errors cancelled, and real Yukawa-line processes overturned it.
