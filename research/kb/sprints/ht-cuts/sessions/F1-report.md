---
type: Session Report
title: "F1 report: MadGraph's HT cuts compiled"
description: "cuts.f's htj and iht definitions with citations, the implementation, per-event tests with mutation evidence, local σ runs and the MadGraph banking request."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-22}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
## MadGraph's definitions (b7687064)

Read from raw.githubusercontent.com at
`b7687064b9a013317ca164aa1395bc9c0e39ae1e`:
`Template/LO/SubProcesses/cuts.f`, `setcuts.f`, `Template/LO/Cards/run_card.dat`
and `madgraph/various/banner.py`.

| question | answer | lines |
|---|---|---|
| which legs count for `htj*` | the jet-class legs, `is_a_j`: gluons and quarks with `\|pdg\| ≤ min(maxjetflavor, 6)`. At the default `maxjetflavor = 4` b quarks are out; at 5 they are in; at 3 charm is out too; at 6 a top counts | `cuts.f:521-529`, `:876-880`; class membership `setcuts.f:217-226` |
| what `iht*` sums | `htj` plus the b-class legs (`is_a_b`, `maxjetflavor < \|pdg\| ≤ 5`). Not every visible leg: leptons, photons and neutrinos are in neither sum. `iht = htj` at `maxjetflavor ≥ 5` | `cuts.f:530-533`, `:898-906` |
| η or pT preselection | none, and no `do_cuts` test: decay products under `cut_decays = F` and legs above 20 GeV count. The template says "ht cuts always apply" on decay products | `cuts.f:524-534`; `run_card.dat:102` |
| edges | fail when `htj < htjmin` or `htjmax ≥ 0 and htj > htjmax`; the same for `iht`. Both thresholds pass at equality | `cuts.f:892`, `:908-909` |
| no jets | `htjmin > 0` rejects (`njets < 1` check); the `htj` cuts sit inside `if (njets > 0)`; the `iht` cut is outside it | `cuts.f:817`, `:842`, `:908` |
| frame | transverse momenta, invariant under the beam-axis boost: lab and partonic frame agree | `cuts.f:528`, `:532` (`pt(p(0,i))`) |
| defaults (what disables each) | `htjmin = 0`, `htjmax = -1`, `ihtmin = 0`, `ihtmax = -1`; `remove_all_cut` resets to the same. A minimum ≤ 0 and a negative maximum never reject | `banner.py:4402-4405`, `:3858-3873` |
| parameter plumbing | `inclHtmin = ihtmin`, `inclHtmax = ihtmax`; `htjmin4(2:4)` are `ht2min..ht4min` | `setcuts.f:498-507` |
| MadEvent's ŝ floor | `smin` takes `htjmin²` among the jet terms | `setcuts.f:571` |

Not compiled, and still refused: the partial sums `ht2min..ht4max` over the 2, 3
or 4 hardest jets (`cuts.f:878-890`), and the `ktdurham` block
(`cuts.f:542-625`), which refills `njets` before the HT code runs.

The answers are in [MadGraph cut conventions](../../../run-card/madgraph-cut-conventions.md),
section "HT".

## What changed (commit d4731f0)

- `vibegraph-lib/src/cuts.rs`
  - `:58-64` module doc: the HT convention.
  - `:336-350` `HtCut`: jet-class and b-class leg indices, four thresholds.
  - `:406` the four names leave `UNIMPLEMENTED_CUTS`; `ht2min..ht4max` stay.
  - `:569-591` `ht_cut`: `None` unless `htjmin > 0 || htjmax ≥ 0 || ihtmin > 0
    || ihtmax ≥ 0`, so the defaults compile no cut. Membership by `letter`
    alone, not `do_cuts`.
  - `:644-651` compile path; `shat_min` takes `ht_shat_floor`.
  - `:917` `admits_every_point` reads `ht`.
  - `:1032-1036`, `:1046-1056` `ht_ok`: `htj` over jets, `iht = htj + Σ b`,
    MadGraph's edges.
  - `:1065-1077` `ht_shat_floor`: `max(htjmin, ihtmin)²` from `√ŝ ≥ Σ p_T`,
    leaving out a minimum with no leg to sum.
  - tests `:2811-3071`, the `admits_every_point` list (`:1550-1553`), and
    `unbindable_value_does_not_excuse_other_cuts` now uses `ht2max`.
- `vibegraph-lib/src/runcard/classes.rs:370-373`: the four names move from
  `R_UNIMPL` to `R_CUT_LITERAL`.
