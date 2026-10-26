---
type: Session Report
title: "F1 report: a mixed-mass card splits into one integrand per set of outgoing masses"
description: "Subprocesses with different outgoing masses are split before flavour grouping, one proton integrand each, combined like a sum over multiplicities; no map code changed. A one-flavour part matches MadGraph, and the total needs a banked run."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-22}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/L`, branch `sprintd-flavour-mixed-masses`, started
from `e02f82f`. The brief said `c0e5907`; `e02f82f` is the commit that adds
the brief on top of it.

## 1. How MadGraph groups these subprocesses

Read at b7687064 from raw.githubusercontent.com, file
`madgraph/iolibs/group_subprocs.py`:

- `SubProcessGroup.group_amplitudes` (lines 421-449) calls
  `find_process_classes` (line 434). It makes one `SubProcessGroup`, and so
  one `P` directory, per class (lines 437-447).
- `find_process_classes` (lines 454-513) uses, under `criteria == "madevent"`
  (lines 482-490), the class

  ```python
  proc_class = [ [(p.is_fermion(), ) for p in is_parts],
                 [(p.get('mass'), p.get('spin'),
                   p.get('pdg_code') % 2 if p.get('color') == 1 else 0,
                   abs(p.get('color')),l.get('onshell')) for (p, l)
                   in zip(is_parts + fs_parts, process.get('legs'))],
                 amplitude.get('process').get('id'),
                 process.get('id')]
  ```

  Every leg's mass is part of the key, initial and final, in leg order. So
  MadGraph **does** split on masses, here. Subprocesses with different
  outgoing masses never share a `P` directory. Each directory is integrated
  over its own channels, and the unweighting combines all of them.

Two details differ from vibegraph, and neither changes σ:

- MadGraph keys on the mass *parameter name* (`ZERO`, `MTA`), not its value.
- It keys in leg order, not as a multiset.

The first means two parameters of equal value would split there and merge
here. The second means `p p > w+ j` and `add process p p > j w-` are two
directories there and one part here, as `content_ordered` already arranges.
Both are the same phase space, so merging them is exact.

## 2. What changed, and the deviation from the brief

**The brief is wrong about where the split can go.** It asks
`derive_flavor_groups` to put mass-split subgroups into one `FlavorGroups`,
"built with the same map construction every group uses today". One
`FlavorGroups` feeds one `ProtonIntegrand`, and that integrand evaluates
**every** group's `|M|²` at every point any of its channels draws:
`shared_scale_sum` at `proton.rs:2065` and `per_group_sum` at `:2130` loop
over all groups. It also weights the point by `αⱼ / Σₖ αₖ gₖ` over every
channel (`phasespace/channel.rs:889`). Groups with different masses inside one
integrand would therefore be read off their mass shell, under a mixture
density that sums channels which cannot reach the point. Making that route
correct needs map code (see `map-changes-for-the-maps-session.md`).

The split therefore goes one level up, where MadGraph puts it: one
`ProtonIntegrand` per set of outgoing masses. `MultiplicitySum` already
composes several such integrands, one per final-state multiplicity.
`derive_flavor_groups` keeps its refusal as the invariant that one
decomposition is one phase space, and its message now says to split first.
No map, VEGAS or budget code changed.

Changes, by file (lines at the commit):

- `vibegraph-lib/src/multiplicity.rs`
  - Module doc "Outgoing masses" section (lines 18-30): why a part is one
    phase space, with the MadGraph citation.
  - `split_by_outgoing_masses` (line 102): one part per multiset of
    outgoing pole masses, in first-reached order. An empty input comes back
    whole, so the decomposition still refuses it by name.
  - `multiplicity_groups` (line 146): splits each multiplicity by masses
    before `derive_flavor_groups`, and logs when it does.
  - `MultiplicitySum::new` (line 279): multiplicities must be non-decreasing,
    and no two parts may share a multiplicity *and* a mass multiset (panics
    otherwise). The old check required strictly increasing multiplicities.
  - `MultiplicitySum::channel_keys` (lines 403-450): a key's `group` is
    offset by the groups of the earlier parts of the same multiplicity
    (line 427). Without the offset, the two parts bank under colliding keys
    (mutation M3). A card with one set of masses banks byte-identical keys,
    and the artifact format is unchanged.
  - `sorted_final_masses` (line 635), and the masses added to the
    budget-share log line.
- `vibegraph-lib/src/proton.rs`
  - Module doc (lines 22-24): points at the split.
  - `ProtonError::UnequalFinalMasses` (lines 561-569): a doc comment
    saying why it stays, and a message ending "split them by outgoing masses
    first".
- `vibegraph-lib/src/artifact.rs`: the `ChannelKey::MultiplicityChannel`
  doc (lines 89-95) states what `group` counts.
- `vibegraph-lib/src/generate.rs`
  - `RoundingOnMixedMultiplicity` message and doc (lines 170-178): "a sum
    over N final-state phase spaces (multiplicities or sets of outgoing
    masses)". A mixed-mass card is now more than one part, and the old text
    would have called it "multiplicities".
  - The `part_sigmas` doc, and the test string at line 1442.
- `vibegraph-cli/src/generate.rs:299-303`: the same message for the CLI.
- `vibegraph-cli/src/integrate.rs`
  - Module and function docs.
  - `report_multiplicities` (line 930): each part's line names its
    masses, because two parts can now share a leg count.
- `vibegraph-lib/src/lhef/emit.rs`: the docs of `WeightedEvent::part` and
  `EmitPlan::parts` say "phase space".

## 3. Tests and mutation evidence

There was **no old test of the refusal** to reframe. `UnequalFinalMasses`
had no test anywhere (`grep -rn UnequalFinalMasses` finds the definition and
the one `return Err`), which is another error in the brief. The refusal is
now pinned inside the first new test.

Both new tests are in `vibegraph-lib/src/multiplicity.rs`. They are hermetic
(built-in SM, synthetic PDF) and use the card
`define l+ = e+ mu+ ta+ / define l- = e- mu- ta- / generate p p > l+ l-`,
where `ta` is 1.777 GeV and `e`, `mu` are massless.

**`mixed_outgoing_masses_split_into_one_decomposition_per_mass_set`** checks:

- `multiplicity_groups` decomposes the card, as 2 parts;
- the parts have groups `[2, 2]` (up-type and down-type initial states each);
- every group of the first part has masses `[0, 0]` and only `e`/`mu`
  members, and every group of the second has `[mτ, mτ]` and only `ta`
  members;
- the parts hold every enumerated subprocess exactly once;
- `derive_flavor_groups`, handed the unsplit multiplicity, still returns
  `UnequalFinalMasses`.

**`mixed_mass_parts_integrate_to_one_flavour_each`** checks:

- the two parts compose into one `MultiplicitySum`, every channel key is
  distinct, and the sum's σ is the sum of its parts';
- `σ(ττ) / σ(ee + μμ)` is one half within `5σ + 0.002`. The tau mass moves
  it by `O(mτ²/m_ll²)`. Measured: `ratio 0.50583 ± 0.00431`.

Mutations: each applied alone by a script (it backs up the file, edits it,
runs `cargo test -p vibegraph-lib --lib multiplicity::tests::mixed_`, and
restores the file). The full log is reproduced here:

```
=== M1 no mass split in multiplicity_groups: vibegraph-lib/src/multiplicity.rs
--- 'let parts = split_by_outgoing_masses(multiplicity, model, evaluated);'
+++ 'let parts = vec![multiplicity];'
exit 101
test multiplicity::tests::mixed_outgoing_masses_split_into_one_decomposition_per_mass_set ... FAILED
test multiplicity::tests::mixed_mass_parts_integrate_to_one_flavour_each ... FAILED
thread '...mixed_outgoing_masses_split...' panicked at vibegraph-lib/src/multiplicity.rs:954:14:   (expect: "a card of mixed outgoing masses decomposes")
=== M2 split key ignores masses
--- '        masses.sort_unstable();\n        masses\n    };'
+++ '        masses.clear();\n        masses\n    };'
exit 101
test ...mixed_mass_parts_integrate_to_one_flavour_each ... FAILED
test ...mixed_outgoing_masses_split_into_one_decomposition_per_mass_set ... FAILED
=== M3 no group offset in channel keys
--- 'let group = group_offset + id.group;'
+++ 'let group = id.group + 0 * group_offset;'
exit 101
test ...mixed_outgoing_masses_split_into_one_decomposition_per_mass_set ... ok
test ...mixed_mass_parts_integrate_to_one_flavour_each ... FAILED
assertion `left == right` failed: two channels share a key: [MergedChannel { final_state: 2, group: 0, channel: 0, pairs: 4 }, MergedChannel { final_state: 2, group: 0, channel: 0, pairs: 4 }]
=== M4 derive_flavor_groups refusal removed: vibegraph-lib/src/proton.rs
--- 'if masses != final_masses {'
+++ 'if false && masses != final_masses {'
exit 101
test ...mixed_outgoing_masses_split_into_one_decomposition_per_mass_set ... FAILED   (at multiplicity.rs:1011, the refusal assertion)
test ...mixed_mass_parts_integrate_to_one_flavour_each ... ok
```

Each mutation fails at least one new test, and each test is killed by at
least one mutation. The line numbers in the log are from before `cargo fmt`
reflowed the assertions.

What the tests cannot see:

- The ratio test is blind to an error that both parts share, and to a mass
  effect below its ~1% resolution. Section 4 measures the mass effect on
  real PDFs: -1.15%.
- Neither test can see whether MadGraph's partition matches ours. Only the
  banked total of section 4 would show that.

## 4. σ against MadGraph

The survey's own two cards are not in this worktree. The survey data stayed on
the CERN host's scratchpad, and the survey page names no card for this
blocker. I used a minimal card that reaches the same split, on the `pp_to_ll`
row's own run card:

```
import model sm
define l+ = e+ ta+
define l- = e- ta-
generate p p > l+ l-
```

with `validation/madgraph/dy13_default_run_card.dat` (NNPDF23_lo_as_0130_qed,
fixed scales 91.188, `ptl 10`, `etal 2.5`, `drll 0.4`). Its `e+ e-` part is
exactly the banked `dy13_default` process (`p p > e+ e-`, same card), so that
part has a MadGraph reference. The total and the `ta+ ta-` part do not.

Command, per seed (release build of this commit, 4 threads):

```
target/release/vibegraph integrate <proc card above> --run-card validation/madgraph/dy13_default_run_card.dat \
  --pdf-dir validation/pdf --fixed-budget --neval 120000 --niter 12 --seed <s> -j 4 --no-tui --out <dir>
