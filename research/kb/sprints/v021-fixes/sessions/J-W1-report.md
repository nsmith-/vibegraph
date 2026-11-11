---
type: Session Report
title: "J-W1 report: the pairing-weighted cut_decays exemption, its guard, its probe and its σ attribution"
description: "cut_decays = F with identical particles across a decay is now Σ_p w_p Θ_p |M|². The weighted integrand has no beam-collinear pole (w|M|²ε² falls as ε², |M|²ε² flat). σ reads +1.72% above MadGraph on g g > z u u~, z > u u~ and +0.44% on g g > z j j, z > j j. Per event that is the interference plus the window union, with zero from the exemption. MadGraph's rule read on vibegraph's events closes on MadGraph's σ to 0.1%."
status: draft
tags: [cuts, decay-chains, identical-particles, ir-safety, madgraph-parity, sprint-j]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-11}]
sources:
  - {id: brief, resource: "J-W1.md", title: "J-W1 brief"}
  - {id: d3, resource: "../../v02-last-refusals/decisions/D3-identical-decays-behind-madgraph-compat.md", title: "Sprint I D3"}
  - {id: ic1, resource: "../../v02-last-refusals/sessions/I-C1-report.md", title: "I-C1 report"}
  - {id: hk1, resource: "../../cms-gridpack-coverage/sessions/H-K1-report.md", title: "H-K1 report §5b"}
---
Branch `sprintj-weighted-cut` in `/home/user/wt/sprintj-w1`, off `main` 9a1349c.
The commits are listed in §7. `$S` below is the scratch directory
`/tmp/claude-0/-home-user-vibegraph/916acef5-3525-525b-9ed0-06971289659f/scratchpad/jw1`.
The user's option A applies: the weighted cut is the only behaviour, and no
`--madgraph-compat` is built.

## 1. What changed

**The partition (scope 1).** A subprocess's diagrams are grouped by the forced
lines they carry (`cuts::diagram_forced_lines`, sorted). One group is one
pairing: one assignment of the identical particles to the decays. Every
stitched diagram forces exactly one set of lines, so the groups partition the
diagrams. `ForcedResonances::of` keeps a `Pairing` per group. Each holds the
lines, the decay products, and that group's own massless-coloured core legs and
`MasslessLimit`s.

**When it applies.** `cut_decays = F` and
`ForcedResonances::exemption_varies` must both hold. The second means the
pairings that force lines do not all have the same decay products. Otherwise
`compile_with` runs the code it ran before, `compile_exempting` with the
every-diagram decay products and the guards on. Two tests pin that this filter
is unchanged:
- `a_filter_without_a_varying_exemption_is_unweighted` checks that `Cuts`
  equals the old body's output on four cards: `g g > z u u~, z > d d~` at F,
  `g g > z u u~, z > u u~` at T, and `e+ e- > z z, z > e+ e-` at F and T. It
  also checks that `pass_pairings ∈ {0,1}`.
- The banked layer (§5).

**The filter.** `Cuts` carries one complete `PairingFilter` per distinct
exemption. `Cuts::pass_pairings` returns a bit per filter, and `pass` is
"some bit set". The outer fields are the filter that exempts the union of the
exemptions. It accepts every point a pairing filter accepts, so any bound read
off it is conservative. The four floors and `shat_min`/`shat_max` take the
weakest over the pairing filters, as `UnionBounds` does over groups. The
decay-chain windows are the same union in every filter. Keeping them the same
keeps `cut_decays` orthogonal to the window: at T the union applies, and Θ_p
differs from it only in the exemption.

**The weights (scope 2).** `pairing.rs` holds the weight code:
- `PairingSplit` compiles one amplitude per pairing from that pairing's diagrams.
- `BoundPairings::weights` returns, for each pass mask:
  - 1 when every pairing's bit is set;
  - 0 when none is;
  - otherwise Σ_{p∈mask}|M_p|²/Σ_q|M_q|², with every |M_p|² evaluated once at
    the main subprocess's coupling.

It reaches the integrands in two ways:
- **Proton path.** The split lives on `FlavorGroup`, built in
  `derive_flavor_groups` from the representative. Each ordering's luminosity is
  multiplied by its own event's weight. Both orderings read the sampled point's
  amplitudes, as they read its |M|². The weights travel with the event
  (`ProtonEvent::group_pairing_weights`), so the group/ordering draw uses the
  weighted terms while the record's `m2` stays the whole |M|².
