---
type: Session Report
title: "H-B1 report: group B's σ deficit is an IR-divergent card, not the u b~ / u~ b split"
description: "The four CKM γ+jets gridpacks run with draj = 0 and no photon isolation (ptgmin = 0), so σ diverges logarithmically in the quark–photon collinear limit: vibegraph's σ grows by 0.019 pb (4.3% of the regulated σ) per unit of ln(1/ΔR²) over draj 0.4 → 0.004 with a constant slope. Neither MadGraph's survey σ nor vibegraph's is a number the other can be gated against. The spacelike-floor cap and the flavour split were tested as causes and do not carry the deficit on the regulated cards. PARTIAL: committed early against container restarts."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-07}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
Branch `sprinth-ckm-groups-sigma` in `/home/user/wt-ckm`, from `origin/main` at
`9d1e9e6f`. Scratch (cards, binaries, drivers, logs, MadGraph directories) is
under `/home/user/H-B1-scratch/`; none of it is committed. The survey cards are
read from `origin/sprinth-diagnosis-cards` (`1795adea`), extracted with
`git archive` into `/home/user/H-B1-scratch/cards/`.

**Status: partial.** This file is committed early because the container
restarted five times during the session; sections marked *pending* are filled
in by later commits on this branch.

## Binaries

- `bin/vibegraph-main`: `cargo build -p vibegraph --bin vibegraph` (dev profile,
  `opt-level = 2`) at `9d1e9e6f`, unmodified.
- `bin/vibegraph-probe`: the same tree plus `H-B1-probe.patch` (uncommitted,
  scratch only). With no `VG_PROBE_*` variable set it is the main binary's
  code path. `VG_PROBE_SPACELIKE_FLOOR=zero` makes `Cuts::spacelike_floor`
  return 0 (no peripheral `t` cap; the pole is floored at the channel default);
  `=min` returns the smallest single-leg `pT_min²` over the final legs.
  `VG_PROBE_GROUP_REL_TOL=<x>` replaces `GROUP_REL_TOL` in the partition.

σ is read from the `--output-ldjson` result record (`sig.py`, unit pb).

## The survey cards are IR-divergent

All four group-B gridpacks (`run_card.dat` as the gridpack used it):

```
  0.0  = ptj      0.0 = drjj     0.0 = draj     0.0 = ptgmin (no isolation)
 -1.0  = etaj    -1.0 = etaa
  pta / ptamax / mmjj: 10/-/4 (GJ incl), 40/-/4 (GJ PTG-40),
                       100/200/100 (VBFtoG 100to200), 200/-/100 (VBFtoG 200)
```

`p p > a j j QCD=0 QED=3` includes `q q' → q q' γ` with the photon radiated
off a final-state quark. With `draj = 0` and no isolation, the quark–photon
collinear limit is uncut and |M|² goes as `1/s_qγ` there, so σ diverges
logarithmically. Measured directly by adding the regulator back:

Base: the VBFtoG PTG-200 gridpack run card with `lhaid 247000`, `use_syst
False`, `gridpack False`, `nevents 10000` (`vg/C0_run_card.dat`); scan cards
`vg/S_ptj<x>_draj<y>.dat` change only `ptj` and `draj`. Proc card `import
model sm-ckm_no_b_mass` / `generate p p > a j j QCD=0`. Each point: `integrate
--seed 1 --fixed-budget` (12 × 120000), `LHAPDF_DATA_PATH` the worktree's
`validation/pdf`.

```
$ cat vg/H-B1-scan-driver.log vg/H-B1-scan2-driver.log   (vibegraph-main)
0.433334 0.001504  chi2/dof=1.720  H-B1-scan-S_ptj20_draj0.4.log
0.489497 0.001689  chi2/dof=1.401  H-B1-scan-S_ptj20_draj0.1.log
0.547002 0.001828  chi2/dof=1.532  H-B1-scan-S_ptj20_draj0.02.log
0.607749 0.002234  chi2/dof=1.661  H-B1-scan-S_ptj20_draj0.004.log

draj 0.4->0.1: dsigma=+0.0562 pb  per unit ln(1/dR^2) = 0.0203
draj 0.1->0.02: dsigma=+0.0575 pb  per unit ln(1/dR^2) = 0.0179
draj 0.02->0.004: dsigma=+0.0607 pb  per unit ln(1/dR^2) = 0.0189
```

