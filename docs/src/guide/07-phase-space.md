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

The decay *angle* of each two-body split is isotropic in the parent's rest
frame, with one exception. A split with a massless vector, a gluon or a
photon, as one daughter carries a splitting kernel's soft enhancement,
$1/z$ or $1/(z(1-z))$ in the daughters' energy fractions, which an isotropic
draw leaves in the weight. Such a split measures its angle from the parent's
direction of flight and draws it with a density $\propto 1/(E_1 E_2)$ in the
daughters' collision-frame energies, which is $dz/(z(1-z))$, confined to the
angles at which both daughters clear the energy the cuts imply for them. A
parent at rest has no direction of flight, and the map is the isotropic one
there.

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

### Map choices

A decomposition leaves a few things open that are choices of map rather
than of physics: every option below is a different parametrisation of the
same phase space, so the estimator is unbiased under any of them and what
moves is the variance of the weight — the evaluations a run needs to reach
an accuracy. `vibegraph integrate` exposes each as a `--map-*` flag whose
default, `auto`, is a rule that reads the process; the settled choices are
banked in the artifact and `generate` rebuilds its channels from those, so an
event sample is always drawn from the maps its grids were trained on.

- **The two-body decay angle** (`--map-split-angle`). The isotropic draw
  above is flat in $\cos\theta$ against the collision-CM axes. A massless
  emission's splitting kernel goes like $1/z$ or $1/(z(1-z))$ in the
  parent's energy fraction $z = E_1/E$, and since $E_1 E_2 = E^2 z(1-z)$ a
  density $\propto 1/(E_1E_2)$ in the angle measured from the parent's
  direction of flight is exactly $dz/(z(1-z))$, regulated at both ends by
  the pair's own mass through $\beta < 1$. `soft-emission` applies that map
  to the splits with a single gluon or photon daughter, and `soft-all` to
  every split whose parent moves. Either shape is confined to the angles at
  which both daughters clear the energy floor the cuts imply, because a
  $1/E$ map left to run down to the kinematic edge spends most of its draws
  below the $p_T$ threshold that rejects them; `windowed` is that
  confinement alone. `auto` picks `soft-emission` where there is such a
  split, a third fewer evaluations on $u\bar u \to ggg$. `soft-all`
  measured best everywhere it was tried — half the evaluations on
  $pp \to \ell^+\ell^- j$, where the shaped split is the lepton pair's — and
  is the one to ask for there. A $2 \to 2$ process has no split whose parent
  moves, so there every choice is the isotropic map.
- **The $\tau = \hat s/s$ draw** of a proton-beam run (`--map-tau`): the
  logarithmic map of the [hadronic chapter](10-hadronic.md), or MadEvent's
  $1/\tau^2$. `auto` keeps the logarithm. MadEvent takes $1/\tau^2$ unless a
  resonance spans the whole final state, whose peak then sits in $\tau$;
  here that measures a quarter fewer evaluations on dijets and slightly
  more on Drell–Yan, and is available as `inverse-square`.
- **The order of a ladder's rungs** (`--map-rung-order`): as the diagram's
  spacelike lines nest outward from beam 0, or reversed, which exists to be
  measured against and measures the same.

MadEvent leaves the decay angle to its adaptive grid — its `one_tree` draws
$\cos\theta$ and $\phi$ flat too — and shapes its invariants partly with the
same analytic transforms used here and partly by pre-warping the grid; note
37 in the research notes lists every map it applies against these.

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

## Fixed beams

