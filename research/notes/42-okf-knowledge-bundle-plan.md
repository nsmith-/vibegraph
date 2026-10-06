---
type: Design
title: "Reference material as an OKF knowledge bundle: refactor plan"
description: "Plan to turn research/ into an OKF v0.2 knowledge bundle and TODO.md into one-file backlog items: target shape, sprint lifecycle, measurement provenance and migration phases."
note: "42"
created: 2026-10-05
status: draft
tags: [okf, knowledge-bundle, backlog, documentation, migration]
generated: {by: claude-code, at: 2026-10-05}
---
# 42 — Reference material as an OKF knowledge bundle: refactor plan (2026-10-05)

The question (user): restructure `research/` in the style of the Open Knowledge
Format, [OKF v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md),
using subagents and a temporary vector database to cluster what the notes
already contain; and (user, same discussion) make the work backlog that
`TODO.md` holds today resilient to several work streams at once. This note is
the plan: the target shape, how a sprint runs against it, the backlog as
one-file items with a generated view, the migration phases, and the decisions
taken in the planning discussion.

**Status: in progress.** Phase B and Phase 0 ran on 2026-10-06, after the PRs
in flight had merged (§8 records how each went); Phase 1 is next. All other
work pauses while the migration runs, so no parallel stream writes notes or
backlog entries in the meantime. The decisions taken are in §9.

## 1. OKF in brief

A bundle is a directory tree of markdown files. Every file except the reserved
`index.md` and `log.md` is a *concept*: YAML frontmatter, then a free-form body.
The only required key is a non-empty `type`. Type values are not registered
centrally, so a bundle defines its own vocabulary.

| Family | Keys | Use here |
|---|---|---|
| Descriptive | `title`, `description`, `resource`, `tags` | every concept |
| Provenance (§5.1) | `sources[]` (`id`, `resource`, `title`, …) plus `[^id]` body footnotes | papers, upstream code permalinks, Rust paths, archived note sections |
| Trust (§5.2–5.3) | `generated: {by, at}`, `verified: [{by, at}]` | tiers: unverified / machine-confirmed / human-reviewed |
| Lifecycle (§5.4–5.5) | `status: draft \| stable \| deprecated`, `stale_after` | `stale_after` only for things that expire with time (§5) |
| Computation (§10) | `type: Attested Computation`, `runtime`, `parameters`, `executor`, `attester` | optional, for validation gates (Phase 5) |

Actors follow `<producer>/<version>` for agents (`claude-code/claude-opus-5-5`),
`human:<id>` for people (`human:nsmith-`), `process:<id>` for automation.
Links are standard markdown, preferably bundle-relative (`/amplitudes/x.md`);
consumers must tolerate broken links. `index.md` files carry no frontmatter,
except that the bundle root's may declare `okf_version: "0.2"`. `log.md` is a
newest-first list under `## YYYY-MM-DD` headings. Unknown frontmatter keys are
allowed, which is what lets §5's `measured:` block exist without a spec change.

## 2. Starting point

- `research/notes/`: 47 notes, ~38k lines, 1,211 `##`/`###` sections,
  numbered chronologically. Genres are mixed: conventions and derivations,
  designs, measurements, surveys of papers and codebases, and sprint plans. The
  sprint plans (19, 24, 27–38) are ~60% of the lines; notes 24, 28 and 29 alone
  run 3k–6k lines each, mostly dated amendments.
- Citations of the form "note NN §x": 109 in `TODO.md`, ~16 in code, validation
  and scripts, one each in `AGENTS.md` and `.agents/`, 362 between notes. Any
  move must keep these resolvable.
- `TODO.md`: 1,072 lines, ~120 bullet entries (median 6 lines, longest 33)
  across the validation, feature and performance backlogs, plus 103 lines of
  closed-sprint history and ~75 lines of hand-written "current position",
  scope decisions, census and standing measurement facts. It is the repo's
  hottest file (24 of the 73 commits in the local history touch it), every
  work stream edits it, and entries accrete narrative: how a problem was
  solved, rather than what is still open.