- `research/kb/run-card/madgraph-cut-conventions.md`: the HT section, the
  implemented list, the `shat_min` list, two sources.
- `validation/madgraph/scripts/pp_to_jj_ht.mg5`, `pp_to_bb_iht.mg5` and two
  `status = "planned"` rows in `validation/manifest.toml` (the banking request
  below).

A card at the defaults compiles `ht: None`, the same `Cuts` value as before
(`ht_defaults_compile_no_cut` asserts equality against cards that spell the
defaults out), and `shat_min` is `max(x, 0)` of the old value, which is the old
value.

## Per-event tests

All in `cuts::tests`:

| test | pins |
|---|---|
| `htj_is_the_scalar_jet_sum_with_inclusive_edges` | scalar sum (two jets at right angles), `htjmin` / `htjmax` both sides and at equality |
| `iht_has_the_same_edges` | `ihtmin` / `ihtmax` edges |
| `b_quarks_count_for_iht_and_for_htj_only_as_jets` | `g g > g b b~`: b's in `iht` and not `htj` at `maxjetflavor = 4`, in both at 5 |
| `charm_leaves_htj_below_maxjetflavor_four` | charm in `htj` at 4, only in `iht` at 3 |
| `leptons_and_photons_are_in_neither_sum` | `iht` is not over every visible leg |
| `a_process_without_jets_fails_every_positive_minimum` | the `njets = 0` branch |
| `decay_products_count_for_ht` | a forced `W`'s jets under `cut_decays = F`: no `ptj`, still in HT |
| `a_heavy_jet_class_leg_counts_for_ht` | a top at `maxjetflavor = 6`: no `ptj`, still in HT |
| `ht_defaults_compile_no_cut` | the defaults and inactive values compile to the bare filter |
| `ht_minima_and_maxima_are_not_refused` | the four names compile; `ht2min..ht4max` still refused |
| `ht_minima_bound_shat` | `shat_min = max(htjmin, ihtmin)²`, reached by an accepted point; no bound without a leg to sum |
| `each_threshold_withdraws_the_claim_that_nothing_is_rejected` (extended) | each HT threshold makes `admits_every_point` false |

### Mutations

A local driver (not committed) applied each mutation to `cuts.rs`, ran
`cargo test -p vibegraph-lib --lib cuts::tests::`, and restored the file. Every mutation fails at least one test:

| mutation | result | failing tests |
|---|---|---|
| M1 `htj < htjmin` → `<=` | 59 passed; 6 failed | heavy_jet, no_jets, b_quarks, charm, ht_minima_bound_shat, htj_is_the_scalar |
| M2 `htj > htjmax` → `>=` | 62 passed; 3 failed | no_jets, b_quarks, htj_is_the_scalar |
| M3 `iht` omits b-class | 63 passed; 2 failed | b_quarks, charm |
| M4 `htj` also sums b-class | 63 passed; 2 failed | b_quarks, charm |
| M5 sums gated on `do_cuts` | 63 passed; 2 failed | heavy_jet, decay_products |
| M6 `iht` also sums leptons and photons | 62 passed; 3 failed | no_jets, ht_minima_bound_shat, leptons_and_photons |
| M7 no `shat_min` floor from HT | 64 passed; 1 failed | ht_minima_bound_shat |
| M8 `htj` check skipped without jets | 63 passed; 2 failed | no_jets, charm |
| M9 vector instead of scalar sum | 60 passed; 5 failed | heavy_jet, decay_products, ht_minima_bound_shat, htj_is_the_scalar, iht_has_the_same_edges |
| M10 HT active at default (`htjmin ≥ 0`) | 62 passed; 3 failed | admits_every_point (x2), ht_defaults_compile_no_cut |
| M11 `iht < ihtmin` → `<=` | 60 passed; 5 failed | no_jets, b_quarks, charm, iht_has_the_same_edges, leptons_and_photons |
| M12 `ihtmax` ignored | 63 passed; 2 failed | b_quarks, iht_has_the_same_edges |
| M13 `ihtmin` left out of the `shat_min` floor | 64 passed; 1 failed | ht_minima_bound_shat |

What these tests cannot see: anything MadGraph does that the citations above
misread. They encode the reading; the σ comparison against a MadGraph run is
what can contradict it.

## σ

No banked MadGraph reference carries an HT cut (the worktree has no
`validation/madgraph/output`, and no manifest row sets one), and MadGraph is not
installed here, so σ is measured on vibegraph's side and the MadGraph runs are
requested below.

### vibegraph, locally

