---
type: Session Report
title: "G-V1 report: card 048's one nonzero point is a τ draw the whole-state decay window did not bound; μF = 0 is only the print"
description: "CMS card 048 (p p > vz, vz > h z at MVz = 3500, WVz = 1 MeV) integrated to one nonzero point per seed because the hadronic τ draw ran from the forced window's lower edge to τ = 1, so the ±52.5 MeV window was 2.2e-5 of the unit interval. Bounding τ from above by the window (artifact format 18) gives 184092 ± 63 pb at the gridpack's PDF against MadGraph's survey 1.811e5 ± 779, and 185426 ± 62 pb against a fresh MadEvent 1.828e5 ± 123 at nn23lo1; the residual 1.5% is MadGraph's small-width treatment, predicted at 1.0155 and measured at 1.0144 ± 0.0008. The per-event scale is √ŝ on both sides; μF = 0 is the summary line only."
status: draft
tags: [decay-chains, scales, non-sm-ufo, cms, phase-space, tau-map, small-width, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
measured:
  host: "cloud container, 4 vCPU shared with two sibling sessions, linux-64, 2026-11-04"
  commit: "base 3257564; fix 6561527 and f6fc78e. CLI binaries (dev profile, sha256 prefix): base cea11c35, probe (base + a per-point eprintln, never committed) 42a4c3ec, fix 83670d26 (the map and cuts change of 6561527 before its tests, the tau_max accessor and the artifact-format bump were added), final f6fc78e 1a9aef09"
  command: "see Method; every run's full CLI output under the session scratchpad, G-V1/runs/<tag>-s<seed>/output.txt"
sources:
  - {id: brief, resource: "G-V1.md", title: "G-V1 brief"}
  - {id: proton, resource: "../../../../../vibegraph-lib/src/proton.rs", title: "ProtonIntegrand::map_point_with, tau_max"}
  - {id: cuts, resource: "../../../../../vibegraph-lib/src/cuts.rs", title: "Cuts::shat_max, forced_shat_ceiling"}
  - {id: transpole, resource: "../../../../refs/mg5amcnlo/Template/LO/Source/transpole.f", title: "MadEvent transpole: the small-width Jacobian factor"}
---
## Verdict

- **Why one nonzero point.** The decay chain `vz > h z` forces the `Vz` line,
  which carries the whole final state, into `|√ŝ − MVz| < bwcutoff·Γ`, with Γ
  floored at `MVz·small_width_treatment` = 3.5 MeV: a ±52.5 MeV window. The
  window raised the `τ` floor (`forced_shat_floor`) but not the ceiling, so the
  logarithmic `τ` draw ran from the window's lower edge to `τ = 1`. The window is
  `4δ/M / ln(s/(M−δ)²)` = 2.21e-5 of the unit interval. On seed 1, one point of
  the 90000 drawn (survey, warm-ups and the gated iteration) landed in it, at
  `u₀` = 1.03e-5; on seed 3, 41 points did, the largest `u₀` 2.2092e-5, at the
  predicted edge. Every other point is outside the window and the cuts zero it.
  The draw is fixed, not refused: both `τ` maps now span `[τ_min, τ_max]` with
  `τ_max` the window's upper edge (`Cuts::shat_max`). The test fails before the
  change.
- **Why the defaults' seeds disagreed (run A).** The same draw, at a wider
  window (MVz = 2000, WVz = 10 GeV, ±150 GeV, 7.5% of the unit interval). At
  the gridpack budget (10000 × 3, fixed), over 25 seeds, χ²/dof was 1.54 with
  the quoted error at 1.56e-3. With the bound it is 0.95, the quoted error is
  7.7e-4, and the mean moves by 1.2e-4 (0.3σ of the difference). At 100000 × 8
  both agree: 0.110526 ± 3.9e-5 and 0.110560 ± 2.5e-5 over 5 seeds. The 3.6 the
  CERN host read over five seeds is the tail of a 1.5. The seeds were never
  biased; their errors were understated by 1.2×, not 1.9×.
- **μF = 0 is a print, not a scale.** On every nonzero point, μF = μR = √ŝ on
  both beams and both orderings. MadEvent's `SCALUP` at the same card is the
  `Vz` invariant mass to LHE precision (max |SCALUP − m_Vz| = 5e-4 GeV over
  10000 events), and so is this side's after the fix (5e-6). That is the
  `dynamical-scale-summary-prints-mu-f-zero` item (#90), nothing more.
- **"1 channels, 0 resonant pairs" is right.** `q q~ > vz > h z` is one
  s-channel diagram; `h` and `Z` are external, so no split has two
  Breit–Wigner daughters. The prints describe the process correctly.
- **MadGraph agrees with this side up to its own small-width treatment.**
  - At the gridpack's PDF (325500), 5 seeds at 100000 × 8: 184092 ± 63 pb,
    against MadGraph's survey line of 1.811e5 ± 779. Ratio 1.0165 ± 0.0044.
  - Fresh MadEvent (pinned 3.7.1) at the same card with `nn23lo1`: 182800 ±
    123 pb. This side at lhaid 247000: 185426 ± 62. Ratio 1.0144 ± 0.0008.
  - The prediction from MadEvent's small-width treatment (below) is 1.0155,
    1.5σ from the measurement.
  - The survey line is plausible, against the backlog item. It is 1.8e5 pb
    because the model has a free width: the partial width `Γ(Vz → hZ)` is fixed
    by the couplings, and Γ_tot = 1 MeV is far below it, so the effective
    branching ratio is far above one.

## The defect, and the fix

`ProtonIntegrand::map_point_with` drew `τ` over `[τ_min, 1]`. `τ_min` already
read the decay-chain windows. The new `forced_shat_ceiling` (cuts.rs) does the
same for the ceiling: for a set of forced lines that includes one with
`slots` = every final leg, the bound is that line's `(M + bwcutoff·Γ_window)²`.
A set without such a line, or a process without forced lines, leaves the
ceiling infinite. Because a point passes on some set, the bound is the largest
over the sets. `tau_max = min(1, shat_max/s)` enters both maps:

- the logarithmic map: `τ = τ_min^(1−u) τ_max^u`, with `(dτ/du)/τ = ln(τ_max/τ_min)`;
- the inverse-square map: `k = 1/τ_min − 1/τ_max`.

At `τ_max = 1` both reduce to the previous expressions bit for bit
(`1.0.powf(u) = 1`, `1/1.0 = 1`). So the change moves no process without a
forced line that carries the whole final state.

No banked gate has such a process. A grep of the tests and the references
finds only `pp_z_chain` (`p p > z, z > e+ e-`) in
`onshell_veto_reference.json`, and no test reads that row. It is checked below
as an extra, informational oracle.

The artifact format goes from 17 to 18 and the oldest readable version with
it. A version-17 grid of such a process has the same keys, dimensions and
channel-map digest. It would replay against a `τ` map it was not trained on,
which `channel_map_digest` cannot see (artifact.rs's own rule).

Tests:

- `cuts::tests::a_whole_state_forced_line_bounds_shat_from_above` covers the Z
  ceiling and the floored narrow width (3500 GeV, 1 MeV → 3.5 MeV). It also
  checks that a partial line, a set without a whole-state line, and no forced
  lines each give an infinite ceiling.
- `proton::tests::a_whole_state_window_bounds_both_tau_maps` uses
  `p p > z, z > e+ e-`. For both maps, `τ_min` and `τ_max` sit at the window's
  edges, `u = 0, 1` map to them, and no draw leaves the window. Both maps
  integrate `∫ τ^(−1/2) dτ/τ` over the narrowed range to within 5σ.
  **It fails before the change.** With `tau_max` forced back to 1 it panics at
  its `τ_max` assertion (`proton.rs:3502`, log
  `G-V1-test-failsbefore.log`, below).

## Method

Inputs come from `origin/sprintg-gridpack-cards` (7b9e8f80), extracted with
`git archive` into the scratchpad and never committed: the proc card, the run
card with `etaonium = 0.6` (the `runpatch`), the gridpack param card and the
staged `ufomodel/` as `Vector_Triplet_free_Width_UFO`. The CLI runs as `gpdb`
runs it:

```
vibegraph integrate proc_card.dat --run-card run_card.dat --neval N --niter K \
  --fixed-budget --seed S -j 1 --no-tui --ufo-dir models [--param-card param_card.dat]
```

`scratchpad/G-V1/runvg.sh` wraps this. B adds `--param-card`; A runs without
one. The "-nn23" variants set `lhaid = 247000` (NNPDF23_lo_as_0130_qed, the set
`nn23lo1` is fitted to). Seed statistics: `stats.py`, which gives the unweighted
mean, the sample sd, the error of the mean as sd/√n, and χ²/dof about the mean
from the quoted errors.

MadGraph is a copy of the pinned submodule (3.7.1) in the scratchpad, with the
gridpack UFO copied into `models/`:

- `generate p p > vz, (vz > h z)` with `define p = g u c d s u~ c~ d~ s~`;
- `output` gives `P1_qq_vz_vz_hz` and `P1_ccx_vz_vz_hz`, the gridpack's two
  directories;
- `generate_events` uses the gridpack's run card with `pdlabel = nn23lo1` (no
  LHAPDF in the container), `gridpack = False`, `nevents = 10000`,
  `iseed = 11`, `use_syst = False`, 2 cores.

A standalone `u u~ > vz, (vz > h z)` gives |M|² at fixed points.

### Reproduction (base 3257564)

The CERN host's B numbers are reproduced to every printed digit, and A's too:

```
B-325500-base s1 exit 0 σ = 11416.288547 ± 11416.288547 pb
B-325500-base s2 exit 0 σ = 7680.468681 ± 7680.468681 pb
B-325500-base s3 exit 0 σ = 28408.717306 ± 24263.767796 pb
B-325500-base s4 exit 0 σ = 17606.616089 ± 17606.616089 pb
B-325500-base s5 exit 0 σ = 0.000000 ± 0.000000 pb
A-325500-base s1 exit 0 σ = 0.109968 ± 0.001517 pb
A-325500-base s2 exit 0 σ = 0.112632 ± 0.001563 pb
A-325500-base s3 exit 0 σ = 0.114011 ± 0.001571 pb
A-325500-base s4 exit 0 σ = 0.111748 ± 0.001538 pb
A-325500-base s5 exit 0 σ = 0.106551 ± 0.001481 pb
```

### The per-event record (probe binary, base + one `eprintln` per nonzero point)

Each line is every nonzero point a run drew, over the α survey (6 × 10000),
both warm-ups and the gated iteration:

| seed | nonzero points / 90000 | `u₀` | √ŝ − MVz (GeV) | μF = μR (GeV) | value |
|---|---|---|---|---|---|
| 1 | 1 | 1.028e-5 | −0.00368 | 3499.99632 (= √ŝ) | 2.93e-1 |
| 2 | 1 | 6.44e-6 | −0.02192 | 3499.97808 (= √ŝ) | 1.97e-1 |
| 3 | 41 | max 2.2092e-5 | −0.05128 … +0.05245 | √ŝ on all 41 | 3.4e-3 … 4.5e-1 |
| 4 | 1 | 4.43e-6 | −0.03147 | 3499.96853 (= √ŝ) | 4.52e-1 |
| 5 | 0 | — | — | — | — |

- Every nonzero point is inside the ±52.5 MeV window (the floored width's,
  15 × 3.5 MeV), and no point outside it carries weight.
- The predicted window in `u₀` is [0, 2.21e-5]. Seed 3's largest hit, 2.2092e-5,
  is at its edge.
- Seed 3's hits grew 1 → 6 → 41 across the survey, warm-ups and iteration 3 as
  VEGAS narrowed onto its first hit.
- Seed 1's one hit landed in the gated iteration: σ = value × (1/N) ×
  conversion = 2.93e-1/10⁴ GeV⁻² = 11416 pb.
- `x = (0.917, 0.0723)` on seed 1: a valence-quark, high-x configuration.

### Hypotheses tested

1. **"The window is a sliver of an unbounded `τ` draw."** This predicts the
   largest `u₀` of any hit (2.21e-5, met) and about 2 hits per 90000 uniform
   points (seeds 1, 2, 4: one each; seed 5: none). It also predicts that bounding
   the draw turns every point into a window point and gives a σ with a sane
   error, which would be false if anything else zeroed the window's points:
   ```
   B-325500-d2 s1 exit 0 σ = 184925.271497 ± 1922.140892 pb
   B-325500-d2 s2 exit 0 σ = 183638.379655 ± 1977.039014 pb
   ```
   It holds. Over 25 seeds at the gridpack budget: mean 184125 ± 388, sd 1939,
   rms quoted 1963, χ²/dof 0.98. At 100000 × 8, over 5 seeds: 184092 ± 63,
   χ²/dof 0.91. The two rungs agree (0.08σ).
2. **"A near-delta Breit–Wigner explains the enormous weights"** (the brief's
   framing). Partly. The Breit–Wigner is narrow (Γ = 1 MeV in the propagator),
   but within the window it is resolved once the draw is bounded. The
   enormous weights come from the draw's measure: one hit in 10⁴ points is
   weighted as 10⁴ × its share. At 100000 × 8, 5 seeds give χ²/dof 0.91 and a
   relative error of 8e-4 per seed without any Breit–Wigner `τ` map. A
   Breit–Wigner `tan` map, which is MadEvent's, would cut the error further;
   it is not needed for correctness.
3. **"This side evaluates the `Vz` propagator with the raw 1 MeV width where
   MadEvent uses `fk_W = max(Γ, M·1e-6)` = 3.5 MeV, so σ would be 3.5× off."**
   Falsified as a σ defect, and measured as a 1.5% convention difference:
   - MadEvent's `matrix1.f` does pass `FK_MDL_WVZ` (the standalone output passes
     `MDL_WVZ`, so the standalone amplitude oracle cannot see the floor). Its
     pole |M|² is 2.8838e12 against 3.5327e13 raw, a ratio of 12.25 = 3.5².
   - But `transpole.f:44` multiplies the phase-space Jacobian by
     `width_fk/width` for the floored width. MadEvent's σ is therefore the
     narrow-width σ at the *true* width, integrated over a Breit–Wigner of the
     *fake* width.
   - This side, at the raw width, gives 185426 ± 62 pb (nn23). With the width
     set to 3.5 MeV in the param card it gives 52427 ± 127, which is ×1/3.537,
     the narrow-width 1/Γ. MadEvent's 182800 sits with the raw-width number, not
     the floored one.
   - The residual is the window's share of each Breit–Wigner. The window is
     ±15 × 3.5 MeV on both sides; it holds `(2/π)atan(2δ/Γ)` of the line,
     0.99394 at Γ = 1 MeV and 0.97879 at 3.5 MeV. So the ratio is 1.01548,
     against 1.0144 ± 0.0008 measured at nn23 and 1.0165 ± 0.0044 against the
     gridpack's survey.
   - The event record shows the two lineshapes, over 10000 unweighted events
     each:

     | side | mean \|m_Vz − MVz\| (MeV) | share within 0.5 MeV |
     |---|---|---|
     | this side | 1.48 (predicted 1.49 for Γ = 1 MeV) | 0.506 (predicted 0.503) |
     | MadEvent | 3.86 (predicted 3.87 for 3.5 MeV) | 0.182 (predicted 0.177) |

   Nothing is changed on this side. MadEvent's choice is a convention that
   broadens its lineshape, and this side's is the physical one. It is in
   **Found**.
4. **"μF = 0 is a scale defect."** Falsified by the per-event record: μF = √ŝ
   on every point, matching MadEvent's `SCALUP`.

### Run A against MadEvent

MadEvent at the model's defaults (its `param_card_default.dat`) with
`nn23lo1` gives `Cross-section : 0.09338 +- 7.185e-05 pb`.

This side at lhaid 247000, over 5 seeds at 100000 × 8, gives 0.0934566 ±
2.1e-5 (χ²/dof 0.69). The pull is 1.0σ. The run without a param card and the
run with MadEvent's default card give bit-identical seeds, so the model's
defaults are read the same on both sides.

### Informational: `p p > z, z > e+ e-` (the one whole-state chain with a reference)

`dy13_default_run_card.dat` with `nevents = 20000` and `cut_decays = True` is
`onshell_veto_reference.json`'s `pp_z_chain` row. Five MadEvent seeds give
631.114 (sd 0.53, rms quoted 0.79, error of the mean about 0.35). At
`--target-rel 1e-3`, 25 seeds per build:

| build | mean | sd | rms quoted | χ²/dof | pull against MadEvent |
|---|---|---|---|---|---|
| base | 631.449 ± 0.134 | 0.669 | 0.573 | 1.40 | 0.9σ |
| fix | 631.487 ± 0.142 | 0.712 | 0.565 | 1.63 | 1.0σ |

- The two builds agree (0.04 ± 0.20).
- The bound shortens the run: 1.08M evaluations against 1.44M on seed 1.
- Both builds understate their per-seed error by about 1.2× (χ²/dof 1.4 to 1.6
  over 24 degrees of freedom), so that predates this change.
- The first five seeds of the fix read χ²/dof 0.17. Twenty more gave 1.63,
  which is AGENTS.md's point about few-seed scatter.

### What the change leaves unmoved

The algebra says `τ_max = 1` reproduces the old bits. As a check, the base and
fix binaries were run at one seed (`--neval 20000 --niter 4 --fixed-budget
--seed 3`, `dy13_default_run_card.dat`):

```
base [p p > e+ e-] σ = 933.134009 ± 6.306949 pb tau log
d2   [p p > e+ e-] σ = 933.134009 ± 6.306949 pb tau log
base [p p > j j]   σ = 533585933.019976 ± 2120206.611938 pb tau inverse-square
d2   [p p > j j]   σ = 533585933.019976 ± 2120206.611938 pb tau inverse-square
```

The two binaries agree to every printed digit on both maps.

## Gate

Run at f6fc78e. Cargo ran with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 --jobs 1`, and every gate with `--test-threads 2`.

```
$ cargo fmt --all --check                                              → exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                → exit 0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings → exit 0
$ cargo test --workspace --no-fail-fast -- --test-threads 2            → exit 0
    47 test binaries, "test result: ok" on each; 1806 passed, 0 failed, 25 ignored
$ cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_sigma -- --nocapture --test-threads 2              → exit 0
    test result: ok. 8 passed; 0 failed; 33 ignored; 0 measured; 0 filtered out; finished in 2500.67s
$ cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_hadronic -- --nocapture --test-threads 2           → exit 0
    test result: ok. 20 passed; 0 failed; 26 ignored; 0 measured; 0 filtered out; finished in 2649.72s
$ python3 scripts/kb.py lint                                           → 0 errors, 4 warnings
    (the 4 warnings are other backlog items' `state: blocked` without `blocked_by`)
```

- **The first workspace run failed, and the second passed.** It failed on
  `artifact::tests::a_version_14_artifact_is_refused` and
  `a_version_16_artifact_is_refused`, which pinned the readable range at 17.
  f6fc78e moves them to 18 and adds 17 to the refused versions; the second run
  is the one above.
- **The new map test fails before the change.** With `tau_max` forced to 1:
  ```
  test cuts::tests::a_whole_state_forced_line_bounds_shat_from_above ... ok
  test proton::tests::a_whole_state_window_bounds_both_tau_maps ... FAILED
  thread 'proton::tests::a_whole_state_window_bounds_both_tau_maps' panicked at vibegraph-lib/src/proton.rs:3502:9
  ```
- **The committed CLI reproduces the fix binary's numbers.** The binary built
  from f6fc78e (sha256 1a9aef09…) gives B seed 1 = 184925.271497 ±
  1922.140892 pb and A seed 1 = 0.111615 ± 0.000764 pb, identical to the fix
  binary's.

The two σ gates are the skill's rows for a phase-space change. No banked row
has a whole-state forced line (above), so both run the old `τ` maps bit for bit.
This session did not run the full `validate.sh` banked layer (every
extended-validation test in the workspace).

## Corrections to the brief

- **The 1.9× error understatement on A is five-seed noise.** Over 25 seeds the
  base build reads χ²/dof 1.54, an understatement of 1.24×. The cause is the
  same unbounded draw as B's (fixed: 0.95).
- **"Near-delta Breit–Wigner, no resonance mapping" is the wrong mechanism.**
  The points of enormous weight are window hits in an unbounded draw. Once the
  draw is bounded, the 1 MeV Breit–Wigner is resolved by VEGAS on the log map
  without a Breit–Wigner map (hypothesis 2).
- **The backlog item calls MadGraph's survey line implausible.** It is
  plausible, and right to 1.5%: the free-width model puts Γ_tot = 1 MeV under
  `Γ(Vz → hZ)`. The item's own number, 0.09296 pb from one nonzero point
  (survey pass 2, e3f543f), was not reproduced here. That run's inputs are not
  on the gridpack-cards branch. At this card a one-hit estimator gives
  anything from 0 to 28409 pb.
- **"0 resonant pairs" is not a symptom.** It counts splits with two
  Breit–Wigner daughters, and this process has none.

## Found

1. **`generate` panics on a grid whose rescan finds no weight.** On the base
   build, `vibegraph generate grid.bin.zst --seed 7 --nevents 2000` on run B
   seed 1's artifact (σ = 11416 pb from one point) panics at
   `unweight.rs:602` with `no channel produced a usable weight: nothing can be
   generated`. The command exits 101 with a backtrace. To close: a named
   refusal in place of the `assert!`, with a CLI test on a grid whose scan
   finds no nonzero point. G-F4's integrate-side refusal of few-point σ makes
   such an artifact rarer but not impossible (`--force`, older artifacts).
2. **A narrow whole-state resonance without a decay chain has no Breit–Wigner
   `τ` map.** `generate p p > h z` (gridpack UFO, B card, `define p = g u d s
   u~ d~ s~`, lhaid 247000, 100000 × 8) carries the 1 MeV `Vz` in the
   s-channel with no forced window, so the bound added here does not apply.
   Five seeds give 224658 ± 52054, 147640 ± 10532, 179939 ± 24211,
   158106 ± 18045 and 168051 ± 19118 pb, with in-run χ²/dof 51.6 on seed 1.
   MadEvent maps `τ` with a Breit–Wigner `tan` map whenever a finite-width
   resonance spans the whole final state (`myamp.f:579`, `transpole`). To
   close: a Breit–Wigner `TauMap` choice for a narrow whole-state resonance,
   or a refusal naming the resonance's Γ/M, gated over ≥5 seeds on this card
   against MadEvent.
3. **MadEvent's small-width treatment is a registered-deviation candidate.**
   For Γ < 1e-6·M, MadEvent evaluates every propagator at `fk_W = M·1e-6`
   (matrix*.f) and multiplies the Jacobian of each Breit–Wigner-*mapped*
   propagator by `fk_W/Γ` (`transpole.f:44-47`). That has three consequences:
   - Its σ is the narrow-width σ at the true width, while its events carry the
     fake width's lineshape (3.86 MeV mean |m − M| against this side's 1.48 on
     card 048).
   - Its σ differs from this side's by the window's share of each
     Breit–Wigner: 1.5% here, predicted and measured.
   - The correction rides on the channel's mapped propagators, so in a
     multichannel process it presumably applies only to points drawn in a
     channel that maps that propagator. That is untested here.

   The standalone output passes the raw `MDL_W*`, so the amplitude oracle and
   any standalone-|M|² comparison are blind to the floor. To close: decide
   whether to register it (`validation/madgraph-defects` or a convention
   note) or to emulate it, with the multichannel case measured on a two-diagram
   narrow-resonance process.
4. **`c c~ > h z` refuses its flavour grouping in the gridpack model.** With
   `define p = g u c d s u~ c~ d~ s~`, `generate p p > h z` gives
   `subprocesses u u~ > h Z and c c~ > h Z share a matrix element, but their
   colour flows cannot be paired up`. `c c~ > h Z` enumerates 4 diagrams
   against `u u~ > h Z`'s 2. A named refusal, but a grouping that calls two
   subprocesses one matrix element while their diagram counts differ looks
   wrong. To close: check why the grouping pairs them and whether the refusal
   or the pairing is the bug.
   Card 048's chain syntax enumerates `c c~ > Vz` with one diagram and is
   unaffected, so this is not a defect of the card.
