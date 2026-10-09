---
type: Design Decision
title: Parse everything, check once, narrow the type
description: "ProcCardAst mirrors ProcessDefinition; check_supported reports every unsupported feature and returns a narrower SupportedCard; squared-order constraints are a hard refusal, never dropped."
status: draft
tags: [process-grammar, check, refusal, squared-orders, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-design, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L199-L253", title: "Note 38 §3.1–§3.2, parse everything, check once; room for MLM and NLO"}
  - {id: n38-g1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L310-L361", title: "Note 38 §4 G1, grammar, AST and the one check"}
  - {id: n38-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1144-L1201", title: "Note 38 §5, decisions (user, 2026-09-25/26)"}
  - {id: n35-c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L788-L841", title: "Note 35 §4 C, the silently dropped NP^2==1"}
  - {id: n35-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1191-L1218", title: "Note 35 §7 D4, squared-order constraints kept out of the SMEFT rows"}
  - {id: code-check, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/check.rs", title: "vibegraph-lib/src/diagrams/check.rs"}
decided: 2026-09-25
decided_by: human:nsmith-
---

**Decision (user, 2026-09-25):** the proc-card parser is full-featured, and **one check**
refuses whatever this generator does not support. [^n38-decisions] The release scope
this serves is [release scope](../pipeline/release-scope.md) (the reviewed decision is
[release-scope-lo-mlm](../decisions/release-scope-lo-mlm.md)); this concept is the
mechanism, not the scope.

## The shape

- **`ProcCardAst`** (`diagrams/parse.rs`) is MadGraph's `ProcessDefinition` with nothing
  dropped: legs as id-sets with their polarization and photon-tag flag; required
  s-channels as an or-of-and list; forbidden particles, forbidden s-channels and forbidden
  on-shell s-channels; amplitude orders with their operators; squared orders; the loop
  spec (option and perturbation orders); `@N` and overall orders; decay chains as child
  definitions; and the command sequence (`import model`, `define` including `|`,
  `generate`/`add process` with their reset semantics, `set` with its arguments, the
  `launch` dialogue). [^n38-design] The grammar it parses is in
  [proc-card grammar](proc-card-grammar.md).
- **`check_supported(&ProcCardAst) -> Result<SupportedCard, UnsupportedCard>`**
  (`diagrams/check.rs`) is the one scan. It reports **every** unsupported feature of the
  card at once (`UnsupportedCard(Vec<Unsupported>)`), not the first.
- **`SupportedCard` is a narrower type.** It has no field for a refused feature, so code
  downstream cannot read a construct the check refused: there is nowhere to hold it.
  Supporting a feature means giving it a field in `SupportedCard` and removing its
  `Unsupported` variant **in the same change as the code that honours it and its gate**.
  [^code-check]
- **The `Unsupported` enum is the feature backlog against MadGraph.** Each variant carries
  its reason, and the module documentation's table lists what would lift each. It replaces
  scattered "descoped" lists.

What the check cannot decide without a model is refused where the model is, during
resolution and enumeration: names that resolve to no particle (`ResolveError`), order
names the model does not define, and the same subprocess reached from two process lines
(`DiagramError::DuplicateSubprocess`).

The run card has had the same pattern longer: every recognised name in
`runcard/classes.rs` is `Consumed` (with its consumer named), `IgnoredBenign` (with an
argument that it cannot reach σ, the record or the cuts) or `IgnoredPhysics` (refused
whenever it could bite). A parameter MadGraph acts on that this crate silently dropped
would be a wrong answer, so it is refused at parse time instead.

## The current refusals

| Variant | Card feature | Status |
|---|---|---|
| `SquaredOrder` | `QCD^2<=4`, `NP^2==1`, `aEW`, `aS` | in scope, not yet supported ([backlog](../backlog/feature/squared-order-constraints-refused.md)) |
| `WeightedOrder` | `WEIGHTED==n`, `WEIGHTED>n` | needs the same amplitude split as squared orders (same item) |
| `ChainOrders` | an overall chain order that a part also bounds with `==` or `>` | [backlog](../backlog/feature/chain-orders-exact-lower-bound-refused.md) |
| `DecayedPolarization` | `p p > w+{0} w-, w+ > e+ ve`; `t{L} > w+ b` | [backlog](../backlog/feature/polarized-intermediate-resonances-refused.md) |
| `PropagatorPolarization` | `{A}`, `{G}`, `{H}`, `{Q}`, `{W}`, `{S}` | same item |
| `DecayOnShellVeto` | `$` inside a decay | [backlog](../backlog/feature/onshell-veto-on-decay-refused.md) |
| `LoopSpec`, `PhotonTag` | `[QCD]`, `[real=QCD]`, `!a!` | NLO ([backlog](../backlog/feature/nlo-generation-missing.md)) |
| `ProcessOption` | `--diagram_filter`, `--optimize`, … (not `--no_warning=duplicate`) | not planned |
| `SetOption` | a physics-bearing `set` off its default | per option |
| `LaunchDialogue` | run- or param-card edits after `launch` | not planned: the cards are their own files |
| `ModelOption`, `Command` | `import model X -modelname`, `add model`, other commands | not planned |
| `InitialState`, `MixedInitialStates` | more than two initial particles; lines with different initial counts | MadGraph errors too |

The module table in `check.rs` still marks `SquaredOrder` and `WeightedOrder` "not
planned"; squared-order constraints were put in scope by the user on 2026-10-09, and the
change that closes the backlog item updates that line. Card-level refusals that need the
model live in `DiagramError` (for example `MixedOnShellVeto`,
[backlog](../backlog/feature/onshell-veto-lists-differing-per-line-refused.md)).

## Squared-order constraints are refused, never dropped

A squared-order constraint bounds the order of an **interference term** in |M|², a
statement about pairs of diagrams. This generator selects diagrams by their own orders and
squares the whole amplitude, so it has no way to honour one. The refusal is
`Unsupported::SquaredOrder` (`check.rs`), with a second refusal in decay-chain resolution
(`resolve.rs`, `ResolveError::DecayConstraint`), pinned by
`a_squared_order_constraint_is_a_hard_error` (`vibegraph-lib/tests/diagrams.rs`). The
message suggests the amplitude-level order (`QED<=n`) instead.

Why a hard error matters: a squared-order constraint that is parsed and **silently
dropped** gives a confident wrong answer. With `NP^2==1` ignored, the SMEFTsim CLI path
produces 34 channels and σ = 3.6686 pb, a different process from the interference-only
one asked for. [^n35-c] Every SMEFTsim row therefore compares the full
|M|² at `NP<=1` ([coupling orders](../model/coupling-orders.md)). [^n35-decisions]

Supporting them means extracting the complex amplitudes grouped by coupling order and
forming only the cross terms the constraint selects. That is the same split reweighting
in a coupling needs (`reweight::poly` already proves each diagram's coupling monomials),
and it is a refactor of `helas/eval`. [^n38-decisions]

## Room for what is not built

Constructs not yet supported are parsed and refused, not parse errors, so supporting one
extends a type instead of re-threading the pipeline: the loop spec, perturbation orders,
squared and split orders, photon tags and polarizations are all in the AST. [^n38-design]

- **Several processes, several multiplicities.** `SupportedCard` holds a list of
  processes, each with its own legs and `@N`. Lines of different final-state
  multiplicities are accepted and summed, with matching left to the run card (`ickkw`,
  `xqcut`, resolved in `runcard/matching.rs`; MLM is described under
  [release scope](../pipeline/release-scope.md)).
- **Provenance on every diagram.** Each `Diagram` records the `@N` and decay-chain node it
  came from, and each propagator its MadGraph `onshell` flag (free, forced, forbidden):
  see [decay chains](decay-chains.md) and [s-channel restrictions](s-channel-restrictions.md).
- **Helicity Monte Carlo.** `nhel = 1` is a run-card refusal (`IgnoredPhysics`: it changes
  both the estimator and the per-event weight); it is in scope and not yet supported
  ([backlog](../backlog/feature/nhel1-run-cards-refused.md)).

[^n38-design]: Note 38 §3.1–§3.2: the AST, the one scan, the narrow type, the `Unsupported` table, room for MLM and NLO (its run-card line on `ickkw`/`xqcut` predates MLM).
[^n38-decisions]: Note 38 §5: the full-featured parser and one check (2026-09-25), squared orders refused and why.
[^n35-c]: Note 35 §4 C: `NP^2==1` silently dropped, then refused.
[^n35-decisions]: Note 35 §7 D4: every SMEFT row compares the full |M|² at `NP<=1`.
[^code-check]: `vibegraph-lib/src/diagrams/check.rs` module documentation.