`target/debug/vibegraph integrate <proc> --run-card <card> --fixed-budget
--neval 75000 --niter 10 --seed <s> --pdf-set NNPDF23_lo_as_0130_qed --pdf-dir
validation/pdf`, with `VIBEGRAPH_NO_NETWORK=1`, seeds 20261101-20261105 unless
stated. Every card: `ebeam1 = ebeam2 = 6500`, `pdlabel = lhapdf`,
`lhaid = 247000`, otherwise MadGraph defaults.

With two final-state partons of equal pT an HT window is a pT window, so each
HT card has a twin that reaches the same phase space through the single-leg
cuts the gated `pp_to_jj` and `pp_to_bb` rows already exercise. The twins differ
in sampling (a single-leg `pT` cut sets the peripheral channels' spacelike
floor; an HT cut does not) and must agree in σ.

| card | σ per seed (pb) | mean (pb) | χ²/dof |
|---|---|---|---|
| `p p > j j`, `htjmin 300`, `htjmax 800` | 212568.6, 213190.1, 213117.1, 212922.6, 213357.8 (each ±443-450) | 213031.2 ± 199.4 | 0.46 |
| `p p > j j`, `ptj 150`, `ptjmax 400` | 212749.3, 213562.7, 213480.9, 212950.1, 212984.5 (each ±261-264) | 213145.5 ± 117.4 | 1.84 |
| `p p > b b~`, `ihtmin 200`, `ihtmax 500` (mjf 4), 15 seeds | first five 4839.5, 4858.9, 4839.7, 4843.7, 4837.3 (each ±8.1-8.2) | 4845.80 ± 2.11 (spread) | 1.00 |
| `p p > b b~`, `ptb 100`, `ptbmax 250`, 15 seeds | first five 4847.5, 4850.8, 4856.3, 4843.2, 4852.9 (each ±5.4) | 4850.59 ± 1.01 (spread) | 0.53 |
| `p p > b b~` iht card at 300000 x 10, seeds 20261301-05 | 4841.29, 4845.50, 4852.51, 4853.67, 4861.85 | 4850.96 ± 3.55 (spread), 1.83 quoted | 3.76 |
| `p p > b b~` ptb card at 300000 x 10 | 4849.66, 4849.31, 4846.02, 4844.81, 4848.15 | 4847.59 ± 0.94 (spread) | 0.62 |

- jj: HT against pT twin −0.054 %, pull −0.49.
- bb at 75000 x 10, 15 seeds each: −0.099 %, pull −2.04 on the measured seed
  spreads. At four times the budget the sign flips: +0.070 %, pull +0.92 on
  the spreads. The HT arm's χ²/dof is 3.76 at 4x: its quoted errors
  understate its own scatter by about 2x, which is what the 1x pull was
  reading. The twins agree to 0.1 %; the HT arm samples worse (see Found).
- `p p > j j j`, `htjmin 400`, one seed, `--target-rel 0.005`: σ =
  177154.4 ± 605.2 pb in 6 iterations (720k evaluations, 2m57s on a loaded
  4-core host), with "12 of 15 channels accept less than one point in 4".
  Not compared against anything; it shows the non-aligned case runs.

### Banking request

Two rows. Both are 2 → 2, cheap for MadEvent, and copy existing rows' cards
with only the HT lines added.

Scripts (committed):
`validation/madgraph/scripts/pp_to_jj_ht.mg5` (`generate p p > j j`, `set
ebeam1 6500`, `set ebeam2 6500`, `set pdlabel lhapdf`, `set lhaid 247000`, `set
htjmin 300`, `set htjmax 800`) and `validation/madgraph/scripts/pp_to_bb_iht.mg5`
(`generate p p > b b~`, the same beams and PDF, `set ihtmin 200`, `set ihtmax
500`, `maxjetflavor` left at its default 4).

Commands, from a checkout with the reference bundle fetched:

```
pixi run -e madgraph build-diagrams
ROWS="pp_to_jj_ht pp_to_bb_iht" pixi run -e madgraph bank-seeded-sigma
```

`build-diagrams` skips every row whose `output/<row>` exists, so it runs only
the two new scripts and writes `output/pp_to_jj_ht/` and `output/pp_to_bb_iht/`
(the banked run, whose `Cards/run_card.dat` the seeded driver then reads).
`bank-seeded-sigma` (default seed plan: 20261101-20261105 at the budget,
20261111-20261112 at 4x) writes `output/seeded_sigma/<row>/s<iseed>/` and
`output/seeded_sigma/<row>/seed_record.json`, and prints the SCREEN, LADDER,
SURVEY and SCATTER lines per row.

