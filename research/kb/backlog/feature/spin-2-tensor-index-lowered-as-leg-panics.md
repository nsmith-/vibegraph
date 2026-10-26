---
type: Backlog Item
title: A spin-2 tensor index is lowered as a leg number and panics
description: "A UFO spin-2 tensor index (1000+k, 2000+k) becomes LorentzEvalNode::Leg and indexes the vertex inputs out of bounds in lower_lorentz; an s-channel graviton crashes compile."
area: feature
state: open
priority: medium
closes_when: "A process with an internal spin-2 propagator (and one with an external spin-2 leg) is refused by AmplitudeEvaluator::compile or the UFO load with an error naming the spin-2 particle or Lorentz structure, pinned by a regression test on a spin-2 toy model, and no Lorentz index outside the vertex's legs can reach lower_lorentz as a Leg."
blocked_by: []
opened: 2026-10-26
tags: [spin, refusal, non-sm-ufo, panic, helas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71", title: "PR #71: CMS-gridpack survey, BulkGravitonTo2W_Inclusive_narrow_Par-M{2500,4000}GeV panic report"}
  - {id: lower, resource: "../../../../vibegraph-lib/src/helas/eval/lower.rs", title: "helas/eval/lower.rs lower_lorentz, the L::Leg(i) => inputs[i] arm (:646)"}
  - {id: root-lorentz, resource: "../../../../vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "helas/eval/root_lorentz.rs LorentzEvalTree::build_child scalar-leaf branch (~:365) and compact_legs (~:746)"}
  - {id: lorentz, resource: "../../../../vibegraph-lib/src/ufo/lorentz.rs", title: "ufo/lorentz.rs to_isize (~:342)"}
---
The #71 CMS-gridpack campaign (build `refdata-9-51-g5bc4a19`, `--adapt
gpmodel,strip,runpatch`) panics on `BulkGravitonTo2W_Inclusive_narrow_Par-M{2500,4000}GeV`
with "index out of bounds: the len is 2 but the index is 1001" at
`helas/eval/lower.rs:646`. The site is the `L::Leg(i) => inputs[i]` arm of
`lower_lorentz`, and lower.rs and root_lorentz.rs are unchanged from 5bc4a19 to main.

**Cause (reproduced).** UFO writes a spin-2 leg `k`'s two Lorentz indices as
`1000+k` and `2000+k`. `ufo/lorentz.rs` `to_isize` shifts every positive index
down by one without checking that it names a leg. `LorentzEvalTree::build_child`
then turns any non-negative index that has no operator left into
`LorentzEvalNode::Leg(idx)`. `compact_legs` subtracts one more when the off-shell
leg comes before it, and `lower_lorentz` indexes the vertex's input list with
the result. So the printed index is the UFO index minus one, or minus two. Nothing
upstream refuses spin code 5 on an internal line. The external-leg guard
(`helicity_states` → `EvalError::UnsupportedSpin`) lists five states for
spin 2, and it covers only externals in any case.

**Reproducer** (scratch, not committed). The base is a copy of
`validation/ufo/vibegraph_toy_UFO` plus a spin-5 `gt` (M = 1 TeV), an
`lt~ lt gt` vertex (`P(2003,1)*Gamma(1003,2,1) + ...`) and a `vt vt gt` vertex
(`Metric(1,1003)*Metric(2003,2) + ... - Metric(1,2)*Metric(1003,2003)`).
`vibegraph integrate` on `lt~ lt > vt vt NP<=2` at fixed 500+500 GeV beams
exits 101 with the reported message exactly: `lower.rs:646:22`, len 2, index 1001,
via `lower_vertex` ← `AmplitudeEvaluator::compile` ← `hadronic::compile_class`.
Putting the graviton in slot 2 instead gives index 1000. `lt~ lt > gt` (an external
graviton) panics at the same site with len 3, index 2002, *before*
`build_external_core`. That settles the open question in
[spin-2-external-panics-instead-of-refusing](spin-2-external-panics-instead-of-refusing.md).

**Resolution.** Refuse first: reject a Lorentz index that names no leg, either
at UFO load, or in `build_child` when the scalar-leaf index is `>= n_legs`.
Refuse spin code 5 on any line with a named reason. Real support is
[spin-2-and-spin-3-2-particles-unsupported](spin-2-and-spin-3-2-particles-unsupported.md).
Closing this item likely closes the external-leg item as well. The regression
test needs a spin-2 toy model, an edited copy of the toy UFO as
`tests/toy_models.rs`' `edited_toy_model` builds.
