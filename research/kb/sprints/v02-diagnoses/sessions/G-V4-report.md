---
type: Session Report
title: "G-V4 report: a Breit–Wigner τ mixture for a narrow whole-state resonance; generate's panic named; the small-width deviation registered"
description: "p p > h z through a 1 MeV Vz at 3.5 TeV scattered by 17% over five seeds under the log τ map (in-run χ²/dof up to 52). A Breit–Wigner component per whole-state pole, mixed with the log map at the channels' shares, gives 185698 ± 23 pb over 25 seeds (χ²/dof 0.78) against MadEvent's 185757 ± 9 (five seeds): 3.2e-4 low, inside the 1.2e-3 bound MadEvent's small-width treatment sets without a window. The rule switches only on a peak under 1e-3 of the log map's unit interval, so every banked row and card 048's chain keep their maps bit for bit. generate's scan panic is the named refusal NoUsableWeight."
status: draft
tags: [phase-space, tau-map, breit-wigner, small-width, madgraph-deviation, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
measured:
  host: "cloud container, 4 vCPU shared with sibling sessions, linux-64, 2026-11-04"
  commit: "base 546c566 (CLI sha256 a60b53a0…); change 4b20d60 + faa34b9 (CLI built from 4b20d60's source before the test-only clippy fix, sha256 d81c342f…)"
  command: "see Method; every run's CLI output under the session scratchpad, G-V4/runs/<tag>-s<seed>/output.txt"
sources:
  - {id: brief, resource: "G-V4.md", title: "G-V4 brief"}
  - {id: g-v1, resource: "G-V1-report.md", title: "G-V1 report"}
  - {id: maps, resource: "../../../../../vibegraph-lib/src/phasespace/maps.rs", title: "TauMap::BreitWigner, NARROW_TAU_SHARE, narrow_in_tau"}
  - {id: proton, resource: "../../../../../vibegraph-lib/src/proton.rs", title: "TauMixture"}
  - {id: deviation, resource: "../../../process/madevent-small-width-treatment.md", title: "The small-width treatment, registered deviation"}
---
## Verdict

- **Breit–Wigner τ map (G8): done, behind a measured rule.** `TauMap::BreitWigner`
  (`--map-tau breit-wigner`) draws `τ` from the mean, over the
  `(group, diagram)` channels, of the density MadEvent maps each channel's `τ`
  with. A channel whose whole-state pole lies in `[τ_min, τ_max]` contributes
  a Breit–Wigner in `τ` at that pole (MadEvent's `transpole(M²/s, MΓ/s)`, at
  the model's own width); every other channel contributes the log map. The
  draw splits `u₀` into one segment per component and weights every point by
  `1/(τ p_mix(τ))`. It is composed with G-V1's `τ_max` bound, and VEGAS still
  adapts on `u₀`. `auto` takes the mixture only where some whole-state pole is
  **narrow**: its peak under `NARROW_TAU_SHARE = 1e-3` of the log map's unit
  interval, `(2Γ/M)/ln(τ_max/τ_min)`. Otherwise the rule is unchanged: `log`
  with a wider whole-state resonance, `inverse-square` without one.
- **`p p > h z` is fixed and agrees with MadEvent.**
  - Under the log map, five seeds read 147640 to 224658 pb, with in-run
    χ²/dof up to 51.6. This reproduces G-V1 to every digit.
  - Under the mixture, 25 seeds read **185698 ± 23 pb**: sd 117 against a
    quoted 133, seed χ²/dof **0.78**, in-run χ²/dof 0.09 to 1.99.
  - Fresh MadEvent 3.7.1 at the same card and PDF, five seeds: **185757 ± 9**
    (from the seed spread). Ratio **0.99968 ± 0.00013**.
- **The ~1.5 % small-width offset the brief expected does not apply here.**
  - That figure is the share of a decay chain's `bwcutoff` window under each
    Breit–Wigner, and `p p > h z` has no window.
  - Without a window, MadEvent's `fk_W/Γ` Jacobian factor cancels the floored
    width over the peak. Only the off-peak part is mis-weighted, by at most
    `(fk_W/Γ − 1)K`, where `K` is the σ's width-independent part, measured at
    87 ± 16 pb.
  - That puts MadEvent above this side by **0 to 1.2e-3**. It reads
    **+3.2e-4 ± 1.3e-4**, so the floor accounts for the whole residual.
  - The lineshapes show the floor directly. The share of events within 0.5 MeV
    of the pole is 0.489 here (0.500 predicted for Γ = 1 MeV) and 0.176 in
    MadEvent's sample (0.177 predicted for 3.5 MeV).
- **Nothing banked moves.**
  - No banked σ, unweighting or generation row has a narrow whole-state pole.
    DY's `Z` sits at about 5e-3 of the unit interval, a factor five above the
    threshold.
  - Before and after the change, `p p > e+ e-`, `p p > j j` and
    `p p > z, z > e+ e-` give identical σ and errors to every printed digit.
  - Card 048's five decay-chain seeds are identical too. Its windowed `Vz`
    sits at 9.5e-3 and stays on the log map.
  - With the mixture named, card 048's per-seed error falls from 153 to 88 pb
    at the same mean.
- **No artifact format bump is needed.** The map is banked by name in
  `MapChoices` (bincode, variant index), and `Log` and `InverseSquare` still
  mean what they meant. Only new artifacts can record `BreitWigner`. An older
  build refuses to decode the unknown variant rather than replaying it.
- **`generate`'s panic is a named refusal.** `Unweighter::try_scan_with`
  returns `NoUsableWeight`, and `generate` surfaces it as
  `GenerateError::NoUsableWeight` (exit 1). The CLI test integrates
  `e+ e- > mu+ mu-` at 45.6 GeV beams with `ptl = 50`, which banks σ = 0, then
  generates from that artifact. The base build panics there with exit 101.
- **The small-width deviation (G9) is written up** as
  `process/madevent-small-width-treatment.md`. It is linked from the defect
  policy's table, the defect register and the release scope's "Deviating from
  MadGraph".

## The map

`proton.rs`, `TauMixture`:

- **Components.** `TauMixture::of` walks every group's `channel_diagrams`, the
  `(group, diagram)` pairs the channels are built from. It reads each
  diagram's whole-state resonance with
  `DiagramChannel::whole_state_resonance`, which now returns `(M, Γ)`. A pole
  strictly inside `(τ_min, τ_max)` counts toward its `(M, Γ)` component, and
  anything else toward the log component. The weights are the counts over the
  total. With no pole in range the mixture is `None`, and the map arm is the
  log map's own expression, bit for bit.
- **Draw.** The log segment is `[0, c_log)`, and each pole's segment follows.
  In a pole's segment, `v ∈ [0, 1]` maps to `z = z_lo + v(z_hi − z_lo)` and
  `τ = τ₀ + w tan z`, with `z_lo = atan((τ_min − τ₀)/w)` and
  `z_hi = atan((τ_max − τ₀)/w)`. So the component spans exactly
  `[τ_min, τ_max]`.
- **Weight.** `(dτ/du)/τ = 1/(τ p_mix(τ))`, with
  `p_mix = c_log/(τ ln(τ_max/τ_min)) + Σ c_k w/(((τ − τ₀)² + w²)(z_hi − z_lo))`.
  A point reached through either component carries the same weight: this is
  the multichannel estimator over the `τ` components.
- **On `p p > h z`** there are four `(group, diagram)` pairs: `u u~`, and
  `d d~` grouped with `s s~`, each with a `Z` and a `Vz` diagram. They give
  `log (0.500)` and `Breit-Wigner at M = 3500 GeV, width 0.001 GeV (0.500)`.
  The `Z` (600 GeV on card B) is below the `h Z` threshold, so it is not a
  component.
- **On card 048's chain**, with the mixture named, the components are
  `log (0.000)` and the `Vz` at 1.000.

**Why a mixture rather than MadEvent's per-channel maps.** Here every channel
shares the outer `(τ, y)` map, and the combiner's density `g` is the inner
channels' density at fixed `ŝ`. A per-channel `τ` map would need every
channel's `τ` density inside `g`. The mixture averages the same densities once
instead. That needs no change to the combiner, keeps the continuum covered by
the log component, and lets VEGAS move points between components.

**Why a threshold, and where it sits.** `MapOptions::resolve` writes no rule
without a measurement behind it.

- The log map is measured fine on DY's `Z` (share 4.9e-3 at `m_ll > 50 GeV`,
  13 TeV) and on card 048's windowed `Vz` (9.5e-3).
- It is measured broken on `p p > h z`'s `Vz` (7e-8).
- `1e-3` keeps every case measured fine on the log map with a factor of five
  to spare, and switches the broken one with four orders of magnitude to
  spare.
- The case in between, an `h` at 5e-6 in a process where the `h` matters, has
  no measurement either way (Found 4).

## Method

The inputs are G-V1's, read from its scratch and never committed:

- the gridpack UFO, `Vector_Triplet_free_Width_UFO`;
- card B's param card: `MVz = 3500`, `WVz = WZ = WH = 1 MeV`, `MZ = 600`,
  `MH = 170`;
- the run card with `lhaid = 247000` (`run_card_nn23.dat`).

The proc card is the one from G-V1's Found 2:

```
import model Vector_Triplet_free_Width_UFO
define p = g u d s u~ d~ s~
generate p p > h z
```

Every run goes through `G-V4/runvg.sh`:

```
vibegraph integrate proc_card.dat --run-card run_card_nn23.dat --param-card param_card.dat \
  --neval 100000 --niter 8 --fixed-budget --seed S -j 2 --no-tui --ufo-dir models
```

The seed statistics come from `G-V4/stats.py`, which is G-V1's script. It
reports the unweighted mean, the sample sd, the error of the mean as sd/√n,
the rms of the quoted errors, and χ²/dof about the mean from the quoted
errors.

MadEvent is a copy of G-V1's pinned 3.7.1 tree:

- `generate p p > h z` with the same `define p`;
- `output` gives `P1_qq_hz`, with channel G1 for the `Z` and G2 for the `Vz`;
- `generate_events` uses G-V1's run card (`pdlabel = nn23lo1`,
  `nevents = 10000`, `gridpack = False`, `use_syst = False`), card B, 2 cores,
  and `iseed` 11 to 15.

