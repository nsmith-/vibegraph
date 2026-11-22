---
type: Session Report
title: "L-R2 report: artifact format 25 records the writer version and still reads 24"
description: "The integrate artifact gains a trailing writer_version (the crate version); format 24 files load unchanged with a null writer; integrate and inspect report writer_version, format_version and enumeration_version."
status: draft
tags: [artifact, versioning, ldjson, cli, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-22}
---
Branch `sprintl-reports`, from 4df286e.

## What changed

- `vibegraph-lib/src/artifact.rs`: `FORMAT_VERSION` 25; `OLDEST_READABLE_VERSION`
  stays 24. `IntegrateArtifact` gains a last field `writer_version:
  Option<String>`, written as `WRITER_VERSION = CARGO_PKG_VERSION`. A format-24
  payload is the format-25 layout without that last field, so the reader appends
  the absent-`Option` tag byte and decodes; the file keeps its own
  `format_version` 24 and `writer_version` `None`. `write_to_path` writes only
  the current format (an older version's bytes lack the trailing field).
  `enumeration_version` was already in the artifact (second field).
- Version string: the crate version only. The format-25 doc line gives the
  reason: a commit hash makes the bytes of two artifacts differ between commits
  with identical physics, which defeats comparing artifact digests bit for bit
  between a branch and main. `integrate` still reports the full describe string
  as `vibegraph_version`.
- Results: `integrate` and `inspect` carry `writer_version` (null for format
  24), `format_version`, `enumeration_version`. Refusal `reason` is unchanged
  (`format_version` now reports `oldest` 24, `expected` 25). Docs updated in
  `docs/src/cli/overview.md`.

## Digest audit

Searched `.rs`, `.sh`, `.py`, `.toml`, workflows and the repo's committed files
for artifact bytes or digests. There are no committed artifact files. The
digests that involve artifact bytes: `artifact_sha256` / the `<grid sha256>`
attribute that `generate` writes into the event file header (SHA-256 of the
compressed artifact file; tests compute it from the artifact they wrote,
`cli_generate.rs`, `cli_integrate.rs`), and any session's main-vs-branch grid
sha256 comparison. These move exactly once, at the 24 to 25 bump, because the
payload gains the writer field and the header changes; two builds of the same
release then write identical bytes, and between commits of one release they
stay equal. The model digest, `diagram_set_digest` and `channel_map_digest` are
computed over the model, diagrams and channel keys, not the artifact bytes, and
do not move. No release tooling pins artifact bytes. Sampler, phase-space,
amplitude and enumeration code is untouched.

## Oracles

Environment as L-R1, `-j 2/3`.
- `cargo test -p vibegraph-lib --lib artifact`: 38 passed. The stale-version
  tests (13 to 23, and one newer) now expect `expected: 25`, `oldest: 24`.
- `cargo test -p vibegraph --test cli_artifact_provenance` (in the workspace run):
  - a fresh artifact is format 25 and its `writer_version` equals
    `CARGO_PKG_VERSION`, reported by both `integrate` and `inspect`;
  - a format-24 fixture, made by removing the writer field from a fresh artifact
    and stamping 24, loads: same σ bits, `process`, `writer_version` null,
    `inspect` reports format 24 and the same σ, and `generate` accepts it;
  - format 26 and format 23 are refused with `reason.kind = format_version`.
- `cargo fmt --all --check` clean; `cargo clippy -j 2 --workspace --all-targets
  -- -D warnings` rc=0; `cargo test -j 3 --workspace --no-fail-fast` rc=0, 71
  `test result: ok` lines, no FAILED.
- σ: the fixture test compares the format-24 file's σ bit for bit with the
  original and `inspect`'s `sigma` with `integrate`'s; no sampling code changed,
  so no cross section moves. `pixi run validate` was not run (CI's banked job).

Cannot detect: the format-24 fixture is a patched format-25 file, not bytes
written by a real 24 binary; the layouts are identical by construction (the field
is last and the reader appends the absent tag), and an old binary's file would
have to differ in some other field to break it, which no change between 24 and
25 made. The collider-energy reason remains unit-tested only.

## Found

- None new. The L-R1 Found item on the writer version is closed by this session.
