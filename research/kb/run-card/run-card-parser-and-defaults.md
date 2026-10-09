---
type: Design
title: MadGraph run_card parsing, defaults and beam modes
description: "run_card.dat syntax, MadGraph's LO defaults transcribed and checked against a banner.py dump, unknown names as hard errors, the edits applied on resolution, and the two accepted beam modes."
status: draft
tags: [run-card, parser, defaults, beams, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-inventory, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L163-L196", title: "Note 18 §1.5, the run-card inventory"}
  - {id: n18-design, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L312-L341", title: "Note 18 §2.6, run card and cuts abstraction"}
  - {id: n18-regime, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L352-L378", title: "Note 18 §3, validation regime (defaults oracle)"}
  - {id: n18-records, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5 decision records (H6 run-card-cuts)"}
  - {id: mg-banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/banner.py#L4208", title: "MadGraph banner.py RunCardLO.default_setup"}
---

# MadGraph run_card parsing, defaults and beam modes

`vibegraph-lib/src/runcard.rs` parses MadGraph's `run_card.dat` into a typed
`RunCard`. The design goal is that an **empty** card reproduces MadGraph's
out-of-the-box LO behaviour, so a MadGraph reference run and vibegraph can read
the literal same file and cut, beam and scale settings cannot drift between
the two sides by construction[^n18-design][^n18-regime]. Which parameters are
read, ignored or refused is [run-card/field-classification](field-classification.md);
the cut semantics are [run-card/madgraph-cut-conventions](madgraph-cut-conventions.md);
the matching block's edits are [run-card/matching-parameters](matching-parameters.md).

## Syntax

One parameter per line, `<value> = <name> ! comment`:

- the comment (`!…`) is stripped first; blank lines, `#` lines and `$`
  structural lines are skipped, as is a line whose "name" contains a space
  (template placeholders);
- names match **case-insensitively** and resolve to the canonical name of the
  defaults table (MadGraph writes `sde_strategy` in the card, `SDE_strategy` in
  `banner.py`);
- an **unknown name is a hard error** (`RunCardError::UnknownParam`), as typo
  protection; a malformed value is `BadValue`;
- values parse by the parameter's kind: floats accept Fortran `d`/`D`
  exponents; integers accept a trailing `.0`; booleans accept `T`, `F`,
  `.true.`, `false`, …; strings may be quoted; list- and dict-valued
  parameters are kept as an opaque payload, with `{}` and `[]` normalised to the
  empty default so MadGraph's "unset" spelling does not read as an override.

## Defaults: transcribed, and checked against MadGraph's own

`PARAM_DEFAULTS` transcribes `RunCardLO.default_setup` (`banner.py`)[^mg-banner]
and is both the source of default values and the set of recognised names (209
entries). Every recognised parameter is retained by name in `RunCard::values`,
because the cut filter builds names at run time (`pt{c}`, `dr{tag}`) and
nothing may be silently dropped. Fourteen are also typed fields (`nevents`,
`iseed`, `lpp1/2`, `ebeam1/2`, `pdlabel`, `lhaid`, `fixed_ren_scale`,
`fixed_fac_scale`, `scale`, `dsqrt_q2fact1/2`, `maxjetflavor`).

The oracle is committed reference data: `validation/madgraph/runcard_defaults.json`,
a dump of `banner.py`'s defaults (pixi task `dump-runcard-defaults` in the
`madgraph` environment; 218 parameters, the extra 9 being `system=True`
internals such as `pdg_cut` and `ptmin4pdg` that are never written to a user
card). `defaults_match_banner_py_dump` compares every scalar default; take
counts from the files, not from this page[^n18-records].

Its blind spots:

- it checks transcribed **values**, not what a cut does to momenta — that is
  the per-cut boundary tests and the σ gates[^n18-regime];
- it does not compare **opaque** payloads. Three stored empty defaults differ
  from MadGraph's (`mxx_only_part_antipart`, `pdgs_for_merging_cut`,
  `systematics_arguments`), pinned by `opaque_defaults_known_to_differ_from_banner_py`
  so a MadGraph bump that moves the set fails; they are harmless only because
  each is classified benign for a reason independent of its default
  ([hygiene/runcard-opaque-defaults-unverified](../backlog/hygiene/runcard-opaque-defaults-unverified.md)).
  `me_frame` now stores MadGraph's `[1, 2]`.

## Resolution: what happens after parsing

`RunCard::from_values`, in order:

1. `matching::resolve` applies `banner.py`'s, `setrun.f`'s and `setcuts.f`'s
   edits to the matching block (refusals for `ickkw ∉ {0,1}`, matched top jets,
   `ptj < xqcut`; `alpsfact = 1` under `use_syst`; the `xqcut` cut rewrites) —
   [run-card/matching-parameters](matching-parameters.md);
2. at proton beams, the per-beam PDF labels resolve as `banner.py`'s
   `PDLabelBlock` does: two equal `pdlabel1/2` set `pdlabel`, two different
   ones are refused (`AsymmetricBeamPdf`), and a card leaving both at their
   default spells the PDF through `pdlabel` and is left alone;
3. the beam check;
4. `classes::refuse_ignored_physics` refuses any `IgnoredPhysics` field off its
   default ([run-card/field-classification](field-classification.md));
5. `me_frame` must be a non-empty list of leg numbers (`frame_id` is
   recomputed from it as `Σ 2^n`, as MadGraph does; the card's own `frame_id`
   is not read).

Every consumer, including the artifact that banks the card, reads the card
after these edits. `RunCard::banner_values` keeps the few values MadGraph's own
record of the card differs on (before the Fortran's rewrites), for the
`<MGRunCard>` a matched file carries; they are `#[serde(skip)]`, so the
artifact is unchanged.

## Beam modes

Exactly two `(lpp1, lpp2)` pairs are accepted (`RunCardError::UnsupportedLpp`
otherwise)[^n18-inventory]:

| `lpp1, lpp2` | `BeamMode` | meaning |
|---|---|---|
| `1, 1` | `Proton` | PDF convolution, `√ŝ = √(x₁x₂s)`; the PDF set is fetched, not embedded ([tooling/pdf-set-distribution](../tooling/pdf-set-distribution.md)) |
| `0, 0` | `FixedEnergy` | the incoming particles are the beams, `√ŝ` from `ebeam1 + ebeam2`, no PDF read |

EVA lepton PDFs (`|lpp| = 3, 4`), ions and asymmetric pairs are refused. Where
the CLI finds the PDF set and UFO models is
[tooling/asset-resolution](../tooling/asset-resolution.md), not the run card.

Two derived cards exist for decays. `RunCard::decay_default` is MadGraph's
default card for a `1 → n` process (`create_default_for_process` with
`remove_all_cut`: every cut removed, `SDE_strategy = 1`, systematics off), and
`RunCard::for_decay(mass)` is any card as MadEvent runs it for one incoming
particle (both beams fixed-energy at `M/2`, μR fixed at `M` unless the card
fixes it, both μF fixed at the card's values). The out-of-the-box
`create_default_for_process` edit that auto-enables matching
(`ickkw = 1`, `xqcut = 30`) on mixed-multiplicity jet cards is MadGraph's
card-writing step; vibegraph reads only the card it is given.

`RunCard::pdfsup` gives the `PDFSUP` MadEvent writes for the card
(`get_pdf_id(pdlabel)`), read off `pdlabel` whatever the beams
([events/lhef-record-conventions](../events/lhef-record-conventions.md)).

## The lhaid trap

The pinned NNPDF23_lo_as_0130_qed set is **lhaid 247000**. 244600
(NNPDF23_nlo_as_0118_qed) and 230000 (NNPDF23_nlo_as_0119, the value in the
parser fixture `vibegraph-lib/tests/data/run_card_parser_fixture.dat`) are
different, NLO, non-QED sets — a
parser fixture is not a statement about which set a run uses[^n18-records]
([scales-pdf/pinned-pdf-set](../scales-pdf/pinned-pdf-set.md)).

Release scope — what is supported at all — is
[pipeline/release-scope](../pipeline/release-scope.md).

[^n18-inventory]: Note 18 §1.5, the inventory from `RunCardLO.default_setup`.
[^n18-design]: Note 18 §2.6, parse plus typed defaults; one shared card.
[^n18-regime]: Note 18 §3, the defaults oracle and its blind spot.
[^n18-records]: Note 18 §5, H6 (parser, defaults dump, 218 dumped / 209 scalars) and H7 (lhaid).
[^mg-banner]: MadGraph `banner.py:4208`, `RunCardLO.default_setup`.
