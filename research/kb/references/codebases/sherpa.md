---
type: Codebase Survey
title: Sherpa / COMIX
description: "Sherpa at e12c72f4: COMIX's colour-dressed Berends-Giele currents, METOOLS wavefunctions and vertices, the native UFO loader, and PHASIC++'s colour and helicity integrators."
resource: "https://gitlab.com/sherpa-team/sherpa/-/tree/e12c72f4dc358759677da62037ae2bea197bed83"
status: draft
tags: [sherpa, comix, berends-giele, external-code, matrix-elements]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n03-sherpa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L10-L162", title: "Note 03 Part 1, Sherpa / COMIX survey"}
  - {id: n03-compare, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L369-L383", title: "Note 03 Part 3, comparison of all surveyed generators"}
  - {id: sh-current, resource: "https://gitlab.com/sherpa-team/sherpa/-/blob/e12c72f4dc358759677da62037ae2bea197bed83/METOOLS/Explicit/Current.H#L43-L178", title: "METOOLS/Explicit/Current.H, class Current"}
  - {id: sh-vertex, resource: "https://gitlab.com/sherpa-team/sherpa/-/blob/e12c72f4dc358759677da62037ae2bea197bed83/METOOLS/Explicit/Vertex.C#L102-L179", title: "METOOLS/Explicit/Vertex.C, Vertex::Evaluate"}
  - {id: sh-amp, resource: "https://gitlab.com/sherpa-team/sherpa/-/blob/e12c72f4dc358759677da62037ae2bea197bed83/COMIX/Amplitude/Amplitude.C#L1366-L1460", title: "COMIX/Amplitude/Amplitude.C, EvaluateAll"}
  - {id: sh-ufo, resource: "https://gitlab.com/sherpa-team/sherpa/-/blob/e12c72f4dc358759677da62037ae2bea197bed83/MODEL/UFO/UFO_Model.H#L11-L44", title: "MODEL/UFO/UFO_Model.H"}
  - {id: sh-hel, resource: "https://gitlab.com/sherpa-team/sherpa/-/blob/e12c72f4dc358759677da62037ae2bea197bed83/PHASIC++/Main/Helicity_Integrator.H", title: "PHASIC++/Main/Helicity_Integrator.H"}
  - {id: sh-col, resource: "https://gitlab.com/sherpa-team/sherpa/-/blob/e12c72f4dc358759677da62037ae2bea197bed83/PHASIC++/Main/Color_Integrator.H", title: "PHASIC++/Main/Color_Integrator.H"}
  - {id: comix, resource: "https://arxiv.org/abs/0808.3674", title: "Gleisberg, Höche, COMIX (JHEP 0812 (2008) 039)"}
---

Sherpa is a full event-generation framework. Its COMIX component computes
tree-level matrix elements by colour-dressed Berends–Giele recursion, the main
alternative to MadGraph's per-diagram evaluation; the algorithm is in the
[COMIX paper](../papers/comix.md). vibegraph follows MadGraph, not COMIX; the
three architectures are set side by side in
[generators compared](generator-architectures-compared.md).

The submodule `research/refs/sherpa` is pinned at
`e12c72f4dc358759677da62037ae2bea197bed83` (`master`). Note 03 surveyed it at
that commit, and the files and lines below were re-checked there[^n03-sherpa].

## Layout

```text
COMIX/      matrix-element generator (Berends–Giele)
METOOLS/    shared ME tools: currents, vertices, wavefunctions, Lorentz and colour calculators
MODEL/      models, including a native UFO loader
PHASIC++/   phase space, integration, colour and helicity sampling
ATOOLS/     utilities: four-vectors, Flavour, spinors
AMEGIC++/   the older diagram-based ME generator
CSSHOWER++/, DIRE/   parton showers
MCATNLO/    MC@NLO matching
```

## Currents and the recursion step

An off-shell current `J(S)` is the sum of all sub-diagrams joining the external
legs in the set `S` to one off-shell leg. `Current` (`METOOLS/Explicit/Current.H:43`)
carries the flavour, the momentum `m_p`, the wavefunction objects `m_j` indexed
by helicity and colour, the incoming and outgoing vertices, and `m_cid`, a
bitmask of the contributing external legs, which is how the recursion finds
currents that can be combined[^sh-current]:

```cpp
class Current {
protected:
  ATOOLS::Flavour m_fl;
  Vertex_Vector m_in, m_out;
  ATOOLS::Vec4D  m_p;
  CObject_Matrix m_j;
  Polarization_Index m_h;
  size_t m_key, m_cid, m_ntc;
  ...
```

Concrete currents are `CS` (scalar, `METOOLS/Currents/S_C.C`), `CF` (spinor,
`F_C.C`) and `CV` (vector, `V_C.C`).

`Vertex::Evaluate()` (`METOOLS/Explicit/Vertex.C:102`) is the recursion step.
For two child currents it loops over their helicity and colour components, asks
each colour calculator whether the colour combination contributes, and if so
calls the matching Lorentz calculator and accumulates into the parent[^sh-vertex]:

```cpp
for (size_t h0(0);h0<sh0;++h0) {
  ...
  for (size_t h1(0);h1<sh1;++h1) {
    ...
    for (size_t c0(0);c0<hjj0->size();++c0) {
      m_cjj[0]=(*hjj0)[c0];
      for (size_t c1(0);c1<hjj1->size();++c1) {
        m_cjj[1]=(*hjj1)[c1];
        for (size_t k(0);k<m_cc.size();++k)
          if (m_cc[k]->Evaluate(m_cjj)) {
            CObject *j(m_lc[k]->Evaluate(m_cjj));
            ...
            j->Multiply(p_v->Coupling(k)*m_cc[k]->Coupling());
            j->SetH(H(hid));
            m_cc[k]->AddJ(j);
```

`Lorentz_Calculator` (`Lorentz_Calculator.H`) has one subclass per Lorentz
structure (FFV, VVV, SSV, …); `Color_Calculator` (`Color_Calculator.H`)
implements the SU(3) contractions and carries the representation in `CInfo`.
Currents carry explicit colour indices through the recursion, which is what
"colour-dressed" means.

## Amplitude orchestration

`COMIX/Amplitude/Amplitude` owns the currents organised by number of external
legs (`m_cur[n]`) and the per-helicity results (`m_ress`, a
`Spin_Structure`). `Amplitude::EvaluateAll()` (`Amplitude.C:1366`) calls
`CalcJL()` (`Amplitude.C:1142`), which builds `m_cur[2]`, `m_cur[3]`, … from the
external wavefunctions in `m_cur[1]`, then sums `Re(A_i A_j*)` over the stored
helicity configurations and the pairs of coupling-order components
(`m_on`)[^sh-amp]:

```cpp
for (size_t k(0);k<m_ress[i].size();++k)
  if (m_ress[i][k]!=Complex(0.0,0.0) &&
      m_ress[j][k]!=Complex(0.0,0.0)) {
    csum+=(m_ress[i][k]*std::conj(m_ress[j][k])).real();
  }
```

There is no loop over external colours in this sum: the external colour
assignment is supplied from outside, by PHASIC++'s `Color_Integrator` (below),
which sums or samples it. This reading of the call sequence was not traced
end to end.
`Spin_Structure` (`METOOLS/Main/Spin_Structure.H`) is a flat vector indexed by
helicity id; `Polarization_Index` maps helicity tuples to that index. Process
entry points are `COMIX/Main/Single_Process.H` (`Partonic(...)`,
`GetAmplitude()`) and `Process_Group.H`.

## Wavefunctions

`CObject` (`METOOLS/Explicit/C_Object.H`) is the base of all wavefunction
objects and carries colour indices and a helicity index with its value.
`CVec4<Scalar>` (`METOOLS/Currents/C_Vector.H`) is the complex four-vector for
gluons and photons; `Spinor<Scalar>` (`ATOOLS/Phys/Spinor.H`) is a
two-component spinor built from a four-momentum. Unlike HELAS wavefunctions,
these carry colour.

## Models

Sherpa reads UFO models natively in C++. `UFO_Model : Model_Base`
(`MODEL/UFO/UFO_Model.H:11`) populates particles and vertices in `ModelInit()`
and maps UFO Lorentz structures to METOOLS calculators in `FillLorentzMap()`[^sh-ufo].
`UFO_Param_Reader` reads parameter cards; `Single_Vertex` is the internal
vertex (Lorentz and colour structure plus coupling); `ATOOLS/Phys/Flavour.H`
is the particle type (`StrongCharge()`, `IntCharge()`, `IntSpin()` as 2s,
`Mass()`, `Width()`).

## PHASIC++: integration, colour and helicity sampling

| Class | File | Role |
|---|---|---|
| `Phase_Space_Integrator` | `PHASIC++/Main/Phase_Space_Integrator.H` | multichannel adaptive integration loop |
| `Phase_Space_Handler` | `PHASIC++/Main/Phase_Space_Handler.H` | channels linked to the process |
| `Color_Integrator` | `PHASIC++/Main/Color_Integrator.H` | sums or samples colour assignments (`cls::sum`, `cls::sample`); `GenerateColours()` (line 87) draws one |
| `Helicity_Integrator` | `PHASIC++/Main/Helicity_Integrator.H` | sums or samples helicities (`hls::sum`, `hls::sample`); `GeneratePoint()` (52) draws, `Optimize()` (57) adapts the weights |

Sampling colour and helicity per point, with adapted weights, is the design
choice COMIX's recursion makes cheap: a sampled point evaluates one colour
assignment and one helicity configuration rather than the full sum[^sh-hel][^sh-col].

[^n03-sherpa]: Note 03 Part 1, surveyed at `e12c72f`; files and line numbers re-checked at `e12c72f4`.
[^sh-current]: `METOOLS/Explicit/Current.H:43–75` at `e12c72f4`.
[^sh-vertex]: `METOOLS/Explicit/Vertex.C:102–179` at `e12c72f4`.
[^sh-amp]: `COMIX/Amplitude/Amplitude.C:1366–1460` at `e12c72f4`.
[^sh-ufo]: `MODEL/UFO/UFO_Model.H` at `e12c72f4`.
[^sh-hel]: `PHASIC++/Main/Helicity_Integrator.H` at `e12c72f4`.
[^sh-col]: `PHASIC++/Main/Color_Integrator.H` at `e12c72f4`.
