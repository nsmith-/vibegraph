---
type: Design
title: "Every restrict-card construct is classified: consumed, benign or refused"
description: "RESTRICT_CARD (ufo/card_classes.rs) lists each construct MadGraph's restriction reads, with its class and an edit of the SM's restrict_default the test restricts; a missing parameter, MadGraph's unfixed one and zero, a non-finite value and a param_<x>.dat beside the card are refused."
status: draft
tags: [restrict-card, classification, hard-errors, madgraph-parity, ufo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---

# Every restrict-card construct is classified

A restriction card goes through the same reader as a param card
([param-card classification](param-card-classification.md)), and is then read
by MadGraph's `model_reader.set_parameters_and_couplings` and
`RestrictModel.restrict_model` (`models/import_ufo.py` at the pin).
`RESTRICT_CARD` (`vibegraph-lib/src/ufo/card_classes.rs`) gives each construct a
class with the same meaning as the param card's, and an example: an edit of
the SM's `restrict_default.dat` that the test restricts the interned SM with.
The loader's side is `ParsedModel::apply_restriction` and `UFOModel::load`
([restriction semantics](restriction-semantics.md)). Every problem in a card is
reported at once (`UfoError::Several`).

## The table

| construct | MadGraph | class |
|---|---|---|
| comment, blank line, trailing comment | skipped | benign |
| entry at an external parameter | its value and the model's default | consumed |
| entry at zero | fixed to zero; zero couplings pruned | consumed (`zeros`, pruning) |
| entry at exactly one | fixed to one | consumed (`ones`) |
| equal values up to sign in one block | merged | consumed (`merged`) |
| `DECAY <pdg> <width>` | the width's default; never merged | consumed |
| negative mass | read as the default | consumed |
| `BLOCK <name> Q= <scale>` | not read by the restriction | benign |
| `QNUMBERS` block | not a parameter | benign |
| entry or block the model lacks | never read; an unknown block warns | benign |
| decay table | not read | benign |
| `XSECTION` block | not read | benign |
| line before the first block | skipped | benign |
| `(block, code)` repeated at the same value | dropped | benign |
| `(block, code)` repeated at another value | "is already define" | refused |
| block opened twice | the second replaces the first | refused |
| an external parameter the card does not set | "not define" / "Invalid restriction card" | refused (`RestrictMissing`) |
| `9.999999e-1` or `0.000001e-99` | neither fixed nor merged, then 1 or 0 | refused (`RestrictMadgraphFree`) |
| non-finite value | read | refused (`RestrictNonFinite`) |
| `DECAY <pdg> Auto` | kept free for MadWidth | refused (`RestrictAutoWidth`) |
| fields after a `DECAY` width, a non-integer code, a non-number, a `scan:` value | a string, or a dropped line | refused |
| `param_<x>.dat` beside `restrict_<x>.dat` | its values become the defaults | refused (`RestrictParamSibling`) |

## Rulings worth their reasons

**A missing parameter is refused.** MadGraph raises "not define" for an entry
missing from a block the model uses, and "Invalid restriction card" for a
missing block, except when it can convert another convention (`loop` read from
`MZ`, SLHA1 to SLHA2, `alpha` to `fralpha`). The loader would keep the
`parameters.py` value. No banked restriction card misses an entry; the
fixture cards of `ufo_input_refusals.rs` were completed.

**An entry the model lacks is benign here and refused in a param card.** The
restriction reads only the model's parameters, and a model's own cards carry
entries for computed masses and widths (all 13 SMEFTsim cards carry 27, the
MSSM_SLHA2 cards 31), so refusing would refuse the shipped cards. In a param
card such an entry is a value the user meant to set.

**MadGraph's unfixed one and zero are refused, not ported.** Porting would set
the default to 1 or 0 while keeping the parameter out of `ones`, `zeros` and
`merged`. No banked card uses either value.

**`param_<x>.dat`.** `restrict_model` replaces every `restrict` in the card's
path by `param` and, when that file exists, takes the model's defaults from it.
SMEFTsim ships `param_card_massless.dat`, which that rule never names.

## A negative default mass

A negative mass is read as MadGraph's restriction reads it, and the
evaluation negates the width at the same code as MadGraph's Fortran does
([param-card classification](param-card-classification.md)), whether the mass
is a restriction default or a param card's. `MSSM_SLHA2`'s `restrict_default`
sets `Mneu3 = -363.756` with a width of 1.916, evaluated as -1.916 (a
neutralino is a Majorana fermion, which diagram generation refuses, so no
MSSM amplitude reads it yet).

## Tests and their blind spots

| test | fails on | cannot detect |
|---|---|---|
| `every_restrict_card_construct_behaves_as_classified` | a row whose example restricts otherwise than its class says (each vertex's couplings, each external's default, and the zero, one and merged sets compared) | a construct missing from the table; a wrong class with a matching example |
| `input_card_refusals.rs` | the four new refusals and the accepted decay table, on a fixture model; the refusals failed on the code before them | a refusal too broad for a card no test writes |
