# Colour

Quarks carry an SU(3) colour index and gluons an adjoint one, so a QCD
amplitude is a tensor in colour space as well as a function of momenta and
helicities. Evaluating colour numerically per phase-space point, by
carrying explicit colour vectors through every vertex, would multiply the
work by powers of three. MadGraph does something better, and vibegraph
follows it exactly: colour is factored out of the kinematics symbolically,
once per process, and the floating-point evaluator never sees a colour
index.

## The factorisation

Because a UFO vertex is a tensor product of a colour structure and a
Lorentz structure ([UFO models](02-ufo.md)), each diagram's amplitude
splits into a colour tensor times a kinematic amplitude. Collect the
distinct colour tensors of all diagrams into a basis
\\(\{\mathcal{C}_f\}\\); the full amplitude is then

\\[
\mathcal{M} = \sum_f \mathcal{C}_f\, J_f(p, h),
\qquad
\sum_{\text{colour}} |\mathcal{M}|^2 = \sum_{f,f'} J_f\, \mathrm{CF}_{ff'}\, J_{f'}^\ast ,
\\]

where each \\(J_f\\), MadGraph's *JAMP*, is a linear combination of diagram
amplitudes with rational coefficients, and
\\(\mathrm{CF}_{ff'} = \sum_{\text{colour}} \mathcal{C}_f \mathcal{C}_{f'}^\ast\\)
is a constant matrix of exact rational numbers. The colour sum is a small
matrix contraction at the end of every helicity evaluation, and everything
else is [helicity amplitudes](04-helicity-amplitudes.md) as before.

## The algebra

Building the basis and the matrix is a symbolic reduction. Each diagram's
colour tensor is a product of vertex atoms with leg and propagator indices
substituted in: `T(a, i, j)` for a quark–gluon vertex, `f(a, b, c)` for
three gluons, `δ` for a colourless vertex, and for the four-gluon vertex
three separate `f f` products, one per colour structure of the vertex. The
product is reduced to a canonical sum of traces and open fundamental
chains by a rewrite system, the same rule set as MadGraph's
`color_algebra.py`, iterated to a fixed point:

| Rule | Identity |
|---|---|
| \\(f\\) to traces | \\(f^{abc} = -2i\,\mathrm{Tr}(a b c) + 2i\,\mathrm{Tr}(c b a)\\) |
| chain merge | \\(T(A)_{ij}\,T(B)_{jk} = T(AB)_{ik}\\) |
| chain closes | \\(T(A)_{ii} = \mathrm{Tr}(A)\\) |
| Fierz | \\(T^a_{ij} T^a_{kl} = \tfrac12 \delta_{il}\delta_{kj} - \tfrac{1}{2N_c}\delta_{ij}\delta_{kl}\\), in its trace and chain forms |
| trace values | \\(\mathrm{Tr}(\emptyset) = N_c,\quad \mathrm{Tr}(a) = 0\\) |

Coefficients are exact rationals times a power of \\(i\\) times a power of
\\(N_c\\), kept as `num_rational::Ratio<i64>` with checked arithmetic. They
first become floating-point numbers at the very end, when the amplitude's
constant pools are built. Each distinct canonical tensor surviving the
reduction is one basis element, and the matrix element
\\(\mathrm{CF}_{ff'}\\) is the reduction of \\(\mathcal{C}_f\\) times the
conjugate of \\(\mathcal{C}_{f'}\\) with \\(N_c = 3\\) substituted.

A colourless process has one basis element and \\(\mathrm{CF} = [1]\\).
\\(u\bar u \to u\bar u\\) has two flows; \\(gg \to gg\\) has six, and its
matrix has a large automorphism group, which is the reason colour is
validated below the \\(|\mathcal{M}|^2\\) level.

## Colour flows for events

The trace basis is what the cross section needs. An event record needs
something else: a *colour flow*, a pairing of every coloured leg's colour
line with an anticolour line, which is what a parton shower reads to decide
where to radiate. At leading order in \\(1/N_c\\) each basis element is a
definite set of such lines ([Maltoni et al. 2003](../bibliography.md#colour)),
so a flow is chosen per event by drawing a basis element with probability
proportional to its diagonal weight \\(\sum_h |J_f|^2\\), MadGraph's `JAMP2`
and `SELECT_COLOR`. The `(colour, anticolour)` labels per leg are read off
the basis element's canonical form and become the `ICOLUP` columns of the
Les Houches record ([event files](11-unweighting-events.md)). The
dictionary from flow to labels is checked against the `leshouche.inc` file
MadGraph generates for the same process.

## What each oracle can and cannot see

Colour is where the project's validation principle was learned the hard
way. The colour matrix is a Gram matrix, invariant under transposing every
index consistently, so an oracle comparing \\(\mathrm{CF}\\) against
MadGraph's passed while a conjugation-convention bug flipped physical
signs. \\(|\mathcal{M}|^2\\) is blind to a global phase on every flow.
Per-flow complex JAMP values, compared against banked MadGraph values up to
one fitted unit-modulus phase per process, are what finally see both, and
per-flow colour-line connectivity is what sees a swapped pair of flows
whose JAMPs happen to coincide. The [validation](12-validation.md) chapter
generalises the lesson.

## Where it lives

[`vibegraph::helas::color`](../api/vibegraph/helas/color/index.html):
[`tensor`](../api/vibegraph/helas/color/tensor/index.html) and
[`coeff`](../api/vibegraph/helas/color/coeff/index.html) for the atoms and
exact coefficients, [`factor`](../api/vibegraph/helas/color/factor/index.html)
for the rewrite system and canonical forms,
[`colorize`](../api/vibegraph/helas/color/colorize/index.html) for the walk
from diagrams to a basis and matrix, and
[`flow_tags`](../api/vibegraph/helas/color/flow_tags/index.html) for the
Les Houches labels. The evaluator's
[`cf_matrix`](../api/vibegraph/helas/eval/struct.AmplitudeEvaluator.html#method.cf_matrix)
and [`select_color_flow`](../api/vibegraph/helas/eval/struct.AmplitudeEvaluator.html#method.select_color_flow)
expose the results.
