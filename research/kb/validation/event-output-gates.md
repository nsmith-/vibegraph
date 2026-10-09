---
type: Validation Gate
title: LHEF, unweighting and generate gates
description: "Byte-for-byte LHEF round trip of the banked runs with mutation controls, validate_unweighting's sigma and shape checks, and end-to-end generate at fixed and proton beams; each blind spot."
status: draft
tags: [lhef, unweighting, generate, events, format-oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n23-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L281-L343", title: "Note 23 E2 — accept/reject and validate_unweighting"}
  - {id: n23-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L383-L561", title: "Note 23 E3 — the format oracle and its mutation controls"}
  - {id: n23-e4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L571-L712", title: "Note 23 E4 — generate, weight strategies and the replay pin"}
  - {id: n23-model, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L713-L804", title: "Note 23 — model identity in the artifact"}
  - {id: n24-p4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1954-L2010", title: "Note 24 P4 — generate at proton beams, the four gates"}
  - {id: n24-p4c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2011-L2041", title: "Note 24 P4 — plan corrections"}
  - {id: n26-home, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/26-refdata-compact-representation.md#L182-L203", title: "Note 26 — the byte-round-trip gate's home"}
  - {id: n36a-unw, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L278-L290", title: "Note 36a §4 — validate_unweighting thresholds and headroom"}
  - {id: code-lhef, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_lhef.rs", title: "vibegraph-lib/tests/validate_lhef.rs"}
  - {id: code-unw, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_unweighting.rs", title: "vibegraph-lib/tests/validate_unweighting.rs"}
  - {id: code-unweight, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/unweight.rs#L1-L45", title: "unweight.rs — why the channel is drawn ∝ w_max"}
  - {id: code-proton, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/tests/cli_generate_proton.rs", title: "vibegraph-cli/tests/cli_generate_proton.rs"}
  - {id: code-generate, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/generate.rs#L425-L445", title: "generate.rs — stochastic rounding refused on mixed multiplicity"}
---

# LHEF, unweighting and generate gates

Three groups of gates cover event output. None compares event *content* with
MadGraph's events: this crate does not share MadGraph's random numbers, so a
per-event comparison would compare unrelated phase-space points. Distribution
agreement with MadGraph's sample is [the samples gate](samples-gate.md); reading
the files back in a real shower is [Pythia interop](../events/pythia-interop.md).

## The format oracle: `validate_lhef`

`banked_files_round_trip_byte_for_byte` discovers every banked run's
`unweighted_events.lhe.gz`, parses `<init>` and every `<event>` into this
crate's record types, writes them back, and requires the record span to be
byte-identical[^n23-e3]. That pins field order, column widths, exponent spelling,
the `px py pz E` ↔ `[E, px, py, pz]` permutation and the sign of a negative zero
against files a real shower reads. The corpus grows with the bundle (37 runs,
744 759 events at `refdata-4`), so the test guards its own coverage: it fails
unless some run has a hadron-collider `<init>` (proton beam ids and an LHAPDF id
in `PDFSUP`), some run has colour lines on an incoming leg, and some run is in
MadGraph's *converted* dialect[^n24-p4].

The dialect guard matters. MadGraph delivers events either as its Python
post-processing reformatted them or passed through as the Fortran wrote them,
and the reader keeps the source text it was given; a pass-through run therefore
round-trips whatever this crate's own layout is. Only converted runs are layout
evidence, so the test re-serialises each run a second time with the source text
dropped, counts the runs whose bytes still match, and names one. The banked
files' format is the Python writer's (`lhe_parser.py`), not
`rw_events.f`'s `(i2,i5,e16.7e3,3e15.7)`, which writes the intermediate per-channel
files; see [MadGraph's LHE output](../references/codebases/madgraph-lhe-output.md).

`the_round_trip_is_sensitive_to_every_convention_sensitive_field` makes the round
trip evidence: on `gg_to_ttx` (every leg coloured in both slots) it applies eight
mutations — `MOTHUP` dropped, `MOTHUP` order swapped, `ISTUP` sign flipped on the
incoming legs, `ICOLUP` slots exchanged, incoming momenta crossed to outgoing,
the momentum tuple rotated, the mass replaced by the momentum's invariant,
`SPINUP` zeroed — and requires each to break the bytes.

`generated_events_serialise_into_a_coherent_file` writes a generated sample for a
colourless and a gluon-initiated `2 → 2` and reads it back: momentum balance,
on-shellness, mean `XWGTUP` against σ, and each event's colour lines are the
selected flow's.

The round trip stays in the banked layer reading the fetched bundle's raw
`.lhe.gz`; it reads raw text and nothing projected can serve it. Three runs reach
every layout this crate writes (`ee_to_mumu`, `gg_to_ttx`, `pp_to_llj_fixed`) if
the corpus ever has to shrink[^n26-home].

*Blind to*: which event was generated (it re-emits MadGraph's values); the
colour-line integers (only connectivity is physical); which helicity an event
should carry; `SCALUP` as μF rather than μR on real kinematics (equal for every
closed-form clustering; pinned by a hand-built unit test in `lhef::build`); a
self-consistently wrong format in the generated-file half, since reader and
writer share assumptions. A file is also a lossy record of its run: `<init>`
cross sections carry seven significant digits, `XWGTUP` eight, momenta eleven.
The record conventions themselves are [LHE record conventions](../events/lhef-record-conventions.md)
and [the LHEF reader and writer](../events/lhef-io-design.md).

## `validate_unweighting`

Does accept/reject over the frozen per-channel VEGAS grids reproduce the
integration it came from? Five fixed-beam rows (`ee_to_mumu`, `uux_to_uux`,
`gg_to_ttx`, `ee_to_tatah`, `ee_to_mumua`), each generated on five seeds
(`GEN_SEEDS`), through the production integrand and MadGraph's own run cards.
The banked MadGraph σ is not the reference: `validate_sigma` gates the integral,
and an unweighting bug does not move the integral, it moves which points are
kept. So the sample is compared with the VEGAS integral and with an independent
weighted estimator over the same grids that uses a *different* channel-selection
rule (`∝ αⱼ` with a `1/qⱼ` weight), so the reference cannot share the generator's
mistake[^n23-e2].

| threshold | class | bound | worst over five seeds | headroom |
|---|---|---|---|---|
| `SIGMA_PULL_LIMIT` (seed-mean σ vs VEGAS / weighted reference) | standardised | 3.5 | 1.56 (`ee_to_tatah` vs VEGAS) | 2.2× |
| `SIGMA_REL_LIMIT` | tolerance | 0.03 | 5.77e-3 (`ee_to_mumua` vs weighted) | 5.2× |
| `SHAPE_CHI2_LIMIT` (χ²/dof per observable) | standardised | 3.0 | 1.25 (`ee_to_tatah` cos θ) | 2.4× |
| `SHAPE_PULL_LIMIT` (worst of ~75 bins) | standardised | 5.0 | 2.54 | 2.0× |

How to read those two classes is [gate thresholds](gate-thresholds.md)[^n36a-unw].
Against VEGAS, four of five rows read slightly negative (−0.36 % to +0.04 %): a
single pass over frozen grids does not inherit VEGAS's iteration combination,
so exact agreement is not expected; the pattern is recorded so a sharpening shows
as a change.

The rules it holds the generator to (detail in
[unweighting](../events/unweighting.md)): the channel is drawn **`∝ w_maxⱼ`**,
not `∝ σⱼ`, because only that leaves kept events `∝ σⱼ` without a compensating
weight, at efficiency `σ / Σⱼ w_maxⱼ`; each `w_maxⱼ` is MadGraph's truncation-ladder
reading of a frozen scan (`MaxRule::Truncated`, the lowest scanned weight leaving
under 1 % of the scan's weight above it), so overweights are expected and are
kept at weight `> 1`, never clipped, and counted as a rate and as a cross-section
share. `ee_to_mumua`'s photon pole is the row with the heaviest overweight tail.

*Blind to*: anything wrong with the integrand (both sides share it —
`amplitude_oracle` and `validate_sigma` cover that) and the labels an event
carries (helicity and colour selection move no weight).

## `generate` end to end

**Fixed beams** (`cli_generate`, default suite). The two weight strategies
([LHEF weight strategy](../events/lhef-weight-strategy.md)) are run on
independent seeds and must agree on shape; `Buffer` (`IDWTUP = −4`) declares the
*sample's* σ, so its mean `XWGTUP` against the integration's σ is a real
comparison, while `StochasticRounding` (`IDWTUP = +3`) carries the integration's
σ by construction and is checked on shape and unit weights only. Neither is the
only way to write the file; stochastic rounding keeps overweights as
`floor(w) + Bernoulli(frac w)` copies, a rule pinned on controlled weights
because a real sample is almost all weight 1, where every rule agrees[^n23-e4].
Stochastic rounding is refused on a card summing several final-state
multiplicities (`refuse_rounding_on_mixed_multiplicity`), whose parts could not
then be normalised to their own cross sections.

**Proton beams** (`cli_generate_proton`, `extended-validation`, needs the banked
`pp_to_llj_fixed` run and the fetched PDF set)[^n24-p4]:

- every event's `NUP`, statuses, mothers, momentum balance and on-shellness, the
  beam partons on their own side of the axis, `SCALUP` and `AQCDUP`;
- **flavour**: every emitted `IDUP` row is one of the subprocesses MadGraph's
  `leshouche.inc` lists, or that subprocess with its beams exchanged — read from
  the generated Fortran, so a flavour MadGraph's 10 000 events happen to miss is
  still admissible;
- **colour**: every event's connectivity is one MadGraph's own events exhibit for
  the same arrangement of gluon, quark, antiquark and leptons;
- a different PDF set from the artifact's is refused by name before the set is
  loaded; a dynamical-scale card runs past the density grid and the coupling
  table; a mixed-multiplicity card is integrated and sampled as a sum.

Admissibility is not frequency: neither oracle says how often a flavour or flow
is drawn. A small `AQCDUP` difference from MadGraph's printed field is expected
(MadGraph's `unwgt.f` divides by a π truncated to eight digits)[^n24-p4c].

**Refusing the wrong artifact.** `generate` re-enumerates the process from a proc
card the caller supplies and compares it, every run-card parameter, and the
[model identity digest](../model/model-identity-digest.md) with what trained the
grids, refusing any difference by name. With both checks deleted, a
different-model run wrote events at σ +1.95 % from the banked value, which is
what the check stands in front of[^n23-model]. The artifact's schema is
versioned; [artifact format versioning](../pipeline/artifact-format-versioning.md)
and `artifact.rs`'s `FORMAT_VERSION` doc comment carry its history.

[^n23-e2]: Note 23 E2. Its "draw the channel ∝ σⱼ" brief was wrong; `unweight.rs`'s module doc carries the derivation.
[^n23-e3]: Note 23 E3.
[^n23-e4]: Note 23 E4. Its `lpp = 1` refusal is gone: proton generation is the P4 gate below.
[^n23-model]: Note 23, post-E4 fix. The `format_version = 3` it introduced has since moved on.
[^n24-p4]: Note 24 P4, gates (a)–(c).
[^n24-p4c]: Note 24 P4, plan corrections item 5.
[^n26-home]: Note 26.
[^n36a-unw]: Note 36a §4.
