---
type: Physics Convention
title: UFO spin codes, ALOHA value families and Lorentz structures by spin
description: "UFO spin codes (2s+1, −1 for ghosts), ALOHA value families by spin, operator tokens per SM vertex class, and ProjM/ProjP naming by the sign in (1±γ5)/2."
status: draft
tags: [ufo, aloha, spin, lorentz, conventions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n02-mg-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/02-reference-implementations.md#L310-L356", title: "Note 02, MadGraph Goal 1: UFO model loading"}
  - {id: n09-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/09-ufo-aloha-type-matrix.md#L32-L58", title: "Note 09, ground truth from UFO and ALOHA"}
  - {id: n09-chirality, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/09-ufo-aloha-type-matrix.md#L74-L93", title: "Note 09, basis and chirality layering"}
  - {id: n10-sm-table, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/10-lorentz-runtime-eval-plan.md#L402-L415", title: "Note 10 §6.1, SM pattern table"}
  - {id: mg-sm-lorentz, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/lorentz.py", title: "MadGraph models/sm/lorentz.py"}
  - {id: mg-aloha-parser, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_parsers.py#L45", title: "ALOHA aloha_parsers.py UFOExpressionParser"}
  - {id: ufo-paper, resource: "https://arxiv.org/abs/1108.2040", title: "UFO — The Universal FeynRules Output"}
---

## UFO spin codes

A UFO particle's `spin` is **2s + 1**, with **−1 reserved for ghosts**. A UFO `Lorentz`
object carries one such code per leg (`spins = [2, 2, 3]`) and a symbolic `structure`
string. [^n09-ufo] [^ufo-paper]

| Code | Field | Status here |
|---|---|---|
| 1 | scalar | supported |
| 2 | Dirac fermion (4-component spinor leg) | supported; Majorana is not ([backlog](../backlog/feature/majorana-fermions-unsupported.md)) |
| 3 | vector | supported |
| 4 | spin-3/2 | no wavefunction or propagator ([backlog](../backlog/feature/spin-2-and-spin-3-2-particles-unsupported.md)) |
| 5 | spin-2 | no wavefunction or propagator (same item) |
| −1 | ghost (anticommuting scalar) | dropped: tree-level diagrams are built in unitary gauge |

Ghosts and Goldstone bosons never enter diagram generation: `build_feyngraph_model`
(`ufo/topo.rs`) skips the particles and every vertex touching one, as MadGraph does at
tree level. feyngraph counts spin as 2s, so the loader passes `spin − 1`.

UFO colour codes on particles are `1`, `3`/`−3`, `6`/`−6` and `8`. One quirk of this
loader: an antiparticle created from `.anti()` gets `color = −self.color` for every
representation, so singlet and octet antiparticles carry `−1`/`−8` where UFO's own
`anti()` leaves self-conjugate representations unchanged
([backlog](../backlog/hygiene/make-anti-negates-singlet-octet-colour.md)). A reader of
`Particle::color` on an antiparticle has to allow for it.

## ALOHA value families

ALOHA builds wavefunctions and propagators by branching on the spin code. The
value-level families are [^n09-ufo]:

| Family | Components | Spin code |
|---|---|---|
| Scalar | 1 | 1 |
| Spinor | 4 | 2 |
| Vector | 4 | 3 |
| Spin3Half (vector-spinor) | 4 × 4 | 4 |
| Spin2 (rank-2 Lorentz tensor) | 4 × 4 | 5 |

External fermions are 4-component Dirac bispinors in ALOHA's generated routines, and
vibegraph matches that: `InDiracWf`/`OutDiracWf` hold four complex components, with no
separate two-component Weyl types. [^n09-chirality] How each family is constructed and
propagated here is in
[external wavefunctions and propagators](../amplitudes/wavefunctions-and-propagators.md).

## The Lorentz operator vocabulary

ALOHA's structure strings are built from `Gamma`, `Sigma`, `Gamma5`, `C`, `Metric`,
`Epsilon`, `Identity`, `ProjM`, `ProjP` and the momentum insertion `P(mu, i)`. ALOHA
parses them with a PLY grammar (`UFOExpressionParser`) [^mg-aloha-parser]; vibegraph
parses the same vocabulary with its own PEG grammar into `LorentzOp`
(`ufo/lorentz.rs`), and refuses any other operator name as `UnknownOperator`. The grammar
itself is in [UFO string grammars](ufo-string-grammars.md). `C` parses but is refused at
rooting (`RootLorentzError::UnsupportedVertex`, `helas/eval/root_lorentz.rs`) with the
Majorana machinery it belongs to. MadGraph's side of these conventions, at the pinned
version, is summarised in [MadGraph5_aMC@NLO](../references/codebases/madgraph5-amcnlo.md).

Two normalisations a model author or kernel writer has to know:

- **`Sigma` is ALOHA's**, half the textbook `(i/2)[γ^μ, γ^ν]`.
- **`Gamma5` is `ProjP − ProjM`.**

The `Sigma` normalisation is pinned in
[Levi-Civita and Sigma conventions](../amplitudes/levi-civita-and-sigma-conventions.md),
`Gamma5` in [gamma chains, Gamma5 and Epsilon](../amplitudes/gamma-chains-gamma5-and-epsilon.md).

### ProjM and ProjP

UFO and ALOHA name the chiral projectors by the sign in `P± = (1 ± γ⁵)/2`, not by
chirality: `ProjM = (1 − γ⁵)/2`, `ProjP = (1 + γ⁵)/2`. Which of them is "left" depends on
the γ⁵ and metric conventions in force, whereas the ± eigenprojector label does not. In
the common HEP convention, and in this code (`LorentzOp::ProjM` documents
`P_L = (1 − γ⁵)/2`), `ProjM` is the left-chiral projector and `ProjP` the right-chiral
one. [^n09-chirality]

## Structures by SM vertex class

The Standard Model's `lorentz.py` [^mg-sm-lorentz] uses this vocabulary as follows:

| Leg spins | Structures | Operators |
|---|---|---|
| `2 2 3` (FFV) | `FFV1` = `Gamma(3,2,1)`; `FFV2`–`FFV5` = `Gamma(3,2,-1)*ProjM(-1,1)` plus 0, −2, 2 or 4 × the `ProjP` term | `Gamma`, `ProjM`, `ProjP` |
| `2 2 1` (FFS) | `FFS1` `ProjM`; `FFS3` `ProjP`; `FFS2` `ProjM − ProjP`; `FFS4` `ProjM + ProjP` | projectors only |
| `3 3 3` (VVV) | `VVV1`: `P × Metric`, six terms | `Metric`, `P` |
| `3 3 3 3` (VVVV) | `VVVV1`–`VVVV5`: sums of `Metric × Metric` | `Metric` |
| `3 3 1`, `3 3 1 1` (VVS, VVSS) | `Metric(1,2)` | `Metric` |
| `3 1 1` (VSS) | `P(1,2) − P(1,3)` | `P` |
| `1 1 1`, `1 1 1 1` (SSS, SSSS) | `'1'` | none |
| `−1 −1 1`, `−1 −1 3` (ghosts) | `'1'`, `P(3,2) + P(3,3)` | dropped with the ghosts |

No SM structure uses `Sigma`, `Gamma5`, `Epsilon` or `C`; those arrive with non-SM models
such as the [vendored SMEFTsim UFO](smeftsim-topu3l.md). A UFO vertex's `couplings`
dictionary is keyed `(colour index, Lorentz index)` into its `color` and `lorentz` lists.

The evaluator does not dispatch on these classes. Each structure is parsed into a
`LorentzOp` product and evaluated by the rooted Lorentz evaluator, so a new structure
needs no new routine as long as its operators are supported.

[^n09-ufo]: Note 09, "Ground truth from UFO and ALOHA": spin codes and ALOHA object families.
[^n09-chirality]: Note 09, "Basis and chirality layering": 4-component external fermions, `ProjM`/`ProjP` naming.
[^mg-sm-lorentz]: `models/sm/lorentz.py` at the MadGraph pin.
[^mg-aloha-parser]: `aloha/aloha_parsers.py` `UFOExpressionParser`, L45 (note 02's reading).
[^ufo-paper]: The UFO paper, for the spin-code convention.
