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
add process p p > l+ l- j QCD=1 QED=3 @2
```

A card is read in three steps, and only the last one needs the model.

**1. Parse everything.** The parser reads the whole of MadGraph's process
language into a syntax tree that drops nothing, using the regular
expressions of MadGraph's own `extract_process` in MadGraph's order: the
process number `@N` (text may follow it, so `p p > j j @1 QED=0` carries both),
the loop specification `[...]`, the coupling-order constraints, then the
forbidden particles `/`, the forbidden s-channels `$$`, the forbidden
on-shell s-channels `$`, and the required s-channels between two `>`. What
remains are the legs. Each leg is read the way MadGraph reads it: first as a
multiparticle label, then as an integer PDG code (`11`, `-11`, `21`), then as
a particle name, and only when none of those applies as a repeat count
followed by a name (`2e+`, `2j`). A leg may carry a polarization (`w+{0}`,
`z{T}`, `e-{L}`) or a photon tag (`!a!`). Decay chains
(`p p > t t~, (t > w+ b, w+ > j j), t~ > w- b~`) nest as MadGraph nests them.

The command sequence keeps MadGraph's semantics too: `generate` and
`import model` discard every process before them, `add process` appends, a
process without `@N` is numbered by its position among the process lines
since the last `generate`, and a label means what the last `define` before
the line said. `define v = z | a` makes an or-multiparticle, usable only as a
required s-channel. `#` starts a comment, `;` separates commands, and a
trailing `\` continues a line. Where MadGraph refuses a line, so does the
parser: an unspaced `p p>e+ e-`, a comparison MadGraph does not accept
(`QED>=2`), `add process` without a process.

**2. Check once.** One pass over the parsed card reports *every* feature this
generator does not honour, all at once, and only a card with none reaches
enumeration — as a narrower type that has no place to put a refused feature.
The refusals are the feature backlog against MadGraph's leading-order process
language:

| Feature | Status |
|---|---|
| `>` required s-channels, or-multiparticles, `$$` | refused until the s-channel diagram filter exists |
| `$` forbidden on-shell s-channels | refused until the per-channel on-shell veto exists |
| decay chains `A > B C, B > D E` | refused until stitched decay enumeration exists |
| 1→n decay processes (`t > w+ b`) | refused until rest-frame phase space exists |
| polarized legs `{0}` `{T}` `{L}` `{R}` | refused until the restricted helicity loop exists |
| propagator projections `{A}` `{G}` `{H}` `{Q}` `{W}` `{S}` | refused |
| squared-order constraints `QCD^2<=4`, `aEW`, `aS` | refused (see below) |
| `WEIGHTED<=n` | refused |
| `[QCD]`, `[real=QCD]`, `!a!` | refused: NLO |
| `add process` with another final-state multiplicity | refused: needs jet merging |
| `set` of a physics-bearing option off its default | refused |
| run-card edits after `launch` | refused: the run card is its own file |

Benign `set` options (paths, compilers, `group_subprocesses`, …) and
commands such as `output` and `display` are accepted and change nothing.
Mixing processes with different numbers of initial particles is a MadGraph
error and stays one.

A squared-order constraint bounds the order of an *interference* term in
$|M|^2$, a statement about pairs of diagrams. This generator selects diagrams
by their own orders and squares the whole amplitude, so it cannot honour one,
and answering the amplitude-level question instead would return a different
cross section with nothing to say so.

**3. Resolve against the model.** Names are matched case-insensitively and
canonicalised to the model's spelling, as MadGraph does. A name or PDG code
the model does not have, and a coupling order it does not define, are hard
errors. A forbidden particle `/ w+` forbids the propagator in either
orientation, as MadGraph's does.

**Multiparticle labels** (`p`, `j`, `l+`, …) are aliases for lists of
particles. The Standard Model's default aliases match MadGraph's, and
`define` lines add more. A process with aliased legs expands to the
Cartesian product over every leg's members, each concrete process being
submitted for enumeration separately; `p p > l+ l- j` expands to a few dozen
of them. The [proton beams](10-hadronic.md) chapter is about what happens to
that many subprocesses afterwards. Every process line's subprocesses are
added to one cross section, so a subprocess reached from two lines
(`generate p p > e+ e-` then `add process u u~ > e+ e-`) is refused rather
than counted twice. MadGraph accepts that card and generates `u u~ > e+ e-`
twice.

**Coupling orders** constrain how many powers of each coupling the diagrams
may carry. `QCD=2 QED=2` keeps diagrams with at most two of each; `==` asks
for exactly that many and `>` for more. When a card gives no constraint,
MadGraph's rule applies: orders are weighted by the model's hierarchy (in
the SM, `QCD` counts once and `QED` twice) and the lowest weighted order
that produces any diagram is selected, so `p p > j j` is pure QCD at leading
order and the electroweak diagrams enter only if asked for.

## Enumeration

The enumeration itself is delegated to
[feyngraph](https://github.com/Jens-Braun/FeynGraph), a Rust diagram
generator built for multi-loop work, of which vibegraph uses the tree-level
case. Its algorithm is **topology-first**: the shapes of all graphs are
enumerated without reference to any particle, and particles are assigned
to each shape afterwards.

1. **Topologies.** A tree with $E$ external legs and $N_k$
   vertices of degree $k$ satisfies $\sum_k (k-2) N_k = E - 2$,
   which fixes the admissible vertex-degree partitions for the degrees the
   model's interactions have (three and four in the Standard Model). For
   each partition the generator fills an adjacency matrix by depth-first
   backtracking, and keeps a graph only if it is the canonical
   representative of its permutation orbit, which is what makes the
   enumeration duplicate-free and yields the symmetry factor as a
   by-product. This is the orderly-generation approach of QGRAF
   ([Nogueira 1993](../bibliography.md#diagrams)) rather than the
   leg-attachment recursion of the original MadGraph, though both are
   topology-first.
2. **Particle assignment.** For each topology, a second backtracking pass
   assigns a particle to every internal line such that every vertex is an
   interaction of the model, pruning as soon as a partial assignment has no
   vertex to complete it, and again keeping only canonical assignments. The
   model is built from the UFO vertex table (`ufo::topo`), so a model with
   different interactions produces different diagrams with no change to the
   enumerator. Topologies are processed in parallel.
3. **Filtering** by the coupling-order constraint, and discarding diagrams
   that vanish under the model's restriction.

> **Aside: what MadGraph 5 does instead.**
> MadGraph 5 ([Alwall et al. 2011](../bibliography.md#the-pipeline-as-a-whole))
> abandoned topology-first enumeration. Its algorithm starts from the list of
> external legs, all flipped to one convention, and recursively **combines
> subsets of legs through the model's interactions**: a subset is replaced
> by the single off-shell leg the matching vertex implies, the reduced list
> is recursed into, and the recursion closes when the remaining legs form a
> final vertex. Duplicates are removed by a canonical tag per diagram, and
> coupling-order limits prune the recursion as it runs.
>
> The difference is structural. Topology-first generates every graph shape
> and only then discovers that most admit no particle assignment, and the
> number of tree shapes grows factorially with the leg count, faster still
> once four-point vertices are admitted. Leg combination never visits a
> shape the model cannot fill, because a subset of legs is only combined
> when a vertex exists for it; $n$-point vertices cost nothing extra, and
> an order constraint stops a branch early rather than discarding its
> output. That pruning is what let MadGraph 5 handle high multiplicities its
> predecessor could not, and it is also what made reusing diagrams across
> flavour-relabelled subprocesses natural. At the multiplicities this
> generator validates, up to $2\\to6$, feyngraph's enumeration is a small
> fraction of a run; a self-implemented leg-combination enumerator is a
> tracked research item, not a present need.

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

The grammar is checked against MadGraph's own parser: a corpus of over a
hundred proc cards, covering every construct above and MadGraph's error
cases, was run through MadGraph's command interface and the process
definitions it built were committed as `validation/madgraph/proc_grammar.json`.
A hermetic test parses the same cards and compares every field (PDG codes,
polarizations, restriction lists, the coupling-order dictionaries MadGraph
derives, process numbers, decay chains), and requires that every card
MadGraph refuses is refused here too.

Diagram counts are the first thing compared against MadGraph, for every
process in the validation set, and they are a sharper oracle than they
look: the counts have caught by-hand census claims that were wrong, and a
count that agrees with the census MadGraph wrote for the same card is what licenses the
per-diagram amplitude comparison in the next chapter. The comparison is
hermetic: the census is committed to the repository as `validation/madgraph/diagrams.json`.

## Where it lives

[`vibegraph::diagrams`](../api/vibegraph/diagrams/index.html):
[`parse`](../api/vibegraph/diagrams/parse/index.html) for the grammar,
[`check`](../api/vibegraph/diagrams/check/index.html) for the one check and
its backlog table, [`resolve`](../api/vibegraph/diagrams/resolve/index.html)
for names against the model,
[`alias`](../api/vibegraph/diagrams/alias/index.html) for the multiparticle
labels, [`selector`](../api/vibegraph/diagrams/selector/index.html) for the
coupling-order filter handed to feyngraph, and
[`diagram`](../api/vibegraph/diagrams/diagram/index.html) for the owned
diagram. `parse_proc_card` (parse and check) and `generate_from_proc_card`
are the entry points.
