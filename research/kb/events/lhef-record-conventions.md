---
type: Physics Convention
title: LHE header and event-line field conventions
description: "What each <init> and <event> field carries and how vibegraph fills it: SCALUP the larger record μF, AQCDUP untruncated α_s(μR), AQEDUP, PDFSUP, EBMUP, MOTHUP, VTIMUP, SPINUP, pole masses; which fields are per-event oracles."
status: draft
tags: [events, lhef, scalup, aqcdup, conventions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n22-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/22-dynamical-scales-plan.md#L127-L156", title: "Note 22 §1.4, the per-event LHE fields as an oracle"}
  - {id: n23-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L383-L561", title: "Note 23 E3, conventions decided and pinned"}
  - {id: n38-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L1032-L1143", title: "Note 38 E1, LPRUP per @N, PDFSUP as MadEvent's, decay-chain records"}
  - {id: n24-corr, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L2011-L2041", title: "Note 24 P4 plan corrections (AQCDUP/AQEDUP digits)"}
  - {id: mg-unwgt-scalup, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/Template/LO/SubProcesses/unwgt.f#L751-L761", title: "MadGraph 3.7.1 unwgt.f, SCALUP and AQCDUP"}
  - {id: mg-pdfid, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/banner.py#L3839", title: "MadGraph banner.py get_pdf_id"}
---

# LHE header and event-line field conventions

This concept fixes what each Les Houches field holds in a file vibegraph
writes, and where that differs from a naive reading. How the values of
`SCALUP` and `AQCDUP` are computed (the scale prescriptions, the clustering, the
matched two-call split) is [scales-pdf/record-scales](../scales-pdf/record-scales.md);
how the document is serialised is [events/lhef-io-design](lhef-io-design.md);
the weight fields (`IDWTUP`, `XWGTUP`, `XSECUP`, `XERRUP`, `XMAXUP`) are
[events/lhef-weight-strategy](lhef-weight-strategy.md) and
[events/multi-process-normalisation](multi-process-normalisation.md).

## `<init>`

| field | vibegraph writes | note |
|---|---|---|
| `IDBMUP` | proton beams: `±2212` from `lpp`; fixed beams: the incoming legs' PDG codes, which every subprocess must share; a decay: the decaying particle and `0` | a fixed-beam card whose subprocesses differ in initial state is refused, since one `<init>` cannot describe them |
| `EBMUP` | the run card's `ebeam1`, `ebeam2`; a decay: the mass and `0` | |
| `PDFGUP` | `0` on both beams | MadGraph's "named by LHAPDF id alone" spelling |
| `PDFSUP` | `RunCard::pdfsup()` = MadGraph's `get_pdf_id(pdlabel)`: the `lhaid` under `pdlabel = lhapdf`, else the LHAPDF id of the built-in set the label names (`nn23lo1` → 247000), `0` for none | read off `pdlabel` whatever the beams: MadEvent 3.7.1 writes 247000 on fixed-energy runs that read no density at all[^mg-pdfid][^n38-e1] |
| `NPRUP`, `LPRUP` | one process line per `@N` on the card, `LPRUP = N` | each event's `IDPRUP` is its `@N`; on MadGraph's matched cards `@N` equals the `P<n>` directory number, so those runs cannot tell the two readings apart[^n38-e1] |

## `<event>` line

| field | vibegraph writes | note |
|---|---|---|
| `IDPRUP` | the event's `@N` | |
| `SCALUP` | `build::scalup` = `max(mu_f_record[0], mu_f_record[1])`, the larger record factorisation scale | **not μR**; see below |
| `AQEDUP` | the model's `aEW` | α_EW does not run on MadGraph's LO path either: every banked run prints `7.5467710e-3 = 1/132.507`[^n22-oracle] |
| `AQCDUP` | `αs(μR)` from the run's α_s source, **untruncated** | MadGraph divides `g²/4` by `3.1415926` and so writes `αs·(1 + 1.7e-8)`; that is a defect of its field, not a convention, and is not reproduced[^n23-e3] |

### `SCALUP` is the factorisation scale

The accord defines `SCALUP` as the scale the parton densities were evaluated
at, and MadGraph writes exactly that[^mg-unwgt-scalup]:

```fortran
if(q2fact(1).gt.0.and.q2fact(2).gt.0)then
   sscale = sqrt(max(q2fact(1),q2fact(2)))
...
aaqcd = g*g/4d0/3.1415926d0
```

So "SCALUP ≠ μR" is not a MadGraph defect to match; it is the definition.
Reading `SCALUP` as the renormalisation scale is a misreading hazard: the two
coincide on most processes (every closed-form prescription, and 18 of the 20
runs surveyed when this was measured), and differ where the clustering reads
them off different vertices (the `2 → 6` rows)[^n22-oracle]. Because no
ordinary banked file separates them, the convention is pinned by a hand-built
unit test, `scalup_is_the_factorisation_scale_not_the_renormalisation_one`
(`lhef/build.rs`), with `μF = [200, 50]` and `μR = 91.188`. The renormalisation
scale reaches the record through `AQCDUP`[^n23-e3].

Under matching the record scale and the density scale differ;
`EventScales::mu_f_record` (`coupling/scales.rs:83`) carries the one
`SCALUP` reads ([events/mlm-matched-event-record](mlm-matched-event-record.md)).
Without matching they are one value (`EventScales::unmatched`).

### `AQCDUP` on a run whose matrix element has no α_s

The record carries the scale the card asks for and the coupling that scale
implies, whether or not the matrix element moves with α_s, because MadGraph's
`setclscales` runs on the same condition. On a QCD = 0 fixed-energy row with
fixed scales, `AQCDUP` is the parameter card's `aS` (`0.118` against MadGraph's
`0.118` on `ud_to_epemud_qcd0`). `FixedBeamIntegrand::record_scales` writes
`AQCDUP = 0` (and `SCALUP = max(dsqrt_q2fact1, dsqrt_q2fact2)`) only for a
library caller that installed no scale prescription at all; `generate` always
installs one. On toy models whose parameter card injects an unphysical `aS`
(α_s ≈ 1.38), `AQCDUP` faithfully reports it; that is a MadGraph-side defect
recorded in [validation/madgraph-defects](../validation/madgraph-defects.md).

## Particle lines

| field | vibegraph writes |
|---|---|
| `ISTUP` | `-1` incoming, `1` outgoing, `2` an intermediate resonance ([events/resonance-records](resonance-records.md)) |
| `MOTHUP` | `[0, 0]` on incoming legs; `[1, n_in]` on outgoing legs (`[1, 0]` on a decay); daughters of a resonance `[k, k]` with `k` its position |
| `ICOLUP` | the selected flow's colour tags, slot 1 the physical colour and slot 2 the anticolour (an incoming quark's line sits in its colour slot after crossing) |
| `PUP` | momenta; the mass column is the model's **pole mass**, not `√p²`, for external legs; a resonance's mass is its virtuality |
| `VTIMUP` | always `0` (`time_of_flight` off its default is refused) |
| `SPINUP` | the selected helicity of each external leg; `9` on a resonance |

Polarised beams are refused at parse (`polbeam1/2`), so `SPINUP` is always a
helicity drawn off the unpolarised sum or a polarized leg's fixed value.

## Which fields are per-event oracles

Every banked MadGraph event file carries `SCALUP`, `AQEDUP` and `AQCDUP` beside
the momenta. That makes them a per-event, finest-level oracle for the scale
function, α_s(μ) and the μF fed to the PDF — checked event by event before any
σ gate ([validation/scale-replay-gate](../validation/scale-replay-gate.md)).
`<mgrwt>` gives a direct per-event μR (`<rscale>`) and per-beam μF
(`<pdfrwt>`) only on runs made with `use_syst`; elsewhere `AQCDUP` recovers μR
to about 1e-6 relative, since `dαs/αs ≈ −0.1·dQ/Q` at seven printed
digits[^n22-oracle]. The columns the `samples` gate compares (`SPINUP`,
`ICOLUP`, flavour, kinematics, and on some rows `SCALUP`/`AQCDUP` at the
printed digits) are [validation/samples-gate](../validation/samples-gate.md).

Comparisons against a banked field have to respect its precision: the
content of MadGraph's scale fields has seven significant digits (the
delivered file prints them at nine, the last two padding), and its `AQCDUP`
carries the 1.7e-8 truncation, which a comparison adds back rather than
absorbs into a tolerance. `AQCDUP` and `AQEDUP` on `pp_to_llj_fixed` sit about
1e-7 from MadGraph's printed values, for stated reasons (two interpolations of
the same α_s grid; the model's `aEW` against `1/aEWM1` from MadGraph's
parameter card)[^n24-corr].

[^n22-oracle]: Note 22 §1.4: the per-event fields, `<mgrwt>` on six runs, SCALUP is μF, AQEDUP constant.
[^n23-e3]: Note 23 E3: SCALUP and AQCDUP decisions and their pinning tests.
[^n38-e1]: Note 38 E1: one `<init>` line per `@N`, `PDFSUP` as MadEvent's.
[^n24-corr]: Note 24 P4, plan correction 5.
[^mg-unwgt-scalup]: MadGraph 3.7.1 `unwgt.f:751-761`.
[^mg-pdfid]: MadGraph `banner.py:3839`, `get_pdf_id`.