MadEvent's log reads `Setting PDF BW 3 1 3500.0` and
`Transforming s_hat BW 6.623e-2 6.623e-8`. So its `Vz` channel maps `τ` on a
Breit–Wigner at the floored width (`6.623e-8 = M · 1e-6 · M/s`).

## Seed tables

`p p > h z`, 100000 × 8, lhaid 247000:

| build, map | n | mean (pb) | sd | error of mean | rms quoted | seed χ²/dof |
|---|---|---|---|---|---|---|
| base, log (auto) | 5 | 175679 | 29880 | 13360 | 28630 | 2.28 |
| change, breit-wigner (auto), seeds 1–5 | 5 | 185712 | 59 | 27 | 134 | 0.20 |
| change, breit-wigner (auto), seeds 1–25 | 25 | 185698 | 117 | 23 | 133 | 0.78 |
| change, `WVz = 3.5 MeV` (the floored width) | 10 | 53118.6 | 29.1 | 9.2 | 36.6 | 0.64 |

- **Base, per seed:** 224658 ± 52054, 147640 ± 10532, 179939 ± 24211,
  158106 ± 18045 and 168051 ± 19118 pb. In-run χ²/dof: 51.6, 42.4, 4.3, 24.0,
  20.4. These are the brief's 1.48e5–2.25e5 pb and 51.6, digit for digit.
