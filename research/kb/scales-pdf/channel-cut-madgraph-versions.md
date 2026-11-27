---
type: Measurement
title: "MadGraph's sde_strategy = 2 channel weight before 3.7 reads q² as a mass"
description: "Before 3.7, get_channel_cut builds every propagator factor from (q² − M)(q² + M) = q⁴ − M², which removes the Breit-Wigner peak. 3.7.1 builds it from q² − M², and vibegraph transcribes 3.7.1. On the CMS DarkHiggs Zp 3000 gridpack (MG 2.9.18) the old form moves the realised configuration split, so μF and σ (2.4 %) move with it."
status: draft
tags: [scales, kt-clustering, configuration-draw, sde-strategy, madgraph-parity, madgraph-versions, cms-gridpacks, measurement]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-27}
sources:
  - {id: mg2918, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/v2.9.18/Template/LO/SubProcesses/genps.f#L1716-L1752", title: "MadGraph 2.9.18 genps.f get_channel_cut"}
  - {id: mg350, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/v3.5.0/Template/LO/SubProcesses/genps.f#L1880-L1902", title: "MadGraph 3.5.0 genps.f get_channel_cut"}
  - {id: mg360, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/v3.6.0/Template/LO/SubProcesses/genps.f#L1913-L1935", title: "MadGraph 3.6.0 genps.f get_channel_cut"}
  - {id: mg371, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L1930-L1951", title: "MadGraph 3.7.1 genps.f get_channel_cut (the pinned submodule)"}
  - {id: draw, resource: "clustering-configuration-draw.md", title: "The clustering configuration is drawn ∝ AMP2_c (or the channel cut)"}
measured:
  - {host: "sprint L physics container (linux, 4 cores)", command: "MadGraph 3.5.7 (pixi -e madgraph) generate_events on the 880 cards with get_channel_cut and matrix1 instrumented; python3 cfgw5.py / cfgw5_bug.py / cmpdump.py on the dumps and on 61 events of the CMS gridpack's own LHE", commit: "vibegraph code at 59045b57 (read, not run)"}
  - {host: "validation host", command: "CMS gridpack DarkHiggs_noDHdecay_Zp3000_s251_Chi300 (MG 2.9.18), runcmsgrid.sh 20k events; vibegraph 2431b587 generate 20k events; SCALUP classification and status-2 counts; G*/results.dat channel σ"}
---
# MadGraph's sde_strategy = 2 channel weight before 3.7 reads q² as a mass

At `sde_strategy = 2`, MadEvent's multichannel weight for configuration `c` is
`get_channel_cut`, a product over the configuration's propagators. Because each
event's clustering configuration is distributed as `CC_c / Σ CC` (see
[clustering-configuration-draw](clustering-configuration-draw.md)), this weight
also decides which configuration a `dynamical_scale_choice = -1` scale is
clustered in. The formula changed between MadGraph versions.

## The two forms

In every version `t = dot(ptemp, ptemp)` is the propagator's **squared**
virtuality `q²`.

**MadGraph 2.9.18 to 3.6.x**:[^mg2918][^mg350][^mg360]

```fortran
c  spacelike (tprid != 0)
   get_channel_cut = get_channel_cut / ((t-Mass)*(t+Mass)+stot*1d-10)**2
c  timelike
   tmp  = (t-Mass)*(t+Mass)
   tmp2 = Mass*Width
   get_channel_cut = get_channel_cut* (tmp**2 - tmp2**2)/(tmp**2 + tmp2**2)**2
```

Here `(t-Mass)*(t+Mass)` is `q⁴ − M²`, not `q² − M²`. In GeV units `M² ≪ q⁴`, so
each line contributes roughly `1/q⁸`. There is no Breit-Wigner peak, and the
weight falls steeply with virtuality. The timelike numerator `tmp² − tmp2²` is
also the real part of a squared propagator, which can be negative. Configurations
whose lines are least virtual, typically a small-`|t|` spacelike line, take the
weight.

**MadGraph 3.7.1**:[^mg371]

```fortran
   get_channel_cut = get_channel_cut / (t-Mass**2+stot*1d-10)**2
   tmp = (t-Mass**2)
   get_channel_cut = get_channel_cut/(tmp**2 + tmp2**2)
```

These are `1/(q² − m²)²` and `1/((q² − M²)² + M²Γ²)`, a proper propagator weight.

**vibegraph** transcribes the 3.7.1 form: `ChannelSet::channel_cuts`
(`vibegraph-lib/src/coupling/cluster/graph.rs`). The same function supplies the
weights of the configuration draw and of the colour-flow draw wherever
`sde_strategy = 2`. It has been unchanged since v0.2.2.

## Where it shows: CMS DarkHiggs Zp 3000 (card 880)

The card is `p p > zp > dm dm hs` in `ZpHiggs_UFO`, with MZp 3000, ΓZp 126.9491
(auto), Mχ 300 Majorana, Mhs 251, `sde_strategy = 2`, `dynamical_scale_choice = -1`,
and NNPDF31 (325500). The gridpack was made with MadGraph 2.9.18. On the
gridpack's own sample vibegraph reads σ 0.976× MadGraph, 5σ, stable across seeds
and builds. The deficit is the largest of the DarkHiggs family and grows with
`MZp − (2Mχ + Mhs)`.

**Configurations.** Each `q q̄` subprocess has five configurations (`configs.inc`
of a 3.5.7 `output` of the same card; the gridpack's `symfact.dat` lists the same
channels):

| config | structure | Zp → dm dm line |
|---|---|---|
| c1 | Zp* → Zp hs, Zp → dm dm | yes |
| c2, c3 | Zp* → dm dm*, dm* → dm hs (c3 maps onto c2 by symmetry) | no |
| c4, c5 | q q̄ → Zp hs by t- and u-channel quark exchange (quark–hs Yukawa), Zp → dm dm | yes |

The `|M|²` of c4 and c5 is negligible, because the Yukawa is the quark mass. At
`sde_strategy = 2` the channel weight is the propagator product alone, so they
still draw a share.

**Scale and σ.** The events clustered with a Zp → dm dm line take μF = mT(dm dm).
The rest take μF = √ŝ. The process is purely electroweak, so μR does not enter.

| | μF = √ŝ | μF = mT(dm dm) |
|---|---|---|
| MadGraph 2.9.18 gridpack, 20k events | 17.2 % | 82.8 % |
| vibegraph, 20k events | 62.2 % | 37.8 % |

On card 158, below the region where this matters, both sides put 99 % of events
at mT(dm dm). Reweighting vibegraph's √ŝ events to mT(dm dm) with the PDF ratio,
at MadGraph's fractions, predicts ×1.017. That accounts for about 70 % of the
deficit to first order. The rest was not re-predicted per event.

**The realised split is the conditional draw under each version's formula.**
Here `P(c | p) = CC_c / Σ CC` is evaluated on 61 events of the gridpack's own LHE,
with momenta printed to 6 digits. Each row is compared with the gridpack's channel
σ shares (`G*/results.dat`, summed over the four flavours):

| CC form | P(c1) | P(c2 + c3) | P(c4 + c5) |
|---|---|---|---|
| pre-3.7, as compiled | 0.000 | 0.186 | 0.814 |
| gridpack σ share (MG 2.9.18) | 0.0004 | 0.172 | 0.827 |
| 3.7.1 (vibegraph) | 0.075 | 0.579 | 0.345 |

Under the 3.7.1 form, the predicted share of configurations with a Zp → dm dm line
(c1 + c4 + c5) is 0.42. vibegraph's realised 37.8 % at mT(dm dm) is consistent
with it. Both programs therefore realise their own conditional draw: only the
formula differs. The gridpack's status-2 records agree, with 81.8 % `zp → dm dm`,
17.1 % `zp → dm dm hs` and 1.1 % none.

**The formula was checked inside MadGraph itself.** A 3.5.7 `output` of the same
card reproduces the gridpack's split. For `u u~` the shares are G4 5.56e-4,
G5 5.52e-4 and G2 2.34e-4 pb, with G1.x near 5e-7; this run used nn23lo1, which
changes only the PDFs. The instrumented `matrix1` dumped `AMP2(1..5)` per point.
The instrumented `get_channel_cut` dumped `t`, the mass, the width and the
running factor per line. A Python transcription of the pre-3.7 form reproduces
MadGraph's `AMP2` with a worst relative difference of 2.9e-6, over 12 000 points
× 5 configurations.

**What this does not explain.**
[two-to-six-scale-spectrum-offset-vs-madevent](../backlog/validation/two-to-six-scale-spectrum-offset-vs-madevent.md)
is measured against MadGraph 3.7.1 banks (`validation/manifest.toml`), which
carry the corrected form, so its offset has another cause.

## Consequences

- A reference made with MadGraph before 3.7 at `sde_strategy = 2` and a clustered
  scale differs from vibegraph wherever the two weights pick different
  configurations. μF moves through the PDFs. αs moves as well wherever the
  matrix element carries it.
- Emulating the old form would mean a version switch on `channel_cuts`, which
  reproduces an upstream defect. It is not implemented. Whether to offer it is
  an open design question.
- A card's `sde_strategy = 1` is unaffected: there the weight is `AMP2_c` and
  `get_channel_cut` returns 1, under the default `tmin_for_channel`.

[^mg2918]: MadGraph 2.9.18, `Template/LO/SubProcesses/genps.f:1732` (spacelike) and `:1747-1749` (timelike).
[^mg350]: MadGraph 3.5.0, `genps.f:1885`, `:1900`.
[^mg360]: MadGraph 3.6.0, `genps.f:1918`, `:1933`.
[^mg371]: MadGraph 3.7.1, `genps.f:1934`, `:1949-1951`.
