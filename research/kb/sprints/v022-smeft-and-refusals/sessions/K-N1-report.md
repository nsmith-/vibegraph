---
type: Session Report
title: "K-N1 report: the fixed-beam sampled-point scale fault and misset's lepton floor, both named refusals"
description: "The fixed-beam scale panic becomes the proton path's SampledPointFault refusal at every integrate and generate site, and a negative misset is refused by name where its square is the maximum of setcuts.f's lepton smin term, with the term's arithmetic pinned to the smin MadEvent printed for three banked runs."
status: draft
tags: [refusal, cuts, hadronic, sampled-point, setcuts, validation]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-19}
---
Branch `sprintk-named-refusals` off main f78fb60b ([brief](K-N1.md)). The two
items were never files on main (`fixed-beam-sampled-point-scale-error-panics`,
`misset-negative-sqrt-shat-floor-unapplied`), so there is nothing to delete; the
brief's "delete the closed items' files" does not apply.

## What changed

**Part A** (`vibegraph-lib/src/hadronic.rs`, `generate.rs`,
`vibegraph-cli/src/integrate.rs`). `FixedBeamIntegrand` carries the same
`OnceLock<SampledPointFault>` as `ProtonIntegrand`, with `fault()` and
`record_fault`; the type is reused, not copied. `apply_scale` returns `bool`:
on a scale error it records `SampledPointFault::Scale(e)`, leaves the
subprocesses at their previous coupling, and its two callers give the point no
weight (`matrix_element_at` returns `0.0`, `select_event` returns `None`).
`refuse_sampled_fault` in `generate.rs` and `integrate.rs` now takes
`Option<&SampledPointFault>`, so the proton and fixed-beam paths share one
function per crate. It is called after `adapt_grids_budget` in `integrate`, and
in `generate` after the scan and after the event loop on the fixed-beam path
(the proton path already had both).

Other panics reachable from a sampled point on the fixed-beam path, read in
`hadronic.rs` outside `mod tests` (none converted; none is a fallible per-point
result):
- `hadronic.rs:574` `channel_input`: a channel group the prescription has no
  forest for. A construction invariant, shared with the proton path, documented
  `# Panics`.
- `hadronic.rs:705` `unreachable!` on a spin code that `compile` already refused.
- `hadronic.rs:2661` `unreachable!` for a factorisation-floor veto on beams with no
  density; `ScaleChoice::from_run_card` reads `lpp != 0`, so it cannot be built.
- `hadronic.rs:2683` `unreachable!` for a per-event prescription on a decay;
  `RunCard::for_decay` fixes a decay's scales.
- `hadronic.rs:3006` `expect` on the survey trajectory's last entry: setup, not a
  point.

**Part B** (`vibegraph-lib/src/cuts.rs`). `lepton_floor_term` reproduces
`setcuts.f:632-675` without the `misset²` argument, and `refuse_misset_floor`
refuses a negative `misset` where `misset²` exceeds it. The decision and its
evidence are below; this is a refusal, not an implementation.

### Part B findings

1. **What vibegraph's floor is.** `shat_min_hint` returns the largest bound the
   cuts imply (`dsqrt_shat²`, a same-flavour pair's `mmll²`, `(Σ pT)²`,
   `(Σ m)²`), a floor that never exceeds the cuts' own support. It has no
   lepton-class `max(...)` to add an argument to, and it takes the transverse
   sum over all classes at once where MadGraph sums class squares. So the
   one-term change the brief hoped for does not exist: matching MadGraph means
   its whole class-by-class `smin`.
2. **When `misset²` is the maximum.** `smin += max(smin_p², smin_m, mmnl²,
   ptllmin², misset²)` runs only if a charged lepton is among the legs the loop
   at `:632` visits, which is every final-state leg but the last. It is the
   maximum when all of the others sit below `misset²` (1 GeV² at `-1`): every
   lepton pT floor, `el` and rank minimum summing to under 1 GeV, no `mmll`
   pair among the counted leptons, `mmnl` and `ptllmin` under 1 GeV. The default
   `ptl = 10` is far above it.
