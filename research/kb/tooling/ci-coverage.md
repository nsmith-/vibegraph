---
type: Caveat
title: What a green CI run covers
description: "ci.yml runs fmt, kb-lint, featureless clippy plus hermetic cargo test, and the banked layer; extended-validation code, cfg-gated guards and the SM blob have their own rules."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [ci, testing, clippy, validation, features]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n19-v1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L79-L91", title: "Note 19 §V1: quick guards (pruned-frame guard, interned-SM check)"}
  - {id: n24-u1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2098-L2352", title: "Note 24 §U1 outcome: fresh-checkout CI simulation"}
  - {id: n29-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1682-L1721", title: "Note 29 §E.3: the release-debug contract tests"}
  - {id: n35-t2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L992-L1041", title: "Note 35 §T2: clippy findings visible only with extended-validation"}
  - {id: n35-clippy, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1462-L1493", title: "Note 35 §10.6: the clippy debt CI's lint step cannot see"}
  - {id: ci-yml, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/.github/workflows/ci.yml", title: ".github/workflows/ci.yml"}
  - {id: validate-sh, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/validate.sh", title: "validation/validate.sh"}
---
# What a green CI run covers

`.github/workflows/ci.yml` runs on pushes to `main` and on pull requests. A
green check means four jobs passed, and each covers a defined slice. The
layer definitions themselves are in
[validation layers](../validation/validation-layers.md); the binding lint rule
is in `AGENTS.md` ("Lints are gates, not advice").

| job | what runs | what it can see |
|---|---|---|
| `fmt` | `cargo fmt --all --check` | formatting, everywhere. It compiles nothing; it is the only place the rule is enforced for everyone, since agent worktrees share no git hooks |
| `kb` | `pixi run kb-lint` | knowledge-bundle conformance (frontmatter, backlog keys, generated indexes) |
| `test` | `cargo clippy --workspace --all-targets --locked -- -D warnings`, then `cargo test --locked --workspace` | the **hermetic** layer: every default-feature target |
| `banked` (needs `test`) | fetch the PDF sets and the refdata bundle, then `pixi run --skip-deps validate` | the **banked** layer: every `extended-validation` gate under `release-debug`, clippy with the feature on, the report collator |

Both compiling jobs set `RUSTFLAGS=-C target-cpu=x86-64-v3`; nothing they
build ships. The oracle layer (anything that runs MadGraph, LHAPDF or f2py)
is never in CI.

## The hermetic layer takes no runtime skips

`cargo test` with no features runs against a checkout with no submodules and
none of the fetched or MadGraph-generated data under `validation/`. It is
complete there: every test the default build registers runs and asserts. One
mechanism keeps it so. A test needing the `mg5amcnlo` submodule, a fetched PDF
set or a frozen MadGraph run carries `required-features =
["extended-validation"]` on its target in its crate's `Cargo.toml`, so it is
absent from the default build rather than deciding at runtime that it has
nothing to do.[^n24-u1] The `--lib` target has no per-test registration, so the few
library unit tests that need banked inputs use
`#[cfg(feature = "extended-validation")]` instead; integration tests never do.

The tests that need the submodule are gated targets that fail loudly when it
is absent, never soft-skipping:
`tests/ufo.rs` (`test_load_loop_sm`, `test_load_mssm`) and
`tests/sm_interned_blob.rs` panic naming `pixi run init-sm-submodule` when the
model source is missing, and the banked layer's inputs fail through
`vibegraph::validation::require`. `root_override_hook_is_transparent` runs in
the default suite against committed amplitude tables and panics if one is
missing. With the feature on, `cargo test` fail-fasts on a missing SM UFO source,
so in a worktree without the submodule content the rest of the `validate_*`
layer never runs; copy at least `models/` in before a banked run
(`AGENTS.md`, "Own the worktrees").

## Lints with the feature on

CI's `test` job lints with **no features**, which leaves every
`extended-validation` target, the gates and their shared `common/` modules,
unlinted there. A lint can therefore be green in that step and red on the code
the banked layer is made of; this happened with 21 errors over six files
(`type_complexity`, `dead_code`, `zombie_processes`, `for_kv_map`,
…).[^n35-t2][^n35-clippy] The fix is structural: `validation/validate.sh` runs, before
the gates,

```
cargo clippy --workspace --all-targets \
  --features vibegraph/extended-validation,vibegraph-lib/extended-validation \
  -- -D warnings
```

so the `banked` job catches it. It runs first because it is cheap to read and
the gates take minutes, and it is not allowed to skip the gates; the script's
exit status names gate failures first, then lint failures, then the
collator. Locally, `cargo clippy` without the feature is not the full lint:
run the command above, or `pixi run --skip-deps validate`.

One documented allow stands: `common/manifest.rs` is reached from
`vibegraph-cli` through a `#[path]` module that bypasses `common/mod.rs`, so
its dead-code warnings describe the caller; it carries `#![allow(dead_code)]`
as `report.rs` and `leshouche.rs` do.

## Guards compiled only in some profiles

Some runtime guards exist only where they cost nothing in a shipped binary:
they are compiled under `#[cfg(any(debug_assertions, feature =
"extended-validation"))]`. The pruned-evaluator frame guard is one:
`assert_partonic_cm_beams_along_z` (`helas/eval/run.rs`) is a plain `assert!`
in that cfg, rejecting a pruned evaluator fed boosted or non-2→n kinematics;
why the frame matters is
[helicity sum and pruning](../amplitudes/helicity-sum-and-pruning.md).[^n19-v1]

**A test that asserts such a guard must carry the guard's own predicate**,
not `cfg(debug_assertions)` alone:

```rust
#[test]
#[cfg(any(debug_assertions, feature = "extended-validation"))]
#[should_panic(expected = "partonic-CM kinematics")]
fn eval_m2_pruned_rejects_boosted_frame() {
```

Under `--profile release-debug` with default features the guard is compiled
out, so an ungated `should_panic` test fails there. Gating it on
`debug_assertions` alone would instead drop it from the banked configuration
(`release-debug` plus `extended-validation`), where it runs and passes:
a silent coverage loss traded for a build error.[^n29-e3]
`one_shot_validation_catches_corrupted_momentum_route` carries the same cfg.
The other `#[should_panic]` tests panic unconditionally and need none. The
rule generalises: a test asserting a guard that some profile compiles out
mirrors that guard's cfg.

## The interned SM blob

`vibegraph-lib/src/ufo/sm_assets/` holds a zstd+bincode blob of the parsed SM
UFO (`sm_parsed.bin.zst`, written by the `gen_sm_blob` dev binary) beside
every restrict card verbatim (`restrict_*.dat`, compiled in with `include_str!`). The blob, not the submodule, is the hermetic truth: it is what a bare
clone and every hermetic test read. Whether it still matches the source it was
built from is checked by `tests/sm_interned_blob.rs` (evaluated physics, and
field by field including the restrict cards byte for byte). That target is
`extended-validation`-gated and runs in the banked layer; run it alone with
`pixi run check-sm-blob-fresh` (which checks the submodule out first) after a
submodule bump or an SM UFO source edit. There is no regenerate-and-`git diff`
step in CI.

## What green does not mean

- The banked job's PDF sets and refdata bundle come from a cache keyed on the
  fetch scripts and the manifest pin; a changed pin re-fetches.
- Release builds are a separate workflow ([release binaries](release-binaries.md));
  CI never builds the musl or macOS binaries, and release acceptance runs only
  against published releases ([release acceptance](release-acceptance.md)).
- Profiles: the hermetic job tests under `dev` at `opt-level = 2` with debug
  assertions on; the banked layer runs under `release-debug`, where
  `debug_assert!`s are off. See [cargo configuration](cargo-configuration.md).

[^n19-v1]: Note 19 §V1, the pruned-frame guard and the interned-SM check as first proposed.
[^n24-u1]: Note 24 §U1, the fresh-checkout simulation of `ci.yml`.
[^n29-e3]: Note 29 §E.3, the `release-debug` contract tests.
[^n35-t2]: Note 35 §T2, where the feature-only clippy findings were first filed.
[^n35-clippy]: Note 35 §10.6, the clippy debt CI's lint step cannot see.
