# Event generation in one page

A leading-order event generator answers two questions about a scattering
process. How often does it happen, which is the **cross section**
$\sigma$? And what does an individual occurrence look like, which is an
**event**: a list of outgoing particles with their four-momenta, drawn with
the probability nature would assign it? Everything in this generator is in
service of one formula for the first and one sampling trick for the second.

## The cross section

For a $2 \to n$ process at fixed partonic energy, the cross section is
the squared scattering amplitude integrated over every configuration the
final state can take:

$$
\sigma = \frac{1}{F}\int d\Phi_n \;\overline{|\mathcal{M}(p_1,\dots,p_{n+2})|^2}.
$$

The pieces:

- $\mathcal{M}$ is the **matrix element**, the quantum-mechanical
  amplitude for the transition. At tree level it is a sum over the Feynman
  diagrams of the process, each a product of vertex factors, propagators and
  external wavefunctions. Computing it is the job of
  [diagram enumeration](03-diagrams.md), [helicity amplitudes](04-helicity-amplitudes.md)
  and the [colour algebra](05-color.md), with the
  [amplitude compiler](06-compiler.md) turning the result into fast code.
- The bar denotes averaging over the initial-state spins and colours and
  summing over the final-state ones: an unpolarised beam sees the average,
  and a detector sees every final spin state.
- $d\Phi_n$ is the **Lorentz-invariant phase space**, the measure over
  the $3n-4$ kinematic degrees of freedom of $n$ on-shell momenta with
  fixed total momentum. [Phase space](07-phase-space.md) describes it and
  the maps used to sample it.
- $F$ is the **flux factor**, $2\hat s$ for massless incoming
  partons, which normalises the amplitude to a rate per unit incident flux.

The integral has no closed form for anything but the simplest processes, so
it is done by Monte Carlo: draw random points in phase space, evaluate the
integrand at each, and average. [VEGAS](08-vegas.md) makes the draws
adaptive, and [multichannel sampling](09-multichannel.md) tailors the draws
to the propagator peaks each diagram contributes.

At a hadron collider the incoming particles are partons carrying fractions
$x_1, x_2$ of the proton momenta, with probabilities given by the parton
distribution functions. The hadronic cross section is the partonic one
convolved with those densities and summed over the parton flavours that can
initiate the process; [proton beams](10-hadronic.md) covers this, together
with the scale choices it introduces.

## Events

A Monte Carlo integration visits phase-space points with a density of its
own choosing and corrects for it with a weight. The visited points, each
carrying its weight, already form a *weighted* event sample. An
**unweighted** sample, in which every event counts the same and the density
of events is the physical one, is obtained by accept/reject: keep a point
with probability proportional to its weight. [Unweighting and event files](11-unweighting-events.md)
describes the procedure, the maximum-weight estimate it rests on, and the
Les Houches file format the events are written in.

## The pipeline, and where each piece lives

| Stage | Input → output | Algorithm | Chapter | Module |
|---|---|---|---|---|
| Model loading | UFO directory → particles, parameters, vertices | Python-AST parsing, expression evaluation | [UFO models](02-ufo.md) | `ufo` |
| Diagram enumeration | process → tree diagrams | topology generation + particle insertion (feyngraph) | [Feynman diagrams](03-diagrams.md) | `diagrams` |
| Amplitude evaluation | momenta, helicities → $\mathcal{M}$ | HELAS wavefunctions and currents, from UFO Lorentz structures | [Helicity amplitudes](04-helicity-amplitudes.md) | `helas` |
| Colour | colour structures → exact colour matrix | symbolic SU(3) reduction | [Colour](05-color.md) | `helas::color` |
| Compilation | diagrams + model → executable program | rooting, lowering, CSE, folding, helicity expansion | [The amplitude compiler](06-compiler.md) | `helas::eval` |
| Phase space | unit hypercube → momenta + weight | RAMBO, per-diagram multichannel maps | [Phase space](07-phase-space.md) | `phasespace` |
| Integration | integrand → $\sigma \pm \delta\sigma$ | VEGAS, multichannel, budget allocation | [VEGAS](08-vegas.md), [Multichannel](09-multichannel.md) | `vegas`, `budget` |
| Hadronic convolution | partonic → hadronic $\sigma$ | PDFs, flavour groups, running couplings, scales | [Proton beams](10-hadronic.md) | `proton`, `pdf`, `coupling`, `cuts` |
| Unweighting | frozen grids → events | accept/reject with a truncated maximum | [Unweighting and event files](11-unweighting-events.md) | `unweight`, `select` |
| Output | events → `.lhe` | Les Houches Event File | [Unweighting and event files](11-unweighting-events.md) | `lhef` |

The `vibegraph` binary exposes this as two commands. `integrate` runs the
first eight rows and banks the adapted grids; `generate` runs the last two
against them. The [command line](../cli/overview.md) pages describe the
workflow.

## What is not here

This is a leading-order, tree-level generator. There are no loop
amplitudes, no parton shower, no hadronisation, no matching or merging.
The events it writes are the input a shower program such as Pythia 8 takes;
the [review by Buckley et al.](../bibliography.md#the-pipeline-as-a-whole)
places this stage in the full simulation chain, and the MadGraph5\_aMC@NLO
paper describes the machinery that the same pipeline grows into at NLO.
