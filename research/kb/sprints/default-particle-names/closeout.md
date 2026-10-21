---
type: Sprint Record
title: "MadGraph's default particle names, and --modelname"
description: "Particles answer to MadGraph's PDG-keyed default names through a renamed lookup table, a renamed-away UFO name is refused as MadGraph refuses it, every printed name stays the UFO's, and --modelname switches the renaming off."
closed: 2026-10-20
status: draft
tags: [process-grammar, model, particle-names, madgraph-parity, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/65", title: "nsmith-/vibegraph#65"}
  - {id: mg-import, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L5841-L5842", title: "MadGraph renames unless -modelname"}
---
One PR ([nsmith-/vibegraph#65](https://github.com/nsmith-/vibegraph/pull/65))
closing `madgraph-default-particle-names-missing`. Sessions:
[F1](sessions/F1-report.md) (feature-dev, Opus) and
[HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet).

## What was established

- **MadGraph renames in place.** Unless `-modelname`/`--modelname` is given
  (`madgraph_interface.py:5841-5842`), it walks `particles_name_default.txt` in
  file order and renames each listed PDG code's particle and antiparticle
  (`base_objects.py:1577-1623`), refusing with "Use -modelname" when a default
  is already taken mid-renaming; `mssm`/`mssm-*` models call PDG 25 `h1`.
  Its lookups match current names only, so a renamed-away UFO name (`m+` on
  `loop_sm`) is refused.
- **Neither alias nor rename here, but a renamed lookup table.**
  `ufo::names::ParticleNames` maps each accepted name to the particle's UFO
  name. Process resolution (legs, `define`, `/ $ $$ >` lists) goes through it,
  so MadGraph's acceptance and refusal hold, while every name that leaves the
  library (subprocess labels, logs, the TUI, errors) stays the UFO's; LHEF
  carries PDG codes and artifacts carry the model digest, which does not
  depend on the naming. A renamed-away name is refused with
  `ResolveError::RenamedParticle`, naming the default and suggesting
  `--modelname`.
- **The table** is MadGraph's `input/particles_name_default.txt`, vendored byte
  for byte with its sha256 pinned and its licence recorded. The interned SM
  is unchanged under either naming (its names already are the defaults up to
  case).
- **`--modelname`** is accepted in either spelling anywhere after `model`;
  `--noprefix`, `--last` and `--modelname=x` stay refused (MadGraph ignores the
  last; the first two change nothing generated).
- **A stale table** (a particle added or removed after load without rebuilding
  it) is caught by a debug assertion at the lookup.

The manager re-ran the name, CLI-model and diagram-census tests from the
committed tree after F1 and HY1.

## Backlog

- Deleted: `madgraph-default-particle-names-missing`.
- Filed: `ufo-antiname-without-anti-entry-unresolvable` (medium),
  `ufo-model-particles-public-field` (low).
- Not filed: subprocess labels print UFO names where MadGraph prints defaults
  (human-facing only, no gate reads them); `--noprefix` acceptance (no card
  needs it yet).
