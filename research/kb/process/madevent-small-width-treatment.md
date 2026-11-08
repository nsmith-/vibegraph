---
type: Design Decision
title: Keep the model's own width below MadEvent's small-width floor
description: "MadEvent evaluates every propagator whose width is below M·1e-6 at the floored width and corrects its Jacobian by fk_W/Γ, so its σ is the narrow-width σ at the true width and its lineshape the floored one's; vibegraph keeps the model's width in both. A registered deviation: 1.5% on a decay chain's bwcutoff window, under 1e-3 without one."
status: draft
tags: [small-width, madgraph-deviation, sigma, decay-chains, breit-wigner]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
decided: 2026-11-04
decided_by: human:nsmith-
sources:
  - {id: g9, resource: "../sprints/v02-diagnoses/sprint.md", title: "Sprint G decision G9: the small-width convention is a registered deviation"}
  - {id: g-v1, resource: "../sprints/v02-diagnoses/sessions/G-V1-report.md", title: "G-V1 report, hypothesis 3: the 1.5% on card 048 predicted and measured"}
  - {id: g-v4, resource: "../sprints/v02-diagnoses/sessions/G-V4-report.md", title: "G-V4 report: p p > h z without a window, and the bound on the residual"}
  - {id: mg-transpole, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/transpole.f#L44-L47", title: "MadGraph transpole.f: the floored width and the fk_W/Γ Jacobian factor"}
  - {id: mg-fk, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L4816-L4820", title: "MadGraph export_v4.py: fk_W = sign(max(|W|, |M|·small_width_treatment), W) in matrix*.f"}
  - {id: mg-banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4452", title: "MadGraph banner.py: small_width_treatment, hidden, default 1e-6"}
measured:
  - {command: "card 048 (p p > vz, (vz > h z), MVz = 3500, WVz = 1 MeV) at nn23lo1 / lhaid 247000, 5 seeds at 100000 × 8 against MadEvent 3.7.1", result: "185426 ± 62 against 182800 ± 123 pb, ratio 1.0144 ± 0.0008, predicted 1.0155"}
  - {command: "p p > h z at the same card and PDF, 25 seeds at 100000 × 8 under the Breit–Wigner τ map, against 5 MadEvent 3.7.1 seeds", result: "185698 ± 23 against 185757 ± 9 pb (MadEvent's seed spread), ratio 0.99968 ± 0.00013; bound from the floor 0 to 1.2e-3"}
---

**Decision (user, 2026-11-04, sprint G decision G9).** Where a propagator's
width Γ is below `M · small_width_treatment` (`1e-6` by default, a hidden run
card parameter), vibegraph keeps the model's own width in the propagator and in
its phase-space map. MadEvent does not. Under the
[defect policy](../validation/madgraph-defect-policy.md) MadEvent's treatment is
an **approximation**, and keeping the width is a **registered deviation**
(outcome 3), a site of the planned MadGraph-compatibility flag. The parity goal
it departs from is in [release scope](../pipeline/release-scope.md), beside the
standing deviation for
[identical particles across decays](identical-particles-across-decays.md).

## What each side computes

- **MadEvent** replaces every width below the floor by `fk_W = M·1e-6` in its
  matrix elements (`matrix*.f` passes `FK_MDL_W*`, defined in `export_v4.py`)
  [^mg-fk]. Where a channel maps the propagator with a Breit–Wigner, `transpole`
  maps it at the floored width and multiplies the Jacobian by `fk_W/Γ`
  (`transpole.f:44-47`) [^mg-transpole]. Over the whole peak the two factors
  cancel: `∫ ds fk_W/Γ /((s − M²)² + M² fk_W²) = π/(MΓ)`, the true width's
  integral. So its σ is the narrow-width σ at the **true** width, while its
  events carry the **floored** width's lineshape.
- **Here** the propagator and the map both use the model's Γ, so σ and the
  lineshape are those of the width the model states. The decay-chain windows
  (`Cuts::compile_with`) floor the window's width exactly as MadEvent's
  `cut_bw` does, so both sides cut at the same `M ± bwcutoff · max(Γ, M·1e-6)`.
- **The standalone output** (`output standalone`) passes the raw `MDL_W*`, so
  the amplitude oracle and every standalone-|M|² comparison are blind to the
  floor. Only a MadEvent σ or event sample sees it.

## How big it is

The cancellation is exact only over the whole line with a constant
coefficient, so the difference is what breaks it:

- **A window cuts the line.** A decay chain's `bwcutoff` window holds
  `(2/π) atan(2δ/W)` of a Breit–Wigner of width `W`, with δ the window's
  half-width. MadEvent's σ carries the floored width's share, vibegraph's the
  true width's. On card 048 (`MVz = 3500`, `WVz = 1 MeV`, δ = 15 × 3.5 MeV) that
  is 0.99394 against 0.97879, a ratio of **1.0155 predicted**; measured
  **1.0144 ± 0.0008** (vibegraph 185426 ± 62 pb against MadEvent 182800 ± 123
  at `nn23lo1`) [^g-v1].
- **Without a window** (`p p > h z` through the same `Vz`) only the off-peak
  part is mis-weighted: MadEvent's factor `fk_W/Γ` multiplies the whole
  channel, so the width-independent part `K` of the channel's σ (the
  propagator's tails and its interference with the other diagrams) is counted
  `fk_W/Γ` times. The offset is at most `(fk_W/Γ − 1) K`. Measured here,
  `K = (fk_W σ(fk_W) − Γ σ(Γ))/(fk_W − Γ) = 87 ± 16 pb` of 185698 pb, a bound of
  **1.2e-3**; MadEvent reads **+3.2e-4 ± 1.3e-4** above vibegraph (185757 ± 9
  against 185698 ± 23 pb) [^g-v4].
- **The lineshape** is MadEvent's floored width's in both cases. On card 048
  the mean `|m_Vz − MVz|` over 10000 unweighted events is 1.48 MeV here
  (predicted 1.49 for Γ = 1 MeV) against 3.86 in MadEvent's (predicted 3.87
  for 3.5 MeV); on `p p > h z` the share within 0.5 MeV of the pole is 0.489
  here (predicted 0.500) against 0.176 in MadEvent's (predicted 0.177).

In a multichannel process MadEvent applies `fk_W/Γ` only to points drawn in a
channel that Breit–Wigner-maps the floored propagator. `p p > h z`'s second
channel (the `Z` exchange) carries the rest of `K` uncorrected, which is why the
measured offset sits below the bound.

## How it is gated

- A σ row with a floored width is compared with MadEvent through this
  deviation: on a decay chain at the predicted window ratio, without one within
  the `K` bound.
- Event-level comparisons of the floored line's invariant mass are
  informational: the lineshapes differ by design.
- With the MadGraph-compatibility flag on, the floor would be reproduced: the
  propagator at `fk_W` and the Breit–Wigner map's Jacobian scaled by `fk_W/Γ`.

[^g9]: Sprint G decision G9, 2026-11-04 (user).
[^g-v1]: G-V1 report, hypothesis 3 and the lineshape table.
[^g-v4]: G-V4 report, the MadEvent comparison and the small-width bound.
[^mg-transpole]: `Template/LO/Source/transpole.f:44-47`.
[^mg-fk]: `madgraph/iolibs/export_v4.py:4816-4820`, the `fk_` width definitions in `matrix*.f`.
[^mg-banner]: `madgraph/various/banner.py:4452`, the run-card parameter.
