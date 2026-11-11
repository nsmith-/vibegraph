---
type: Session Report
title: "J-A1 report: diagram and integrate artifacts refuse another enumeration"
description: "ENUMERATION_VERSION is recorded in both artifacts (diagram format 3, integrate format 24) and a mismatch is refused by name with the remedy. A known-answer test pins five cards' set and process-facts digests to it, and two sign mutations showed it fails on an unbumped change."
status: draft
tags: [artifacts, enumeration, diagrams, hard-errors, release, sprint-j]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-11}]
sources:
  - {id: brief, resource: "J-A1.md", title: "The session brief"}
  - {id: ia, resource: "../../../pipeline/integrate-artifact.md", title: "Integrate artifact: check 2 and its blind spot"}
  - {id: fv, resource: "../../../pipeline/artifact-format-versioning.md", title: "Artifact format versioning"}
---
Branch `sprintj-stale-artifacts`, from main `9a1349c`. All the work is in one
commit, whose hash is in the manager's handback (this report is part of that
commit).

## What changed

- **The constant.** `vibegraph::diagrams::artifact::ENUMERATION_VERSION = 1`.
  Its doc lists what it covers: every field of the banked `DiagramSet`s that
  enumeration, decay-chain stitching and feyngraph conversion compute. That
  is the subprocess list and order, legs, polarizations and `mirrored`. Per
  diagram it is the legs (particle, charge, spin, index, direction),
  propagators (particle, endpoints, momentum routing, on-shell flag),
  vertices (interaction index, flow group, ray order), the Fermi sign, the
  symmetry factor and the provenance. **It also covers the diagram
  artifact's `ProcessFacts`**, which are read off the same enumeration and
  are banked beside the sets but not covered by their digest. This goes
  beyond the brief, which named only the `DiagramSet` content. A change to
  how facts are computed would otherwise replay a stale default run card
  without a word.
- **The records.** `DiagramArtifact.enumeration_version` and
  `IntegrateArtifact.enumeration_version` are each the second field, right
  after `format_version`. Formats: diagram 2 → 3, integrate 23 → 24, and
  both floors moved to the new version. A version-2 or version-23 file
  cannot say which enumeration produced it, so it is refused like any older
  format.