- **Fixed-beam path.** The callers build `subprocess_pairing_splits` and install
  it with `use_pairing_weights`. A pairing-weighted filter without installed
  amplitudes panics rather than integrate the unweighted union. The callers are
  `integrate.rs` and `generate.rs`.

**Cost (scope 1).** On `g g > z u u~, z > u u~` (32 diagrams in 4 pairings, dev
profile, scratch timing over 2000 RAMBO points at √ŝ = 500):
- |M|² costs 33.2 µs per point;
- the four pairing amplitudes together cost 18.1 µs;
- `pass_pairings` costs 2.3 µs.

The pairing amplitudes are evaluated only at mixed points. Those are 25% of the
F events on that card (5079 of 20000) and 6.2% on the whole `g g > z j j,
z > j j` card (2495 of 40000).

**The guard (scope 3).** At a weighted card, `compile_pairing_weighted` runs
`refuse_ir_unsafe_exemption` and `refuse_unregulated_limits` once per exemption.
Each run reads the `MasslessLimit`s and core legs of that exemption's own
pairings (`ForcedResonances::of_exemption`). The outer filter is compiled with
the guards off. The every-pairing guard over the union of exemptions would
refuse the very divergence the weights cancel.

**Docs (scope 6).** The following were rewritten to the measured numbers below:
- the `cut_decays` paragraph of the `cuts.rs` module docs;
- `release-scope.md`'s third deviation;
- the identical-particles concept's `cut_decays = F` section, its gating
  section and its description;
- the `madgraph-defects.md` register row.

The old −26% and the D3 "~0.2%" for this case are gone from all three
documents.

## 2. The probe (scope 4)

`pairing::tests::the_weighted_integrand_has_no_beam_collinear_pole` rebuilds
I-C1's probe as a committed test, since I-C1's source was in a lost scratchpad.
The kinematics are as follows:
- Leg 0, a u, has E = 40 and pT = ε along +z.
- Leg 1, a ū, is at polar angle 2.6, with the energy that puts m₀₁ = M_Z exactly.
- Legs 2 and 3 sit back to back across the beam in the rest frame of the
  remainder, whose mass is set per case.

The geometry is mine, not I-C1's, so the coefficients differ from its table.
The card is F with `ptj 20`, `etaj 5`, `drjj 0.4`. The command:

```
CARGO_TARGET_DIR=/home/user/cargo-target-jw1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p vibegraph-lib --lib pairing:: -- --nocapture
case 1 eps    1e-2 passed 0101 |M|^2 eps^2 1.389454e-3 w 5.475e-6 w|M|^2 eps^2 7.607276e-9 control |M|^2 eps^2 9.238e-9
case 1 eps    1e-3 passed 0101 |M|^2 eps^2 1.388170e-3 w 5.480e-8 w|M|^2 eps^2 7.606704e-11 control |M|^2 eps^2 9.237e-11
case 1 eps    1e-4 passed 0101 |M|^2 eps^2 1.388089e-3 w 5.480e-10 w|M|^2 eps^2 7.606647e-13 control |M|^2 eps^2 9.237e-13
case 2 eps    1e-2 passed 0101 |M|^2 eps^2 1.022357e-2 w 1.040e-6 w|M|^2 eps^2 1.063086e-8 control |M|^2 eps^2 1.413e-8
case 2 eps    1e-3 passed 0101 |M|^2 eps^2 1.024898e-2 w 1.037e-8 w|M|^2 eps^2 1.062968e-10 control |M|^2 eps^2 1.413e-10
case 2 eps    1e-4 passed 0101 |M|^2 eps^2 1.025136e-2 w 1.037e-10 w|M|^2 eps^2 1.062956e-12 control |M|^2 eps^2 1.413e-12
case 3 eps    1e-2 passed 0101 |M|^2 eps^2 6.754372e-4 w 1.050e-5 w|M|^2 eps^2 7.093362e-9 control |M|^2 eps^2 8.337e-9
case 3 eps    1e-3 passed 0101 |M|^2 eps^2 6.750433e-4 w 1.051e-7 w|M|^2 eps^2 7.092848e-11 control |M|^2 eps^2 8.336e-11
case 3 eps    1e-4 passed 0101 |M|^2 eps^2 6.749902e-4 w 1.051e-9 w|M|^2 eps^2 7.092796e-13 control |M|^2 eps^2 8.336e-13
test pairing::tests::the_weighted_integrand_has_no_beam_collinear_pole ... ok
```

