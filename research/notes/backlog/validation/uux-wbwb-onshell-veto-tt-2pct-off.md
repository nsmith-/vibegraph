---
type: Backlog Item
title: u u~ > w+ b w- b~ $ t t~ sits about 2% above patched MadEvent
description: With MadGraph's FFV2P1D_1 defect patched, MadEvent reads ~2% below this side on the $ t t~ row and still moves with budget; the row stays info.
area: validation
state: open
priority: high
closes_when: The ~2% gap between this side and the patched MadEvent on the $ t t~ row is attributed, and the row is either gated or documented as a reference-side defect with a measured cause.
blocked_by: []
opened: 2026-09-26
tags: [process-grammar, onshell-veto, madgraph-defect, info-cell, top]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L289-L297", title: "TODO.md entry T013"}
---
On `u u~ > w+ b w- b~ $ t t~` at √s = 500, fixed μ = 173, MadEvent with
the `FFV2P1D_1` defect patched (`validation/madgraph/patches/aloha-p1d-flipped-fermion.patch`;
the defect is in [note 07](../../07-mg5-code-quality.md)'s appendix) reads
0.04942–0.04982 pb on four 10k-event seeds and 0.04934 ± 0.00007 at 50k. This
side reads 0.05059–0.05080 (0.050695 ± 0.000024 at a 5e-4 target), and does
not move with budget or under `--map-split-angle isotropic` (0.050724 ±
0.000049). MadEvent drifts down with budget; this side does not.

Ruled out: the pointwise amplitude (the patched standalone `SMATRIX` equals
the zeroed amplitudes here to 1e-12 at six points covering both, one, the
other and no top in the window) and the unrestricted process (fixed-μ rows
agree, 15.060 against 15.052).

The row is reported, not gated (`info`). Next: run the seeded patched
reference (`validation/madgraph/gen_onshell_veto.sh`, row `uu_tt`) at two
budgets, then decompose σ by window (both tops outside, one inside) on both
sides. Detail: [note 38 §4 S3](../../38-process-grammar-sprint-plan.md).
