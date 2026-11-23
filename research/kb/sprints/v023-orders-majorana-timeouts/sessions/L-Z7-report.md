---
type: Session Report
title: "L-Z7 report: hygiene pass on parallel enumeration, the flavour prune and compilation on the pool"
description: "Findings on #185 (PR 3b) for maintainability, non-vacuity, visibility and abstractions, written before any edit; then the fixes, their mutations and the identity oracle against main at -j 1 and -j 4."
status: draft
tags: [hygiene, enumerate, compile, amplitude-probe, parallelism, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-hyg-enumerate` off main 8b3af5ab. Scope: the squash commit
`ae88d176` (#185) — `diagrams/mod.rs`, `diagrams/artifact.rs`,
`diagrams/chain.rs`, `ufo/conserved.rs`, `amplitude_probe.rs`, the compile
paths in `hadronic.rs`, `proton.rs`, `multiplicity.rs` and `generate.rs`, and
the CLI's `parallel.rs`, `enumerate.rs`, `integrate.rs` and `main.rs`.

## Findings

Written and committed before any code edit. Line numbers are at 8b3af5ab.
Points: 1 maintainability, 2 non-vacuity, 3 visibility, 4 abstractions.
Confidence: *checked* (the evidence settles it) or *suspected* (what settles
it is named, and the mutation batch below settles it).

Ranked by what a reader of the code loses if it stays.

| id | pt | site | class | finding | evidence | proposed fix | conf. | disposition |
|---|---|---|---|---|---|---|---|---|
| Z7.1 | 3, 1 | `vibegraph-lib/src/generate.rs:344` | dead code | `pub fn check_amplitude_probe` has no caller: `generate` calls the private `check_amplitude_probe_compiled` since #185 | `grep -rn 'check_amplitude_probe\b' vibegraph-lib vibegraph-cli` returns the definition (344), its own doc link (354) and a doc link (418), no call | fold the two into one private `check_amplitude_probe(artifact, compiled, evaluated)`, keeping the name two kb pages cite (`pipeline/integrate-artifact.md:278`, `events/generate.md:56`) | checked | fix |
| Z7.2 | 3, 1 | `vibegraph-lib/src/amplitude_probe.rs:257`, `:365` | dead code; contract on a test-only API | `pub fn measure` and `pub fn check` are called only by this module's tests; production calls `measure_compiled`/`check_compiled`, whose docs say only "[`measure`] through evaluators compiled already", so the probe's contract is documented on the test-only pair | `grep -rnE 'measure\(|[^_]check\(' vibegraph-lib/src/amplitude_probe.rs` lists 424, 502, 507, 511, 567, 573, all inside `mod tests`; no other file calls either | move the two into `mod tests` as helpers, put the contract on `measure_compiled`/`check_compiled`, narrow `check_compiled` (only caller `generate.rs`) to `pub(crate)` | checked | fix |
| Z7.3 | 1, 3 | `vibegraph-lib/src/multiplicity.rs:145` | dead code | `split_by_external_masses` became a test-only wrapper of `split_by_external_masses_of` and is kept alive with `#[cfg_attr(not(test), allow(dead_code))]`; its doc (the contract) sits on the wrapper production never calls | `grep -rn 'split_by_external_masses\b' vibegraph-lib vibegraph-cli`: definition 146 and one test call 1477 | delete the wrapper, give the generic function the name and the doc, and have the test pass `\|s\| s` | checked | fix |
| Z7.4 | 3 | `vibegraph-lib/src/proton.rs:1111` | public item that should be private | `pub fn derive_flavor_groups_with` has one caller, `multiplicity.rs:293`, in the crate | `grep -rn derive_flavor_groups_with vibegraph-lib vibegraph-cli`: proton.rs 1098, 1111 and multiplicity.rs 53, 293 only | `pub(crate)`; the one public doc link to it (`multiplicity_groups_of_with`) becomes plain text | checked | fix |
| Z7.5 | 3 | `vibegraph-lib/src/ufo/mod.rs:7`, `ufo/conserved.rs:37`, `:94`, `:104`, `ufo/mod.rs:653` | public items that should be private | `pub mod conserved`, `pub struct ConservedNumbers`, `pub fn len`, `pub fn values` and `pub fn UFOModel::conserved_numbers` have no user outside the crate; `len` and `values` have none outside `conserved.rs`'s tests | `grep -rn 'conserved_numbers\|ConservedNumbers\|\.values()\|\.len()' ...`: outside `ufo/conserved.rs` only `diagrams/mod.rs:1766` (`conserved_numbers`, `is_empty`, `balanced`) and its test (`none()`); nothing in `vibegraph-cli` or `tests/` | narrow the module, type and accessor to `pub(crate)`; delete `len`/`values`, the tests read the field | checked | fix |
| Z7.6 | 2 | `vibegraph-lib/src/diagrams/mod.rs:1729` | vacuous test (for one claim) | `the_flavour_prune_drops_no_subprocess_with_a_diagram` checks that pruning drops nothing it should keep, and calls `flavour_balanced` directly; nothing checks that the walk consults the prune at all, so deleting the walk's call keeps every test green | mutation **M5**: `if !flavour_balanced(..)` → `if false && !flavour_balanced(..)` | extend the test with a model whose conserved numbers are deliberately wrong (one number on the gluon alone): the walk must then drop exactly the sets whose gluon counts differ between the sides | suspected (M5 settles it) | fix |
| Z7.7 | 2 | `vibegraph-lib/src/proton.rs:1130` | untested path | the reset that drops a precompiled evaluator when `content_ordered` reorders the set's outgoing legs runs only with precompiled evaluators, which no unit test passes: every test reaches `derive_flavor_groups_with` through `derive_flavor_groups` (all `None`). L-T3's report names the gap ("Cannot see a card whose sets `content_ordered` reorders") | mutation **M12**: delete the reset loop | a test: `p p > w+ j @1` + `add process p p > j w- @2` with every set's evaluator precompiled in its enumerated order gives each group the evaluator the uncompiled path gives (same external order, same \|M\|² at probe points, bit for bit) | suspected (M12 settles it) | fix |
| Z7.8 | 1 | `vibegraph-lib/src/diagrams/mod.rs:316`, `:1507`, `:1509` | doc does not match body | `EnumerationPool::Auto`'s doc promises the pool to every walk over `AUTO_PARALLEL_COST`, but a walk without a line position (the gluon-free trials of the WEIGHTED search) stays on one thread whatever its size; and `pool != EnumerationPool::Ambient` in `if !parallel && threads > 1 && pool != EnumerationPool::Ambient` can never be false there (`Ambient` with `threads > 1` makes `parallel` true) | quoted lines 1504–1509 | say so on `Auto`; drop the redundant conjunct | checked | fix |
| Z7.9 | 4 | `generate.rs:349`, `:432`; `amplitude_probe.rs:262`, `:371`; `vibegraph-cli/src/integrate.rs:1060`; `proton.rs:1141` | missing reusable abstraction | every caller of `compile_classes` (and `complete_compiles`) first builds `let refs: Vec<&DiagramSet> = sets.iter().collect();` because the functions take `&[&DiagramSet]` | `grep -n 'Vec<&DiagramSet> = .*iter().collect()'` lists the six sites | take `&[S]` with `S: Borrow<DiagramSet> + Sync`, so a `&[DiagramSet]` and a `&[&DiagramSet]` both pass | checked | fix |
| Z7.10 | 4 | `vibegraph-lib/src/diagrams/mod.rs:415` vs `:343` | duplicated code | `enumerate_lines_in` builds a fresh one-thread pool per call for `Serial`, while `serial_pool()` keeps one for the process | quoted lines | none: the per-call pool lets concurrent `Serial` enumerations (tests, `widths.rs`, `config.rs`) run side by side; the static pool would serialise them. A change of concurrency, not hygiene | checked | leave |
| Z7.11 | 1 | `vibegraph-lib/src/hadronic.rs:740` | panic on a fallible path | `complete_compiles`'s `.expect("one compile per set without an evaluator")` | quoted | none: an invariant of the function's own construction (one compile per `None`), not a fallible input | checked | leave |
| Z7.12 | 1 | `vibegraph-lib/src/amplitude_probe.rs:250` | dead code | `compile_refusal`'s `other => HadronicError::Compile(other.to_string())` arm: `compile_class` returns only `HadronicError::Compile` | `hadronic.rs:686-694` | none: `HadronicError` is not `Clone`, and the arm keeps the function total if `compile_class` grows a refusal | checked | leave |
| Z7.13 | 1 | `vibegraph-lib/src/diagrams/mod.rs:1337` | long function | `Walk::assign` is about 108 lines, of which about 40 are the two WEIGHTED closures | quoted | none here: the closures are the filed `weighted-order-computed-three-ways` item's first site, moved by #185 from `generate_sets_inner`; the item's site name is now stale (Found) | checked | leave (filed) |
| Z7.14 | 1 | brief: "the Auto-parallel threshold (2^14) as an unexplained literal" | — | `AUTO_PARALLEL_COST` (`diagrams/mod.rs:336`) is a named constant whose doc gives the walk sizes and timings it was set from | quoted doc | — | checked | not reproduced |
| Z7.15 | 4 | brief: "a parallel path and its serial path kept as two copies" | — | the walk's assignment body exists once (`Walk::assign`); `flush`'s two arms differ only in `par_drain` against `drain`; `compile_subprocesses`, `derive_flavor_groups` and `multiplicity_groups_of` are one-line wrappers of their `_with` forms | quoted | — | checked | not reproduced |
| Z7.16 | 2 | brief: "a refusal raised inside the pool that is tested only on the serial path" | — | for enumeration, `a_refusal_inside_the_pool_is_the_serial_refusal` tests it. For compilation and the probe, no test reaches `ProbeError::Compile` or `ProbeError::NoPoints` on any path, serial or pooled; the first-in-set-order rule holds by construction (`collect` into `Vec<Result>`, then the first `Err`) | `grep -rn 'ProbeError::Compile\|ProbeError::NoPoints\|cannot place amplitude probe' vibegraph-lib vibegraph-cli` finds only the constructors | Found: no fixture makes `compile_class` refuse | checked | Found |

Backlog items naming the reviewed paths (grep of path tails and item names
over `research/kb/backlog`): `weighted-order-computed-three-ways` (Z7.13),
`weighted-search-bound-read-from-debug-log`, `integrand-methods-duplicated`,
`spin-three-halves-refusal-gaps`, `ignored-probes-unmaintained`,
`onshell-veto-lists-differing-per-line-refused`,
`feyngraph-assign-counts-allocation` (its kb staleness is L-T3's Found 4),
`amplitude-probe-skips-two-to-one`. None is a finding here except the stale
site name in the first.

**Identity oracle, chosen before the first edit.** main 8b3af5ab and this
branch's head, each built in its own worktree and target (dev profile), run
from the same directory:
- `vibegraph enumerate --svg none` at `-j 1` and `-j 4`: sha256 of
  `diagrams.json` and of `diagrams.bin.zst`, on `e+ e- > mu+ mu-` (sm),
  `p p > e+ e- j` (sm-no_b_mass) and DY+0–2j (`p p > e+ e-`, `add process`
  with 1 and 2 `j`, sm-no_b_mass).
- `vibegraph integrate --fixed-budget --neval 3000 --niter 3 --seed 5` at
  `-j 1` and `-j 4` on `p p > e+ e-` + `add process p p > e+ e- j` with
  `validation/madgraph/pp_to_llj_mlm_run_card.dat`: σ and its error as
  printed, and sha256 of the grid file.

## What changed

Commits `601ee171`, `919d823c` and `b6b0862c`. Final dispositions:
Z7.1–Z7.9 *fixed*; Z7.10–Z7.12 *leave*; Z7.13 *leave (filed)*; Z7.14 and
Z7.15 *not reproduced*; Z7.16 *Found*. Both suspected findings (Z7.6, Z7.7)
held: their mutations survived the suite before the fix.

| finding | change |
|---|---|
| Z7.1 | `generate::check_amplitude_probe` is now the one private function, taking the run's compiles; the uncalled public wrapper and `check_amplitude_probe_compiled` are gone. The name the kb pages cite is kept |
| Z7.2 | `amplitude_probe::measure` and `check` are helpers in `mod tests`. The contract (one `SetProbe` per set in set order, refusal first in set order, evaluators returned for the integrand) is documented on `measure_compiled` and `check_compiled`. `check_compiled` is `pub(crate)` |
| Z7.3 | `split_by_external_masses` is the generic function, with the contract doc and a sentence on `set_of`. The test-only wrapper and its `#[cfg_attr(not(test), allow(dead_code))]` are deleted. It stays `pub(crate)`: `proton.rs` doc-links it four times, and rustdoc does not resolve those links to a private function (first try made it private: 4 new `unresolved link` warnings, fixed in `b6b0862c`) |
| Z7.4 | `derive_flavor_groups_with` is `pub(crate)`. `multiplicity_groups_of_with`'s doc says what the evaluators must be and that a reordered set is compiled again, rather than linking it |
| Z7.5 | `ufo::conserved` is `pub(crate)` with `ConservedNumbers`, `is_empty` and `UFOModel::conserved_numbers`. `len` and `values` are deleted, and the tests read the field. The test-only `none()` became `Default::default()`, and a test-only `from_values` was added for Z7.6 |
| Z7.6 | `the_flavour_prune_drops_no_subprocess_with_a_diagram` now also enumerates `p p > e+ e- j j` under one invented number, a unit on the gluon only. It requires exactly the sets whose two sides hold as many gluons (bincode bytes and `Debug`), and that this is a non-empty proper subset |
| Z7.7 | New test `proton::tests::a_reordered_set_is_compiled_again_whatever_the_caller_compiled` (below) |
| Z7.8 | `EnumerationPool::Auto`'s doc says the WEIGHTED search's trial walks stay on one thread. `if !parallel && threads > 1 && pool != EnumerationPool::Ambient` became `if !parallel && threads > 1` |
| Z7.9 | `compile_classes` and `complete_compiles` take `&[S]` with `S: Borrow<DiagramSet> + Sync`. The `Vec<&DiagramSet>` collections at `generate.rs`, `integrate.rs` and `proton.rs` are gone (the two in `amplitude_probe.rs` went with Z7.2) |

Tests added: 1 (`a_reordered_set_is_compiled_again_whatever_the_caller_compiled`).
Tests deleted: none. One test was extended (Z7.6).

## Oracles

### Identity: main against head, bit for bit

`scratchpad/lz7/oracle.sh <binary> <tag>` runs each binary from this
worktree, dev profile. main is 8b3af5ab, built in `/home/user/wt/lz7-main`
(deleted afterwards). head is `919d823c`, built in this worktree. The 16-hex
values are sha256 prefixes.

```
--- main 8b3af5ab                                          --- head 919d823c
enumerate ee   -j1 json=9ae189bfcc5c4647 bin=fa143aa62622df43   (same)
enumerate ee   -j4 json=9ae189bfcc5c4647 bin=fa143aa62622df43   (same)
enumerate eej  -j1 json=42c2a9b7b7b2bb30 bin=1770335d9adc11f7   (same)
enumerate eej  -j4 json=42c2a9b7b7b2bb30 bin=1770335d9adc11f7   (same)
enumerate dy02 -j1 json=3ae7ffbed5cd170d bin=c849a4dc717c1f50   (same)
enumerate dy02 -j4 json=3ae7ffbed5cd170d bin=c849a4dc717c1f50   (same)
integrate mlm -j1 σ=937.0178812779731 err=28.934146865202155
                  grid=3b2479a0243231886a9a9e160e30547c552b7cf0e21f3a9b160be909c60d70cf
integrate mlm -j4 (the same three values), on main and on head
```

`b6b0862c` changes only an item's visibility, so the binary is the same.
`e+ e- > mu+ mu-`'s json hash equals L-T3's (9ae189bfcc5c).

What it cannot see:
- a card whose sets `content_ordered` reorders (the MLM card has none); the
  new unit test covers that;
- a decay card;
- `generate`'s path, which no oracle here runs. Z7.1 and Z7.9 change its
  calls only.

### Mutations

`scratchpad/lz7/mutate.py` applies one edit, or several joined with `+`, runs
the named filter under the lock, and restores the file. The logs are
`/home/user/wt/logs/lz7-mutations-pre.log` and `lz7-mutations-post.log`.
Filters are `cargo test -p vibegraph-lib --lib -- …` unless they say
otherwise.

| id | edit | guards | before fix | after fix |
|---|---|---|---|---|
| M1 | `conserved.rs`: antiparticle sign `-1` → `1` | conserved tests, prune test | caught (4 tests fail) | — |
| M2 | `balanced`: `.all` → `.any` | `balance_…`, prune test | caught (2) | — |
| M3 | null space: `v[p] = -m[row][free]` → `+` | conserved tests | caught (6) | — |
| M5 | the walk's `if !flavour_balanced(..)` → `if false && …` | prune test | **survived** (run with M12: 103 passed) | caught: `the_flavour_prune_drops_no_subprocess_with_a_diagram` fails |
| M7 | `flush`: results in reverse walk order | `a_parallel_walk…`, `a_refusal_inside_the_pool…` | caught (both fail) | — |
| M8 | a queued assignment's mirror mark lost (`p.mirrored = false`) | `a_parallel_walk…` | caught | — |
| M10 | `vibegraph-cli` `generate`: `ThreadArgs` → `ParallelArgs` (`--bin vibegraph`) | `only_the_enumerating_commands_take_parallel_diagrams` | caught | — |
| M11 | `ParallelArgs::enumeration` default `Auto` → `Serial` (`--bin vibegraph`) | `enumeration_decides_per_walk…` | caught | — |
| M12 | `derive_flavor_groups_with`: drop the reset of a reordered set's evaluator | proton and multiplicity tests | **survived** (with M5, as above) | caught: `a_reordered_set_is_compiled_again_whatever_the_caller_compiled` fails |
| M13 | `complete_compiles`: missing compiles taken in reverse | `process_lines_group_by_content_rather_than_slot_order` | caught | — |

M4 (coprime sign) and M6 (drop three-point vertices) were cut when the shared
lock made each mutation cost about ten minutes. M5 and M12 ran together
before the fix: all 103 filtered tests passed, so neither mutation is seen.
The new tests pass unmutated: 111 passed, 0 failed, 1 ignored, on
`diagrams::tests ufo::conserved proton::tests multiplicity::tests amplitude_probe`.

### What each test cannot detect

- **Z7.6, the extended prune test.** It shows that the walk consults the
  model's numbers. It cannot show that the *derived* numbers prune anything:
  with the real numbers, pruning and not pruning give the same sets by
  design. That is visible only in the walk's `-vv` count, `{flavour_kills}
  violating another conserved number`. It also cannot see a prune that is
  correct on this card but over-prunes on a card it does not reach.
- **Z7.7, the reorder test.** It cannot see an evaluator the caller compiled
  under parameters other than `evaluated`: the function trusts the caller on
  that. Its |M|² comparison is `to_bits` equality between two compiles of the
  same set in one process. It is an identity check, not a physics tolerance.
- **`TopologyAssignment`.** No mutation reaches the refusal that replaced the
  `.expect`: the cached topologies always match the line's leg count.
- **Probe and compile refusals.** No mutation reaches these either (Z7.16): no
  fixture makes `compile_class` refuse.

## Gates

On `b6b0862c` unless stated. Every cargo command ran under
`flock /home/user/wt/heavy.lock`, with `CARGO_TARGET_DIR` set to this
worktree's `target`, `CARGO_INCREMENTAL=0` and `*_DEBUG=0`.

```
$ cargo fmt --all --check                                        (clean)
$ cargo clippy --workspace --all-targets -- -D warnings          (on 919d823c)
    Finished `dev` profile [optimized] target(s) in 44.45s       rc=0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings   (on 919d823c)
    Finished `dev` profile [optimized] target(s) in 24.79s       rc=0
$ cargo doc -p vibegraph-lib --no-deps --document-private-items
warning: `vibegraph-lib` (lib doc) generated 13 warnings          (main 8b3af5ab: 13; none new)
$ cargo doc -p vibegraph --no-deps --document-private-items      0 warnings
$ flock /home/user/wt/heavy.lock /tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-hyg-enumerate lz7
head b6b0862c
FMT_OK
CLIPPY_RC=0
CLIPPY_EV_RC=0
TEST_RC=0
lines=79 passed=2353 failed=0        (ignored 28)
```

The run hid `research/refs/mg5amcnlo`, `validation/madgraph/output` and
`validation/pdf/NNPDF23_lo_as_0130_qed`, and restored them afterwards. The
suite count moved by the one added test. `pixi run --skip-deps kb-lint`
reads `1394 concepts, 480 backlog items; 0 errors, 7 warnings`, none of them on this report.

## Long-run requests

The `extended-validation` targets that read these paths (`validate_hadronic`
calls `multiplicity_groups` and `split_by_multiplicity`, and the sigma and
sample gates call `compile_subprocesses`) compile clean under the clippy
gate above. They were not run here: the brief leaves the banked gate to CI's
banked job. Request: CI's banked job on this branch's head. Expected: no
change, since the identity oracle shows σ and the grid bit for bit on the
banked MLM card.

## Found

1. **No test reaches a probe or compile refusal** (Z7.16).
   - `ProbeError::Compile` and `ProbeError::NoPoints` are built in
     `amplitude_probe.rs` and named by no test, on the serial path or across
     the pool.
   - `compile_classes`' first-in-set-order refusal holds by construction.
   - Close with a fixture that `AmplitudeEvaluator::compile` refuses (or a
     test-only failing set), asserting the variant and the set label through
     `measure_compiled` and `compile_subprocesses_with`.
2. **`weighted-order-computed-three-ways` names a moved site.** Its last
   paragraph puts the `weighted_fn` closure in `generate_sets_inner`. Since
   #185 the closure is in `Walk::assign` (`diagrams/mod.rs`), and next to it
   is a second near-copy, `constraint_fn`, which the item does not list.
3. **`multiplicity::multiplicity_groups` and `split_by_multiplicity` have no
   production caller since #185.**
   - `enumerate` now groups the sets it already holds
     (`multiplicity_groups_of`).
   - Both stay public because `tests/validate_hadronic.rs` and unit tests call
     them.
   - Close by deciding whether they are library API, or by moving those tests
     to the `_of` forms and deleting them.

## Method

| technique | share | result |
|---|---|---|
| Diff read of `ae88d176` with the L-T3 report's own "cannot see" lines | 25% | Z7.7 came straight from the report's stated blind spot; Z7.8 |
| Grep each new `pub` item and each `_with`/`_compiled` wrapper's callers across the workspace | 20% | Z7.1–Z7.5, Z7.9; the dead wrappers were invisible to the compiler because they are `pub` |
| `grep allow(dead_code)` on the diff | 5% | Z7.3 |
| Mutation batch (script, lock-serialised) | 35% | 2 survivors of 11 run, both fixed and re-run |
| Identity oracle against a main build | 15% | bit for bit on all 8 rows |

**Leads that did not hold.** All three of the brief's "likeliest findings":
- the 2^14 literal is named and measured;
- there are no parallel and serial copies;
- the pool refusal is tested for enumeration, and for compiles there is no
  test on either path.

**A first fix that was wrong.** Narrowing `split_by_external_masses` from
`pub(crate)` to private looked free, because it has one caller. Only the
private-items doc build showed four doc links to it from `proton.rs`.

## Brief corrections

- **The MLM oracle card needs `import model sm`.** On sm-no_b_mass's
  5-flavour `p`, the run card refuses the card: `final-state leg 2 (PDG 5)
  leaves incoming leg 0 (PDG 21) through a massless spacelike propagator …
  regulate it with ptb > 0`. With 4-flavour `sm`, the card ran.
- **None of the three likeliest findings held** (Method above).
- **`/tmp/claude-0/lp/verify-hidden.sh` takes no lock itself.** It was run
  under one `flock` for its whole fmt/clippy/test sequence.
- **Lock-serialised mutations cost about 10 minutes each** while three
  sessions share the lock. Mutations expected to survive can share one run:
  if a joint run is green, each mutation survives alone. Mutations expected
  to be caught need a run each, unless they hit disjoint tests.
