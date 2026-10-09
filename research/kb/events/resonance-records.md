---
type: Physics Convention
title: Status-2 resonance records
description: "MadEvent's addmothers/cut_bw rules for status-2 lines (flag, same-flavour daughters, order, mothers, colour, mass), written for decay chains and for matched events."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [events, lhef, resonances, decay-chains, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1032-L1143", title: "Note 38 E1, MadEvent's record rules and the decay-chain gates"}
  - {id: n41-m4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1415-L1616", title: "Note 41 M4, resonances under matching"}
  - {id: mg-addmothers, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/iolibs/template_files/addmothers.f#L240-L350", title: "MadGraph 3.7.1 addmothers.f"}
  - {id: mg-cutbw, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/myamp.f#L2", title: "MadGraph myamp.f cut_bw"}
  - {id: n07-io, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L206-L227", title: "Note 07, MadGraph LHE output defects"}
---

# Status-2 resonance records

An LHE event may list intermediate particles with `ISTUP = 2`: resonances
whose mass a shower should preserve. MadEvent writes one for a propagator of
the event's configuration when `cut_bw` (`myamp.f`) leaves it flagged `OnBW`[^mg-cutbw],
and `addmothers` (`addmothers.f:253`, `ickkw = 0`) fills in the record[^mg-addmothers].
The rules below are vibegraph's reading, implemented in
`vibegraph-lib/src/lhef/resonance.rs` (`SubprocessResonances`) and written by
`SubprocessRecord::event_with_intermediates` (`lhef/build.rs`)[^n38-e1].

## Which lines are flagged

The candidate lines are the timelike propagators of the event's
configuration — the one its colour flow was drawn in
([events/colour-and-helicity-selection](colour-and-helicity-selection.md)),
resolved once per configuration from the diagram MadGraph writes it from
(`AmplitudeEvaluator::config_diagram`). A line is flagged when:

- it has positive width (`prwidth > 0`; a zero-width line is never a
  candidate) and its invariant mass is inside its window,
  `|√p² − M| < bwcutoff·Γ`, with `Γ` floored at `M·small_width_treatment`
  (`prwidth_tmp`), **and** it is narrow, `Γ/M < 0.1` — unless a decay chain
  forces it on shell (`gForceBW = 1`), in which case the width ratio is not
  tested. Forced lines are always inside their windows, since `cut_bw` rejects
  the point otherwise ([phase-space/resonance-and-pole-maps](../phase-space/resonance-and-pole-maps.md));
- the flag is withdrawn from a line with a daughter of its own flavour: always
  when that daughter is an external leg; otherwise from whichever of the two a
  decay chain does not force; when neither is forced, from the one further from
  its pole (`myamp.f:146-176`).

So free lines are flagged too: the `W` inside `t > b e+ ve` carries a record
whenever it lands in its own window (on 50 000 MadEvent
`p p > t t~, t > b e+ ve, t~ > b~ mu- vm~` events, 49 101 carry both `W`s, 897
one, 2 none), and the records of one subprocess differ event to event[^n38-e1].

No record is written when the drawn colour flow is outside the configuration's
leading-colour set (`is_LC`, `addmothers.f:129-131, 195`); the CLI counts those
events and reports them.

## The record layout

| item | rule |
|---|---|
| order | incoming legs, then resonances, then outgoing legs in process order; parents precede children |
| sibling order | MadEvent's follows `configs.inc` (`i = -ns … -1`), so it depends on the configuration (`g g` s-channel config writes `t W+ t~ W-`, t-channel configs `t~ W- t W+`); vibegraph orders siblings by lowest outgoing leg, since positions carry no physics |
| mothers | top-level resonance `1 2` (`1 0` on a decay, `unwgt.f:741`); a nested resonance and the daughters of a resonance `k k`, with `k` the parent's position |
| momentum | the daughters' sum |
| mass | the daughters' virtuality, not the pole mass |
| colour | the daughters' lines with colour/anticolour pairs contracted (`elim_indices`): a `t` gets `501 0` with the `b`'s 501, a `W` `0 0` |
| `SPINUP` | `9` |
| identical decays | one pairing (`NSYM = 1`): `e+ e- > z z, z > e+ e-` lists the `Z`s over legs `(3,4)` and `(5,6)`; vibegraph picks the pairing by restricting the configuration draw to configurations whose forced lines are in their windows (`mask_unadmitted`) |

On MadEvent's side none of 199 099 records violates the momentum or mass rule.
A record whose daughters leave colour lines that cannot fit the resonance's
representation, or two resonances over the same legs, is not written with a
wrong value: the event is written without intermediates and counted as
refused[^n38-e1].

At proton beams a flavour-group member takes the representative's line or its
antiparticle by the charge of the legs below it (`member_line_pdg`). UFO
antiparticle entries carry `color = -1/-8` for singlets and octets (the model's
`make_anti` negates every colour, where UFO's `anti()` does not), which is
handled locally ([hygiene/make-anti-negates-singlet-octet-colour](../backlog/hygiene/make-anti-negates-singlet-octet-colour.md)).

## When vibegraph writes them

| card | records |
|---|---|
| decay chain (`p p > t t~, t > b e+ ve, …`) | forced and free on-window lines, as above |
| plain process (`e+ e- > mu+ mu-` at the pole, `p p > mu+ mu- / a`) | **none**; MadEvent writes the on-window free `Z` (or `W`) here too |
| matched (`ickkw = 1`) | the clustered configuration's lines whose leg sets the integration channel put on its Breit–Wigner ([events/mlm-matched-event-record](mlm-matched-event-record.md)) |

The plain-process gap is open work: the backlog item
[feature/plain-process-onwindow-resonance-records](../backlog/feature/plain-process-onwindow-resonance-records.md)
records the decision to match MadEvent, accepting that this changes every
existing LHE file for such processes, because a shower keeps a resonance's
mass when it reshuffles momenta only if the resonance is in the record.
Under matching the test is `checkbw` over the integration channel's
propagators, keyed by leg set and applied to the clustered configuration's lines
— not "the lines the clustering found on their Breit–Wigner"[^n41-m4].

## Evidence

`cli_decay_chain_events` (`decay_chain_events_reference.json`; two generation
seeds × 20k events against five MadEvent 3.7.1 runs of 10k, χ² homogeneity with
p-floor 1e-4): on `pp_ttx_lep_dyn` the record trees p 0.099, `SPINUP` 0.93,
`m(t)` 0.17, `m(W+)` 0.17, `m(W−)` 0.021, `m(t̄)` 1.8e-3. An eight-seed probe read
`m(t̄)` p 0.02–0.89 and `m(W−)` 0.01–0.40, and MadEvent's own `m(W−)` against its
`m(W+)` reads p 0.04, so the low values follow the fixed reference sample.
`ttx_nested` (a forced `W+` under the `t`) and `zz_ee` pass likewise, and every
record on both sides satisfies the colour and momentum rules[^n38-e1]. Pythia
reads the decay-chain sample at process level and through the shower
([events/pythia-interop](pythia-interop.md)). How the `samples` columns are
built is [validation/samples-gate](../validation/samples-gate.md); per-field
conventions shared with external legs are
[events/lhef-record-conventions](lhef-record-conventions.md); decay-chain
enumeration is [process/decay-chains](../process/decay-chains.md).

MadGraph's own history of resonance-record defects (3.6.6's "intermediate
particles wrongly written to the event file", 1.5.9's wrong propagator codes
for symmetric diagrams) is why each written status-2 line is checked against an
actual propagator of the configuration rather than trusted
([validation/madgraph-defects](../validation/madgraph-defects.md))[^n07-io].

[^n38-e1]: Note 38 E1: MadEvent's record rules read from `addmothers.f`, `unwgt.f:737`, `myamp.f:76` and five MadEvent runs; the gates.
[^n41-m4]: Note 41 M4, where §1.4's plan text was short of the source.
[^mg-addmothers]: MadGraph 3.7.1 `addmothers.f:253-268`, the status assignment.
[^mg-cutbw]: MadGraph `myamp.f`, `cut_bw` (declared at `:2`; the positive-width gate and window test at `:123-139`, the same-flavour withdrawal at `:146-176`).
[^n07-io]: Note 07, I/O and output-format defects.