```

The seeds are 20260719-23, the `pp_to_ll` gate's own. The same five seeds were
also run on `validation/madgraph/dy13_proc_card.dat` as the control. Per seed,
from the run's own log:

| seed | e+e- part (masses 0, 0) | ta+ta- part (masses 1.777, 1.777) | total | control `p p > e+ e-` |
|---|--:|--:|--:|--:|
| 20260719 | 935.155881 ± 1.374200 | 923.247129 ± 1.361690 | 1858.403010 ± 1.934587 | 934.645517 ± 0.968515 |
| 20260720 | 933.266374 ± 1.377867 | 923.735414 ± 1.383796 | 1857.001788 ± 1.952795 | 933.703576 ± 0.968739 |
| 20260721 | 934.980762 ± 1.391948 | 923.047065 ± 1.359511 | 1858.027827 ± 1.945711 | 934.428328 ± 0.972665 |
| 20260722 | 936.176496 ± 1.375805 | 924.489810 ± 1.356823 | 1860.666306 ± 1.932307 | 932.501525 ± 0.970238 |
| 20260723 | 933.123351 ± 1.371250 | 924.369740 ± 1.360915 | 1857.493091 ± 1.931946 | 934.327489 ± 0.974927 |

Under the seed policy (unweighted mean ± max(quoted, spread/√n)), against
the committed `hadronic_sigma_reference.json` `default` = 933.23 ± 0.47991 pb:

| | mean (pb) | χ²/dof | against MadGraph |
|---|--:|--:|--:|
| e+e- part | 934.541 ± 0.616 | 0.91 | pull +1.68, rel +0.140% |
| ta+ta- part | 923.778 ± 0.610 | 0.23 | no reference |
| total | 1858.318 ± 0.867 | 0.53 | no reference |
| control `p p > e+ e-` | 933.921 ± 0.434 | 0.80 | pull +1.07, rel +0.074% |

How to read this:

- The `e+ e-` part of the split card agrees with MadGraph inside the
  `pp_to_ll` gate's bounds (`|pull| < 3`, `|rel| < 0.005`).
- It agrees with the control run on the same seeds (difference 0.62 ± 0.75,
  pull +0.82), so splitting off the tau part leaves the light part unchanged.
- `ττ/ee = 0.98848`: the tau mass costs 1.15% under these cuts.
- The reference used here is the committed single run. The bundle's seed
  record for `dy13_default`, which the `pp_to_ll` gate reads, is not in this
  worktree (`validation/madgraph/output` is absent).
- The total, and the ττ part, have no MadGraph number. Hence the banking
  request below.

### Banking request: `p p > l+ l-` with `l = e, ta` on the dy13 default card

**Card.** The process is the four lines above, with
`dy13_default_run_card.dat` used verbatim. The process and run card are
those of `pp_to_ll`'s `dy13_default` variant, except that the multiparticle
adds `ta`. So a disagreement in the `e` directory is the existing row's, and
one in the `ta` directory, or in the total, is this split's.

**Generator.** The pinned MadGraph 3.7.1 submodule (b7687064) via
`mg5_pinned.sh`, as for refdata 10.

**Script.** `bank_seeded_sigma.sh` gains a row `dy13_etau_default`:

- `row_kind` (line 191): `dy13_etau_default) echo dy13etau ;;`
- `write_generation_script` (line 220):
  `dy13etau) printf 'define l+ = e+ ta+\ndefine l- = e- ta-\ngenerate p p > l+ l-\noutput %s -nojpeg\n' "$tmpl" > "$out" ;;`
- the card map (lines 408-412): `dy13etau) card="$HERE/dy13_default_run_card.dat"; generator="define l+ = e+ ta+; define l- = e- ta-; generate p p > l+ l- ; run card dy13_default_run_card.dat" ;;`
- `@hadronic` (line 171): add `dy13_etau_default`.

The reference is then a seeded record under the seed policy (5 seeds at the
row's budget plus 2 at 4×, `hard_survey = 2`), kept under
`output/seeded_sigma/dy13_etau_default/`.

**Commands.**

```
cd validation/madgraph
./bank_seeded_sigma.sh dy13_etau_default
ls output/seeded_sigma/dy13_etau_default/s*/   # per seed: SubProcesses/results.dat, P*/symfact.dat, G*/results.dat
```

**Expected outputs.**

- **Two `P` directories**, one per lepton mass. A `qq~ → e+e-` directory and
  a `qq~ → ta+ta-` directory, because `find_process_classes` keys on `MTA`
  against `ZERO`. One directory would contradict section 1 and should stop the
  banking.
- The `e` directory's channel results summed: about 933 pb (the existing
  `dy13_default` record).
- The `ta` directory: about 923.8 pb (vibegraph 923.778 ± 0.610).
- The total, `SubProcesses/results.dat` field 1: about 1858 pb (vibegraph
  1858.318 ± 0.867).
- Per-directory sums matter because the `ta` part is the new physics. A total
  alone would hide two offsetting errors.

**Manifest row** (full text):

```toml
[[process]]
key = "pp_to_ll_etau"
process = "p p > l+ l-"
script = "validation/madgraph/bank_seeded_sigma.sh"
class = "multi-channel"
n_final = 2
status = "planned"
rationale = "Drell-Yan with l = e, ta on the dy13 default card: one multiplicity whose subprocesses differ in outgoing masses (ta at 1.777 GeV, e massless), so it is the only row that integrates two phase spaces of the same leg count as two parts and sums them. MadGraph keys its subprocess groups on every leg's mass (group_subprocs.py find_process_classes) and puts the two in separate P directories; this crate splits them before flavour grouping (multiplicity::split_by_outgoing_masses). The e part is pp_to_ll's dy13_default process on the same card, so a disagreement in the ta part or the total is about the split."
artifacts = ["output/seeded_sigma/dy13_etau_default", "dy13_default_run_card.dat", "dy13_param_card.dat"]
[process.categories]
diagrams = { tier = "covered-by", rows = ["pp_to_ll"] }
amplitudes = { tier = "covered-by", rows = ["uux_to_mumu"], note = "the ta+ ta- subprocesses differ from it by the external mass; ee_to_tatah and ee_to_mumu_tata_qcd0 carry massive taus at amplitude level" }
integrals = { tier = "banked", mode = "info", note = "five seeds a side under the seed policy, read as pp_to_ll's dy13 cells are (|pull| < 3, |rel| < 0.005, seeds' chi2/dof < 4), on the total and on each P directory's sum against the matching part. Informational until the first measurement agrees; flip to gate in the PR that lands the bundle. Pre-registered from this side (F1, 5 seeds at 120000 x 12): total 1858.318 +- 0.867 pb, ta part 923.778 +- 0.610, e part 934.541 +- 0.616. Known-wrong control: the ta part integrated on the massless e channels (one part, masses 0) is the old refusal's alternative and must miss the ta P directory by the tau-mass effect, about -1.15% (10.7 pb) here, over ten times the combined error at this budget." }
samples = { tier = "uncovered", note = "no event sample banked; the per-part event split would be the column a sample adds" }
```

**Tests that would read it.** One new test in `validate_hadronic.rs` that
runs the card through `multiplicity_groups` and `MultiplicitySum` instead of
`groups_for`. `check_dy_run` builds one `FlavorGroups`, and on this card that
would hit `UnequalFinalMasses`. The test reads `MultiplicitySum::part_results`
against the per-directory sums and the total against `results.dat`. This is a
test-side change only.

## 5. Gate

All run in `/home/user/wt/L` with `CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the final tree.

