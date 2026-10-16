---
type: Session Report
title: "B-D report: seven concepts brought in line with the root-free convention sign"
description: "Seven kb concepts that still described Diagram::anchor, the anchor reference tree or the removed role-sign functions now describe convention_sign as the code has it, and link the convention concepts."
status: draft
tags: [kb, sign-convention, rooting, documentation, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-16}
sources:
  - {id: 12053c4, resource: "https://github.com/nsmith-/vibegraph/commit/12053c4", title: "kb: bring seven concepts in line with the root-free convention sign"}
---
Documentation only; no code touched, `history/` untouched. Every claim was
checked against `helas/eval/root_diagram.rs` and `root_lorentz.rs` at the branch
tip (`convention_sign`, `propagator_phase_sign`, `vertex_root_signs`,
`triple_gluon_sign`, `live_reversed_parity`, `RootedTerm::build_sign`). All seven
concepts are `status: draft`; none gained a `verified:` field.

## Per concept

All paths under `research/kb/`.

- `amplitudes/evaluator-architecture.md`, "root" paragraph (~:74). Now: the
  diagram's `fermi_sign` is `Diagram::sign × convention_sign` read from the graph
  alone, the live tree's reversed parity is cancelled, no second tree is built;
  links the inventory.
- `amplitudes/four-fermion-vertices.md`, "The tensor-path line sign" section and
  the `code-current-line` source and footnote. Now: a cyclic tensor-path vertex
  takes `RootedTerm::build_sign` (−1 per crossed line of the structure) through
  `vertex_root_signs`, read from the vertex, not a rooting; the MadGraph table is
  kept as the adjudication of that sign; the `vg_c4q` defect is attributed to
  `triple_gluon_sign`. `fermion_current_line_sign` and "anchor rooting" are gone.
- `amplitudes/fermion-flow-and-crossing.md`, the standalone `P_χ`/`1`/`γ⁵` row
  and the paragraph after the table. Now: no flip and no reversed parity; a
  crossed line's reordering −1 is the line's (`Diagram::fermion_line_sign`,
  `propagator_phase_sign`), not `pair_crossed`/`standalone_projector_crossed`
  (both deleted); the rooting-dependent parity is cancelled and replaced by the
  graph-read `vertex_root_signs`.
- `amplitudes/global-phase-i-counting.md`, the Yang–Mills VVV bullet. Now: no
  colourless VVV sign; the only vector-vertex sign is `triple_gluon_sign`.
- `amplitudes/charge-flow-phase-study.md`, the W1 witness-pair paragraph and
  the `channel_counts` bullet. Now: the gluon contact and exchange diagrams carry
  the same net sign today; the arrow-free arms W1 named do not exist; the
  arrow-free factors that remain are the scalar propagator phase and the
  triple-gluon −1. The study's own results table is left as a record of its date,
  as its text already says.
- `validation/bit-exact-amplitude-debugging.md`, the GF(2) section opening and
  closing paragraph, and the `channel_counts` bullet. Now: the worked steps stay
  as the record of how the earlier anchor-keyed rule was found; once, in two
  places, the section says that rule has been replaced by `convention_sign`, and
  links the inventory, vector-vertex-signs and rooting-invariance.
- `performance/diagram-rooting.md`, description, "Why it is sound" and the
  orientation-dependence sentence. Now: no sign depends on the root;
  `fermi_sign` is graph-read with the live parity cancelled; the sweep keeps it so.

## Gate

```
$ pixi run -e kb kb-index
wrote 2 index files
$ pixi run -e kb kb-lint
604 concepts, 202 backlog items; 0 errors, 2 warnings
   (two blocked backlog items without blocked_by, not this session's)
$ grep -rn 'Diagram::anchor\|yang_mills_vvv_sign\|vector_contact_sign\|gluon_scalar_current_sign\|fermion_current_line_sign' research/kb --include=*.md | grep -v 'history/\|sprints/'
(no output)
```

## Brief corrections

None needed; line numbers were close. `pair_crossed` and
`standalone_projector_crossed` (fermion-flow-and-crossing) and `channel_counts`
(two concepts) were not in the brief's grep but were removed or stale and were
fixed in the same passages.

## Found

- `amplitudes/charge-flow-phase-study.md` results table (:78-90) and rule
  section still list the removed arms (contact −1, VVV source, scalar sink) in
  their dated form; left as the study's record, but a reader skimming the table
  sees "yes" cells for rules the code no longer has. To close: a one-line note
  above the table, or a current-status column.
- `performance/diagram-rooting.md` carries a `verified:` stamp from 2026-10-09
  predating this edit; the manager should decide whether to restamp.
- The B-H report's Found 2 listed seven concepts; all are now done. Found 3-8
  there (backlog items, `pub(super)` narrowing) are untouched.
