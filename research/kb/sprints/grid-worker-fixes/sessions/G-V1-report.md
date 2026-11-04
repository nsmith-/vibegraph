---
type: Session Report
title: "G-V1 report: the ttx MLM row's own events against MadEvent's run_01"
description: "vibegraph generates p p > t t~ @0 + p p > t t~ j @1 under the row's card; 18 weighted population shares against MadEvent's banked run_01 agree within |pull| 1.89 (five more seeds: 1.94), gated in the banked layer at 13 s. The c-member line change leaves the llj and ll_0j2j samples byte-identical."
status: draft
tags: [grid-worker-fixes, generate, mlm, ttx, samples, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker`, from 9aa4df0.
All runs used `CARGO_INCREMENTAL=0`, with the banked targets under
`--profile release-debug --config profile.release-debug.debug=0`, which are
the cargo commands the pixi tasks wrap. Scratch is under
`/tmp/claude-0/-home-user-vibegraph/7497b6d2-0784-5f2c-a2c9-2ed0beaa757e/scratchpad/G-V1/`.

## The test

`vibegraph-cli/tests/cli_generate_proton.rs`
`ttx_matched_sample_record_fractions_against_madevent` (banked, not ignored).
It uses this proc card:

```
import model sm
generate p p > t t~ @0
add process p p > t t~ j @1
```

with the committed `validation/madgraph/pp_to_ttx_0j1j_mlm_run_card.dat`. It
runs integrate (`--fixed-budget --neval 20000 --niter 4 --seed 20260731`) and
generate (10 000 events, same seed). The record is then compared with
`validation/madgraph/output/pp_to_ttx_0j1j_mlm/Events/run_01/unweighted_events.lhe.gz`
(10 000 events, read only).

**Estimator.** This side's shares are **weighted**. Generate keeps an event
drawn above its channel maximum at a weight above one. On this card that is
0.63 % of events and 4.8 % of σ, with a largest w/w_max of 10.3. Generate
also normalises each `@N` to its own integration. A count would therefore
measure the accept/reject pass, not the distribution. On the first sweep
seed, the @1 share pulls −0.68 by count and −0.01 by weight. The variance is
`Σ w²(1{sel} − p)² / (Σw)²`, unpooled. MadEvent's weights are equal, so the
same formula reduces to `p(1 − p)/n` there. The pull is
`(p₁ − p₂)/√(v₁ + v₂)`, gated at |pull| < 5. This departs from the llj
test's pooled count binomial, for the reason above.

**Exact assertions on both files.** No status-2 line appears on either
file: the tops are external legs, so neither side writes a top resonance,
and that share is 0 = 0 rather than a pull. `IDPRUP` equals the multiplicity
(0 or 1). `@0` has no qg initial state. Each event has at most one outgoing
non-top parton.

## Fraction table (gate seed 20260731)

| share | vibegraph (weighted; counts) | MadEvent | pull |
|---|---|---|---|
| @1 | 0.5280 (5294/10000) | 0.5306 (5306/10000) | −0.37 |
| @1: gluon jet | 0.7488 (3973/5294) | 0.7459 (3958/5306) | +0.34 |
| @1: d jet (\|pdg\| 1) | 0.0808 (427/5294) | 0.0818 (434/5306) | −0.18 |
| @1: u jet | 0.1376 (720/5294) | 0.1440 (764/5306) | −0.92 |
| @1: s jet | 0.0203 (109/5294) | 0.0155 (82/5306) | +1.89 |
| @1: c jet | 0.0124 (65/5294) | 0.0128 (68/5306) | −0.19 |
| @1 quark jets: antiquark | 0.1749 (233/1321) | 0.1669 (225/1348) | +0.55 |
| initial gg | 0.7888 (7888/10000) | 0.7864 (7864/10000) | +0.42 |
| initial qg | 0.1326 (1321/10000) | 0.1348 (1348/10000) | −0.45 |
| initial qq̄ | 0.0786 (791/10000) | 0.0788 (788/10000) | −0.06 |
| @0: gg | 0.8697 (4088/4706) | 0.8707 (4087/4694) | −0.14 |
| @0: qq̄ | 0.1303 (618/4706) | 0.1293 (607/4694) | +0.14 |
| @1: gg | 0.7165 (3800/5294) | 0.7118 (3777/5306) | +0.53 |
| @1: qg | 0.2512 (1321/5294) | 0.2541 (1348/5306) | −0.34 |
| @1: qq̄ | 0.0323 (173/5294) | 0.0341 (181/5306) | −0.52 |
| `<asrwt>` emission scale | 0.5231 (5245/10000) | 0.5253 (5253/10000) | −0.30 |
| @1: `<asrwt>` emission scale | 0.9909 (5245/5294) | 0.9900 (5253/5306) | +0.46 |
| `<pdfrwt>` chain > 1 entry, per beam | 0.2607 (5227/20000) | 0.2617 (5234/20000) | −0.23 |
| status-2 lines (top resonances) | 0 | 0 | exact |

The largest |pull| is 1.89, over 18 shares. An independent Python reader
(`G-V1-fractions.py`) gives the same numbers from the same file, at `-j 2`.
The test runs at the default thread count, which is consistent with generate
being thread-independent.

### What one seed shows, and the sweep

At 10 000 events a side, five standard deviations correspond to about 3.5 %
absolute on a share near one half, 2.4 % on the qg share, and 1.2 % on the
strange-jet share, which is about 75 % of that share. The gate therefore
pins the labelling, the multiplicity split and the record populations
against gross error. It cannot see percent-level shape or flavour
differences.

Seeds 20261101–05 were run at the gate's budget (`G-V1-sweep.log`). Over
those five seeds and every share, no |pull| exceeded 1.94 (`@0` gg, seed
20261104). The largest mean pulls were `@0` gg / qq̄ at ±0.88, initial qq̄ at
−0.83, and @1 emission at −0.49. The sweep cannot calibrate the error. All
seeds compare with one MadEvent sample, so their pulls share that sample's
fluctuation. Shares that follow the integration's @0/@1 ratio (the @1 share,
the emission share, the long-chain share) also have Σpull²/n of 0.03–0.07.
For those, the weighted delta-method variance overstates the spread, because
the ratio is fixed by the integration rather than drawn. The gate is
conservative on those shares, and their small pulls are not evidence of
better than ~0.5 % agreement.

**Mutation control.** Swapping u ↔ d on the outgoing jets of the gate
sample moves the d-jet share to +8.96σ and the u-jet share to −10.30σ. A
flavour mislabel at that scale fails the gate.

## The V2 premise: lab-against-CM clustering

The brief asks for an estimate of V2's lab-against-CM clustering difference.
**That difference is no longer in the code.** #23 (9e141de) clusters
`term_at` at the partonic CM (`proton.rs` `per_group_sum`). The manifest's
integrals cell then has @0 at pull +0.31 and @1 at +0.70 against the
`hard_survey = 2` seed record. V2's "residual excess" was attributed in the
same PR to MadEvent's default survey, which under-integrates
`g q > t t~ q` (`validation/madevent-default-survey-bias.md`).

That survey bias matters here, because `run_01` ran at the default survey
(`hard_survey = 0` in its banner). Its own @1 is 572.52 pb, against 578.34
in the seeded record. In the worst case, the whole −5.8 pb sits in qg and
MadEvent's events follow its own channel σ. MadEvent's qg share then reads
low by about 5.8/1092 × (1 − 0.133) ≈ 0.0046, which is about 1σ (sd 0.0048).
Its @1 share reads low by about 0.003, about 0.4σ. The measured G2 bias
(−0.55 % of gq, about 0.8 pb) is worth about 0.15σ on qg. The observed qg
pull is −0.45, which fits either size. For scale, the old frame defect
(−4.3 pb on gg in @1) would have moved the @1 share by about 0.002, which is
0.3σ. Neither effect can move any share by more than about 1σ at this
sample size.

## Placement and wall time

The test is in the **banked layer**, as an ordinary test in
`cli_generate_proton`, which `validate-generate-proton` already runs. It
needs no new pixi task. Measured on a 4-vCPU container:

```
$ cargo test -p vibegraph --profile release-debug --config profile.release-debug.debug=0 --features extended-validation --test cli_generate_proton -- --nocapture ttx_matched_sample
largest |pull| 1.89 over 18 shares
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 13.28s
```

The whole target took 42.8 s with it, and 55.1 s on the re-run after the
clippy fix (`--test-threads 1`). By comparison, G-F2's gq test took 25.7 s
for the target, and the llj `pp_to_llj_fixed` gate integrates 300 000 × 8.

Bookkeeping: `validation/manifest.toml`. The `pp_to_ttx_0j1j_mlm` samples
cell records this measurement. `cli-generate-proton`'s inputs now name
`output/pp_to_ttx_0j1j_mlm/Events/run_01`.

## The c-member line change on llj and ll_0j2j

The gates that read those rows pass (table below). Passing them is weak
evidence, so a finer check was run. Two binaries were built: the current
one, and the same tree with e9041eb's `generate.rs` and `resonance.rs`
reverse-applied (restored afterwards). Each integrated and generated the
llj matched card (`p p > e+ e- j`, 20 000 × 4, 3000 events) and the
`ll_0j2j` card (the reweight test's proc card, same budget). In both rows
the grids and the LHE files are **byte-identical** before and after the fix
(`cmp`). They carry 2858 and 2848 status-2 Z lines respectively. This
confirms G-F2's claim that the remapped lines are zero-width and never
written.

## Gates

| Command | Result |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` | rc 0 after the fix. The first run failed `type_complexity` on the new test; a type alias fixed it |
| `cargo test --workspace` | rc 0, every binary `ok` (lib 1196 passed, 13 ignored) |
| `cargo test -p vibegraph … --test cli_generate_proton -- --nocapture --test-threads 1` (validate-generate-proton) | 11 passed, 0 failed, 2 ignored (42.8 s; 55.1 s on the re-run after the clippy fix) |
| `… --test cli_generate_proton -- --ignored matched_sample_record_fractions_against_madevent` (validate-mlm-samples, llj) | 1 passed: Z +0.44, jet at E_tot +0.01, emission −0.01, long chains −0.00 |
| `… --test cli_reweight_proton -- --test-threads 1` (reads the ll_0j2j card) | 2 passed |
| `… -p vibegraph-lib --test validate_mlm_records` | 1 passed |
| `… -p vibegraph-lib --test validate_lhef` | 3 passed |

`validate_mlm_dumps` was not run. Its dumps are oracle-layer and are not in
this container.

## Brief corrections

- The V2 lab-against-CM difference was fixed in #23 before this sprint. The
  "residual excess" was attributed there to MadEvent's default survey. What
  still matters for this comparison is that `run_01` is a default-survey
  sample (above).
- "Two-sample binomial comparison" holds on MadEvent's side only. This side's
  weights are not uniform, so the gate uses the weighted variance.
- "The share listing the top resonances, if both sides write them": neither
  side writes one, so the gate asserts zero on both files.

## Found

1. **The llj fraction test's `#[ignore]` reason is stale.** It says "M0's
   matched MadEvent runs are outside the reference bundle". But
   `pp_to_llj_mlm/Events/run_01` is in the bundle (cut 9 and later,
   `manifest.toml` lines 184–187; `pixi.toml`'s comment above
   `validate-mlm-samples` already says so). It ran here in 6.8 s. To close
   it, promote `matched_sample_record_fractions_against_madevent` to the
   banked layer, with the weighted estimator used here, and retire
   `validate-mlm-samples`. Its manifest note also records 0.9547 for the Z
   share where this run gives 0.9527. The difference is within errors, but
   the recorded cell is from 2026-10-03, before #23's clustering change.
2. **`run_01` of `pp_to_ttx_0j1j_mlm` is a default-survey sample.** Its @1 is
   572.5 against 578.3 pb at `hard_survey = 2`. At 10 000 events this is
   below 1σ on every share, but a larger-sample or shape comparison against
   it would inherit MadEvent's G2 under-integration. To close it, bank a
   `hard_survey = 2` event run for the row (the seeded record has the σ but
   no events in use here). That is new reference data, so it is the user's
   job on the bank host.
3. **No distribution-level comparison of the ttx sample.** These are
   population shares only. The top p_T, the t t̄ mass, the jet p_T above
   xqcut, and the clustering-scale distributions are untested against
   MadEvent. To close it, add binned χ² comparisons in the
   `validate_samples_proton` style for this row's run_01.
