---
type: Sprint Record
title: OKF migration of the research notes
description: "The migration of the research notes and TODO.md into the research/kb OKF bundle, 2026-10-06 to 10-09: what each phase did and what it produced."
closed: 2026-10-09
status: stable
tags: [okf, migration, knowledge-bundle, backlog, sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n42-phaseb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L366-L401", title: "Note 42 §8 Phase B (with As executed)"}
  - {id: n42-phase0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L403-L435", title: "Note 42 §8 Phase 0 (with As executed)"}
  - {id: n42-phase1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L437-L470", title: "Note 42 §8 Phase 1 (with As executed)"}
  - {id: n42-phase2-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L472-L512", title: "Note 42 §8 Phases 2–5 and Trial"}
  - {id: n42-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L514-L535", title: "Note 42 §9: decisions (user, 2026-10-05)"}
  - {id: n42-risks, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L537-L555", title: "Note 42 §10: risks"}
  - {id: n42-start, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L51-L74", title: "Note 42 §2: starting point"}
---
The migration turned the numbered notes in `research/notes/` and the backlog
that `TODO.md` held into the topic-organised bundle under `research/kb/`
([the knowledge bundle](../workflow/knowledge-bundle.md)). It ran from
2026-10-06 to 2026-10-09 and closed with Phase 4. Phase 5 was dropped.

**Ground rules (user, 2026-10-05):**[^n42-decisions]

- No other work runs while the migration does, so new notes need no interim
  naming scheme and note numbers cannot collide mid-migration.
- Phases 2 and 3 fan out to about 15–20 parallel agents.
- The `measured:` convention ([measurement provenance](../workflow/measurement-provenance.md))
  starts after the migration. Drafted numbers carry only the host and commit
  their notes gave.

## Starting point

At planning time there were 47 numbered notes (about 38k lines in 1,211
sections, about 60% of them sprint plans, mostly dated amendments). There were
"note NN §x" citations to keep resolvable: 109 in `TODO.md`, about 16 in code,
validation and scripts, one each in `AGENTS.md` and `.agents/`, and 362 between
notes. `TODO.md` had 1,072 lines and about 120 entries; it was the hottest file,
touched by 24 of the 73 commits then in the local history.[^n42-start]

## Phase B — split the backlog (before the notes)

Splitting `TODO.md` does not depend on migrating the notes, so it went first.
Items lived in `research/notes/backlog/` beside the notes until Phase 4 moved
them to `backlog/`.[^n42-phaseb]

1. Mechanically split each `TODO.md` entry into an item file.
2. One agent pass per area trims narrative, writes `closes_when`, assigns
   `priority` and links the notes that hold the detail. Entries describing
   finished work are not carried over, but their caveats are captured by
   severity and scope. A standing fact that bounds how results may be read
   becomes or joins a concept; something still to do becomes a new item.
3. Closed-sprint history becomes `Sprint Record` stubs. The scope decisions
   become `Design Decision` concepts, which the user reviews and stamps
   `verified`.
4. Add the generator with its filters, the lint and the docs page.
5. Replace `TODO.md` with a stub. Re-point `AGENTS.md`, `.agents/agents/*.md`
   and the skills: the manager reads the view, and briefs name a session or an
   item.

**As executed (2026-10-06).**

- Nine parallel agents took `TODO.md`'s 122 backlog segments (118 entries and 4
  section intros), each checking its entries against the code first.
- The result was 130 items: 40 validation, 38 feature, 37 performance and 15
  hygiene.
- 10 entries were finished with nothing to capture. 4 were finished but held a
  caveat no note recorded; those went to `facts/`.
- Every entry ID is accounted for in the agents' reports. Stale entries were
  corrected: renamed functions, superseded figures, wrong section citations,
  and refusals that no longer exist.
- Phase B's other outputs sat beside the notes until Phase 4:
  `sprints/<name>/closeout.md` (one record per closed-sprint line), `decisions/`,
  `facts/` and `pipeline/status.md`.
- The tooling is `scripts/kb.py`. CI runs `kb-lint` in its own job.
- The user reviewed the decisions on 2026-10-09. Two stand:
  [release-scope-lo-mlm](../decisions/release-scope-lo-mlm.md) and
  [pr-per-backlog-item](../decisions/pr-per-backlog-item.md). Three are
  deprecated: the two earlier release scopes, and the sprint rhythm that
  pr-per-backlog-item supersedes.

## Phase 0 — conform in place

Add frontmatter (`type`, `title`, `description`, `tags`, `generated`,
`status`) to every note where it stands, and generate `index.md` and `log.md`.
Nothing moves, so no citation breaks, and at the end the notes are already a
conformant bundle rooted at `research/notes/`.[^n42-phase0]

**As executed (2026-10-06).**

- Three agents added frontmatter to the 54 notes, leaving the bodies
  byte-identical.
- Types follow note 42's table, with these departures: 15, 18, 21, 22 and 23
  are `Sprint Plan`; 08 and 13 are `Design`; 14 is `Paper` (egglog); 26 is
  `Feasibility Study`.
- 08–11 are `deprecated`, superseded by 13.
- Several notes carry stale status banners under a closed sprint: 27
  "ACTIVE", 28 "APPROVED", 38 "PLANNED" and 41-mlm "WAITING ON refdata-9".
  Their frontmatter says `stable`; Phase 2 resolves the bodies.
- The results files carry `measured:` blocks for the hosts and commits they
  name.

## Phase 1 — chunk, embed, cluster (scratchpad only)

Split the notes at `##`/`###` headings, recording each chunk's note, section
path, line range, dates and citations. Embed with a small local CPU model into
a throwaway vector store in the session scratchpad. Nothing is added to
`pixi.toml`, and nothing from this phase is committed. Cluster, then report
proposed concepts, near-duplicates across notes, and contradictions, which go
to the drafting agents as "resolve, don't average" items. A `kb-query` command
returns the top-k chunks for a text. **Checkpoint:** the user approves the
taxonomy.[^n42-phase1]

**As executed (2026-10-06).**

- 1,376 chunks from 61 files, embedded with `bge-small`.
- Raw embeddings clustered by source note, because sprint-plan prose reads
  alike. Subtracting each note's mean embedding gave cross-note topic clusters,
  but none separated cleanly: HDBSCAN left most chunks as noise, and spectral
  clustering left one large generic bucket.
- So the clusters and each chunk's nearest neighbours in other notes served
  only as hints to eleven agents. Those agents read every chunk and assigned it
  a kind (727 knowledge, 124 decision, 171 measurement, 354 history-only) and
  one or more concepts. They recorded 169 contradictions, each checked against
  the code for which side holds.
- Six reconciliation agents merged the 428 overlapping proposals. Every
  proposal is accounted for as merged, moved or dropped, and every
  non-history chunk feeds a concept.
- The proposal added `run-card/`, `hadronic/`, `tooling/` and `workflow/` to
  the provisional folders.
- The user approved the taxonomy on 2026-10-09: **241 concepts in 16 topic
  folders**, each mapped to the chunks that feed it.

## Phase 2 — draft concepts

Sixteen agents draft in parallel, one group of concepts each, and write only
their own concepts under `research/kb/<folder>/`. Nothing else in the
repository changes in this phase.[^n42-phase2-5]

- Each concept is `status: stable`, with `generated.by` set to the agent.
- Each states current truth: it settles its listed contradictions against the
  code or the later note, keeps every caveat, and carries no sprint narrative.
- Sprint plans are mined for their decisions, which become `Design Decision`
  concepts in topic folders. Each plan shrinks to a `Sprint Record`.
- Each agent reports which chunks it used or left out (each exactly once),
  what it folded, what it could not settle, and new work it found.
- The reviewed decisions under `decisions/` are linked, never redrafted.

**As executed (2026-10-09).** Sixteen agents drafted 229 concepts. Of the 241
approved, `workflow/sprint-rhythm` folded into a sibling and eleven papers
became `sources` entries on the concepts that cite them rather than `Paper`
concepts.

## Phase 3 — adversarial verification and coverage

Separate verifier agents trace every number and claim back to a source chunk.
A coverage map assigns each chunk ID to at least one concept, or tags it
`history-only`. **A script checks coverage; the drafting agents do not get to
assert it.** This is the guard against the migration's main risk, a drafted
concept dropping a caveat its note carried. The archived notes also stay
available as `sources`.[^n42-risks]

## Phase 4 — move and re-cite

1. Generate a mapping from (note, §) to concept ID.
2. Rewrite the external citations from it: backlog items, code, validation,
   scripts, `AGENTS.md`, `.agents/`, `research/refs/README.md` and
   `research/README.md`.
3. Move the original notes to `kb/history/notes/` as `Working Note`,
   `status: deprecated`, each linking to its replacements. Citations between
   notes keep resolving inside the archive.
4. Move the backlog to `kb/backlog/` and the reviewed decisions to
   `kb/decisions/`, and point `scripts/kb.py`'s root and the docs build at
   `research/kb/`.
5. Update the agent briefs (start at `research/kb/index.md`) and the planning
   section of `AGENTS.md`.
6. Add a `new-sprint` scaffold script.

**As executed (2026-10-09).** The 54 notes moved to `history/notes/` as
deprecated Working Notes, each with `original_type` and a `replaced_by` list
built from the chunk-to-concept map (the concepts its chunks feed, largest
first). The backlog, the reviewed decisions and the Sprint Records moved in
beside the topic folders. The four Phase B facts and the pipeline status table
retired: each was absorbed by a concept, and links to them were rewritten. 192
relative links in the moved files were recomputed for the new layout. Citations
outside the bundle that named a `research/notes/` path (code comments,
`AGENTS.md`, the agent briefs, `README.md`, `TODO.md`, the docs, the
validation scripts) were re-pointed at the replacing concepts; bare
"note NN" mentions, as in the manifest's notes, still resolve under
`history/notes/`. `kb.py`'s root is `research/kb/`, its lint checks every link
outside the archive, and `pixi run new-sprint` scaffolds a sprint folder. The
concepts stay `status: draft` with machine-tier `verified` stamps until a
person reviews them.

## Phase 5 — Attested Computations for gates (dropped)

The plan was to make each validation gate an OKF Attested Computation: the
pixi task as executor, a `validation-report` run as receipt, the manifest
collator as attester. Dropped (user, 2026-10-09): a CI run already is one for
everything CI runs (the commit, the pinned command and the required check), so
receipts would duplicate it. What CI never runs, the long-tier gates, keeps the
one useful part as a backlog item without the OKF machinery:
[long-tier-gates-write-no-receipt](../backlog/validation/long-tier-gates-write-no-receipt.md).

## After the migration

The first sprint is the dedicated
[hygiene sprint](hygiene/closeout.md), run in the full
[sprint lifecycle](../workflow/sprint-lifecycle.md) shape, including the
draft-PR claim. It is the lifecycle's test on live work. Then work moves to
[one PR per backlog item](../decisions/pr-per-backlog-item.md), and the
lifecycle's briefs, reports and close-out apply per PR.

[^n42-phaseb]: Note 42 §8 Phase B and its "As executed".
[^n42-phase0]: Note 42 §8 Phase 0 and its "As executed".
[^n42-phase1]: Note 42 §8 Phase 1 and its "As executed".
[^n42-phase2-5]: Note 42 §8 Phases 2–5 and Trial; the Phase 2 rules are those of the drafting brief.
[^n42-decisions]: Note 42 §9.
[^n42-risks]: Note 42 §10.
[^n42-start]: Note 42 §2.