```
$ cargo fmt --all --check            # after one `cargo fmt --all` reflowing three new assertions
(no output) exit 0

$ cargo clippy --workspace --all-targets -- -D warnings
    Checking vibegraph-validation-report v0.1.0 (/home/user/wt/L/validation-report)
    Finished `dev` profile [optimized] target(s) in 2m 30s
clippy exit 0

$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
    Checking vibegraph-lib v0.1.0 (/home/user/wt/L/vibegraph-lib)
   Compiling vibegraph v0.1.0 (/home/user/wt/L/vibegraph-cli)
    Finished `dev` profile [optimized] target(s) in 1m 01s
clippy-ev exit 0

$ cargo test --workspace
test result: ok. 1064 passed; 0 failed; 12 ignored; ... (vibegraph-lib --lib, the new tests included)
...
test result: ok. 1 passed; 0 failed; 2 ignored; ... (Doc-tests vibegraph)
test exit 0
summed over the log: 47 suites, 1648 passed, 0 failed, 19 ignored

$ python3 scripts/kb.py index && python3 scripts/kb.py lint
wrote 2 index files
914 concepts, 296 backlog items; 0 errors, 4 warnings   (the 4 warnings are pre-existing backlog `state: blocked` items)
```

The MadGraph net (`pixi run --skip-deps validate`) was not run: pixi is not
installed here and the reference bundle is absent. No amplitude, colour,
coupling or enumeration code changed. The banked `pp_to_ll` cells are untouched
in what they integrate: a card with one set of outgoing masses goes through
`split_by_outgoing_masses` as one part, with byte-identical channel keys,
and `a_sum_of_one_multiplicity_is_that_multiplicity` still passes bit for bit.

