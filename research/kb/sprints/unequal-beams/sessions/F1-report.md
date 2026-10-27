---
type: Session Report
title: "F1 report: proton runs with unequal beam energies"
description: "ProtonIntegrand takes both beam energies; √s = 2√(E1E2); build_frames puts partons at x_i·E_i and boosts by MadGraph's cm_rap; equal beams are bit-identical by construction and pinned; three new tests, six mutations caught."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-21}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
Worktree `/home/user/wt/S`, branch `sprintd-unequal-beams`. Scope items 1–5
of the brief are done; item 6 held: `vegas.rs`, `budget.rs` and the
phase-space maps are untouched and still read √s as a plain `f64`
(`process_shape`, `maps.channel`, `multiplicity_maps`), now computed as
`ProtonBeams::sqrt_s()`. `collider-energy-sum-of-beam-energies` is not
deleted: it closes with the banked unequal-beam row (V1) or by the user's call.

## What changed

- **`ProtonBeams`** (`vibegraph-lib/src/proton.rs`): a `Copy` pair of lab
  energies, beam 1 along `+z`, with `new`, `from_run_card`, `equal(√s)`,
  `energies()` and `sqrt_s() = 2√(E₁E₂)`. It replaces the `sqrt_s_had: f64`
  argument of all five `ProtonIntegrand` constructors (`new`,
  `new_with_maps`, `new_unmerged_with_maps`, `new_unbounded`, `build`). The
  integrand keeps `s_had = √s·√s` exactly as before and stores the beams in
  place of `sqrt_s_had`, whose only other reader was `build_frames`.
- **`integrate.rs` / `generate.rs`**: `ProtonBeams::from_run_card(rc)` at the
  two sites that summed the energies; `sqrt_s_had = beams.sqrt_s()` still
  feeds `multiplicity_maps` and the artifact's recorded `sqrt_s`.
- **`build_frames`**: partons at `x₁E₁` and `x₂E₂`; outgoing legs boosted by
  `β = (x₁ − r·x₂)/(x₁ + r·x₂)` with `r = E₂/E₁`. The `(τ, y)` draw is unchanged.
- **Callers in tests**: the hermetic and extended-validation tests that
  passed `SQRT_S_HAD` (13000) pass a `BEAMS` const of 6500 × 6500 (or
  `ProtonBeams::equal`); those that summed `rc.ebeam1 + rc.ebeam2`, and
  `cli_integrate`'s rebuild (which used `artifact.sqrt_s_had`), use
  `ProtonBeams::from_run_card(&rc)`, as `generate` does.
- **Docs**: `runcard.rs` `BeamMode` (both variants; the `Proton` doc also said
  `√ŝ = x₁x₂ s`, now `ŝ = x₁x₂ s`), the pT-maximum ceiling doc in `cuts.rs`,
  and the module doc of `cli_fixed_energy.rs`. The ceiling's code,
  `0.5·(ebeam1 + ebeam2)`, is unchanged: it is ≥ √(E₁E₂) and ≥ √ŝ/2 for
  fixed beams of any mass, so it stays sound (see Found 3). Other sites that
  mention `ebeam1 + ebeam2` describe MadGraph's own rule (`seed_record.rs`,
  `mmjj > 0.01·(ebeam1+ebeam2)`) or an equal-energy run
  (`validate_vegas.rs`, 91.2 GeV), and are correct as written.

A `cargo fmt` pass re-wrapped the constructor calls whose argument got
shorter (`BEAMS` for `SQRT_S_HAD`); that is most of the line count in
`proton.rs` and `validate_hadronic.rs`.

## MadGraph (b7687064), read at the pinned commit

Fetched from `raw.githubusercontent.com/mg5amcnlo/mg5amcnlo/b7687064/…`:

- `Template/LO/SubProcesses/genps.f:345`:
  `cm_rap=.5d0*dlog(xbk(1)*ebeam(1)/(xbk(2)*ebeam(2)))`, and `:348`
  `s(-nbranch) = xbk(1)*xbk(2)*stot`. `tanh(cm_rap) = (x₁E₁ − x₂E₂)/(x₁E₁ + x₂E₂)`,
  which is the `β` above divided through by `E₁`.
- `genps.f:656-676`: `stot = m1² + m2² + 2(E₁E₂ − p₁ᶻp₂ᶻ)` with
  `m1 = m2 = 0.938` for `|lpp| = 1, 2` (`:662-663`). vibegraph keeps the
  massless `4E₁E₂` (the sprint's decision: the massive form moves every
  equal-beam √s by an ulp). The difference is `O(m_p²/s)`, about `2e-8`
  relative at 13 TeV.
- `Template/LO/SubProcesses/unwgt.f:633-717`: `write_leshouche` boosts the
  event to the lab by `get_betaz` (`:145-170`) from the incoming parton's CM
  momentum over `x` to the beam's lab momentum. In the massless limit that
  `get_betaz` branch gives `(x₂E₂ − x₁E₁)/(x₂E₂ + x₁E₁)`, the same velocity
  with MadGraph's `zboost_with_beta` sign convention. MadGraph builds that
  beam momentum with mass `m1 = 0.938`, so its written momenta differ from
  vibegraph's massless boost at `O(m_p²/E²)`, at equal beams as well.