3. **The brief's premise is wrong for pair mass.** The loop counts the first
   lepton of an `e+ e-` pair alone, so `nb_l = 1` and no `mmll²` pair is formed:
   `mmll` does not lift the term for a two-lepton process, only for three or
   more counted leptons. The K-C1 comment ("every default lepton-pT or pair-mass
   cut already empties it") is true of `ptl` and false of `mmll` on a lepton
   pair.
4. **Whether it can change σ.** MadEvent applies `smin` as the lower edge of the
   `τ` draw (`myamp.f:569-570`), so it removes whatever support the other cuts
   leave between the implied floor and `smin`. It can leave some:
   - Lepton-only process, no lepton cut: the term is the whole floor, 1 GeV², and
     there is support below it. vibegraph already refuses that card for want of
     any floor (command below), so the reference runs where this crate stops.
   - Another class's threshold saturated by a reachable configuration (two jets
     at `ptj`: `(2 ptj)²` is reached at 90° back to back, with soft leptons): the
     term adds up to 1 GeV² above it, and the interval has support of positive
     measure, vanishing as a power of `1/ŝ_threshold`.
   - Not binding: a final-state mass threshold or `dsqrt_shat²` above
     `S_other + misset²`, which I cannot decide without the other classes'
     `smin` terms (below).
   So the term can bind, and the effect on σ is real but of the order of a
   threshold sliver, below any σ gate here. That is the brief's "invisible in σ"
   case; the floor value is the finer oracle.
5. **Why refuse rather than implement.** An implementation applies `S_other +
   max(...)` as a floor, which needs the jet, b, photon and PDG blocks
   (`setcuts.f:527-631`, `:678-690`) reproduced as MadGraph computes them, with
   the `ht*min` and overcount quirks the crate already refuses elsewhere.
   Adding `misset²` to a bound that is not MadGraph's `smin` would make the floor
   a mixture of the two. The refusal names the term and the exits.
6. **What the refusal costs.** It fires on a lepton process with `misset < 0`
   and every lepton term under `misset²`, including a process where a large
   final-state mass would have hidden the term (finding 4, last bullet). No
   banked card has `misset < 0` (the four that name it write `0.0`), and the
   seven family cards of K-C1 are `p p > w+ j`, with no lepton leg.

## Oracles