## Commits

One commit on `sprintd-flavour-mixed-masses`, author Nicholas Smith, carrying the code, both tests, this report and `map-changes-for-the-maps-session.md`. Not pushed.

## Found

1. **The fixed-energy path refuses a multi-flavour card wholesale.**
   `compile_subprocesses` (`hadronic.rs:975-1012`) requires every subprocess
   to have identical external particles (`InconsistentExternals`). So
   `e+ e- > l+ l-` with `l ∋ ta` is refused at `lpp = 0`, and so is
   `e+ e- > l+ l-` over `e, mu`, whose masses agree. It is a refusal, not
   wrong physics. The proton path's part split would carry over to
   `FixedBeamIntegrand` if that path had a composite like `MultiplicitySum`.
   Evidence: the code read. Closed by: a fixed-beam sum over parts, or a
   documented refusal naming the cause.
2. **Mass-name versus mass-value keys.** MadGraph keys on the mass parameter
   name and vibegraph on its value (section 1). The two differ only when
   distinct parameters share a value, where merging is exact. It is worth one
   sentence in the flavour-groups concept, so a later reader does not "fix"
   it. Evidence: `group_subprocs.py:485`.
3. **The release build warns, and clippy does not catch it.**
   `vibegraph-lib/src/helas/eval/analysis.rs:208` has `resolve_mom` unused
   (`dead_code`) in `cargo build --release -p vibegraph`, which the lint gate
   (debug, all targets) does not see. It is on `e02f82f`; this session did not
   touch the file. Closed by: removing it or gating it on the cfg that uses it.
4. **The survey does not name the two affected cards.** The survey page (and
   the backlog item) count 2 cards with this blocker but name neither. The
   item's `closes_when` ("one of the survey's two cards passes grouping with
   σ matching MadGraph") cannot be checked from this repository. Closed by:
   recording the card ids from `results.jsonl` on the CERN host, or by
   restating `closes_when` against the banked row above.
