---
type: Session Report
title: "L-P1 report: the TprimeBToTZ miss is the b-collinear corner a transfer bound cuts out of the dominant channel"
description: "Card 126's low seeds miss g → b b̄ with the b̄ collinear (|t_b| < 100 GeV²): configuration 2's spine bounds the b̄ rung at −ptj² though the b̄ carries no pT cut. Each rung's bound is now held to what a single-leg side of its line proves; 20 seeds at the campaign budget land in one population."
status: draft
tags: [sampling, multichannel, spine, t-channel, cuts, confidently-wrong, cms-gridpacks, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-multichannel` off main a5bf8fca (brief commit ddd98052). The
backlog item `multichannel-subdominant-channel-carries-dominant-region` is
**not** deleted: its `closes_when` still needs the complex-mass-scheme-off
MadGraph σ (below, "vg-validator requests"). Its corrected text is in Found.

## The answer in one paragraph

The missed region is **configuration 2's own collinear corner**: the gluon
splits to `b b̄` and the `b̄` leaves along the gluon's beam with
`|t_b| = |(p_g − p_b̄)²| < 100 GeV²` (b̄ `pT ≲ 8 GeV`). It is not a T′ tail:
the heavy points spread over `m(Zt)` from 1.1 to 2.6 TeV, the whole broad
T′. Configuration 2's channel has **zero density** there by construction:
`spine_chain` (`phasespace/diagram_channel.rs`, the `t_max_cap` it sets)
bounds every rung of a peripheral chain at `t ≤ −spacelike_floor`, and
`Cuts::spacelike_floor` (`cuts.rs`) is the largest single-leg `pT` threshold
of *any* leg, here `ptj² = 100 GeV²`. The rung that emits the `b̄` alone
gets that bound although the `b̄` has no `pT` cut (`maxjetflavor 4`,
`ptb 0`), so the cut is not one the event selection makes. The integrand
there is the dominant diagram's `1/(t − m_b²)²` peak. Only channels 4 and 1,
whose maps emit the `b̄` inside a timelike split rather than off a bounded
rung, have density there, and very little of it; they find the corner as
rare points of weight up to 16% of σ each. A seed that draws one is "high";
one that does not is "low" with a small quoted error. The estimator is
unbiased (the union of supports covers the corner), so this is a variance
failure, not a bias. That is why the existing coverage gate
(`every_bounded_channel_set_covers_its_own_fiducial_region`) cannot see it.

## Reproduction on main a5bf8fca

Cards reproduced from `/home/user/wt/cards-sprintL/tprime126.md` into
`/home/user/wt/lp1-card/` (proc, run with the hidden parameters, param card
with the zero masses and widths spelled out; the QNUMBERS blocks were left
out, the binder does not need them). Model from the CMS model store,
`VLQ_v4_4FNS_UFO.tgz`:

```
$ sha256sum VLQ_v4_4FNS_UFO/restrict_3rdL_noCKM.dat
0459cb74199afbf3d97a7fb1d54fef1c96561c4456827cb2900e98458db45504  (9245 B)
```

That matches the card file. vibegraph's own model digest also matches:
`model VLQ_v4_4FNS_UFO-3rdL_noCKM (0ff393dc…9688ff9)`. gpdb's
`model_sha256` (52d46852…) did not reproduce under six readings of its
"sorted relpath\tsha256" recipe (got 83bdef2c… for the plain reading); since
vibegraph's digest and the restriction hash both match, the recipe, not the
model, is the likely difference.

Command (`/home/user/wt/lp1-bin/run_seed.sh`):

```
vibegraph integrate proc_card.dat --run-card run_card.dat --param-card param_card.dat \
  --ufo-dir models --target-rel 0.01 --neval 120000 --seed N -j 4 --parallel-diagrams \
  --output-ldjson --no-network --out …     (LHAPDF_DATA_PATH → NNPDF31_nnlo_as_0118_nf_4_mc_hessian)
```

| seed | σ (pb) | err | ch4 share | ch4 χ²/dof | wall |
|---|---|---|---|---|---|
| 1 | 2.52930e-4 | 2.40e-6 | 9.1% | 1.50 | 40.7 s |
| 2 | 2.39866e-4 | 5.31e-7 | 1.4% | 3.50 | 15.4 s |
| 3 | 2.40053e-4 | 5.43e-7 | 2.1% | 1.13 | 14.7 s |

Seeds 2 and 3 equal the host's v0.2.2 values to the printed digits; seed 1
differs at the 3e-4 level (2.52866e-4 on the host). **One seed costs 15 s
(stops at the minimum iterations) to 41 s (runs to 20 iterations)** on this
4-core container, so 20-seed sweeps at the campaign budget fit locally.

## The region, from per-event output

A throwaway probe (not committed; `/home/user/wt/lp1-bin/probe.patch`) printed
every point whose contribution `value · w_VEGAS / neval` exceeded 3e-16 GeV⁻²
(about 5e-4 of σ in one iteration) with its momenta, every channel's density
`gₖ`, the α-weighted shares `αₖgₖ/g`, and each flavour group's per-configuration
`AMP2` (`ProtonIntegrand::point_anatomy`, `ordering_contributions`,
`eval_amp2`). Analysis script `/home/user/wt/lp1-bin/ana.py`; final-state order
is `Z t b̄ d` for `g u > Z t b~ d`.

Seed 1 (hit), base code, 1073 heavy points:

| drawn in | channel 2 density | points | Σ contribution (GeV⁻²) |
|---|---|---|---|
| channel 2 | > 0 | 742 | 5.36e-13 |
| channel 4 | **0** | 264 | **3.38e-13** |
| channel 4 | > 0 | 47 | 2.6e-14 |
| channel 1 | **0** | 5 | 3.2e-14 |
| channel 1 | > 0 | 14 | 1.0e-14 |

Every heavy point carries configuration 2's `AMP2` as its largest entry by
three orders or more. Its heaviest point (channel 4, contribution 1.04e-13,
a sixth of σ in one iteration):

```
LP1P ch 4 … shares 6.5e-8,1.46e-2,0.000e0,…,9.853e-1,… dens …,5.480e-13,0.000e0,…,9.326e-12,…
  amp2 [1.010e-13, 6.520e-11, 1.503e-4, 1.012e-13, 2.779e-10]
  b̄: pT 3.30 GeV, t_b = −62.5 GeV²; d: pT 36.6; m(Zt) 2045 GeV
```

The support edge, over all 1073 points:

```
dens2 == 0: n 269, t_b in [−99.37, −6.62] GeV², b̄ pT ≤ 7.68
dens2  > 0: n 804, t_b closest to zero −100.78 GeV², b̄ pT down to 0.16
```

The edge is `t_b = −100 GeV² = −ptj²` exactly. Seed 2 (miss), same probe:
481 heavy points and **none** with channel 2's density zero. The low seeds
are the ones that never drew in the corner.

**Why configuration 2's map misses it.** Configuration 2 (`s0.d2`,
propagators `b`, `W−`, `tp`) is a two-rung spine: the gluon rung emits `S₁ =
{b̄}`, the `W` rung leaves `{d}` as recoil, and `tp → Z t` is a timelike
split under its Breit–Wigner. `spine_chain` gives both rungs
`t_max_cap = −fiducial_scale` (`diagram_channel.rs`, the `SpineRung`
construction), with `fiducial_scale = Cuts::spacelike_floor() = 100`
(`maps.rs` `MapChoices::channel` passes it to `from_diagram_regulated`).
`apply_fiducial_t_max` then narrows the gluon rung's window to
`t ≤ −100`, and `Channel::density` returns an exact zero above it. The
`d`-rung's bound is sound (the recoil `{d}` is a jet with `pT > 10`); the
`b̄`-rung's is not. Channel 4 (`s0.d4`, `tp`, `tp~`, `W−`) attaches the gluon
to the T′ line, so its gluon rung emits `{Z, t}` and the `b̄` comes out of a
timelike split: it covers the corner, at `α₄ g₄ ≈ 1e-13`, far from where its
own `|M|²` lives.

The documented derivation already said this could happen: the bound is a
bound "for a three-body final state" and "only a scale past three outgoing
legs" (`Cuts::spacelike_floor`'s doc, `phase-space/spacelike-floor.md`). This
card is the first measured case where the scale cuts accepted, heavily
weighted phase space out of the dominant channel.

**Not the mechanisms the brief listed.** No BW window or `narrow_in_tau`
threshold is involved (the T′ is drawn under its Breit–Wigner across its whole
width in every heavy point), and the VEGAS grid is not the limit: a grid
cannot adapt into a region its map gives zero density.

## The fix

`DiagramChannel::with_side_transfer_bounds` (`diagram_channel.rs`), applied
last in `MapChoices::channel` (`maps.rs`), with `Cuts::single_leg_transfer_bound`
(`cuts.rs`) and the `ChannelBounds` trait method. Per rung, the line splits
the final state into the emitted side `S` and the rest `R`. With massless
beams `S` takes a fraction `x ≤ 1` of its beam's light-cone momentum, so
`t = M² − (M² + pT²)/x ≤ −pT²` for any system, and transverse balance gives
`S` and `R` the same `pT`. A single-leg side with threshold `pT_min` therefore
proves `|t| ≥ pT_min²`. The rule:

- either side a single leg: the bound becomes `min(current, max of the proven squares)`,
  and is dropped where that is zero;
- both sides composite: the bound is kept as the heuristic scale it was;
- a rung with no bound is never given one.

It only removes or lowers bounds, so it can only widen a channel's support;
it never adds a bias. Where every single-leg side carries the hardest
threshold, the map key is unchanged and the run is bit-for-bit. This touches
the spacelike transfer bound, not a resonance map, window or `narrow_in_tau`;
the brief's wide-resonance coordination rule does not apply, but the manager
should still decide whether PR 11's owner wants to see it.

Tests (hermetic, `--lib`):
`a_side_bound_drops_only_what_no_single_leg_cut_proves` (uncut side → no
bound; a lower-cut side → its own square; a proven bound unchanged, map key
included; a harder side never raises it),
`dropping_a_side_bound_restores_the_collinear_support` (draws below the old
bound have positive density on the new channel and exactly zero on the old
one: the control that shows the test sees the bound),
`a_single_leg_side_bounds_the_transfer_at_its_own_threshold`.

## Gates

All σ runs below use the same command as the reproduction; summaries from
`/home/user/wt/lp1-bin/scatter.py` (unweighted mean, sd, scatter χ²/dof
against the quoted errors).

**Card 126, 20 seeds at `--neval 120000 --target-rel 0.01`** (binary
`vibegraph-fix1` = this commit plus the env-gated probe hooks, unset):

```
seed   1 2.51487e-04 ± 5.30e-07   ch4 3.6%  ch4 χ²/dof 0.65
seed   2 2.53562e-04 ± 6.16e-07   ch4 1.4%  2.34
seed   3 2.53377e-04 ± 5.38e-07   ch4 2.0%  2.06
seed   4 2.52646e-04 ± 6.28e-07   ch4 1.9%  1.21
seed   5 2.51786e-04 ± 5.26e-07   ch4 1.9%  1.48
seed   6 2.51243e-04 ± 5.51e-07   ch4 1.9%  1.81
seed   7 2.52081e-04 ± 5.48e-07   ch4 1.5%  1.51
seed   8 2.52166e-04 ± 5.32e-07   ch4 2.7%  0.08
seed   9 2.51122e-04 ± 5.33e-07   ch4 1.5%  2.28
seed  10 2.51726e-04 ± 5.52e-07   ch4 2.2%  2.32
seed  11 2.52248e-04 ± 5.30e-07   ch4 1.9%  1.75
seed  12 2.51673e-04 ± 5.29e-07   ch4 3.0%  0.26
seed  13 2.53592e-04 ± 1.58e-06   ch4 3.6%  56.28
seed  14 2.50952e-04 ± 5.30e-07   ch4 3.0%  2.26
seed  15 2.52226e-04 ± 5.44e-07   ch4 1.6%  1.01
seed  16 2.51693e-04 ± 5.46e-07   ch4 2.0%  0.67
seed  17 2.52090e-04 ± 5.33e-07   ch4 2.3%  1.23
seed  18 2.52123e-04 ± 5.64e-07   ch4 2.0%  4.98
seed  19 2.52882e-04 ± 5.77e-07   ch4 1.6%  2.04
seed  20 2.52195e-04 ± 5.51e-07   ch4 3.0%  1.34
n 20 mean 2.52144e-04 ± 1.69e-07 (sd 7.57e-07, mean quoted err 6.02e-07)
     scatter χ²/dof 1.54  min 2.5095e-04 max 2.5359e-04
```

- **The probe build is the committed code.** A clean release build of
  f70ab694 reproduces seed 2 bit for bit: `0.0002535615000139615 ±
  6.162057290876577e-07` from both `v0.2.2-15-gf70ab694` and the probe build.
- **One population.** All 20 seeds sit in 2.510–2.536e-4. The base code's low
  population was 2.384–2.41e-4.
- **The spread is close to the quoted errors.** Scatter χ²/dof is 1.54 on 19
  dof (p ≈ 0.07). The sd is 1.26× the mean quoted error, so the quoted errors
  are still slightly small at this budget.
- **Channel 4 is sane on 19 of 20 seeds** (χ²/dof 0.08–4.98). Seed 13 reads
  56: one channel-4 point of contribution 1.5e-14 with configuration **3**'s
  `AMP2` dominant and a hard `b̄` (pT 583 GeV, share to channel 2 0.49). That
  is a different, much smaller corner (seed 13 sits 1.9 sd above the 20-seed mean).
  The `b̄`-collinear hole itself no longer shows: every remaining heavy point
  in channel 2 has channel-2 share ≈ 1, including `t_b` down to −42 GeV².

**Budget axis, 20 seeds at `--neval 480000`:**

```
n 20 mean 2.51820e-04 ± 5.13e-08 (sd 2.29e-07, mean quoted err 2.62e-07)
     scatter χ²/dof 0.76  min 2.5126e-04 max 2.5219e-04
```

The two rungs differ by 3.2e-7 (0.13%), 1.8σ on their measured spreads,
with the larger budget *lower*. A missed region would push σ up with budget.
The first five 480k seeds alone read χ²/dof 0.13 (p ≈ 0.03), a warning
that cleared once 15 more were drawn.

**Unbound control (diagnostic, not the fix):** every rung's bound removed
(`without_transfer_bound`), 5 seeds at 120k: mean 2.52477e-4, sd 7.6e-7,
χ²/dof 1.92. It lands in the same population, which confirms the bound
caused the miss.

**Not judged against MadGraph.** The gridpack's 2.504e-4 is complex-mass
scheme and is not used here. The fixed mean sits 0.7% above it, which says
nothing until the CMS-off reference arrives.

**Banked rows, bit for bit against main a5bf8fca** (CLI, both binaries, seed
1, `--neval 30000 --target-rel 0.02`, the banked cards,
`/home/user/wt/lp1-bin/banked_bits.sh`):

```
pp_to_llj_fixed     base 423.46718524986636 ± 1.6136450550615036   fix identical
pp_to_jj            base 679559536.3867197  ± 1653668.5355061267   fix identical
pp_to_bb            base 376703814.9150388  ± 418002.69479062234   fix identical
pp_to_ll_0j2j_mlm   base 1073.1410298707576 ± 7.702552316260028    fix identical
pp_to_ttx_0j1j_mlm  base 1091.5412607830253 ± 2.752713109099068
                    fix  1092.0877921413414 ± 2.752050909976843    (moved, 0.14σ)
```

`pp_to_ttx_0j1j_mlm` moves as expected: in `p p > t t~ j` a rung has the
single uncut top on one side. Its gate, run on this commit:

```
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_hadronic -- --ignored --nocapture sigma_ttx_0j1j_mlm_vs_madevent
[… at0]  vibegraph σ = 5.132301e2 ± 1.288e-1 pb (10 seeds, χ²/dof = 0.83) | MadEvent 5.131058e2 ± 2.216e-1 | pull = +0.49 | gate: pass
[… at1]  vibegraph σ = 5.790925e2 ± 2.848e-1 pb (10 seeds, χ²/dof = 1.29) | MadEvent 5.783374e2 ± 1.128e0  | pull = +0.65 | gate: pass
[… total] vibegraph σ = 1.092323e3 ± 2.986e-1 pb (10 seeds, χ²/dof = 1.12) | MadEvent 1.091440e3 ± 1.149e0 | pull = +0.74 | gate: pass
test result: ok. 1 passed; 0 failed
```

The manifest's recorded measurement on main is total 1092.328 ± 0.2876
(pull +0.75), @0 513.1846, @1 579.1429. So the row moved within its error.
The manifest cell's note is not updated here (bookkeeping).

**Banked σ gates** (`--config 'profile.release-debug.debug=0'`):

```
cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --nocapture
test every_bounded_channel_set_covers_its_own_fiducial_region ... ok
test sigma_gate_matches_madgraph ... ok
test the_grammar_rows_match_madevents_seeds ... ok
test result: ok. 8 passed; 0 failed; 33 ignored; finished in 669.10s

cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_hadronic -- --nocapture
test result: ok. 20 passed; 0 failed; 26 ignored; finished in 212.88s
```

(`validate_hadronic` prints one panic message from
`lepton_part_refuses_an_unknown_directory_name`, a `should_panic` test.)

**Lints:** `cargo fmt --all --check` clean;
`cargo clippy --workspace --all-targets -- -D warnings` → `Finished`, rc 0.

**Hermetic suite** with `research/refs/mg5amcnlo` emptied, `validation/madgraph/output`
and the fetched `validation/pdf` sets moved aside (tracked files kept), then
restored:

```
cargo test --workspace
75 test binaries: 2307 passed, 0 failed, 28 ignored   (rc 0)
```

## What each oracle cannot see

- **The seed sweep (σ, spread, χ²/dof)** cannot see a corner that *every*
  seed misses. It sees a corner some seeds find. A 20-seed population is
  blind to a region drawn less than about once in 20 runs, and blind to any
  bias common to all seeds. A pinned seed can't separate "fixed" from
  "lucky", which is why the gate is a sweep.
- **The per-event probe** sees only points that were drawn. A region no channel
  ever reaches would never print. It names this corner because channels 4 and 1
  did reach it. Its threshold (5e-4 of σ per point) also hides diffuse
  under-sampling.
- **The 120k-versus-480k rung comparison** is blind to an offset both budgets
  share.
- **Bit-for-bit on four banked cards** shows the rule leaves rows whose
  single-leg sides all carry the hardest threshold untouched. It cannot see
  rows outside the sample. The moved row (`ttx_0j1j`) is checked only within
  its MadEvent gate's resolution (about 0.6% on @1).
- **The coverage gate** (`every_bounded_channel_set_covers_its_own_fiducial_region`)
  checks that the union of supports covers the accepted region. It is blind
  by design to a corner covered only by a channel with negligible density,
  which is this failure.
- **The unit tests** pin the rule and that its support change is real. They
  do not pin that the rule is *sufficient*: a composite side with uncut legs
  (both sides composite) still keeps the heuristic bound.

## vg-validator requests

1. **CMS-off MadGraph σ for card 126** (already in flight per the manager):
   MG 3.7.1, the staged cards with `set complex_mass_scheme` removed. The
   expected answer, if this fix is right, is near 2.52e-4 pb (vibegraph's
   20-seed means are 2.5214e-4 ± 1.7e-7 at 120k and 2.5182e-4 ± 5e-8 at 480k).
2. **The host's campaign row re-run on this branch** to confirm off-container:
   branch `sprintl-multichannel` at the commit carrying this report, card 126, seeds 1–20, the
   campaign command (`--target-rel 0.01 --neval 120000 -j 4
   --parallel-diagrams`). Expected: all seeds in 2.50–2.54e-4, scatter
   χ²/dof about 1–2, channel 4 at 1.4–3.6% of σ.
3. **The gpdb cohort with an uncut massive or b leg next to a cut jet**,
   re-run on this branch: single-top-like and `t t̄ j`/`b b̄ j` cards. The
   same bound applies there, and campaign rows that read low with small
   errors are candidates.

## Corrections to the brief and card file

- **lhaid 325500 is `NNPDF31_nnlo_as_0118_nf_4_mc_hessian`**, not
  `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` (that is 325300). This is from
  LHAPDF's `pdfsets.index`, and vibegraph's log confirms it: `PDF set:
  NNPDF31_nnlo_as_0118_nf_4_mc_hessian (member 0, lhaid 325500)`. A 4-flavour
  set suits the 4FNS model.
- **The miss is not a T′ tail and not a subdominant channel's region**: it is
  the dominant configuration's own collinear corner, cut out of its own map.
- **The brief listed mechanisms** (BW window, `narrow_in_tau`, adaptation), and
  none is the cause. The cause is the spacelike transfer bound.
- **"Seeds 2–4 … in 4 iterations"** is right in the host's table. The
  backlog item says "stopping at 6 iterations"; with the 2 warm-ups both
  describe the same runs.

## Found

1. **The backlog item's text, rewritten** for the manager to apply
   (`multichannel-subdominant-channel-carries-dominant-region`):
   *title:* "A transfer bound cut the dominant channel's b-collinear corner out of its map"
   *description:* "TprimeBToTZ M1800: configuration 2's spine bounded its b̄
   rung at −ptj² though the b̄ is uncut, so only channels 4 and 1 reached
   |t_b| < 100 GeV², as rare heavy points; seeds that missed them read ~5% low
   with small errors. Rung bounds now follow what a single-leg side proves."
   *closes_when:* unchanged, except "mean within MadGraph's error" is judged
   against the complex-mass-scheme-off reference. The body should replace the
   "below-peak tail of the broad T′" guess and the candidate fixes with the
   region and mechanism above. None of the three candidate fixes (α weighting,
   budget floors, drawing in channel 4 with configuration 2's weight) would
   have closed it: the corner had no density in the channel that owned it.
2. **Two composite sides keep a heuristic bound.** A rung whose line has
   composite systems on both sides keeps `t ≤ −spacelike_floor` with no proof.
   An uncut leg inside such a side can make the same kind of hole, at higher
   multiplicity. To close it: a measurement that drops those bounds too, on
   the banked ≥4-body spine rows (`ud_to_epemud_qcd0`, `ee_to_mumu_tata_qcd0`,
   the 2→6 rows) and on campaign cards, and keeps the drop only if no gated
   σ moves beyond its error.
3. **The coverage gate needs a density floor.** It counts an accepted point as
   covered if any channel's density is positive, which passes this card while
   the corner's only coverage was points weighing about 5 000 times channel 4's mean weight (seed 1's heaviest point alone carried 1.8× channel 4's whole integral). A check of
   the largest `f/g` over the accepted flat draws, per row, would have flagged
   it.
4. **The probe hook is reusable.** The env-gated per-point dump (`probe.patch`:
   `vegas.rs` passes the VEGAS weight, and `ChannelIntegrand::lp1_describe` is
   implemented for `ProtonIntegrand`) is the hadronic counterpart of
   `hadronic/tail_probe.rs`, which is fixed-beam only. It is worth landing as an
   ignored probe or a debug flag. It found this region in one run.
5. **Residual heavy corners on card 126**:
   - seed 13, configuration 3 with a hard `b̄`, drawn in channel 4: contribution 1.5e-14;
   - seed 15, channel 6, χ²/dof 137 on a 7e-9 pb channel.

   Both are small. They are why the scatter χ²/dof reads 1.54 rather than 1.0
   at 120k.
6. **gpdb's model_sha256 recipe does not reproduce** from the model-store
   tarball (six readings tried). Since vibegraph's digest and the restriction
   hash match, the gpdb recipe should be written down exactly.
