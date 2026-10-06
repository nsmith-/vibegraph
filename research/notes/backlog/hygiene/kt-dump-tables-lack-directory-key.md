---
type: Backlog Item
title: K2 clustering-dump tables are not keyed by process directory
description: The kT clustering dump merges per-directory tables across subprocess directories, which forces candidate-forest disambiguation and makes NQCD collide.
area: hygiene
state: open
priority: low
closes_when: gen_kt_cluster_dumps.py writes the per-directory tables (RUN, CONST, CONST2, NQCD, MAP, PDG, RES, IFOR) keyed by process-directory name, the dumps are re-extracted, and validate_kt_cluster.rs drops its candidate-forest disambiguation.
blocked_by: []
opened: 2026-08-06
tags: [kt-clustering, oracle, dump-format, rebank]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L545-L551", title: "TODO.md entry T043"}
---
`validation/madgraph/gen_kt_cluster_dumps.py` already reads each shard's `SHARD`
record and attaches `directory` to every per-event object. The `PER_DIRECTORY`
tables (line 199) are still stored per tag and de-duplicated by text, so a run
whose bank spans several subprocess directories merges their tables.
`pp_to_bb_qcd2` and `pp_to_llj{,_qcd2_qed2}` hit this, and `NQCD` collides
outright: `(this_config 1, config 1)` is 2 in one directory and 0 in the other.

The consumer works around it (`vibegraph-lib/tests/validate_kt_cluster.rs`,
`DirectoryForests`). It separates forests by `IFOR` row length (`8 + maxsproc`),
re-derives `nqcd` from colour, and consults the event's candidate list for 7
flavour assignments. Keying the tables by directory removes that whole class of
exception. This matters only at a re-extraction or re-bank. Detail:
[note 28 §K3.4](../../28-kt-spine-feature-sprint-plan.md).
