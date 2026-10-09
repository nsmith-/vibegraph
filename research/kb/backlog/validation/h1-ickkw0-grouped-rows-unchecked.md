---
type: Backlog Item
title: Banked ickkw = 0 rows unchecked for the permuted first-call bias
description: Grouped MadEvent references with non-identity symmetry permutations carry H1's first-call bias even unmatched; no banked ickkw = 0 row has been checked.
area: validation
state: open
priority: high
closes_when: Every banked grouped MadEvent row using a clustering scale (dynamical_scale_choice = -1 or matching) is checked for non-identity permutations, and each affected row is re-measured against an unbiased reference or carries H1 as a registered deviation.
blocked_by: []
opened: 2026-09-29
tags: [h1, madevent-reference, scales, grouping]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L220-L228", title: "TODO.md entry T004"}
---
H1 is a registered deviation (user, 2026-09-29): grouped MadEvent's first
`setclscales` call clusters the unpermuted `PP`
(`super_auto_dsig_group_v4.inc:805,842`); this crate clusters `P1`, the
momenta the matrix element reads. Measured: +1.8 % of `P2_qq_llqq`, about
+0.25 % of `pp_to_ll_0j2j_mlm`'s `@2`, mostly through first-call rejections.

R1 shows the bias without matching. On `generate u q > z u q` (13 TeV,
`define q = u d`) with the default card (`ickkw = 0`,
`dynamical_scale_choice = -1`), grouped MadEvent reads 104.28 pb against 108.60
non-grouped (−4.0 %, −7.6σ). With the one-line fix it reads 108.62 (0.0σ).
Every grouped MadEvent reference with `dynamical_scale_choice = -1` and
non-identity permutations is therefore biased, matched or not.

To do: list the banked rows whose grouped subprocess directories carry
non-identity permutations in their symmetry configs (`validation/manifest.toml`
plus the process directories under `validation/madgraph/output`). Then re-run
any affected row's reference with
`validation/madgraph/patches/first-call-unpermuted-momenta.patch` or with
`group_subprocesses False`. The vectorised path (`update_scale_coupling_vec`)
has the same shape and is unmeasured. H1's share on `pp_to_ttx_0j1j_mlm` is
covered by [ttx-mlm-at1-sigma-high](ttx-mlm-at1-sigma-high.md).

Detail: [note 41 D2 and R1](../../history/notes/41-mlm-feature-sprint-plan.md); the upstream draft is in
[note 07](../../history/notes/07-mg5-code-quality.md) ("`super_auto_dsig_group_v4.inc` — Direct
Bug Found"); reproducer `validation/madgraph/repro/permuted_first_call/`.
