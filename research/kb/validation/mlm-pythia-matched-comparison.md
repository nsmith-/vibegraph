---
type: Validation Gate
title: Matched MLM comparison through Pythia
description: "Both sides' matched files through one Pythia main164 configuration: acceptance per @N, merged sigma, jet-rate shapes, MadEvent nulls and the <scales> negative control."
status: draft
tags: [mlm, pythia, matching, shower, info-cell]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {command: "pixi run -e pythia validate-mlm-pythia (21 MadEvent against 20 vibegraph files, Pythia 8.312, seeds 20261201-10)"}
sources:
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4, the record and the CDATA finding"}
  - {id: n41-m5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1617-L1822", title: "Note 41 M5, matched end to end"}
  - {id: n41-fa, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1823-L2005", title: "Note 41 F-A, the weight tail"}
  - {id: n41-c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2558-L2701", title: "Note 41 C, the acceptances at four times the statistics"}
  - {id: n41-p12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2299-L2557", title: "Note 41 P12, per-part normalisation"}
---
# Matched MLM comparison through Pythia

The shower applies the MLM veto, so the matched sample's real test is what a
shower does with it. MadEvent's and vibegraph's matched `pp_to_ll_0j2j_mlm` files
go through one Pythia configuration, MadGraph's own, and the comparison reads the
matching acceptance per `@N`, the merged σ and the differential jet rates. It is
the only check that sees the shower's reading of `<scales>`. It is `info`. What
vibegraph writes for the shower is
[events/mlm-matched-event-record](../events/mlm-matched-event-record.md); what
it does and leaves to the shower is [scales-pdf/mlm-matching](../scales-pdf/mlm-matching.md).

## The driver

- `validation/pythia/mlm_match.py` + `mlm_match.cc`, pixi task
  `validate-mlm-pythia` in the `pythia` environment (Pythia 8.312). The C++
  driver reads a Pythia command file as `main164` does and runs `main164`'s hook,
  `JetMatchingMadgraph`, subclassed only to record each event's IDPRUP, the
  process-level veto, the MLM veto, whether `next()` returned it, and `getDJR()`.
- **Side A** (MadEvent) defaults to the 21 banked files: `run_01` and the twenty
  fresh directories `run_s20261101`…`run_s20261120`, read from the σ reference's
  `events` fields; it stops naming the missing file on a bundle without them.
  **Side B** comes from `generate_mlm_samples.sh` (task
  `generate-mlm-pythia-samples`): twenty vibegraph samples, seeds 20260928–47,
  each 10000 events from an integration at `--fixed-budget --allocate neyman
  --neval 200000 --niter 8` with `generate --seed` equal to the integration seed.
- Ten Pythia seeds (20261201–10) on every file. A run whose command file is
  unchanged, with each input's and the driver's sha256 in it, is read back rather
  than re-run.

## MadGraph's own Pythia settings

MadGraph 3.7.1 runs Pythia's `main164` unless `--old_interface` is given
(`madevent_interface.py:4600-4655`); the command file is
`setup_Pythia8RunAndCard` applied to `Template/LO/Cards/pythia8_card_default.dat`:

| setting | value | source |
|---|---|---|
| `Beams:frameType` | 4 | `banner.py:1925`, always written |
| `Check:epTolErr` | 1e-2 | `banner.py:1931` |
| `JetMatching:etaJetMax` | 1000 | `banner.py:1936`, always written (Pythia's default is 2.5) |
| `JetMatching:setMad` | off | `madevent_interface.py:4398` |
| `JetMatching:qCut` | 1.5·xqcut = 30 | `:4408-4409`, when the card leaves −1 (the default card does) |
| `Beams:setProductionScalesFromLHEF` | on | `:4419` |
| `JetMatching:merge`, `scheme` | on, 1 | `:4456-4457` |
| `JetMatching:nQmatch` | `maxjetflavor` = 4 | `:4460` |
| `JetMatching:coneRadius` | 1.0 | `:4462` |
| `JetMatching:nJetMax` | `max_n_matched_jets` = 2 | `:4467-4471`, `export_v4.py:5010-5024` |
| `JetMatching:doShowerKt` | off | `pythia8_card_default.dat` |

- `jetAlgorithm` is inert: `JetMatchingMadgraph::initAfterBeams` forces the kT
  `SlowJet`.
- The matching counts a light parton only if its production scale is below
  1.999·√(E_A E_B) (`sortIncomingProcess`, 8.312's `JetMatching.h`); that is the
  path by which `<scales>` enters, beside each parton's shower start.
- The qCut rule in 3.7.1 is 1.5·xqcut; "max(1.5·xqcut, xqcut + 10)" is not in the
  source. Everything else is Pythia's defaults (Monash, MPI and hadronisation on,
  `doVeto = on`); the veto and the DJRs are decided in `doVetoPartonLevelEarly`,
  so hadronisation cannot change these observables[^n41-m5].

## Statistics

The Les Houches event is the unit: each event's outcome is averaged over the
Pythia seeds, errors are the spread over events (delta method for normalised
shapes), and 200 bootstrap resamplings re-derive the bin errors. Every event
counts with its `XWGTUP`, as Pythia weights it at `IDWTUP = −4`; the merged σ
follows `main164`, the accepted events' summed `XWGTUP` over the file's event
count. "Merged σ = σ_LHE × acceptance" holds only for an equal-weight file.
Per-file values and their χ²/dof about each side's mean test that events are
independent. Jet-rate bins are 0.1 wide in log10(d/GeV) on [0, 3] with under- and
overflow, merged from the left until each holds 100 accepted events on both
sides. At ten seeds the shower's share of an event's outcome variance is small,
so events, not seeds, set the error.

## Results

21 MadEvent files against 20 vibegraph files, ten Pythia seeds, MadGraph's
settings[^n41-c]:

| | MadEvent | vibegraph | B − A | pull |
|---|---|---|---|---|
| acceptance `@0` | 0.8191 ± 0.0003 | 0.8187 ± 0.0004 | −0.05% | −0.80 |
| acceptance `@1` | 0.3603 ± 0.0011 | 0.3587 ± 0.0012 | −0.44% | −0.95 |
| acceptance `@2` | 0.3459 ± 0.0018 | 0.3432 ± 0.0021 | −0.78% | −0.98 |
| acceptance, all | 0.6447 ± 0.0006 | 0.6441 ± 0.0007 | −0.09% | −0.66 |
| merged σ (pb) | 685.77 ± 0.68 | 685.98 ± 0.80 | +0.03% | +0.19 |
| jet rates χ² d01 / d12 / d23 | | | | 24.5/26, 14.4/23, 20.7/20 |

Both sides' files scatter as their errors say (acceptance χ²/dof 0.63–1.00 on
MadEvent's, 0.75–0.92 on vibegraph's; merged σ 1.04 and 0.53). The comparison
resolves ±0.3% (`@1`) and ±0.5% (`@2`) per side at 1σ. No bin of any localisation
variable (initial state, lowest `pt_clust`, parton pT and |η|, `SCALUP`,
`pt_clust/pT`) reaches 3σ; near the matching scale the sides agree bin by bin
within 1.5σ.

**Nulls** (cached runs re-keyed):

| split | `@0` | `@1` | `@2` | all | merged σ | χ² d01 / d12 / d23 |
|---|---|---|---|---|---|---|
| MadEvent files 1–11 against 12–21 | +0.09 | −0.47 | −0.07 | −0.96 | −1.26 | 17.1/25, 35.6/22, 18.3/19 |
| vibegraph seeds 28–37 against 38–47 | +0.67 | +0.23 | +1.36 | +0.69 | +0.66 | 34.3/25, 13.6/22, 21.7/19 |

A MadEvent-against-MadEvent split reads d12 at p = 0.034, so a d12 tension of that
size is inside the comparison's own scatter. Five files a side read the `@1` and
`@2` acceptances about 2σ low; that came from MadEvent's first five files reading
1.45σ high on `@1`, and does not survive four times the statistics.

**Sample-set caveat.** The vibegraph samples must be written with each `@N`
normalised to its integration (`lhef/emit.rs`, `Buffer`); without it the file's
declared σ is the sample's own estimate, and the merged σ scatters file to file
at χ²/dof 4.67[^n41-p12]. Overweight events (`w/w_max` up to 119, from the
adapted grids' heavy tail,
[phase-space/vegas-grid-weight-tail](../phase-space/vegas-grid-weight-tail.md))
are kept at their own weight, so counting events instead of weights moves the
acceptance.

## Controls

Measured in M5, on five files a side (MadEvent's `run_01` and four fresh
directories against five vibegraph samples):

- **`<scales>` removed** (every `<scales …>` line deleted from the vibegraph
  files, so each parton's scale is `SCALUP`): `@1` goes to +25σ against MadEvent
  and `@2` to +8.1σ, the merged σ moves +6.9%, and the jet-rate χ² goes to 937/24,
  494/21 and 348/17, with a step in d01 at log10 qCut. The comparison sees
  `<scales>` at more than ten times its resolution[^n41-m5].
- **qCut 45 on both sides**: MadEvent's acceptances move to 0.907 / 0.254 /
  0.210 and vibegraph follows them within 1.3σ.

## `setMad = on` and the CDATA card

MadGraph wraps `<MGRunCard>` in `<![CDATA[ … ]]>`, and Pythia 8.312 drops a CDATA
section's content: on MadEvent's own file `Info::header("MGRunCard")` is 4 bytes,
`setMad = on` warns "Madgraph merging parameters not found", leaves `merge` at
its default (off) and matches nothing (10000/10000 accepted). This crate writes
the card as escaped element text, which Pythia reads. With the CDATA markers
deleted from MadEvent's files, both headers drive Pythia identically (qCut 20 =
xqcut, not MadGraph's 1.5·xqcut; nQmatch 4) and the acceptances agree within
0.9σ. The driver stops on a run whose settings Pythia did not read back
([pythia-setmad-drops-cdata-run-card](../backlog/validation/pythia-setmad-drops-cdata-run-card.md),
[events/pythia-interop](../events/pythia-interop.md))[^n41-m4].

## Status and open items

The row is `info`, with no collator row and no decided tolerance
([mlm-pythia-comparison-no-tolerance](../backlog/validation/mlm-pythia-comparison-no-tolerance.md)).
The separate Pythia consumption gate (`validate-pythia`, `consume.py`) reads
event structure only; its blind spots are
[pythia-gate-header-semantics-unchecked](../backlog/validation/pythia-gate-header-semantics-unchecked.md)
and [pythia-gate-momenta-unchecked](../backlog/validation/pythia-gate-momenta-unchecked.md).
The σ itself is gated by [mlm-sigma-gate](mlm-sigma-gate.md).

[^n41-m5]: Note 41 M5: MadGraph's settings, the driver, the negative controls, and where M4 was short (its smoke reading ran at Pythia's default `etaJetMax = 2.5` and its qCut 10 came from setting `merge = on` explicitly).
[^n41-c]: Note 41 C, results, spreads, nulls and localisation.
[^n41-p12]: Note 41 P12 policy 1 and its Pythia table; F-A's localisation of the overweights.
[^n41-m4]: Note 41 M4, "The CDATA finding".
