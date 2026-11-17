---
type: Session Report
title: "K-P1 report: one-body final-state phase space"
description: "2 → 1 processes at proton beams integrate through a δ(ŝ − M²) one-body map (τ = M²/s, as genps.f) and match MadGraph (p p > z: pull −1.44 over 20 against 5 seeds); on fixed-energy beams MadGraph returns zero, and vibegraph keeps a named refusal."
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
| `76ec03f` | kb: this report, draft |
| `5dd8a76` | fix(multiplicity): state what MadEvent does with a fixed-beam 2 -> 1 |
| (final) | this report; `one-body-final-state-phase-space-missing` deleted |

## What MadGraph does (genps.f, b7687064)

- **Proton beams.** `genps.f:336-341`: for `nexternal.eq.3` the τ coordinate is
  drawn and then overwritten, `x(ndim-1) = pmass(3)²/stot`, with
  `sjac = 1/stot`, the δ(τ s − M²) integrated over τ. `GENCMS` then multiplies
  by `taumax − taumin = 1` and draws η flat over ±½ ln τ. `one_tree`
  (`genps.f:772`) sets `p3 = p1 + p2`. The flux is `1/(2ŝ)` divided by
  `(2π)^(3·1−4)`, a factor 2π. So
  `σ = ∫ dy (x₁f₁)(x₂f₂)/τ · (1/s) · 2π|M̄|²/(2ŝ)`. The width does not enter;
  the pole mass fixes τ, and the τ coordinate stays in the grid unread.
- **Fixed beams (lpp = 0).** Reading `gen_mom` alone, `s(-nbranch) = stot`
  and `sjac = 1` with no δ, which predicted a reported `2π|M̄|²/(2s)` (a δ
  coefficient). **Measured, that prediction is wrong**: the driver configures
  the grid with `3·1 − 4 = −1` dimensions (`G1/log.txt`: "-1 dimensions 1000
  events"; the `ndim < 0` clamp in `genps.f:234` is not what the driver
  sees), the survey's iteration 0 is `NaN`, every later iteration `0`, and
  `generate_events` stops with "Survey return zero cross section", at
  √s = 91.188 GeV and at √s = 200 GeV alike. MadGraph has no fixed-beam σ for
  a 2 → 1.

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

### Banked layer

`pixi run --skip-deps validate` (which runs `validate_lhef` and
`validate_sigma`): CI banked on the PR head carrying this report. The local
run was started and stopped by the manager before its first gate ran (its log
ends at `Compiling vibegraph v0.2.1`): not a result. The result is recorded
from CI.

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

Local runs, `kp1-mg.sh` (fresh copy of the process directory per seed,
`iseed` set, `nb_core = 2`), results.dat field 1 and 2:

```
p p > z, dy13_default_run_card          e+ e- > z, lpp = 0
1 0.42118E+05 0.41931E+01               √s = 91.188: 1 0.00000E+01 0.00000E+01
2 0.42118E+05 0.43400E+01               √s = 200:    1 0.00000E+01 0.00000E+01
3 0.42126E+05 0.44856E+01               ("Survey return zero cross section")
4 0.42117E+05 0.43474E+01
5 0.42126E+05 0.45115E+01
MG n=5 mean=42121.0 pb sd=4.58 sd/sqrtn=2.05 quoted/n=1.96 chi2/dof=1.08

matched p p > z (j), pp_to_llj_mlm_run_card
1 0.57201E+05 0.57719E+02
2 0.57225E+05 0.55898E+02
3 0.57220E+05 0.66422E+02
MG n=3 mean=57215.3 pb sd=12.7 quoted/n=34.8
```

- `p p > z`: vibegraph 20-seed mean 42115.35 pb (error max(sd/√n, quoted/n)
  = 3.35) against MadGraph 42121.0 ± 2.05: difference −5.65 pb
  (−1.3 × 10⁻⁴), **pull −1.44** on 3.93 combined. Both seed sets are
  self-consistent (χ²/dof 0.93 and 1.08).
