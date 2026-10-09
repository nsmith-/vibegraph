---
type: Paper
title: "HELAS: HELicity Amplitude Subroutines"
description: "KEK-91-11 (Murayama, Watanabe, Hagiwara, 1992): the Fortran wavefunction and vertex subroutines behind MadGraph; naming, NHEL/NSF conventions, the vertex catalogue and a worked W+W- -> tt~."
resource: "https://inspirehep.net/literature/336604"
status: draft
tags: [helas, helicity-amplitudes, wavefunctions, paper, conventions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n01-helas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L345-L455", title: "Note 01, HELAS summary (from the OCR'd report)"}
  - {id: n01-helas-stub, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/01-paper-summaries.md#L148-L154", title: "Note 01, first HELAS entry (reference only)"}
  - {id: n00-refs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/00-overview.md#L57-L69", title: "Note 00, references"}
  - {id: kek-pdf, resource: "https://lib-extopc.kek.jp/preprints/PDF/1991/9124/9124011.pdf", title: "KEK preprint scan (the URL research/refs/fetch-papers.sh uses)"}
  - {id: repr, resource: "vibegraph-lib/src/helas/repr/lorentz.rs", title: "ComplexVector and Bispinor"}
  - {id: mg-helas, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/HELAS/ixxxxx.F", title: "MadGraph HELAS ixxxxx.F header: nsf +1 particle, -1 anti-particle"}
---

HELAS (Murayama, Watanabe and Hagiwara, KEK Report 91-11, January 1992; not
on arXiv) is a FORTRAN77 library for helicity amplitudes of arbitrary
tree-level diagrams. Given external momenta and helicities, its subroutines
build wavefunctions, propagate them through vertices as off-shell currents,
and return the complex amplitude. MadGraph emits HELAS calls from each
diagram's topology[^n01-helas]. [ALOHA](aloha.md) generalised the hand-written
routines to any UFO Lorentz structure; the library still sits in MadGraph's
`HELAS/` directory ([the MadGraph survey](../codebases/madgraph5-amcnlo.md)).

## Wavefunctions

Every wavefunction is a `complex(6)` array: four Lorentz or spinor components
and the four-momentum packed into two complex numbers,
`(P(0)+iP(3), P(1)+iP(2))`.

| Subroutine | Computes |
|---|---|
| `IXXXXX(P, FMASS, NHEL, NSF, FI)` | flowing-in fermion, `u(p)` or `v(p)`: `\|f>` |
| `OXXXXX(P, FMASS, NHEL, NSF, FO)` | flowing-out fermion, `ū(p)` or `v̄(p)`: `<f\|` |
| `VXXXXX(P, VMASS, NHEL, NSV, VC)` | vector polarization `ε(p)` or `ε*(p)` |
| `SXXXXX(P, NSS, SC)` | scalar (unity plus momentum) |

- `NHEL` is the helicity: ±1 for spin 1/2 (in units of 1/2), +1/0/−1 for spin 1.
- `NSV` and `NSS` are +1 for a final-state and −1 for an initial-state
  boson. `NSF` is +1 for a particle (`u`, `ū`) and −1 for an antiparticle
  (`v`, `v̄`), whichever side it is on: the outgoing `t̄` below is
  `IXXXXX(…, -1, …)`[^mg-helas].
- `P(0:3)` has `P(0)` the energy, always positive.

## Vertices

Each vertex type has an amplitude routine (all legs supplied) and current
routines (one leg off-shell, propagator included).

| Vertex | Amplitude | Currents |
|---|---|---|
| FFV | `IOVXXX` | `FVIXXX`, `FVOXXX` (fermion); `JIOXXX`, `J3XXXX` (vector) |
| FFS | `IOSXXX` | `FSIXXX`, `FSOXXX`, `HIOXXX` |
| VVV | `VVVXXX` | `JVVXXX` |
| VVS | `VVSXXX` | `JVSXXX`, `HVVXXX` |
| VSS | `VSSXXX` | `JSSXXX`, `HVSXXX` |
| SSS | `SSSXXX` | `HSSXXX` |
| VVVV | `WWWWXX`, `W3W3XX` | `JWWWWX`, `JW3WXX` |
| VVSS | `VVSSXX` | `JVSSXX`, `HVVSXX` |
| SSSS | `SSSSXX` | `HSSSXX` |
| collinear e–γ | — | `EAIXXX`, `EAOXXX`, `JEEXXX` |

Names are eight characters padded with `X`. Input codes are `I` (flowing-in
fermion), `O` (flowing-out fermion), `V` and `S`; outputs are `J` (off-shell
vector), `H` (off-shell scalar) and `F` (off-shell fermion); amplitude routines
are named by their inputs (`IOV`). `W`/`3` denote W and Z, `E`/`A` the
collinear electron and photon. The routine names in MadGraph's copy differ in
places (`JWWWXX` for the paper's VVVV vector current).

The FFV coupling is a two-element array: `G(1)` multiplies the left-chiral
projector `(1−γ⁵)/2`, `G(2)` the right-chiral `(1+γ⁵)/2`.

## A worked amplitude

`W⁺W⁻ → t t̄`: four diagrams in three amplitude calls, since `J3XXXX`
combines the γ and Z exchanges:

```fortran
CALL VXXXXX(PWM, WMASS, NHWM, -1, WM)              ! W- incoming
CALL VXXXXX(PWP, WMASS, NHWP, -1, WP)              ! W+ incoming
CALL OXXXXX(PT,  TMASS, NHT,  +1, FO)              ! t outgoing
CALL IXXXXX(PTB, TMASS, NHTB, -1, FI)              ! t~ outgoing (flowing-in)
CALL J3XXXX(FI, FO, GAU, GZU, ZMASS, ZWIDTH, J3)   ! Z/γ current
CALL VVVXXX(WP, WM, J3, GW, AMPS)                  ! s-channel
CALL FVIXXX(FI, WM, GWF, 0., 0., FVI)              ! off-shell b
CALL IOVXXX(FVI, FO, WP, GWF, AMPT)                ! t-channel
CALL HIOXXX(FI, FO, GCHT, HMASS, HWIDTH, HTT)      ! Higgs current
CALL VVSXXX(WM, WP, HTT, GWHH, AMPH)               ! s-channel Higgs
AMP = AMPS + AMPT + AMPH
```

Utilities: `MOMNTX` (momentum from E, m, cos θ, φ), `MOM2CX` (two-body CM
momenta), `BOOSTX`, `ROTXXX`, and `COUP1X`–`COUP4X` (SM couplings for VVV,
FFV, VVS/Higgs and FFS).

Conventions (Appendix A): Dirac matrices in the Weyl (chiral) representation;
massless spinors defined through a reference momentum; unitary gauge for weak
bosons, which minimises the diagram count; single precision, with a separate
double-precision `DHELAS`.

## Relevance to vibegraph

vibegraph keeps HELAS's structure (external wavefunctions, off-shell currents
with the propagator attached, a final contraction to a complex amplitude) and
its conventions: the Weyl basis, the `NHEL`/`NSF` meaning of helicity and
crossing, and unitary gauge for massive vector bosons (Goldstones and ghosts
are dropped before diagram enumeration). It does not keep the `complex(6)`
layout or the `G(2)` coupling array. Wavefunctions are typed values generic
over the scalar field, `Bispinor<F, Adj>` with the Dirac adjoint in the type
and `ComplexVector<F, V>` with its variance in the type (`helas/repr`); momenta
travel separately; couplings are the UFO's complex coupling values, with
chirality carried by the Lorentz structure's projectors[^repr]. The design is
[the repr layer](../../amplitudes/repr-layer-geometry-and-axes.md); the
wavefunctions and propagators themselves are
[wavefunctions and propagators](../../amplitudes/wavefunctions-and-propagators.md),
and fermion flow and crossed legs are
[fermion flow and crossing](../../amplitudes/fermion-flow-and-crossing.md).

[^n01-helas]: Note 01, second HELAS entry (summarised from the OCR'd report); authors and report number confirmed on INSPIRE.
[^repr]: `vibegraph-lib/src/helas/repr/lorentz.rs`: `ComplexVector` (line 338), `Bispinor` (767).
[^mg-helas]: Header comments of `HELAS/ixxxxx.F` and `oxxxxx.F` (`nsf`: +1 particle, −1 anti-particle), `vxxxxx.F` and `sxxxxx.F` (`nsv`/`nss`: +1 final, −1 initial) at `b7687064`. Note 01 states the final/initial meaning for all three. vibegraph's `DiracWf::from_momentum` takes `nsf` as `Charge::{Particle, Antiparticle}` (`helas/wavefn.rs:26`).
