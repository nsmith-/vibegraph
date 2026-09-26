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
| decay chains `A > B C, B > D E` | enumerated (see below); refused for integration until phase space keeps each resonance in its Breit–Wigner window |
| overall orders on a decay chain (`@1 QED=2` after the process number) | refused |
| propagator projections `{A}` `{G}` `{H}` `{Q}` `{W}` `{S}`, and a polarization on a particle a decay chain decays | refused |
| squared-order constraints `QCD^2<=4`, `aEW`, `aS` | refused (see below) |
| `WEIGHTED==n`, `WEIGHTED>n` | refused: MadGraph reads them as squared-order constraints |
| `[QCD]`, `[real=QCD]`, `!a!` | refused: NLO |
| `add process` with another final-state multiplicity | refused: needs jet merging |
| `set` of a physics-bearing option off its default | refused |
| run-card edits after `launch` | refused: the run card is its own file |

Benign `set` options (paths, compilers, `group_subprocesses`, …) and
commands such as `output` and `display` are accepted and change nothing.

A process with one initial particle (`generate t > b e+ ve`) is a `1 → n`
decay, integrated to a [partial width](07-phase-space.md#decays-at-rest).
It is enumerated like any other process — every diagram with the decaying
particle as its one incoming leg, every internal line then an s-channel —
and the same enumeration is available for a single decay on its own
(`enumerate_decay`), which is what joining decays onto a core process
builds on. Mixing processes with different numbers of initial particles is
a MadGraph error and stays one.

**Decay chains** are enumerated by stitching, as MadGraph generates them: the
core (`p p > t t~`) and each decay (`t > w+ b`, recursively for a decay with
decays of its own) are enumerated separately, each with its own
lowest-order search, and each decay's diagrams are glued onto the matching
final-state leg of each core diagram. The leg becomes an internal line,
flagged *forced on shell* (MadGraph's `onshell = True`), and the decay's
products take its place in the final state, so `e+ e- > t t~, t > w+ b,
t~ > w- b~` has the final state `w+ b w- b~`. Which decay goes to which leg
follows MadGraph: as many decays as decaying legs go to them in order
(`e+ e- > z z, z > e+ e-, z > mu+ mu-` is one subprocess, `e+ e- mu+ mu-`);
otherwise every combination of the decays with repetition, each unordered
combination once (`z > l+ l-` on two Z bosons gives `ee ee`, `ee μμ` and
`μμ μμ`). A decay whose particle is in no core final state is an error here;
MadGraph drops it with a warning, which usually means a nested decay lost its
parentheses.

One difference from MadGraph is deliberate. MadGraph keeps each decay's
products on the legs it assigned and divides by a factor for identical decay
chains, dropping the interference between assignments. Here the stitched
diagrams are closed under every permutation of identical final-state
particles, so a stitched subprocess is exactly the set of diagrams of the
undecayed final state in which every chain resonance is an s-channel line
with its stated products, and the final state's own identical-particle
factor applies: `e+ e- > z z, z > e+ e-` has four diagrams (MadGraph two,
over a factor 2). Where the core itself holds the resonance with the same
products (`e+ e- > z e+ e-, z > e+ e-`), one diagram would carry two lines
either of which could be the decay's, and the card is refused.

Integrating a decay chain needs phase space that keeps each forced line
within `bwcutoff` widths of its mass, as MadEvent does; until it has it, the
check refuses decay chains for integration and event generation, and
`check_enumerable` with `generate_decay_chains` is the library entry point
for their diagrams.

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
`define` lines add more. Importing a model rewrites `p` and `j` as MadGraph
does: when the model's b quark is massless (`sm-no_b_mass`, `sm-no_masses`,
`sm-zeromass_ckm`) `b b~` join them, the five-flavour scheme, and when it is
massive they leave; a label defined *through* `p` afterwards sees the
rewritten one, while a `define p = …` after the import is taken as written.
So `import model sm-no_b_mass` + `generate p p > e+ e-` includes
`b b~ > e+ e-`. (The SMEFTsim restrictions in the validation set, `massless`
included, keep the b massive and stay four-flavour.) A process with aliased legs expands to the
Cartesian product over every leg's members, each concrete process being
submitted for enumeration separately; `p p > l+ l- j` expands to a few dozen
of them. The [proton beams](10-hadronic.md) chapter is about what happens to
that many subprocesses afterwards. Every process line's subprocesses are
added to one cross section, so a subprocess reached from two lines
(`generate p p > e+ e-` then `add process u u~ > e+ e-`) is refused rather
than counted twice. MadGraph accepts that card and generates `u u~ > e+ e-`
twice.

**Polarized legs** restrict an external leg to named helicities, with
MadGraph's codes: `{0}` longitudinal, `{T}` the two transverse states, `{L}`
and `{R}` helicity −1 and +1 (for a vector `{L}` is still −1, not
longitudinal), `{+1}`/`{-1}` and the other signed codes, and any
concatenation of them (`{0T}`). A comma is not a separator, as in MadGraph,
where it starts a decay. The restriction applies to the helicity sum only:
the diagrams are those of the unpolarized process, and the helicity
combinations summed over are the product of each leg's own list — MadGraph's
`NHEL` table for the card, row for row. What else follows MadGraph exactly:

- A polarized *incoming* leg averages over the helicities it lists, not over
  its particle's states: `e+ e-{L} > mu+ mu-` divides by 2 where the
  unpolarized card divides by 4, the cross section of a fully polarized beam.
  This is MadGraph's `IDEN`.
- Two outgoing legs are identical, for the `1/n!` symmetry factor, only when
  they are the same particle polarized alike: `z{0} z{0}` carries `1/2`,
  `z{0} z{T}` carries 1. A polarized and an unpolarized leg of one particle
  are different subprocesses throughout — deduplication, grouping,
  labels.
- Helicity 0 on a massless boson is kept by the parser and dropped at
  generation, and an assignment left with no helicity is dropped with it:
  `define v = z a` then `u u~ > v{0} g` generates `u u~ > z{0} g` alone, and
  `e+ e- > a{0} z` has no subprocess and is an error.
- A card where the same outgoing particle is both polarized and not, or
  polarized with overlapping lists (`p p > z{T} z`, `z{L} z{T}`), is
  refused: MadGraph calls it ambiguous and a batch run stops there.

In a `1 → n` decay the outgoing legs may be polarized (`t > w+{0} b`), and
the decaying particle averages over its states as an incoming leg does; the
decaying particle itself may not be (`t{L} > w+ b`), since at rest its
helicity is a spin projection on an axis the wavefunction routine picks.

Three more things MadGraph accepts are refused here: a code that is no helicity
state of the particle (`z{2}`, `h{R}`: MadGraph hands the number to the
wavefunction routine regardless), a code listed twice (`z{00}`, which
MadGraph would sum twice), and two process lines whose subprocesses of the
same particles would both count some helicity state (`e+ e- > z{0} h` with
`add process e+ e- > z h`). Lines that split one process by disjoint
polarizations (`z{0} h` and `z{T} h`) add up to it, as they should.

A polarized squared amplitude is not Lorentz invariant when a *massive* leg
is polarized, since a massive particle's helicity depends on the frame.
MadGraph evaluates it in the rest frame its run card's `me_frame` names,
by default `[1, 2]`, the partonic centre of mass. That is the frame every
evaluator here is handed, so the default needs nothing; a run card naming
another frame is refused for such a process. A massless leg's helicity is
invariant, and a helicity sum is too, so neither a massless polarization nor
an unpolarized process cares. In an event record a polarized leg's `SPINUP`
is its helicity, one fixed value for a single-state polarization.

**Coupling orders** constrain how many powers of each coupling the diagrams
may carry. `QCD=2 QED=2` keeps diagrams with at most two of each; `==` asks
for exactly that many and `>` for more. When a card gives no constraint,
MadGraph's rule applies: orders are weighted by the model's hierarchy (in
the SM, `QCD` counts once and `QED` twice) and the lowest weighted order
that produces any diagram is selected, so `p p > j j` is pure QCD at leading
order and the electroweak diagrams enter only if asked for. `WEIGHTED<=n`
(or `WEIGHTED=n`) asks for that weighted bound directly: `u u~ > d d~
WEIGHTED<=4` keeps the gluon, photon, Z and W diagrams where the automatic
choice keeps the gluon alone.

**S-channel restrictions** select diagrams by their timelike propagators.
`e+ e- > z > mu+ mu-` keeps the diagrams with an s-channel Z; `$$ t t~`
drops every diagram with an s-channel top of either kind. Several names
between the `>` must all be present (`> z h >`), and `|` gives alternatives
(`> z | a >`, or `define v = z | a` then `> v >`). A propagator is named as it
flows towards the final state, as MadGraph names it, so `u d~ > w- > e+ ve`
has no diagram (its W is a W+), and `$$ t` alone keeps the diagrams with an
s-channel t~. The restrictions act inside the automatic order search, as in
MadGraph: `u u~ > a > d d~` is found at the electroweak order because the
QCD order has no diagram with an s-channel photon. They change which
diagrams are summed, nothing else — there is no Breit–Wigner window, and
dropping part of a gauge-invariant set is in general not gauge invariant,
which is logged as a warning. A process line none of whose subprocesses has
a diagram is an error, as it is in MadGraph. The single `$` (`p p > e+ e- $ z`) is not a
diagram filter: every diagram is kept, and a marked propagator is zeroed where
it is inside its Breit–Wigner window (see
[multichannel integration](09-multichannel.md#forbidden-on-shell-s-channels)).
Every process line of a card has to name the same `$` list.

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

The s-channel restrictions, explicit `WEIGHTED` bounds and the five-flavour
rewrite have a census of their own against MadGraph's generation
(`validation/madgraph/schannel_census.json`): for each card, every
subprocess with its diagram count and, per diagram, the ids of its s-channel
propagators as MadGraph orients them, compared by a hermetic test. Reading
a propagator the wrong way round changes those ids, so the census pins the
orientation, not only the counts.

Diagram counts are the first thing compared against MadGraph, for every
process in the validation set, and they are a sharper oracle than they
look: the counts have caught by-hand census claims that were wrong, and a
count that agrees with the census MadGraph wrote for the same card is what licenses the
per-diagram amplitude comparison in the next chapter. The comparison is
hermetic: the census is committed to the repository as `validation/madgraph/diagrams.json`.
The decay census (`t > b e+ ve a`, `z > e+ e- mu+ mu-`, `w+ > j j`, …) is
compared against MadGraph's own generation of the same cards.

Decay chains are held to two oracles. The stitched diagrams of each case are
compared, as owned diagrams up to renumbering and including their signs and
forced lines, with the diagrams of the undecayed final state that hold every
chain resonance with its products, and each pair is compared amplitude by
amplitude, helicity by helicity and colour flow by colour flow. And a census
of decay-chain cards against MadGraph's own combination
(`validation/madgraph/decay_chain_census.json`) compares the subprocesses,
their final-state order with the products in place, and per subprocess
MadGraph's diagram count against the stitched diagrams whose forced lines
lead to the legs MadGraph assigned.

## Where it lives

[`vibegraph::diagrams`](../api/vibegraph/diagrams/index.html):
[`parse`](../api/vibegraph/diagrams/parse/index.html) for the grammar,
[`check`](../api/vibegraph/diagrams/check/index.html) for the one check and
its backlog table, [`resolve`](../api/vibegraph/diagrams/resolve/index.html)
for names against the model,
[`alias`](../api/vibegraph/diagrams/alias/index.html) for the multiparticle
labels, [`selector`](../api/vibegraph/diagrams/selector/index.html) for the
coupling-order filter handed to feyngraph,
[`schannel`](../api/vibegraph/diagrams/schannel/index.html) for the
s-channel filter on converted diagrams and the resonance matching decay
chains are compared with, and
[`diagram`](../api/vibegraph/diagrams/diagram/index.html) for the owned
diagram, its propagators' on-shell flags and its provenance (the process
number and the decay-chain nodes it was stitched from). `parse_proc_card`
(parse and check) and `generate_from_proc_card` are the entry points;
`check_enumerable` and `generate_decay_chains` enumerate decay chains.
