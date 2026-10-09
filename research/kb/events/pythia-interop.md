---
type: Validation Gate
title: "Pythia 8 as the consumer: read-back gate and MadGraph's matching settings"
description: "Pythia must consume every event of our samples, with a colour-mutation negative control; and the settings MadGraph 3.7.1 drives main164 with for MLM (setMad off, qCut 1.5·xqcut)."
status: draft
tags: [events, pythia, lhef, mlm, shower]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n25-design, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L396-L403", title: "Note 25 §5.6, Pythia consumption"}
  - {id: n25-landed, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/25-validation-layering-plan.md#L585-L621", title: "Note 25 §10, what each session landed (L5)"}
  - {id: n41-record, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L146-L190", title: "Note 41 §1.4, shower side as first read"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4, Pythia on the matched record, the CDATA finding"}
  - {id: n41-m5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1617-L1822", title: "Note 41 M5, MadGraph's own Pythia settings"}
  - {id: n38-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1032-L1143", title: "Note 38 E1, Pythia reads the decay-chain sample"}
---

# Pythia 8 as the consumer

Two things live here: the **read-back gate**, which asks whether the shower
at the far end of the pipeline can read what we write, and the **settings**
MadGraph 3.7.1 uses when it hands a matched sample to Pythia, which any
comparison through Pythia must reproduce. The matched comparison's results
(acceptances, merged σ, jet rates) are
[validation/mlm-pythia-matched-comparison](../validation/mlm-pythia-matched-comparison.md);
the record fields Pythia's matching reads are
[events/mlm-matched-event-record](mlm-matched-event-record.md).

## The read-back gate

`pixi run -e pythia validate-pythia` (`validation/pythia/consume.py`, in the
pixi `pythia` environment, Pythia 8.312) runs `Pythia::init` on samples
`generate-pythia-samples` writes through the shipped binary from already
validated cards: `pp_to_llj_fixed`'s banked run card, the committed `dy13`
default card, and a `t t̄` decay-chain card (2000 events each, fixed seed and
fixed budget, so the bytes are rerunnable)[^n25-design][^n25-landed][^n38-e1].
It is the only gate that uses colour lines as *input* rather than comparing
them as data.

- **Process level — the gate.** `PartonLevel` and `HadronLevel` off, so a
  refused event is a refused *file* (unreadable record, unmatched colour index,
  a particle Pythia cannot put on shell). Each sample must give exactly as many
  successful `next()` calls as `<event>` blocks, with the reconstructed
  final state's PDG multiset matching the record, and the next call must hit end
  of file, so a truncated read scores short. Metric: n/n consumed.
- **Shower level — informational.** The same file through the full chain; a
  shower stuck in a loop is Pythia's physics, not our format.
- **Negative control.** One final-state parton of the coloured sample gets a
  dangling `ICOLUP(1)` (index 599, above anything the writer emits); Pythia
  must reject exactly that event and accept its neighbours. Without it, n/n
  would be consistent with Pythia ignoring colour.

Each (sample, pass) runs in a child process, so Pythia's messages are captured
whole and a hard abort is recorded rather than killing the driver. The gate is
standalone (its own environment), listed beside the banked layer
([validation/samples-gate](../validation/samples-gate.md) covers the
distributions).

**What it cannot see**, both filed:

- a permuted or corrupted momentum (only the PDG multiset is compared), and any
  colour mutation beyond the single one:
  [validation/pythia-gate-momenta-unchecked](../backlog/validation/pythia-gate-momenta-unchecked.md);
- whether Pythia reads `SCALUP`, `AQCDUP` and the `<init>` cross section as
  meant, and any `IDWTUP = +3` file (only `Buffer` samples are fed):
  [validation/pythia-gate-header-semantics-unchecked](../backlog/validation/pythia-gate-header-semantics-unchecked.md).

It reads event structure, not weights, so a weight-only change (such as the
per-part normalisation) does not need it rerun.

## MadGraph 3.7.1's Pythia settings for MLM

MadGraph 3.7.1's `do_pythia8` runs Pythia's own `main164` (the
MG5aMC_PY8_interface is `--old_interface` only), with a command file from
`setup_Pythia8RunAndCard` applied to `Template/LO/Cards/pythia8_card_default.dat`.
Read from the pinned tree[^n41-m5]:

| setting | value | source |
|---|---|---|
| `Beams:frameType` | 4 | `banner.py:1925`, always written |
| `Check:epTolErr` | 1e-2 | `banner.py:1931`, always written |
| `JetMatching:etaJetMax` | 1000 (Pythia's default is 2.5) | `banner.py:1936`, always written |
| `JetMatching:setMad` | off | `madevent_interface.py:4398` |
| `JetMatching:qCut` | `1.5·xqcut` when the card leaves −1 (30 at `xqcut = 20`) | `:4408-4409` |
| `Beams:setProductionScalesFromLHEF` | on | `:4419` |
| `JetMatching:merge`, `scheme` | on, 1 | `:4456-4457` |
| `JetMatching:nQmatch` | `maxjetflavor` (4) | `:4460` |
| `JetMatching:coneRadius` | 1.0 | `:4462` |
| `JetMatching:nJetMax` | `max_n_matched_jets` (2 on the mixed DY card) | `:4467-4471`, `export_v4.py:5010-5024` |
| `JetMatching:doShowerKt` | off | `pythia8_card_default.dat` |

Everything else is at Pythia's defaults (Monash tune, MPI and hadronisation on,
internal PDF, `doVeto = on`; `doVeto` is switched off only for the old interface
with `use_syst`). Further facts the comparison depends on:

- **`jetAlgorithm` is inert**: `JetMatchingMadgraph::initAfterBeams` forces the
  kT `SlowJet`.
- **`<scales>` enters through the exclusion**: the matching counts a light
  parton only if its production scale is below `1.999·√(E_A E_B)`
  (`sortIncomingProcess` in 8.312's `JetMatching.h`), beside each parton's
  shower start. A parton whose `pt_clust` is the collider energy is excluded.
- The qCut rule is `1.5·xqcut`; "`max(1.5·xqcut, xqcut + 10)`" is not in the
  3.7.1 source.
- The veto and the jet rates are decided in `doVetoPartonLevelEarly`, so
  hadronisation cannot move those observables.
- **Merged σ**, as `main164` normalises at `IDWTUP = −4`, is the sum of the
  accepted events' `XWGTUP` over the file's event count. "σ_LHE × acceptance"
  holds only for an equal-weight file; vibegraph's overweights make the two
  differ, so the comparison weights every event by its `XWGTUP`.

## `setMad = on` and the CDATA card

MadGraph wraps `<MGRunCard>` in `<![CDATA[ … ]]>`, and Pythia 8.312 drops a
CDATA section's content: on MadEvent's own file `Info::header("MGRunCard")` is
4 bytes. With `setMad = on` and no explicit `merge`, Pythia then matches
nothing (10000/10000 accepted). With the CDATA markers removed it reads the full
card (`qCut = xqcut = 20`, `nQmatch 4`, `clFact 1`) and agrees with vibegraph's
file, which writes the card as escaped element text. MadGraph's own driving,
`setMad = off`, is unaffected; reporting the interplay upstream is
[validation/pythia-setmad-drops-cdata-run-card](../backlog/validation/pythia-setmad-drops-cdata-run-card.md).
Note that `setMad = on` sets `qCut = xqcut`, not MadGraph's `1.5·xqcut`[^n41-m4][^n41-m5].

## Readings that are not MadGraph's configuration

An early matched smoke run was taken at Pythia's default `etaJetMax = 2.5`
and with `merge = on` set explicitly; its acceptances, and its reading of
`qCut = 10`, `nQmatch = 5` on MadEvent's file under `setMad = on`, are not what
MadGraph runs and are not comparison values[^n41-m5]. The first reading of
§1.4's shower side (an in-repo `JetMatching.h` copy and the old interface) was
likewise superseded by the reading above[^n41-record].

[^n25-design]: Note 25 §5.6, the gate as designed (n/n events consumed).
[^n25-landed]: Note 25 §10, L5: Pythia 8.312 reads both samples 2000/2000 with a colour-mutation control.
[^n38-e1]: Note 38 E1, the decay-chain sample consumed 2000/2000 at process level and through the shower.
[^n41-record]: Note 41 §1.4, the shower side as first written.
[^n41-m4]: Note 41 M4, Pythia on the matched record and the CDATA finding.
[^n41-m5]: Note 41 M5, MadGraph's own settings and where §1.4 and M4 were short.
