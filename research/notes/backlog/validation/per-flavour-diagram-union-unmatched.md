---
type: Backlog Item
title: Diagram gates compare a summed count, not MadGraph's per-flavour subprocess union
description: diagrams.json holds only summed NGRAPHS per process, so multi-channel diagrams cells cannot see whether we enumerate MadGraph's exact set of concrete subprocesses.
area: validation
state: open
priority: medium
closes_when: extract_diagrams.py banks a per-concrete-process {in, out, ngraphs} list, and the diagram gate matches every vibegraph subprocess to it by sorted PDGs, with the subprocess sets equal.
blocked_by: []
opened: 2026-07-19
tags: [diagrams, flavour, madgraph-regen, v7, process-grammar]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L477-L480", title: "TODO.md entry T034"}
  - {id: todo2, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L481-L487", title: "TODO.md entry T035"}
---
`validation/madgraph/diagrams.json` carries each P-class's representative
`NGRAPHS`, summed. `count_mg_style_topologies`
(`vibegraph-lib/tests/validate_madgraph_diagrams.rs:178`) collapses our
subprocesses into coarse classes and compares one representative per class
against that sum. This assumes our first-enumerated subprocess matches
MadGraph's `matrix1` representative.

The manifest's "includes the per-flavour concrete-subprocess union" notes
therefore describe the intent, not the current assertion.
`proc_grammar_oracle.rs` stops at the process definition, so nothing else
covers the union.

Design (V7):
- **Extractor.** Parse the `C     Process:` header lines of each
  `SubProcesses/P*/matrix<N>_orig.f` into `{in, out, ngraphs}`, using a bounded
  name→PDG table. Do not reverse-engineer `leshouche.inc`'s `IDUP` mapping.
- **Rust side.** Key both sides by `(sorted initial PDGs, sorted final PDGs)`
  and compare per subprocess.
- **Validation order.** Validate on `pp_to_ll` before the qq4l class.

Real-finding risk: this exposes whether our multiparticle definitions and
flavour-symmetry pruning produce MadGraph's exact subprocess set. A mismatch
there is a finding needing physics judgement, not a test bug.

Detail: [note 19 §V7](../../19-validation-pass-plan.md).