- `.agents/agents/*.md` each carry a hand-maintained "Research note index".
- `research/refs/` holds four submodules (MadGraph5_aMC@NLO, Sherpa,
  POWHEG-BOX-V2, FeynGraph) and the paper fetch script; fetched papers land in a
  gitignored `papers/`.
- `docs/src/` is the user-facing mdBook. It stays separate; its
  `bibliography.md` overlaps the bundle's `Paper` concepts and may later link
  to them.

## 3. Target shape

Bundle root **`research/kb/`**. The root `index.md` declares
`okf_version: "0.2"`. Folders are organised by topic, not by date:

```
research/
  kb/
    index.md  log.md
    pipeline/             overview; the cross-section and event-weight chain
    model/                UFO parsing, UFO↔ALOHA type matrix, Lorentz runtime eval
    amplitudes/           HELAS / typed-repr conventions, vector-vertex signs, colour flow
    process/              process grammar, diagram enumeration
    phase-space/          resonance sampling, MadEvent maps, soft angle, VEGAS, variance-flow
    scales-pdf/           dynamical and per-group scales, kT clustering, hadronic σ
    events/               LHEF, distributions, proton events
    validation/           layering, refdata representation, seed headroom, bit-exact oracle method
    performance/          eval optimisation, egglog, BCE, rooting / arena / threading / AVX2 studies
    references/papers/    one concept per paper; resource = arXiv / DOI URL
    references/codebases/ MG5aMC, Sherpa, POWHEG-BOX, FeynGraph; resource = upstream permalink
    backlog/<area>/       one file per open work item (§7)
    sprints/<name>/       one folder per sprint (§4)
    history/notes/        the original notes 00–41, archived verbatim
  refs/                   submodules and paper fetching; outside the bundle (§6)
  ufo/                    sample UFO models; outside the bundle
```

The topic folders are provisional. Phase 1's clustering proposes the real
taxonomy, and you approve it before anything is drafted.

**Type vocabulary** (ours; OKF registers none): `Overview`, `Physics
Convention`, `Derivation`, `Design`, `Design Decision`, `Algorithm`,
`Validation Methodology`, `Validation Gate`, `Measurement`, `Feasibility
Study`, `Procedure`, `Paper`, `Codebase`, `Codebase Survey`, `Sprint`,
`Session Brief`, `Session Report`, `Sprint Record`, `Audit`, `Backlog Item`,
`Caveat` (a standing fact that bounds how results may be read), `Working Note`.
The pre-migration notes also carry `Paper` and `Sprint Plan` (a whole sprint's
plan in one file); Phase 2 retires `Sprint Plan` in favour of the §4 shape.

**Conventions:**
- Concept bodies state current truth. History lives in `log.md` and git, which
  matches the comment rule in `AGENTS.md`: no "Status: APPROVED" banners, no
  "design amendment 2" sections.
- `verified` with a `human:` actor is added only when a person actually
  reviewed the concept, never in bulk.
- `index.md` files are generated from frontmatter by a script, never by hand.
  The backlog folders have none: an index there would change with every item
  and conflict between branches, so `pixi run backlog` is their listing.
- A conformance lint runs in CI as a pixi task. It checks that every
  non-reserved `.md` under `kb/` parses and has a non-empty `type`, and that the
  generated indexes are current.
- The tooling — index generator, lint, backlog view — is Python, run as pixi
  tasks (`backlog`, `kb-lint`, `kb-index`) in a small `kb` pixi environment
  (`python`, `pyyaml`). The default environment carries no Python, so the
  tools get their own feature rather than widening it. Frontmatter is parsed
  with PyYAML, never a hand-written parser. `ci.yml` already sets up pixi for
  the lint; `docs.yml` gains a `setup-pixi` step so `build-docs.sh` can render
  the backlog page.

## 4. Sprint lifecycle under the bundle

A sprint becomes a folder of linked concepts instead of one plan file that
grows to thousands of lines. Lasting knowledge is promoted into the topic
folders at close-out. Shape, using `process-grammar` (note 38) as the example:

