---
type: Physics Convention
title: Polarized external particles
description: "MadGraph's polarization syntax and codes, NHEL lists, IDEN averaging, identical-particle keying and me_frame; polarized |M|² is frame dependent."
status: draft
tags: [polarization, helicity, process-grammar, madgraph-parity, frame]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-pol, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/01-paper-summaries.md#L466-L482", title: "Note 01, polarized matrix elements in MG5_aMC: conventions and truncated propagator"}
  - {id: n38-p1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L882-L1031", title: "Note 38 §4 P1, polarized external particles"}
  - {id: n38-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1474-L1527", title: "Note 38 §8.4, the seeded e+ e- > w+ w- reference"}
  - {id: n38-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1528-L1686", title: "Note 38 §8.5, twenty-seed sweep of the polarized rows"}
  - {id: mg-pol-codes, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/interface/madgraph_interface.py#L5110-L5188", title: "MadGraph madgraph_interface.py, polarization codes"}
  - {id: mg-hel-matrix, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/helas_objects.py#L4834", title: "MadGraph helas_objects.py get_helicity_matrix"}
  - {id: mg-denominator, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/helas_objects.py#L4910", title: "MadGraph helas_objects.py get_denominator_factor"}
  - {id: mg-identical, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/base_objects.py#L3742", title: "MadGraph base_objects.py identical_particle_factor"}
  - {id: mg-check-pol, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/base_objects.py#L3869", title: "MadGraph base_objects.py check_polarization"}
  - {id: mg-massless-zero, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/core/diagram_generation.py#L1748-L1795", title: "MadGraph diagram_generation.py, helicity 0 of massless bosons removed"}
  - {id: mg-frame, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/banner.py#L4296", title: "MadGraph banner.py, me_frame"}
measured:
  - {commit: 3c023b2, pr: 12, landed_in: 1539abc, command: "cargo test -p vibegraph-lib --test polarization_census --test polarization_frame; amplitude_oracle polarized rows"}
---

A polarized leg restricts that leg's helicity sum to the listed states. For external
particles this changes only the helicity loop: the diagrams are the unpolarized ones, and
an external leg's helicities add incoherently, so `w+{0}` plus `w+{T}` equals `w+` exactly
(pointwise to 3.8e-16, in any frame). What changes the physics is that **a polarized |M|²
is not Lorentz invariant**, so the evaluation frame becomes part of the answer.
[^n38-p1] The underlying method is in
[polarized matrix elements in MG5_aMC](../references/papers/polarized-matrix-elements-mg5.md).

## MadGraph's syntax

A polarization is written in braces after the particle, `w+{0}`, `z{T}`, `e-{L}`. The
codes, read from the pinned parser [^mg-pol-codes]:

| Code | Helicities (`NHEL`) | Allowed on |
|---|---|---|
| `T` | `[1, -1]` | spin 1 only |
| `L` | −1 (left) | any spin; on a vector MadGraph warns that `L` is left, not longitudinal |
| `R` | +1 | any spin |
| `0` | 0 (longitudinal) | not scalars or fermions (spin codes 1, 2 are an error) |
| `+d`, `-d`, `d` | ±d, \|d\| ≤ 3 | any |
| `A` (99), `G` (4), `H` (5), `Q` (6), `W` (7), `S` (9) | propagator projections: auxiliary, metric, θ, longitudinal − θ, Ward-protected full propagator, auxiliary + width | spin 1 only, and only on a propagator |

Codes concatenate inside the braces: `a{0T}` is `{0}` plus `{T}`. A comma is not a
separator there, because the process line has already been split on commas for decay
chains: `a{0,T}` is a MadGraph parse error. Helicities are the listed codes **in the
listed order** (`get_helicity_matrix`, `helas_objects.py:4834`) [^mg-hel-matrix].

**Conventions.** Fermion helicity is ±1 in MadGraph's `NHEL` (±1/2 physically), relative
to the fermion's momentum. Vector helicities are ±1 (transverse) and 0 (longitudinal),
defined in a chosen frame. The only further vector label is the auxiliary `A`, which
exists for off-shell propagators; there is no "±2" vector helicity among MadGraph's codes.
[^n01-pol]

**Polarized intermediate resonances** use a truncated propagator: for a resonance of
definite helicity λ the numerator is replaced by the projection
`Δ_μν(q) → ε*_μ(q,λ) ε_ν(q,λ)` while the line stays off shell
([truncated-propagator polarization](../references/papers/truncated-propagator-polarization.md)).
That is a change to the propagator, not the helicity loop; it is not supported here.

## What is supported

- **External legs, initial or final**: `{0}`, `{T}`, `{L}`, `{R}`, `{±d}`, on any particle
  whose helicity states include them. `SupportedLeg` carries the text between the braces;
  enumeration reads it into MadGraph's codes per concrete particle, and
  `DiagramSet.polarizations` carries each leg's `NHEL` list to
  `AmplitudeEvaluator::compile`, which sums over the product of the lists. The helicity
  loop itself is in [helicity sum and pruning](../amplitudes/helicity-sum-and-pruning.md).
- **Refused** ([backlog](../backlog/feature/polarized-intermediate-resonances-refused.md)):
  a polarization on a particle a decay chain decays, or on a decay's own initial particle
  (`DecayedPolarization`: `p p > w+{0} w-, w+ > e+ ve`; `t{L} > w+ b`, whose helicity at
  rest is a spin projection on an axis nothing pins); and the propagator codes
  (`PropagatorPolarization`). `{A}` on an external leg is a MadGraph error and stays one
  (`helas_objects.py:686`). Polarized decay products are supported: `t > w+{0} b` and
  `t > w+{T} b` match MadGraph's `NHEL` and `IDEN` (6).
- **Refused here though MadGraph generates them**: a code that is no helicity state of the
  particle (`z{2}`, `h{R}`), a code listed twice (`z{00}`, which MadGraph would sum twice),
  and two lines whose same-particle subprocesses would both count a helicity state
  (`z{0} h` plus `z h`). Lines that split a process by disjoint polarizations (`z{0} h`
  plus `z{T} h`) are accepted.
- **Beam polarization** (`polbeam1/2`) is a separate run-card feature and is refused
  ([backlog](../backlog/feature/beam-polarization-unsupported.md)). Helicity Monte Carlo
  (`nhel = 1`) is in scope and not yet supported
  ([backlog](../backlog/feature/nhel1-run-cards-refused.md)).

## MadGraph rules ported, each pinned

- **Helicity 0 of a massless boson** is kept by the parser, so a multiparticle mixing
  massive and massless bosons can be polarized longitudinally, and removed at generation
  (`diagram_generation.py:1751`, `:1791`) [^mg-massless-zero]: all zeros for an initial
  leg, one for a final leg; a leg left with no helicity drops the assignment.
  `u u~ > v{0} g` with `v = z a` generates `u u~ > z{0} g` alone; `e+ e- > a{0} z` and
  `g{0} g > t t~` have no diagrams (`NoDiagramException`).
- **Ambiguity** (`check_polarization`, `base_objects.py:3869`, called from `do_add`)
  [^mg-check-pol]: an outgoing particle both polarized and not, or with overlapping
  unequal lists, makes MadGraph ask, and a batch run answers no and raises `InvalidCmd`
  (`p p > z{T} z`, `z{L} z{T}`, `z{RL} z{LR}`, `a{0} a`). Ported as
  `resolve::polarizations_unambiguous` and refused both in `resolve_card` and in
  enumeration.
- **Averaging** (`get_denominator_factor`, `helas_objects.py:4910`) [^mg-denominator]: a
  polarized incoming leg contributes `len(polarization)` in place of its spin-state count.
  `IDEN` is 2 for `e+ e-{L} > mu+ mu-`, 1 for `e+{R} e-{L}`, 18 for `u{L} u~ > z{0} g`, 128
  for `g{R} g > t t~` and 256 for `g{T} g`. `initial_spin_color_average` follows it. A
  fixed-energy card whose subprocesses average differently is refused
  (`InconsistentSpinAverage`), and flavour groups only join subprocesses polarized alike.
- **Identical particles** key on `(id, polarization)` (`identical_particle_factor`,
  `base_objects.py:3742`) [^mg-identical]: `z{0} z{0}` and `z{T} z{T}` take 1/2; `z{L} z{R}`
  and `z{0} z{T}` take 1. `hadronic::outgoing_symmetry_factor` and
  `Subprocess::symmetry_factor` follow it, and within a line the subprocess dedup key is
  `(name, polarization)`, as MadGraph's `tag = zip(prod, polids)`
  (`diagram_generation.py:1765`). See
  [identical-particle factor](../phase-space/identical-particle-factor.md) and
  [subprocess enumeration](subprocess-enumeration.md).
- **Events**: the `SPINUP` of a polarized leg is its fixed helicity.

## The frame

MadGraph evaluates a polarized |M|² after `boost_to_frame` into the rest frame of the
particles listed in the run card's `me_frame` (`genps.f:1759`). `me_frame` defaults to
`[1, 2]`, the partonic centre of mass (`banner.py:4296`) [^mg-frame]; `frame_id = Σ 2^n`
over it (`banner.py:4705`), and `auto_dsig_v4.inc:134` boosts only when `frame_id != 6`.
The run card shows the frame block only for a polarized **massive** leg (`banner.py:5027`).

Here the evaluators already require the partonic centre of mass, so the default frame
needs no boost. `me_frame` is `Consumed` (`RunCard::frame_id`, read by
`hadronic::refuse_polarized_frame` for a polarized massive leg), its stored default is
MadGraph's `1, 2`, and a non-default `me_frame` on such a card is refused until the boost
exists. `frame_id` is `IgnoredBenign`, a value MadGraph recomputes.

The frame is pinned by a mutation (`polarization_frame` and `proton` unit tests): at the
banked centre-of-mass points this side reproduces MadGraph; boosted by
β = (0.3, −0.2, 0.5) the polarized massive rows move by 0.32–0.95 (relative) while the
helicity sums stay within 1.6e-14, and the massless `e-{L}` row within 2.5e-14. The proton
integrand of `u u~ > z{0} g` reproduces a centre-of-mass assembly to 1.5e-12 and sits 6.7
(relative) from the laboratory-frame one. One trap: the banked W points sit on the card's
printed `MW = 80.419`, 6e-8 off the derived mass both programs use, and an off-shell W's
helicity sum moves by about 2e-8 under that boost (∝ β²), so the invariance control
projects onto the mass shell first. [^n38-p1]

## Evidence

- **Census** (`validation/madgraph/dump_polarization_census.py` →
  `polarization_census.json`, hermetic `polarization_census`): 35 cards through MadGraph's
  generation and `HelasMatrixElement`; 24 generated and matched subprocess for subprocess
  on legs, `NHEL` set and `IDEN`; 7 refused by both; 4 refused here only (the cases
  above).
- **Amplitudes** (hermetic `amplitude_oracle`, gated): `ee_to_wp0wmt`, `ee_to_wp0wm`,
  `ee_to_z0h`, `uux_to_ztg`, `ee_to_mumu_eml`, `ee_to_tlt` against MadGraph's own `NHEL`
  table, per diagram and per colour flow; worst |M|² 2.2e-13, per diagram ≤ 6.0e-15.
- **σ** (`e+ e- > w+{0} w-`, 500 GeV, MadGraph's default card): the seeded MadEvent
  reference is 0.257798 ± 0.0002 pb; five seeds here read a mean of 0.257932 (pulls −0.02
  to +0.48 against a single 10k-event MadEvent run, 0.2578 ± 0.00047), and twenty seeds a
  χ²/dof of 0.92. The unpolarized `e+ e- > w+ w-` agrees with
  its seeded reference (7.19516 ± 0.0048 pb): twenty seeds here give a mean of 7.19608, rel
  +2.6e-4, pull +0.41, χ²/dof 1.45 over 19 dof (p ≈ 0.09). A five-seed subset reads 2.4,
  and a single MadEvent run had suggested a −0.23% offset; neither held up.
  [^n38-b1] [^n38-z2] `w+{0}` + `w+{T}` = `w+ w-` to 3.0e-5 across independent seeds.
- **Samples** (`vibegraph generate`, 20000 events × 3 seeds against MadEvent's 10000):
  the W+ `SPINUP` is 0 on every event on both sides; the W− distribution χ² reads 1.45/2,
  0.35/2, 2.70/2.

Related: [proc-card grammar](proc-card-grammar.md) for how legs are parsed.

[^n01-pol]: Note 01, the polarized-matrix-element paper summary: helicity conventions and the truncated propagator. Its "λ = ±2 (axial)" vector label is not among MadGraph's codes and is not carried here.
[^n38-p1]: Note 38 §4, polarized external particles: every MadGraph rule above, the refusals, the frame, census, amplitude, σ and sample evidence.
[^n38-b1]: Note 38 §8.4: the seeded references for `e+ e- > w+ w-` and `w+{0} w-`; the −0.23% was a single MadEvent run's.
[^n38-z2]: Note 38 §8.5: twenty seeds close the χ²/dof ≈ 2.4 finding.
[^mg-pol-codes]: `madgraph/interface/madgraph_interface.py` L5110–L5188.
[^mg-hel-matrix]: `madgraph/core/helas_objects.py` `get_helicity_matrix`, L4834.
[^mg-denominator]: `madgraph/core/helas_objects.py` `get_denominator_factor`, L4910.
[^mg-identical]: `madgraph/core/base_objects.py` `identical_particle_factor`, L3742.
[^mg-check-pol]: `madgraph/core/base_objects.py` `check_polarization`, L3869; called at `madgraph_interface.py` L3324.
[^mg-massless-zero]: `madgraph/core/diagram_generation.py` L1748–L1795.
[^mg-frame]: `madgraph/various/banner.py` L4296 (`me_frame`), L4705 (`frame_id`), L5027 (frame block).
