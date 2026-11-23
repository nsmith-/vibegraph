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