A run with `lpp = 0` collides the incoming particles themselves rather than
partons out of a hadron, so there is no $\tau$ sampling and no
[luminosity](10-hadronic.md): the initial state is fixed by the run card's
two beam energies and the two beams' own pole masses. Everything the initial
state contributes follows from those four numbers: the partonic invariant
$\hat s$ (which is $(E_a + E_b)^2$ only for massless beams of equal energy),
the centre-of-mass beam momenta the matrix element and the channel maps are
built on, the flux factor of the
[cross-section formula](01-pipeline.md#the-cross-section) — the Møller
invariant $F = 2\,\lambda^{1/2}(\hat s, m_a^2, m_b^2)$, which reduces to
$2\hat s$ for massless beams and is a few per cent smaller for beams of
tens of GeV at hundreds — and the boost between the laboratory frame the
[cuts](10-hadronic.md#cuts) are applied in and the centre-of-mass frame the
momenta are generated and recorded in. Deriving all four from one initial
state is what keeps the beams the amplitude sees, the beams the maps sample
and the $\sqrt{\hat s}$ the outgoing map is built on from ever disagreeing.

<details>
<summary>The kinematics in full</summary>

**The invariant.** In the laboratory the beams are on shell along $\pm z$ at the energies the
card gives,

$$
p_a = \big(E_a,\, 0,\, 0,\, +\sqrt{E_a^2 - m_a^2}\big), \qquad
p_b = \big(E_b,\, 0,\, 0,\, -\sqrt{E_b^2 - m_b^2}\big),
$$

and the partonic invariant is the square of their sum,

$$
\hat s = (p_a + p_b)^2 = m_a^2 + m_b^2 + 2\big(E_a E_b + |\vec p_a|\,|\vec p_b|\big).
$$

This is $(E_a + E_b)^2$ only when both beams are massless *and* carry equal
energy; two massless beams of unequal energy collide at $4E_aE_b$, and two
60 and 70 GeV beams at 250 GeV each collide at $\sqrt{\hat s} = 499.99275$ GeV
rather than 500.

The matrix element and the channel maps work in the partonic centre of mass,
where the two beams share one momentum magnitude and split the energy by
their masses:

$$
E_a^\ast = \frac{\hat s + m_a^2 - m_b^2}{2\sqrt{\hat s}}, \qquad
E_b^\ast = \frac{\hat s - m_a^2 + m_b^2}{2\sqrt{\hat s}}, \qquad
|\vec p^{\;\ast}| = \frac{\lambda^{1/2}(\hat s, m_a^2, m_b^2)}{2\sqrt{\hat s}},
$$

with the [Källén function](https://en.wikipedia.org/wiki/K%C3%A4ll%C3%A9n_function)
$\lambda(x,y,z) = x^2 + y^2 + z^2 - 2xy - 2yz - 2zx$. The same $\sqrt{\hat s}$
is what the outgoing map is built on, so the external set the amplitude is
handed conserves four-momentum by construction; deriving the beams and the map
from one initial state is what keeps it that way.

**The flux.** The flux factor $F$ of the
[cross-section formula](01-pipeline.md#the-cross-section) is the
[Møller](https://en.wikipedia.org/wiki/Luminosity_(scattering_theory)#Relativistic_form)
invariant, the relative-velocity factor written covariantly:

$$
F = 4\sqrt{(p_a\!\cdot\!p_b)^2 - m_a^2 m_b^2}
  = 2\,\lambda^{1/2}(\hat s, m_a^2, m_b^2)
  = 4\,|\vec p^{\;\ast}|\sqrt{\hat s}.
$$

For massless beams $\lambda^{1/2}(\hat s, 0, 0) = \hat s$ and this is the
familiar $2\hat s$. For massive ones it is *smaller*, by

$$
1 - \frac{F}{2\hat s} = \frac{m_a^2 + m_b^2}{\hat s} + O\!\left(\frac{m^4}{\hat s^2}\right),
$$

3.46% on the 60/70 GeV pair above — a deficit that goes straight onto
$\sigma = F^{-1}\!\int d\Phi_n\,|\mathcal{M}|^2$ if the massless form is used
where it does not apply.

**Two frames, one event.** The laboratory and the partonic centre of mass differ by a boost along $z$ of
rapidity

$$
y_{\rm cm} = \tfrac12 \ln\frac{p^0 + p^3}{p^0 - p^3}, \qquad
p^0 = E_a + E_b, \quad p^3 = |\vec p_a| - |\vec p_b|,
$$

which vanishes for beams of equal energy and mass and is $5.4\times10^{-3}$ for
the 60/70 GeV pair. That is the whole difference between the frame the
[cuts](10-hadronic.md#cuts) are applied in and the frame the momenta are
generated and recorded in: rapidity is not invariant under it, so a
configuration is carried into the laboratory before the cut filter reads it,
while the event record keeps the centre-of-mass momenta.

</details>

> **MadGraph compatibility.** `genps.f` forms exactly this
> $\hat s$ (`stot = m1**2 + m2**2 + 2*(pi1(0)*pi2(0) - pi1(3)*pi2(3))`, the
> second beam's $p_z$ negative), takes the flux as
> `1d0/(2d0*SQRT(LAMBDA(s, m(1)**2, m(2)**2)))`, and generates in the partonic
> centre of mass. Rather than boosting, it shifts: `genps.f` stores the beam
> system's rapidity as `cm_rap` and `kin_functions.f`'s `rap()` adds it to every
> rapidity a cut reads. The two agree on every cut in this crate's filter —
> transverse momentum, invariant masses and rapidity *differences* are all
> invariant under a $z$ boost, and the single-leg rapidity picks up exactly
> $y_{\rm cm}$ either way. They part only on a single-leg *energy* threshold,
> which MadGraph compares against the centre-of-mass energy; no banked run card
> sets one.

## Decays at rest

A proc card with one initial particle, `generate t > b e+ ve`, is a
$1 \to n$ decay, and what it measures is the particle's **partial width**
into that final state rather than a cross section:

$$
\Gamma = \frac{1}{2M}\,\frac{1}{(2s+1)\,N_c}\int d\Phi_n\;S\,\sum|\mathcal{M}|^2,
$$

the decaying particle of pole mass $M$ at rest, $(2s+1)N_c$ its spin and
colour states averaged over, $d\Phi_n$ the phase space of the products at
total momentum $(M, 0, 0, 0)$, and $S$ the final state's identical-particle
factor. The flux $F$ of a scattering becomes $2M$, and the invariant the
outgoing map is built on becomes $M$; nothing else in the integrand changes,
and the same [channel maps](#channels-from-diagrams) serve it: with one
incoming leg every internal line is an s-channel, so each channel is the
all-timelike tree. `vibegraph integrate` reports the width in GeV
(`Γ = … GeV`), whatever the beams on the run card say.

MadEvent reads the run card of a decay run its own way, and so does this
crate. With no run card, the default is MadGraph's decay default: the LO
card with every cut removed. With one, its cuts are honoured, applied to the
products in the rest frame (MadEvent's `cuts.f` has no frame to boost to
with one incoming particle), except the $\hat s$ window, which `cuts.f` reads
only for two. The renormalisation scale is the particle's mass unless the
card fixes one, and the factorisation scales are the card's constants —
`setcuts.f`'s `nincoming = 1` branch, which also switches the parton
densities off — so no dynamical scale prescription is ever evaluated for a
decay.

> **What is validated.** Every open two-body width of the SM UFO's own
> `decays.py` (19 channels of $t$, $W^+$, $Z$ and $H$) is reproduced to
> $5\times10^{-13}$, and $\sum|\mathcal{M}|^2$ at fixed rest-frame points
> matches both those widths and MadGraph's standalone matrix element for the
> many-body decays, interfering diagrams included, to $10^{-12}$.
> `t > w+ b`, `t > b e+ ve`, `h > e+ e- mu+ mu-` and `z > e+ e-`, two of them
> again under lepton and $b$ cuts, are compared over ten seeds each against the
> exact width where one is known and against MadEvent's ten-seed mean
> otherwise (`vibegraph-cli/tests/cli_decay.rs`). `h > e+ e- mu+ mu-` has an
> exact width because its one diagram factorises into two off-shell $Z$ lines,
> integrated by quadrature; there this crate's seeds agree with it, and
> MadEvent's sit 0.14% below it at 10k events and 0.05% below at 100k,
> converging on it with budget.

## Decay chains

A decay chain, `e+ e- > t t~, t > w+ b, t~ > w- b~`, is a process whose
decaying legs are internal lines [forced on shell](03-diagrams.md). MadEvent
writes each as `gForceBW = 1`, and its cut routine (`cut_bw`, `myamp.f:76`)
rejects a point outright unless every forced line of the configuration it was
drawn in satisfies

$$
\bigl|\sqrt{p^2} - M\bigr| < \texttt{bwcutoff}\cdot\Gamma,
$$

with $\Gamma$ floored at $M\cdot\texttt{small\_width\_treatment}$ and a line of
zero width never tested (`myamp.f:164`–`185`); outside the window the point
fails `passcuts` and its matrix element is never evaluated. What is gated
against MadEvent is therefore the decay chain's own cross section, not
$\sigma\times\text{BR}$: the window keeps $\approx(2/\pi)\arctan(2\cdot
\texttt{bwcutoff})$ of each Breit–Wigner, about 0.979 at the default 15.

Here the window is a cut of the process, applied by the same filter as the
run card's cuts. MadEvent reads it per configuration, but on a card without
identical particles across its decays every configuration forces the same
lines, so the two agree. With identical particles across decays, where this
crate keeps every pairing ([Diagrams](03-diagrams.md)), different diagrams
force different leg sets, and a point passes when *every* line of *some*
diagram's set is inside its window — a predicate as symmetric under the
exchange of identical particles as the identical-particle factor needs, and
one that no choice of sampler can move.

The [channel maps](#channels-from-diagrams) confine each forced invariant to
its window: the line's Breit–Wigner substitution is drawn over the window's
intersection with the kinematic range rather than over the whole range, so no
point is spent where the cut rejects it. That narrows the channel's support,
and its density is exactly zero at a configuration whose forced invariant lies
outside — which matters on a card whose diagrams force different leg sets:
the integrand is non-zero wherever some set is inside its windows, and a
channel claiming density at points it cannot draw would bias the
[mixture](09-multichannel.md). Where the rest of a draw leaves the window out
of kinematic reach, the channel keeps its full range, and its density reads
the same range there. MadEvent reaches the same place by another route: it
raises the forced invariant's lower edge to $M - \texttt{bwcutoff}\cdot\Gamma$
(`set_peaks`, `myamp.f:397`) and lets its grid and the cut do the rest. The
forced windows also bound $\hat s$ from below for the $\tau$ draw of a proton
run: $\sqrt{\hat s}$ is at least the sum of the outermost forced lines' lower
edges and the remaining legs' masses.

`cut_decays` (default `F`) switches the run card's cuts off on every leg a
forced line produces (`setcuts.f:192`): its single-leg cuts and every
pairwise $\Delta R$ and invariant-mass cut it is part of; `ptll` and `mmnl`
are set without reading it and stay on. MadEvent reads the forced lines of
configuration 1; here a leg is a decay product when a forced line produces it
in every diagram, the same legs on a card without identical particles across
decays.

> **What is validated.** Five decay chains, each over ten seeds against five
> MadEvent runs on the same proc and run cards
> (`vibegraph-cli/tests/cli_decay_chain.rs`, references in
> `validation/madgraph/decay_chain_sigma_reference.json`):
> `e+ e- > z z, z > e+ e-, z > mu+ mu-` with `cut_decays` off and on,
> `e+ e- > t t~, t > w+ b, t~ > w- b~`, the nested
> `e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~`, and
> `p p > t t~, t > b e+ ve, t~ > b~ mu- vm~` at 13 TeV with fixed scales.
> All five sit within 1.5 combined standard errors of MadEvent's seed mean,
> with this crate's seeds at χ²/dof 0.8–1.9. On the `cut_decays = T` row
> MadEvent is the one that misses: its seeds scatter four times wider than they
> quote, low, and at 50k events it lands 0.06% from this crate's value.
> `e+ e- > z z, z > e+ e-`, with identical particles across its decays, is
> reported beside them, not gated: keeping both pairings and their interference
> puts it 0.26% ± 0.08% above MadGraph's one pairing over a factor 2.
>
> **Leg count.** Decaying the resonances in the matrix element does not stall
> the sampler — every rung of `tests/decay_chain_ladder.rs` converges — but it
> costs more than the extra matrix-element work: the unweighting efficiency
> falls from the core's by 2.6× on `e+ e- > z z` with both bosons decayed, 5.7×
> on `p p > t t~` with both tops decayed, and 16× on the fully decayed
> `p p > t t~ h h` (eight legs, 40 ms per unweighted event), while the decayed
> $|\mathcal{M}|^2$ costs about what the core's does. The resonances are
> mapped exactly; what the channels draw flat are the decay angles.

## Frames

The helicity-summed $|\mathcal{M}|^2$ is a Lorentz invariant and could be
evaluated in any frame. What fixes the frame is
[helicity pruning](06-compiler.md#helicity-pruning): the combinations an
evaluator drops were found to vanish in the partonic centre-of-mass frame
with the beams along $\pm z$, and some of them vanish only there, since
the helicity of a massive particle is not boost invariant. A pruned sum is
therefore the full sum only for momenta in that frame, and that is the
frame the matrix element is handed. The [cuts](10-hadronic.md#cuts) are
applied in the laboratory frame, whose rapidity and transverse observables
are not boost invariant, so a hadronic point is boosted before the cut
filter sees it.

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