- **The refusal.** On read, after the format version matches, a `(u32, u32)`
  prefix is decoded and the enumeration version compared before the body is
  decoded. The errors are `DiagramArtifactError::EnumerationVersion` ("…
  enumerate the proc card again with `vibegraph enumerate`") and
  `ArtifactError::EnumerationVersion` ("… integrate again from the proc card
  with `vibegraph integrate`"), both naming the found and expected versions.
  `integrate` reads a diagram artifact, and `generate`/`inspect` an
  integrate artifact, before any other work (`integrate.rs:713`,
  `generate.rs:360`). So the refusal comes before anything is integrated or
  generated.
- **Writers.** `DiagramArtifact::enumerate` and `integrate.rs`'s
  `IntegrateArtifact` construction record `ENUMERATION_VERSION`. An
  integration from a diagram artifact records it too, which is sound
  because reading that artifact already checked its version.
- **The known-answer test.**
  `diagrams::artifact::tests::enumeration_output_matches_the_pinned_version`
  pins `(PINNED_ENUMERATION_VERSION, diagram_set_digest, process-facts
  digest)` for five cards. Each card is enumerated through
  `DiagramArtifact::enumerate` (the artifact's own path) on the serial pool:

  | card | model | reaches |
  |---|---|---|
  | `e+ e- > e+ e-` | SM default | Fermi signs of both parities |
  | `e+ e- > z z, z > mu+ mu-` | SM default | decay chain, identical fermions across its decays (stitched sign −1) |
  | `p p > z j, z > e+ e-` | SM default | several subprocesses, mirrored sets, decay chain |
  | `u u~ > z{0} g` | SM default | a polarized leg |
  | `e+ e- > e+ e- NP<=1` | vendored SMEFTsim, `restrict_vg_c4l` | Majorana-free four-fermion contact vertex, both flow groups of one interaction |

  Besides the digests, the test asserts the set still reaches each feature:
  signs {−1, +1}, a stitched diagram with repeated outgoing particles and
  more than one decay origin, a vertex with four fermion rays in flow groups
  {0, 1}, a polarized leg, a mirrored set, and every symmetry factor equal
  to 1. On failure it prints each changed card's new digests and says to
  bump `ENUMERATION_VERSION` and re-pin the table with
  `PINNED_ENUMERATION_VERSION` in the same change. If the version is bumped
  but the table is not re-pinned, the test fails on the version assertion.
  It runs in 0.25–0.35 s (five separate-process runs: 0.35, 0.25, 0.26,
  0.29, 0.26 s).
- **Docs.** The diagram artifact's module doc, `FORMAT_VERSION`'s
  version-24 entry, `integrate-artifact.md` and `artifact-format-versioning.md`.
  - `integrate-artifact.md`: a new check 2 for the enumeration version (the
    later checks are renumbered), the five-card set with its residual blind
    spot, the field in the contents table, and version 23 → 24.
  - `artifact-format-versioning.md`: the floors 24 and 3, and how the
    enumeration version differs from a format version. The brief did not
    name this file, but its stated floors were wrong after this change.
  - `docs/src/cli/overview.md`: one user-facing sentence on the refusal.

## The brief was wrong in one place

**"A symmetry factor ≠ 1" cannot be reached at tree level.** feyngraph's
`symmetry_factor` is `vertex_symmetry × propagator_symmetry`. Propagator
symmetry needs parallel edges, so it needs a loop. Vertex symmetry counts
graph automorphisms that fix the external legs, and in a tree every internal
vertex is fixed by the set of legs beneath it, so the only such automorphism
is the identity. Stitched decay chains multiply their parts' factors
(`chain.rs:634`), which are all 1. An exploratory census of 13 SM cards
(including `g g > g g g`, `h h > h h h`, `z z > h h h`, `w+ w- > w+ w-` and
`e+ e- > z z, z > mu+ mu-`) found only 1s. Instead of covering the case, the
test asserts every factor is 1, so a change that produced another value
would fail it. The field is in the digest regardless:
`digest_sees_a_changed_diagram` changes it and sees the digest move.

## Oracles

Every result below is a command with its recorded output. All cargo runs
used `CARGO_TARGET_DIR=/home/user/cargo-target-ja1 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

**Refusal tests, before the change.** Each read path's comparison was
disabled (`if false && enumeration_version != ENUMERATION_VERSION`), and
`cargo test -p vibegraph-lib --lib -- another_enumeration_version_is_refused_by_name`
gave:

```
test artifact::tests::another_enumeration_version_is_refused_by_name ... FAILED
test diagrams::artifact::tests::another_enumeration_version_is_refused_by_name ... FAILED
thread 'artifact::tests::another_enumeration_version_is_refused_by_name' panicked at vibegraph-lib/src/artifact.rs:1147:18:
thread 'diagrams::artifact::tests::another_enumeration_version_is_refused_by_name' panicked at vibegraph-lib/src/diagrams/artifact.rs:634:18:
   2: core::option::expect_failed
test result: FAILED. 0 passed; 2 failed; ...
```

The panics are the `expect_err` / `.err().expect(...)` lines, "another
enumeration version must be refused": with the check off, the file was read.
After the comparison was restored (files copied back, `grep -c "if false"`
→ 0 in both), both pass.

**Mutation 1: drop the line sign from conversion.** In `diagram.rs:404`,
`diagram.sign *= diagram.fermion_line_sign(model)` became
`*= -diagram.fermion_line_sign(model)`, with no version bump. Running
`cargo test -p vibegraph-lib --lib -- --exact diagrams::artifact::tests::enumeration_output_matches_the_pinned_version`
gave:

```
the enumeration output changed at ENUMERATION_VERSION 1:
"generate e+ e- > e+ e-\n" (Sm): sets ab415872…3c42 (pinned df070864…f28f), facts d6b775e9…d217 (pinned d6b775e9…d217)
"generate u u~ > z{0} g\n" (Sm): sets a085d42f…8925 (pinned 52839bbd…e9ff), facts 54295cbb…007e (pinned 54295cbb…007e)
"generate e+ e- > e+ e- NP<=1\n" (SmeftFourLepton): sets a03aa814…9d66 (pinned 96a7f460…6eb3), facts cc16e6d7…b909 (pinned cc16e6d7…b909)
If the change is intended, bump ENUMERATION_VERSION and record every digest printed here …
test result: FAILED. 0 passed; 1 failed
```

The mutation was reverted with `git checkout`. The two decay-chain cards
did not move: stitched diagrams get their sign recomputed from the graph in
`chain::relabelled`, not from conversion.

**Mutation 2: drop the line sign from stitched diagrams.** In
`chain.rs:689`, `d.sign = d.fermion_pairing_sign(model) * d.fermion_line_sign(model)`
became `d.sign = d.fermion_pairing_sign(model)`. The same command gave:

```
the enumeration output changed at ENUMERATION_VERSION 1:
"generate e+ e- > z z, z > mu+ mu-\n" (Sm): sets ef004cb5…017d (pinned 99a0d995…ee4a), facts 8e7ac86a…04bd5 (pinned 8e7ac86a…04bd5)
test result: FAILED. 0 passed; 1 failed
```

The mutation was reverted with `git checkout`. Only the identical-fermion
chain card sees this one; `p p > z j, z > e+ e-` has line sign +1
throughout. So a single card per feature matters, and dropping the `z z`
card would have hidden this mutation.

**Determinism.** The KAT binary was run five times as separate processes
(each with fresh `HashMap` random states), and all five passed.
`digest_is_independent_of_the_enumeration_pool` (existing) covers 1 vs 4
threads. Every type reachable from `DiagramSet` and `ProcessFacts` uses
ordered collections or `Vec`s, the encoding is bincode's fixed-width
little-endian, and nothing in the sets is a float. So the digest should not
depend on the platform, but **this was not measured off Linux x86-64**. CI
runs only on `ubuntu-latest`, so a macOS developer would be the first to see
a platform-dependent order.

**Gates**, all run on the committed tree:

| command | result |
|---|---|
| `cargo fmt --all --check` | rc 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | `Finished`, rc 0 |
| `cargo test --workspace` | 58 `test result: ok` lines, 2072 passed, 0 failed, 26 ignored, rc 0 |
| `python3 scripts/kb.py lint` | `1259 concepts, 434 backlog items; 0 errors, 5 warnings` (the five warnings were there before this change) |

`pixi run validate` was not run: no amplitude, colour, coupling or
enumeration code changed. The only change near enumeration is a field
recorded beside its output.

## What each test cannot detect

- `another_enumeration_version_is_refused_by_name` (both artifacts): an
  enumeration change made without bumping the version. The KAT is what sees
  that.
- `a_version_2_artifact_is_refused_by_name` and
  `a_version_23_artifact_is_refused`: they build the older shape by cutting
  four bytes out of the current encoding, and assert first that those bytes
  are the enumeration version. A field reordered elsewhere in the struct
  would not be modelled.
- `enumeration_output_matches_the_pinned_version`: a change that moves only
  cards outside the five. That is the residual blind spot named in
  `integrate-artifact.md`. Examples would be a Majorana diagram,
  multi-jet colour-ordered enumeration at higher multiplicity, `$`/`/`
  filters, coupling-order selection beyond `NP<=1`, three-body decay chains
  and nested decays, and models other than SM-default and SMEFTsim-c4l. It
  also moves on a change to the interned SM blob or the vendored SMEFTsim,
  because particle and vertex ids are indices into the model. Bumping the
  version there is harmless, and the artifacts' model digest refuses those
  files anyway.

## Found

1. **Worktrees must not share a cargo target dir.** All three sessions were
   told to use `CARGO_TARGET_DIR=/home/user/cargo-target`, and with it a
   session can run another worktree's test binary. Evidence: after I added
   three tests in `sprintj-a1`,
   `cargo test -p vibegraph-lib --lib diagrams::artifact` printed
   `Finished … in 0.15s` and ran 9 tests, and
   `/home/user/cargo-target/debug/deps/vibegraph-ea62a6b70b92494f --list`
   did not contain the new tests. Cause: workspace members get the same
   metadata hash in every worktree, and dep-info paths are relative to the
   workspace. A sibling's later build leaves the output newer than this
   tree's sources, so cargo reports it fresh. Fix: an AGENTS.md sprint-ops
   note saying each worktree gets its own `CARGO_TARGET_DIR`, and that any
   gate result taken from a shared dir must be re-run. The manager has
   already moved this sprint's sessions to their own dirs.
2. **The KAT has never run off Linux.** CI runs only on `ubuntu-latest`. To
   close it, run `enumeration_output_matches_the_pinned_version` once on
   macOS/aarch64 and record the result in the KAT's doc, or add a macOS
   test job.
3. **`integrate`'s JSON result and `inspect` do not report the enumeration
   version.** Both report `format_version` (`output.rs:290`,
   `inspect.rs:150`). To close it, add the field to the result schema
   (which bumps its own version) and to `inspect`'s header line.
4. **Some KAT coverage is missing.** Majorana fermions are refused today
   (`ConvertError::MajoranaFermion`), so nothing needs them yet. But nested
   decays (`t > w+ b, w+ > e+ ve`) and an `$` veto card are reachable now
   and are not in the set. To close it, add one nested-decay card and one
   `$` card to `KNOWN_ANSWERS`, which costs about 50 ms each.