All with `CARGO_TARGET_DIR=/home/user/wt/sprintk-named-refusals/target`,
`CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, debug info off.

**A1. Failing first.** The new test
`hadronic::tests::a_refused_scale_on_a_sampled_point_is_a_fault_and_not_a_panic`
builds a `g g > g g` fixed-beam integrand at `dynamical_scale_choice = 3`
(`αs`-dependent, a per-event scale), hands the sampled-point seam momenta at
rest (no transverse energy, so `ScaleError::DegenerateKinematics`), and asserts
`matrix_element_at == 0.0`, `select_event == None`, the first fault is kept and a
later one does not replace it; a physical point first is the control (weighted,
no fault). With `apply_scale`'s `Err` arm put back to the old `panic!`
(script `ab.sh` in the scratchpad), same test:

```
test hadronic::tests::a_refused_scale_on_a_sampled_point_is_a_fault_and_not_a_panic ... FAILED
thread '...' panicked at vibegraph-lib/src/hadronic.rs:2757:23:
per-event scale on a sampled point: scale choice 3 gives a non-positive scale 0 on these momenta
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1438 filtered out
```

and on the branch `cargo test -p vibegraph-lib --lib refused_scale_on_a_sampled`:
`1 passed`. A sampled point of a real run does not reach this error (the
RAMBO and channel maps give positive transverse energy), so the seam is driven
directly.

**A2. Bit for bit against main f78fb60b.** Two binaries, one from this branch's
HEAD, one with the four source files checked out from f78fb60b, each
`integrate PROC --run-card validation/madgraph/CARD --out OUT --fixed-budget
--neval 20000 --niter 3 --seed 7 -j 2 --no-tui --no-network -y`,
`LHAPDF_DATA_PATH=validation/pdf`; `sha256sum OUT/grid.bin.zst`:

| card, process | σ main | σ branch | grid sha256 (main = branch) |
|---|---|---|---|
| `config_share_ee_mumu`, `e+ e- > mu+ mu-` (fixed, no `αs`) | 419.72 ± 0.75 fb | 419.72 ± 0.75 fb | 06ebd25d0071715f… |
| `config_share_ee_uuxg`, `e+ e- > u u~ g` (fixed, `dynamical_scale_choice = -1`, `αs`) | 347.8 ± 2.1 fb | 347.8 ± 2.1 fb | ddef97a2e5b630c2… |
| `dy13_default`, `p p > e+ e-` (hadronic, default cuts) | 935 ± 11 pb | 935 ± 11 pb | 063e6ff624eba7cd… |

`ee_uuxg` is the card that runs `apply_scale`'s changed path on every point; the
other two do not reach it. The grids differ from K-C1's recorded hashes only
because my proc cards are written by hand (`generate e+ e- > mu+ mu-` and the
like), not the campaign's.

**B1. The lepton term against MadEvent's own `smin`.** The banked runs print
`Define smin to` from `setcuts.f:706`; each is the lepton term (every other
argument of the final `max` is smaller). Test
`cuts::tests::the_lepton_floor_term_is_the_smin_madevent_prints` asserts
`lepton_floor_term`:

| run (`validation/madgraph/output/...`) | log | printed | `lepton_floor_term` |
|---|---|---|---|
| `pp_to_ll_qcd0`, `p p > e+ e-`, `ptl = 10` | `SubProcesses/P1_qq_ll/G1/run_01_log.txt` | 100 | 100 |
| `bbx_to_tapta_ckm_no_b_mass`, massive `ta+ ta-` | `.../P1_qq_taptam/G1/run_01_log.txt` | 100 | 100 |
| `ee_to_mumu_tata_qcd0_control_20260801`, four leptons | `.../P1_ll_lltaptam/G3/run_control_20260801_log.txt` | 900 | 900 |

The 900 and the 100 are what the loop's one-short range predicts (`3 × 10` and
`1 × 10`, squared), not `4 × 10` and `2 × 10`: the check that
`nexternal-1` is real. The test then pins the branches no banked run takes
(`el`, the rank minima by lepton count, `mmll` pairs minus masses, `mmnl`,
a negative `ptllmin`) against hand arithmetic of the Fortran.

**B2. The refusal.** Test
`cuts::tests::a_negative_misset_is_refused_where_its_square_is_the_lepton_floor`:
refused at `ptl = 0`, `0.9`, and `-2 = misset, 1.5 = ptl`; compiled at the default
`ptl`, at the tie `1 = ptl`, with `mmnl`, `ptl1min` or `ptllmin` above 1 GeV, with
`mmll = 50` on four leptons, at `misset = 0`, on a process with no lepton, and
with the lepton on the last leg only. The two existing sentinel tests now skip a
set the refusal names at `misset = -100`.

From the CLI, `--check` on `dy13_default_run_card.dat` with `-1 = misset`,
`0 = ptl`, `50 = mmll` appended (the later lines win), `p p > e+ e-`:

```
main:    check ok: p p > e+ e-
branch:  ERROR failed to decompose into flavour groups: cut compilation failed: run
         card's misset = -1 is refused: setcuts.f:675 squares it into the lepton term
         of MadEvent's lower limit on ŝ, and on this process the lepton cuts leave that
         term at 0 GeV², below misset² = 1 GeV², so the limit is raised above what the
         cuts imply and can remove phase space they allow, which is not reproduced; set
         misset = 0, or raise a lepton cut so that its term exceeds misset²