```
kb/sprints/process-grammar/
  index.md                reading order: overview → decisions → sessions
  log.md                  dated approvals, amendments, re-scopes
  sprint.md               type: Sprint — goal, exit criteria, scope, session graph
  audit.md                type: Audit — snapshot of today's surface (note 38 §2)
  decisions/D1-….md       type: Design Decision — one per decision
  sessions/G1.md          type: Session Brief — scope, agent type, dependencies, gate
  sessions/G1-report.md   type: Session Report — written by the dev agent
  closeout.md             type: Sprint Record — what was banked, census delta
```

1. **Open (sprint agent).** The manager dispatches a sprint agent session with
   the backlog items in scope. Its first task is a plan of work: the sprint
   folder with `sprint.md` at `status: draft` and `active: true`, and the
   session briefs. It opens a **draft PR** carrying that plan, whose body lists
   the claimed items one per line (`Backlog: <slug>`). The open draft PR *is*
   the claim (§7.1). Before writing anything new, it looks up the topic's
   existing concepts. Those still at `draft` are open
   questions; measurements whose recorded commit predates heavy churn in the
   area are re-measurement candidates (§5). The sprint agent pushes to the
   draft PR only at infrequent checkpoints, typically once the manager has
   checked a dev session's report, so intermediate CI runs stay few.
