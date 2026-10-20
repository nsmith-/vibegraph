---
type: Backlog Item
title: The CLI cannot bind a model to a param card
description: integrate and generate always run at the restriction card's defaults; there is no --param-card option, though the library already reads SLHA cards.
area: feature
state: open
priority: high
closes_when: "`integrate --param-card` binds the model via `EvaluatedModel::from_model_card`, the artifact records the card and `generate` refuses a different one, and a moved-parameter run matches MadGraph's σ at the same card."
blocked_by: []
opened: 2026-10-06
tags: [cli, param-card, slha, reweight, docs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L856-L882", title: "TODO.md entry T072"}
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
The library reads SLHA cards (`ufo/slha.rs`, `ParamCard: FromStr`) and binds a model
to one with `EvaluatedModel::from_model_card` (`vibegraph-lib/src/ufo/mod.rs:562`).
The tests and the MadGraph reweight oracle use this path.

Nothing in `vibegraph-cli` references a param card. Every `integrate` and
`generate` run therefore computes at the selected restriction card's values. The
only command-line ways to move a parameter are another restriction or
`generate --reweight-card` hypotheses. A param-card path inside a `launch` block
is refused (`vibegraph-lib/src/reweight/card.rs`).

The [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
makes this a silent wrong σ: 54 of 65 LO CMS cards move the param card (top
mass and Yukawa, MW, MZ, widths, 1/α, G_F), 53 through customizecards \`set\`
lines. As shipped, card 075 is ×12 off MadGraph and 072 ×5,200; with the card
baked into the UFO and the gridpack's PDF, both agree with MadGraph.
customizecards are out of scope: they rewrite the param card before generation,
so a gridpack's final \`Cards/param_card.dat\` already carries them.

Scope (user, 2026-10-06):

- `integrate --param-card <file>` binds through `from_model_card`.
- The artifact records the card (contents or digest) beside the model identity.
  `generate` uses the same card or refuses, as it already does for model and
  process.
- `generate --reweight-card` takes that card as its card point: hypothesis
  denominators and the per-event audit run against it.
- A card that sets a parameter the restriction zeroed to a non-zero value is
  refused, because the restriction also pruned that parameter's vertices.
- Validation:
  - a card equal to the restriction's defaults reproduces the default run bit for
    bit;
  - a moved `MZ` or `ymt` matches MadGraph's σ at the same card. The
    `reweight_mg_oracle` rows already carry MadGraph's cards.

Two docs passages read as if the CLI took a card today and need correcting until
this lands: `docs/src/cli/overview.md:114` ("a run with no param card of its own …")
and `docs/src/guide/02-ufo.md:95` ("the `param_card.dat` read at run time supplies
the values the run actually uses").
