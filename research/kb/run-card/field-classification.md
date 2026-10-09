---
type: Design
title: "Every run-card field is classified: consumed, benign or refused"
description: "FIELD_CLASSES gives each of the 209 run-card names Consumed, IgnoredBenign (with a positive inertness argument) or IgnoredPhysics (refused off default); audit method and blind spots."
status: draft
tags: [run-card, classification, hard-errors, audit, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n29-c20, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3847-L3876", title: "Note 29 C2.0, the measured trigger"}
  - {id: n29-c21, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3877-L3904", title: "Note 29 C2.1, audit method"}
  - {id: n29-c22, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3905-L3965", title: "Note 29 C2.2, the classification asserted"}
  - {id: n29-c23, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L3966-L4024", title: "Note 29 C2.3, the audit table"}
  - {id: n29-c25, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L4122-L4146", title: "Note 29 C2.5, opaque defaults"}
  - {id: n29-c27, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L4164-L4256", title: "Note 29 C2.7, acceptance tests"}
  - {id: n29-c29, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L4291-L4338", title: "Note 29 C2.9, risks and the residual blind spot"}
  - {id: n29-b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L5523-L5756", title: "Note 29 chain B results, the SDE_strategy and tmin_for_channel rulings"}
  - {id: n29-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L6088-L6125", title: "Note 29 close-out, per-chain verdicts"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1, matching fields moved to Consumed"}
---

# Every run-card field is classified

A parameter MadGraph acts on and vibegraph silently drops is a wrong answer.
So every name the parser recognises ([run-card/run-card-parser-and-defaults](run-card-parser-and-defaults.md))
has exactly one row in `FIELD_CLASSES` (`vibegraph-lib/src/runcard/classes.rs`)
saying what happens to it, and the rows that cannot be honoured become hard
errors. This implements the release-scope rule that unsupported surfaces are
hard errors ([pipeline/release-scope](../pipeline/release-scope.md)).

## The three classes

```rust
pub enum FieldClass {
    Consumed(&'static str),          // read; the string names the consumer
    IgnoredBenign(&'static str),     // not read, and unable to reach σ, the record or the cuts;
                                     // the string argues why
    IgnoredPhysics { why: &'static str, when: Applicability },
                                     // not implemented and able to change the output;
                                     // refused off MadGraph's default
}
```

Counted from the table (209 rows, one per `PARAM_DEFAULTS` name): **139
Consumed**, 48 of which are cuts that are parsed and detected rather than
applied (an active value is already a `CutError::UnimplementedCutActive`,
[run-card/madgraph-cut-conventions](madgraph-cut-conventions.md)); **49
IgnoredBenign**; **21 IgnoredPhysics**. Recount from the code rather than
trusting this page; `ignored_physics_fields_are_refused` asserts the 21.

The 21 refused fields: `time_of_flight`, `polbeam1/2`, `nb_proton1/2`,
`nb_neutron1/2`, `mass_ion1/2`, `bias_module`, `ktscheme`, `chcluster`,
`custom_fcts`, `lhe_version`, `boost_event`, `event_norm`, `nhel`, `limhel`,
`fixed_couplings`, `tmin_for_channel`, `small_width_treatment`. Each carries
its reason in the code; a few that are easy to get wrong:

| field | why it is refused |
|---|---|
| `ktscheme` | at 2, `cluster.f` uses Pythia's `pydj`/`pyjb` in place of the `dj`, `djb`, `zclus` implemented here; its initial-state branch reads `ickkw == 2 .or. ktscheme == 2`, so it bites with matching off |
| `chcluster` | restricts the clustering to the integration channel's own diagram; the test sits outside any matching switch |
| `small_width_treatment` | the clustering applies the width floor at a hardcoded 1e-6, and MadGraph applies it to the propagators at generation time; reading the card would move the clustering without moving the matrix element |
| `tmin_for_channel` | see below |
| `nhel` | helicity Monte Carlo changes the estimator and the per-event weight ([feature/nhel1-run-cards-refused](../backlog/feature/nhel1-run-cards-refused.md)) |
| `event_norm` | normalisation of `XWGTUP` ([events/lhef-weight-strategy](../events/lhef-weight-strategy.md)) |
| `fixed_couplings` | MadGraph itself stops on `False` ("form factor with fixed_couplings not supported anymore") |
| `polbeam1/2` | polarised beams and the `SPINUP` they imply are not implemented; refused by the generic loop, there is no dedicated error variant |

`Applicability` has a `ProtonBeams` variant for a field that can only bite when
both beams carry a PDF; no row uses it today (the per-beam PDF labels it was
written for are now consumed, resolved as `banner.py`'s `PDLabelBlock` does).

## Two rulings worth their reasons

**`SDE_strategy` is Consumed.** It looks like a directive for MadEvent's own
integrator, which this crate replaces, but it is not benign: it decides the
per-configuration weight that both the clustering-scale configuration draw and
the colour-flow configuration draw follow — `AMP2_c` at 1, `get_channel_cut`'s
propagator-denominator product at 2 (`EventScaleSource::weights_configurations_by_amp2`)
— so it reaches every per-event quantity that follows a configuration
([scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md),
[events/colour-and-helicity-selection](../events/colour-and-helicity-selection.md))[^n29-b].

**`tmin_for_channel` stays IgnoredPhysics although it is read.** Off its default
it multiplies `get_channel_cut` by an exponential suppression of spacelike
lines below `t/s_tot = tmin`, which vibegraph does not form (and at
`SDE_strategy = 1` MadGraph's `genps.f` reads an uninitialised `t` in that
factor). The field is read by `weights_configurations_by_amp2`, but reading is
not implementing: reclassifying it as consumed would remove the refusal and let
such a card integrate silently under a rule that does not describe it. A hard
error is strictly stronger than a silent fallback.
`the_configuration_draw_needs_both_run_card_fields` (`hadronic.rs`) asserts both
guards[^n29-b].

The MLM block moved to Consumed when matching was implemented: `ickkw`, `xqcut`,
`alpsfact`, `asrwgtflavor`, `auto_ptj_mjj`, `use_syst`, `pdfwgt`.
`highestmult`, `clusinfo` and `pdgs_for_merging_cut` stay benign (read only on
branches never reached at `ickkw = 1`, or CKKW-L bookkeeping), and
`ktdurham`/`ptlund`/`dparameter` stay refused cuts
([run-card/matching-parameters](matching-parameters.md))[^n41-m1].

## The audit method

A literal-name search **under-reports badly**: `Cuts::compile` builds names at
run time (`format!("pt{c}")`, `dr{tag}`, `mm{tag}` over the letter classes and
pair tags), so most of the cut block has a consumer and no literal occurrence.
The audit that survives is three passes[^n29-c21]:

1. a literal sweep for `"<name>"` over the workspace sources;
2. a typed-field sweep for the 14 `RunCard` fields read as `rc.<field>`
   (this found that `iseed` has no consumer: the seed comes from the CLI);
3. a constructed-name sweep, expanding every `format!` passed to
   `RunCard::float`/`int`/`get` over its generator set — the pass no LSP query
   can do.

Where a field has no consumer, the evidence is a **positive** argument for
inertness — an existing guard that makes it unreachable, or a reading of
MadGraph's source showing it cannot enter σ or the record — never "no consumer
found". The strings are written to be checked one at a time against MadGraph's
source; doing exactly that against `cluster.f` caught two misclassified fields
in review, which is why every field whose argument is not uniform across a
block carries its own string[^n29-close].

The audit's corpus measurement found no gated reference resting on an unread
physics-relevant field: every name any banked card sets away from its default
was either consumed or dispositioned, so the classification is prophylactic
rather than corrective[^n29-c20][^n29-c23]; `banked_run_cards_are_accepted`
keeps every banked card parsing.

## Enforcement

`refuse_ignored_physics` runs in `RunCard::from_values` **after** the beam
check (which is what makes `ProtonBeams` decidable), returning
`RunCardError::UnsupportedField { name, value, default, why }`. It only rejects;
it never derives or rewrites a value, so it cannot move any σ row, and
`RunCard` gained no field, so no artifact format changed[^n29-c22][^n29-c29].

## Tests and their blind spots

| test | fails on | provably cannot detect |
|---|---|---|
| `every_run_card_field_is_classified` (`classes.rs`) | a name without a row, a stale row, a duplicate, an empty reason | a **wrong** classification — a physics field parked as benign passes this and every other test here; the reason strings are the oracle, read by a human |
| `ignored_physics_fields_are_refused` | an `IgnoredPhysics` row without enforcement | interactions unsafe only jointly (one field perturbed at a time) |
| `banked_run_cards_are_accepted` (`validate_scales.rs`; hermetic sibling `the_committed_run_cards_are_accepted` in `scales_run_cards.rs`) | an enforcement rejecting a card a banked reference ran with — the most likely defect | an enforcement that is too weak |
| `opaque_defaults_known_to_differ_from_banner_py` | a MadGraph bump that changes which opaque defaults differ | whether those fields are individually harmless (that rests on their benign reasons) |

The failure and blind-spot statements are the design's own[^n29-c27].

**Residual blind spot.** Without MadGraph's `user_set` tracking, "the card
wrote this field" and "the field differs from its default" are one predicate,
so a card writing a field *at* its default reads as one omitting it. For a
refused field that is correct. It bites in one constructed case: a hand-written
card setting both `pdlabel` and `pdlabel1 = pdlabel2 = nn23lo1` (the default)
resolves to `pdlabel`, where MadGraph would take `nn23lo1`; MadGraph's own
writer never emits both[^n29-c29].

**Opaque defaults.** Benign classification is also what keeps the three
opaque fields whose stored default differs from MadGraph's from being
enforced; 35 banked cards write MadGraph's own `mxx_only_part_antipart`, which
reads here as an override. The fix belongs before any enforcement covers them:
[hygiene/runcard-opaque-defaults-unverified](../backlog/hygiene/runcard-opaque-defaults-unverified.md)[^n29-c25].

[^n29-c20]: Note 29 C2.0, the trigger measured on the banked corpus.
[^n29-c23]: Note 29 C2.3, the per-field audit table (its counts predate the matching and SDE_strategy reclassifications).
[^n29-c21]: Note 29 C2.1, the three-pass audit and the positive-evidence rule.
[^n29-c22]: Note 29 C2.2, the enum, the enforcement point, the deliberate non-changes.
[^n29-c25]: Note 29 C2.5, opaque defaults.
[^n29-c27]: Note 29 C2.7, T1–T4 with their failure and blind-spot statements.
[^n29-c29]: Note 29 C2.9, what it cannot break and the `user_set` blind spot.
[^n29-b]: Note 29 chain B results, the rulings on `SDE_strategy` and `tmin_for_channel`.
[^n29-close]: Note 29 close-out, chain C2's verdict.
[^n41-m1]: Note 41 M1, the run-card changes for matching.