The cases are m₂₃ = 160, 111.19 and 180, I-C1's three. What the columns show:
- **Unweighted control.** |M|²ε², the integrand of the union filter with no
  weights, is constant. It agrees to 2.3e-4 at worst between ε = 1e-3 and 1e-4
  and to 2.7e-3 over the two decades. It is the 1/pT² pole.
- **Weighted integrand.** w|M|²ε² falls by exactly 1e4 over two decades.
  w|M|² itself is finite (≈ 7.6e-5, the size of the distinct-flavour |M|²).
- **Accepting filters.** Two pairing filters accept the point, `0101`: the
  on-pole pair's and the one exempting legs 0 and 3. Both weights fall as ε².
- **Distinct-flavour control.** `z > d d~` has no pole: |M|²ε² ∝ ε².

The test asserts:
- constancy of the control to 1e-3 over the last decade and 1e-2 over both;
- the weighted fall at more than 1e3;
- the control falling at more than 1e3.

**Blind spot.** The probe tests one limit, an exempt quark collinear to a
beam, at three points of the remaining phase space. It cannot see a divergence
in a limit it does not drive. The per-exemption guard and the Cauchy–Schwarz
bound cover those.

## 3. Other oracles

Each line is the command `cargo test -p vibegraph-lib --lib <name> -- --nocapture`
on ac74b42, with its output.

- **The identity.**
  `pairing::tests::the_weighted_integrand_is_each_pairing_s_own_exemption_on_the_incoherent_sum`:
  `1846 points with a fractional weight; worst relative difference 3.38e-16`.
  - It compares Σ_p w_p Θ_p |M|² from the production weights against
    Σ_p Θ_p |M_p|²·|M|²/Σ_q|M_q|², built from pairing amplitudes compiled in the
    test.
  - The identity is algebraic once w_p is defined. What the test pins is the
    implementation: bit mapping, partition and evaluation.
  - It cannot see an error shared by both sides, the partition rule or
    `compile_class`.
- **Each ordering's weight.**
  `proton::tests::each_ordering_carries_the_pairing_weight_of_its_event`:
  `pairing weights: worst 1.59e-12 against each written event's own; 478 fractional weights; 17 points whose orderings pass different pairings`.
  - The setup is `d d~ > z u u~, z > u u~` from `define q = d d~; generate q q > …`
    (mirrored), beams 6500/4000, `etaj 1.5`.
  - The oracle compiles the mirrored ordering's pairing amplitudes from its own
    line, `d~ d > …`, evaluates them at the exchanged event, and applies the
    group's filters to `record_momenta`.
  - It cannot see the filters' thresholds, which both sides share.
  - The beam quarks are deliberately not the identical flavour: see Found 1.
- **The guard.** `cuts::tests::a_pairing_whose_own_decay_is_collinear_singular_is_refused`:
  - At F, `e+ e- > z u u~, z > u u~ u u~` is refused with
    `UncutCollinearSplitting` and regulator `cut_decays = T …`. The decay's own
    g* → u ū splits among the exempt legs of a pairing.
  - The pre-change rule (`compile_exempting` with the every-diagram products,
    which are none here) compiles the same card, asserted in the test. So the
    refusal fails before the change.
  - At T it compiles.
  - A card where some products are decay products in every pairing (a gluon
    in a 3-body decay) is refused by the old guard already. That is why the
    refusal card has every product identical to a core quark.
- **The filters.** `cuts::tests::identical_quarks_across_a_decay_compile_one_filter_per_pairing`:
  - four filters, each cutting exactly its two core legs;
  - the spacelike floor stays 400;
  - a point with one pairing's pair on the pole at pT 0.5 near the beams passes
    exactly that pairing's bit.
- All 95 `cuts::` tests pass, including the pre-existing exemption-refusal
  test, whose floor assertion is unchanged.

## 4. σ against MadGraph (scope 5)

**Settings.** MadGraph is the pinned submodule (3.7.1) through
`madevent_seeds.sh` in the `madgraph` pixi env (LHAPDF 6.5.6):
- `$S/jw1-mg.sh`: a fresh copy of the generated directory per seed, iseed 1–5;
- `hard_survey 2`, `nevents 10000`, `nb_core 3`;
- each run checked by `mes_check_run`: positive σ and 10000 events.

