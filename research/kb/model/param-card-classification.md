---
type: Design
title: "Every param-card construct is classified: consumed, benign or refused"
description: "PARAM_CARD (ufo/card_classes.rs) lists each construct MadGraph's SLHA reader accepts in a run's param card with its class and an example the test binds; decay tables are carried to the <slha> header, a negative mass is refused unless the particle is a Majorana fermion (whose width is negated as in MadGraph's Fortran), scan values and extra DECAY fields are refused, XSECTION is skipped as MadGraph skips it."
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
| decay table (branching-ratio rows) | written into the `<slha>` header, read by a shower | consumed: recorded in the artifact, written below the `DECAY` line |
| decay table not summing to one, or naming a particle the model lacks | read as numbers, nothing checks it (`load_decay` L100-L120) | consumed: carried as written |
| decay channel repeated at the same branching ratio | dropped (`Block.append` L240-L244) | benign: carried once |
| decay channel given two branching ratios | the first kept silently (L391-L395) | refused: a shower reads both rows |
| decay row whose `NDA` is not its daughter count | `NDA` dropped (L115-L117) | refused: a shower reads `NDA` |
| decay row with no daughter, or a field not a number | a channel of none, or an exception | refused |
| decay table below a negative code | kept under the antiparticle's code | refused (`DecayTableParticle`): the header has no line for it |
| negative mass on a particle that is not a Majorana fermion | the width at the same code is negated with it (`write_inc_file` L692-L695) | refused (`NegativeMass`), from a default, a restriction or a card: the code downstream of the amplitudes is not validated for it |
| negative mass on a Majorana fermion | the same | binds; the evaluation negates the width the same way (no amplitude reads it: Majorana fermions are refused at diagram generation) |
| value at a parameter the restriction fixed to zero | pruned | refused (`RevivesZeroed`) |
| value off one at a parameter fixed to one | `param_card_rule` | refused (`MovesFixedOne`) |
| one member of a merged group moved alone | the others follow the first | refused (`SplitsMerged`) |

The code is the authority; recount from `PARAM_CARD` rather than this page.

## Rulings worth their reasons

**`XSECTION` is benign, not refused.** MadGraph's reader tests `block` and
`decay` before `xsection` (L356, L362, L374), so a header after an `XSECTION`
block ends it and the blocks after it are read. The reader here does the same.

**A decay table is carried.** No vibegraph σ or weight reads a branching
ratio, but MadGraph copies the card it ran with, verbatim, into the event
file's `<slha>` header (`banner.py` `add`), and a shower reading SLHA decay
tables decays the states it treats with them. `integrate` records the tables
in the artifact (`RunInputs::decays`, artifact version 18) and `generate`
writes them below their particles' `DECAY` lines in MadWidth's layout
(`update_width_in_param_card`): a `#  BR  NDA  ID1 ...` heading, then per
channel the branching ratio, the daughter count, the daughters as the card
lists them, and the partial width as a comment. MadGraph checks neither the
sum nor the daughters, so neither is refused. What is refused is a row
MadGraph and a shower would read differently: an `NDA` that is not the
daughter count, a channel given two branching ratios, and a table under a
negative code, which the header (one `DECAY` line per particle, positive code)
has no line for.

Measured against MadGraph (the pinned checkout, `e+ e- > mu+ mu-` launched
with a card carrying tables for t, W, H and τ): MadGraph's header is the card
byte for byte. vibegraph's differs in what it rewrites: each value is
re-printed (`%e` where that reads back exactly), the comment of each row is
the partial width rather than the card's comment, the column heading is
written for every table (MadGraph writes it only where the card does), and
lines of the card that are not a parameter, a table row or a heading are not
carried.

