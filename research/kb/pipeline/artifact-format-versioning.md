---
type: Design Decision
title: Artifact format versioning
description: "How every persisted vibegraph artifact is versioned: what bumps the version, how a reader decodes, upgrades or refuses an old file by name, and why the recorded version survives an upgrade."
status: draft
tags: [artifact, versioning, serialization, cli]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: artifact-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/artifact.rs#L20-L118", title: "artifact.rs: FORMAT_VERSION history and version constants"}
  - {id: artifact-read, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/artifact.rs#L688-L795", title: "artifact.rs: version_for, refuse_unmerged_grids, read_from_path"}
  - {id: n29-b9, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5198-L5223", title: "Note 29 §B.9: a meaning change with no schema change"}
  - {id: n29-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5650-L5661", title: "Note 29 chain B results: upgrades must keep the recorded version"}
  - {id: n24-fv4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1617-L1648", title: "Note 24: version dispatch, the fv3 reader and the refusal test"}
  - {id: n37-defects, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L542-L551", title: "Note 37 §6.5: two version-guard defects"}
---
vibegraph persists two artifacts: the `vibegraph integrate` grid file
([the integrate artifact](integrate-artifact.md)), and the diagram artifact
`vibegraph enumerate` writes and `integrate` reads
([diagrams.json](diagrams-json.md)). The compiled-program cache would be a
third. The rules here apply to
every such file. Each artifact's own fields and history belong in its own
concept.

## The rules

1. **One version number per artifact, read first.** The encoding is bincode,
   zstd-compressed. The payload's first field is `format_version`, decoded on
   its own (`VersionHeader`) before the body. A reader then dispatches on it:
   decode directly, decode through an older version's struct and upgrade, or
   refuse by name. It never misreads a file into whatever the current field
   order happens to accept. Positional bincode would do exactly that, silently,
   if the version were not checked first.
2. **Bump on a shape change.** Adding, removing or retyping a field bumps the
   version. So does adding an enum variant an older reader cannot decode.
3. **Bump on a meaning change too, even with no shape change.** If a field
   keeps its type but a reader would misinterpret the old value, the version
   moves, and the reader that cares refuses the old value where it matters.
   The integrate artifact's version 7 is the worked case. The schema did not
   change, but `sigma_pb` on a clustering-scale run started to mean a different
   rule's cross section. An old artifact replayed by `generate` would have
   written the old rule's σ as `XSECUP` on a new-rule sample, a boundary a card
   reaches silently. So `generate` refuses a `< 7` artifact only on a card that
   selects the clustering scale. Fixed-scale artifacts keep working.[^n29-b9]
   For the integrate artifact's channel maps, the check is automatic from
   version 15: the artifact banks `channel_map_digest`, a SHA-256 over every
   channel's `DiagramChannel::map_key` in channel order, and `generate`
   refuses an artifact whose digest differs from the one its rebuilt channels
   give. A change to which map a channel gets (topology, poles, floors,
   transfer bounds) therefore refuses old artifacts without a bump. The key
   encodes parameters, not sampling code, so a change to how a map with an
   unchanged key draws or weighs a point still bumps by hand.
4. **Upgrades keep the file's own version.** An upgraded struct carries the
   version the file was written at, not the current one. A guard downstream
   (`format_version < SCALE_DRAW_VERSION`, `< MULTIPLICITY_VERSION`,
   `< MERGED_CHANNEL_VERSION`) can only see an old artifact if the version
   survives the upgrade. Normalising it on read once blinded the version-7
   guard to exactly the files it exists for. The upgrade tests for versions 3,
   4 and 5, and the round trip over 6–8, assert the preserved version.[^n29-b4]
5. **Upgrade only what the old file knows.** An upgrade fills a new field with
   what the old writer must have meant, or with an explicit "not recorded"
   (`None`). It never guesses. Example: only two writers could produce a
   version-3 integrate file, so a lone channel upgrades to `Whole` and several
   to per-diagram channels; none can become a hadronic key.[^n24-fv4]
6. **Write the oldest version that holds the content.** When a new version only
   adds a variant, the writer records the oldest version whose schema holds
   every key it actually banks (`IntegrateArtifact::version_for`). An artifact
   that uses none of the new keys is byte for byte what the older writer
   produced, and older readers still read it. One that does use them is refused
   by an older reader *by version*, rather than failing on an unknown enum tag.
7. **Name the threshold, do not compare against the current version.** A guard
   compares against the constant for the version that introduced the meaning
   it checks (`SCALE_DRAW_VERSION = 7`, `MULTIPLICITY_VERSION = 10`,
   `MERGED_CHANNEL_VERSION = 11`), never against `FORMAT_VERSION`. Comparing
   against the current version once made a version-7 artifact read as
   pre-draw. A reader that matched `FORMAT_VERSION | 6` once refused version 7
   after a bump to 8. The dispatch must list every readable version
   explicitly.[^n37-defects]
8. **Refuse with the remedy.** An unreadable version returns
   `UnsupportedVersion { found, oldest, expected }`, whose message names the
   readable range and says to regenerate with `vibegraph integrate`. A
   meaning-change refusal names the rule that changed, never a plan item.
9. **Pin the version tests to old shapes.** A refusal test must write an
   unreadable version's real shape. A test that wrote `FORMAT_VERSION - 1`
   silently became a test of the upgrade path once that version was readable
   again. The refusal test writes version 2's shape, where every field after
   `process` sits one slot early: the exact payload a positional misread would
   consume.[^n24-fv4]

Each bump is recorded in the version constant's doc comment, one paragraph per
version saying what changed and why. That doc is the schema history; this
concept does not copy it.

## Keeping old readers

The oldest readable version is a deliberate floor (`OLDEST_READABLE_VERSION`).
A version that shares a schema with a later one costs nothing to keep. One
with its own struct costs a frozen copy in a `vN` module and an `upgrade`. No
committed file or test consumes a banked integrate artifact: every test writes
its own with `vibegraph integrate`. So raising the floor breaks only users'
files, and they are told to regenerate.

The integrate artifact's floor is its current version, 24. No file older
than 13 records a proc card, so `generate`, which takes none, has nothing to
replay from one; 13 records no PDF checksums; 14 holds grids trained under
channel maps that 15 builds differently for the same keys, and no channel-map
digest that could tell; 21 embeds no model, diagrams or amplitude probe; 22
banks no configuration share or channel membership, which only integrating
again can measure, so reading it with empty shares would present a ranking
that was never measured (`a_version_22_artifact_is_refused` writes its real
shape); 23 does not record which enumeration produced its diagram sets
(`a_version_23_artifact_is_refused`). The `v3`–`v9` readers, `version_for` and the threshold constants
went with the versions they served, so rules 5–7 have no live instance in it;
the examples above are their history.

The diagram artifact (`DiagramArtifact`, `diagrams/artifact.rs`) follows the
same rules with a version sequence of its own. Its floor is its current
version, 3. Version 3 added the enumeration version, and a version-2 file is
refused for the same reason as an integrate artifact at 23
(`a_version_2_artifact_is_refused_by_name`). Version 2 added the process facts MadGraph's default run card
reads (`ProcessFacts`), so `integrate` can build that card without
enumerating. A version-1 file has none, and they cannot be recovered from its
sets: a decay-chain line's facts come from its core, enumerated again without
the decays, and the sets do not say which line produced them. Upgrading would
mean enumerating, which is what the artifact exists to avoid, so version 1 is
refused by name with the remedy, re-run `vibegraph enumerate`. Version 1 was
the first, and was current only briefly. The refusal test writes version 1's
real shape.

**The enumeration version is not a format version.** Both artifacts also
record `ENUMERATION_VERSION`, the version of what enumeration computes, and
refuse a file that records another
([integrate artifact](integrate-artifact.md), check 2). A format version
says how to decode the bytes; the enumeration version says whether the
decoded diagrams are what this build would enumerate, which no schema change
marks. Bumping it changes no struct, so no file needs an upgrade path: it is
refused with the remedy to enumerate or integrate again.

The interned SM model blob (`ufo/sm_assets/sm_parsed.bin.zst`) is a build input,
not a run artifact, and carries no version. It is regenerated from the pinned
submodule by `gen_sm_blob`, and the test `interned_blob_matches_submodule_exactly`
guards it. Artifacts that embed a model refer to it by
[digest](../model/model-identity-digest.md), so a changed blob refuses every
artifact written against the old one. That is the intended behaviour.

[^n29-b9]: Note 29 §B.9, the recommendation that became version 7.
[^n29-b4]: Note 29 chain B results, "B-3 and B-4".
[^n24-fv4]: Note 24, P3 outcome, "Artifact schema: fv3 → fv4, with the fv3 reader kept".
[^n37-defects]: Note 37 §6.5.