The run card for both sides is MadGraph's default with these overrides:
`ptj 20`, `etaj 5`, `drjj 0.4`, fixed scales 91.188, `pdlabel lhapdf`,
`lhaid 247000`, `cut_decays F|T` (`$S/jw1-rc-*.dat`).

One deviation: MadEvent downloaded `NNPDF23_lo_as_0130_qed` into the env's
LHAPDF directory rather than reading `validation/pdf`. It is the same set and
the same lhaid.

vibegraph is the release build of 4b5d3bb (`$S/jw1-vibegraph`), run as:
`integrate <proc> --run-card <rc> --seed s --target-rel 0.003 -j 3`, from
`$S/jw1-vg.sh`. The integrand code is unchanged from 4b5d3bb to ac74b42.
Stats come from `$S/jw1_stats.py`: unweighted mean, sd, se as sd/√5, and
χ²/dof about the mean.

```
### g g > z u u~, z > u u~, cut_decays = F
vibegraph  n=5 mean 7.3302 pb sd 0.0151 (2.06e-03) se(scatter) 0.00675 se(quoted) 0.00895 chi2/dof 0.56
           seeds: 7.314±0.019, 7.317±0.02, 7.333±0.021, 7.351±0.021, 7.336±0.019
madgraph   n=5 mean 7.206 pb sd 0.0299 (4.15e-03) se(scatter) 0.0134 se(quoted) 0.0124 chi2/dof 1.11
           seeds: 7.2468±0.033, 7.2003±0.027, 7.1728±0.025, 7.1854±0.025, 7.2246±0.028
vg-mg: +0.1242 pb (+1.724e-02); pull +8.30 (scatter), +8.13 (quoted)
### g g > z u u~, z > u u~, cut_decays = T
vibegraph  n=5 mean 5.361 pb sd 0.00819 (1.53e-03) se(scatter) 0.00366 se(quoted) 0.00662 chi2/dof 0.31
           seeds: 5.348±0.015, 5.358±0.015, 5.365±0.015, 5.366±0.015, 5.368±0.014
madgraph   n=5 mean 5.2876 pb sd 0.0195 (3.68e-03) se(scatter) 0.00871 se(quoted) 0.00931 chi2/dof 0.86
           seeds: 5.2853±0.023, 5.2864±0.018, 5.3067±0.019, 5.2572±0.022, 5.3025±0.021
vg-mg: +0.07338 pb (+1.388e-02); pull +7.77 (scatter), +6.42 (quoted)
### g g > z j j, z > j j, cut_decays = F
vibegraph  n=5 mean 150.76 pb sd 0.302 (2.01e-03) se(scatter) 0.135 se(quoted) 0.177 chi2/dof 0.59
           seeds: 150.96±0.36, 150.48±0.37, 150.43±0.41, 151.13±0.41, 150.8±0.42
madgraph   n=5 mean 150.09 pb sd 0.515 (3.43e-03) se(scatter) 0.23 se(quoted) 0.256 chi2/dof 0.78
           seeds: 150.35±0.62, 149.97±0.59, 149.44±0.55, 149.9±0.46, 150.81±0.62
vg-mg: +0.666 pb (+4.437e-03); pull +2.50 (scatter), +2.14 (quoted)
```

**Consistency with earlier runs.** vibegraph's T seed 1 is 5.348 ± 0.015, H-K1's
number exactly. The T path is unchanged by this work. MadGraph's seed 1
reproduces H-K1's single-seed 7.211 ± 0.028 (7.247 ± 0.033 here, mean 7.206)
and H-K1's 150.61 five-seed mean (150.09 here; H-K1 is 1.6 se away).

The rows are not expected to match: this is an attributed difference.

**The attribution, per event.** The σ rows alone cannot split the difference,
so the attribution is a per-event oracle. Take vibegraph's own unweighted events
(`generate --seed 7`, 20000 per ggzuu grid and 40000 for the jj grid). On each
event evaluate, in the partonic CM, |M|² and every |M_p|². In the lab frame,
evaluate the pairing filters Θ_p and each pairing's own Z window W_p (bwcutoff
15). MadGraph's rule is its single pairing, with its own window and its
configuration-1 exemption, on its single-pairing |M|². By the exact factor of 2
per identical pair, that is Σ_p Θ_p W_p |M_p|². Its σ is predicted as
σ_vg × ⟨Σ_p Θ_p W_p |M_p|² / integrand⟩ over the events. The script is the
scratch test `$S/jw1_events.rs` / `$S/jw1_events_jj.rs`, run from
`vibegraph-lib/src/pairing.rs` and removed afterwards.