**A negative mass is refused off a Majorana fermion** (user decision,
2026-11-04, amending G7). MadGraph's `write_inc_file`
(`check_param_card.py` L692-L695) writes every external width at
`DECAY [pdg]` negated into the Fortran when the card's mass at `[pdg]` is
negative, which keeps a propagator's `M·Γ` positive. `EvaluatedModel` applies
the same rule to every evaluation, and binding (`bind_card`, `bind_recorded`,
and `integrate` without a card, which binds the defaults) refuses a negative
mass on any particle that is not a Majorana fermion
(`CardError::NegativeMass`), whether the model's default, a restriction card
or the param card gives it. The reason is what sits downstream of the
amplitudes: the Breit-Wigner phase-space map (`bw_scale` reads `M·Γ`), the
on-shell checks, the cuts and the LHEF resonance record (which treats a
width at or below zero as none) were never compared with MadGraph for a
negative mass. Every negative mass in a shipped MadGraph model is a Majorana
fermion's, and diagram generation refuses Majorana fermions, so after the
refusal no negative mass reaches those sites. The header writes the card's width.

The rule stays pinned below the refusal. The pin is the `ee_to_mumu_negmz` row of `standalone_jamps`: `e+ e- > mu+ mu-`
at 90 GeV with `MZ = -91.188`, MadGraph's amplitudes evaluated at the
parameters MadEvent's Fortran reads (the generator runs MadGraph's own
`write_inc_file` and points the standalone model's `param_read.inc` at its
output, as `output madevent` does). Per diagram and per helicity it agrees to
1.2e-13 of the largest graph (flows 1.1e-13, |M|² 1.4e-13); on the code before the rule the photon/Z relative phase was wrong
(worst graph 1.45e-1, |M|² 8.2e-3). The row binds its card with
`EvaluatedModel::from_model_card`, below `bind_card`'s refusal, as every
`standalone_jamps` row does.

MadGraph's standalone Fortran applies a different rule, `width =
sign(width, mass)` for Majorana fermions only (`export_v4.py` L9793-L9800),
and its Python model evaluation `-abs(width)` for the same set
(`model_reader.py` L241-L248). They agree with `write_inc_file` for a positive
width on a Majorana fermion, which is every negative mass in a shipped
MadGraph model (`MSSM_SLHA2`'s `Mneu3 = -363.756`, in all four of its
restrictions). They disagree for a negative mass on any other particle, which
binding refuses. The MSSM case
itself cannot be evaluated here: diagram generation refuses a Majorana
fermion on any line (`ConvertError::MajoranaFermion`).

**An entry the model lacks is refused although MadGraph never reads it.** A
mistyped block or code would otherwise leave the parameter the card meant at
its default, silently.

## Tests and their blind spots

| test | fails on | cannot detect |
|---|---|---|
| `every_param_card_construct_behaves_as_classified` (`card_classes.rs`) | a row whose example does not behave as its class says: a benign row that moves the bound point, a consumed one that does not, a refused one accepted or refused for another reason | a construct missing from the table, and a wrong class with a matching example; the MadGraph column is read by a human |
| `input_card_refusals.rs` | each refusal (extra `DECAY` field, scan, `NDA`, a channel given twice, a table under a negative code, a negative mass from a card, a restriction or a default, every problem at once), a carried decay table, a Majorana negative mass that binds with its width negated (`MSSM_SLHA2`), and the `XSECTION` skip; each refusal failed on the code before it | refusals that are too broad |
| `standalone_jamps::ee_to_mumu_negmz` | a width evaluated with the wrong sign beside a negative mass, per diagram and per helicity against MadEvent's parameters | a rule that differs only for a width the restriction fixed, a negative card width, or a mass that is not external (only `MZ` is moved) |
| `slha_block` tests (`lhef/write.rs`), `decay_tables_are_recorded_and_written_into_the_header` (`cli_param_card.rs`) | a table missing from the header, under the wrong line, or moving the run | a layout a particular shower parses differently from MadWidth's |

The test binds to the default SM with `MT`/`ymt` tied and `WT` held at one,
because no SM restriction merges or fixes to one. "Moves the result" compares
every parameter's value bit for bit and the moved list.
