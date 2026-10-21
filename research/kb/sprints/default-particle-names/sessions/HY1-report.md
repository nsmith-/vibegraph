---
type: Session Report
title: "HY1 report: hygiene review of the default-name table and --modelname"
description: "Review of F1's diff: the name table can no longer go stale unnoticed in debug builds, one loader entry point, the UFOModel doc corrected, narrower visibility, two tests added and one tightened; 5 mutations, 1 survivor found and closed."
status: draft
tags: [hygiene, particle-names, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
---
Session HY1 of [the sprint](../sprint.md), worktree `/home/user/wt/F`, branch
`sprintd-default-particle-names`, reviewing `63f50a6` (F1). Behaviour (which names
resolve) is unchanged.

## Findings (written before any edit)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| H1 | 3 | `ufo/mod.rs` `UFOModel::particles` (pub) beside `names` (derived) | `grep -rnE "particles\.(insert\|remove\|get_mut\|retain\|...)"` over `vibegraph-lib` and `vibegraph-cli` returns one writer after load: the test at `diagrams/resolve.rs:919`. Readers of `.particles` number about 40 (`config.rs:30`, `resolve.rs:103`, `ufo/particles.rs:350`, tests), so making it private is wide. | checked | fixed (check at the lookup) |
| H2 | 1 | `ufo/mod.rs:150` `UFOModel` doc | says every map is "keyed by the Python variable name"; `particles` is keyed by the UFO `name` | checked | fixed |
| H3 | 4 | `UFOModel::load_with_digest` / `load_with_digest_and_naming` | callers of `load_with_digest`: `load` and `vibegraph-cli/tests/cli_ufo_model.rs:37`; of `_and_naming`: `config.rs:128` | checked | fixed (one function, `naming` parameter) |
| H4 | 1 | `tests/particle_names.rs:73` | doc says "no longer resolves" (history framing) | checked | fixed |
| H5 | 3 | `ufo/names.rs` `madgraph_default_names`, `of_model`, `madgraph`, `with_naming` | `grep` outside `names.rs` finds only `ufo/mod.rs` (crate-internal) | checked | fixed (`pub(crate)`) |
| H6 | 4 | `names.rs` `resolve` / `renamed_to` | the same exact-then-ASCII-case-insensitive lookup written twice | checked | fixed (`lookup`) |
| H7 | 2 | `names::tests::mssm_calls_the_light_higgs_h1` | mutation `starts_with("mssm-")` to `starts_with("mssm")` survives: no model named `mssm<other>` tested | checked | fixed (negative case `mssm_slha2`) |
| H8 | 2 | stale table has no test | the F1 failure (`a_repeat_token...`) was found only because that test happened to insert a particle | checked | fixed with H1 |
| H9 | 1 | toy UFO `vibegraph_toy_names_UFO` | 9 files; `object_library.py` and `function_library.py` are byte-identical to the other toys' (`cmp`), the rest is the model (2 vertices, 2 couplings, 2 parameters); README says so | checked | leave: minimal, shares the helpers |
| H10 | 2 | `vendored_table_is_the_pinned_file` | uses `digest_bytes` (`sha2`, itself pinned by known-answer tests in `identity.rs`); M6 in F1 caught a byte change | checked | leave: not hand-written |
| H11 | 4 | `of_model` takes `(name, pdg)` pairs but ignores the code | shares the iterator shape with `madgraph` so `with_naming` has one input | checked | leave: a signature convenience, not worth a second iterator type |

Backlog check: `grep` over `research/kb/backlog` for `ufo/names.rs`, `/resolve.rs`,
`/parse.rs`, `/check.rs`, `/config.rs`, `ufo/mod.rs`, `/sm.rs`, `particle_names`
returns items that mention other files by those tails (`lhef/parse.rs`, `PhotonTag` in
`check.rs`); none names a site in this change.

## Fixed

Commit `fix(ufo): catch a stale particle-name table; one loader` (code and tests).

- **H1/H8.** `UFOModel::particle_names()` holds a `debug_assert!` that
  `ParticleNames::is_built_from(particles)` (the table's UFO names equal the
  particles' names, as sets). Its doc says `with_particle_naming` must be called
  after editing `particles`. Release builds pay nothing. Making `particles`
  private was not done (about 40 readers, one writer, a test).
- **H2.** Doc corrected.
- **H3.** `load_with_digest(path, restrict, naming)`; `load` passes the default;
  `config.rs` and the CLI test updated. No second entry point.
- **H4, H5, H6.** As above.

## Mutations

Driver `namesF-HY1-mut.py` (apply, run, restore, `touch`).

| # | mutation | result |
|---|---|---|
| M1 | `particle_names()` check replaced by `true` | `a_stale_table_is_caught_at_the_lookup` fails (exit 101) |
| M2 | `starts_with("mssm-")` to `starts_with("mssm")` | `mssm_calls_the_light_higgs_h1` fails; the F1 version of that test passes this mutation |
| M3 | `is_built_from` returns `true` | `the_table_knows_which_particles_it_was_built_from` and the stale test fail |
| M4 | `is_built_from` compares counts only | **survived** the first version of the unit test (added/removed cases only); a same-count replacement case (`("zz", 13)` for `("m-", 13)`) added, and it then fails |
| M5 | drop the rebuild in `a_repeat_token_that_names_a_particle_is_ambiguous` | fails, panicking at the new assert (F1's failure, now loud and located) |

The stale-table test exists only under `cfg(debug_assertions)`; in a release test
run the check is absent and the test is not compiled. A replaced particle whose
UFO name is unchanged but whose PDG code changed is not seen (the table compares
names only).

## Gate

All from `/home/user/wt/F` with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
cargo fmt --all --check                                         exit 0
cargo clippy --workspace --all-targets -- -D warnings           Finished, exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings   Finished, exit 0
cargo test --workspace      exit 0; 41 result lines: 1555 passed, 0 failed, 18 ignored
cargo doc -p vibegraph-lib --no-deps --document-private-items   8 warnings, none in names.rs, ufo/mod.rs
cargo doc -p vibegraph --no-deps --document-private-items        1 warning (cache::resolve::locate), not touched
python3 scripts/kb.py lint   831 concepts, 290 backlog items; 0 errors, 4 warnings (unrelated, as before)
```

Suite count 1553 to 1555: the two added tests,
`names::tests::the_table_knows_which_particles_it_was_built_from` and
`names::tests::a_stale_table_is_caught_at_the_lookup`. The doc warnings are in
`coupling/`, `phasespace/channel.rs`, `lhef/mod.rs`, `ufo/parameters.rs`. No banked
`extended-validation` target was run: the change touches no amplitude, colour,
coupling or diagram code, only a loader signature and the name table.

## Method

Grep for writers and readers of `particles` (about 50%), a diff read against the
checklist (30%), mutation of the new checks (20%). The suspicion that the toy UFO
duplicated the others' helpers did not hold: they are the shared library files and
the README says so. A first tightening (M4) missed its mutation, the pattern
`hygiene-review.md` already records: a "list checked both ways" needs the replace
case, not just add and remove.

## Found

- **A particle edited in place keeps the table but changes the model.** Changing a
  particle's PDG code or `name` field (not its map key) after load is not seen by
  the check. No writer exists; closing it needs `particles` private behind
  accessors (about 40 readers).
- F1's open Found items (antiname without an `.anti()` entry, labels printing UFO
  names, `--noprefix`/`--last` refused) are unchanged; none was in HY1's brief.

## Brief corrections

The brief says `ParticleNames` is built from `UFOModel::particles` and a test
inserted a particle: right, and that test is the only writer in the workspace.
