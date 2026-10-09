---
type: Design
title: "The knowledge bundle: OKF format, layout and conventions"
description: "What research/kb is for; OKF v0.2 in brief; the topic folders, type vocabulary and current-truth bodies; generated indexes and kb-lint; citing external code by permalink and papers by arXiv/DOI."
status: draft
tags: [okf, knowledge-bundle, documentation, conventions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n42-intro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L11-L26", title: "Note 42: the question and plan"}
  - {id: n42-okf, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L28-L49", title: "Note 42 §1: OKF in brief"}
  - {id: n42-shape, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L76-L133", title: "Note 42 §3: target shape, type vocabulary, conventions"}
  - {id: n42-external, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L258-L280", title: "Note 42 §6: external codebases stay outside the bundle"}
  - {id: n42-phaseb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L366-L401", title: "Note 42 §8 Phase B: tooling as executed"}
  - {id: n42-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/42-okf-knowledge-bundle-plan.md#L514-L535", title: "Note 42 §9: decisions (user, 2026-10-05)"}
  - {id: okf-spec, resource: "https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md", title: "Open Knowledge Format v0.2 specification"}
  - {id: kb-py, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/scripts/kb.py", title: "scripts/kb.py: index, lint and backlog view"}
---
`research/kb/` holds the project's reference material as an
[OKF v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
knowledge bundle.[^okf-spec] That covers physics conventions, derivations,
designs, validation methodology, measurements, surveys of papers and upstream
codebases, the backlog, and sprint folders. It is organised by **topic, not
date**. A session reads outward from the concept or backlog item its brief
names, rather than scrolling a chronological plan. The migration that built it
is [okf-migration](../sprints/okf-migration.md).

## OKF in brief

A bundle is a directory tree of markdown files. Every file except the reserved
`index.md` and `log.md` is a *concept*: YAML frontmatter, then a free-form
body. The only required key is a non-empty `type`. OKF registers no type
values, so the bundle defines its own vocabulary.[^n42-okf]

| Family | Keys | Use here |
|---|---|---|
| Descriptive | `title`, `description`, `resource`, `tags` | every concept |
| Provenance | `sources[]` (`id`, `resource`, `title`) plus `[^id]` body footnotes | papers, upstream permalinks, Rust paths, archived note sections |
| Trust | `generated: {by, at}`, `verified: [{by, at}]` | unverified, machine-confirmed or human-reviewed |
| Lifecycle | `status: draft \| stable \| deprecated`, `stale_after` | `stale_after` only for facts that expire with time |
| Computation | `type: Attested Computation`, `runtime`, `parameters`, `executor`, `attester` | not used: CI is the attestation for the gates it runs |

- **Actors.** Agents are written `<producer>/<version>`
  (`claude-code/claude-opus-5-5`), people `human:<id>` (`human:nsmith-`), and
  automation `process:<id>`.
- **Links** are standard markdown. Consumers must tolerate broken ones. Concepts
  here use file-relative links.
- **Reserved files.** `index.md` carries no frontmatter, except the root's,
  which declares `okf_version: "0.2"`. `log.md` is a newest-first list under
  `## YYYY-MM-DD` headings.
- **Extra keys.** Unknown frontmatter keys are allowed. That is what lets the
  [`measured:` block](measurement-provenance.md), `closes_when`, `active` and
  `decided_by` exist without a spec change.

## Layout

| Folder | Holds |
|---|---|
| `pipeline/` | The end-to-end chain, release scope, the persisted artifacts |
| `model/` | UFO parsing and grammars, restriction, coupling orders, model identity |
| `process/` | Proc-card grammar, the supported-card check, enumeration, decays, polarization |
| `amplitudes/` | HELAS and repr conventions, signs and phases, colour, the evaluator |
| `phase-space/` | Channels, maps, multichannel, VEGAS, budgets |
| `scales-pdf/` | α_s, scale choices, kT clustering, MLM scales, PDF interpolation |
| `hadronic/` | The proton integrand, flavour groups, the beam-mirror identity |
| `run-card/` | Run-card parsing, field classification, cuts, matching parameters |
| `events/` | Unweighting, selection, LHEF, `generate`, the matched record |
| `validation/` | Layers, oracles, gates, references, MadGraph defects |
| `performance/` | The evaluator's performance design and studies |
| `tooling/` | Cargo, CI, profiling, releases, assets, the MadGraph toolchain |
| `workflow/` | How work runs: this bundle, the backlog, sprints, agent dispatch |
| `references/papers/` | One concept per paper the project builds on; `resource` is the arXiv or DOI URL |
| `references/codebases/` | MadGraph5_aMC@NLO, Sherpa, POWHEG-BOX, FeynGraph and comparative surveys |
| `decisions/` | Reviewed scope and process decisions |
| `backlog/<area>/` | One file per open work item ([backlog items](backlog-items.md)) |
| `sprints/<name>/` | One folder per sprint ([sprint lifecycle](sprint-lifecycle.md)) |
| `history/notes/` | The original numbered notes, archived verbatim as `Working Note`, `status: deprecated` |

Topic concepts cite note sections in `sources` by permalink at a commit, with
a line range; a body link to an archived note is `../history/notes/<file>.md`.
The 362 citations between archived notes keep resolving inside the archive.

**Outside the bundle:** `research/refs/` (submodules and paper fetching),
`research/ufo/` (sample UFO models), and the user-facing mdBook under
`docs/src/`. Its `bibliography.md` overlaps the `Paper` concepts.

## Type vocabulary

`Overview`, `Physics Convention`, `Derivation`, `Design`, `Design Decision`,
`Algorithm`, `Validation Methodology`, `Validation Gate`, `Measurement`,
`Feasibility Study`, `Procedure`, `Paper`, `Codebase`, `Codebase Survey`,
`Sprint`, `Session Brief`, `Session Report`, `Sprint Record`, `Audit`,
`Backlog Item`, `Caveat` (a standing fact that bounds how results may be read)
and `Working Note` (the archived notes). `Sprint Plan`, a whole sprint in one
file, is retired in favour of the sprint folder.[^n42-shape]

## Conventions

- **Bodies state current truth.** History lives in `log.md`, git and the
  archived notes. A concept carries no status banners ("APPROVED", "PLANNED"),
  no "amendment 2" sections, and no "originally … later" narrative. This matches
  the comment rule in `AGENTS.md`. Binding project rules stay in `AGENTS.md`;
  concepts that overlap them carry the worked cases and link back.
- **Trust is data.** Agents write `generated.by` themselves. `verified` with a
  `human:` actor is added only when a person actually reviewed the concept,
  never in bulk. Manager spot-checks add `verified` with the manager's agent
  actor. A drafted concept is `status: draft` until reviewed.
- **Indexes are generated, never hand-written.** `pixi run kb-index` writes each
  folder's `index.md` from the frontmatter, grouped by type. The backlog
  folders have none: an index there would change with every item and conflict
  between branches, so `pixi run backlog` is their listing.
- **Sources resolve.** A note section is cited by permalink at a commit with a
  line range. Upstream code is cited by permalink at the pinned commit (below).
  Our own code is cited by path.

## Tooling

`scripts/kb.py`, run as pixi tasks in a small `kb` environment (`python`,
`pyyaml`). The default environment carries no Python, so the tools get their
own feature rather than widening it. Frontmatter is parsed with PyYAML, never a
hand-written parser. Python was chosen because the tools run as pixi tasks
anyway, so a Rust crate would buy nothing.[^n42-decisions]

| Task | Does |
|---|---|
| `pixi run kb-index` | Regenerates every `index.md`; `--check` reports stale ones. |
| `pixi run kb-lint` | Runs in CI as its own job. |
| `pixi run backlog` | Renders the backlog view ([backlog items](backlog-items.md)). |

The scripts read one bundle root, `ROOT` in `kb.py`: `research/kb/`.
`pixi run new-sprint <name>` scaffolds `sprints/<name>/` with `sprint.md`,
`log.md`, `sessions/` and `decisions/`.

`kb-lint` checks the following:

- every non-reserved `.md` has parseable frontmatter with a non-empty `type`;
- every `description` is one line;
- the generated indexes are current;
- every backlog item passes the item checks: area, state and priority values,
  the area matching its folder, required `title`, `description`,
  `closes_when` and `opened`, description length, slug uniqueness and
  `blocked_by` resolution.

It does not check links between concepts, and it does not yet flag a `stable`
sprint that still links to `draft` design concepts.[^kb-py]

## External codebases and papers

The submodules under `research/refs/` are external code and are normally not
checked out. MadGraph is checked out only when a reference is regenerated.
So:[^n42-external]

- `kb/` never contains them. The scripts only look inside the bundle root,
  so populating a submodule cannot break conformance.
- A `Codebase` concept's `resource`, and every `sources` entry citing external
  code, is the **upstream permalink at the pinned commit**, never a
  `research/refs/...` path. GitHub uses `/blob/<sha>/<path>#L<n>`; the GitLab
  projects (Sherpa, POWHEG-BOX-V2) use `/-/blob/<sha>/<path>`. The pins are the
  submodule commits:
  - mg5amcnlo `b7687064` (tag `3.7.1`);
  - Sherpa `e12c72f4`;
  - POWHEG-BOX-V2 `e26982d7`;
  - FeynGraph `1dc4ea7`. vibegraph *builds* against FeynGraph `fd5aa83` (the git
    rev in `vibegraph-lib/Cargo.toml`), so a FeynGraph citation says which of
    the two was read.
- A survey concept records the commit it read, so a later session can compare
  it with the current pin and decide whether to re-read.
- Survey concepts **quote the short excerpts they depend on**, so an agent
  without a checkout can use them. The permalink is for checking.
- Checkout instructions stay in `research/refs/README.md` and the
  `extended-validation` skill. OKF's `references/` mirroring is not used for
  submodule content.
- Fetched papers stay in the gitignored `papers/`. A `Paper` concept exists
  only for a paper the bundle summarises or builds on; its `resource` is the
  arXiv or DOI URL. A paper cited once is a `sources` entry on the concept that
  cites it.

[^okf-spec]: OKF v0.2 specification.
[^n42-okf]: Note 42 §1.
[^n42-shape]: Note 42 §3.
[^n42-external]: Note 42 §6.
[^n42-decisions]: Note 42 §9, decisions 2 and 9.
[^kb-py]: `scripts/kb.py`, `ROOT` and `lint()`.