- **Change, 25 seeds:** 185527 to 185881 pb. In-run χ²/dof of the last
  iteration, sorted: 0.088 0.132 0.207 0.362 0.451 0.471 0.545 0.550 0.565
  0.577 0.636 0.672 0.754 0.882 0.920 0.931 1.017 1.031 1.049 1.122 1.228
  1.364 1.637 1.843 1.989.
- **Few-seed scatter:** the first five seeds read χ²/dof 0.20, and twenty more
  gave 0.78, which is AGENTS.md's warning about few-seed scatter again.
- **Budget:** each run takes 10 to 12 s, so cost is not the constraint. The
  base map's seeds would not converge at any budget, because the peak stays at
  7e-8 of `u₀`.

Card 048's chain (`p p > vz, (vz > h z)`, `define p` with `c`) at the same
card and PDF, 100000 × 8:

| build, map | mean (pb) | sd | rms quoted | seed χ²/dof |
|---|---|---|---|---|
| base, log (auto) | 185426 | 139 | 153 | 0.83 |
| change, log (auto) | 185426 | 139 | 153 | 0.83 |
| change, breit-wigner (named) | 185428 | 112 | 88 | 1.62 |

The first two rows agree seed by seed to every printed digit. Seed 1 reads
185419.859163 ± 154.047614 on both, the same as G-V1's `B-nn23-d2-big`.