## Equal beams are bit-identical: the argument

1. **√s.** Old: `E + E` (exact, `= 2E`). New: `2·√(fl(E·E))`. In binary
   floating point with round-to-nearest, `√(fl(x·x)) = |x|` whenever `x²`
   neither overflows nor underflows. This is a known property of correctly
   rounded `*` and `sqrt` in radix 2 (it fails in radix 10); see the
   *Handbook of Floating-Point Arithmetic* (Muller et al.). This report does
   not prove it: the sweep test below pins it at the energies that matter.
   Doubling is exact. So `sqrt_s()` returns `E₁ + E₂` to the bit, and `s_had = √s·√s`,
   `τ_min` and every map input are unchanged. `ProtonBeams::equal(√s)`
   round-trips because halving and doubling are exact.
2. **Lab frame.** Old: `e_beam = (E₁+E₂)/2 = E` (exact), partons at
   `xᵢ·e_beam`, `β = (x₁ − x₂)/(x₁ + x₂)`. New: partons at `xᵢ·Eᵢ = xᵢ·E`, the
   same product. `r = E₂/E₁ = 1` exactly, so `r·x₂ = x₂` exactly and `β` is
   the same expression evaluated on the same operands. This holds without
   branching on equality. The plain form `(x₁E₁ − x₂E₂)/(x₁E₁ + x₂E₂)` is
   *not* bit-identical: mutation M3 below shows it moving a boosted leg by 8
   ulps at trial 2.
3. **The CM frame**, the `(τ, y)` draw, `channel_cuts(…, s_had, …)` and the
   maps read only `s_had` / √s, which are unchanged by (1).

Pinned by `equal_beams_collide_at_the_energy_sum_to_the_bit`, which compares
`to_bits` at 6500, 4000, 500, 250, 100, 86.5, 50 and 45.6 GeV (every beam energy
in the banked cards and the hermetic tests) and at 100 000 swept energies over
`[1e-2, 1e5]` GeV. It also checks the `equal(√s)` round trip. The frame is
pinned by `equal_beams_build_the_energy_sum_lab_frame_to_the_bit`: 4000 production
points across every LLJ channel, comparing every component of every lab leg
to the old construction by `to_bits`. 3900 or more of those points must have
`x₁ ≠ x₂` and so carry a real boost. Every existing seeded proton test in
the hermetic suite passes unchanged.

## New unit test at unequal energies

`unequal_beams_place_and_boost_the_partons_as_madgraph_does` (`proton.rs`),
at 6500 × 4000 GeV on the LLJ card (`etal = 2.5`):

- `√s = 2√(2.6e7) = 10198.03902718557` (not 10500), and `s_had = 1.04e8`,
  each to `1e-15`.
- A hand-built partonic configuration has a lepton at `pT = 100`, `η = 2.4`, a
  lepton at `η = −1`, and the jet balancing them. It is placed at
  `x₁ = x₂ = √ŝ/√s`. Partons are `[x·6500, 0, 0, x·6500]` and
  `[x·4000, 0, 0, −x·4000]` exactly. The partonic system moves at `β = 5/21`.
  Every outgoing leg keeps its `px, py` and shifts in rapidity by
  `½ ln(13/8) = 0.24276`. The forward lepton matches the hand value
  `100·(cosh, 1, 0, sinh)(2.4 + ½ ln(13/8))` to `1e-11`. Four-momentum
  balances in the lab.
- Cut decision: that lepton lands at lab rapidity 2.643 and the configuration
  **fails** `etal`. The same configuration between equal beams of
  √(6500·4000) GeV each (same √s, so `x` and `ŝ` are unchanged) stays at rest
  and **passes**.
- At an asymmetric point `x₁ = 0.3`, `x₂ = 0.05`, every leg shifts by
  `cm_rap = ½ ln(x₁E₁/(x₂E₂))` to `1e-12`.

## Mutation evidence

Driver `beamsS-F1-scripts/mutate.py` (scratchpad) applies each mutation to
`proton.rs` and runs
`cargo test --workspace --lib -- proton::tests::equal_beams proton::tests::unequal_beams`,
then restores and touches the file (`restored True` after each sweep):

