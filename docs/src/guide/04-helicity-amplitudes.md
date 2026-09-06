# Helicity amplitudes

Textbook cross-section calculations square the amplitude analytically:
\\(|\mathcal{M}|^2\\) becomes a trace over Dirac matrices, the trace is
evaluated with Casimir's trick, and the result is a polynomial in the
momenta. For two diagrams the interference terms already double the work;
for the dozens or hundreds of diagrams of a multi-leg process, the
\\(N^2\\) growth makes the method hopeless. The **helicity amplitude**
method sidesteps it. Fix the helicity of every external particle, compute
the complex number \\(\mathcal{M}(h_1,\dots,h_n)\\) numerically as a sum
over diagrams, square that one number, and sum the squares over helicity
combinations:

\\[
\sum_{\text{spins}} |\mathcal{M}|^2 = \sum_{h_1 \dots h_n} \Big| \sum_{d} \mathcal{M}_d(h_1,\dots,h_n) \Big|^2 .
\\]

The cost is linear in the number of diagrams, and all interference is
included automatically because the diagrams are added before squaring.

## HELAS

HELAS ([Murayama, Watanabe & Hagiwara 1992](../bibliography.md#helicity-amplitudes))
organises the computation of one diagram as a chain of three kinds of
routine.

**Wavefunctions** turn an external momentum and helicity into a numerical
object: a four-component spinor \\(u(p,h)\\) or \\(v(p,h)\\) for a fermion
(`ixxxxx` for flowing-in, `oxxxxx` for the flowing-out conjugate
\\(\bar u, \bar v\\)), a polarisation four-vector \\(\epsilon^\mu(p,h)\\) for
a vector boson (`vxxxxx`), a bare 1 for a scalar (`sxxxxx`). Each carries
its momentum along, since the routines downstream need it.

**Off-shell currents** contract all but one leg of a vertex with the
wavefunctions already known, and multiply by the propagator of the
remaining leg. The result has the same shape as an external wavefunction of
that leg, a spinor or a vector with a momentum, so it can be fed into the
next vertex exactly as an external one would be. HELAS names these by
their output: `jioxxx` builds a vector current from a fermion pair,
`fvixxx` a fermion current from a fermion and a vector.

**Amplitudes** contract every leg of the last vertex and return the complex
number \\(\mathcal{M}_d\\).

A diagram is evaluated from the leaves inward: wavefunctions for every
external leg, currents for every internal line working toward one chosen
vertex, and one amplitude call at that vertex. The choice of which vertex is
last is free, and each choice produces a different set of intermediate
currents; the [compiler](06-compiler.md) chapter returns to this under the
name *rooting*, because it is what decides how much work diagrams can share.

Conventions matter enormously here, since the numbers have to agree with a
reference bit for bit and a spinor's phase is convention-laden. vibegraph
adopts HELAS's Appendix A throughout: Dirac matrices in the Weyl (chiral)
basis, where the upper two spinor components are left-chiral and
\\(\gamma^5 = \mathrm{diag}(-1,-1,1,1)\\); massless spinors defined with the
usual reference-momentum prescription; polarisation vectors with HELAS's
signs. MadGraph's helicity labels are those of
[Buarque Franzosi et al. 2020](../bibliography.md#helicity-amplitudes).

## ALOHA: vertices from Lorentz structures

HELAS is a fixed library of vertex routines for the Standard Model. A UFO
model can contain any Lorentz structure, so MadGraph 5 replaced the library
with ALOHA ([de Aquino et al. 2012](../bibliography.md#helicity-amplitudes)),
which reads each UFO Lorentz structure, multiplies it symbolically by
wavefunctions and propagators for each choice of output leg, and writes the
resulting expression out as a Fortran, C++ or Python routine per vertex and
per output leg.

vibegraph plays the same role without generating source code. A UFO Lorentz
structure such as `Gamma(3,2,-1)*ProjM(-1,1)` is a small tensor network:
operators with spinor and Lorentz index slots, some slots attached to legs,
the rest contracted between operators. Choosing an output leg fixes which
indices are free, and the structure can then be *rooted* into a tree of
contractions whose leaves are the input wavefunctions and whose root
produces the output current. Each contraction is a primitive the crate
implements once, generically: \\(\gamma^\mu\\) acting on a spinor, a chiral
projector, a metric contraction, a momentum insertion, the Dirac bilinear
\\(\bar u \Gamma v\\). The same handful of primitives, composed by the
structure, reproduces every ALOHA routine the Standard Model needs,
including the fused chiral forms MadGraph uses for the FFV vertex.

The propagators are HELAS's: \\((\not q + m)/(q^2 - m^2 + i m \Gamma)\\) for
a fermion, \\(-g^{\mu\nu}/q^2\\) for a massless vector, the unitary-gauge
numerator \\(-(g^{\mu\nu} - q^\mu q^\nu/m^2)/(q^2 - m^2 + i m\Gamma)\\) for
a massive one, with the fixed-width Breit–Wigner denominator MadGraph uses
at tree level.

## Representations, in the type system

The primitives are written once over an abstract scalar field `F` and over
representation types that know their own transformation behaviour, in the
`helas::repr` layer. A Lorentz vector carries a phantom type recording
whether it is contravariant or covariant, so the metric is applied exactly
where an index is lowered and nowhere else. A Dirac wavefunction carries a
phantom type for its flow (a ket \\(u\\), \\(v\\) or a bra
\\(\bar u\\), \\(\bar v\\)), so a vertex cannot be handed two kets. The types
cost nothing at run time: they exist so that a convention error is a
compile error rather than a wrong sign discovered by a failing cross-check.

The scalar field is a trait, not `f64`. Choosing `F = f64` gives the scalar
evaluator; choosing an \\(N\\)-lane SIMD array evaluates \\(N\\) phase-space
points per pass with identical arithmetic per lane. The
[compiler](06-compiler.md#simd-lanes) chapter says what that requires.

## Fermion signs and crossing

Two bookkeeping rules complete the amplitude. The relative sign between
diagrams that differ by an odd permutation of external fermions, the
**Fermi sign**, is read off the diagram enumeration. And because
[enumeration](03-diagrams.md#enumeration) presents every leg as incoming,
an outgoing fermion arrives bound to the UFO slot of its antiparticle; the
crossing relation \\(\bar u_1 \Gamma v_2 = -\bar u_2 (C\Gamma^T C^{-1}) v_1\\)
makes this exact for vector structures and conjugates the chiral projector
for chiral ones. The rooting carries an explicit `crossed` bit per fermion
line so the right form is chosen.

## Helicity filtering

Most helicity combinations of a chiral theory vanish identically: a massless
fermion line must conserve chirality, and combinations that violate it are
exactly zero. MadGraph evaluates every combination at a few points, drops
the ones that came out zero, and generates code for the survivors only.
vibegraph does the same, with a probe designed so that the pruned sum is bit
for bit the unpruned one; the [compiler](06-compiler.md#helicity-pruning)
chapter has the details and the survivor counts, which match MadGraph's
generated tables exactly.

## Where it lives

[`vibegraph::helas`](../api/vibegraph/helas/index.html):
[`repr`](../api/vibegraph/helas/repr/index.html) for the scalar trait,
Lorentz vectors, bispinors, propagators and intertwiners;
[`wavefn`](../api/vibegraph/helas/wavefn/index.html) for the external
wavefunctions and the conventions, with LaTeX in its doc comments;
[`vertex`](../api/vibegraph/helas/vertex/index.html) for a few hand-written
HELAS routines used as oracles in tests; and
[`eval`](../api/vibegraph/helas/eval/index.html) for the general evaluator
built from Lorentz structures, which the next two chapters describe.
