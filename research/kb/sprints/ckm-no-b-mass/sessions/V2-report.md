---
type: Session Report
title: "V2 report: the interaction count is unitary gauge on MadGraph's side"
description: "MadGraph's 67 interactions at sm-ckm_no_b_mass are this crate's 133 vertices less the 66 with a ghost or a Goldstone; the count test goes through the same predicate diagram enumeration uses, and a Feynman-gauge build would give 109, not 133."
status: draft
tags: [model, restriction, ckm, sm, sprint-d, report, validation]
verified: [{by: claude-code/claude-sonnet-5-5, at: 2026-10-21}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-21}
sources:
  - {id: mg-import-ufo, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L1241", title: "MadGraph import_ufo.py particle and interaction import (ghost and Goldstone handling)"}
---
## The predicate

Before: one inline closure in `ufo/topo.rs::build_feyngraph_model`
(`|p| p.is_goldstone || p.ghost_number != 0`), used for the particles and for the
vertices. Nothing else in the crate excludes them (`grep` of `ghost|goldstone`
over `vibegraph-lib/src` outside `ufo/particles.rs` found only that closure;
`diagrams/check.rs` has none). So there was a single predicate, but private and
inline; a test would have had to copy it.

Now one definition, three layers:

- `Particle::is_unphysical` (`vibegraph-lib/src/ufo/particles.rs:43`), with
  `is_ghost` and `is_goldstone` accessors;
- `Vertex::is_physical` (`vibegraph-lib/src/ufo/vertices.rs:41`), the line
  diagram enumeration filters on: `ufo/topo.rs:104`
  (`if !vertex.is_physical(particles)`), and the particle filter at
  `ufo/topo.rs:84` (`particle.is_unphysical()`);
- `UFOModel::unitary_gauge_vertices` (`ufo/mod.rs:331`), which the count test
  uses (`tests/smeftsim.rs`, `restricted_interaction_counts_match_madgraph`).

The count test therefore goes through the exact predicate the topology model is
built with.

## Brief correction: the 24 / 42 / 67 split

The brief says 133 = 24 with a ghost + 42 with a Goldstone + 67 with neither.
That is exclusive, ghost first. 50 vertices carry a Goldstone; 8 of them also
carry a ghost. The test pins `(with_ghost, goldstone_without_ghost) = (24, 42)`
and the failing mutation below shows 50 directly (133 - 50 = 83).

## The hermetic pin and its mutations

`tests/sm_ckm_no_b_mass.rs::unitary_gauge_leaves_67_of_133_vertices` asserts
133 vertices, (24, 42), `unitary_gauge_vertices().count() == 67` and
`133 - 24 == 109` (the tree-level Feynman-gauge count, below). Mutating
`Particle::is_unphysical` (driver `/tmp/V2-mut.sh`, log
`.../scratchpad/G-V2-mut.log`, restored afterwards; file diff checked):

| `is_unphysical` becomes | count | result |
|---|---|---|
| `self.is_goldstone()` (ghosts counted back in) | 83 | FAIL, `left: 83 right: 67` |
| `self.is_ghost()` (Goldstones counted back in) | 109 | FAIL, `left: 109 right: 67` |
| `false` | 133 | FAIL, `left: 133 right: 67` |
| unmutated | 67 | `test result: ok` |

The same mutation also changes the topology model, so the diagram-side use is
the one being mutated, not a copy.

Not exercised: the `interned_sm` branch of
`restricted_interaction_counts_match_madgraph` (it needs an `sm` pair in
`interactions.json`, absent until the M3 banks one). Compiled and clippy-clean;
its first run is the M3's. SMEFTsim pairs take the other branch,
`vertices.len()`, unchanged.

Blind spot, recorded in the test and the count test's doc: a count matches if
one vertex is swapped for another of the same kind. It checks MadGraph's split
and pruning, not which vertices survive.

## Feynman gauge: recommend yes, as a one-off, expect 109

All lines are at MadGraph `b7687064` (raw.githubusercontent.com).

- `aloha/__init__.py:2`: `unitary_gauge = True` is the default.
- `madgraph/interface/madgraph_interface.py` L8080-8086 (`set2_gauge`, no model
  loaded): `set gauge Feynman` sets `aloha.unitary_gauge = False`.
- `models/import_ufo.py:558-559`: the model's `gauge` list is read from the UFO;
  `models/sm/__init__.py` declares `gauge = [0, 1]` (both allowed).
- `import_ufo.py:1245-1255`: Goldstones are skipped only when
  `(unitary_gauge in [1,2] and 0 in gauge) or 1 not in gauge`. With
  `unitary_gauge = False` and `1 in gauge` they are kept (and typed `goldstone`).
- `import_ufo.py:1241-1243`: `if not self.perturbation_couplings and
  particle_info.spin < 0: return`. Ghosts are never read for a tree-level model,
  **in any gauge**.
- `import_ufo.py:1779-1781`: `add_interaction` returns for any vertex with a
  particle that was not imported.
- `import_ufo.py:608-609, 776-`: the Goldstone-into-vector merge belongs to
  `FD` (`unitary_gauge == 3`) only, not Feynman.

So Feynman gauge adds the 42 ghost-free Goldstone vertices to 67: **109**. The
brief's expectation of 133 is wrong, because ghost vertices cannot be reached
without a loop model. The build would check the Goldstone vertices (this
crate's `is_goldstone` set), at the cost of one `display interactions` run with
no `generate`. It cannot check ghost vertices. Recommendation: yes, run it once
by hand; do not bank it in `interactions.json` (the extractor reads manifest
rows' build logs and the test has no gauge notion). Script and command are in
`V1-report.md` §6.

## What changed in `V1-report.md`

A new `### 6` (the brief says §5; §5 is the tests table, so the next number
was used) before `## Found`: supersedes §4's `expect 133` and §5's 133, gives
the expected 67, the exact `interactions.json` entry, the manifest note, the
Feynman script, and the resume order for the M3. §4 is not rewritten.

## Gate

`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`,
driver `/tmp/V2-gate.sh`, log `.../scratchpad/G-V2-gate.log` (full test log
`G-V2-test-full.log`):

```
$ cargo fmt --all --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 40.62s
exit 0
$ cargo test --workspace
exit 0      # 39 `test result: ok` lines, none with a failure
test unitary_gauge_leaves_67_of_133_vertices ... ok
test restricted_interaction_counts_match_madgraph ... ok
$ python3 scripts/kb.py index
wrote 1 index files
$ python3 scripts/kb.py lint
833 concepts, 292 backlog items; 0 errors, 4 warnings   # the 4 warnings predate this session
```

`pixi run --skip-deps validate` was not
run: pixi and `validation/madgraph/output` are absent here (as in V1), and the
only behaviour touched in code a banked gate runs is a refactor of the topology
filter to the same boolean, covered by the full `cargo test --workspace`
diagram sweeps.

## Found

- **The `interned_sm` branch of the count test has no run until an `sm` pair is
  banked.** A hermetic fixture (a committed one-entry stand-in or a unit test of
  the branch condition) would let the branch fail before the M3 runs; today the
  M3's first run is its only exercise. Closes with a test over a synthetic
  `interactions.json` path, or by the M3's run.
- **`restricted_interaction_counts_match_madgraph` prints and continues for an
  unloadable informational pair.** An `sm` pair whose rows are still `info`
  would hide a load failure behind `[pair] no count`. Closes when the M3 flips
  the cells, or by a check that every banked pair of an interned-SM row loads.
- **The tree-level ghost drop is gauge-independent in MadGraph, so ghost
  vertices have no MadGraph oracle at all** at this crate's SM. Their 24 (and
  8 ghost-Goldstone) vertices are uncovered by any banked reference; a loop
  model (`loop_sm`) is the only MadGraph import that lists them.
