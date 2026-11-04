---
type: Session Report
title: "G-F2 report: a quark line in a mixed-flavour group takes each member's quark"
description: "member_line_pdg assumed a group's members differ from the representative only by conjugation; a QCD group joins u and d. A line whose flavour a leg below it carries now takes the member's flavour of that leg. generate writes events on the quark-jet and the full-jet ttx cards, and the per-subprocess counts agree with separate integrations (chi2 3.90 / 7)."
status: draft
tags: [grid-worker-fixes, generate, resonance, flavour-groups, ttx, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker`. Closes
`generate-refuses-quark-only-jet-ttx`.

## The card

`gq.dat`, rebuilt:

```
import model sm
define p = g u c d s u~ c~ d~ s~
define j = u c d s u~ c~ d~ s~
generate p p > t t~ j
```

with `validation/madgraph/output/pp_to_ttx_0j1j_mlm/Cards/run_card.dat`
(`ickkw = 1`, `xqcut = 30`, lhaid 247000, `bwcutoff = 15`). The CLI test
uses the committed `validation/madgraph/pp_to_ttx_0j1j_mlm_run_card.dat`,
which has the same values and different formatting.

## Reproduction

```
$ vibegraph integrate gq.dat --run-card run_card.dat --out quick --fixed-budget --neval 20000 --niter 3 --seed 1 -j 2
σ = 149.491300 ± 1.216033 pb
$ vibegraph generate quick/grid.bin.zst --seed 2 --nevents 200 -o quick/ev.lhe -j 2
ERROR the resonance 2 of g u > t t~ u has no flavour in the subprocess with outgoing [6, -6, 1] that shares its matrix element
(exit 1)
```

The enumeration gives 8 subprocesses in 2 groups: `g q > t t~ q` for
q = u, c, d, s, and the same for the antiquarks. QCD does not see quark
flavour, so u-type and d-type quarks share one `|M|²`.

## Root cause

`group_resonances` (`generate.rs`) gets the PDG code of each of the
representative's timelike lines in each member through `member_line_pdg`
(`lhef/resonance.rs`). That function assumed that members differ from the
representative only by conjugation. A member's line was the representative's
line where the legs below it have the same charge, and its antiparticle where
they have the opposite charge.

In a QCD group that assumption is false. The `s`-channel `u` of
`g u > t t~ u` (slots: all three outgoing legs) continues into the outgoing
`u`. In `g d > t t~ d` it is a `d`. The legs below have charge 2/3 against
−1/3, which is neither equal nor opposite, so the function returned `None` and
generate refused. The member lookup itself was correct. The mapping rule was
wrong.

**Fix.** A new function, `carried_flavour`, runs first. When outgoing legs
below the line carry the line's flavour or its antiparticle in the
representative, the member's line is the member's flavour on those legs,
conjugated where the leg is the antiparticle. If no leg carries the flavour,
or if two legs name different member flavours, the charge rule decides, as
before. So for any line that no leg below carries (W, Z, γ, g, a top over its
decay products), the result is the same as before.

## Who else reaches the path

`group_resonances` runs only under `ickkw > 0` or decay chains. A run reaches
the refusal when it has a group that joins up-type and down-type quarks and
has a timelike quark line. Such groups exist only where the quark couples
through QCD alone, so photon and Z couplings do not split u from d.

- **`p p > t t~ j` with the full `j` (gluon included) is refused too.**
  Measured before the fix with the same binary and run card:
  `σ = 585.10 ± 5.12 pb`, then generate fails with the same message. So the
  banked row `pp_to_ttx_0j1j_mlm` (`@0 + @1`) could not have been generated
  by vibegraph either. After the fix, a 500-event generate on that artifact
  writes events (exit 0).
- Other matched or decay-chain QCD processes have the same structure, for
  example matched `p p > j j`, or `t t~ j` with decay chains. None of these
  were run here.
- `pp_to_llj_mlm` and `pp_to_ll_0j2j_mlm` are not refused. Z couplings
  separate u from d, so their groups are {u, c} and {d, s}. In those groups
  the charge rule gave the c member the representative's `u` on the
  `s`-channel quark line. That was wrong, and nothing refused it. After the
  fix the member gets `c`.

**Could events written before the fix carry a wrong resonance record?** No.
A record lists a line only when `on_shell` flags it (|√p² − M| < bwcutoff·Γ),
or under matching when the channel's `cut_bw` put it on its Breit–Wigner
(`clustered_on_shell`). Both need a nonzero width. The lines that got a wrong
member flavour were massless, zero-width quark lines, so they were never
written. The widthful SM lines (t, W±, Z, H) were mapped correctly by the
charge rule, and the fix leaves them unchanged unless a leg below carries
them. In that case it maps them to the same flavour. The 20 000 gq events
written here contain 0 status-2 lines.

## Tests (each fails before the fix)

- `vibegraph-lib` `lhef::resonance::tests::a_line_carried_by_a_leg_takes_the_members_flavour_of_it`:
  the u line over `[6, -6, 2]` maps to 1, −1 and 4 in the d, d~ and c members,
  and the u~ line maps to s~. When the legs are inconsistent
  (`u > u u u~` against `u > u c c~`), the charge rule decides.
- `vibegraph-lib` `generate::tests::a_quark_line_of_a_mixed_flavour_group_carries_each_members_quark`
  (hermetic, the narrowest unit on the real path): the quark-jet ttx groups go
  through `group_resonances`. It asserts that at least one group mixes u-type
  and d-type quarks, that every quark line carries the member's outgoing quark,
  and that every other line is a gluon.
- `vibegraph` `cli_generate_proton::a_quark_jet_top_pair_is_generated_across_a_mixed_flavour_group`
  (banked: it needs the fetched PDF set, and the hermetic suite has none, so
  the integrate-and-generate cannot run in the default suite). It runs
  integrate (20 000 × 3) and generate (1 000 events) on the gq card. It asserts
  no status-2 line and that all of u, d, u~, d~ are labelled. Wall time 25.7 s
  for the whole target, which also covers the other tests in it.

Fails before the fix. For this run, only the `carried_flavour` call was
removed and the tests were kept:

```
$ cargo test -p vibegraph-lib --lib -- a_quark_line_of_a_mixed_flavour_group_carries_each_members_quark a_line_carried_by_a_leg_takes_the_members_flavour_of_it
test lhef::resonance::tests::a_line_carried_by_a_leg_takes_the_members_flavour_of_it ... FAILED
test generate::tests::a_quark_line_of_a_mixed_flavour_group_carries_each_members_quark ... FAILED
  left: None
 right: Some(1)
every member names its resonance lines: Other("the resonance 2 of g u > t t~ u has no flavour in the subprocess with outgoing [6, -6, 1] that shares its matrix element")
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 1207 filtered out
```

The banked CLI test, run the same way under the release-debug profile:

```
$ cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_generate_proton -- a_quark_jet
ERROR the resonance 2 of g u > t t~ u has no flavour in the subprocess with outgoing [6, -6, 1] that shares its matrix element
test a_quark_jet_top_pair_is_generated_across_a_mixed_flavour_group ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out
```

With the fix restored, the same test passes inside the `cli_generate_proton`
run listed under Gates, and prints
`outgoing quarks labelled: {-4, -3, -2, -1, 1, 2, 3, 4}`.

## Per-subprocess check

```
$ vibegraph integrate gq.dat --run-card run_card.dat --out main --fixed-budget --neval 200000 --niter 6 --seed 20261103 -j 2
σ = 148.777701 ± 0.186570 pb
$ vibegraph generate main/grid.bin.zst --seed 7 --nevents 20000 -o main/ev.lhe -j 2
```

V2 measured 148.64 ± 0.26 pb on its gq card. `vibegraph check-events` on
the file reports 20 000 events, IDWTUP −4, XSECUP = mean XWGTUP = 148.7777 pb.

Integrate gives no per-subprocess σ. Its channels are merged across the two
groups. So each subprocess was integrated alone, with the same run card,
budget and seed (`generate p p > t t~ q`, which also covers both beam
orderings). The sum is 148.762 pb, against 148.778 ± 0.187 pb. The expected
count is N·σ_q/Σσ, with a multinomial error. The integration errors are
negligible next to it.

| q | σ_q (pb) | expected | observed | pull | weight fraction / σ fraction |
|---|---|---|---|---|---|
| u | 76.4430 ± 0.1020 | 10277.2 ± 70.7 | 10291 | +0.19 | 0.51574 / 0.51386 |
| d | 38.8267 ± 0.0497 | 5220.0 ± 62.1 | 5191 | −0.47 | 0.25961 / 0.26100 |
| s | 4.7464 ± 0.0054 | 638.1 ± 24.9 | 671 | +1.32 | 0.03336 / 0.03191 |
| c | 3.7233 ± 0.0042 | 500.6 ± 22.1 | 509 | +0.38 | 0.02547 / 0.02503 |
| u~ | 7.3768 ± 0.0085 | 991.8 ± 30.7 | 1015 | +0.76 | 0.05025 / 0.04959 |
| d~ | 9.2005 ± 0.0107 | 1236.9 ± 34.1 | 1207 | −0.88 | 0.06002 / 0.06185 |
| s~ | 4.7218 ± 0.0054 | 634.8 ± 24.8 | 620 | −0.60 | 0.03090 / 0.03174 |
| c~ | 3.7235 ± 0.0042 | 500.6 ± 22.1 | 496 | −0.21 | 0.02466 / 0.02503 |

χ² = 3.90 for 7 dof. Each subprocess's events split evenly between
gluon-first and quark-first. For example, u: 5106 / 5185 and d: 2579 / 2612.
This is one seed of one sample, a self-consistency check of the event
labelling against this side's own integrand. It says nothing about agreement
with MadEvent.

## Gates

All gates ran on the fixed tree with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
The banked gates ran with
`--profile release-debug --config profile.release-debug.debug=0`, which are
the cargo commands the pixi tasks wrap.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| `cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` | rc 0 |
| `cargo test --workspace` | rc 0; 1645 passed, 0 failed, 25 ignored |
| `cargo test -p vibegraph … --test cli_generate_proton -- --test-threads 1` (validate-generate-proton) | 10 passed, 0 failed, 2 ignored |
| `… -p vibegraph --test cli_decay_chain_events` (decay-chain resonance records) | 7 passed |
| `… -p vibegraph --test cli_decay_chain` | 1 passed |
| `… -p vibegraph-lib --test validate_mlm_records` (matched status-2 records) | 1 passed |
| `… -p vibegraph-lib --test validate_unweighting` (validate-unweighting) | 1 passed |
| `… -p vibegraph-lib --test validate_lhef` (validate-lhef) | 3 passed |
| `… -p vibegraph-lib --test validate_sigma` (validate-sigma, phase space) | 8 passed, 33 ignored |
| `… -p vibegraph-lib --test validate_hadronic` (validate-hadronic, phase space) | 20 passed, 26 ignored |

The panic lines in the `validate_sigma` and `validate_hadronic` logs come
from `#[should_panic]` tests. The full `validation/validate.sh` (every banked
target plus the collator) was not run. The oracle-layer MLM tasks need
MadGraph, and Pythia was not run either.