- Matched `p p > z (j)`: vibegraph 57192.6 ± 14.1 (5 seeds) against
  MadGraph 57215.3 ± 34.8 (3 seeds): −22.7 pb, pull −0.60. Recorded, not
  gated; 3 MadGraph seeds cannot calibrate the spread.
- `e+ e- > z`: MadGraph returns zero; vibegraph refuses by name
  (`refuse_one_body_final_state`), as the manager's rule for this case asks.

What the hadronic comparison cannot see: anything in |M̄|² that is constant
over the rapidity, apart from the 1.3 × 10⁻⁴ it resolves (both sides use the
same couplings and pole mass); MadGraph's `stot` with the proton mass differs
by m_p²/s ≈ 5 × 10⁻⁹, far below it.

### Events

`generate` on the p p > z grid writes the Z (status 1, mothers 1 2, mass
91.188) as p1 + p2 boosted along z, the record MadGraph writes:

```
vibegraph:  23  1    1    2    0    0 +0.0 -0.0 -3.3307392693e+02 3.4533098926e+02 9.1188000000e+01
MadGraph:   23  1    1    2    0    0  0.0 -0.0 -0.24636391836E+04 0.24653262012E+04 0.91188000000E+02
```

On the matched card a 0-jet event carries `<scales pt_clust_3="13000.00000">`
and `<mgrwt>` with `<rscale> 0 0.91188000E+02`, `<asrwt>0`, as MadGraph's
does. `validate-lhef` is part of the banked gate below.

## Brief corrections

- The item's `closes_when` and the brief ask for e+ e- > z σ against
  MadGraph on a fixed-beam card. MadGraph has none: it returns zero (NaN
  survey) at √s = M_Z and at 200 GeV. The fixed-beam oracle is the named
  refusal, and the item should close on the hadronic card alone. The
  plan's prediction (from reading `gen_mom` alone) that MadGraph would report
  the δ coefficient `2π|M̄|²/(2s)` was refuted by the run. MadGraph's own
  output, `e+ e- > z` at `ebeam = 45.594` (`SubProcesses/P1_ll_z/G1/log.txt`
  and the `generate_events` log):

  ```
  -1 dimensions   1000 events  2 invarients  5 iterations  1 config(s),  (0.99)
  Masses: 0.000E+00 0.000E+00 0.912E+02
   Iteration  0   Mean:        NaN Abs mean:        NaN   Fluctuation:        NaN   0.186E+11   *****%
   Iteration  1   Mean: 0.0000E+00 Abs mean: 0.0000E+00   Fluctuation:        NaN   0.000E+00   100.0%
  Survey return zero cross section.
  ```

  and at `ebeam = 100` the same "Survey return zero cross section", with
  `results.dat` `0.00000E+01 0.00000E+01` for both.
- The brief names MadGraph 3.5.7; the reference is the pinned 3.7.1
  (b7687064), which the banked bundle was made with.

## Found

- **MadGraph's fixed-beam 2 → 1 is a silent zero.** MadEvent configures a
  −1-dimensional grid for `nexternal = 3` at `lpp = 0`, its survey is `NaN`
  and the run reports σ = 0 with the generic "zero cross section" advice
  (widths, PDFs, cuts), none of which is the cause. Nothing for vibegraph to
  do beyond the named refusal it has; worth one line in the run-card or
  refusal documentation, so a user comparing against MadGraph's 0 is not
  sent looking for a width.
- **The item's `closes_when` names a fixed-beam MadGraph σ that does not
  exist.** Closing on the hadronic card plus the named fixed-beam refusal is
  what was measured here; the manager should record that reading when the
  item is deleted.
- **The matched `p p > z (j)` card has no banked row.** Measured here at
  pull −0.60 on 3 MadGraph seeds; a seeded-σ row (≥ 5 seeds a side) would
  turn it into a gate. Closing it means banking the row with
  `bank_seeded_sigma.sh` and adding it to the manifest.
