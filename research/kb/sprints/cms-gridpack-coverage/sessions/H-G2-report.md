---
type: Session Report
title: "H-G2 report: three panics and a wrong refusal become typed outcomes, and uncut photon splittings are refused"
description: "A 2 → 1 subprocess is refused by name before the grouping probe reaches RAMBO (proton and fixed-energy paths), and a one-body part the cuts zero outright (DarkPhoton's lone zp under pt_min_pdg) is dropped from the multiplicity sum, as in MadGraph. Invariant draws are clamped to their range, and points with degenerate outgoing momenta or a non-finite kT measure are zeroed and counted; idm_dilepton integrates (39 of 17.5M points degenerate). The setup scale probe draws across channels and refuses only on factorisation-floor evidence; QCD-4Jets_HT-100to200, refused before after one xqcut-vetoed probe point, integrates. A card that leaves a photon's collinear splitting off a massless coloured parton uncut (draj = 0, no isolation) is refused; the four group-B cards now read refused."
status: draft
tags: [proton, cuts, phase-space, scales, matching, ir-safety, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-07}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: mg-cuts-iso, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L1104", title: "MadGraph cuts.f photon isolation (ptgmin, R0gamma, xn, epsgamma, isoEM)"}
  - {id: mg-setcuts-r2min, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L340", title: "MadGraph setcuts.f pairwise ΔR minima: draj for a–j, drab for a–b"}
  - {id: mg-banner-draj, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4520", title: "MadGraph banner.py: draj discarded when photon isolation is used"}
---
Branch `sprinth-guards` in `/home/user/wt-guards`, from `origin/sprinth-mirror-rotation`
`2905aada`; later merged with `origin/sprinth-mirror-rotation` (`24116d71`, kb index only)
and `origin/main` (`082298c1`, #107 squashed; clean merge, no conflicts). Scratch
(cards, binaries, drivers, logs, MadGraph directories) is `/home/user/H-G2-scratch/`;
nothing there is committed. The diagnosis cards are `origin/sprinth-diagnosis-cards`,
extracted with `git archive` into `/home/user/H-G2-scratch/validation/...`.

## Binaries and drivers

- `bin/vibegraph-base`: H-V2's `release-debug` build of exactly `2905aada`
  (`/home/user/H-V2-scratch/target/release-debug/vibegraph`, copied before any rebuild).
- `bin/vibegraph-g1`: this branch before G3's probe tally (G1/G2/G4 code); used only for the
  first DarkPhoton runs.
- `bin/vibegraph-g2`: `611b2389` (all four guards and the probe debug line), `release-debug`.
- `bin/vibegraph-probe-ch0`: `611b2389` with the probe forced onto channel 0 and a debug
  line per scaled draw; built once to verify G3's diagnosis, source restored afterwards.
- `scripts/rungp.py LABEL BINARY WORKDIR`: stages a card as `vgrun.py` does (the bundled
  `ufomodel/` under `--ufo-dir`, `restrict_*.dat` copied from `restrict_default.dat`),
  patches `lhaid` to 247000 (the only NNPDF LO set here; the gridpacks use 325300/325500,
  so no σ below is comparable to `source.json` except where a local MadGraph run says so),
  and runs `integrate ... --neval 10000 --niter 3 --fixed-budget --seed 1 -j 4
  --parallel-diagrams --output-ldjson --no-network` unless stated.

## G1: 2 → 1 subprocesses

### Diagnosis, verified

```
$ RUST_BACKTRACE=1 vibegraph-base integrate g1/ppz.dat ...        (import model sm / generate p p > z)
thread 'main' panicked at vibegraph-lib/src/phasespace/rambo.rs:52:5:
RAMBO needs at least two final-state momenta
   2: vibegraph::phasespace::rambo::rambo::<f64>
   3: vibegraph::proton::derive_flavor_groups
   4: vibegraph::multiplicity::multiplicity_groups
$ vibegraph-base integrate g1/eez.dat --run-card run_card.dat ...  (e+ e- > z, lpp = 0, ebeam 45.6)
thread 'main' panicked at vibegraph-lib/src/phasespace/rambo.rs:52:5:
   3: <vibegraph::hadronic::FixedBeamIntegrand>::use_running_coupling
   4: vibegraph::integrate::integrate_fixed_energy
$ rungp.py DarkPhoton-...-MZd-40 vibegraph-base runs/base-dp40
runs/base-dp40/output.txt:484: thread 'main' panicked at vibegraph-lib/src/phasespace/rambo.rs:52:5:
RAMBO needs at least two final-state momenta
```

The fixed-beam path panics too, one step later (the scale probe in
`use_running_coupling`, which draws with RAMBO).

### Fix

- `ProtonError::OneBodyFinalState { process, cut_to_zero }` (proton.rs), raised in
  `derive_flavor_groups` after every subprocess's evaluator and cuts are compiled and before
  `probe_momenta`, next to `NotTwoIncoming`. The message names the missing one-body
  `δ(ŝ − M²)` phase space.
- `cut_to_zero` is `Cuts::rejects_zero_transverse_momentum()` over every subprocess of the
  part: some outgoing leg has a positive `pT` minimum. At collinear beams a lone outgoing
  particle has `pT = 0` exactly, and `cuts.f` (and `Cuts::pass`) fail `pt < pt_min`, so the
  part is zero by construction.
- **Multi-multiplicity cards** (the decision asked for): `multiplicity_groups` drops a
  one-body part whose `cut_to_zero` holds, with a warning naming it, and keeps the other
  multiplicities; that is MadGraph's result exactly (its `p p > zp` is cut to zero by
  `pt_min_pdg = {1023: 100}`). A one-body part the cuts do not zero refuses the whole card,
  because its contribution would be missing from the sum. A card that is nothing but
  cut-to-zero one-body parts is refused as zero by construction.
- `refuse_one_body_final_state` (multiplicity.rs) for the one-phase-space paths, called in
  `integrate_fixed_energy` (CLI) and in `generate.rs` beside `refuse_mixed_multiplicity`.

### Tests (fail before, pass after)

`multiplicity::tests::a_single_outgoing_particle_is_refused_before_the_probe`,
`a_one_body_part_the_cuts_zero_is_left_out_of_the_sum` (`p p > z` + `p p > z j` with
`pt_min_pdg {'23': 10}` keeps only the `z j` part; `p p > z` alone is refused; `{'23': 0}`
refuses), `fixed_energy_refuses_a_single_outgoing_particle`. Before (all fixes reverted,
`H-G2-revert-fixes.patch`, see "Fails before"): the first two panic in `rambo.rs:52`, the
third returns `Ok(())`.

### Cards

```
$ vibegraph-g2 integrate g1/ppz.dat ...
ERROR failed to decompose into flavour groups: subprocess u u~ > Z has a single outgoing particle, whose phase space is the one-body δ(ŝ − M²); vibegraph has no one-body phase space to integrate it over
$ vibegraph-g2 integrate g1/eez.dat --run-card run_card.dat ...
ERROR e+ e- -> Z has a single outgoing particle, whose phase space is a one-body δ(s − M²); vibegraph has no one-body phase space to integrate it over
$ rungp.py DarkPhoton-...-MZd-40 vibegraph-g1 runs/g1-dp40      (no param card, as the campaign's attempt 2)
WARN u u~ > Zp and the other single-particle final states of its phase space contribute nothing: ... the part is left out of the sum
  (same for c c~, d d~, s s~ > Zp)  →  exit 0, σ = 0.2670 ± 0.0141 pb, ESS 350
```

Without the param card both DarkPhoton cards run the model default `MZd = 20` and give the
same σ (0.26699 pb, bit-identical), so that number is not either card's. The param card is
refused only for its `chi` (1000022) mass and `Zd → chi chi` line, which the model lacks;
with those two lines removed (`runs/dp{40,86}-param.dat`):

```
$ rungp.py DarkPhoton-...-MZd-40 vibegraph-g2 runs/g2-dp40-pc --param-card --param-file runs/dp40-param.dat
dp40 σ = 0.2391 ± 0.0066 pb     MadGraph (lhaid 325500, survey) 0.2167     ratio 1.10
dp86 σ = 4.612 ± 0.152 pb       MadGraph (lhaid 325500, survey) 4.431      ratio 1.04
```

(PDF differs: 247000 here. Not a σ verdict, an outcome: both cards integrate.)

## G2: idm_dilepton_mH80_mA160

### Diagnosis, verified

```
$ rungp.py idm_dilepton vibegraph-base runs/base-idm --param-card --set small_width_treatment=0.000001
label=idm_dilepton exit=101 wall=301.3s
thread '<unnamed>' panicked at vibegraph-lib/src/proton.rs:3214:33:
per-event scale on a sampled point: the event carries no cluster scale: Clustering(NoAdmissiblePair)
   2: <vibegraph::proton::ProtonIntegrand>::shape
```

The mechanism, link by link:

- `draw_invariant`'s BW branch is unclamped, and its edges are reached only to rounding.
  The test below finds `x = 0` for the Z (91.1876, 2.4952) over `[0.04468996, 1e4]` giving
  `s = 0.044689959971947246 < lo`; at `x = 1` the draw overshoots `hi` (a 160 GeV,
  2.6 MeV line over `[·, 1.69e8]` by ~1e-8 relative, `tan` near π/2). A pair below its
  threshold, or a sibling above what the parent has, gets `p* = 0` (`kallen(...).max(0)`),
  and at `s ≤ 0` `sample_branch` sets every daughter momentum to zero (`sqrt_s > 0` false).
- The point's map weight `r2_factor` is 0 there, but the proton integrand weights a point by
  `αⱼ/g(p)` recomputed from the momenta, not by the map weight, so `shape` still clusters it.
- `kt::dj` with a massless leg at zero momentum: `η = ½ ln((a + p_z)/(a − p_z)) = ½ ln(0/0)`,
  NaN; the NaN measure loses every `<` comparison, and with the `ν ν̄` pair the only
  admissible one the clustering returns `NoAdmissiblePair`, which `scales_at` panics on.
  The kt test below shows `dj(0, 0)` is NaN and the old code's result is
  `Err(NoAdmissiblePair)`.

I did not catch the specific failing point in the act (no instrumented rerun of the 5-minute
panic); the chain above is verified link by link in tests, and the fixed binary counts 39
degenerate points on the same card (below), which is that population.

### Fix

- (a) `draw_invariant` clamps every branch's result to `[lo, hi]`. `invariant_measure` is
  the map's analytic `ds/dx`; at the clamped `s` it moves by at most `2|δs|/|s − m²|`
  relative, the rounding the clamp undoes (pinned in the test).
- (b) (a) alone does not make zero momenta impossible (`s = lo = 0` for a massless pair is in
  range), so `ProtonIntegrand::shape` zeroes a point any of whose outgoing momenta is
  non-finite or has `E ≤ 0` (a massless leg at rest), before cuts and scales, and counts it.
  The setup scale probe skips such points too.
- (c) `kt::cluster` returns `ClusterFailure::NonFiniteMeasure` for a NaN/inf pair measure
  instead of letting it drop out of the comparison; `point_veto` maps it to
  `ScaleVeto::Degenerate` (`PointScales::Vetoed(ScaleVeto)` now carries why), the term is
  zero, and `PointCensus::degenerate` counts the point once. `integrate` warns with the
  count whenever it is nonzero.

### Tests (fail before, pass after)

- `phasespace::diagram_channel::tests::the_invariant_draw_stays_inside_its_range` — before:
  `m 91.1876, Γ 2.4952: x = 0 drew s = 4.4689959971947246e-2 outside [0.04468996, 10000]`.
- `coupling::cluster::kt::tests::a_non_finite_measure_refuses_the_event_by_name` — before:
  `left: Err(NoAdmissiblePair) right: Err(NonFiniteMeasure)`.
- `proton::tests::a_point_with_a_massless_leg_at_rest_is_zeroed_and_counted` — before:
  `PointCensus { evaluated: 2, passed_cuts: 0, nonzero: 0, degenerate: 0 }` (the count is
  what fails; with no guard the zero-momentum point fell to the cuts this time).

### Cards and σ

```
$ rungp.py idm_dilepton vibegraph-g2 runs/g2-idm-s1 --param-card --set small_width_treatment=0.000001
exit 0, wall 1453.5 s, 6210 channels
iteration 3: 1.850153e-11 ± 3.634548e-13
support: 4096334 of 5942158 kept points carry weight (effective sample size 2114)
WARN 39 of 17535404 integration points were zeroed as degenerate
σ = 0.007204 ± 0.000142 pb     (lhaid 247000; the gridpack's 0.009 is at 325500)
```

MadGraph on the full card is not affordable here: its survey of one 2 → 6 flavour
(`p p > h2 h2 mu+ mu- ve ve~`, 1436 jobs, `hard_survey 2`) reached 687 jobs in 56 minutes,
and the full card's survey has ~29 000. What was compared instead is the 2 → 4 part on the
same param card, run card and PDF (MadGraph needed `aS` moved from `FRBLOCK 8` to
`SMINPUTS 3` to import the bundled UFO; the process is `QCD = 0`, so αs does not enter):

```
$ mg/H-G2-mg4-seeds.sh 101 102          (p p > h2 h2 mu+ mu-, lhaid 247000, hard_survey 2, nevents 2000)
seed 101  Cross-section :   0.002312 +- 1.14e-05 pb
seed 102  Cross-section :   0.002314 +- 1.511e-05 pb
$ scripts/idm4.sh                        (vibegraph-g2, --neval 50000 --niter 6, seeds 1-5)
seed 1 0.002332840137341779 6.52e-06    seed 2 0.0023305310895536586 6.53e-06
seed 3 0.002338447826729809 6.66e-06    seed 4 0.002337025949194517 6.64e-06
seed 5 0.0023441743549480866 6.62e-06
vg mean 0.0023366 sd 5.29e-06 sem 2.37e-06 chi2/dof 0.65
mg mean 0.002313 ± 0.0000095 (2 seeds)   ratio 1.0102   pull 2.4
$ vibegraph-base, seed 1, same command: σ 0.002332840137341779 ± 6.521986404370003e-06,
  grid.bin.zst sha256 94adeffd...27815 = vibegraph-g2's
```

The guards do not move this card (bit-identical σ and grid), so the +1.0% (2.4σ against two
MadGraph seeds) predates this session; see Found.

## G3: the factorisation-floor refusal on QCD-4Jets_HT-100to200

### Diagnosis, verified

```
$ rungp.py QCD-4Jets vibegraph-base runs/base-qcd --neval 10000 --niter 2
exit 1, wall 19.9 s
{"error_kind":"card","message":"run card scale prescription: every sampled point's factorisation scale fell below the 2 GeV floor ..."}
$ rungp.py QCD-4Jets vibegraph-probe-ch0 runs/probe-ch0-qcd     (channel 0 only, debug lines)
DEBUG vibegraph::proton: probe: scaled point draw=13
DEBUG vibegraph::proton: probe: scaled point draw=12
DEBUG vibegraph::hadronic: no scale probe point resolved a scale below_floor=0 jet_cut=1 degenerate=0 draws=64
```

Of the card's three multiplicity parts, two find a scaled point on channel 0; the third
draws 64 points on channel 0, one passes the cuts, it is an `xqcut` veto, and no point at all
is below the floor. The old probe counted it as the floor's.

### Fix: keep the refusal, on floor evidence only

MadGraph has no setup refusal (`reweight.f` zeroes a floor or `xqcut` point and moves on).
I kept the probe, because it is the only thing that turns a card whose support is wholly
below the floor (the `a_wholly_sub_threshold_card_is_refused_at_setup` case, `scalefact
1e-4`) into a refusal naming its cause before an integration of zeros, and narrowed it to
what it can prove:

- `ScaleProbeTally` (hadronic.rs) counts the cut-passing probe points by veto; only
  `FactorisationFloor` counts toward the refusal, and it needs at least
  `SCALE_PROBE_MIN_FLOOR = 16` (a quarter of the 64 draws) with no point scaled. An `xqcut`
  veto or a degenerate point is no evidence either way; fewer than 16 floor points lets the
  integration run, as MadGraph would.
- The proton probe goes round the sampling channels (`draw % n_channels`), so one
  channel's region cannot stand for the card.
- The message now says what was observed: `"{below_floor} of the {draws} points the setup
  probe drew passed the cuts with a factorisation scale below the 2 GeV floor ..., and none
  passed above it"`. A debug line records the tally whenever no point resolves a scale.
- The fixed-beam probe uses the same tally. There the floor is never applied (no parton
  density), so it can no longer refuse on `xqcut` vetoes.

The existing `a_wholly_sub_threshold_card_is_refused_at_setup` passes unchanged (≥ 16 floor
points among the 64), and its one-decade-up partner is still accepted.

### Test (fails before, passes after)

`hadronic::tests::the_scale_probe_refuses_only_on_floor_evidence`: an all-`JetCut` tally (1
or 64 points), an all-degenerate tally and 15 floor + 40 `JetCut` do not refuse; 16 floor + 3
`JetCut` refuses with `below_floor = 16`. Before (old rule: any vetoed cut-passing point,
any veto): it panics at hadronic.rs:3330, the single-`JetCut` case refusing. An
integration-level all-`xqcut` probe was attempted on `p p > l+ l- j` with `xqcut = 6000`, but
the run card refuses `xqcut > ptj` (`XqcutAboveJetThreshold`), and with one jet at
`ptj ≥ xqcut` the beam measure `pT²` is never below `xqcut²`; the CLI run of the card itself
is the end-to-end check.

### Card

```
$ rungp.py QCD-4Jets vibegraph-g2 runs/g2-qcd-s1 --neval 10000 --niter 3
exit 0, wall 289.9 s; 123 channels; ~0.5 ms/eval
iteration 3: 5.625999e-1 ± 1.741636e-2 GeV^-2
support: 69640 of 228674 kept points carry weight (effective sample size 1025)
σ = 2.191e8 ± 0.068e8 pb     MadGraph (lhaid 325300, survey) 1.643e8     ratio 1.33
```

One seed, a small budget, and a different PDF (LO `as_0130` here against NNLO `as_0118` in
the gridpack; αs enters a 2 → 2 QCD σ squared, 2 → 4 to the fourth): the ratio is not a
σ verdict. A local MadGraph run of the card at 247000 was not attempted (the 2 → 4 MLM
survey is far beyond the budget the 2 → 6 idm run showed). The card now integrates.

## G4: IR-unsafe photon cards

### What regulates `q → q γ` in MadGraph (read, pinned 3.7.1)

- The pairwise ΔR minimum, `setcuts.f:340-362`: `r2min = draj` for an `a`–`j` pair and
  `drab` for `a`–`b`, both only when both legs have `do_cuts`.
- Frixione isolation, `cuts.f:1104-1222`, on whenever `ptgmin ≠ 0`: for every photon, the
  `pT` sum of jet-class partons within `ΔR ≤ R0gamma` must stay below
  `epsgamma·pT_γ·((1 − cos ΔR)/(1 − cos R0gamma))^xn`; with `xn > 0` that closes at
  `ΔR → 0` and regulates the pole. `banner.py:4520` then discards `draj` ("draj cut
  discarded since photon isolation is used"), so on an isolating card the isolation is the
  only regulator.
- Nothing else: `pta`, `ptj`, `etaj` and `xqcut` (partons only) leave the collinear limit
  open.

vibegraph does not implement isolation: `ptgmin ≠ 0` is refused as an unimplemented cut
before any of this is read. So the rule is: a final-state photon and a final-state massless
coloured particle attached to one vertex of some kept diagram, with no compiled pair cut
between them carrying a positive ΔR minimum, is refused.

### Fix

- `ForcedResonances::of` records `photon_splittings`: `(photon, parton)` outgoing-leg pairs
  that some diagram attaches to one vertex (`photon_parton_vertices`; photon is PDG 22,
  parton is coloured and massless).
- `Cuts::compile_with` calls `refuse_uncut_photon_splitting` after the pair cuts are built:
  each such pair needs a `PairCut` with `dr2_min > 0`, else
  `CutError::UncutPhotonSplitting { photon, parton, pdg }`. An uncut decay product or a leg
  with no pair cut counts as uncut.
- Narrowness: a photon off a lepton line, a gluon-only final state, or quarks from a decay
  the photon is not part of do not register a splitting and are not refused.

### Test (fails before, passes after)

`cuts::tests::an_uncut_photon_quark_splitting_is_refused`: `u d > a u d` registers
`[(0, 1), (0, 2)]`; `draj = 0` is refused with `(photon, parton, pdg) = (0, 1, 2)`; the
default `draj = 0.4` and `draj = 0.01` compile; `draj = 0, ptgmin = 20` is refused as
`UnimplementedCutActive { name: "ptgmin" }`; `e+ e- > mu+ mu- a`, `u u~ > a g` and
`u u~ > z a, z > d d~` at `draj = dral = 0` register nothing and compile. Before: it panics
at cuts.rs:3405 (`draj = 0` compiles).

### Cards

The four group-B cards import `loop_sm-ckm_no_b_mass`, which is not available here;
`sm-ckm_no_b_mass` was used as in H-B1 (`runs/<label>.proc`), with the gridpack's param card:

```
$ rungp.py GJ-2Jets-EWK_Bin ... (and GJ-2Jets-EWK_madgraph, VBFtoG_Bin-PTG-100to200, VBFtoG_Bin-PTG-200)
exit 1, wall 0.2-0.4 s each
{"error_kind":"process","message":"failed to decompose into flavour groups: cut compilation failed: final-state photon (outgoing leg 0) and massless coloured parton (outgoing leg 1, PDG 2) meet at a vertex of a kept diagram, and the run card leaves their collinear limit uncut: no ΔR minimum between them (draj, or drab for a b-class parton) and no photon isolation. ...; set draj > 0"}
$ ... VBFtoG_Bin-PTG-200 --set draj=0.4 --neval 2000 --niter 1     exit 0, σ = 0.418 ± 0.041 pb
$ ... VBFtoG_Bin-PTG-200 --set ptgmin=20                           exit 1, "run card activates cut 'ptgmin' (= 20, MG default 0) which is not implemented"
```

## The 11 diagnosis cards

`vibegraph-g2` unless stated, `rungp.py` defaults (10000 × 3, seed 1, lhaid 247000).

| card | closing campaign (outcome.txt) | now | σ (pb) here | MG σ (pb, gridpack PDF) |
|---|---|---|---|---|
| TCP_m50_ht_400toInf | ok, 1.587e-4 | ok | 2.76e-4 ± 0.19e-4 | 4.079e-5 |
| TCP_m55_ht_400toInf | ok, 1.837e-4 | ok | 2.83e-4 ± 0.17e-4 | 4.364e-5 |
| VBF_HH_kl_1p00_cv_1p00_c2v_1p00 | ok, 1.631e-3 | ok | 1.77e-3 ± 0.13e-3 | 1.911e-3 |
| GJ-2Jets-EWK_Bin-PTG-40 | ok, 96.7 | refused, UncutPhotonSplitting (G4) | — | 164.6 (not a σ: H-B1) |
| GJ-2Jets-EWK | ok, 2966 | refused, UncutPhotonSplitting (G4) | — | 5894 (not a σ) |
| VBFtoG_Bin-PTG-100to200 | ok, 5.64 | refused, UncutPhotonSplitting (G4) | — | 7.57 (not a σ) |
| VBFtoG_Bin-PTG-200 | ok, 0.797 | refused, UncutPhotonSplitting (G4) | — | 1.08 (not a σ) |
| DarkPhoton …MZd-40 | panic, RAMBO | ok (1-body part dropped, G1) | 0.239 ± 0.007 (chi lines stripped from param card) | 0.2167 |
| DarkPhoton …MZd-86 | panic, RAMBO | ok (1-body part dropped, G1) | 4.61 ± 0.15 (same) | 4.431 |
| idm_dilepton_mH80_mA160 | panic, NoAdmissiblePair | ok (39 degenerate points, G2) | 0.00720 ± 0.00014 | 0.009 |
| QCD-4Jets_HT-100to200 | refused, "every μF below 2 GeV" | ok (G3) | 2.19e8 ± 0.07e8 | 1.643e8 |

Group A's offsets (TCP ×6.8 and ×6.5 here; VBF_HH 0.93) are other sessions' (massive light quark, mirror)
and the PDF differs; VBF_HH here is at `2905aada`'s successor with #107, so its 0.93 is a
one-seed 10⁴ reading.

## Bit for bit

`bfb/bfb.sh BINARY TAG`: `integrate --neval 20000 --niter 4 --fixed-budget --seed 1 -j 4`
then `generate --seed 1 --nevents 2000`, on A = `p p > e+ e- @0 / e+ e- j @1` with
`pp_to_llj_mlm_run_card.dat` (MLM, clustering, `xqcut` vetoes, the probe), B = `dy13`
(proc and default run card), C = `p p > z j, z > ve ve~` with
`decay_chain_pp13_run_card.dat` (a Z Breit–Wigner draw to a massless pair).

```
card  σ (pb), base = g2                       grid.bin.zst sha256 (base = g2)        events body sha256 (base = g2)   events
A     946.5423763777234 ± 9.358660781241214   78431da1774461d786bd693695080805d5f9c863a69f79c1bc90fe8e09ca4ef0   fce0eb919dc322fa985d2574eb6bf4d8786deb206c47fddce2321bd3bec3e8ae   2000
B     941.4429743050365 ± 6.6305402214229465  f1c3ad6aff6604a7268d89829d6982efe33a94dd1e4b183fefca1eded6d4b57b   69de74eb52213c262d68fcf5c6b4d6bf6e8e7e41920a2c5e2b75b944513588ad   2000
C     752.4692905039495 ± 3.9078243746567303  ea3c32a3d93913360a709dee7450fed1ea831ef9835728218cbee9cf6b9731f9   67f2d2834837de0f0d49e1aaf2f583b8b9104b34e3e7d2cf72496285cfb3c822   2000
```

The whole-file event hashes differ only in the header line `artifact out-<tag>-A/grid.bin.zst`
(`diff` shows that one line); the bodies (from `<event>` on) are identical. Also
bit-identical: `p p > h2 h2 mu+ mu-` above.

## Fails before

All eight new tests were run against the tree with the fixes reverted
(`/home/user/H-G2-scratch/H-G2-revert-fixes.patch`, applied to `3cb1ff64`, then reverted):

```
test coupling::cluster::kt::tests::a_non_finite_measure_refuses_the_event_by_name ... FAILED
test hadronic::tests::the_scale_probe_refuses_only_on_floor_evidence ... FAILED
test cuts::tests::an_uncut_photon_quark_splitting_is_refused ... FAILED
test phasespace::diagram_channel::tests::the_invariant_draw_stays_inside_its_range ... FAILED
test multiplicity::tests::a_one_body_part_the_cuts_zero_is_left_out_of_the_sum ... FAILED
test multiplicity::tests::fixed_energy_refuses_a_single_outgoing_particle ... FAILED
test multiplicity::tests::a_single_outgoing_particle_is_refused_before_the_probe ... FAILED
test proton::tests::a_point_with_a_massless_leg_at_rest_is_zeroed_and_counted ... FAILED
test proton::tests::a_wholly_sub_threshold_card_is_refused_at_setup ... ok
test result: FAILED. 1 passed; 8 failed
```

(The reverted G3 rule there is the old one: any veto counts, one point refuses.) With the
fixes all nine pass.

## Gates

On the merged head `a0c700e9` (this branch + `origin/main` at `5513bbfc`, #107 and #108),
`CARGO_INCREMENTAL=0`, an untracked `.cargo/config.toml` with `[profile.release-debug]
debug = 0` (deleted afterwards); `scripts/gates2.sh`, log `logs-H-G2-gates2.log`:

```
HEAD a0c700e9
== fmt          cargo fmt --all --check                                   fmt exit 0
== clippy       cargo clippy --workspace --all-targets -- -D warnings     clippy exit 0
== test         cargo test --workspace                                    test exit 0
                55 suites: 1986 passed, 0 failed, 25 ignored
== validate     bash validation/validate.sh  (logs-H-G2-validate2.log)   VALIDATE_EXIT 0
                2440 passed, 0 failed
                [report] 77 rows x 4 categories: the measured cells are the declared cells
```

**Banked σ rows, against the manager's `validate.sh` at this branch's base `2905aada`**
(`/home/user/H-V2-scratch/MGR-validate-2905aada.log`): every line carrying a `σ` from
either log, sorted, 203 each, timing-only lines dropped by the filter:

```
$ diff MGR-validate-2905aada.log.sigma logs-H-G2-validate2.log.sigma
179c179   variance×CPU (L4 uniform vs L5 adapted): ... (731 ns/pt) ... | ... (285 ns/pt) ...
```

The one differing line is a per-point timing inside an otherwise identical line (σ, variance
and α agree to every printed digit). No banked σ row moved, with #108 merged in as well.

An earlier `fmt`/`clippy`/`cargo test --workspace` run on `df1c0171` (before #108) also
passed (1978 passed, 0 failed); its `validate.sh` was stopped when #108 landed and rerun
above on the merged head.

## Brief corrections

- G1's fixed-beam check: the fixed-beam path does panic, at the scale probe in
  `FixedBeamIntegrand::use_running_coupling`, not in grouping; it is refused now.
- G2: the BW excursion is not only at `x = 0` below `lo`; at `x = 1` the draw overshoots `hi`
  (by ~1e-8 relative for a narrow line drawn far above its pole), which leaves a sibling
  more than the parent has — the same zero-momentum outcome. And (a) alone cannot make the
  zero-momentum point impossible: `s = lo = 0` is in range for a massless pair. (b) is
  needed, as the brief anticipated.
- G2's "points with a non-physical invariant ... get zero weight": the proton weight is
  `αⱼ/g(p)` from the momenta, so the map's own zero `R₂` never zeroed such a point; this is
  why it reached the clustering at all.
- G3: the run card refuses `xqcut > ptj`, so an integration-level all-`xqcut` probe card is
  hard to build; the test is on the tally, and the card run is the end-to-end check.
- G4: isolation in MadGraph replaces `draj` (`banner.py:4520`), and vibegraph refuses
  `ptgmin ≠ 0` as unimplemented, so "isolation in effect" never reaches the new check.
- The group-B gridpacks' `loop_sm-ckm_no_b_mass` is not loadable here (no `loop_sm` UFO);
  the runs use `sm-ckm_no_b_mass`, as H-B1 did.
- Process note: stopping the abandoned full-idm MadGraph run, I used `pkill -f` on its
  directory name and `pkill -9 -x madevent`, which would also have killed a sibling's
  MadGraph; none was running that I know of, but it was against the discipline.

## Found

1. **The one-body phase space.** `p p > z`, `p p > zp` with no `pT` cut on it, `e+ e- > z`
   are refused, not integrated. Closing it: a `δ(ŝ − M²)` map (`τ = M²/s`, flat or BW in
   `ŝ` for a finite width, rapidity the only coordinate) and the matching `2 → 1` clustering
   scale (`kt.rs` already has the `n == 3` core branch).
2. **Fixed beams with `ickkw = 1`, `xqcut > 0`.** `FixedBeamIntegrand::event_scales_of` has
   `unreachable!` for `PointScales::Vetoed`, reasoning that the floor never applies at fixed
   beams; an `xqcut` (`JetCut`) veto is a `Vetoed` too, and `runcard/matching.rs` does not
   refuse `xqcut` at `lpp = 0`. Not run here. Closing it: run `e+ e- > j j j` at `ickkw = 1,
   xqcut = 30`, `lpp = 0`; zero the point (as the proton path does) or refuse the card.
3. **Other uncut collinear and soft limits** (out of scope of G4): massless jets at
   `ptj = 0` with `drjj = 0` and no `xqcut` (`q → q g` final-state, `g → g g`), a photon off a
   massless charged lepton at `dral = 0` (`e+ e- > mu+ mu- a` in the default SM has massless
   muons), and a coloured parton above `maxjetflavor` that no class pair cut reaches.
   Closing it: extend `photon_parton_vertices` to every massless-pair splitting and refuse
   where no pair cut or `xqcut` regulates it, checked on the banked cards.
4. **idm `p p > h2 h2 mu+ mu-` reads +1.0% (2.4σ) above MadGraph** at 247000: vibegraph
   0.0023366 ± 0.0000024 (5 seeds, χ²/dof 0.65) against MadGraph 0.002313 ± 0.0000095
   (2 seeds, `generate_events`). Unchanged by this session (base binary bit-identical).
   Closing it: ≥ 5 MadGraph seeds, then per-diagram dumps if it holds.
5. **The full idm card is beyond a local MadGraph run here**: one 2 → 6 flavour's survey,
   1436 jobs at `hard_survey 2`, did 687 in 56 minutes. Its σ against MadGraph at a common
   PDF needs a bigger host.
6. **MadGraph 3.7.1 refuses the gridpack's bundled InertDoublet UFO** ("aS parameter should be
   assigned to SMINPUTS#3"; it sits in `FRBLOCK 8`), which vibegraph loads. A local
   comparison needs the one-line move this report made.

## Commits

On `sprinth-guards`, author Nicholas Smith <nick.smith@cern.ch>, each ending
`Assisted-by: claude-code:claude-opus-5-5`:

* 3cb1ff64 fix: guard 2->1 subprocesses, degenerate points, the floor probe and uncut photon splittings
* f8ee53d6 merge of `origin/sprinth-mirror-rotation` (24116d71)
* 611b2389 hadronic: log what the scale probe saw when no point resolved a scale
* df1c0171 merge of `origin/main` (082298c1, #107)
* a0c700e9 merge of `origin/main` (5513bbfc, #108)
* (this report) docs: H-G2 session report

Files: `vibegraph-lib/src/{proton,hadronic,cuts,multiplicity,support,generate}.rs`,
`vibegraph-lib/src/coupling/cluster/kt.rs`, `vibegraph-lib/src/phasespace/diagram_channel.rs`,
`vibegraph-cli/src/integrate.rs`, this report and the sessions index.