## The MadEvent comparison

| MadEvent seed | σ (pb, banner `Integrated weight`) | quoted |
|---|---|---|
| 11 | 185779.563 | 124.7 |
| 12 | 185769.349 | 132.1 |
| 13 | 185758.998 | 145.3 |
| 14 | 185728.648 | 125.7 |
| 15 | 185748.931 | 116.3 |

- **MadEvent's own spread.** Mean 185757.1, sd 19.6, error of the mean 8.8,
  seed χ²/dof **0.024**. Its quoted errors are about 6.6× its seed spread. The
  five runs shared one process directory, so their independence is not
  established (Found 1).
- **This side against MadEvent.** The difference is −59 ± 25 pb with
  MadEvent's seed spread (pull −2.4), and −59 ± 61 pb with its quoted error
  over five seeds (pull −1.0).
- **The small-width bound.**
  - Write each side's σ at width `W` as `P/W + K`, with `K` independent of the
    width.
  - From this side's two widths, `K = (fk_W σ(fk_W) − Γ σ(Γ))/(fk_W − Γ)
    = (3.5 × 53118.6 − 185698)/2.5 = 86.8 ± 15.9 pb`.
  - MadEvent's `fk_W/Γ` multiplies the part of `K` drawn in its `Vz` channel.
    So MadEvent sits above this side by between 0 and `2.5 K = 217 ± 40` pb,
    that is 0 to 1.2e-3.
  - The measured +59 pb (3.2e-4) is inside that range. It is consistent with
    roughly a quarter of `K` being in the `Vz` channel. The rest is the `Z`
    exchange, which MadEvent's `Z` channel carries uncorrected.
- **The lineshape.** `G-V4/lhe_hz.awk` reads the `h Z` invariant mass of each
  event:

  | sample | events | share within 0.5 MeV | share within 52.5 MeV |
  |---|---|---|---|
  | this side, `vibegraph generate` on seed 1's grid | 2000 | 0.4890 (predicted 0.5000) | 0.9945 (0.9939) |
  | MadEvent seed 11 | 10000 | 0.1764 (predicted 0.1771) | 0.9775 (0.9788) |

  The predictions are `(2/π) atan(δ/(W/2))` at `W` = 1 and 3.5 MeV.
- **The replay through `generate`.** The same generation checks that the map
  replays: 2000 events at efficiency 0.247, with the sample's σ at
  183006 ± 4107 pb against the integration's 185814 ± 134 (pull −0.68).

## What the change leaves unmoved

The base (a60b53a0) and change (d81c342f) binaries, run with `--neval 20000
--niter 4 --fixed-budget --seed 3` on `dy13_default_run_card.dat` unless
another card is named:

```
DY-ppee-base s3 σ = 933.134009 ± 6.306949 pb             tau log (auto)
DY-ppee-bw1  s3 σ = 933.134009 ± 6.306949 pb             tau log (auto)
DY-ppjj-base s3 σ = 533585933.019976 ± 2120206.611938 pb tau inverse-square (auto)
DY-ppjj-bw1  s3 σ = 533585933.019976 ± 2120206.611938 pb tau inverse-square (auto)
ZC-base      s3 σ = 626.651313 ± 2.844435 pb             tau log (auto)   (p p > z, z > e+ e-, G-V1's zc card)
ZC-bw1       s3 σ = 626.651313 ± 2.844435 pb             tau log (auto)
```

**Census of the banked rows.** The census covers the hadronic processes in
`validation/`'s scripts and in the gated tests: the `p p > l+ l-` and
`e+ e-` variants, `p p > l+ l- j(j)`, `p p > j j`, `p p > b b~ (QCD=2)`,
`p p > t t~ (j)`, `p p > e+ e- j j $ h`, `g u > e+ e- u`, `u b~ > e+ ve` and
the rest.

- None carries a whole-state pole narrower than the `Z` or `W`, which sit at
  2.9e-3 or more of the unit interval even at `m_ll > 1 GeV`.
- The `h` appears as a whole-state line only in fixed-beam or amplitude-only
  rows (`b b~ > ta+ ta-`, `g g > h`, `b b~ > h`), where `τ` is not drawn.
- So no banked row's sampling order moves, and the σ gates below ran every
  banked row on its old map.
- The gate logs carry no `tracing` output, so the census is read off the
  process list and the rule, not off the gate runs.

## Mutation evidence

`G-V4/mutate.sh` applies each mutation to the source, runs the tests, and
restores the file. `git diff --stat` after the script shows only the
committed change.

- **Unmutated.** The six rule tests pass. The mixture test reads
  `Z Lorentzian: 3.10239086 ± 4.5e-3 against 3.10332677` and
  `tau^-1/2: 518.30695 ± 1.4 against 518.00000`.
  - That first run failed on a precision bound written before the binomial
    error of the pole half was worked out. The 1.5e-3 relative error is that
    split's `1/√n`.
  - The committed test bounds the two at 3e-3 and 5e-3, and it passes:
    `cargo test -p vibegraph-lib --lib -- maps:: tau diagram_channel` →
    `test result: ok. 62 passed; 0 failed`.
- **M1, the mixture density without its pole component:**
  `tau^-1/2: 1051.733 ± 1.16 against the exact 518`. FAILED.
- **M2, the pole segment drawn through the log map:**
  `tau^-1/2: 797.485 ± 1.74 against the exact 518`. FAILED.
- **M3, the rule without its narrow branch:**
  `a_narrow_whole_state_resonance_takes_the_breit_wigner_mixture` and
  `the_rule_reads_the_width_of_a_real_resonance` FAILED at their `TauMap`
  assertions. The other four rule tests pass.
- **The CLI refusal before the change.** The base binary on the test's
  artifact:
  ```
  thread 'main' (16427) panicked at vibegraph-lib/src/unweight.rs:602:9:
  no channel produced a usable weight: nothing can be generated
  exit 101
  ```
  The test asserts exit 1 and the named message, so it fails there.

What each oracle can and cannot see:

- **`the_breit_wigner_tau_mixture_integrates_known_functions`** runs on
  `p p > e+ e-` with the mixture named, the `Z` and `γ` channels at ½ each.
  - It checks the edges (`u₀ = 0, 1 → τ_min, τ_max`) and that no draw leaves
    the range.
  - It checks that two integrals come out unbiased, each one carried by one
    component.
  - It cannot see a wrong component weight that leaves the density consistent
    with the draw. Exact asserts pin the weights (0.5 and 0.5) instead.
- **The `p p > h z` sweep against MadEvent** is the end-to-end oracle. It
  cannot see a mixture error smaller than the small-width offset's 1.2e-3
  bound.

## Gate

At faa34b9. Cargo ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 --jobs 2`, one step after another (`G-V4/gates.sh`,
detached), with logs `G-V4-gate-<step>.log`:

```
$ cargo fmt --all --check                                                  → exit 0
$ cargo clippy --jobs 2 --workspace --all-targets -- -D warnings           → exit 0
$ cargo clippy --jobs 2 --workspace --all-targets --features vibegraph/extended-validation -- -D warnings → exit 0
$ cargo test --jobs 2 --workspace --no-fail-fast -- --test-threads 2       → exit 0
    50 test binaries, "test result: ok" on each; 1811 passed, 0 failed, 25 ignored
$ cargo test --jobs 2 -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_sigma -- --nocapture --test-threads 2                  → exit 0
    test result: ok. 8 passed; 0 failed; 33 ignored; 0 measured; 0 filtered out; finished in 860.04s
$ cargo test --jobs 2 -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_hadronic -- --nocapture --test-threads 2               → exit 0
    test result: ok. 20 passed; 0 failed; 26 ignored; 0 measured; 0 filtered out; finished in 657.95s
$ cargo test --jobs 2 -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_unweighting -- --nocapture --test-threads 2            → exit 0
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.45s
$ cargo test --jobs 2 -p vibegraph --profile release-debug --features extended-validation \
    --test cli_generate_proton -- --nocapture --test-threads 1             → exit 0
    test result: ok. 9 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 42.88s
$ python3 scripts/kb.py lint                                               → 0 errors, 5 warnings (after this report)
```

- **Panics in the passing logs.** The two `panicked at` lines in them belong to
  `should_panic` tests: `a_row_the_bundle_carries_may_not_be_absent` and
  `lepton_part_refuses_an_unknown_directory_name`, both `... ok`.
- **The first gate attempts.** Two attempts failed before this run:
  - In one, every cargo step exited at once on a stray environment variable,
    `CARGO_PROFILE_RELEASE_DEBUG_DEBUG`, which cargo parsed as
    `profile.release.debug_debug`.
  - The other failed at clippy on 4b20d60 with `type_complexity` in the new
    test. faa34b9 names the tuple type.
- **What the gate-time lint read.** The lint inside the gate run read one
  error: the stale sessions index, caused by this report's uncommitted draft.
  The committed index is regenerated.
- **What was not run.** This session did not run the full banked layer
  (`validate.sh`).

## Corrections to the brief

- **"Expect the ~1.5 % small-width offset" holds for a decay chain, not for
  `p p > h z`.** The 1.5 % is the `bwcutoff` window's share of each
  Breit–Wigner. Without a window, MadEvent's Jacobian factor cancels its floor
  over the peak, and the offset is bounded by 1.2e-3 (measured 3.2e-4).
- **"Draw τ with a Breit–Wigner map … as MadEvent does."** MadEvent maps each
  channel separately and weights channels by `|M_j|²`. Here the channels share
  one outer map under a density-based combiner. The map is the channel average
  of the per-channel densities. It reproduces MadEvent's map wherever one
  channel dominates, and covers the continuum otherwise.
- **"For a hadronic channel … outside a decay chain."** The mixture applies to
  any whole-state pole, in a decay chain or not. On card 048's chain the window
  already resolves the peak, so `auto` keeps the log map there and the chain is
  unchanged. With the mixture named, the chain improves: error 153 → 88 pb.
- **"State whether the change moves sampling order on any banked row."** It
  moves none (the census and bit-identity checks above).
- **"p without c."** This is correct and needed: with `c`, the gridpack model
  refuses the `u u~`/`c c~` grouping (G-V1 Found 4).

## Found

1. **MadEvent's quoted σ errors on `p p > h z` are about 6.6× its five-seed
   spread.** Seed χ²/dof is 0.024: quoted ~129 pb, sd 19.6 pb. The five runs
   shared one process directory. Either the seeds are not independent (a
   survey reusing the grids left in `G*/`) or the quoted error is conservative.
   A comparison at this precision depends on which. To close: rerun five seeds
   in five fresh `output` directories and compare the spreads.
2. **The knowledge bundle's MadEvent map notes call the small-width floor
   variance-only.** The "Width floors" section of
   `references/codebases/madevent-phase-space-maps.md` says "Both are
   unbiased; only the variance differs". The floor does move σ: through the
   window share (1.4 % on card 048) and the off-peak Jacobian (up to 1.2e-3 on
   `p p > h z`). To close: point that section at
   `process/madevent-small-width-treatment.md`.
3. **The phase-space concepts do not yet describe the Breit–Wigner `τ`
   mixture.** `phase-space/map-choices.md` and
   `phase-space/hadronic-tau-y-sampling.md` list two `τ` maps. To close: add the
   third map and the narrowness rule, with this report's measurements.
4. **Narrow whole-state poles at a share around `10⁻⁵` have no measurement.**
   An `h` (`Γ/M ≈ 5e-5`) spanning a hadronic final state, as in a process with
   initial `b` quarks, switches to the mixture under the rule with no row
   behind it either way. To close: a hadronic row with a physically relevant
   narrow `h` s-channel (`b b~ > h > τ τ` from `p p` with a five-flavour `p`),
   measured under both maps.
5. **Where MadEvent applies `fk_W/Γ` across its channels is inferred, not
   measured directly.** The bound argument assumes the factor applies only to
   points drawn in a channel that Breit–Wigner-maps the floored line (G-V1
   Found 3). The measured +3.2e-4, inside 0 to 1.2e-3, is consistent with that
   but does not isolate it. To close: compare MadEvent's per-channel
   `results.dat` for `G1` and `G2` at two widths against this side's channel
   split.