## Brief corrections

- The refusal is not limited to a quark-only jet. The full `p p > t t~ j`,
  and so the `pp_to_ttx_0j1j_mlm` row's own process, fails the same way.
- The brief asks for "integrate's σ split". Integrate reports no
  per-subprocess σ, and on this card it reports no per-group σ either, because
  the channels are `MergedChannel` across both groups. The split used here
  comes from per-subprocess integrations.
- The brief says the member lookup picks the wrong member. It does not. The
  representative-to-member flavour rule is what was wrong.
- An integrate-and-generate test cannot run in the default (hermetic) suite,
  because the hermetic suite has no PDF set. The hermetic pin is the unit test
  on `group_resonances`, and the CLI test is banked.

## Found

1. **Nothing pins generate on the `pp_to_ttx_0j1j_mlm` row's own process.**
   `matched_sample_record_fractions_against_madevent` covers only
   `pp_to_llj_mlm`, which is why this refusal went unseen until V2. To close
   it, add the `ttx` row to the matched samples cell (vg events against the
   banked MadEvent sample). That would also measure the record fractions this
   report checks only against itself.
2. **Integrate reports no per-subprocess σ.** A "per-subprocess counts
   against integrate's split" check therefore needs one integration per
   subprocess, and merged channels hide even the per-group split. To close it,
   have integrate accumulate each member's share of every term, the
   luminosity-weighted `|M|²` it already computes, and report or bank it.
3. **The charge rule's fallback is still a guess for lines no leg carries.**
   One example is a quark line under a non-diagonal CKM (a u line over
   {d, e+, ve} in a member with s). It is harmless while such lines have no
   width. If a model gives a width to a line whose flavour is set neither by a
   leg nor by charge, the record could name the wrong flavour without anything
   refusing it. To close it, read the member's line off the member's own
   diagram (`FlavorGroup::member_diagram_set`), matching configurations by
   their propagator slot sets.