```
ggzuu F: events 20000; mixed-filter events 5079; <Σ|M_q|²/|M|²> = 0.99218; <MG-reading/integrand> = 0.98382 ± 0.00061; window-only factor <W> = 0.99142
ggzuu T: events 20000; mixed-filter events 0;    <Σ|M_q|²/|M|²> = 0.99209; <MG-reading/integrand> = 0.98592 ± 0.00050; window-only factor <W> = 0.99365
ggzjj F: same-flavour events 10163, others 29837; mixed-filter events 2495; <Σ|M_q|²/|M|²> = 0.99760; <MG-reading/integrand> = 0.99576 ± 0.00022; window-only factor <W> = 0.99813
```

| card | vibegraph | predicted MadGraph | MadGraph measured | closure | interference | window union | exemption |
|---|---|---|---|---|---|---|---|
| ggzuu F | 7.330 ± 0.007 | 7.212 ± 0.008 | 7.206 ± 0.013 | +0.08 % | +0.78 % | +0.86 % | 0 |
| ggzuu T | 5.361 ± 0.004 | 5.286 ± 0.005 | 5.288 ± 0.009 | −0.04 % | +0.80 % | +0.64 % | – |
| ggzjj F | 150.76 ± 0.14 | 150.12 ± 0.14 | 150.09 ± 0.23 | +0.02 % | +0.24 % | +0.19 % | 0 |

**Why the exemption contributes zero.** The identity of §3 makes the weighted
integrand, event by event, MadGraph's per-pairing exemption on Σ_p Θ_p |M_p|²,
multiplied by |M|²/Σ_q|M_q|². What is left against MadGraph is therefore:
- the interference MadGraph drops, the ⟨Σ_q|M_q|²/|M|²⟩ factor;
- the window: vibegraph's union over pairings against each pairing's own
  window, the ⟨W⟩ factor.

At T the same two factors apply, with the cut identical on both sides.

Notes on reading the table:
- The jj card's "others" events, its twelve distinct-flavour subprocesses, enter
  with a factor 1. H-K1 measured those subprocesses to agree to 1e-4.
- The interference on these cards is 0.8% of the same-flavour σ, not D3's
  ~0.2%. D3's figure is the `z z → 4e` study's. Here the core quark pair is
  close in phase space to the decay pair.

**What the attribution cannot see:**
- It assumes MadGraph's single-pairing |M|² is our pairing subset's |M_p|². That
  is not dumped per diagram. The closure to 0.1% on three rows is the evidence
  for it, not a proof.
- It reads vibegraph's events. A region MadGraph accepts that vibegraph's union
  does not would be missed, but MadGraph's region (its own window, its own
  exemption) is a subset of the union.

**On the pulls.** The σ pulls (+8.3, +7.8, +2.5) measure the registered
deviation, not a sampler problem:
- χ²/dof is 0.31–0.59 on vibegraph and 0.78–1.11 on MadGraph;
- the per-event closure is within 1 combined se on every row.

**Budget.** Five seeds at 0.3% read no missed region, but they say nothing about
budget convergence. No higher-budget rung was run. That is fine for an
informational row, and it would be needed before anyone gated on it.

## 5. Gates

All gates ran on ac74b42 in `/home/user/cargo-target-jw1`, with debug info and
incremental compilation off.

GATES

## 6. Item and brief corrections

- **The item's `closes_when`.** It reads "With --madgraph-compat off". Under
  option A there is no flag, so the condition is simply "cut_decays = F is
  applied as Σ_p w_p Θ_p |M|²".
- **The probe source.** The brief says the probe `ic1_window_probe.rs` "is in the
  report". Only its description and log excerpts are. The source lived in a
  sprint I scratchpad that no longer exists, so §2 is a rebuild.
- **The brief's numbers.** "MadGraph 150.61, 5 seeds" was LHAPDF 247000 (H-K1),
  as here, so it is comparable. "7.211 ± 0.028" was one seed; five are above.
