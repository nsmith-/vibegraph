# Phase space

The integral in the [cross-section formula](01-pipeline.md#the-cross-section)
runs over the **Lorentz-invariant phase space** of the final state,

\\[
d\Phi_n = (2\pi)^4\, \delta^4\Big(P - \sum_{i=1}^n p_i\Big)\; \prod_{i=1}^n \frac{d^3 p_i}{(2\pi)^3\, 2E_i},
\\]

a measure over \\(n\\) on-shell momenta with fixed total momentum
\\(P\\). It has \\(3n - 4\\) dimensions, and the integrand it multiplies,
\\(|\mathcal{M}|^2\\), is far from flat: it peaks where a propagator goes
nearly on shell, by many orders of magnitude for a narrow resonance or a
massless exchange. A Monte Carlo integration lives or dies on how well its
sampling density follows those peaks, so the design of the maps from
random numbers to momenta is where most of an event generator's
performance is decided.

## Maps from the unit hypercube

Every sampler here has the same shape: a point \\(u\\) in the unit
hypercube \\([0,1]^d\\) goes in, and \\(n\\) momenta plus a weight come out,
the weight being the Jacobian that makes a flat average over \\(u\\) an
estimate of \\(\int d\Phi_n\\). That shape is a deliberate seam. The
[VEGAS](08-vegas.md) integrator only ever sees a function on the unit
hypercube, so an adaptive grid composes in front of any map without
knowing what the map does, and maps can be swapped or combined without the
integrator changing.

For a \\(2\to2\\) massless process the only kinematic degree of freedom is
the scattering angle, \\(d\Phi_2 = d\cos\theta / (16\pi)\\), and the
linear map \\(\cos\theta = 2u - 1\\) is the whole story. The general case
needs more.

## RAMBO: flat \\(n\\)-body phase space

RAMBO ([Kleiss, Stirling & Ellis 1986](../bibliography.md#phase-space-and-integration))
generates \\(n\\) massless momenta uniformly over phase space from
\\(4n\\) uniform numbers: draw \\(n\\) isotropic massless vectors with
energies distributed as \\(\Gamma(2)\\), boost and scale the set so it sums
to \\((\sqrt{\hat s}, \vec 0)\\), and attach the analytically known volume
as the weight. Masses are restored by a single Newton solve for a rescaling
of the three-momenta, with the corresponding Jacobian. It is exact, simple
and generic, and it is the sampler of last resort: it has no idea where the
peaks are, so on a resonant process its variance is enormous. vibegraph
uses it for probing (the [helicity pruning](06-compiler.md#helicity-pruning)
points) and as the flat baseline the multichannel maps are measured
against.

## Channels from diagrams

MadEvent's central idea ([Maltoni & Stelzer 2003](../bibliography.md#the-pipeline-as-a-whole))
is that the diagrams already say where the peaks are. Each propagator is a
peak in one invariant, and a diagram's tree of propagators is a nested set
of subsystems of the final state. Reading the tree off the diagram's
momentum routing gives a **channel**: a phase-space map built as a chain of
two-body decays, the total system splitting into two daughters, each
daughter a single particle with a fixed mass or a subsystem with an
invariant mass that is sampled and then recursed into
([Byckling & Kajantie 1973](../bibliography.md#phase-space-and-integration)).

How each sampled invariant is drawn depends on what the diagram puts there:

- A timelike line with a finite width is drawn through the Breit–Wigner
  substitution \\(s = m^2 + m\Gamma \tan\theta\\), so the density follows
  \\(1/((s-m^2)^2 + m^2\Gamma^2)\\) and the resonance is flat in
  \\(\theta\\).
- A zero-width pole at or below the kinematic floor, the massless
  \\(\gamma^\ast\\) of a lepton pair above all, has no width to regulate a
  \\(1/(s - m^2)^2\\) rise. Its invariant is drawn logarithmically in
  \\(s - m^2\\) down to a floor.
- A subsystem with no pole keeps a flat draw over its kinematic range.

A **spacelike** (t-channel) line is not a subsystem mass but a momentum
transfer \\(t \le 0\\), and a diagram with one is decomposed as a *spine*: a
peripheral emission off a beam whose polar angle is fixed by \\(t\\), with
the emitted and recoil subsystems recursing into the same two-body
machinery. Its transfer is importance-sampled too, since the forward peak
of a \\(t\\)-channel exchange is as sharp as any resonance.

Leaving one of these peaks on a flat draw is not merely inefficient. The
estimator acquires a tail so heavy that a run either misses the region and
underestimates, or hits it and overestimates, and because
[VEGAS combines iterations](08-vegas.md#combining-iterations) by their
estimated variance, the iterations that miss report a small integral *and*
a small error and dominate the result. The failure is a confidently wrong
cross section with a small error bar, not a visibly noisy one. That failure
mode is the reason the [multichannel](09-multichannel.md) chapter's budget
rules never let a channel stop sampling its own region.

Identical particles in the final state divide the measure by
\\(\prod_s n_s!\\), applied per subprocess since it depends on the outgoing
multiset, not on the matrix element.

## Random numbers

Sampling is defined in two decoupled layers. A counter-based generator,
ChaCha8, produces the integer bit stream; because it is counter-based, a
`(stream, position)` pair names an exact location in its output, with
\\(2^{64}\\) independent streams per seed. A documented conversion turns
each 64-bit draw into a uniform in \\([0,1)\\) in the scalar type. Two
consequences follow. Parallel integration assigns
`stream = (iteration, chunk)` and is bit-identical at any thread count. And
a lane-batched evaluation reproduces the scalar one exactly, because the
same integer draw feeds every lane through the same arithmetic.

## Frames

A matrix element is evaluated in the partonic centre-of-mass frame with the
beams along \\(\pm z\\), which is the frame the pruned evaluator requires
and in which \\(|\mathcal{M}|^2\\) is an invariant. The
[cuts](10-hadronic.md#cuts) are applied in the laboratory frame, whose
rapidity and transverse observables are not boost invariant, so a hadronic
point is boosted before the cut filter sees it.

## Where it lives

[`vibegraph::phasespace`](../api/vibegraph/phasespace/index.html):
[`channel`](../api/vibegraph/phasespace/channel/index.html) for the
`PhaseSpaceMap` / `Channel` / `Combiner` seam and the
[`MultiChannel`](../api/vibegraph/phasespace/channel/struct.MultiChannel.html)
combiner, [`diagram_channel`](../api/vibegraph/phasespace/diagram_channel/index.html)
for the per-diagram maps, [`rambo`](../api/vibegraph/phasespace/rambo/index.html),
and [`rng`](../api/vibegraph/phasespace/rng/index.html). The flux and
averaging factors sit with the integrands in
[`hadronic`](../api/vibegraph/hadronic/index.html).
