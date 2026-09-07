# Phase space

The integral in the [cross-section formula](01-pipeline.md#the-cross-section)
runs over the **Lorentz-invariant phase space** of the final state,

$$
d\Phi_n = (2\pi)^4\, \delta^4\Big(P - \sum_{i=1}^n p_i\Big)\; \prod_{i=1}^n \frac{d^3 p_i}{(2\pi)^3\, 2E_i},
$$

a measure over $n$ on-shell momenta with fixed total momentum
$P$. It has $3n - 4$ dimensions, and the integrand it multiplies,
$|\mathcal{M}|^2$, is far from flat: it peaks where a propagator goes
nearly on shell, by many orders of magnitude for a narrow resonance or a
massless exchange. A Monte Carlo integration lives or dies on how well its
sampling density follows those peaks, so the design of the maps from
random numbers to momenta is where most of an event generator's
performance is decided.

## Maps from the unit hypercube

Every sampler here has the same shape: a point $u$ in the unit
hypercube $[0,1]^d$ goes in, and $n$ momenta plus a weight come out,
the weight being the Jacobian that makes a flat average over $u$ an
estimate of $\int d\Phi_n$. That shape is a deliberate seam. The
[VEGAS](08-vegas.md) integrator only ever sees a function on the unit
hypercube, so an adaptive grid composes in front of any map without
knowing what the map does, and maps can be swapped or combined without the
integrator changing.

For a $2\to2$ massless process the only kinematic degree of freedom is
the scattering angle, $d\Phi_2 = d\cos\theta / (16\pi)$, and the
linear map $\cos\theta = 2u - 1$ is the whole story. The general case
needs more.

## RAMBO: flat $n$-body phase space

RAMBO ([Kleiss, Stirling & Ellis 1986](../bibliography.md#phase-space-and-integration))
generates $n$ massless momenta uniformly over phase space from
$4n$ uniform numbers: draw $n$ isotropic massless vectors with
energies distributed as
[$\Gamma(2)$](https://en.wikipedia.org/wiki/Gamma_distribution), boost and
scale the set so it sums to $(\sqrt{\hat s}, \vec 0)$, and attach the
analytically known volume as the weight. Masses are restored by a Newton
solve for one common rescaling of the three-momenta, with the corresponding
Jacobian. It is exact, simple and generic, and it is the sampler of last
resort: it has no idea where the peaks are, so on a resonant process its
variance is enormous. vibegraph uses it for probing (the
[helicity pruning](06-compiler.md#helicity-pruning) points) and as the flat
baseline the multichannel maps are measured against.

<details>
<summary>The construction in full</summary>

**Massless.** Four uniforms per momentum build one massless vector $q_i$:
$\cos\theta_i = 2r_1 - 1$ and $\varphi_i = 2\pi r_2$ give an isotropic
direction, and $q_i^0 = -\ln(r_3 r_4)$ gives an energy distributed as
$\Gamma(2)$, the sum of two independent
[exponentials](https://en.wikipedia.org/wiki/Exponential_distribution). The
ensemble density is then $\prod_i d^4q_i\, \delta(q_i^2)\, \theta(q_i^0)\,
e^{-q_i^0}$, which is isotropic and factorised but sums to some arbitrary
$Q = \sum_i q_i$ rather than to the wanted total.

One boost and one scale fix that. With $M = \sqrt{Q^2}$,
$\vec b = -\vec Q / M$, $\gamma = Q^0/M$ and $x = \sqrt{\hat s}/M$,

$$
p_i^0 = x\,\big(\gamma\, q_i^0 + \vec b \cdot \vec q_i\big),
\qquad
\vec p_i = x\,\Big(\vec q_i + \Big[q_i^0 + \frac{\vec b \cdot \vec q_i}{1+\gamma}\Big]\vec b\Big),
$$

and the $p_i$ sum exactly to $(\sqrt{\hat s}, \vec 0)$. The point of the
construction is that this map's Jacobian does not depend on the
configuration, so the $p_i$ are distributed uniformly over the invariant
volume and every point carries the same weight,

$$
R_n = \int \prod_{i=1}^n \frac{d^3 p_i}{2E_i}\; \delta^4\Big(P - \sum_i p_i\Big)
    = \Big(\frac{\pi}{2}\Big)^{n-1} \frac{\hat s^{\,n-2}}{(n-1)!\,(n-2)!}.
$$

A flat integrand therefore has exactly zero variance under massless RAMBO.
The $2\pi$ factors of the full $d\Phi_n$ measure are not in this weight;
they live in the cross-section prefactor.

**Massive.** Masses are restored by scaling every three-momentum by one
common factor $\xi$,

$$
k_i = \Big(\sqrt{m_i^2 + \xi^2 |\vec p_i|^2},\; \xi\, \vec p_i\Big).
$$

Three-momentum conservation survives because $\sum_i \vec p_i = 0$ scales
with it, so energy conservation is the single remaining condition and it
fixes $\xi$:

$$
\sum_{i=1}^n \sqrt{m_i^2 + \xi^2 |\vec p_i|^2} = \sqrt{\hat s}.
$$

The left side is monotone in $\xi$, so the root is unique and Newton from
the massless-limit guess $\xi_0 = \sqrt{1 - (\sum_i m_i / \sqrt{\hat s})^2}$
converges. Writing $f(\xi)$ for the difference of the two sides,
$f'(\xi) = \xi \sum_i |\vec p_i|^2 / E_i$, so the update is

$$
\xi \;\leftarrow\; \xi - \frac{f(\xi)}{\xi \sum_i |\vec p_i|^2 / E_i},
$$

iterated until $|f| < 10^{-13}\sqrt{\hat s}$, with a hundred steps as a cap
it does not approach. The rescaling is not volume preserving, so the weight
picks up its Jacobian, evaluated on the rescaled momenta:

$$
R_n \cdot
\Big(\frac{\sum_i |\vec k_i|}{\sqrt{\hat s}}\Big)^{2n-3}
\Big(\sum_i \frac{|\vec k_i|^2}{E_i}\Big)^{-1}
\sqrt{\hat s} \; \prod_i \frac{|\vec k_i|}{E_i},
$$

which reduces to $1$ in the massless limit. Unlike $R_n$ this factor
depends on the configuration, so massive RAMBO is not flat, only unbiased.

**Uniforms in, momenta out.** The map takes its $4n$ uniforms as an
argument rather than an RNG. That keeps the generator swappable and makes
the map replayable against an external reference: feed the same uniforms
and the momenta must match.

</details>

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

- A timelike line with a finite width is drawn through the
  [Breit–Wigner](https://en.wikipedia.org/wiki/Relativistic_Breit%E2%80%93Wigner_distribution)
  substitution $s = m^2 + m\Gamma \tan\theta$, so the density follows
  $1/((s-m^2)^2 + m^2\Gamma^2)$, a
  [Cauchy distribution](https://en.wikipedia.org/wiki/Cauchy_distribution)
  in $s$, and the resonance is flat in $\theta$.
- A zero-width pole at or below the kinematic floor, the massless
  $\gamma^\ast$ of a lepton pair above all, has no width to regulate its
  rise. Its invariant is drawn
  [log-uniformly](https://en.wikipedia.org/wiki/Reciprocal_distribution) in
  $t = s - m^2$ above a floor, giving a density $\propto 1/t$, with a small
  linear piece covering the range below the floor so the map keeps the full
  kinematic support.[^logmap]
- A subsystem with no pole keeps a flat
  ([uniform](https://en.wikipedia.org/wiki/Continuous_uniform_distribution))
  draw over its kinematic range, whose lower edge is the subsystem's mass
  threshold raised to whatever floor the process's cuts imply.

A **spacelike** (t-channel) line is not a subsystem mass but a momentum
transfer $t \le 0$, and a diagram carrying spacelike lines is decomposed as
a *spine*: an ordered chain of peripheral emissions off one beam, each rung
emitting one blob against the momentum transfer the earlier rungs left
behind. A rung's polar angle is fixed by its sampled $t$, leaving only the
azimuth free, and the emitted and remainder subsystems recurse into the
same two-body machinery. The transfer is importance-sampled too, with a
density $\propto 1/(m^2 - t)$, since the forward peak of a $t$-channel
exchange is as sharp as any resonance.[^spine]

[^logmap]: The map is two-piece in $t = s - m^2$. Above the floor
    $t_0 = \max\big(t_{\min},\, \min(10\ \mathrm{GeV}^2,\, t_{\max}/50)\big)$
    the draw is $t = t_0 (t_{\max}/t_0)^y$; the leading tenth of the random
    number covers $[t_{\min}, t_0]$ linearly, so the estimator stays
    unbiased, and when the kinematic edge already sits at or above the
    floor the logarithmic piece takes the whole draw. A zero-width pole
    strictly *inside* the range is a genuine singularity this map does not
    address, and the flat draw stands. The exponent is a choice, and $1/t$
    rather than $1/t^2$ is the right one: the squared propagator does
    contribute $1/(s-m^2)^2$ to $|\mathcal{M}|^2$, but the current a
    massless zero-width line couples to supplies a numerator vanishing
    linearly at the pole (the lepton tensor of a massless pair is
    proportional to the pair's invariant mass squared), and what is left is
    a $1/t$ rise, the $dm^2/m^2$ of a quasi-real photon. Uniform in $\ln t$
    makes that integrand exactly constant above the floor, which is the
    property the map is pinned on. The floor settles the rest: a massless
    pole has no kinematic lower edge, so $t_0$ is an invented number whose
    influence should be as weak as possible, and a $1/t$ draw normalises as
    $\ln t_0$ while a $1/t^2$ draw normalises as $1/t_0$ and would put half
    its points within a factor of two of that constant, starving the region
    above it where the cuts accept events. MadEvent reaches the same
    profile by pre-shaping the VEGAS grid into logarithmic bins, with the
    same floor and the same tenth of the bins reserved below it; vibegraph
    does it as an analytic map, which does not depend on a per-channel grid.

[^spine]: The substitution is $t = m^2 - (m^2 - t_{\min}) e^{-xN}$ with
    $N = \ln[(m^2 - t_{\min})/(m^2 - t_{\max})]$. A spacelike line has no
    width, so when the pole cannot shape the draw at all, a massless
    exchange whose window reaches the collinear edge $t_{\max} = m^2 = 0$,
    the draw falls back to flat and the rung reduces to an isotropic
    two-body split. That edge is also where the spine is regulated: rungs
    are bounded at $t \le -\Lambda$ for the fiducial scale $\Lambda$ the
    process's transverse-momentum cuts imply, which is where the cuts stop
    accepting anyway, and the pole location is floored at a small fraction
    of it so a near-degenerate window still draws against a pole far above
    the cancellation noise in $t$.

<figure>
<svg class="psfig" viewBox="0 0 860 630" width="100%" role="img" aria-labelledby="psTitle psDesc" xmlns="http://www.w3.org/2000/svg">
<title id="psTitle">Channel decomposition of a Feynman diagram into two-body decays</title>
<desc id="psDesc">Two panels. The upper panel shows an s-channel tree diagram for electron-positron annihilation into a muon pair and an electron pair, beside the nested chain of two-body decays it defines: the total system at fixed partonic energy splits into the muon-plus leaf and the subsystem of the remaining three legs, whose invariant is drawn with a logarithmic rise from the massless muon propagator; that subsystem splits into the muon-minus leaf and the electron pair, whose invariant is drawn with a Breit-Wigner from the Z propagator. The lower panel shows a t-channel spine: an electron beam emits an electron through a spacelike photon carrying momentum transfer t, whose value fixes the emission polar angle and leaves only the azimuth free, while the remainder recoils against the second beam and recurses into the same two-body split.</desc>
<style>
svg.psfig text { fill: currentColor; font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; }
svg.psfig .ps-pl { font-size: 13px; font-weight: 600; }
svg.psfig .ps-lb { font-size: 12px; }
svg.psfig .ps-sm { font-size: 11px; }
svg.psfig .ps-ln { stroke: currentColor; stroke-width: 1.4; fill: none; }
svg.psfig .ps-ds { stroke: currentColor; stroke-width: 1.4; fill: none; stroke-dasharray: 6 4; }
svg.psfig .ps-tn { stroke: currentColor; stroke-width: 0.9; fill: none; }
svg.psfig .ps-rule { stroke: currentColor; stroke-width: 0.8; fill: none; stroke-opacity: 0.35; }
svg.psfig .ps-dt { fill: currentColor; }
</style>
<text class="ps-pl" x="20" y="24">s-channel: the propagator tree is a chain of two-body decays</text>
<path class="ps-ln" d="M40,80 L140,150 M40,220 L140,150 M140,150 L215,150 M215,150 L380,88 M215,150 L275,205 M275,205 L380,162 M275,205 L320,262 M320,262 L380,234 M320,262 L380,296"/>
<circle class="ps-dt" cx="140" cy="150" r="3"/><circle class="ps-dt" cx="215" cy="150" r="3"/>
<circle class="ps-dt" cx="275" cy="205" r="3"/><circle class="ps-dt" cx="320" cy="262" r="3"/>
<text class="ps-lb" x="22" y="76">e⁺</text>
<text class="ps-lb" x="22" y="232">e⁻</text>
<text class="ps-sm" x="177" y="140" text-anchor="middle">γ*/Z*</text>
<text class="ps-sm" x="238" y="186" text-anchor="end">µ</text>
<text class="ps-sm" x="290" y="243" text-anchor="end">Z</text>
<text class="ps-lb" x="386" y="88">µ⁺</text>
<text class="ps-lb" x="386" y="166">µ⁻</text>
<text class="ps-lb" x="386" y="238">e⁺</text>
<text class="ps-lb" x="386" y="300">e⁻</text>
<path class="ps-tn" d="M426,80 V150 M426,108 H452 M426,146 H452 M450,158 V226 M450,184 H476 M450,222 H476 M474,234 V292 M474,260 H500 M474,288 H500"/>
<text class="ps-lb" x="432" y="74">{µ⁺ µ⁻ e⁺ e⁻}   s = ŝ, fixed by the beams, not drawn</text>
<text class="ps-lb" x="456" y="112">µ⁺   leaf, fixed mass</text>
<text class="ps-lb" x="456" y="150">{µ⁻ e⁺ e⁻}   from the µ line: log-rise in s − m²</text>
<text class="ps-lb" x="480" y="188">µ⁻   leaf, fixed mass</text>
<text class="ps-lb" x="480" y="226">{e⁺ e⁻}   from the Z line: Breit–Wigner</text>
<text class="ps-lb" x="504" y="264">e⁺   leaf, fixed mass</text>
<text class="ps-lb" x="504" y="292">e⁻   leaf, fixed mass</text>
<path class="ps-rule" d="M20,322 H840"/>
<text class="ps-pl" x="20" y="346">t-channel: a spine of peripheral emissions</text>
<path class="ps-ln" d="M40,420 L150,420 M150,420 L322,366 M40,570 L255,520 M255,520 L315,520 M315,520 L405,480 M315,520 L405,565"/>
<path class="ps-ds" d="M150,420 L255,520"/>
<path class="ps-tn" d="M195,420 A45,45 0 0,0 193.0,406.3"/>
<circle class="ps-dt" cx="150" cy="420" r="3"/><circle class="ps-dt" cx="255" cy="520" r="3"/>
<circle class="ps-dt" cx="315" cy="520" r="3"/>
<text class="ps-lb" x="40" y="410">e⁻   beam 0, along +z</text>
<text class="ps-lb" x="40" y="586">q   beam 1</text>
<text class="ps-sm" x="203" y="414">θ</text>
<text class="ps-sm" x="188" y="478" text-anchor="end">γ*</text>
<text class="ps-sm" x="285" y="512" text-anchor="middle">q</text>
<text class="ps-lb" x="328" y="364">e⁻   emitted blob B₁</text>
<text class="ps-lb" x="411" y="480">q</text>
<text class="ps-lb" x="411" y="569">g</text>
<text class="ps-lb" x="432" y="392">rung 1: emitted blob B₁ = {e⁻}, anchored to beam 0</text>
<text class="ps-sm" x="456" y="416">t = (p_beam0 − p_B₁)² ≤ 0, drawn with density ∝ 1/(m² − t)</text>
<text class="ps-sm" x="456" y="438">t = m² − (m² − t_min)·exp(−xN),  N = ln[(m² − t_min)/(m² − t_max)]</text>
<text class="ps-sm" x="456" y="460">θ is fixed by t; only the azimuth φ is free</text>
<text class="ps-lb" x="432" y="496">remainder R₁ = recoil {q g}</text>
<text class="ps-sm" x="456" y="520">its invariant comes from the q line: log-rise, then the same</text>
<text class="ps-sm" x="456" y="542">two-body split into q and g, isotropic</text>
<text class="ps-sm" x="432" y="578">a ladder repeats the rung: nested sides S₁ ⊂ S₂ ⊂ …, one t per line</text>
<text class="ps-sm" x="20" y="612">Draws: Breit–Wigner for a finite width, log-rise for a zero-width pole at or below the floor, flat for a line with no pole.</text>
</svg>
</figure>

The figure shows both decompositions on one example each. In the upper
panel the diagram is $e^+e^- \to \mu^+\mu^-e^+e^-$ through an s-channel
$\gamma^\ast/Z^\ast$ that radiates a $Z$ off the muon line, and beside it is
the decay chain its two internal lines define. The root is the whole final
state $\{\mu^+\mu^-e^+e^-\}$ at $s = \hat s$, fixed by the beams and not
drawn. The first split peels off the $\mu^+$ as a leaf of fixed mass and
leaves the subsystem $\{\mu^-e^+e^-\}$, whose invariant mass is the momentum
flowing through the internal muon line: a zero-width pole sitting below the
subsystem's threshold, so that invariant is drawn with the log-rise map in
$s - m^2$. That subsystem splits in turn into the $\mu^-$ leaf and the pair
$\{e^+e^-\}$, whose invariant is the $Z$ propagator's, drawn with the
Breit–Wigner substitution. Every node of the chain is one of the three
draws in the legend, and the tree's shape is read off the propagators
alone. In the lower panel the diagram is $e^-q \to e^-qg$ with a spacelike
photon, and the decomposition is a spine rather than a tree. The first
rung anchors to beam 0, the electron along $+z$, and emits the blob
$B_1 = \{e^-\}$ against the momentum transfer $t = (p_{\text{beam}\,0} -
p_{B_1})^2 \le 0$, drawn with the $1/(m^2 - t)$ map written under it; once
$t$ is drawn the emission's polar angle $\theta$ is fixed and only the
azimuth is free. What is left, the remainder $R_1 = \{qg\}$, recoils against
beam 1 and re-enters the timelike machinery of the upper panel: its
invariant comes from the internal quark line, a log-rise again, and its
final split into $q$ and $g$ is an isotropic two-body decay. A diagram with
more spacelike lines repeats the rung as a ladder, one $t$ per line, with
the emitted sides nested inside one another.

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
$\prod_s n_s!$, applied per subprocess since it depends on the outgoing
multiset, not on the matrix element.

## Random numbers

Sampling is defined in two decoupled layers. A counter-based generator,
ChaCha8, produces the integer bit stream; because it is counter-based, a
`(stream, position)` pair names an exact location in its output, with
$2^{64}$ independent streams per seed. A documented conversion turns
each 64-bit draw into a
[uniform](https://en.wikipedia.org/wiki/Continuous_uniform_distribution)
in $[0,1)$ in the scalar type. Two
consequences follow. Parallel integration assigns
`stream = (iteration, chunk)` and is bit-identical at any thread count. And
a lane-batched evaluation reproduces the scalar one exactly, because the
same integer draw feeds every lane through the same arithmetic.

## Frames

A matrix element is evaluated in the partonic centre-of-mass frame with the
beams along $\pm z$, which is the frame the pruned evaluator requires
and in which $|\mathcal{M}|^2$ is an invariant. The
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
