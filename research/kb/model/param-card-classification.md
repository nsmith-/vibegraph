---
type: Design
title: "Every param-card construct is classified: consumed, benign or refused"
description: "PARAM_CARD (ufo/card_classes.rs) lists each construct MadGraph's SLHA reader accepts in a run's param card with its class and an example the test binds; decay tables, negative masses, scan values and extra DECAY fields are refused, XSECTION is skipped as MadGraph skips it."
status: draft
tags: [param-card, classification, hard-errors, madgraph-parity, slha]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---

# Every param-card construct is classified

The run card's rule ([run-card field classification](../run-card/field-classification.md))
holds for the param card too: a construct MadGraph acts on and vibegraph
silently drops is a wrong answer. `PARAM_CARD` (`vibegraph-lib/src/ufo/card_classes.rs`)
has one row per construct MadGraph's reader accepts (`ParamCard.read`,
`models/check_param_card.py` at the pin) and the readers of its result for a
run (`write_inc_file`, `update_dependent`, the `param_card_rule` check). Each
row carries a class, what MadGraph does with the line number, and an example.
The classes are the run card's, with "refused" in place of `IgnoredPhysics`:

- **Consumed**: read, and it moves the run.
- **IgnoredBenign**: accepted, with a written argument that it cannot reach σ,
  a weight or the event record.
- **Refused**: an error naming the construct; every refused entry of a card is
  reported at once (`SlhaError::Several`, `CardError::Several`).

The reader is `ufo::slha::ParamCard` and the binding `ufo::bind::bind_card`
([restriction semantics](restriction-semantics.md) for what the restriction
fixes).

## The table

| construct | MadGraph | class |
|---|---|---|
| comment or blank line | skipped (L353) | benign: comments are read only by the edit card's `set <name>` shortcut |
| trailing comment | split off (L65) | benign: entries are read by block and code |
| case of `BLOCK`, `DECAY` and block names | lines lower-cased (L355) | benign |
| entry at an external parameter | written to the Fortran by block and code | consumed |
| `DECAY <pdg> <width>` at an external width | block `decay` (L362-L372) | consumed |
| `BLOCK <name> Q= <scale>` | read only as `mdl__<block>__scale` of a model with running elements | benign: the loader reads no `running.py` |
| `BLOCK QNUMBERS <pdg>` | not a parameter (model_reader L102) | benign: the `<slha>` header writes the model's |
| `(block, code)` repeated at the same value | dropped (L240-L244) | benign |
| entry for a computed mass or width | overwritten (`update_dependent`) | benign, reported in `BoundCard::ignored` |
| line before the first block | skipped (L380) | benign |
| `XSECTION` block | rows unparsed up to the next `BLOCK`/`DECAY` (L374-L377) | benign: the reader skips the same rows |
| `(block, code)` repeated at another value | "is already define" | refused |
| block opened twice | the second replaces the first | refused |
| entry the model has no parameter for | never read | refused (`CardError::Unknown`) |
| non-integer code | line dropped with a warning | refused |
| value not a number (Fortran `d` included) | kept as a string | refused |
| non-finite value | read | refused (`NonFinite`) |
| `scan:` value | one run per value | refused |
| `DECAY <pdg> Auto` | MadWidth at launch | refused (`AutoWidth`) |
| fields after a `DECAY` width | the width becomes a string | refused |
| decay table (branching-ratio rows) | written into the `<slha>` header, read by a shower | refused (`DecayTable`) |
| negative mass | the width is negated with it (`write_inc_file` L692-L694) | refused (`NegativeMass`) |
| value at a parameter the restriction fixed to zero | pruned | refused (`RevivesZeroed`) |
| value off one at a parameter fixed to one | `param_card_rule` | refused (`MovesFixedOne`) |
| one member of a merged group moved alone | the others follow the first | refused (`SplitsMerged`) |

The code is the authority; recount from `PARAM_CARD` rather than this page.

## Rulings worth their reasons

**`XSECTION` is benign, not refused.** MadGraph's reader tests `block` and
`decay` before `xsection` (L356, L362, L374), so a header after an `XSECTION`
block ends it and the blocks after it are read. The reader here does the same.

**A decay table is refused.** No vibegraph σ or weight reads a branching
ratio, but MadGraph writes the card it ran with into the event file's
`<slha>` header, and a shower reading SLHA decay tables decays the states it
treats with them. The header written here comes from the model
(`lhef::write::slha_block`) and has none, so a sample would shower
differently. The `DECAY` width stays: remove the table and the card binds.

**A negative mass is refused.** MadGraph's `write_inc_file` negates the
particle's width when its card mass is negative, which keeps the propagator's
`M·Γ` positive. The propagators here use the width as given, so the imaginary
part of the denominator would flip sign.

**An entry the model lacks is refused although MadGraph never reads it.** A
mistyped block or code would otherwise leave the parameter the card meant at
its default, silently.

## Tests and their blind spots

| test | fails on | cannot detect |
|---|---|---|
| `every_param_card_construct_behaves_as_classified` (`card_classes.rs`) | a row whose example does not behave as its class says: a benign row that moves the bound point, a consumed one that does not, a refused one accepted or refused for another reason | a construct missing from the table, and a wrong class with a matching example; the MadGraph column is read by a human |
| `input_card_refusals.rs` | each new refusal (decay table, extra `DECAY` field, scan, negative mass, every problem at once) and the `XSECTION` skip; all failed on the code before them | refusals that are too broad |

The test binds to the default SM with `MT`/`ymt` tied and `WT` held at one,
because no SM restriction merges or fixes to one. "Moves the result" compares
every parameter's value bit for bit and the moved list.
