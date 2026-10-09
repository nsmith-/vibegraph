---
type: Backlog Item
title: The artifact reader's current-schema arm names FORMAT_VERSION
description: "read_from_path matches 9 | MULTIPLICITY_VERSION | FORMAT_VERSION; bumping FORMAT_VERSION to 12 would refuse readable version-11 files unless that arm is edited too."
area: hygiene
state: open
priority: low
closes_when: "The current-schema arm names each version it reads by its own constant (MERGED_CHANNEL_VERSION for 11), so a FORMAT_VERSION bump cannot drop one, with a test that would fail if it did."
blocked_by: []
opened: 2026-10-09
tags: [artifact, format-version, bump-hazard]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code, resource: "../../../../vibegraph-lib/src/artifact.rs", title: "artifact.rs read_from_path (~:770) and the version constants (~:97-113)"}
  - {id: note37, resource: "../../37-madevent-map-survey-and-soft-angle.md", title: "Note 37 §6.5, the earlier refusal after a bump"}
---
`IntegrateArtifact::read_from_path` (`vibegraph-lib/src/artifact.rs` ~:770)
decodes the current schema under `9 | MULTIPLICITY_VERSION | FORMAT_VERSION`.
Today `FORMAT_VERSION == MERGED_CHANNEL_VERSION == 11`. A version-12 bump that
leaves the arm alone sends version-11 files to the `UnsupportedVersion`
refusal. The failure is loud, not a misread, but it refuses files this build can
read. [Note 37 §6.5](../../37-madevent-map-survey-and-soft-angle.md) records the same pattern refusing version-7 files once already.

`a_merged_channel_is_a_version_11_file_and_older_grids_are_refused_on_one`
would catch it only while `version_for` still writes 11 for merged grids. The
fix is to name `MERGED_CHANNEL_VERSION` in the arm. Found by Phase 3 verifier
V14; confirmed against the code.
