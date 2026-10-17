---
type: Session Report
title: "V2 report: an adaptation-free estimate puts both q q̄ → e⁺e⁻ g rows on MadEvent's equal-weight value"
description: "Two fixed densities, never adapted, on this crate's integrand: u u~ > e+ e- g reads 0.556431 ± 0.000091 pb and d d~ > e+ e- g 0.620387 ± 0.000112 pb (20 seeds each). Against MadEvent's reported mean they pull +3.01 and +2.67; against its equal-weight mean +0.45 and −0.19; against this side's gate −0.74 and −0.28. D1's condition is met. The flag lives only in the banked records, so the outcome is a re-record request."
status: draft
tags: [validation, sigma, madevent-reference, iteration-combination, fixed-density, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: code, resource: "https://github.com/nsmith-/vibegraph/commit/d620185", title: "The committed probe: probe_fixed_density_llj_annihilation"}
  - {id: d1, resource: "../decisions/D1-fixed-beam-seeds-and-ew.md", title: "D1, point 2"}
---
Scope: [V2](V2.md), the design [D1](../decisions/D1-fixed-beam-seeds-and-ew.md)
point 2. One code commit, `d620185`, touches only
`vibegraph-lib/tests/validate_sigma.rs`: an `#[ignore]`d probe and the seam it
needs. No gate, reader, tolerance, record or reference changed. The logs are in
the session scratchpad as `sigmalean-V2-*` (directory
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad`).

## The estimator

`probe_fixed_density_llj_annihilation` (`validate_sigma.rs`). Every point is
drawn from a density fixed before the first point, and each seed's value is the
plain mean of `f/p`, with its sample error. There is no iteration, no grid
refinement and no weighting of one part of the sample against another.

**Two densities**, so a defect in either map shows up as a disagreement between
them:

- **Analytic** (`AnnihilationDensity`, written for this probe; none of it is
  this crate's phase-space code). It factorises `dΦ₃ = dΦ₂(s; Q, g) dQ²/2π
  dΦ₂(Q; e⁺e⁻)`:
  - `Q²` is drawn from a 0.5 / 0.25 / 0.25 mixture of a Z Breit–Wigner (91.188,
    2.441404), `1/Q²` and `1/(s − Q²)`;
  - the gluon's cos θ is drawn from a 0.3 / 0.7 mixture of flat and
    `1/(1 + ε − cos²θ)`, with `ε = (2 ptj/√s)²`; φ is flat;
  - the e⁺e⁻ pair is isotropic in the Q rest frame, then boosted.

  Support: `Q² ∈ [7.893, 230000] GeV²`.
  - The upper end follows from `E_g ≥ pT_g > ptj`, so `s − Q² ≥ 2√s·ptj`.
  - The lower end is half the cut-implied floor. `ptl = 10` and `drll = 0.4`
    give `m(ℓℓ)² ≥ 2 ptl² (ΔR²/2 − ΔR⁴/24) = 15.8 GeV²`.
  - Both ends are also asserted on flat-RAMBO points that pass the cuts.
- **Frozen multichannel.** This is the production per-diagram mixture, with its
  `α` adapted once on the gate seed's survey (`MULTICHANNEL_SURVEY × ITERS`) and
  then held. It is evaluated through `FixedBeamIntegrand::value` at plain uniform
  points, with no VEGAS grid. The density is chosen once from a frozen
  adaptation, which the brief allows. It shares the production maps with the
  gate, but not VEGAS or its iteration combination.

**The integrand is this crate's.** That means `ScaleAwareAmplitude::eval_m2` at
`αs(μR)`, where μR is the point's own clustered scale. That scale comes from
`event_scales_at` with a trailing uniform, so the configuration is drawn as
the integrand draws it. On top of that come the run card's compiled cuts, the
`1/(2s)` flux and the spin-colour average. The integrand is assembled by hand,
so before any estimate it is checked against the crate's own integrand.
`event_in_channel` on 200 000 flat-RAMBO points, divided by the massless
volume `s/(256π³)`:

```text
check: 183411/200000 flat points pass the cuts; the hand-assembled integrand matches
event_in_channel to 4.44e-16 worst relative; their m(ll)² spans [187.77, 229970.7] GeV²
```

Both rows print the same line. The cut verdict is asserted equal on every point,
and the outgoing legs are asserted to be `(-11, 11, 21)`.

**What it cannot see.** It cannot see anything the gate's integrand shares with
it: the matrix element, the cut filter and the scale prescription. Those are
gated elsewhere: the bit-exact amplitude net, and the `samples` rows' per-event
SCALUP/AQCDUP KS. So it tests the integration on both sides and nothing else. It
also inherits any MadEvent/vibegraph difference in the integrand itself, which
those gates bound.

`ddx_to_epemg` runs the same seeds and the same density draws as
`uux_to_epemg`. The two rows' per-seed values are therefore correlated:
measured correlation +0.90 for the analytic density, +0.77 for the frozen one.
They are not independent confirmations of the estimator, only of the
flavour-dependent integrand.

## The run

```text
RAYON_NUM_THREADS=2 VG_FIXED_DENSITY_SEEDS=20 VG_FIXED_DENSITY_POINTS=3000000 \
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
  --test validate_sigma -- --ignored --nocapture --exact probe_fixed_density_llj_annihilation
```

The output is `sigmalean-V2-run20.log`:
`test result: ok. 1 passed … finished in 1056.35s`, with `real 18m50.843s`,
`user 31m41.811s`. That is 20 seeds × 3 000 000 points per density per row.
The run also re-integrates this side's five gate seeds through
`integrate_reported`. It reproduces F1's cells bit for bit:
`5.567234e-1 ± 3.852e-4` and `6.204896e-1 ± 3.458e-4`.

### Per seed (pb)

| seed | uux analytic | uux frozen MC | ddx analytic | ddx frozen MC |
|---|--:|--:|--:|--:|
| `0xf1de0001` | 0.556343 ± 0.000409 | 0.556913 ± 0.000331 | 0.620379 ± 0.000500 | 0.620850 ± 0.000297 |
| `0xf1de0002` | 0.556082 ± 0.000409 | 0.556937 ± 0.000331 | 0.619808 ± 0.000500 | 0.620581 ± 0.000297 |
| `0xf1de0003` | 0.556028 ± 0.000409 | 0.556663 ± 0.000330 | 0.619841 ± 0.000499 | 0.620686 ± 0.000296 |
| `0xf1de0004` | 0.557114 ± 0.000409 | 0.556756 ± 0.000331 | 0.621082 ± 0.000500 | 0.620711 ± 0.000297 |
| `0xf1de0005` | 0.556199 ± 0.000409 | 0.556587 ± 0.000333 | 0.620459 ± 0.000500 | 0.620393 ± 0.000298 |
| `0xf1de0006` | 0.556354 ± 0.000409 | 0.556829 ± 0.000330 | 0.620683 ± 0.000500 | 0.620780 ± 0.000296 |
| `0xf1de0007` | 0.556521 ± 0.000409 | 0.556962 ± 0.000331 | 0.620347 ± 0.000500 | 0.620898 ± 0.000297 |
| `0xf1de0008` | 0.555977 ± 0.000409 | 0.556877 ± 0.000331 | 0.619892 ± 0.000500 | 0.620756 ± 0.000297 |
| `0xf1de0009` | 0.556311 ± 0.000409 | 0.556572 ± 0.000331 | 0.620193 ± 0.000500 | 0.620503 ± 0.000297 |
| `0xf1de000a` | 0.556580 ± 0.000409 | 0.556534 ± 0.000331 | 0.620518 ± 0.000500 | 0.620363 ± 0.000297 |
| `0xf1de000b` | 0.555928 ± 0.000409 | 0.556505 ± 0.000331 | 0.619865 ± 0.000500 | 0.620789 ± 0.000297 |
| `0xf1de000c` | 0.556806 ± 0.000409 | 0.556148 ± 0.000330 | 0.620774 ± 0.000500 | 0.620471 ± 0.000296 |
| `0xf1de000d` | 0.556550 ± 0.000409 | 0.556208 ± 0.000330 | 0.620302 ± 0.000500 | 0.620266 ± 0.000295 |
| `0xf1de000e` | 0.556537 ± 0.000409 | 0.556297 ± 0.000330 | 0.620683 ± 0.000500 | 0.620092 ± 0.000296 |
| `0xf1de000f` | 0.556741 ± 0.000409 | 0.556277 ± 0.000331 | 0.620700 ± 0.000500 | 0.620188 ± 0.000297 |
| `0xf1de0010` | 0.556190 ± 0.000409 | 0.556277 ± 0.000330 | 0.620008 ± 0.000500 | 0.620485 ± 0.000296 |
| `0xf1de0011` | 0.556524 ± 0.000409 | 0.556283 ± 0.000330 | 0.620648 ± 0.000500 | 0.620373 ± 0.000297 |
| `0xf1de0012` | 0.556639 ± 0.000409 | 0.556324 ± 0.000330 | 0.620522 ± 0.000500 | 0.620402 ± 0.000296 |
| `0xf1de0013` | 0.556869 ± 0.000409 | 0.556546 ± 0.000331 | 0.620747 ± 0.000500 | 0.620400 ± 0.000297 |
| `0xf1de0014` | 0.556326 ± 0.000409 | 0.556183 ± 0.000330 | 0.620287 ± 0.000500 | 0.620422 ± 0.000296 |

- **Weights are bounded.** The largest weight over the mean is 13–15 for
  the analytic density on every seed of both rows, and 27–34 for the frozen
  mixture.
- **Fraction of points that pass the cuts:** 66.6 % for analytic (both rows),
  70.2 % (uux) and 76.6 % (ddx) for frozen.

### Seed-policy reads (`combine`: unweighted mean, `err = max(quoted, spread)`)

| row | density | σ (pb) | ± | rel err | seed χ²/dof (19 dof) |
|---|---|--:|--:|--:|--:|
| uux_to_epemg | analytic | 0.5564310 | 0.0000915 | 1.64e-4 | 0.60 |
| uux_to_epemg | frozen MC | 0.5565339 | 0.0000739 | 1.33e-4 | 0.69 |
| ddx_to_epemg | analytic | 0.6203869 | 0.0001118 | 1.80e-4 | 0.53 |
| ddx_to_epemg | frozen MC | 0.6205205 | 0.0000663 | 1.07e-4 | 0.59 |

The two densities agree on both rows:
`analytic − frozen multichannel: -1.029e-4 ± 1.176e-4 pb (-0.87σ)` (uux) and
`-1.335e-4 ± 1.300e-4 pb (-1.03σ)` (ddx). Every error is at most 0.018 %,
inside the brief's 0.05 %.

### Is the error honest? The χ²/dof sits below 1, so I calibrated it

All four 19-dof readings are below one. Against 19 dof, their one-sided
p-values are about 0.09, 0.17, 0.05 and 0.08. The ddx readings are not
independent of the uux ones (correlation 0.90 and 0.77 above). AGENTS.md reads
a χ²/dof far below 1 as loudly as one far above, so I ran the same probe on
`uux_to_epemg` at more seeds and fewer points:

| run | log | analytic χ²/dof | frozen χ²/dof | analytic σ | frozen σ |
|---|---|--:|--:|--:|--:|
| 200 seeds × 100 000 | `sigmalean-V2-calib200.log` | 0.82 (199 dof) | 0.80 (199 dof) | 0.5562554 ± 0.0001584 | 0.5563923 ± 0.0001279 |
| 2000 seeds × 10 000 | `sigmalean-V2-calib2000.log` | **1.02** (1999 dof) | **1.01** (1999 dof) | 0.5567213 ± 0.0001603 | 0.5564532 ± 0.0001285 |

Same command as above, with `VG_FIXED_DENSITY_ROWS=uux_to_epemg` and the seed
and point counts changed. Wall times were `real 4m59.971s` and
`real 3m54.199s`.

- **At 1999 dof** the quoted error is calibrated (σ of χ²/dof is 0.032).
- **At 199 dof** both densities read about 2σ low.
- **At 19 dof** the readings are within chance.

I have no mechanism for an error that over-quotes only at large N. The draws
are iid from independently keyed ChaCha8 streams, and the weights are
bounded, so I record the 200-seed reading as unexplained (see **Found**).

**The verdict below does not depend on it.** The estimate's error is 0.09e-3 to
0.11e-3 pb, against MadEvent's 0.45e-3 to 0.56e-3. Doubling the estimate's
error would move every pull below by under 9 %: the smallest R pull, ddx's
+2.67, would go to +2.47. All three runs also give the same
verdict, as the table below shows.

## Verdict per row

All pulls are `(est − ref)/√(err² + err_ref²)`, as printed by the probe.

- The references are the seed record's five budget seeds under `combine`.
- R is MadEvent reported, EW is MadEvent equal-weight.
- "this side" is the gate's five-seed cell (F1's recorded run, reproduced bit
  for bit above).

| row | estimate | R 0.555054 ± 0.000448 / 0.619132 ± 0.000457 | EW 0.556177 ± 0.000559 / 0.620488 ± 0.000534 | this side 0.556723 ± 0.000385 / 0.620490 ± 0.000346 |
|---|---|--:|--:|--:|
| uux_to_epemg | analytic, 20 × 3M | **+3.01** (rel +2.48e-3) | **+0.45** (+4.57e-4) | −0.74 (−5.25e-4) |
| uux_to_epemg | frozen MC, 20 × 3M | +3.26 (+2.67e-3) | +0.63 (+6.42e-4) | −0.48 (−3.41e-4) |
| uux_to_epemg | analytic, 200 × 100k | +2.53 | +0.14 | −1.12 |
| uux_to_epemg | frozen MC, 200 × 100k | +2.87 | +0.38 | −0.82 |
| uux_to_epemg | analytic, 2000 × 10k | +3.50 | +0.94 | −0.01 |
| uux_to_epemg | frozen MC, 2000 × 10k | +3.00 | +0.48 | −0.67 |
| ddx_to_epemg | analytic, 20 × 3M | **+2.67** (rel +2.03e-3) | **−0.19** (−1.64e-4) | −0.28 (−1.66e-4) |
| ddx_to_epemg | frozen MC, 20 × 3M | +3.01 (+2.24e-3) | +0.06 (+5.16e-5) | +0.09 (+4.97e-5) |

The 20 × 3M analytic run is the declared measurement. It was the first full
run, and the others were run afterwards to calibrate its χ²/dof.

**D1's condition is met on both rows, with both densities, at every run size.**
The estimate lands on EW (|pull| ≤ 0.94) and not on R (pull +2.5 to +3.5).
`ddx_to_epemg`, the control, behaves the same way. This side's gate lands on
the estimate too (|pull| ≤ 1.12).

Decision power is bounded by MadEvent's own errors, not by this estimate's.
Even an exact estimate at EW would pull only about +2.5 against R
(`(EW − R)/err_R` = 0.001123/0.000448). The paired EW − R in the record is
+0.001123 ± 0.000324 (uux) and +0.001356 ± 0.000384 (ddx). The estimate minus R
is +0.001377 ± 0.000457 (uux) and +0.001255 ± 0.000470 (ddx). So the whole
paired shift is MadEvent's iteration weighting, as on the 2 → 4 reproducer in
[madevent-iteration-weighting-bias](../../../validation/madevent-iteration-weighting-bias.md).
Here it reads 0.20–0.25 % low at `hard_survey = 2` and the 10 000-event budget.

## `validated_ew`: a record re-write request

The flag lives only in the banked records. The gate reads it through
`SeedRecord::estimator()` (`tests/common/seed_record.rs`), from
`validation/madgraph/output/seeded_sigma/<row>/seed_record.json`. No committed
override exists in the reader, so per the brief I propose none and edit no
banked data. Nothing is set.

**What the user needs to do (M3, the refdata work area).**

1. Re-write the three records from their kept files with the flag. No MadEvent
   run is needed: a seed whose kept files exist is read back, not re-run. In
   the bank driver this is:

   ```text
   ROWS="uux_to_epemg ddx_to_epemg ee_to_mumu_tata_qcd0" \
   VALIDATED_EW_ROWS="uux_to_epemg ddx_to_epemg ee_to_mumu_tata_qcd0" \
   pixi run -e madgraph bank-seeded-sigma
   ```

   If the template step would regenerate a process directory, `seed-record` can
   be run directly instead. These are the values each record already carries:

   ```text
   seed-record --row <row> --kept validation/madgraph/output/seeded_sigma/<row> \
     --budget 10000 --generator "<the record's own generator string>" \
     --tree pinned --mg-version 3.7.1 --hard-survey 2 --validated-ew
   ```
2. Assemble and pin a new bundle (`assemble_bundle.sh`, refdata-11, new
   `sha256` in `validation/manifest.toml`).

**I verified the direct form on copies.** I ran it on copies of the three kept
directories in the scratchpad (`sigmalean-V2-ewpatch/`), with `seed-record`
built from this branch. All three exit 0.

- Diffed against the banked records, the `seeds` arrays are identical and
  `validated_ew` goes `false → true`. Only `summary` otherwise differs, and no
  gate reads it.
- In `summary`, `screen` is now on the equal-weight estimator:
  - uux pulls `[-0.47, +0.42]`;
  - ddx `[-0.56, -0.26]`;
  - tata `[-0.11, +0.72]`;
  - all within the screen.
- The tool also writes `scatter` where the banked records have `heavy_tail`
  (see **Found**).

**What the gates would then read.** I computed this from F1's recorded cells
with the gate's formula; it is not run, because no banked record was changed:

| row | this side | EW reference | pull | rel |
|---|---|---|--:|--:|
| uux_to_epemg | 0.5567234 ± 0.0003852 | 0.5561766 ± 0.0005585 | +0.81 | +9.83e-4 |
| ddx_to_epemg | 0.6204896 ± 0.0003458 | 0.6204884 ± 0.0005345 | +0.00 | +1.86e-6 |
| ee_to_mumu_tata_qcd0 | 1.378080e-3 ± 1.632e-6 | 1.377512e-3 ± 1.804e-6 | +0.23 | +4.13e-4 |

**`ee_to_mumu_tata_qcd0` is in the request because D1 names it.** This
estimate does not cover it. It is a 2 → 4 with 28 channels, and its EW − R is
+3.19e-6 ± 1.44e-6 (2.2σ). Its record's scatter test has two channels above
χ²/dof 3: G21 at 4.22 and G23 at 3.03, look-elsewhere p 0.045, which the
current tool calls within chance. The user decided this by family. Whether to
let it ride on this 2 → 3 result is theirs to confirm (see **Found**).

## Gates on `d620185`

The gates ran in one detached script, `sigmalean-V2-gates.sh`, with the lean
cargo env vars. Per-gate logs are `sigmalean-V2-gate-<name>.log`, and the
status is in `sigmalean-V2-gates-status.log`.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --nocapture` | `test result: ok. 8 passed; 0 failed; 32 ignored … finished in 1177.54s`; `sigma gate: 34 process(es) asserted against banked MadGraph sigma` |
| `cargo test --workspace` | exit 0: 37 `test result: ok` lines, 1498 passed, 0 failed |

- **The `with_integrand` seam moves no gate number.** The `validate_sigma`
  run rewrote 42 cells under `target/validation-report/integrals/`. Each one's
  `sigma_vg_pb`, `sigma_vg_err_pb`, `sigma_mg_pb`, `sigma_mg_err_pb`, `pull`,
  `rel`, `chi2_dof`, `per_seed` and `status` are identical to F1's cells,
  copied before the run (`sigmalean-V2-F1-integrals/`): 0 differ.
- **Not run: `pixi run --skip-deps validate`.**
  - The change is one test file. The suite that file builds, and every lint
    `validate.sh` runs, are in the table above.
  - The other banked suites share none of the touched code, and F1 measured
    them at 2.5 h on this host.
  - No amplitude, eval, colour or coverage-list code changed.
- **The binary of the declared 20 × 3M run predates two lines of `d620185`.**
  They are the `(e+, e-, g)` leg-order assertion and the `model` field it
  reads, added while that run was going. The two calibration runs and the
  `validate_sigma` run above were built from `d620185` itself. Both calibration
  runs passed the assertion on `uux_to_epemg`, and the probe's code is otherwise
  the same.
- **CPU.**
  - The declared run took `user 31m41.811s`, and the two calibration runs
    `user 7m40.610s` and `user 5m8.518s`.
  - A 2 × 200 000-point pilot (`sigmalean-V2-pilot.log`) finished in
    24.11 s.
  - The estimator used about 45 CPU-minutes in all, inside the brief's 2 h.

## The brief

- **"Commit it as an `#[ignore]`d probe beside #22's."** PR #22 committed no
  probe. Its V1 report says "no probe test committed": its estimator was a
  scratch Fortran driver (`bwps5.f`) linked to MadGraph's standalone
  `matrix.o`. This is the first committed fixed-density estimator. It sits in
  `validate_sigma.rs` beside `probe_llj_parton_seed_stability`.
- **"aim for ≤ 0.05 %"**: met (0.011–0.018 %). The decision is limited by
  MadEvent's five-seed error (0.08–0.10 %), not by this estimate's. See the
  verdict section.
- **"D1's condition is met when it lands on EW (|pull| < 2) and not on the
  reported value."** I read "not on R" as |pull_R| ≥ 2. With these reference
  errors, a perfect estimate at EW reaches only about 2.5 against R. A rule
  asking for more would be unreachable without more MadEvent seeds.
- **Diagnostic MadEvent runs**: none were needed or made.

## Found

1. **`ee_to_mumu_tata_qcd0` would get `validated_ew` without an adaptation-free
   check of its own.**
   - Evidence: D1 point 2 names three rows, and this estimate covers the two
     2 → 3 rows. The tata record's scatter test has G21 (χ²/dof 4.22) and G23
     (3.03) above 3, and V1 found EW noisier than R on heavy-tailed rows.
   - To close: the user confirms D1's family inference for that row, or a
     fixed-density 2 → 4 estimate is built for it. That is costlier: 28
     channels, about #22's 6e8-point scale.
2. **Fixed-density χ²/dof below 1 at 200 × 100k on both densities.**
   - Evidence: 0.82 and 0.80 on 199 dof, about 2σ low. At 20 × 3M the readings
     are 0.53–0.69 (within chance individually), and at 2000 × 10k they are
     1.02 and 1.01. Weights are bounded (max/mean ≤ 34), and the streams are
     independent ChaCha8 keys.
   - The verdict is insensitive to it. It is open whether the quoted error
     over-states at large N.
   - To close: a 200 × 1M run of the probe (about 1 h CPU on 2 threads) and
     the same χ²/dof read. If it is still near 0.8, look for correlation
     between successive points.
3. **The banked seed records predate the current `seed-record` summary
   format.**
   - Evidence: refdata-10's records carry `summary.heavy_tail`, and today's
     tool writes `summary.scatter` (p-values, verdict) instead. This is from
     the dry run above.
   - No gate reads `summary`. But any re-record (this request, the pending
     20 + 20 ladder) changes it on every row it touches.
   - To close: the user re-records all seeded rows once with the current tool
     when refdata-11 is assembled, so the bundle carries one format.
4. **The 2 → 3 measurement belongs on the knowledge page.**
   - Evidence: [madevent-iteration-weighting-bias](../../../validation/madevent-iteration-weighting-bias.md)
     records only the 2 → 4 reproducer. This report measures the same
     mechanism on `u u~ / d d~ > e+ e- g`: reported 0.20–0.25 % low, EW on the
     estimate, at `hard_survey = 2` and 10 000 events.
   - To close: the close-out session adds this row to the page's table.
