---
type: Backlog Item
title: "MadGraph's PDG-based default particle names are not provided"
description: "MadGraph renames particles by PDG code from particles_name_default.txt unless --modelname is given; vibegraph does neither, so mu+ on loop_sm fails and --modelname is refused."
area: feature
state: open
priority: medium
closes_when: "A model's particles answer to MadGraph's default names by PDG code (mu+ on loop_sm, u~ and mu- on BSM UFOs, by a test), and 'import model X --modelname' is accepted and switches only that renaming off."
blocked_by: []
opened: 2026-10-20
tags: [process-grammar, model, particle-names, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
MadGraph renames each particle whose PDG code is listed in
`input/particles_name_default.txt` to the default name, unless the model is
imported with `--modelname` (`base_objects.py:1577`). vibegraph resolves the
UFO's own names only (`ResolveError::UnknownParticle`,
`vibegraph-lib/src/diagrams/resolve.rs:28`), and refuses any `import model`
option as `Unsupported::ModelOption`
(`vibegraph-lib/src/diagrams/check.rs:483`, marked "not planned" in the
module table at `:30`).

Measured in the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md):
`mu+` on `loop_sm`, whose muon is `m+` (cards 003, 021); `u~` on
InertDoublet (064); `mu-` on HC_NLO_X0 (036). `--modelname` is refused on 5
cards, though all it does is keep the UFO's names.