Expected: the run cards in `output/<row>/Cards/run_card.dat` carry
`300.0 = htjmin`, `800.0 = htjmax` (`pp_to_jj_ht`) and `200.0 = ihtmin`,
`500.0 = ihtmax`, `4 = maxjetflavor` (`pp_to_bb_iht`); σ near 2.13e5 pb and
4.85e3 pb if vibegraph's reading is right. A MadGraph σ away from those by more
than the combined errors is a definition disagreement to diagnose per event,
not a tolerance to widen.

Manifest rows (committed, `status = "planned"`, `integrals` `blocked` on the
run): `pp_to_jj_ht` and `pp_to_bb_iht` in `validation/manifest.toml`. Once
banked, the follow-up adds the σ gates beside `pp_to_jj`'s and `pp_to_bb`'s in
`validate_hadronic.rs` and drops `status`.

### What these rows can and cannot see

- They see the edges, the `shat_min` floor, and whether the inclusive sum takes
  the b-class legs (`pp_to_bb_iht` has no other cut on its final state).
- With two partons `HT = 2 pT`: they cannot tell a scalar from a vector sum,
  cannot see which jets a sum of three or more takes, and cannot see the
  jet/b split inside one event (`pp_to_jj` has no b, `pp_to_bb` no jet). The
  per-event tests pin those; a 3-jet row (`p p > j j j`, `htjmin 400`) would be
  the σ-level check, at a vibegraph cost that is currently poor (see Found).
- A parton-level HT is not a jet HT: no clustering, no η acceptance. MadGraph
  defines it the same way, so this is a difference from detector-level
  analyses, not from the reference.

## Gates

Run with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

```
$ cargo fmt --all --check
exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking vibegraph-validation-report v0.1.0 (/home/user/wt/K/validation-report)
    Checking ureq v3.3.0
    Finished `dev` profile [optimized] target(s) in 2m 16s
exit 0

$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
    Checking vibegraph-lib v0.1.0 (/home/user/wt/K/vibegraph-lib)
   Compiling vibegraph v0.1.0 (/home/user/wt/K/vibegraph-cli)
    Finished `dev` profile [optimized] target(s) in 59.30s
exit 0

$ cargo test --workspace
...
     Running unittests src/lib.rs (target/debug/deps/vibegraph-9ce35d39951b2098)
test result: ok. 1073 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 48.68s
...
test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.34s
exit 0
(47 test binaries and doc-test runs: 1657 passed, 0 failed, 19 ignored)

$ cargo test -p vibegraph-lib --lib cuts::
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; 1020 filtered out; finished in 0.13s

$ python3 scripts/kb.py lint
912 concepts, 296 backlog items; 0 errors, 4 warnings
```

The four lint warnings are on backlog items this session does not touch
(`state: blocked` without `blocked_by`). The MadGraph net (`pixi run validate`)
was not run: this session changes cuts, not amplitudes, and the worktree has no
reference bundle.

## Found

- **An HT cut gives the channels no regulator.** `spacelike_floor` reads
  single-leg `pT` minima only, so an HT-only card builds its peripheral channels
  unregulated. Evidence: the `ihtmin/ihtmax` `p p > b b~` card at 300000 x 10
  scatters at χ²/dof 3.76 over five seeds where its `ptb` twin, the same phase
  space, reads 0.62; its quoted error is about half its measured spread. On
  `p p > j j j` with `htjmin 400`, 12 of 15 channels accept under a quarter of
  their points. What would close it: a bound or scale the HT minima imply for
  the channel maps (for two final partons, each `pT ≥ htmin/2`; beyond that a
  scale, not a bound), with a σ ladder showing the scatter falls to χ²/dof ≈ 1.
- **The σ gate for HT is not yet run against MadGraph.** The two planned rows
  need the banking request above, then gates in `validate_hadronic.rs`; the
  backlog item's `closes_when` needs that MadGraph comparison.
- **No σ-level check of a non-aligned HT.** With two partons HT is `2 pT`. A
  `p p > j j j` row with `htjmin` would be the first to test the sum over three
  jets against MadGraph; it depends on the regulator item above for a usable
  vibegraph cost.
- **The partial sums `ht2min..ht4max` stay refused.** They need the
  pT-ordered jet list `cuts.f:826-835` builds, and the same list would serve
  `ptj1min..ptj4max` (also refused). One sorted-jets helper would close both
  families.
