# Feynman diagrams

At tree level the matrix element is a sum over Feynman diagrams: every
connected tree whose external lines are the process's particles and whose
internal vertices are interactions of the model. Enumerating those trees is
the first computation a generator performs, and the earliest automated one:
the original MadGraph ([Stelzer & Long 1994](../bibliography.md#diagrams))
was an automatic diagram enumerator that wrote HELAS calls.

## The process grammar

A process is written in MadGraph's grammar, in a `proc_card.dat`:

```text
import model sm
define l+ = e+ mu+
define l- = e- mu-
generate p p > l+ l- j QCD=2 QED=2 @1
```

The parser reproduces MadGraph's own `extract_process`: modifiers are
stripped from the string in a fixed order by plain string operations
(coupling-order constraints such as `QCD=2`, a process number `@1`,
decay-chain and exclusion syntax), and what is left splits on `>` into
initial and final legs. Particle names are matched case-insensitively and
canonicalised to the model's spelling, as MadGraph does. Decay-chain syntax
is recognised and rejected, since it is outside the supported scope.

**Multiparticle labels** (`p`, `j`, `l+`, …) are aliases for lists of
particles. The Standard Model's default aliases match MadGraph's, and
`define` lines add more. A process with aliased legs expands to the
Cartesian product over every leg's members, each concrete process being
submitted for enumeration separately; `p p > l+ l- j` expands to a few dozen
of them. The [proton beams](10-hadronic.md) chapter is about what happens to
that many subprocesses afterwards.

**Coupling orders** constrain how many powers of each coupling the diagrams
may carry. `QCD=2 QED=2` keeps diagrams with at most two of each. When a card
gives no constraint, MadGraph's rule applies: orders are weighted by the
model's hierarchy (in the SM, `QCD` counts once and `QED` twice) and the
lowest weighted order that produces any diagram is selected, so `p p > j j`
is pure QCD at leading order and the electroweak diagrams enter only if
asked for.

## Enumeration

The enumeration itself is delegated to
[feyngraph](https://github.com/Jens-Braun/FeynGraph), a Rust diagram
generator. Its algorithm is the topology-first scheme of the original
MadGraph paper:

1. Generate every tree **topology** with the right number of external
   legs, recursively: the unique three-leg topology, then each larger one
   by attaching a new leg to every existing line and every existing vertex
   of the smaller ones, deduplicated. Four legs give four topologies, five
   give twenty-five.
2. **Insert particles**: for each topology, assign a particle to every
   internal line such that every vertex is an interaction of the model. The
   model is built from the UFO vertex table (`ufo::topo`), so a model with
   different interactions produces different diagrams with no change to the
   enumerator.
3. Filter by the coupling-order constraint and discard diagrams that vanish
   by the model's restriction.

Each diagram comes back with what the amplitude needs: its external legs,
its vertices with the UFO interaction they realise and their legs in the
interaction's slot order, its propagators with the particle and a momentum
routing (each internal momentum as a signed combination of the external
ones), the relative sign from the permutation of fermion lines, and the
symmetry factor. `diagrams::Diagram` is the owned form the rest of the crate
works from.

All legs are presented in the **all-incoming convention**: an outgoing
particle is enumerated as its incoming antiparticle. The HELAS layer undoes
this when it builds wavefunctions, and it is the reason the fermion-flow
bookkeeping described in the [helicity amplitudes](04-helicity-amplitudes.md)
chapter has a `crossed` bit.

## What is validated

Diagram counts are the first thing compared against MadGraph, for every
process in the validation set, and they are a sharper oracle than they
look: the counts have caught by-hand census claims that were wrong, and a
count that agrees with the census MadGraph wrote for the same card is what licenses the
per-diagram amplitude comparison in the next chapter. The comparison is
hermetic: the census is committed to the repository as `validation/madgraph/diagrams.json`.

## Where it lives

[`vibegraph::diagrams`](../api/vibegraph/diagrams/index.html):
[`parse`](../api/vibegraph/diagrams/parse/index.html) for the grammar,
[`alias`](../api/vibegraph/diagrams/alias/index.html) for the multiparticle
expansion, [`selector`](../api/vibegraph/diagrams/selector/index.html) for the
coupling-order filter handed to feyngraph, and
[`diagram`](../api/vibegraph/diagrams/diagram/index.html) for the owned
diagram. `parse_proc_card` and `generate_from_proc_card` are the entry
points.
