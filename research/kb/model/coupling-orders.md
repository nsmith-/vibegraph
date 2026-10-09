---
type: Algorithm
title: "Coupling orders: interaction splitting and expansion_order caps"
description: "One interaction per coupling-order tuple, as MadGraph's add_interaction; an unconstrained order is capped only when 0 < expansion_order < 99; the WEIGHTED hierarchy keeps auxiliary fields out."
status: draft
tags: [ufo, coupling-orders, weighted, smeftsim, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n02-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L519-L527", title: "Note 02, UFO parsing: FeynGraph vs vibegraph"}
  - {id: n35-probe, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L144-L174", title: "Note 35 §1.3, the measured SMEFTsim loader probe"}
  - {id: n35-conv, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L175-L214", title: "Note 35 §1.4, conventions read from the pinned MadGraph source"}
  - {id: n35-l1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L604-L675", title: "Note 35 §4 L1, loader and model-topology surface (ufo-lorentz)"}
  - {id: mg-add-interaction, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L1773", title: "MadGraph import_ufo.py add_interaction / order_to_int"}
  - {id: mg-expansion-import, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L662-L677", title: "MadGraph import_ufo.py, model expansion_order"}
  - {id: mg-check-expansion, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/base_objects.py#L3757-L3770", title: "MadGraph base_objects.py Process.check_expansion_orders"}
  - {id: mg-optimal-orders, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L1685-L1688", title: "MadGraph diagram_generation.py, expansion caps after find_optimal_process_orders"}
measured:
  - {commit: 00858a8, pr: 5, landed_in: e73b158, command: "cargo test -p vibegraph-lib --features extended-validation --test smeftsim"}
---

A UFO coupling carries an `order` dictionary (`{'QED': 1}`, `{'NP': 1, 'QED': 1}`, …).
Diagram selection works on those orders: an explicit constraint (`QED<=2`, `NP<=1`)
bounds each diagram's order sum, and with no explicit constraint the generator runs
MadGraph's automatic search for the lowest `WEIGHTED` order, where
`WEIGHTED = Σ hierarchy(order) × order` and the hierarchy comes from
`coupling_orders.py`. A model without that file, or with no hierarchy data in it, gets
the Standard-Model default `QCD = 1, QED = 2` (`ufo/mod.rs`, `default_sm_hierarchy`).
How the constraint operators read is part of the process grammar
([proc-card grammar](../process/proc-card-grammar.md)); squared-order constraints are
refused by the [card check](../process/supported-card-check.md).

## One interaction per coupling-order tuple

A UFO vertex may hold couplings of different orders. SMEFTsim's `FFV` vertices bundle
the SM gauge coupling with dipole and current-shift couplings whose orders include
`NP`. Read as one vertex with the union of its couplings' orders, an SM photon
current carries `NP = 1`, two of them exceed any `NP<=1` or `WEIGHTED` bound, and
`e+ e- > mu+ mu-` in the SM limit of SMEFTsim had **0** diagrams where MadGraph has 2.
[^n35-probe]

The loader therefore splits as MadGraph's `add_interaction` does (`order_to_int`):
one interaction per distinct coupling-order tuple, each holding only that tuple's
`(colour, Lorentz)` coupling entries. [^mg-add-interaction] In vibegraph this is
`split_vertices_by_coupling_order` (`vibegraph-lib/src/ufo/mod.rs`), run inside
`ParsedModel::parse`, before any restriction:

- splits are named `<vertex>#<n>`, 1-based in the order their tuples first appear;
- a vertex with a single tuple keeps its name and content unchanged, which is every
  vertex of the Standard Model;
- each split drops the Lorentz structures none of its couplings reference.

`build_feyngraph_model` (`ufo/topo.rs`) reads the order tuple off a split vertex's
couplings and fails with `Vertex '…' mixes coupling orders across its couplings` if
they disagree, so a loader change cannot quietly reintroduce the union.

**One deliberate difference.** The split key is the *sorted* order map; MadGraph keys
on the UFO dict's insertion order. Two couplings writing the same orders in a
different sequence would split apart in MadGraph and merge here. No pair like that
exists in the six models checked. [^n35-l1]

**Splitting is per order tuple, then per fermion flow.** It is not per Lorentz
structure, which is what FeynGraph's own parser does and what note 02 proposed as the
template. [^n02-ufo] After the order split, `build_feyngraph_model` emits one feyngraph
vertex per fermion-pairing group of the vertex's structures (`<name>@<g>`). That second
split lives at the feyngraph/diagram layer, not in the UFO interaction set, and matters
only for four-fermion vertices whose structures pair the legs differently; see
[four-fermion vertices](../amplitudes/four-fermion-vertices.md).

### Measured counts

On the vendored [SMEFTsim topU3l UFO](smeftsim-topu3l.md) [^n35-l1]:

| Restriction | Split interactions | Arity histogram |
|---|---|---|
| none (as parsed) | 1985 | `{3: 527, 4: 1234, 5: 212, 6: 12}` |
| `restrict_SMlimit_massless` | 62 | `{3: 50, 4: 10, 5: 2}` |
| `restrict_massless` | 913 | `{3: 256, 4: 564, 5: 82, 6: 11}` |

The two restricted counts equal MadGraph's own, banked per (model, restrict card)
pair in `validation/madgraph/interactions.json` and asserted by
`restricted_interaction_counts_match_madgraph`; MadGraph builds no unrestricted row,
so the 1985 (from 904 UFO `Vertex` entries, 2737 coupling entries) is a constant in
`interaction_splitting_matches_madgraph`. The arity histograms are vibegraph's own
measurement, since MadGraph's log does not report them (`vibegraph-lib/tests/smeftsim.rs`,
behind `extended-validation`). Smaller counts (60 and 540 vertices) that appear in
older notes are pre-split. For the Standard Model,
`splitting_is_the_identity_on_the_standard_model` (`ufo/mod.rs`) pins that splitting
changes nothing, with the bit-for-bit `amplitude_oracle` as the end-to-end check.

With splitting in place the SM-limit SMEFTsim diagram counts are MadGraph's:
`e+ e- > mu+ mu-` 2 (s-channel γ and Z), `g g > t t~` 3, `e+ e- > t t~` 2. Under
`QCD<=2` alone, `g g > t t~` gains a fourth diagram, `g g → H → t t̄` through the
effective `ggH` vertex (order `SMHLOOP`, hierarchy 99), which the automatic `WEIGHTED`
search drops.

## expansion_order caps

`coupling_orders.py` also gives each order an `expansion_order`. MadGraph sets the
model's `expansion_order` map only when every declared order carries the attribute;
one missing attribute leaves the map empty. [^mg-expansion-import] The loader does the
same (`parse_coupling_orders`, `ufo/mod.rs`).

The rule that uses the map is `Process.check_expansion_orders`: [^mg-check-expansion]

```python
tmp = [(k,v) for (k,v) in expansion_orders.items() if 0 < v < 99]
```

Only an order with `0 < expansion_order < 99` caps anything; 0, 99 and negative values
cap nothing. vibegraph reproduces the window in `ufo::expansion_order_caps` and folds
the caps into a process's orders in `capped_orders` (`diagrams/mod.rs`):

- an existing `<=` or `=` bound above the cap is lowered to the cap;
- an order the process leaves unconstrained gets `<= cap` added;
- an `==` or `>` constraint on the order is left alone.

The caps never decide whether the automatic `WEIGHTED` search runs. MadGraph applies
`check_expansion_orders` only after `find_optimal_process_orders` has looked at the
process's own orders [^mg-optimal-orders], so vibegraph decides `auto_weighted` from the
process's own orders before folding the caps in. Pins:
`expansion_order_caps_use_madgraphs_window` (`ufo/mod.rs`, with a synthetic SM cap),
`expansion_order_caps_reach_generation_only_inside_madgraphs_window` (`diagrams/mod.rs`)
and `the_interned_sm_declares_no_expansion_order_cap`.

**No model in reach caps anything.** The SM declares 99 for every order. SMEFTsim
declares 99 for every order except `NPprop`, which is 0 and therefore outside the
window.

## What keeps SMEFTsim's auxiliary fields out

SMEFTsim's four propagator-correction auxiliary fields (`Z1`, `W1±`, `t1`, `H1`,
PDG 9000005–9000008) appear only in vertices carrying `NPprop`. A natural reading of
`NPprop`'s `expansion_order = 0` is that it caps them out of every process. That reading
is wrong [^n35-conv]: the cap window excludes 0. In a process with no explicit orders,
what keeps them out is their hierarchy-99 weight in the automatic `WEIGHTED` search.
`NP` itself and `SMHLOOP` also carry hierarchy 99, which is why a SMEFT process has to
ask for `NP<=1` explicitly.

Caveat: once a process gives explicit orders, the automatic search does not run, and
an order it leaves unconstrained (such as `NPprop`) is not capped either. The auxiliary
fields carry custom `propagators.py` forms, so a selected diagram in which one of them
propagates is refused (`ConvertError::CustomPropagator`); see
[UFO parsing](ufo-parsing.md).

## Related

- [Restrict-card semantics](restriction-semantics.md): the restriction prunes split
  interactions by their couplings.
- [UFO parsing](ufo-parsing.md): where the split runs in the load path.

[^n02-ufo]: Note 02, cross-cutting notes on FeynGraph's UFO parser (read at FeynGraph `1dc4ea7`).
[^n35-probe]: Note 35 §1.3, the measured loader probe and the diagnosis of the zero SM-limit diagrams.
[^n35-conv]: Note 35 §1.4, the `expansion_order` hypothesis and its falsification.
[^n35-l1]: Note 35 §4, the loader and model-topology surface: split counts, the sorted-key difference, SM-limit diagram counts.
[^mg-add-interaction]: `models/import_ufo.py` `add_interaction`, `order_to_int` at L1816.
[^mg-expansion-import]: `models/import_ufo.py` L662–L677.
[^mg-check-expansion]: `madgraph/core/base_objects.py` `check_expansion_orders`, L3757; the window at L3766.
[^mg-optimal-orders]: `madgraph/core/diagram_generation.py` L1685–L1688 on the process definition, and L2089 per concrete process.
