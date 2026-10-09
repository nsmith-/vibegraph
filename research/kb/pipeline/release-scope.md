---
type: Design
title: "Release scope and the hard-error rule"
description: "How the release scope is enforced: every card surface vibegraph does not honour is refused at parse time, never silently accepted; where each refusal lives and how a feature leaves the list."
status: draft
tags: [scope, hard-errors, process-grammar, run-card]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-intro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L11-L42", title: "Note 38: process-grammar sprint scope"}
  - {id: n38-check, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L199-L253", title: "Note 38 §3.1–3.2: parse everything, check once; room for MLM and NLO"}
  - {id: n38-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1144-L1201", title: "Note 38 §5: decisions (2026-09-25/26)"}
  - {id: dec-parity, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/decisions/release-scope-mg-lo-parity.md#L13-L30", title: "Decision: release scope is MadGraph LO process parity (deprecated)"}
  - {id: dec-sm, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/decisions/release-scope-sm-fixed-order.md#L14-L25", title: "Decision: fixed-order SM scope (deprecated; source of the hard-error rule)"}
  - {id: n29-c1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1225-L1233", title: "Note 29 chain C1: hard errors at each parser boundary"}
  - {id: n29-rulings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L6126-L6136", title: "Note 29 close-out: manager rulings (tmin_for_channel stays refused)"}
  - {id: check-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/check.rs#L1-L41", title: "check.rs: the one check and the Unsupported table"}
  - {id: classes-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/runcard/classes.rs#L1-L60", title: "runcard/classes.rs: field classification"}
---
**What is in the release** is the reviewed decision
[release-scope-lo-mlm](../decisions/release-scope-lo-mlm.md): leading-order
generation at arbitrary leg multiplicity with MLM merging, including the full
proc-card grammar, decay chains, 1→n decays, the s-channel restrictions,
polarized legs and `add process`. Beam polarization, beams other than
unpolarized proton–proton or fixed-energy partonic, CKKW-L and NLO stay in the
backlog. This concept covers the rest: how that boundary is enforced, and how
a feature crosses it. The pipeline it guards is in [the overview](overview.md).

## The hard-error rule

Every surface a card can reach that vibegraph does not honour is a **hard
error at parse time, never a silent acceptance**.[^dec-sm] A parameter MadGraph
acts on and vibegraph silently drops gives a wrong answer with nothing to show
for it, so it is refused. The refusals are unconditional: there is no
`ParsingOptions`-style override, following the precedent of `UnsupportedLpp`.[^n29-c1]

A refusal names the feature and the reason, and reports every problem in the
card at once rather than the first.

| Input | Where it is refused | How |
|---|---|---|
| Proc card | `diagrams::check::check_supported` | The card is parsed in full into `ProcCardAst` (MadGraph's `ProcessDefinition`, nothing dropped). One pass returns `Result<SupportedCard, Vec<Unsupported>>`. |
| Proc card, model-dependent | Enumeration | Names that resolve to no particle, order names the model does not define, and one subprocess reached from two process lines. |
| Run card | `RunCard::from_values` | Only `(lpp1, lpp2)` = `(1, 1)` or `(0, 0)` pass (`UnsupportedLpp`). Then every recognised field is classified, and an `IgnoredPhysics` field moved off its MadGraph default is `UnsupportedField`. |
| Run-card cuts | `cuts::detect_unimplemented` | A cut parsed but not applied is refused when it deviates from the default (`Consumed(R_UNIMPL)`, e.g. `ktdurham`, `ptlund`). |
| Scale and matching settings | `coupling::scales` | `ickkw` outside {0, 1} (`UnsupportedMatching`); matching or `xqcut` on fixed beams (`FixedBeamMatching`). |
| UFO model | Diagram conversion | A custom `propagators.py` form (`ConvertError::CustomPropagator`). |

`SupportedCard` is a **narrower type**: it has no field for a refused feature,
so code downstream cannot read a `$` on a decay or a squared-order constraint,
because the type has nowhere to hold one.[^check-rs] The run card follows the same
pattern through its classification table
([run-card field classification](../run-card/field-classification.md)): each
name is `Consumed`, `IgnoredBenign` (with a written argument for why it cannot
reach σ, the event record or the cuts) or `IgnoredPhysics`.[^classes-rs] The proc
card's side is detailed in [the supported-card check](../process/supported-card-check.md).

`IgnoredPhysics` means "not implemented", not "not read". A field may be read
precisely to decline the branch it selects, and the refusal then covers that
branch. `tmin_for_channel` is the standing case, and it stays refused. Off its
default it multiplies MadGraph's `get_channel_cut` by an exponential
suppression of spacelike lines below `t/s_tot = tmin`. The channel weight
vibegraph builds is the `SDE_strategy = 2` branch, which forms no such
suppression.[^n29-rulings]

## The `Unsupported` enum is the feature backlog against MadGraph

Each variant of `Unsupported` carries the MadGraph feature and the work that
would lift it. Its doc table in `check.rs` replaces scattered "descoped" lists.
Variants today include `SquaredOrder`, `WeightedOrder` (`==`/`>`), `LoopSpec`
and `PhotonTag` (NLO), `DecayedPolarization`, `PropagatorPolarization`,
`DecayOnShellVeto`, `ChainOrders`, `ProcessOption`, `SetOption`,
`LaunchDialogue`, `ModelOption` and `Command`. Two more, `MixedInitialStates` and
`InitialState`, report MadGraph errors rather than missing features.

**Lifting a refusal** means three things in the same change:

1. Give the feature a field in `SupportedCard`.
2. Remove its `Unsupported` variant.
3. Add the gate that validates it against MadGraph.

The data structures are built not to block MLM and NLO. Loop specs, squared
orders and photon tags are parsed and refused rather than dropped, and
`SupportedCard` holds a list of processes, each with its own legs and `@N`. A
single final-state multiplicity is never assumed at the type level.[^n38-check]

Some refusals are in scope and open as backlog items. Squared-order
constraints are one
([squared-order-constraints-refused](../backlog/feature/squared-order-constraints-refused.md)),
and the `check.rs` table still marks them "not planned"; the PR that lifts the
refusal updates that line. NLO is out of scope
([beyond leading order](beyond-leading-order.md)).

## Deviating from MadGraph

Within scope the goal is parity with MadGraph. Where MadGraph is wrong or
makes an arbitrary choice, there are three outcomes: reproduce it, refuse the
input, or register a documented deviation. They are defined in
[the MadGraph defect policy](../validation/madgraph-defect-policy.md).

The standing registered deviation in process generation is **identical
particles across decays**. MadGraph keeps one pairing and divides by
`identical_decay_chain_factor`, dropping the interference between pairings.
vibegraph keeps every pairing, with the interference and the final state's own
identical-particle factor ([identical particles across decays](../process/identical-particles-across-decays.md)).
So a decay-chain σ with identical particles across decays is not expected to
equal MadGraph's: it reads about +0.2% above on such cards, and only cards
without that overlap gate σ.[^n38-dec] The MadGraph-only study behind that
figure lived outside the repository and is not recoverable. Note 38 §5 is the
record of its numbers.

## Where the scope came from

The fixed-order SM scope (2026-08-02) was widened to MadGraph LO process
parity (2026-09-25),[^dec-parity] and then to arbitrary-multiplicity LO with MLM
(2026-10-09). The hard-error rule is the one part that has stayed unchanged
throughout. The two earlier decisions are deprecated.

[^dec-sm]: Decision `release-scope-sm-fixed-order` (deprecated), "the rule that outlives this decision".
[^dec-parity]: Decision `release-scope-mg-lo-parity` (deprecated).
[^n29-c1]: Note 29, Chain C1 design: one guard per parser boundary, no override.
[^n29-rulings]: Note 29 close-out, manager rulings.
[^check-rs]: `vibegraph-lib/src/diagrams/check.rs` module doc.
[^classes-rs]: `vibegraph-lib/src/runcard/classes.rs` module doc and `refuse_ignored_physics`.
[^n38-check]: Note 38 §3.1–3.2.
[^n38-dec]: Note 38 §5, decision of 2026-09-26 and its MadGraph-only measurement.
