---
type: Backlog Item
title: Process, model and artifact doc comments contradict the code
description: "Docs in diagrams/, ufo/ and artifact.rs say s-channel restrictions are refused, unknown parameters return 0, raw colour strings have no summed indices, and upgrades normalise their version."
area: hygiene
state: open
priority: low
closes_when: "Every site listed in the body describes the current code."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, process-grammar, ufo, artifact, hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: check, resource: "../../../../vibegraph-lib/src/diagrams/check.rs", title: "diagrams/check.rs module doc and SupportedProcess (~:78-96)"}
---
Each site was checked against the code on 2026-10-09 (paths under `vibegraph-lib/src/` unless stated):

- `diagrams/selector.rs:26-29` (`build_selector`): the s-channel restrictions "are refused before enumeration". They are supported and filter converted diagrams (`diagrams/schannel.rs`).
- `diagrams/check.rs:6-8`: downstream code "cannot read a `$` restriction or a decay chain". `SupportedProcess` carries `forbidden_onshell_s_channels` and `decays` (~:78-96).
- `diagrams/check.rs:19-24`, the module table: `ChainOrders`, `PropagatorPolarization`, `SquaredOrder` and `WeightedOrder` say "(not planned)". Each has an open backlog item, and squared orders are in scope by user decision (2026-10-09).
- `ufo/expr.rs:91` (`eval`): "Unknown parameter references panic in debug builds and return 0 in release". It panics in every build (~:97-99).
- `ufo/color.rs:5-8`: negative indices are "never present in a raw UFO model file". The same doc's example `f(-1,1,2)*f(3,4,-1)` is the SM four-gluon vertex, and SMEFTsim writes `T(-1,i,j)*T(-1,k,l)`.
- `artifact.rs:55` (`FORMAT_VERSION`, version-7 paragraph): versions 3 through 5 "normalise to `FORMAT_VERSION`" on upgrade. Every upgrade keeps the file's own version, and the round-trip tests assert it (~:1259, ~:1363).
- `artifact.rs:344` (`IntegrateArtifact::maps`): "a file older than version 8 reads back `MapChoices::LEGACY`". Maps arrived in version 9, and versions 6 to 8 upgrade to `LEGACY`, so "older than version 9".
- `validation/ufo/README.md:41-43` (repo root): "The other nine SMEFTsim variants" names five.

The FORMAT_VERSION bump hazard next to the second site is a separate item,
[artifact-reader-arm-names-format-version](artifact-reader-arm-names-format-version.md).
Found by drafters D13 and D14 and verifiers V13 and V14.
