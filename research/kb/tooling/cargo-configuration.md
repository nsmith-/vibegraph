---
type: Design
title: Cargo profiles, target directories and load-bearing feature flags
description: "dev profile at opt-level 2 keeps debug assertions; release (fat LTO) and release-debug (thin LTO, debug info) profiles; a shared CARGO_TARGET_DIR breaks worktrees; serde_json float_roundtrip."
status: draft
tags: [cargo, build, profiles, worktrees, testing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-h5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L758-L768", title: "Note 18 §5 H5 decision record: the float_roundtrip footgun"}
  - {id: n24-commit-time, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L3114-L3149", title: "Note 24 close-out: the commit-time regression"}
  - {id: n35-disk, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/35-ufo-lorentz-sprint-plan.md#L788-L841", title: "Note 35: SMEFT cross section session, disk finding"}
  - {id: n41-m2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L850-L861", title: "Note 41 §M2: a shared CARGO_TARGET_DIR does not separate worktrees"}
  - {id: n41-m2-review, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1003-L1007", title: "Note 41 §M2: a concurrent session overwrote this worktree's test binary"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L3291-L3297", title: "Note 41 §Z: CARGO_PROFILE_RELEASE_DEBUG_DEBUG cannot work"}
  - {id: cargo-toml, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/Cargo.toml", title: "Workspace Cargo.toml"}
measured:
  command: "touch vibegraph-lib/src/lib.rs && cargo fmt --check && cargo test (warm target directory), before and after [profile.dev] opt-level = 2"
---
# Cargo profiles, target directories and load-bearing feature flags

## The profiles

All in the workspace `Cargo.toml`, each with its reason in a comment there:

| profile | settings | used by |
|---|---|---|
| `dev` | `opt-level = 2`; debug assertions and overflow checks on (inherited) | `cargo test`, the hermetic CI job, local work |
| `release` | `lto = true` (fat) | shipped binaries ([release binaries](release-binaries.md)) |
| `release-debug` | `inherits = "release"`, `debug = 1`, `lto = "thin"` | every heavy validation gate (`pixi run validate`, the `validate-*` tasks), `scripts/profile.sh`, the validation-report collator |

There is no `profiling` profile; `release-debug` is the profiling profile
([profiling](profiling.md)). Thin LTO keeps most cross-crate inlining in the
hot amplitude path, which crosses crate boundaries (generated amplitude
programs in `vibegraph-lib`, driven from `vibegraph-cli` and the test
binaries), at a fraction of fat LTO's link time; that is what makes it
affordable to rebuild on every push in CI's banked job. `release` stays fat
because its binaries ship.

## Why `dev` is optimised

The default suite is physics: VEGAS adaptations, amplitude evaluations and
phase-space sweeps. At `opt-level = 0` the arithmetic, not the compiler, is
what `cargo test` waits on. Measured on one tree with a warm target directory,
running what the repository's pre-commit hook ran at the time (`cargo fmt --check && cargo test`
after touching `vibegraph-lib/src/lib.rs`):[^n24-commit-time]

| | `opt-level = 0` | `opt-level = 2` |
|---|---|---|
| whole run (touch + fmt + test) | 3m16.2s | 1m05.0s |
| lib unit tests (546) | 88.95 s | 4.45 s |
| `diagram_channel` (12) | 28.06 s | 1.89 s |
| `cli_generate` (3) | 10.56 s | 0.84 s |
| `cli_fixed_energy` (2) | 6.89 s | 0.30 s |
| test execution, all binaries | ~141 s | ~10 s |

Same 624 tests with the same assertions in both columns. These numbers are
from the tree of that time; the suite has grown since, so read them as the
size of the effect, not as current timings.

**Why not test under `release` or `release-debug` instead:** those drop every
`debug_assert!` and overflow check, silently. `dev` at `opt-level = 2` keeps
them. The cost is a full rebuild when the profile changes (3m45s measured
then); `debug = 1` on `dev` is the next lever if rebuild time matters. Guards
gated on `debug_assertions` interact with this choice; see
[CI coverage](ci-coverage.md) for the cfg rule.

The same investigation found that a pre-commit run reported at 18m46s was a
*cold* target directory, a fresh worktree paying a full dependency build, not
slow tests. A fresh worktree's first build is the cost to expect before
pre-creating several of them.

## Target directories and worktrees

**A shared `CARGO_TARGET_DIR` does not separate two worktrees of one
workspace.** Cargo hashes path packages workspace-relative, so two checkouts
of the same workspace produce the same artifact names in a shared target
directory. Two failures have come of it:[^n41-m2]

- a second worktree's build silently reused the first's binary until
  `cargo clean -p` forced a rebuild, which would have made a byte-identity
  comparison vacuous;
- a concurrent session building into the same directory overwrote another
  worktree's test binary, so a gate run silently executed a build without the
  fields under test.[^n41-m2-review]

So each worktree (each concurrent session) builds into its own target
directory, and before trusting a "same binary" or "changed binary" check,
compare the binaries' hashes. The comparison method is
[no-change claims](../validation/no-change-claims.md); dispatch rules for
worktrees are in [agent dispatch](../workflow/agent-dispatch-and-worktrees.md)
and `AGENTS.md`.

**Disk pressure is a hazard.** Debug-info targets are large: a worktree
`target/` has reached 16–17 GB mid-gate and filled a container's disk, and
three concurrent workspace builds did not fit one container.[^n35-disk] A
private target with debug info off stays near 0.7–2 GB. Dropping debug info:

- `CARGO_PROFILE_DEV_DEBUG=0` works for `dev`.
- **`release-debug` cannot be set from the environment.** Cargo reads
  `CARGO_PROFILE_RELEASE_DEBUG_DEBUG` as a key under `profile.release` and
  fails ("could not load config key `profile.release` … invalid type: Option
  value") whatever the value.[^n41-z] Use `--config
  'profile.release-debug.debug=0'` on a cargo command line, or, for
  `validation/validate.sh` and pixi tasks, which take no cargo flags, an
  untracked `.cargo/config.toml` at the worktree root
  (`[profile.release-debug]` / `debug = 0`), deleted afterwards. It changes
  debug info only, never the code a gate measures. The `extended-validation`
  skill carries the same instructions.

When cleaning up after a killed run, `pkill -f "cargo test --workspace"` is not
worktree-scoped: it kills every session's matching run on the host.

## `serde_json`'s `float_roundtrip`

`serde_json`'s default float parser is not correctly rounded: round-tripping a
`VegasGrid` through JSON moved some `f64` values by one ulp until the
`float_roundtrip` feature was enabled.[^n18-h5] Any JSON round trip that
asserts bit-for-bit equality on `f64` needs it, or an explicit tolerance.

Where it is enabled: `vibegraph-lib` (`serde_json = { version = "1.0",
features = ["float_roundtrip"] }`, a normal dependency) and
`validation-report`. `vibegraph-cli` lists plain `serde_json` as a
dev-dependency only; because `vibegraph-lib` is in the same build graph,
Cargo's feature unification gives the CLI's tests the same `serde_json` with
`float_roundtrip` on. A crate that used `serde_json` without depending on
`vibegraph-lib` would not inherit it.

## Other workspace settings

`[workspace.lints.clippy]` allows `neg_cmp_op_on_partial_ord` (the
`if !(x > 0.0)` NaN-rejecting guard is deliberate) and
`unusual_byte_groupings` (hex test seeds such as `0x5EED_1`). New
project-wide allows go there with a comment, per `AGENTS.md`.

[^n18-h5]: Note 18 §5, H5 decision record, "Footgun for downstream JSON-round-trip work".
[^n24-commit-time]: Note 24 close-out, "The commit-time regression".
[^n35-disk]: Note 35, the SMEFT cross-section session's environment finding (17 GB `target/`), and §T2's operational note (16 GB debug tree; three concurrent workspace builds).
[^n41-m2]: Note 41 §M2, byte identity at `ickkw = 0`.
[^n41-m2-review]: Note 41 §M2, the byte-identity caution extended across sessions.
[^n41-z]: Note 41 §Z, `CARGO_PROFILE_RELEASE_DEBUG_DEBUG`.
