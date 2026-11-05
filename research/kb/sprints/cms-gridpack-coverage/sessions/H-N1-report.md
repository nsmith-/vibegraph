---
type: Session Report
title: "H-N1 report: near-degenerate flavour groups merge at rounding and split otherwise; the γ+jets split is the W width, not CP violation"
description: "D1 implemented: GROUP_REL_TOL 1e-10 → 1e-9 and the DegenerateGroups refusal (GROUP_SEPARATION_MIN) removed. Worst rounding-level separation over the banked proton cards and the survey reproductions is 1.48e-10 (p p > w- z j j QCD=0, the survey's 1.5e-10), identified by its f32/f64 scaling; the closest physical one is 1.2e-7. The 9.08e-7 u b~ / u~ b split is the reflection-odd part of |M|² carried by the W width: CP holds to 3e-16, the CKM phase moves the split only through |V_ub|. Every previously derivable partition unchanged; mutations caught; σ of p p > a j j QCD=0 against local MadGraph."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-05}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
---
Branch `sprinth-near-degenerate` in `/home/user/wt-nd`, from `origin/main` at
`20aca9fe`. Scratch (probes, MadGraph directories, logs, drivers) is under
`/home/user/H-N1-scratch/`; none of it is committed. MadGraph is a copy of the
pinned tree (`b7687064`) at `/home/user/H-N1-scratch/mg/mg5` with
`vibegraph-lib/src/ufo/sm_assets/restrict_ckm_no_b_mass.dat` added to
`models/sm`, as the banked CKM rows do (MadGraph does not ship that card).

## What changed

`vibegraph-lib/src/proton.rs` only, confined to the grouping constants, the
partition and the separation check:

- `GROUP_REL_TOL` 1e-10 → **1e-9**, its documentation replaced by the
  measurement below.
- The separation check after the partition loop is gone, with
  `GROUP_SEPARATION_MIN` and `ProtonError::DegenerateGroups`. Under D1 a pair
  separated by more than `GROUP_REL_TOL` is two groups, and two groups are
  always a sound decomposition (each is evaluated with its own compiled
  amplitude and luminosity), so the refusal protected nothing: a rounding pair
  that straddled the tolerance costs one evaluation per point, not a wrong σ.
  The only path it guarded that a merge could break is the opposite one, a
  physical pair *below* the tolerance, which the refusal never saw either.
- The colour-incompatible exemption from #100 lived inside that check and goes
  with it. The join loop is unchanged: a pair within the tolerance whose
  colour bases cannot be paired still starts its own group. Raising the
  tolerance lets more pairs reach `colour_pairing`, which is the sound
  direction (a pairing failure splits, never merges).
- Module documentation: the refusal paragraph replaced by the split rule.
- Two tests, below. The `a_pair_whose_flows_cannot_be_paired_is_grouped_apart`
  doc line that referred to the separation floor is trimmed.

Nothing else used the removed items:

`grep -rn "DegenerateGroups\|GROUP_SEPARATION_MIN\|separate by only"` over
`*.rs`, `*.md`, `*.toml` before the change matched `vibegraph-lib/src/proton.rs`
(definitions, the raise site, the module doc) and, outside code, only this
item, `research/kb/hadronic/flavour-groups.md` (Found 1) and the sprint
records.

## The measurement: worst rounding-level separation

Probe (uncommitted, `/home/user/H-N1-scratch/h_n1_probe.rs`, mounted as a
`#[cfg(test)]` submodule of `proton` while it ran): for each card, enumerate,
split by multiplicity and external masses, put each part in
`derive_flavor_groups`' slot order (`content_ordered`), compile, and evaluate
every subprocess at the partition's own probe points (`probe_momenta`,
`PROBE_SEED`) **in f64 and in f32**. For every pair with equal polarizations it
records `worst_rel` in both precisions. A separation that is rounding grows by
~2^29 ≈ 5e8 from f64 to f32 (amplified rounding grows the same way); a coupling
or kinematic difference does not move. Pairs with `rel32 > 1e3 · rel64` are
classified as rounding, the rest as physical.

