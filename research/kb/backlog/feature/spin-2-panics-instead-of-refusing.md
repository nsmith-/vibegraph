---
type: Backlog Item
title: A spin-2 particle panics instead of being refused or evaluated
description: "A UFO spin-2 tensor index (1000+k, 2000+k) is lowered as a leg number and indexes out of bounds in lower_lorentz, so any graviton crashes compile; refuse it or support spin 2."
area: feature
state: open
priority: medium
closes_when: "Either (a) a process with an internal spin-2 propagator and one with an external spin-2 leg are each refused by the UFO load or AmplitudeEvaluator::compile with an error naming the spin-2 particle or Lorentz structure, pinned by regression tests on a spin-2 toy model, and no Lorentz index outside a vertex's legs can reach lower_lorentz as a Leg; or (b) spin-2 support lands: both processes evaluate and match MadGraph per helicity on a banked row, which closes the spin-2 half of spin-2-and-spin-3-2-particles-unsupported with this item."
blocked_by: []
opened: 2026-10-26
tags: [spin, refusal, non-sm-ufo, panic, helas]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71", title: "PR #71: CMS-gridpack survey, BulkGravitonTo2W_Inclusive_narrow_Par-M{2500,4000}GeV panic report"}
  - {id: lower, resource: "../../../../vibegraph-lib/src/helas/eval/lower.rs", title: "helas/eval/lower.rs lower_lorentz, the L::Leg(i) => inputs[i] arm (:646)"}
  - {id: root-lorentz, resource: "../../../../vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "helas/eval/root_lorentz.rs LorentzEvalTree::build_child scalar-leaf branch (~:365) and compact_legs (~:746)"}
  - {id: lorentz, resource: "../../../../vibegraph-lib/src/ufo/lorentz.rs", title: "ufo/lorentz.rs to_isize (~:342)"}
  - {id: particles, resource: "../../../../vibegraph-lib/src/ufo/particles.rs", title: "ufo/particles.rs helicity_states (~:91-101)"}
  - {id: run, resource: "../../../../vibegraph-lib/src/helas/eval/run.rs", title: "helas/eval/run.rs build_external_core's panic arm (~:1948)"}
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
the result. So the printed index is the UFO index minus one, or minus two.

**Nothing refuses spin 2.** `Particle::helicity_states` lists five helicities
for spin code 5, so `AmplitudeEvaluator::compile`, which refuses a spin only
when that list is `None` (as it is for spin 3/2), lets spin 2 through. That
check covers external legs only in any case; nothing looks at spin on an
internal line.

**Reproducer** (scratch). The toy UFO plus a spin-5 `gt` (1 TeV) with `lt~ lt gt`
(`P(2003,1)*Gamma(1003,2,1) + ...`) and `vt vt gt` (`Metric(1,1003)*Metric(2003,2) + ...`)
vertices. `lt~ lt > vt vt NP<=2` at fixed 500+500 GeV gives the reported message
exactly (len 2, index 1001, via `lower_vertex` ← `compile`); the external
`lt~ lt > gt` panics at the same site (len 3, index 2002) before
`build_external_core`. Behind it, `build_external_core` (`run.rs` ~:1948) and
`analysis.rs` (~:251) also panic on spin code 5; those arms are in
[lorentz-and-spin-code-paths-panic-instead-of-refusing](../hygiene/lorentz-and-spin-code-paths-panic-instead-of-refusing.md).

**Two ways to close it.**
- **Refuse (a).** Reject a Lorentz index that names no leg, either at UFO load
  or in `build_child` when the scalar-leaf index is `>= n_legs`. Refuse spin
  code 5 on any line with a named reason, by making `helicity_states` return
  `None` for it or by checking internal lines too. The regression tests need a
  spin-2 toy model: an edited copy of the toy UFO, as `tests/toy_models.rs`'
  `edited_toy_model` builds.
- **Support (b).** Add spin-2 support: tensor index handling in the Lorentz
  tree, tensor external wavefunctions, and the spin-2 propagator. Validate it
  against MadGraph per helicity, internal and external; this also closes the spin-2 half of
  [spin-2-and-spin-3-2-particles-unsupported](spin-2-and-spin-3-2-particles-unsupported.md).
  #71's bulk-graviton gridpacks are the motivating sample.
