---
type: Session Report
title: "B-H report: hygiene review of the root-free convention sign, and the concepts it supersedes"
description: "Hygiene review of B′'s convention_sign change on PR #30: findings written before any edit, then the local ones fixed under a per-diagram digest identity check, and the five kb concepts that described the anchor convention rewritten around convention_sign."
status: draft
tags: [hygiene, review, sign-convention, rooting, relabelling, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources: []
---
Review of B′'s code commits on `amp-signs/wpwm-epem-neutrino-sign` at
`5d4a346`: `6f2df1e` (the relabelling test) and `f1bb5bb` (the convention
sign), that is `git diff d6f2d41 5d4a346 -- vibegraph-lib validation/madgraph`.
Line numbers are at `5d4a346`.

## Findings (written before any edit)

| id | pt | site | evidence | mutation the test misses | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 1, 4 | `root_diagram.rs:1018` (`external_bindings`) against `:405–440` (`bake_node`'s leg arm) and `:690–810` (`collect_fermion_pairs`, `mixed_line_final_legs`) | The convention sign binds each external spinor with its own copy of the live tree's leg rule (`charge.anti()` when uncrossed, `crossed: !incoming && !uncrossed`, `ExtLegInfo::adjoint`), and finds the uncrossed legs by a second algorithm: `external_bindings` walks `Diagram::fermion_lines`, the live tree walks the raw rooted tree (`collect_fermion_pairs`). Both compute "the final leg of each line with an initial-state end", a graph property. `vertex_root_signs` reads one, the kernels' bindings the other; a change to either rule alone makes the convention sign disagree with the live tree's binding with nothing to say so | — | checked | fix: one `mixed_line_final_legs(diagram, model)` from `fermion_lines`, one leg-binding function, both used by the live tree and by the convention sign; the raw-tree walk goes |
| H2 | 1 | `root_diagram.rs:1054` (`line_end`) against `diagram.rs:481` (`Diagram::fermion_lines`) | Both follow a fermion line through each vertex's flow-group partner. `line_end` starts at a `(vertex, slot)` and returns the end leg; `fermion_lines` starts at each leg and returns ends and vertices, without slots | — | checked | leave: merging needs a slot-recording walk that no other reader wants; the two share only the three-line step |
| H3 | 1 | `relabelling.rs:229` (`diagram_values`) | A second copy of `renumbering::single_diagram_amplitudes` (`renumbering.rs:233`): build a one-diagram `DiagramSet`, compile, bind, evaluate per helicity. It differs only in taking the helicity list (the permuted one) instead of the evaluator's own. It is also an eighth site of the filed `one-diagram-diagramset-built-by-hand`, which names seven | — | checked | fix: one helper, which takes the helicities |
| H4 | 1 | `relabelling.rs:176` (`ray_leg_sets`) | Re-walks the tree to find the external legs behind each ray. `Diagram::legs_on_start_side` (`diagram.rs:582`, private) computes exactly that per propagator | — | checked | fix: use it |
| H5 | 1 | `relabelling.rs:150` (`load`) | Re-implements `renumbering::load` (`renumbering.rs:101`): a per-(model, card) cache and the restrict-card path. Its `Model` enum is the `(dir, card)` pair the other takes | — | checked | fix: use `renumbering::load` |
| H6 | 2 | `relabelling.rs:258–345` | `compared` is printed, never asserted. A pair with at most one compared diagram cannot show a relative flip and passes whatever the signs: no green-on-empty guard | `VANISHING = 1.0` (only the largest diagram compared) passes | checked (run below) | fix: require at least two compared diagrams per pair |
| H7 | 2 | `relabelling.rs` | Does the test still catch a leg-keyed sign on the fixed tree? B′'s "before" run is evidence on the old convention only | a `−1` keyed to the vertex holding leg 0 | suspected: settle by running it | — (run below) |
| H8 | 2 | `root_diagram.rs:1338` (`named_processes_exercise_every_convention_factor`) | It calls each factor function directly, so it is blind to `convention_sign` dropping a factor from its product (that is the MadGraph rows' to see, per B′'s mutation table). Its own job is that each factor fires: to be shown by mutation of each factor's parts | each part of each factor set to `+1` | suspected: settle by running them | — (run below) |
| H9 | 1 | `root_diagram.rs:944` | `DiagramEval::fermi_sign` is documented "±1 from the diagram's Fermi permutation sign"; it is `Diagram::sign × convention_sign × live reversed parity` (`:1213`) | — | checked | fix: doc |
| H10 | 1, 3 | `root_diagram.rs:563` (`DiagramEvalTree::reversed_convention_sign`) | The name says convention sign; the value is the live tree's runtime reversed-bilinear parity, which `compile_single_diagram` multiplies in to *cancel*, while the convention's own parity comes from `vertex_root_signs`. `pub(super)`, and its only caller is `compile_single_diagram` in the same module (`git grep -n reversed_convention_sign` outside `root_diagram.rs` finds only doc links) | — | checked | fix: rename to `live_reversed_parity`, narrow to private, update the two doc links |
| H11 | 1 | `rooting_soundness.rs:5`, `:358` | "was originally validated only for feyngraph's `VtxIdx(0)` orientation" is history framing; "without the slow, currently-failing full sweep" is false: B′'s run of the sweep reads `270 re-rootings, 0 failures` | — | checked | fix: both comments |
| H12 | 2 | `rooting_soundness` (the brief's question) | What does it still pin? Claim (B′, and the module doc): the kernels' rooting invariance *and* the live-parity cancellation in `fermi_sign`. Settle with `fermi_sign` missing `* tree.reversed_convention_sign()` against the default-suite tests and the ignored sweep | dropping the live-parity cancellation | suspected: settle by running it | — (run below) |
| H13 | 4 | `root_lorentz.rs:415–486` (C-H's H13) | `ProjM` and `ProjP` arms of `build_child` are the same 15 lines with the chiralities swapped; those two, `Identity`, `Gamma5` and `Metric` each spell the same "the other index of a two-index op, or `unreachable!`" pick | — | checked | fix: one `partner_index` helper and one projector arm; identity by digest |
| H14 | 1 | `root_lorentz.rs:340` (`build_child`) | About 215 lines | — | checked | leave: one arm per `LorentzOp` variant, the reason the dispatch is one match; H13 shortens it |
| H15 | 3 | `root_diagram.rs:650` (`spine_sign_from_flow`), `:909` (`root_tree_at`) | `pub(super)`, no caller outside `root_diagram.rs` (`git grep` at `d6f2d41` and `5d4a346`). Not touched by the change | — | checked | Found (outside the change) |
| H16 | 1 | `gen_standalone_jamps.py:226` | A 104-character comment line from the change's rewrap; every other line of the file is ≤ 100 | — | checked | fix |
| H17 | 1 | `standalone_jamps.rs` row docs, `gen_standalone_jamps.py` row comments | They now describe rows by "holding leg 0", a property no convention reads any more | — | checked | leave: true as written, and it says why each row discriminates a labelling-keyed rule, the class the relabelling test guards |
| H18 | 2 | `amplitude_oracle.rs:435` (`CONFIG_AMP_SIGNS`) | 11 patterns re-banked as whole-pattern flips. The banked patterns now carry the free per-process global sign | — | checked | leave: a two-way tripwire is its stated purpose (B′'s "per-process global sign is free") |
| H19 | — | `SEVEN_DIGIT_COUPLING_ROWS` (`standalone_jamps.rs:268`) | The brief: replaced by a per-graph exemption on PR #33 | — | — | leave (per the brief) |

Comment grep over the added lines
(`git diff d6f2d41 5d4a346 -- vibegraph-lib validation/madgraph/gen_standalone_jamps.py | grep -nE '^\+.*(\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer|originally|anchor|W1|F1|session|sprint|previously|formerly|old )'`)
returns only the four "leg 0" row comments of H17. `git diff … | grep -nE '^\+.*\bpub(\(|\s)'`
is empty: the change adds no `pub` item.

Backlog check (path tails `/root_diagram.rs`, `/root_lorentz.rs`,
`/rooting_soundness.rs`, `/relabelling.rs`, `/renumbering.rs`,
`/diagram_eval.rs`, `/standalone_jamps.rs`, `/diagram.rs`, and the module
names):

- `one-diagram-diagramset-built-by-hand`: H3's site is an eighth it does not
  name; H3's fix removes it again.
- `comment-leftovers-after-hygiene-sprint`: its `root_diagram.rs` site, "note 16
  §6" (F-A Found 4), was in `mg_guard_processes_exercise_every_convention_channel`,
  which the change deleted. The code shows that site fixed.
- `unit-signs-flip-between-f64-and-i8`: `build_at_leg`'s tree path now returns a
  constant `1.0`; the `f64` sign survives on the tensor path
  (`build_tensor_term`) and its `if sign < 0.0` conversion in `root_term`. Still
  open, one site smaller.
- `mixed-dirac-and-matrix-free-vertex-terms`: its body says "An `Identity` term
  takes the scalar-sink −1 and a `Gamma` term does not, so a mixed vertex would
  most likely panic". The scalar-sink build sign is gone; a mixed vertex now
  fails `uniform_sign` on its reversed-bilinear parity only where a bilinear is
  reversed (B′'s H11 account), and the cited lines `diagram_eval.rs:131-169`,
  `:265` have moved. Stale, to Found.
- `tensor-uniformity-asserts-panic`: unchanged; `vertex_root_signs` reaches the
  same `VertexInfo::tensor` assert at every vertex, as the live tree did.
- `weighted-order-computed-three-ways`, `majorana-fermions-unsupported`,
  `diagram-enumeration-cost-unmeasured`: name `stitching.rs` or `diagram.rs`
  for other reasons; unaffected.
