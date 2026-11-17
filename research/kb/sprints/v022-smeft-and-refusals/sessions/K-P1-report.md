---
type: Session Report
title: "K-P1 report: one-body final-state phase space"
description: "2 → 1 processes at proton beams integrate through a δ(ŝ − M²) one-body map (τ = M²/s, as genps.f); fixed-energy beams keep a named refusal because MadEvent's number there is a δ coefficient, not a cross section."
status: draft
tags: [phase-space, one-body, resonance, hadronic, report, validation, sprint-k]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-one-body`, branch `sprintk-one-body`, based on
`main` at `703f57f`. MadGraph is the pinned submodule (3.7.1, b7687064), run
from a copy under `/home/user/wt/sprintk-one-body-mg/`. The host has 4 cores,
shared with sibling sessions.

| commit | what |
|---|---|
| `9719fb3` | feat(phasespace): integrate 2 -> 1 processes at proton beams |

## What MadGraph does (genps.f, b7687064)

- **Proton beams.** `genps.f:336-341`: for `nexternal.eq.3` the τ coordinate is
  drawn and then overwritten, `x(ndim-1) = pmass(3)²/stot`, with
  `sjac = 1/stot`, the δ(τ s − M²) integrated over τ. `GENCMS` then multiplies
  by `taumax − taumin = 1` and draws η flat over ±½ ln τ. `one_tree`
  (`genps.f:772`) sets `p3 = p1 + p2`. The flux is `1/(2ŝ)` divided by
  `(2π)^(3·1−4)`, a factor 2π. So
  `σ = ∫ dy (x₁f₁)(x₂f₂)/τ · (1/s) · 2π|M̄|²/(2ŝ)`. The width does not enter;
  the pole mass fixes τ, and the τ coordinate stays in the grid unread.
- **Fixed beams (lpp = 0).** `s(-nbranch) = stot`, `sjac = 1`, no δ:
  `p3 = p1 + p2` is off shell at mass √s, and MadEvent reports
  `2π|M̄|²(s)/(2s)` in pb, which is the coefficient of δ(s − M²), not a
  cross section. (To be confirmed by the local run below.)

## What changed

- `phasespace/diagram_channel.rs`: `ChannelTopology::OneBody` for a single
  outgoing leg. ndim 0; it puts `(√ŝ, 0, 0, 0)` on the leg with weight and
  density 1, so the δ stays with the caller that draws ŝ. Own map key and map
  identity; every builder leaves it alone.
- `proton.rs`: `ProtonIntegrand::one_body_tau` fixes τ = M²/s; the outer map's
  Jacobian is `2 y_max/M²` (GeV⁻²). The grouping probe for one outgoing leg is
  the single on-shell configuration at √ŝ = M. `ProtonError::OneBodyFinalState`
  is gone; in its place `OneBodyCutToZero` (the drop-from-sum path, unchanged
  behaviour), `MasslessOneBody` and `OneBodyOutOfReach` (refusals).
- `multiplicity.rs`: the rename, and the fixed-beam refusal now says why.
- Touched beside PR 9: `derive_flavor_groups`' one-body block before the probe,
  and the one match arm in `multiplicity_groups_of`.

## Oracles

### Hermetic

`cargo test --workspace`: 58 suites, 2118 passed, 0 failed (`TEST_RC=0`).
`cargo clippy --workspace --all-targets -- -D warnings`: clean.

`a_one_body_point_is_the_narrow_resonance_cross_section_times_luminosity`:
the integrand of `u u~ > z` equals `4π²Γ(Z → uū)/(3M) · lumi · 2y_max/(τs)` to
1e-10 at 50 points, Γ from the decay integrator's two-body phase space; the τ
coordinate is unread. It cannot see the densities' normalisation or an error
common to the decay and production matrix elements (a coupling).

### Banked σ lines unchanged (bit for bit)

main `703f57f` and the branch, same seed (11), `--neval 40000 --fixed-budget --niter 4`
(`kp1-bitdiff.sh`):

```
ee_mumu rc=0 σ = 419.60 ± 0.38 fb grid=d025bd28b24531df   (both)
dy13    rc=0 σ = 938.2 ± 4.6 pb grid=f4a9a7e9d6c48876     (both)
llj_mlm rc=0 σ = 267.9 ± 1.3 pb grid=ce8a8a69ac756b24     (both)
diff → IDENTICAL
```

Fixed-beam (`e+ e- > mu+ mu-`, config_share_ee_mumu card), hadronic
(dy13 default), MLM (`p p > e+ e- j`, pp_to_llj_mlm card): the grid artifacts'
SHA-256 agree, so the channels, α and grids are bit-identical. Structurally,
every changed path is gated on one outgoing leg.

### vibegraph seed sweeps

`p p > z`, dy13_default_run_card (NNPDF23 lhaid 247000, μ = 91.188), seeds
1–20, default target 0.1%:

```
n=20 mean=42.11535 nb  spread(sd)=0.01450  sd/sqrt(n)=0.00324  quoted/n=0.00335  chi2/dof=0.93
```

Matched `p p > z @0` + `p p > z j @1`, pp_to_llj_mlm_run_card (ickkw 1,
xqcut 20), seeds 1–5:

```
n=5 mean=57.19260 nb  spread(sd)=0.03160  sd/sqrt(n)=0.01413  quoted/n=0.01431  chi2/dof=0.98
```

### Against MadGraph

Pending: local MadGraph runs (p p > z × 5 seeds, e+ e- > z at √s = 91.188 and
200 GeV, the matched card).

### Events

`generate` on the p p > z grid writes the Z (status 1, mothers 1 2, mass
91.188) as p1 + p2 boosted along z. `validate-lhef` pending.

## Brief corrections

The item's "e+ e- > z σ against MadGraph" oracle assumes MadGraph returns a
cross section at fixed beams; it returns the δ(s − M²) coefficient in pb
(pending confirmation). vibegraph refuses by name.

## Found

(to be completed)