2. **Survey.** Reading upstream code (note 38 §1, "MadGraph semantics read from
   the pinned source") produces or updates a reusable `Codebase Survey` under
   `references/codebases/`, so the next sprint starts from it instead of
   re-reading MadGraph. Findings specific to the sprint go in `audit.md`.
3. **Design and approval.** The design edits topic concepts in place at
   `status: draft`, with one `Design Decision` concept per decision. Approval is
   data: `verified: [{by: human:nsmith-, at: …}]`, status flipped to `stable`,
   and an `**Approval**` entry in the sprint's `log.md`.
4. **Sessions.** A `Session Brief` is short. It holds the scope and links to the
   binding design concepts, the decisions, its gate and the backlog items it
   closes (`closes: [...]`). The dispatch brief names the session concept (or,
   for work outside a sprint, a single backlog item), and the dev agent follows
   links from there; that is OKF's progressive disclosure. A dev agent never
   reads the whole backlog (§7.3). The note index in `.agents/agents/*.md` becomes
   "start at `research/kb/index.md`". The agent writes its report as
   `sessions/<id>-report.md`, with `generated.by` itself and `sources` its commit
   hashes. That is the one bundle file a dev agent may write; new work it
   discovers goes in the report's "Found" section, not into the backlog. The
   trust tiers
   then encode "subagent reports are evidence, not truth":

   | Tier | Meaning |
   |---|---|
   | unverified | the agent's own claim |
   | machine-confirmed | the manager spot-checked it (`verified: claude-code/…`) |
   | human-reviewed | a person checked it |

   A design deviation still stops the session. The manager edits the design
   concept in place and adds a dated line to the sprint's `log.md`.
5. **Validation.** Briefs link to `Validation Gate` concepts. If Phase 5 is
   adopted, a gate is an Attested Computation whose receipt is a recorded
   `validation-report` run, checked by the manifest collator.
6. **Close-out.** The only point where session results reach shared concepts,
   so parallel sessions never conflict over them.
   - **Promote:** changed topic concepts go to `stable` after review. New
     lessons go into methodology concepts. Superseded concepts become
     `deprecated` and link to their replacement.
   - **Measurements:** become `Measurement` concepts with a `measured:` block
     (§5). Take them after the sprint's last code change where possible.
   - **Record:** `closeout.md` holds what was banked and the census change;
     `landed_in` is filled for the sprint's measurements once the PR merges.
   - **Backlog:** delete the files of items the sprint closed, drop the
     `Backlog:` lines of items it leaves open (releasing the claim), and file
     the reports' "Found" entries as new items.
   - **Bookkeeping:** set `active: false` on `sprint.md`, add to the root
     `log.md`, regenerate indexes, run the lint, and mark the PR ready for
     review.

`sprint.md` stays a readable overview, with the session list and decision
summaries inline, so the plan can still be reviewed in one file. The lint can
flag a sprint marked `stable` that still links to `draft` design concepts.

## 5. Measurements and staleness

Measurements go stale when code changes, not when time passes. No machinery
tracks that automatically. A measurement records the facts, and a later session
judges whether re-measuring is warranted.

- `stale_after` stays empty for anything tied to code that will likely change.
  It is kept for the rare fact that genuinely expires with time.
- The commit the numbers came from is recorded, together with what survives a
  squash-merge. PRs land on `main` as one squash commit, often a whole sprint
  (`1539abc` is all of `process-grammar`, `nsmith-/vibegraph#12`). So a
  measurement commit taken inside a sprint never appears on `main`, and its
  branch may be deleted. GitHub keeps `refs/pull/<n>/head`, so the PR number
  keeps the commit fetchable.

```yaml
measured:
  commit: 4f2c9e1     # the tree the numbers came from
  pr: 12              # fetch with `git fetch origin pull/12/head`
  landed_in: 1539abc  # the squash commit on main, filled at close-out
  host: "M3 Max, macOS 15"
  command: pixi run bench-eval -- gg_ttg
```

A file measured on several hosts or at several commits carries a list of such
mappings. A session deciding whether to re-measure reads:
- what changed on `main` since landing:
  `git log --oneline <landed_in>..origin/main -- <paths the body names>`;
- what changed between measurement and merge, after fetching the PR ref:
  `git log --oneline <commit>..FETCH_HEAD`;
- a direct comparison, `git diff --stat <commit> origin/main -- <paths>`. It
  compares trees, so it works across a squash once the object is fetched.

Cloud sessions are shallow clones, so the agent briefs say to fetch the ref or
deepen history first. A missing `landed_in` means the PR has not merged or
close-out skipped it; `pr` tells which. `host` decides whether two perf numbers
are comparable, not whether one is stale.

Deferred, to revisit only if this proves too loose: deriving each measurement's
dependency set (e.g. from a coverage run of its command) and computing
staleness from per-file git blob IDs.

## 6. External codebases stay outside the bundle

The submodules under `research/refs/` are external code. They are normally not
checked out; MadGraph is checked out only when a new validation reference is
generated. So:

- `kb/` never contains them. The lint and index scripts only look inside
  `kb/`, so populating a submodule cannot break conformance.
- A `Codebase` concept's `resource`, and every `sources` entry citing external
  code, is the upstream permalink at the pinned commit, e.g.
  `https://github.com/mg5amcnlo/mg5amcnlo/blob/<sha>/madgraph/...`. GitLab
  repositories (Sherpa, POWHEG-BOX-V2) use `/-/blob/<sha>/`. Never
  `research/refs/...` paths.
- A survey concept records the pinned commit it read, so a later session can
  compare it with the current pin and decide whether to re-read.
- Survey concepts quote the short excerpts they depend on, so an agent without
  a checkout can use them; the permalink is for checking.
- Checkout instructions stay in `research/refs/README.md` and the
  `extended-validation` skill. OKF's `references/` mirroring convention is not
  used for submodule content; `kb/references/codebases/` holds short concepts
  that describe and link out.
- Fetched papers stay in the gitignored `papers/`. `Paper` concepts link the
  arXiv or DOI URL.

## 7. The backlog as items

### 7.1 One file per item

Each open work item is a concept under `kb/backlog/<area>/<slug>.md`:

```yaml
---
type: Backlog Item
title: acceptance.yml has never passed
description: The release acceptance workflow 404s on release assets; the first run since going public should pass.
area: hygiene            # validation | feature | performance | hygiene
state: needs-user        # open | blocked | needs-user
priority: medium         # low | medium | high
closes_when: An acceptance.yml run against a published release is green.
blocked_by: []           # links to other items
opened: 2026-08-10
---
Problem, what would resolve it, links to the concepts that hold the detail.
```

What this fixes in the structure rather than by discipline:
- **Parallel streams stop conflicting.** They touch different files; a conflict
  happens only when two streams edit the same item, which is a real conflict.
- **IDs never collide.** Filename slugs, never renamed, so two branches cannot
  both create item 43. Sequential note numbers have the same weakness; the
  bundle's slug paths remove it there too.
- **`closes_when` is required**, so every item says what done means.
- **Done means deleted.** The PR that finishes an item removes its file. The
  account of how it was solved lives in the sprint record or session report,
  where the trust tiers mark it as an agent's claim until checked. The backlog
  has no place for a success story, so none accumulate there.

Dev agents do not create, edit or close items: they list new work under
"Found" in their session report, and the manager or the close-out session files
it.

**Claims are draft PRs, not frontmatter.** A sprint claims items by opening a
draft PR whose body lists them (`Backlog: <slug>`, §4.1). A claim written into
an item file would only be visible once merged; a draft PR is visible to every
stream the moment it opens, and claiming edits no item file. The claim ends
when the PR merges (the closed items' files are gone) or closes.

### 7.2 The generated view

`pixi run backlog` renders the backlog from the item files and the sources
below. The view is **not committed**: a committed generated file changes with
every item edit, so two squash-merged PRs would conflict on it again, and
GitHub cannot resolve that server-side. `TODO.md` becomes a stub that says
where the backlog lives and is not edited again.

The docs workflow renders the same view as a book page: `scripts/build-docs.sh`
writes `docs/src/backlog.md` (gitignored) before `mdbook build`, the way it
refreshes the CLI reference, and `SUMMARY.md` lists it. With a token available,
the workflow annotates claimed items from the open draft PRs' `Backlog:` lines;
offline, `pixi run backlog` shows items without claims.

| Today's `TODO.md` section | Generated from |
|---|---|
| Current position | sprint folders with `active: true` |
| Scope decisions (user) | `Design Decision` concepts carrying a `verified: human:` stamp |
| Census | `validation/manifest.toml`, already the source of truth |
| "Open, and the user's call" | items with `state: needs-user` |
| Standing measurement facts | `Measurement` concepts |
| Closed-sprint history | one line from each `Sprint Record`'s `description` |
| Validation / feature / performance backlogs | items grouped by `area`, then `priority` |

The lint checks: `description` is one sentence under a length cap;
`closes_when` is present; `blocked_by` links resolve (stricter than OKF
requires, by choice); an item body past ~40 lines warns, since detail belongs in
a linked concept.

### 7.3 Who reads what

The full view is for planning: the user, and the manager choosing or scoping a
sprint. Agents doing the work are pointed at one thing — a session brief or a
single item — and read outward from its links. `pixi run backlog` takes
filters for that: `--item <slug>` prints the item, its `blocked_by` chain and
the titles and descriptions of the concepts it links; `--area`, `--state` and
`--priority` narrow the planning view. No agent brief tells a dev agent to
read the whole backlog first.

## 8. Migration phases

### Phase B — split the backlog (first; independent of the notes) — done 2026-10-06

`TODO.md` is the hottest conflict point, and splitting it does not depend on
migrating the notes, so it goes first. Until Phase 4 the items live in
`research/notes/backlog/` beside the notes and link to notes by path.

1. Mechanically split each `- **title**` bullet into an item file.
2. One agent pass per area trims narrative, writes `closes_when`, assigns
   `priority` (low / medium / high) and links the notes that hold the detail.
   Entries describing finished work are not carried over as items, but any
   caveat they hold is captured, best effort, by severity and scope: a standing
   fact that bounds how results may be read (e.g. the `refdata-2` vs
   `refdata-3` σ comparability warning) becomes or joins a concept; something
   that still wants doing becomes a new item.
3. Closed-sprint history becomes `Sprint Record` stubs; the scope decisions
   become `Design Decision` concepts, which the user reviews and stamps
   `verified` in this phase.
4. Add the generator with its filters, the lint and the docs page (the `kb`
   pixi environment, §3).
5. Replace `TODO.md` with the stub, and update the `TODO.md` instructions in
   `AGENTS.md`, `.agents/agents/*.md` and the skills: the manager reads the
   view; dispatch briefs name a session or an item.

**As executed.** `TODO.md`'s 122 backlog segments (118 entries, 4 section
intros) went to nine parallel agents, each checking its entries against the
code before writing. The result: 130 items, 40 validation, 38 feature, 37
performance and 15 hygiene. 10 entries were finished with nothing left to
capture, and 4 were finished but left a caveat no note recorded, which went
to `facts/`. Every entry id is accounted for in the agents' reports. The
agents corrected the entries where they had gone stale: renamed functions,
superseded figures, wrong section citations, and refusals that no longer
exist. Phase B's other concepts sit beside the notes until Phase 4:
`sprints/<name>/closeout.md` (one `Sprint Record` per closed-sprint line),
`decisions/` (the scope decisions and the sprint rhythm, awaiting review),
`facts/` (`Caveat` and `Measurement` concepts) and `pipeline/status.md`.
The tooling is `scripts/kb.py`; CI runs `kb-lint` in its own job.

### Phase 0 — conform in place (one session, mechanical) — done 2026-10-06

Add frontmatter (`type`, `title`, `description`, `tags`, `generated`,
`status`) to the existing notes where they stand. Generate `index.md`, add
`log.md`, add the lint. Nothing moves, so no citation breaks; at the end of the
phase the notes are already a conformant bundle. Provisional types, to be
confirmed by reading each note:

| Notes | Type |
|---|---|
| 00 | Overview |
| 01 | Paper (survey of several) |
| 02, 03, 07, 14 | Codebase Survey |
| 05 | Procedure |
| 08, 09, 11, 13, 39 | Physics Convention / Derivation |
| 04, 06, 10, 15, 16, 18, 21, 22, 23, 25, 26, 40 | Design |
| 12 | Validation Methodology |
| 17, 30, 36a, `*-results.md` | Measurement |
| 37 | Codebase Survey + Design |
| 41 | Feasibility Study |
| 19, 20, 24, 27–29, 31–36, 38 | Sprint Plan |

During Phase 0 the bundle root is `research/notes/`; Phase 4 moves it to
`research/kb/`.

**As executed.** Three agents added the block to the 54 notes, leaving the
bodies byte-identical. Departures from the table above: 15, 18, 21, 22 and 23
are `Sprint Plan`; 08 and 13 are `Design`; 14 is `Paper` (the egglog paper);
26 is `Feasibility Study`. 08–11 are `deprecated`, superseded by 13. Several
notes carry stale status banners in their bodies (27 "ACTIVE", 28 "APPROVED",
38 "PLANNED", 41-mlm "WAITING ON refdata-9") under a closed sprint;
frontmatter says `stable`, and Phase 2 resolves the bodies. The results files
carry `measured:` blocks for the hosts and commits they name.

### Phase 1 — chunk, embed, cluster (scratchpad only, never committed)

1. Split the notes at `##`/`###` headings (~1.2k chunks). Each chunk records
   note number, section path, line range, date stamps, and what it cites (note
   §refs, file paths, arXiv IDs).
2. Embed with a small local CPU model (e.g. `bge-small` via `fastembed`) into a
   throwaway LanceDB, in a virtualenv in the session scratchpad. Nothing is
   added to `pixi.toml`.
3. Cluster with HDBSCAN on a hybrid similarity: embedding cosine blended with
   the citation graph. Embeddings alone tend to group by writing style (sprint
   prose) rather than topic.
4. Report: proposed clusters → concepts; near-duplicates across notes (the same
   convention restated, e.g. 13, 16, 39), which become merge candidates; and
   contradictions (a later amendment overriding an earlier design), which go to
   the drafting agents as explicit "resolve, don't average" items.
5. Provide a `kb-query "<text>"` command returning the top-k chunks, for the
   drafting agents.

**Checkpoint:** the user approves the taxonomy.

### Phase 2 — draft concepts (subagents)

One agent per cluster, ~8–10. Each gets its cluster's chunks plus `kb-query`
for context the clustering missed. Each writes only into its own folder, and
marks every concept `status: draft` with `generated.by` itself. Sprint plans are
mined for their decisions, which become `Design Decision` concepts in topic
folders; each plan shrinks to a `Sprint Record` linking to them.

### Phase 3 — adversarial verification and coverage

Separate verifier agents trace every number and claim back to a source chunk.
A coverage map assigns each chunk ID to at least one concept or explicitly tags
it `history-only`. The script checks coverage; the drafting agents do not get
to assert it.

### Phase 4 — move and re-cite

Generate a mapping from (note, §) to concept ID. Rewrite the external citations
(backlog items, code, validation, scripts, `AGENTS.md`, `.agents/`,
`research/refs/README.md`, `research/README.md`) from it. Move the original
notes to `kb/history/notes/` as `Working Note`, `status: deprecated`, each
linking to its replacements; the 362 citations between notes keep resolving
inside the archive. Update the agent briefs (§4.4) and `AGENTS.md`'s planning
section, and add a `new-sprint` scaffold script.

### Phase 5 (optional) — Attested Computations for validation gates

The pixi validate tasks are the executor, a `validation-report` run is the
receipt, the `validation/manifest.toml` collator is the attester. Pilot on two
or three gates. If it holds, the backlog view's census can be derived from
attested receipts instead of the manifest's recorded cells.

### Trial

The first sprint after the migration runs in the §4 shape, including the
draft-PR claim, and is the lifecycle's test on live work. It takes a backlog
item chosen at that point (MLM, the one planned earlier, has since merged).

## 9. Decisions (user, 2026-10-05)

1. **Fan-out.** Phases 2–3 run ~15–20 agents in parallel; approved.
2. **Bundle root** is `research/kb/`.
3. **The backlog renders on the docs site** as part of the docs workflow
   (§7.2).
4. **No parallel work during the migration**, so new notes need no interim
   naming scheme and sequential-number collisions cannot arise mid-migration.
5. **The measurement convention (§5) starts after the migration.**
6. **Caveats in finished-work `TODO.md` entries** are captured best effort as
   concepts or new items by severity and scope (Phase B step 2).
7. **The user verifies the scope decisions** during Phase B.
8. **`priority` is `low | medium | high`.**
9. **Tooling is Python under pixi** (§3): it runs as pixi tasks anyway, so a
   Rust crate would buy nothing.
10. **The trial sprint** takes a backlog item chosen when the migration ends.
11. **Draft PRs are the claim mechanism** (§4.1, §7.1), with pushes at
    infrequent checkpoints.

Still deferred, for evidence that the simple version falls short: deriving
each measurement's code dependencies to compute staleness (§5).

## 10. Risks

- **Fragmentation.** Agents and reviewers lose the single scrollable plan.
  Mitigated by `sprint.md` as an inline overview and by `index.md` reading
  orders.
- **Skipped promotion at close-out.** Topic concepts silently drift behind the
  sprint folders. The lint catches the draft-under-stable case; the rest is
  close-out discipline.
- **Lossy distillation.** A drafted concept drops a caveat the original note
  carried. Phase 3's coverage map and claim tracing guard this, and the
  archived notes stay available as `sources`.
- **No browsable backlog in the repository tree.** The view is generated, not
  committed; the docs page is the browsable copy and is as fresh as the last
  docs deploy. The fallback, if that proves too inconvenient, is committing the
  view with a CI freshness check and the rule "on conflict, regenerate".
- **Unreadable claims.** A draft PR whose body drops or mistypes a `Backlog:`
  line claims nothing. The docs build warns on a slug that names no item.
- **Frontmatter overhead per sprint.** Kept small by the scaffold script and
  generated indexes.