- **"Every other card bit for bit, pinned by a test".** The test pins the
  compiled filter. Bit-for-bit σ is pinned by the banked layer. A weighted
  card's σ moves by construction.
- **Wording for site (d) of `madgraph-compat-sites-unconditional`**, for the
  manager to apply:
  - **Off (the default):** "every pairing coherently, the window union, and
    `cut_decays = F` through the pairing-weighted cut (implemented in v0.2.1,
    sprint J)". Drop "Until that is implemented … D2's stricter rule".
  - **Size:** "+1.72% (F) / +1.39% (T) on `g g > z u u~, z > u u~`, +0.44% on
    `g g > z j j, z > j j` at F (sprint J, J-W1): per event +0.78% interference
    and +0.86% window union, zero from the exemption". This replaces "about
    −0.2% for the pairing; −26% / −6.8% for the stricter cut".
  - **History:** add "option A (user, 2026-11-11): no flag is built; MadGraph's
    single pairing stays this site".

## 7. Commits

COMMITS

## Found

1. **`u~ u > u u~` compiled as its own line disagrees with its exchanged
   ordering.** This is an amplitude-level asymmetry, and the manager is
   handling it.
   - **The numbers.** At √ŝ = 500, θ = 0.7, φ = 0.3: `u u~ > u u~` at cm gives
     4.709463437305e3. `u~ u > u u~` at the exchanged event (`exchange_into`)
     gives 4.398818122664e3, a relative difference of 6.6e-2.
   - **Controls that agree to 0.0:**
     - `u~ u > u~ u` at the exchanged event with its outgoing legs swapped;
     - the C-conjugate lines `u~ u > u~ u` at cm, `u~ u~ > u~ u~`, `u~ c~ > u~ c~`;
     - the distinct-flavour `u u~ > d d~`.
   - **Random RAMBO points.** Up to 12.5% on `u~ u > u u~`, 14.7% on
     `u~ u > u u~ g`, 27% on `u~ u > z u u~`, 33% on `u~ u > z u u~, z > d d~`.
   - **The reproducer** was the ignored test
     `antiquark_first_line_with_its_flavour_in_the_final_state` at 4b5d3bb. It
     was removed at ac74b42 on the manager's instruction, since PR 5 adds its
     own enforced version.
   - **Reach.** `p p > j j` never compiles `u~ u` as its own line: the ordering
     is folded into the `u u~` set's `mirrored` flag, and its only antiquark-first
     sets are all-antiquark. These reach paths are not checked:
     - explicit `u~ u > …` lines;
     - `p p~` beams;
     - `DiagramSet::with_final_order` in `content_ordered`;
     - fixed-energy `e+ e- > e- e+`.
   - **To close it:** per-diagram dumps of `u~ u > u u~` against MadGraph
     standalone, then the fix. Making that row enforced also closes it.
2. **Systematics and reweighting do not see the pairing weight.**
   `reweight/engine.rs` forms new/old |M|² ratios on the whole |M|². On a
   weighted card the weight w = Σ_{p∈Θ}|M_p|²/Σ|M_q|² also depends on the
   parameters reweighted (couplings, masses), and that is not propagated. The
   evidence is reading the code; nothing was measured. To close it: either
   reweight with the weighted integrand, or refuse reweighting on
   pairing-weighted cards.
3. **The fixed-beam path reads one `ForcedResonances` over every subprocess's
   diagrams.** That predates this work (`integrate.rs`, `generate.rs`). On a
   fixed-beam card mixing a same-flavour chain with distinct-flavour ones
   (`e+ e- > z j j, z > j j`), the old intersection rule cut every leg of every
   subprocess. It now goes weighted, and each subprocess's single pairing gets
   its own exemption. That is MadGraph's behaviour on those subprocesses, but
   it is a change on such cards, not measured here. To close it: one fixed-beam
   σ row (`e+ e- > z j j, z > j j` at F) against MadGraph.
4. **Members of a flavour group share the representative's pairing
   amplitudes.** They are grouped on equal |M|² traces and equal filters, which
   include the exemptions, but the per-pairing |M_p|² of a member is not
   compared. To close it: extend the grouping probe to the pairing amplitudes
   of weighted groups.
5. **No budget rung on the weighted rows.** See §4. To close it: two seeds at
   `--target-rel 0.001` against the five at 0.003, if these rows are ever
   gated.
