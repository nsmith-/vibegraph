# Proton beams

A proton is not an elementary particle, and a collider that accelerates
protons collides their constituents. The QCD factorisation theorem lets
the hadronic cross section be written as the partonic one, convolved with
the probability of finding each parton at each momentum fraction:

$$
\sigma_{pp\to X} = \sum_{a,b} \int_0^1 dx_1\, dx_2\; f_a(x_1, \mu_F)\, f_b(x_2, \mu_F)\;
\hat\sigma_{ab \to X}(x_1 x_2 s;\, \mu_R, \mu_F).
$$

The **parton distribution functions** $f_a(x, \mu_F)$ are measured, not
computed, and are read from published fits. The factorisation scale
$\mu_F$ and the renormalisation scale $\mu_R$ at which
$\alpha_s$ is evaluated are choices, and at leading order the result
depends on them; a generator has to make the same choices as its reference
to agree with it. This chapter is about the four things a proton run adds
to a partonic one: the densities, the sum over flavours, the running
coupling, and the scales.

## Parton distributions

PDF sets are distributed in the LHAPDF6 format
([Buckley et al. 2015](../bibliography.md#proton-beams)): a directory of
text files holding $x f(x, Q^2)$ tabulated on a grid in $x$ and
$Q^2$, per flavour, in one or more $Q^2$ bands. vibegraph reads
these files directly, with no dependency on the LHAPDF library, and
reproduces its default interpolator: a local cubic Hermite in each of
$\ln x$ and $\ln Q^2$, with knot slopes from finite differences.
The choice matters. A global spline through the same knots is a different
function off the knots, and the MadGraph cross section it feeds would
differ. Points outside the tabulated range go through LHAPDF's continuation
rule for the same reason. A set also carries the $\alpha_s(M_Z)$ it was
fitted with, which is the coupling a run reading its densities has to use.

The default set is `NNPDF23_lo_as_0130_qed`, MadGraph's LO default. The
[command line](../cli/overview.md#data-the-binary-does-not-carry) page
describes how sets are fetched and pinned.

The outer integral is done over $(\tau, y)$ with
$\tau = x_1 x_2$ and $y = \tfrac12 \ln(x_1/x_2)$, the partonic
energy fraction and the boost of the partonic frame, with the per-diagram
[multichannel map](09-multichannel.md) inside it for the final state.

## Flavour groups

`p p > l+ l- j` expands to a few dozen concrete subprocesses, and most of
them are the same partonic calculation under a different flavour label:
$u\bar u$ and $c\bar c$ initial states, electrons and muons, a
quark or an antiquark against a gluon. Compiling and evaluating each
separately would repeat arithmetic the matrix element does not
distinguish.

Subprocesses are therefore partitioned into **flavour groups**, one
compiled amplitude, one phase-space map and one cut filter per group, with
the group's luminosity being the sum over its members' parton densities.
Nothing is hand-listed. Two subprocesses join a group when their
$|\mathcal{M}|^2$ agree at a shared set of probe points spanning several
energies, and the grouping is refused unless they also share their
outgoing masses, their cut filter and their colour basis, because
$|\mathcal{M}|^2$ equality alone does not license reusing one member's
colour flow for another. A partition whose groups are separated by less
than a set margin is treated as a failed measurement, not a decomposition.

Diagram enumeration produces one ordering per unordered initial state:
$g u$ is generated and $u g$ is not. Both are physical, since the
two beams' densities are evaluated at different momentum fractions, and the
missing ordering is restored through the identity

$$
|\mathcal{M}_{ba}(p_1, p_2, q)|^2 = |\mathcal{M}_{ab}(p_1, p_2, R q)|^2,
$$

with $R$ the rotation by $\pi$ about the $x$ axis, which swaps
the beams. A group contributes $f_a(x_1) f_b(x_2) |\mathcal{M}(q)|^2 +
f_b(x_1) f_a(x_2) |\mathcal{M}(Rq)|^2$ under one cut indicator: the
rotation is an argument to the matrix element, not a change to the event.

## The running coupling

$\alpha_s(\mu_R)$ is obtained by solving the renormalisation-group
equation at the requested loop order. MadGraph solves it implicitly, by
Newton iteration on the integrated $\beta$ function, with flavour
thresholds at fixed masses rather than the model's quark masses and a
stopping tolerance of $5\times10^{-4}$ on the relative step. The
returned value is a specific iterate rather than the exact root, so
reproducing MadGraph means reproducing the iteration, and vibegraph's
implementation is a port of `alfas_functions.f` validated bit for bit
against it. The compiled amplitude takes the per-event coupling through the
[monomial rescaling](06-compiler.md#rescaling-the-strong-coupling) path.

> **MadGraph compatibility.** Stopping a Newton iteration on a relative step of
> $5\times10^{-4}$ returns an iterate rather than the root, and fixing the
> flavour thresholds at 1.42 and 4.7 GeV ignores the quark masses the model was
> actually configured with. A cleaner design would solve the same
> renormalisation-group equation to convergence, at the model's own thresholds,
> or take $\alpha_s$ from the set that supplied the densities. Reproducing the
> iteration instead is what makes the per-event coupling bit-for-bit comparable
> with the reference, which is how the `AQCDUP` field of 180 000 banked events
> becomes a test of the running rather than a plausibility check.
> `vibegraph-lib/src/coupling/alphas.rs`.

## Scales

A run card asks for one renormalisation scale and one factorisation scale
per beam, three numbers. Each is either **fixed** to the card's value, or
**dynamical**, computed per event by the prescription
`dynamical_scale_choice` names. The default prescription, $-1$, names
no closed form at all: it clusters the event.

MadGraph's default runs a $k_T$ clustering
([Catani et al. 2001](../bibliography.md#proton-beams)) backwards over the
event's external momenta until a $2\to2$ core remains, using the
process's own diagrams to decide which legs may be merged, and reads the
scales off the vertices the clustering passed through: $\mu_R$ as the
geometric mean of the participating vertex scales, $\mu_F$ on each beam
from the last vertices at which that beam's line was still a parton and
still coloured.

**Why a clustering, and not a formula.** A leading-order event is one hard
configuration, but the configuration a shower would build out of it is a
sequence of splittings, each with its own hardness, and a single global number
like $\sqrt{\hat s}$ describes none of them. Running the $k_T$ algorithm
backwards reconstructs one such history: the pair with the smallest measure is
the most likely last emission, and undoing emissions repeatedly leaves a
$2\to2$ core with a scale attached to every vertex passed on the way. Reading
$\mu_R$ and $\mu_F$ off those vertices makes the coupling and the densities
track the hardness of the splitting each belongs to instead of one number for
the whole event. This is the scale assignment CKKW matching is built on, where
the same reconstructed history also supplies the Sudakov form factors the matrix
element is reweighted by. MadGraph's default keeps the reconstruction and,
with matching switched off, applies none of that reweighting, so the clustering
here does nothing but set the scales; vibegraph refuses a card that asks for
matching rather than running the scale half of a machine whose other half is
missing.

**Why the process's own diagrams decide which merges exist.** A history is only
meaningful if the process could have produced it. A merge stands for a
propagator, so a pair of lines whose combined set of external legs is not the
subtree of some propagator in some channel of this process corresponds to no
internal line at all, and merging it would attach a scale to a splitting the
process does not contain. The merge table is therefore derived from the channel
forests: a pair may merge only when a surviving channel carries a line whose
external-leg set is exactly that pair's, and the questions the scale walk asks
later, is this line a parton, is it coloured, are asked of the PDG code on that
internal line rather than of the daughters. The table depends on the channel a
second way, since channels whose QCD coupling order differs from the one being
integrated are dropped, so a process with mixed QCD and QED orders has one merge
table per order.

**What the resonance tagging is for.** A pair that came from the decay of an
on-shell particle is not an emission whose hardness is a transverse momentum:
its scale is the mother's mass. The integration channel's own timelike lines are
tested against their Breit-Wigner windows for each event, a pair whose line is
tagged on shell is measured by its invariant mass rather than by the $k_T$
measure, and the channels still admissible are restricted to those carrying
every tagged resonance. That keeps the reconstruction from clustering a decay
product onto a leg it did not come from, and it is one of the routes by which
the scale depends on which channel produced the event and not on its momenta
alone.

**Why $\mu_F$ is read per beam.** Factorisation gives each beam its own density and
so its own scale, and the scale that belongs to a beam is where that beam's line
stopped being something a density describes. The walk records three vertices per
beam: the first at which the line split, the last at which it was still a
parton, and the last at which it was still coloured. $\mu_F$ on a beam is the
geometric mean of the last two, because past that vertex there is no parton left
on that side for a density to have evolved.

**Why $\mu_R$ is a geometric mean.** $\alpha_s$ enters once per QCD vertex of the
reconstructed history, and vertices at different hardnesses mean a product of
couplings at different scales; the single scale that stands in for all of them
is the one whose repeated coupling comes closest to that product, which is what
a geometric mean of the vertex scales gives. MadGraph takes it over the
participating vertices, four factors when both beams contribute a parton line,
two when one does, one when neither. Both formulas and every one of their
branches are pinned against the reference per event; the reference's own
rationale for the mean, and for reading $\mu_F$ off the vertices it does, is
not recorded in this project's notes, so the last two readings are not
established beyond the shape of the expressions.

The answer depends on details a formula would not show. A crossed beam–leg
candidate is inflated by $1 + 10^{-6}$ as a tie-break; the winner of an exact
tie is the first pair visited; an initial-state merge can boost and rotate the
whole event, so later measures are taken in the rotated frame; and MadGraph
memoises the jet count per integration channel from the first event that reached
it, so its scale is not a function of the event's momenta alone. vibegraph
reproduces the measures, the tie-breaks and the frame changes, and the merge
sequence is validated against an instrumented MadGraph run event by event,
because a cross section cannot see a scale that is wrong by a factor that
averages out. The memo is the one piece it does not carry: the jet count starts
empty on every event, which reproduces what MadGraph does on its first event of
a channel and leaves the scale a function of the event and its channel rather
than of the order events were generated in.

> **MadGraph compatibility.** The $1 + 10^{-6}$ inflation and the
> first-pair-visited rule are not physics; they are how `cluster.f` happens to
> break ties. The inflation does not cancel when every admissible candidate is
> crossed, so it survives into the scale as a spurious part in $10^6$, which is
> why ten events of one banked partonic $2\to2$ run report `SCALUP` of
> `250.0001` where every other event of it reports `250`. A cleaner design
> would break ties by a rule that cannot leak into the value, or decline to
> separate candidates no measure separates. What
> reproducing MadGraph's rule buys is that the merge sequence and the scales can
> be replayed event by event against an instrumented reference run, which is the
> finest oracle this part of the pipeline has: a cross section would never see a
> tie broken the other way.
> `vibegraph-lib/src/coupling/cluster/kt.rs`.

The scale a shower reads from the event record, `SCALUP`, is the larger of
the two factorisation scales, not $\mu_R$; the two coincide for most
clusterings and differ on the ones that split them. That is the accord's own
definition of the field as well as what MadGraph writes, so it is not a
compatibility concession.

## Cuts

The run card's cuts are compiled per process into a flat check list with
MadGraph's conventions from its `cuts.f`: the single-leg $\eta$ cuts and
the $\Delta R$ separations use rapidity, not pseudorapidity; legs are
classified into MadGraph's letter classes (jet, b, photon, lepton) to decide
which thresholds apply. The filter runs in the laboratory frame and before
the matrix element, so a rejected point costs a phase-space map and nothing
else.

## Where it lives

[`vibegraph::pdf`](../api/vibegraph/pdf/index.html) for the grid reader,
[`interp`](../api/vibegraph/pdf/interp/index.html) and
[`extrap`](../api/vibegraph/pdf/extrap/index.html);
[`vibegraph::proton`](../api/vibegraph/proton/index.html) for flavour groups
and the [`ProtonIntegrand`](../api/vibegraph/proton/struct.ProtonIntegrand.html);
[`vibegraph::coupling`](../api/vibegraph/coupling/index.html) with
[`alphas`](../api/vibegraph/coupling/alphas/index.html),
[`scales`](../api/vibegraph/coupling/scales/index.html) and the
[`cluster`](../api/vibegraph/coupling/cluster/index.html) module whose
submodules separate the merge graph, the clustering and the scale walk;
[`vibegraph::cuts`](../api/vibegraph/cuts/index.html); and
[`vibegraph::runcard`](../api/vibegraph/runcard/index.html) with MadGraph's LO
defaults.
