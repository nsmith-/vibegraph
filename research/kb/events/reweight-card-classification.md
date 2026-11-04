---
type: Design
title: "Every reweight-card construct is classified: consumed, benign or refused"
description: "REWEIGHT_CARD (reweight/classes.rs) lists each command and option MadGraph's reweight module accepts, with its class and an example the test parses and resolves; change helicity False and three inert change options are accepted, a command after done, a parameter fixed to one and a split merge are refused."
status: draft
tags: [reweight, classification, hard-errors, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
---

# Every reweight-card construct is classified

A `reweight_card.dat` is a command file for MadGraph's reweight module
(`madgraph/interface/reweight_interface.py` at the pin), with each `launch`
block's parameter edits read by the edit-card interface (`AskforEditCard`,
`common_run_interface.py`). `REWEIGHT_CARD` (`vibegraph-lib/src/reweight/classes.rs`)
gives each command and option a class with the meaning of the param card's
([param-card classification](../model/param-card-classification.md)) and an
example the test parses (`reweight::card`) and resolves (`reweight::resolve`)
on the same test model. Every line the parser refuses is reported at once
(`CardError::Several`).

## The table

| construct | MadGraph | class |
|---|---|---|
| comment, blank line | skipped | benign |
| `launch` | one hypothesis | consumed |
| `launch` with no change | a weight at the card's point | consumed |
| `--rwgt_name=<id>` | the weight's id | consumed |
| `--rwgt_info=<text>` | the weight's description | consumed |
| other `launch` options (`--keep_card`) | stored, or the previous card reused | refused |
| a `--rwgt_name` used twice | the later replaces the earlier | refused |
| a `--rwgt_name` outside `[A-Za-z0-9_.-]` | written as given | refused |
| `set [param_card] <block> <code…> <value>` | the param card's entry | consumed |
| `set <name> <value>` | the entry named in the default card's comments | consumed |
| `param_card` keyword, name vs address | one entry | benign |
| one parameter set twice | the later wins | benign |
| Fortran `d` exponent | a number | benign |
| a value not a number (`Auto`, `default`) | MadWidth, or the default | refused |
| a `scan:` value | one hypothesis per value | refused |
| `set x = v`, `set x=v` | the `=` dropped | refused |
| unknown or internal parameter | skipped, or overwritten | refused |
| parameter fixed to zero | not in the reweighting model | refused (`Locked`) |
| parameter fixed to one, set elsewhere | not in the reweighting model | refused (`FixedOne`) |
| one member of a merged group moved alone | the others follow the first | refused (`SplitsMerged`) |
| parameter that moves `aS` | reweighted with the event's `aS` | refused (`StrongCoupling`) |
| mass of an external particle | momenta moved to the new shell | refused (`ExternalMass`, in the engine) |
| `set` before the first `launch` | a `launch` inserted, with a warning | refused |
| a param-card path | that card is the hypothesis | refused |
| `change helicity False` | the helicity-summed ratio | benign: the ratio `generate` writes |
| `change helicity True` | the ratio in the event's own helicity (MadGraph's default) | refused |
| `change output default` | weights added to the events | benign |
| `change output 2.0` / `unweight` | one file per hypothesis | refused |
| `change rwgt_dir`, `change multicore` | MadGraph's working directory and parallelism | benign |
| `change model`, `process`, `mode`, `systematics`, `keep_ordering`, `use_eventid`, `allow_missing_finalstate`, `boost`, `tree_path`, `virtual_path`, `soft_threshold`, `identical_particle_in_prod_and_decay`, `include_sudakov` | each changes what is evaluated or written | refused |
| `change` with an unknown option | the line is discarded | refused |
| `done`, `exit`, `quit` with nothing after | end | benign |
| a command after `done`, `exit` or `quit` | still run: the card is read to its end | refused (`AfterEnd`) |
| any other command (`compute_widths`, `import`, …) | the module's or the edit card's | refused |
| a card with no `launch` | nothing | refused |

## Rulings worth their reasons

**A command after `done` is refused.** `done` ends an edit-card question, not
the file: `import_command_file` runs every line to the end, so a `launch` after
`done` is a hypothesis MadGraph writes. The parser used to stop reading there.

**Fixed-to-one and merged parameters are refused.** MadGraph builds its
reweighting library from the restricted model, in which a parameter fixed to
one is gone and a merged member is its group's first times a factor; moving
either alone changes nothing in MadGraph's weight. `bind_card` already holds a
param card to both rules; the reweight resolver did not.

**MadGraph's default helicity treatment is not refused.** A card with no
`change helicity` line asks MadGraph for the per-helicity ratio, and
`generate` writes the helicity-summed one. Both are unbiased for σ, and the
summed ratio is the documented behaviour
([reweight-helicity-summed-only](../backlog/feature/reweight-helicity-summed-only.md)).
Refusing every card without `change helicity False` is a decision for that
item; this table records the divergence and accepts the explicit line.

## Tests and their blind spots

| test | fails on | cannot detect |
|---|---|---|
| `every_reweight_card_construct_behaves_as_classified` | a row whose example resolves otherwise than its class says (ids, descriptions and values compared) | a construct missing from the table; a refusal's reason (the needle is often the line itself) |
| `input_card_refusals.rs` | the new refusals and acceptances on a fixture model with a merged pair and a fixed one; the refusals failed on the code before them | an option refused for the wrong reason |