| mutation | equal √s bits | equal frame bits | unequal hand values |
|---|---|---|---|
| M1: old frames (`e_beam = √s/2`, `β = (x₁−x₂)/(x₁+x₂)`) | ok | ok | **FAILED** `left: [580.9046961112284, …] right: [740.51109525221, …]` (parton 1) |
| M2: `sqrt_s = E₁ + E₂` | ok | ok | **FAILED** at the √s assertion |
| M3: `β = (x₁E₁ − x₂E₂)/(x₁E₁ + x₂E₂)` | ok | **FAILED** `trial 2 leg 2`, `…541498` vs `…541506` | ok |
| M4: `sqrt_s` = MadGraph's massive `√stot` | **FAILED** `beams of 6.5e3 GeV`, `…007359` vs `…007360` | ok | **FAILED** (√s) |
| M5: parton energies swapped between beams | ok | ok | **FAILED** `left: [455.69913553982155, …]` |
| M6: partons right, `β = (x₁−x₂)/(x₁+x₂)` | ok | ok | **FAILED** at the rapidity-shift assertion |

Each new test fails under at least one mutation. M3 shows why `β` is written
with `r`, and M4 shows why √s is the massless form.

What these tests cannot see: they test our own formula against hand values
derived from MadGraph's source. They do not compare against a MadGraph run.
An `O(m_p²/E²)` disagreement with MadGraph's massive `stot` and `get_betaz`
is below every tolerance here. That is V1's banked unequal-beam row.

## Gate

Run on the committed tree (`ffc8d65`) with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ cargo fmt --all --check
fmt exit=0
$ cargo test --workspace
test exit=0
40 `test result` lines; summed: passed 1552, failed 0, ignored 16
  (vibegraph lib unit tests: 1018 passed; 0 failed; 10 ignored)
  test proton::tests::equal_beams_collide_at_the_energy_sum_to_the_bit ... ok
  test proton::tests::equal_beams_build_the_energy_sum_lab_frame_to_the_bit ... ok
  test proton::tests::unequal_beams_place_and_boost_the_partons_as_madgraph_does ... ok
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 48.60s
clippy exit=0
$ python3 scripts/kb.py lint      (after `python3 scripts/kb.py index` for this report)
834 concepts, 290 backlog items; 0 errors, 4 warnings
```

The four lint warnings were there before this session: each is `state: blocked`
without `blocked_by`, on an item this branch does not touch. No
extended-validation run: this worktree has no PDF sets, no
`validation/madgraph/output` and no `research/refs/mg5amcnlo` content. The
equal-beam bit-identity argument above is why the banked seeded rows cannot
move. Confirming that in CI is V1's gate.

## Commits

Author `Nicholas Smith <nick.smith@cern.ch>`, one `Assisted-by` line each, not pushed:

- `ffc8d65` feat(proton): unequal proton beams collide at 2√(E1E2) and boost by cm_rap
- the commit carrying this report (and the regenerated sessions index).

## Found

1. **A proton run card with a non-positive or non-finite beam energy is
   not refused.** `RunCard::parse` (`runcard.rs` ~:431-443) checks `lpp` but
   not `ebeam1/2`. With `ebeam1 = 0`, `√s = 2√(0·E₂) = 0`, so `τ_min = ∞`. A
   negative value gives NaN. Before this change `E₁ + E₂` gave a finite but
   meaningless √s. MadGraph clamps a beam energy below the beam mass up to the
   mass (`genps.f:670-671`). To close: refuse `ebeam ≤ 0` (or below the proton
   mass on `lpp = 1`) at parse, with a test.
2. **vibegraph's proton √s and lab boost are massless; MadGraph's are not.**
   MadGraph uses `stot` with 0.938 GeV beams and `get_betaz` from a massive
   beam (`unwgt.f:645-656`), so the two differ by `O(m_p²/E²)` (about `2e-8` at
   6500 GeV) in √s, `ŝ` and the written lab momenta, at equal beams too. This
   is by decision and far below every tolerance. A per-event LHEF comparison
   at `1e-9` would see it. To close: record it in the beams/kinematics concept
   so a future tight per-event oracle knows the expected residual.
3. **The pT-maximum no-op ceiling is conservative at unequal proton beams.**
   `detect_unimplemented` (`cuts.rs` ~:1063) treats a `ptj<n>max` / `ptl<n>max`
   / `missetmax` bound at or above `(E₁+E₂)/2` as default. At unequal proton
   beams the true ceiling is `√(E₁E₂)`, which is smaller, so a bound in
   `[√(E₁E₂), (E₁+E₂)/2)` excludes nothing but is still refused as
   unimplemented. It is sound, not wrong. To close: a beam-mode-aware ceiling
   (`√(E₁E₂)` on `lpp = 1`; for massive fixed beams `√ŝ/2`, which can exceed
   `√(E₁E₂)`), if such a card ever matters.
4. **`generate` does not compare the artifact's banked `sqrt_s_had` with the
   √s it rebuilds.** `card_mismatches` (`generate.rs` ~:271) compares the
   whole run card, so the beam energies are checked. But an unequal-beam
   artifact banked before this change recorded `E₁ + E₂`. Its grids were
   trained on the old frames, and it would replay against the new
   kinematics without complaint. Unequal-beam artifacts were never valid, so
   this matters only to a stale local artifact. To close: refuse when
   `artifact.sqrt_s_had` differs from `ProtonBeams::from_run_card(rc).sqrt_s()`.