Cards: the banked proton rows' processes (`pp_to_ll`, `pp_to_ll_qcd0`,
`pp_to_bb`, `pp_to_llj`, `pp_to_jj`, `pp_to_ttx_0j1j`'s `p p > t t~ j`,
`pp_to_ll_0j2j`'s `p p > e+ e- j j`, `pp_to_epemjj_qcd0` under `sm-no_b_mass`),
`p p > j j j` (H-G1's extra card), `p p > j j` under `sm-no_b_mass`,
`p p > e+ ve` under `sm-ckm_no_b_mass`, and the survey reproductions below.

Before the change (`H-N1-probe-before-{a,b}.log`):

| card | subprocesses | worst rounding | closest physical | derive |
|---|---|---|---|---|
| p p > l+ l- | 8 | 0 | 0.764 | Ok, 2 groups |
| p p > l+ l- QCD=0 | 8 | 0 | 0.764 | Ok, 2 |
| p p > b b~ | 5 | 0 | 0.998 | Ok, 2 |
| p p > l+ l- j | 24 | 0 | 0.755 | Ok, 6 |
| p p > j j | 65 | 5.73e-13 | 0.424 | Ok, 7 |
| p p > j j (5f) | 96 | 5.73e-13 | 0.424 | Ok, 7 |
| p p > j j j | 97 | 4.27e-16 | 0.494 | Ok, 15 |
| p p > t t~ j | 13 | 0 | 0.560 | Ok, 4 |
| p p > e+ e- j j | 64 | 0 | 0.673 | Ok, 28 |
| p p > e+ e- j j QCD=0 QED=4 $ h (5f) | 87 | 0 | 0.793 | Ok, 34 |
| p p > e+ ve (ckm 5f) | 6 | 0 | 0.947 | Ok, 4 |
| p p > a j j QCD=0 (ckm 4f) | 120 | 5.55e-14 | 0.834 | Ok, 35 |
| **p p > a j j QCD=0 (ckm 5f)** | 255 | 5.55e-14 | **9.08e-7** (u b~ > a u b~ / u~ b > a u~ b; f32 8.43e-7) | **Err DegenerateGroups 9.080e-7** |
| p p > w+ z j j QCD=0 (ckm 5f) | 264 | 6.30e-11 (u s > W+ Z d d / c d > W+ Z s s; f32 1.34e-2) | 3.43e-4 | Ok, 248 |
| **p p > w- z j j QCD=0 (ckm 5f)** | 264 | **1.477e-10** (d c~ > W- Z s d~ / s u~ > W- Z d s~; f32 3.36e-2, ×2.27e8) | 3.43e-4 | **Err DegenerateGroups 1.477e-10** |

Every pair between 1e-12 and 1e-6 across all cards, with its f32/f64 ratio
(`H-N1-probe-after.log`, the same separations):

```
rounding (ratio 1.7e8 – 1.3e9): 21 W±Z pairs from 3.3e-11 up to 1.477e-10
physical (ratio ~1):            9.080e-7 (ckm 5f γjj), 1.183e-7 (the same at η = 0)
```

**Worst rounding-level separation: 1.477e-10.** The new bound, 1e-9, is a
factor 6.8 above it. The closest physical separation measured is 1.18e-7 (on a
modified card) and 9.08e-7 on the restriction's own, factors 120 and 900
above. The headroom is thin on the rounding side: it is the electroweak gauge
cancellation of a 2 → 4 vector-boson-scattering amplitude that amplifies the
rounding, and a larger cancellation (more legs, higher energy rungs) could
cross 1e-9. Crossing it splits a group, which is sound (above).

## The survey cards reproduced

The survey's cards are not extracted. The smallest cards reproducing the two
separations:

- **γ + 2 jets EWK** (GJ-2Jets-EWK, VBFtoG ×2): `import model
  sm-ckm_no_b_mass; generate p p > a j j QCD=0`, `maxjetflavor 5`, the
  restriction's own parameters. It is refused before the change at
  **9.080e-7**, the item's number to all four digits, on `u b~ > a u b~`
  against `u~ b > a u~ b`. Closeness: the restriction is the one CMS bundles;
  the process is what the gridpack names suggest; the CMS param card, cuts and
  any `$$`/`/` exclusions are unknown. `$$ w+ w-`, `$$ w+ w- z`, `$$ t t~`,
  `/ h` and `$$ z` variants were also probed (`H-N1-probe-variants.log`): the
  first two have no near pair, the last three reproduce 9.080e-7 unchanged.
  The item's second number, **3.88e-8, is not reproduced** by any of these.
- **VBS W±Z → 4q** (VBS_WPMZTo4Q): `generate p p > w- z j j QCD=0` under
  `sm-ckm_no_b_mass`, refused before the change at **1.477e-10**, the survey's
  1.5e-10. The W+ line has its worst pair at 6.3e-11, already merged at 1e-10.
  The 4q decays are presumably MadSpin's: a decay chain `w+ > j j, z > j j`
  is refused by the enumeration (identical particles between decay and
  production make the on-shell line ambiguous), so the survey card cannot have
  reached `DegenerateGroups` that way. Closeness: production process as named;
  the card's param card and the exact `add process` lines unknown. The
  combined `w+`/`w-` card (528 subprocesses) derives after the change too.

## The γ+jets split is the W width, not CP violation

D1 and the brief expected the split to vanish with the CKM phase *or* the W
width at zero. **Only the W width removes it.** At η = 0 it drops to 1.18e-7
and stays above the tolerance (f32 1.69e-7: physical).

The diagnosis, measured on the pair alone (`u b~ > a u b~ QCD=0` + `add
process u~ b > a u~ b QCD=0`, `a_conjugate_pair_split_by_the_w_width_is_two_groups`):
CP maps the conjugate's `|M|²` at `p` onto the subprocess's at the reflected
`M p = (E, −pₓ, p_y, p_z)` (parity, then the π rotation about x that puts the
beams back). Comparing `u b~` at `p` with `u~ b` at `M p`:

```
"": at p 9.080e-7, conjugate at M p 3.308e-16
"BLOCK WOLFENSTEIN\n  4 0.0\n": at p 1.183e-7, conjugate at M p 3.308e-16
"BLOCK WOLFENSTEIN\n  3 0.3656569430490826\n  4 0.0\n": at p 9.080e-7, conjugate at M p 3.308e-16
"DECAY 24 0.0\n": at p 3.162e-16, conjugate at M p 3.162e-16
```

- CP holds to rounding on every card, the CKM phase included: this pair
  shows no CP violation at all. (Inferred, not checked symbolically: no
  rephasing-invariant CKM phase enters this subprocess's `|M|²`.)
- The same-`p` split is twice the reflection-odd part of `|M|²`: a
  Levi-Civita contraction of four momenta (a 2 → 3 has enough independent
  momenta for one), which needs an absorptive phase; here that is the W width.
- The CKM phase moves the split only through `|V_ub|² = A²λ⁶(ρ² + η²)`:
  at η = 0 with ρ raised to keep `ρ² + η²`, the split is 9.080e-7 again.

So the pair is a genuine physical distinction (P-odd, CP-even, carried by
`Γ_W·|V_ub|²`), and grouping it apart is correct. The brief's premise that it
is CP violation is falsified by the reflected comparison.

## Tests

`vibegraph-lib/src/proton.rs`, module `tests`:

- `a_pair_separated_by_rounding_shares_one_group`: `d c~ > w- z s d~ QCD=0` +
  `s u~ > w- z d s~ QCD=0` under `CkmNoBMass`, `maxjetflavor 5`. Asserts the
  premise (`1e-10 < rel < GROUP_REL_TOL`, `rel32 > 1e6 · rel`), then one group
  of two. Prints `f64 separation 1.477e-10, f32 separation 3.358e-2`.
- `a_conjugate_pair_split_by_the_w_width_is_two_groups`: the four cards above.
  Asserts CP at `M p` to 1e-12 on each, the group count from the trace (2, 2,
  2, 1), the split above `100 · GROUP_REL_TOL` on the restriction's card, below
  half of that at η = 0, within 1% of it with `|V_ub|` kept, and below 1e-12
  at `Γ_W = 0`.

Before the change both fail: the rounding pair is split at 1e-10 and then
refused at 1.477e-10, and the conjugate pair is refused at 9.080e-7 (the
probe's before-log `derive Err(...)` lines above are the same refusals on the
full cards). Mutations (`H-N1-mutations.sh`, log `H-N1-mutations.log`), each
on the final source, restored by `cmp` afterwards (`RESTORED`):

| mutation | result |
|---|---|
| M1 `GROUP_REL_TOL = 1e-10` | `a_pair_separated_by_rounding_shares_one_group` FAILED (proton.rs:5557, the premise `rel < GROUP_REL_TOL`); the conjugate test still passes |
| M2 separation refusal restored (`rel <= 1e-6` panics, with the colour exemption) | `a_conjugate_pair_split_by_the_w_width_is_two_groups` FAILED in `derive_flavor_groups` (proton.rs:902); the rounding test passes (its pair now merges) |
| M3 conjugate compared at `p`, not `M p` | conjugate test FAILED: `"": CP fails by 9.080e-7` |
| M4 η = 0 without keeping `\|V_ub\|` | conjugate test FAILED at the `vub_kept` assertion (proton.rs:5650): 1.183e-7 against 9.080e-7 |

## No already-gated partition changes

The probe printed every group's members (`GROUP` lines) before and after;
`H-N1-groups-{before,after}.txt`, compared per card:

```
pp_to_ajj_qcd0_ckm4f before_groups=35 after_groups=35 differing_lines=0
pp_to_ajj_qcd0_ckm5f before_groups=0 after_groups=82 differing_lines=82
pp_to_ajj_qcd0_ckm5f_eta0 before_groups=0 after_groups=82 differing_lines=82
pp_to_ajj_qcd0_ckm5f_ww0 before_groups=79 after_groups=79 differing_lines=0
pp_to_bb before_groups=2 after_groups=2 differing_lines=0
pp_to_epemjj before_groups=28 after_groups=28 differing_lines=0
pp_to_epemjj_qcd0 before_groups=34 after_groups=34 differing_lines=0
pp_to_epve_ckm5f before_groups=4 after_groups=4 differing_lines=0
pp_to_jj before_groups=7 after_groups=7 differing_lines=0
pp_to_jj_5f before_groups=7 after_groups=7 differing_lines=0
pp_to_jjj before_groups=15 after_groups=15 differing_lines=0
pp_to_ll before_groups=2 after_groups=2 differing_lines=0
pp_to_ll_qcd0 before_groups=2 after_groups=2 differing_lines=0
pp_to_llj before_groups=6 after_groups=6 differing_lines=0
pp_to_ttxj before_groups=4 after_groups=4 differing_lines=0
pp_to_wmzjj_qcd0_ckm5f before_groups=0 after_groups=248 differing_lines=248
pp_to_wpmzjj_qcd0_ckm5f before_groups=0 after_groups=496 differing_lines=496
pp_to_wpzjj_qcd0_ckm5f before_groups=248 after_groups=248 differing_lines=0
```

(`before_groups=0` is a refused card; `pp_to_wpmzjj` was added to the probe
after the before-run, and contains the refused W−Z pair.) Every card that
derived before derives to the identical partition; the only cards that move
are the ones that were refused. In the W−Z card the 1.477e-10 pair is the one
merge the new bound adds (264 subprocesses, 248 groups).

## σ: p p > a j j QCD=0 under sm-ckm_no_b_mass against local MadGraph

Before the change this card is refused (`DegenerateGroups 9.080e-7`); after
it, 255 subprocesses in 82 groups, the u b~ / u~ b pair in two of them.

- MadGraph: `/home/user/H-N1-scratch/mg/ajj.mg5` (`import model
  sm-ckm_no_b_mass; generate p p > a j j QCD=0`; 255 processes, 171 matrix
  elements, one P directory), its own generated `run_card.dat` with `nevents
  5000`, `pdlabel lhapdf`, `lhaid 247000`, `use_syst False`, `hard_survey = 2`
  added (`mg/ajj_run_card.dat`); defaults otherwise (`ptj 20`, `pta 10`,
  `etaj 5`, `etaa 2.5`, `drjj 0.4`, `draj 0.4`, `maxjetflavor 5`,
  `dynamical_scale_choice -1`). One fresh copy of the never-launched output per
  seed (`H-N1-mg-seeds.sh`), seeds 101–105; σ from each
  `SubProcesses/results.dat`.
- vibegraph: the same proc card, MadGraph's generated run card with only
  `pdlabel`, `lhaid`, `use_syst` changed (`vg/ajj_run_card.dat`),
  `integrate --seed N --target-rel 0.002`, seeds 1–5, `LHAPDF_DATA_PATH` the
  worktree's `validation/pdf`. Binary: this branch's dev-profile build.

```
MadGraph  208.18 0.494 | 207.86 0.511 | 207.60 0.637 | 208.21 0.506 | 208.08 0.544
vibegraph 208.46 0.41  | 208.09 0.41  | 208.67 0.39  | 208.01 0.39  | 208.62 0.38
$ python3 -I seedstats.py  (unweighted mean, sample sd, sem, chi2/dof on the quoted errors)
vibegraph n=5 mean=208.37 sd=0.3036 sem=0.1358 (rel 0.065%) chi2/dof=0.598
MadGraph  n=5 mean=207.986 sd=0.2557 sem=0.1144 (rel 0.055%) chi2/dof=0.202
diff +0.384 +- 0.177 pb, rel +1.85e-03, pull +2.16
```

+0.18% at 2.2σ on the seed sems. MadGraph's χ²/dof 0.20 says its quoted
errors are about twice its seed scatter, so the sems, not the quoted errors,
are the scale. The size matches the +0.1–0.3% this sprint's other local
MadEvent comparisons read on unrelated cards (H-C1 +0.10% at `hard_survey =
2`, H-M2's mirror rows "about 1e-3 high together"), so it is not attributed to
this card. What this σ can and cannot see: it is an end-to-end check that the
card now integrates to MadGraph's number at the 0.2% level. It is blind to the
split itself: the `u b~` group's share of σ is far below 0.2%, and merging it
with its conjugate would move σ by about 1e-6 of that share.

## Gates

All on the final source (the probe module removed; `git diff --stat`:
`vibegraph-lib/src/proton.rs | 266`).

```
$ cargo fmt --all --check                                   → exit 0
$ cargo clippy --workspace --all-targets -- -D warnings     → Finished, exit=0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings
                                                            → Finished, exit=0
$ cargo test --workspace                                    → exit=0; 55 suites: 1954 passed, 0 failed, 25 ignored
$ CARGO_TARGET_DIR=…/target-validate nohup setsid bash validation/validate.sh   (.cargo/config.toml: release-debug debug = 0, deleted after)
  85 suites: 2408 passed, 0 failed, 110 ignored; no "!!!" line
  77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered).
  The measured cells are exactly the cells the manifest declares, every gate cell passed, …
```

The 5 ⚠️ are the standing informational cells (`gg_to_gg` and `gg_to_gg_cg`
diagrams, `ee_to_wpwm_cw` and `ee_to_zh_smeft` amplitudes, `ee_to_mumua`
samples), the same count H-C1 recorded. Both new tests ran in the banked
layer (`test proton::tests::a_conjugate_pair_split_by_the_w_width_is_two_groups ... ok`,
`… a_pair_separated_by_rounding_shares_one_group ... ok`).

## Closes_when

The item asks that the 9.08e-7 and 3.88e-8 separations be shown physical or
rounding, with a test on one of those processes. 9.08e-7 is shown physical
(W width, `|V_ub|²`) and tested; the survey's other refused separation,
1.5e-10, is shown rounding and tested. **3.88e-8 is not reproduced**, so the
item's file is not deleted by this branch. Under the new rule whatever it is
gets two groups (it is 260 times the worst rounding measured, so very likely
physical), and the card proceeds; the CERN-host rerun of cards 006, 010, 020
and 089 would show which pair it is. The manager decides whether that closes
the item.

## Found

1. **`research/kb/hadronic/flavour-groups.md` is stale.** Steps 4–5 state
   `GROUP_REL_TOL = 1e-10`, the `GROUP_SEPARATION_MIN = 1e-6` refusal and its
   colour exemption. Closes with an edit to the new rule and the measurement
   (this session does not edit concepts).
2. **Conjugate pairs are assumed to share |M|² at the same momenta, and that
   fails wherever a width meets a P-odd structure.** The grouping catches it
   numerically (it splits them), but only to the extent the probe points
   see the ε-term: the reflection-odd part vanishes on planar configurations
   and is small where the width's propagator is far off shell. Evidence: the
   9.08e-7 pair above, which exists only through `Γ_W`. Closes with the
   symbolic criterion of `flavour-grouping-unsound-sampled-criterion`, or a
   probe check that compares the conjugate at `M p` as well.
3. **The rounding margin is a factor 6.8.** The worst rounding pair grows
   with the gauge cancellation; a 2 → 5 or 2 → 6 EW card could pass 1e-9 and
   split groups that are one program (cost only). Evidence: 1.477e-10 on a
   2 → 4. Closes with the program-identity criterion of
   `flavour-grouping-unsound-sampled-criterion`, which would make the
   tolerance moot.
4. **`g g > q q~` and `q q~ > g g` share one group** on every `p p > j j`
   card (rounding 5.7e-13, f32 ratio 2.7e6): time-reversed processes with one
   spin-summed `|M|²`, weighted apart by their own averaging factors. Not a
   bug as far as this session checked (pre-existing, gated bit for bit in
   `pp_to_jj`), but worth one line in the concept doc next to the
   conjugate-pair rule.
5. **3.88e-8 unreproduced** (above): the CERN host should record, for the
   five survey cards, the pair and separation each now splits on.
