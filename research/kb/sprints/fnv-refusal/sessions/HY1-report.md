---
type: Session Report
title: "HY1 report: hygiene review of the fermion-flow refusal"
description: "Review of F1's diff: five findings fixed with mutation evidence (a swapped particle/antiparticle kind, the veto-order, the charge convention doc, the error prefix, a toy's MadGraph-only order), two sites no mutation reaches, three Found."
status: draft
tags: [hygiene, majorana, fermion-flow, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
---
Commit `1727b8d` on `sprintd-fnv-refusal`. Pixi absent; commands run directly with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

## Findings (written before any edit)

| id | pt | site | evidence | conf | disposition |
|---|---|---|---|---|---|
| H1 | 1 | `diagrams/diagram.rs` `check_fermion_flow` doc | convention ("all-incoming, by PDG sign", the evaluator flips final-state legs back) was spread over a long sentence with `Leg::charge` as an aside, the vertex check's PDG-sign reading unstated | checked | fixed |
| H2 | 2 | `cli_fermion_flow.rs` FNV test, `diagram.rs` `kind:` | mutation `kind` swapped (`"particles"`↔`"antiparticles"`): old tests pass (0 failed) because the needle was `"on one fermion line as two incoming"` | checked | fixed |
| H3 | 2 | `diagrams/mod.rs` call position | mutation: check moved before `retain`: old tests pass; no test uses a veto | checked | fixed |
| H4 | 2 | debug assertion `!=` | mutation `!=`→`==`: caught by `dirac_only_processes_of_the_same_models_integrate` only (the assertion is `debug_assert!`; dev and test profiles keep it on, root `Cargo.toml` L28-30). One test, one path | checked | leave (pinned; see Mutations) |
| H5 | 3/1 | `DiagramError::Convert` display | CLI prints "diagram conversion error: particle 'nt' is a Majorana fermion…"; the refusal is not a conversion failure, same for `CustomPropagator`. No test or doc matches the prefix (`grep -rn "diagram conversion error"` hits only F1's report) | checked | fixed (prefix dropped; `ConvertError` doc names the refusals) |
| H6 | 1 | both toy `coupling_orders.py` | `QCD` declared "for MadGraph's WEIGHTED search"; these fixtures never go through MadGraph and no coupling carries QCD | checked | fixed (removed; tests pass) |
| H7 | 1 | toy UFOs minimality | each has only boilerplate the loader needs plus the vertices a test reaches; `object_library.py` and `function_library.py` are byte-identical to the two existing toys (md5) | checked | leave |
| H8 | 4 | CLI test helpers | `integrate`/`assert_*` in `cli_fermion_flow.rs` duplicate the proc-card/run-card writing of `cli_ufo_model.rs`/`cli_hard_errors.rs` | suspected→checked below | Found |
| H9 | bundle | `toy-ufo-models.md` | lacks both fixtures | checked | fixed |
| H10 | 3 | `check_fermion_flow` `pub(crate)`, new `ConvertError` variants | one caller outside the module (`diagrams/mod.rs`); variants are `pub` on an already-`pub` enum, `role`/`kind` are `&'static str` | checked | leave |

Backlog check: no filed item names `diagrams/diagram.rs`, `layout.rs`, `kernel.rs` sites in the diff
(`grep -rlE` over `research/kb/backlog`; the hits name other lines of `diagrams/mod.rs`).

H8: the helpers differ in what they assert and in the ufo-dir flag; `cli_ufo_model.rs` and
`cli_hard_errors.rs` each carry their own copy of the card writing, so the repetition is
cross-file (three copies), which routes to Found, not a local fix.

## Fixed

- **H1, H5, H6, H9 (1727b8d):** doc rewrite, `#[error("{0}")]`, QCD removal, bundle entry.
- **H2:** FNV test asserts `vertex 'V_atbtdq' joins 'at' and 'bt'` and `two incoming particles`;
  new `a_vertex_pairing_two_antiparticles_is_refused` (`at~ bt~ > at~ bt~`:
  `V_atbtdqc`, `two incoming antiparticles`).
- **H3:** new `a_refused_diagram_removed_by_an_s_channel_veto_does_not_refuse`:
  `at bt > at bt $$ dq` integrates (σ = 0.844543 ± 0.006092 pb, equal to `/ dq`; a single `$ dq`
  keeps the diagram and is still refused, so `$$` is the form that removes it).
- Suite: 1542 → 1544, the two tests above.

## Mutations (`/…/scratchpad/fnvD-HY1-mut.py`, `cargo test -p vibegraph --test cli_fermion_flow`)

| mutation | old tests | new tests |
|---|---|---|
| pdg compare `==`→`!=` | 2 fail | (old suffices) |
| `kind` swapped | **0 fail** | 2 fail (FNV, antiparticle) |
| check moved before `retain` | **0 fail** | 1 fail (veto test) |
| majorana filter without spin test | 1 fail (Dirac control) | – |
| majorana filter without `is_self_conjugate` | 4 fail | – |
| legs not checked / props not checked | 1 fail each (the matching Majorana test) | – |
| role label swapped | 1 fail | – |
| debug assertion `!=`→`==` | 1 fail (Dirac control) | – |
| spin filter dropped (`|| false`) | 0 fail | **no mutation reaches it**: only fermion slots are paired, so the filter never rejects a pair |
| `b <= a`→`b < a` | 0 fail | **no mutation reaches it**: only an unpaired fermion slot has `partner(a) == a` and no toy model writes one (F1's Found) |

Applied temporarily and reverted (`touch`ed); the committed tree is the tested one.

## Gate (worktree `/home/user/wt/D`)

```
fmt exit 0
cargo clippy --workspace --all-targets -- -D warnings            exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings  exit 0
cargo doc -p vibegraph-lib --no-deps --document-private-items    exit 0, 8 warnings, none in diagram.rs or mod.rs (all pre-existing: configs.rs, channel.rs, alphas.rs, scales.rs, parameters.rs, lhef/mod.rs)
cargo test --workspace                                            exit 0; 41 suites passed 1544 failed 0 ignored 16
cli_fermion_flow: 6 passed
python3 scripts/kb.py lint -> 825 concepts, 289 backlog items; 0 errors, 4 warnings (index regenerated: research/kb/validation/index.md)
```

No banked extended-validation target was run: the change refuses diagrams and touches no amplitude.

## Found

- **CLI test helpers repeated across three files** (`cli_fermion_flow.rs`, `cli_ufo_model.rs`,
  `cli_hard_errors.rs`): each writes the proc/run cards and spawns `vibegraph integrate --ufo-dir`.
  Close with a shared `tests/common` module.
- **The refusal toys' `object_library.py` and `function_library.py` are the fourth byte-identical copies**
  (`vibegraph_toy_UFO`, `_color_`, `_fnv_`, `_majorana_`). A shared copy would need the UFO loader to
  support a path outside the model directory; not local.
- **No unit test calls `check_fermion_flow` directly**; the refusal is pinned only through the CLI
  (a library-level test on the two fixture models would match the `ConvertError` variant without a
  process spawn). Close with a `vibegraph-lib/tests` case that loads the two UFOs.
