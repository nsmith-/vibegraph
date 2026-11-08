---
type: Backlog Item
title: "A param card's mass or width for a particle the model lacks is refused; MadGraph ignores it"
description: "DarkPhoton's gridpack card sets MASS 1000022 (chi), absent from the model; bind_card refuses it as CardError::Unknown, while MadEvent's reader skips any entry not in ident_card.dat silently."
area: feature
state: open
priority: medium
closes_when: "A MASS or DECAY entry at a PDG code with no model particle binds as benign, listed in BoundCard::ignored and warned, while unknown codes in parameter blocks stay refused (or the user rules otherwise); the classification row is updated, a test pins it, and DarkPhoton runs through gpdb with its param card."
blocked_by: []
opened: 2026-11-08
tags: [param-card, refusal, madgraph-parity, cms, darkphoton]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: closeout, resource: "../../sprints/cms-gridpack-coverage/closeout.md", title: "Sprint H close-out draft: the closing rerun, DarkPhoton caveat"}
  - {id: lha-read, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/template_files/fortran/lha_read.f#L116-L176", title: "lha_read.f: LHA_blockread keeps only entries named in ident_card.dat"}
  - {id: rule, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/check_param_card.py#L1308", title: "check_param_card.py: ParamCardRule.check_param_card checks restriction entries only"}
  - {id: classes, resource: "../../model/param-card-classification.md", title: "Param-card classification: entry the model has no parameter for"}
---
The DarkPhoton gridpack's `param_card.dat` sets a mass for PDG 1000022 (`chi`),
which its model does not have. `ufo::bind::bind_card` refuses it as
`CardError::Unknown` (`bind.rs`, the `("mass", [pdg])` arm, about line 386).
The classification table makes this a deliberate row: "entry the model has no
parameter for | never read | refused". It gives no ruling for it.

MadGraph at the pin ignores such entries without a message:
- MadEvent's Fortran reader matches each card line against the model's
  `ident_card.dat` (`lha_read.f:116–176`, `LHA_blockread`). A line with no
  match is never recorded. Only a model parameter missing from the card warns
  (`:334–341`).
- On the Python side, `check_valid_param_card` → `ParamCardRule.check_param_card`
  (`check_param_card.py:1308`, `:1836`) checks the restriction's fixed entries
  only.

Refusing guards against a mistyped code, which would otherwise move nothing.
But a mass or width for a particle the model lacks cannot reach σ, and
gridpacks built from a larger model's card carry them. Accepting those two
blocks as benign, with a warning, keeps the typo guard for parameter blocks.
This is a ruling to make. The DarkPhoton cards run correctly with the entry
trimmed by hand (0.985 and 0.971 of MadGraph).

gpdb's fallback for this refusal is a separate problem:
[gpdb-compares-sigma-after-physics-changing-patches](../validation/gpdb-compares-sigma-after-physics-changing-patches.md).