A constant slope of 0.019 pb per unit of `ln(1/ΔR²)` — 4.3% of the
`draj = 0.4` σ per e-fold — over two decades of `draj`: the textbook
collinear logarithm, and no sign of saturation. At `draj = 0` the integral
is infinite. What any program reports for it is set by where its sampler
stops resolving the collinear region (MadEvent: its VEGAS grid on the
`s_qγ` invariant and double precision; vibegraph: its channel maps and
budget). That is exactly the budget dependence the CERN ladder reads
(PTG-200 0.738 → 0.915 at 10× budget): more points reach further into the
logarithm, and the number grows without a limit to converge to.

The jet-pT direction, by contrast, saturates (`ptj = 0` is harmless here at
the level these runs resolve): with the cap removed (`zero`), `draj = 0.4`,

```
$ cat vg/H-B1-scan3-driver.log   (vibegraph-probe, VG_PROBE_SPACELIKE_FLOOR=zero)
0.432966 0.000828  chi2/dof=1.893  H-B1-scan-zero-S_ptj20_draj0.4.log
0.470583 0.001003  chi2/dof=1.604  H-B1-scan-zero-S_ptj1_draj0.4.log
0.4694   0.001063  chi2/dof=1.693  H-B1-scan-zero-S_ptj0.01_draj0.4.log
0.468846 0.001094  chi2/dof=1.740  H-B1-scan-zero-S_ptj0.0001_draj0.4.log
```

`ptj` 1 → 1e-4 moves σ by −0.4% ± 0.3%: flat.

## Hypotheses tested and set aside

### The spacelike-floor cap

`Cuts::spacelike_floor` is the largest single-leg `pT_min²` over all legs:
`pta² = 40000 GeV²` on the VBFtoG-200 card, though the jets carry `ptj = 0`.
Every peripheral rung's transfer is capped at `t ≤ −floor`, so a rung that
emits a soft jet is outside every spine channel's support and is reached only
by the three all-timelike channels. That is a real coverage gap in principle
(its docstring's "for a three-body final state [the floor] is always [a
bound]" does not hold when the hardest cut is not on the emitted leg). It is
not what carries B:

```
C1 = H-N1's run card with pta 10 -> 200 (ptj 20 < pta), --target-rel 0.005, seed 1
  vibegraph-probe (no env, = main)   0.480597 0.002054  chi2/dof=1.469
  VG_PROBE_SPACELIKE_FLOOR=zero      0.479433 0.00149   chi2/dof=1.608
S_ptj20_draj0.4, fixed budget, seed 1
  main 0.433334 0.001504   zero 0.432966 0.000828
```

Agreement at 0.2–0.5% where the cap is active against `ptj = 20` jets. At
`ptj ≤ 1` the capped run reads about 1% under the uncapped ones (main
`ptj 0.2` 0.46437 ± 0.00185 against zero `ptj 0.01` 0.4694 ± 0.0011; main
`ptj 1` 0.4742 ± 0.0034 with an effective sample size of 18822 against
60000–78000 on the neighbours, a single large weight): a percent-level
under-coverage, one seed a point, not the 25–55% of B. *Pending: a seed sweep
of main against zero at `ptj = 0.01`.*

### The u b~ / u~ b split

Forced merge (`VG_PROBE_GROUP_REL_TOL=1e-5`) against kept apart, same binary,
seed 1, `--fixed-budget` (12 × 120000), on the as-shipped VBFtoG-200 card
(`S_asshipped.dat` = `C0_run_card.dat`) and on the regulated
`S_ptj20_draj0.02.dat`:

```
$ cat vg/H-B1-split-driver.log; grep 'channels share|support:' (per log)
kept apart  as-shipped  refused: "the cross section is carried by an effective 14.8 points,
                        below the 20 it is accepted on"; last iteration 3.476e-9 ± 9.03e-10 GeV^-2
                        (1.35 ± 0.35 pb), χ²/dof 13.3; ESS 15
            544 (group, diagram) channels share 27 distinct phase-space maps
merged      as-shipped  1.3001 0.2688  chi2/dof=15.931; ESS 23
            535 (group, diagram) channels share 27 distinct phase-space maps
kept apart  draj 0.02   0.547002 0.001828  chi2/dof=1.532; ESS 83263   (bit-identical to vibegraph-main)
merged      draj 0.02   0.54631  0.001791  chi2/dof=1.559; ESS 86072
```

The merge took (535 against 544 group–diagram pairs: the pair's nine diagrams
now ride one group). On the regulated card the two agree to −0.13% ± 0.47%,
with the same χ²/dof and effective sample size; on the as-shipped card both
are a handful of draws (ESS 15 and 23, χ²/dof 13–16) and agree within their
26% errors. Merging does not change the sampling: the channel grids are per
distinct map, not per group (27 maps, 27 grids in both runs), so keeping the
pair apart adds one term to the integrand mixture but splits no grid's
budget, unlike the VBF_HH mass split. The split is not B's cause.

### The mirror term (the VBF_HH mechanism)

`FlavorGroup::mirror_into` keeps the beams and rotates the outgoing legs, so
a group's mirror term peaks where the beam–jet pairing is exchanged; if no
channel samples that peak, the mirror half reads low. Instrumented in the
scratch probe (`H-B1-probe4.patch`, `VG_PROBE_MIRROR=1`): `group_term` and
`per_group_sum` record each group's `spin_color_average · L · |M|²` for the
direct and the mirrored ordering at the current point; the VEGAS block loop
(`vegas.rs`, after `f · wgt`) apportions the point's jacobian-weighted value
over those terms and accumulates per channel; the dump is
`Σ_channels (Σ points)/n_points`, i.e. the run's own σ decomposition. Its total
reproduces σ (1.40286e-9 GeV⁻² against the run's 1.405e-9 = 0.547 pb; the dump
is at 1.5M of the run's 1.51M evaluations). By pp symmetry with rapidity-even
cuts each mirrored group's ratio is 1 in expectation.

```
$ VG_PROBE_MIRROR=1 vibegraph-probe4 integrate ... --seed 1 --fixed-budget   (vg/H-B1-mirror4-*.txt)
card              groups (share>1e-3, mirrored)  mirror/direct min..max  <1 / >1   Σmirror/Σdirect  max point / σ
draj 0.02  (reg)  28                             0.987 .. 1.005          21 / 7    0.9954           118
as-shipped        28                             0.765 .. 3.079           3 / 25   1.1555           2.66e4
```

On the regulated card every mirrored group's halves agree to ≤ 1.3% (the
dominant `u d → γ u d` group, 43% of σ: 0.9973), and the u b~ / u~ b groups
read 1.0006 and 1.0007: the mirror peak is sampled. On the as-shipped card the
ratios scatter both ways, mostly *above* 1, and single points carry up to
2.7·10⁴ times σ (`maxd`/`maxm` 4e-5–9e-5 GeV⁻² on groups whose whole term is
~1e-10): spikes in either ordering from the collinear region, not a starved
mirror peak (which would read systematically below 1).

The effective sample size is the sharpest reading of the divergence: the
same binary, card family and budget gives ESS 60000–86000 at any finite
`draj` and 15–23 at `draj = 0`.

## Pending

- Local MadGraph on the same regulated cards (`mg/M_ptj20_draj0.4.dat`,
  `M_ptj20_draj0.02.dat`, `M_ptj0_draj0.dat`, H-N1's card with `ebeam 6800`,
  `pta 200`, `mmjj 100`, `drjj 0`, `etaj/etaa -1`) against vibegraph on the
  identical files.
- Task 1, H-N1's card on this branch.