```

(`mmll = 50` does not help on a pair: finding 3.) The same card with only
`0 = ptl` and `misset = 0` is refused for want of any floor
(`subprocess u u~ > e+ e- has no lower bound on ŝ`), with the branch binary.

**B3. Bit for bit where the term is not the maximum.** The three runs of A2 are
that oracle: `mumu` and `dy13_default` have `ptl = 10` and no `misset` line
(default `0`); `uuxg` has no lepton. Identical σ and grid hashes.

## What each test provably cannot detect

- A1 drives the seam with a degenerate point of one scale choice. It cannot see a
  fault raised by the clustering branch (`MissingChannels` and the like) or the
  `select_event` path through a real `generate`; both call the same
  `apply_scale`. The refusal sites in `generate.rs` and `integrate.rs` have no
  CLI test: nothing a real run draws reaches the error, so none can be built
  without a hook.
- A2 is one seed, one budget, three cards: equal `grid.bin.zst` hashes show the
  sampled sequence and weights are untouched for those inputs, not that no other
  input moves. It cannot see a card whose point never meets a scale error, which
  is every card.
- B1 shares its arithmetic with the code under test and covers the lepton
  block alone: each banked run has only leptons, so it cannot tell whether
  MadGraph's other classes add as the report assumes, and it pins the `ptl`
  branch of `nb_l = 1, 3` only. The `el`, `ptl<n>min`, `mmll`, `mmnl` and
  `ptllmin` branches are transcription of the Fortran, not a MadGraph
  measurement.
- Nothing here ran MadGraph at `misset = -1`: the claim that `misset²` raises the
  printed `smin` there is read from `setcuts.f:675`, with the loop range
  confirmed on the `ptl` runs. No cross-section difference was measured; the
  sliver is argued, not seen.
- B2 cannot detect a process where the term is the maximum but a final-state
  mass hides it (it refuses that too, finding 6), nor `vibegraph run-card`,
  which reports `refusal: null` for `misset = -1` because that report reads the
  card alone and this refusal needs the process.

## Gates

- `cargo fmt --all --check`: clean (`FMT_OK`).
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished`, rc 0
  (`kn1-clippy.log`).
- `cargo test --workspace`: 69 test binaries, 2233 passed, 0 failed, rc 0
  (`kn1-test.log`). `cargo test -p vibegraph-lib --lib cuts::`: 116 passed.
- `pixi run --skip-deps kb-lint`: 1297 concepts, 433 backlog items; 0 errors,
  5 warnings, all on backlog items this branch does not touch.
- `pixi run --skip-deps validate` (the banked layer) was **not run**: it is a
  heavy job and the brief reserves those to the sprint session. The A2/B3
  comparisons above stand in for it on three cards; the change touches `cuts.rs`
  only at `misset < 0`, which no banked card sets.

## Found

- **Measure `smin` at `misset = -1` in MadGraph.** Run `generate_events` (or the
  `setcuts` path alone) on `p p > e+ e- j` with `ptj = 20`, `ptl = 0`,
  `mmll = 0`, `misset = -1` and `0`, read `Define smin to`: 401 against 400
  would confirm the term, and a `p p > j j e+ e-` pair would show the threshold
  sliver in σ. It also settles finding 4's last bullet. A MadGraph run: not done
  here.
- **Reproduce MadEvent's `smin` class by class**, if cards with lepton cuts off and
  `misset < 0` on a heavy final state matter: it would let the refusal narrow to
  the cases where the floor actually rises, and cover the `ht*min` and jet-block
  terms `shat_min_hint` does not read.
- **`vibegraph run-card` cannot report a process-dependent refusal.** `misset = -1`
  shows `refusal: null`; the same holds for the other refusals made in
  `Cuts::compile` from the legs. Worth a note in the report's schema.
- **A sampled-point refusal at the end of `generate` leaves the finished event
  file.** `emit_to` renames its partial file to `--out` before
  `refuse_sampled_fault` runs, on the fixed-beam path as on the proton one, so
  a refused run exits with an error and a complete-looking LHE on disk. Closing
  it means removing the output on that refusal, or refusing before the rename.
