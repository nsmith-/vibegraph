---
type: Algorithm
title: Decay-chain enumeration by stitching
description: "Core and decays are enumerated separately and glued at forced lines; MadGraph's decay assignment; permutations of identical particles between blocks; container equality as the oracle."
status: draft
tags: [decay-chains, diagrams, process-grammar, madgraph-parity, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n38-decays, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L129-L156", title: "Note 38 §1.3, decays and decay chains in MadGraph"}
  - {id: n38-g1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L310-L361", title: "Note 38 §4 G1, decay assignment measured with MadGraph"}
  - {id: n38-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L599-L688", title: "Note 38 §4 D2, decay-chain enumeration by stitching"}
  - {id: n38-d3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L689-L801", title: "Note 38 §4 D3, decay chains accepted; overall orders"}
  - {id: code-chain, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/diagrams/chain.rs#L1-L40", title: "vibegraph-lib/src/diagrams/chain.rs module documentation"}
  - {id: mg-decay-chain-process, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L5661", title: "MadGraph madgraph_interface.py extract_decay_chain_process"}
  - {id: mg-decay-amp, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L1337", title: "MadGraph diagram_generation.py DecayChainAmplitude"}
  - {id: mg-combine, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/helas_objects.py#L5427", title: "MadGraph helas_objects.py combine_decay_chain_processes"}
  - {id: mg-legs-with-decays, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/base_objects.py#L3667", title: "MadGraph base_objects.py Process.get_legs_with_decays"}
measured:
  - {commit: 337b5c3, pr: 12, landed_in: 1539abc, command: "cargo test -p vibegraph-lib --test decay_chain_census"}
---

A decay-chain card such as `p p > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~` asks for the
diagrams of the full final state in which each named resonance is an s-channel line
decaying to the stated products, with that line held near its mass shell. vibegraph
builds those diagrams by **stitching**: it enumerates the core and each decay on their
own and glues them together, the way MadGraph's `DecayChainAmplitude` does.

## MadGraph's reading

**Syntax** (`extract_decay_chain_process`, `madgraph_interface.py:5661`)
[^mg-decay-chain-process] [^n38-decays]:

- `,` starts a decay at the current level;
- `(` goes down one level; a parenthesised group with no `,` has its parentheses dropped;
- `@N` followed by `ORDER=n` sets overall orders for the whole chain;
- `[...]` and squared-order constraints are refused with decay chains (`do_add`).

**Generation.** `DecayChainAmplitude` (`diagram_generation.py:1337`) generates the core
and each decay as separate amplitudes [^mg-decay-amp]. A decay must have exactly one
initial particle. The core's decaying legs are flagged `onshell = True`, written as
`gForceBW = 1`: MadEvent cuts the point outside the `bwcutoff` window and samples a
Breit–Wigner inside it. `trim_diagrams(decay_ids)` removes nothing; it only sets that flag.
A decay whose particle appears in no core final state is **discarded with a warning**
(`diagram_generation.py:1405`).

## Stitching (`diagrams/chain.rs`)

[^code-chain] [^n38-d2]

1. **Enumerate the parts.** The core is enumerated as an ordinary process. Each decay is
   enumerated as a 1 → n process (see [1 → n decays](decay-processes.md)) with its **own**
   lowest-`WEIGHTED` search, recursively for a decay with decays of its own.
2. **Assign decays to legs** per core subprocess, as MadGraph's
   `combine_decay_chain_processes` does (`helas_objects.py:5427`; the assignment branch
   at L5510–L5566) [^mg-combine]:
   - as many decayable legs as decays, decay *i* applying to leg *i*: in order;
   - otherwise, as many legs as decays, each decay one concrete process with the same
     particles: the decays of each particle to its legs, in order;
   - otherwise every combination, with repetition, of all the decays of each particle over
     its legs, each unordered combination once.

   Measured against MadGraph's own generation [^n38-g1]: `z z, z > e+ e-, z > mu+ mu-`
   gives one matrix element (ee + μμ); `z z, z > e+ e-` decays both Z to ee; three decays
   over two Z give six; `z > l+ l-` gives ee/ee, ee/μμ and μμ/μμ.
3. **Glue.** The decayed final-state leg and the decay's incoming leg become one
   propagator, from the core vertex into the decay vertex, flagged `OnShell::Forced`. The
   decay's products replace the leg in place, in the decay's order, as
   `Process.get_legs_with_decays` does (`base_objects.py:3667`) [^mg-legs-with-decays].
4. **Rebuild from the graph.** The stitched diagram's propagator momenta
   (`Diagram::tree_momentum`) and its sign (`Diagram::fermion_pairing_sign` × the line
   sign) are recomputed from the whole graph, not combined from the parts: the momentum
   representative drops the last external leg, and the pairing parity reads the global leg
   order. The symmetry factor is the parts' product, 1 on every tree.
5. **Permute between blocks.** Every permutation of identical final-state particles
   *between* blocks (the core's legs and each decay's products) is applied and each graph
   kept once. This is a deliberate departure from MadGraph, which keeps one pairing and
   divides by `identical_decay_chain_factor`; see
   [identical particles across decays](identical-particles-across-decays.md).

**Why the sign is rebuilt.** A naive sign (core sign × decay signs) passes every case
where block insertion leaves the pairing parity alone and crossed lines keep their flip.
It fails on `g b > w- t, t > w+ b`, where the initial `b` line gains the `t` propagator.
That is a global sign, visible only to container equality and per-diagram amplitudes,
never to |M|². [^n38-d2]

### Representation

- `Prop::onshell: OnShell { Free, Forced, Forbidden }` is MadGraph's per-propagator flag
  and part of the canonical diagram key. Phase space reads the `Forced` lines, with mass
  and width from `Prop::particle`. `Forbidden` exists for `$` lines, but the on-shell veto
  keeps its own per-subprocess bookkeeping
  ([s-channel restrictions](s-channel-restrictions.md)) and no enumerated diagram carries
  it today.
- `Diagram::provenance: Provenance { process (the @N), decays: Vec<DecayOrigin { node,
  prop }> }`, not part of the key. `ChainNode` is the preorder index of a decay on the
  card: the core is 0, and `(t > w+ b, w+ > e+ ve), t~ > w- b~` numbers its decays 1, 2, 3.
  Event records find a forced line's mother through `Diagram::final_state_side`
  ([resonance records](../events/resonance-records.md)).
- `Prop::particle` is the particle of the slot at `endpoints[1]`; renumbering a diagram
  that flips a line must conjugate its particle. The canonical form and rooting are in
  [rooting invariance and the anchor](../amplitudes/rooting-invariance-and-anchor.md).

### What is refused

`DiagramError::DecayChain` refuses, each with its reason:

- a decay whose particle is in no core final state (MadGraph drops it with a warning;
  here it is almost always a missing parenthesis);
- a core subprocess no decay applies to (MadGraph keeps it undecayed, beside decayed
  subprocesses of another multiplicity);
- decays that give final states of different multiplicities;
- an **ambiguous forced line**: in `e+ e- > z e+ e-, z > e+ e-` the core holds its own
  s-channel `Z → e+ e-`, so after permutation one graph is stitched twice with a different
  line forced (64 stitched against 60 filtered).

Card-level refusals: a polarization on a decayed leg (`DecayedPolarization`), a `$` inside
a decay (`DecayOnShellVeto`), and an overall order that a part bounds with `==` or `>`
(`ChainOrders`).

**Overall orders** (`@1 QED=4`) are supported: each part's upper bound becomes the lesser
of its own and the overall one, added where the part has none, which switches that part's
automatic search off (`diagram_generation.py:570`, `:1972`); a stitched diagram exceeding
an overall order is removed (`helas_objects.py:3986`). [^n38-d3]

## Oracles

1. **Container equality** (`helas::eval::stitching`): the stitched diagrams against the
   undecayed final state enumerated at the stitched `WEIGHTED` order and filtered by
   `schannel::match_resonances` (each chain resonance an s-channel line with the right
   oriented id and exactly its stated products on its final-state side, recursively), the
   matched lines flagged forced. Ten cards, 147 diagrams, including
   `e+ e- > z z, z > e+ e-` (4), nested `(t > w+ b, w+ > e+ ve)` (2),
   `g g > t t~ g, t > w+ b` (16, one four-gluon), `u u~ > z g g g, z > e+ e-` (50, two
   four-gluon) and `p p > z j, z > l+ l-` (24 subprocesses, 48 diagrams). All equal as
   containers except two of `e+ e- > w+ w- z, z > mu+ mu-`, which bind the forced Z and an
   s-channel Z to the two Z slots of a `W W Z Z` / `H Z Z` vertex in the other order; the
   test pairs those by a slot-order-free description. Dropping the between-block
   permutations fails it (`z z, z > e+ e-`: 2 stitched, 4 filtered).
2. **Per-diagram amplitudes**: all 147 pairs agree per helicity and per colour flow,
   worst 1.2e-16 relative (bit-equal except the two slot-swapped pairs).
3. **MadGraph census** (`validation/madgraph/dump_decay_chain_census.py` →
   `decay_chain_census.json`, hermetic `decay_chain_census`, `HelasMultiProcess` on the
   pinned checkout): 24 cards. The decay-chain set stitches 17 cards and matches 54
   (process, decays) pairs, final-state order included, with MadGraph's per-matrix-element
   diagram count equal to the stitched diagrams whose forced lines lead to MadGraph's
   blocks; 2 are refused (a two-particle "decay", refused by both; a dropped `w+` decay,
   refused here by design). Five overall-order cards (`pp_ttx_qed2`, `pp_ttx_qed4`,
   `pp_ttx_qcd0`, `ee_ttx_qed4`, and `ee_zz_qed3`, refused by both) pin the order rules.
   [^n38-d2] [^n38-d3]

These oracles are blind to the forced-window phase space, which the σ gates cover.

## Downstream

Decay-chain cards integrate and generate. The comparison with MadGraph is its decay-chain
σ, not σ × BR: the window keeps about (2/π)·atan(2·`bwcutoff`) of the Breit–Wigner, 0.979
at the default 15. The forced windows and maps are in
[resonance and pole maps](../phase-space/resonance-and-pole-maps.md), the cost ladder in
[decay chains without MadSpin](../performance/decay-chains-without-madspin.md), and the
event record in [resonance records](../events/resonance-records.md). `cut_decays` is
consumed. A decay chain's dynamic scale is the core process's, as MadEvent's: the
clustering tags a forced line on-BW inside its window whatever its width
(`ForestLine::forced`, `coupling/cluster/kt.rs`, after `myamp.f`'s `gForceBW = 1`
branch), so the resonance reaches the scale setting as one core leg.

[^n38-decays]: Note 38 §1.3: MadGraph's decay-chain syntax, `DecayChainAmplitude`, `gForceBW = 1`, the dropped decay.
[^n38-g1]: Note 38 §4, grammar, AST and the one check: decay assignment measured with MadGraph's generation.
[^n38-d2]: Note 38 §4, decay-chain enumeration by stitching: stitching, representation, refusals, the three oracles and the mutations.
[^n38-d3]: Note 38 §4, decay-chain phase space and σ: chains accepted by the check, overall orders, the census extension, the BW window fraction.
[^code-chain]: `vibegraph-lib/src/diagrams/chain.rs` module documentation.
[^mg-decay-chain-process]: `madgraph/interface/madgraph_interface.py` `extract_decay_chain_process`, L5661.
[^mg-decay-amp]: `madgraph/core/diagram_generation.py` `DecayChainAmplitude`, L1337; the dropped decay at L1405.
[^mg-combine]: `madgraph/core/helas_objects.py` `combine_decay_chain_processes`, L5427.
[^mg-legs-with-decays]: `madgraph/core/base_objects.py` `get_legs_with_decays`, L3667.
