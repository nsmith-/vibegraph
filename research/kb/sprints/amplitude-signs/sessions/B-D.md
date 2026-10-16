---
type: Session Brief
title: "B-D: bring seven more concepts in line with the root-free convention sign"
description: "Rewrite the passages of seven kb concepts that still describe Diagram::anchor, the anchor reference tree or the removed role-sign functions, so the bundle describes convention_sign as it is."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [B-H]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
PR #30 replaced the anchor-read role signs with `convention_sign`
(`helas/eval/root_diagram.rs`). B-H rewrote the five concepts that were about
the convention itself. Seven more mention it in passing and are now false
(B-H report, Found 2):

- `amplitudes/evaluator-architecture.md` (~:74)
- `amplitudes/four-fermion-vertices.md` (~:124-152)
- `amplitudes/fermion-flow-and-crossing.md` (~:82)
- `amplitudes/global-phase-i-counting.md` (~:98)
- `amplitudes/charge-flow-phase-study.md` (~:100-143)
- `validation/bit-exact-amplitude-debugging.md` (~:92-160)
- `performance/diagram-rooting.md` (~:4, :56, :128)

The paths are a guide; find each concept with `grep -rl` if one has moved.

For each passage: read the current code, then rewrite it to describe what the
code does now, in its own terms, and link the rewritten convention concepts
(`amplitudes/convention-sign-inventory.md`,
`amplitudes/rooting-invariance-and-canonical-form.md`,
`amplitudes/vector-vertex-signs.md`, `amplitudes/fermion-line-sign.md`). No
history framing: say what is, not what was, and let git keep the history.

A **methodology** concept such as `bit-exact-amplitude-debugging` may keep its
worked cases, which record what was done at the time, but it must not state the
old rule as the current one. Where a case names a removed function, say once
that the rule it produced has since been replaced, and link the inventory.

Set each edited concept's `status: draft`. Do not touch code. Do not edit
`research/kb/history/`.

Gate: `pixi run -e kb kb-index` and `pixi run -e kb kb-lint` (0 errors), and
`grep -rn 'Diagram::anchor\|yang_mills_vvv_sign\|vector_contact_sign\|gluon_scalar_current_sign\|fermion_current_line_sign' research/kb --include=*.md | grep -v 'history/\|sprints/'`
returns nothing that states those as current.
