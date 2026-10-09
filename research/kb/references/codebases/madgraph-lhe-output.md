---
type: Codebase Survey
title: MadGraph's LHE output path and format strings
description: "How MadGraph 3.7.1 writes unweighted_events.lhe: Fortran first, then a Python pass that reformats only what it parses, so files arrive in two dialects; format strings, SCALUP, AQCDUP and status 2."
resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/lhe_parser.py"
status: draft
tags: [madgraph, lhef, event-output, formats, external-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n23-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L383-L561", title: "Note 23 E3 outcome (LHEF writer, MadGraph as format oracle)"}
  - {id: mg-lhe-parser, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/lhe_parser.py#L133-L160", title: "lhe_parser.py, Particle.parse and the particle-line format"}
  - {id: mg-lhe-event, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/lhe_parser.py#L2600-L2612", title: "lhe_parser.py, the event-info line format"}
  - {id: mg-banner-init, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/various/banner.py#L1096-L1108", title: "banner.py, the <init> line formats"}
  - {id: mg-rw-events, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/Source/rw_events.f#L183", title: "rw_events.f, the Fortran event-info format"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/Template/LO/SubProcesses/unwgt.f#L752-L761", title: "unwgt.f, SCALUP and the coupling fields"}
  - {id: mg-addmothers, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/template_files/addmothers.f#L253", title: "addmothers.f, status-2 resonance records"}
  - {id: lhef-mod, resource: "vibegraph-lib/src/lhef/mod.rs#L1-L80", title: "lhef module docs: formats, two dialects, lossy records, SCALUP"}
  - {id: validate-lhef, resource: "vibegraph-lib/tests/validate_lhef.rs#L157-L263", title: "banked_files_round_trip_byte_for_byte and the mutation test"}
measured:
  - {commit: 04d5da4, command: "validate_lhef (banked round trip, 20 runs)"}
---

This is how MadGraph 3.7.1 (`b7687064`) produces the `unweighted_events.lhe` a
shower reads, and what that means for anything that reads or imitates it. Our
own record conventions are [LHEF record conventions](../../events/lhef-record-conventions.md);
the reader and writer design is [LHEF I/O](../../events/lhef-io-design.md);
the gates are [event-output gates](../../validation/event-output-gates.md).
The rest of MadGraph is [the MadGraph survey](madgraph5-amcnlo.md).

## Two writers, two dialects

The file is written twice. `Source/rw_events.f` emits it from Fortran, with the
event-info line `'(i2,i5,e16.7e3,3e15.7)'` (`rw_events.f:183`, `:281`). The
Python post-processing then reads it back and writes it out again through
`madgraph/various/lhe_parser.py` and `banner.py`, but it converts only the
fields it had to parse. So a delivered file arrives in one of two spellings[^lhef-mod]:

- **converted**: every line re-formatted by Python, when a pass (systematics,
  for instance) forced a full parse;
- **pass-through**: only the rescaled weight re-formatted, every other field
  kept as the Fortran string (`0.25000000000E+03`, lifetime and helicity `0.`
  and `1.`, an info line with no column padding).

Nothing in the run card says which one a run will produce. A reader that
re-emits must therefore keep each line's source text, and a writer that claims
MadGraph's layout must be checked against **converted** files, not
pass-through ones.

The Python formats[^mg-lhe-parser][^mg-lhe-event][^mg-banner-init]:

```text
init beam line:    "%(idbmup1)i %(idbmup2)i %(ebmup1)e %(ebmup2)e %(pdfgup1)i %(pdfgup2)i %(pdfsup1)i %(pdfsup2)i %(lha_stra)i %(nprup)i"   banner.py:1106
init process line: "%(cross)e %(error)e %(wgt)e %(id)i"                                                                          banner.py:1096
event line:        "%2d %6d %+13.7e %14.8e %14.8e %14.8e"                                                                      lhe_parser.py:2606
particle line:     " %8d %2d %4d %4d %4d %4d %+13.10e %+13.10e %+13.10e %14.10e %14.10e %10.4e %10.4e"                      lhe_parser.py:149
```

Against the Fortran specifier these differ on every line: two exponent digits
rather than three, nine significant digits in the scale fields rather than
seven. The values reaching the scale and coupling fields were computed at
seven significant digits in Fortran, so a converted file's scale fields end in
two padding zeros. No value is narrower than its field, so in practice the
layout is one space between columns, with the widths mattering only for the
small integers.

## Precision of a file

`<init>` cross sections carry seven significant digits, `XWGTUP` eight, momenta
eleven. Re-serialising a file reproduces it byte for byte, but recovers the run
only to those precisions.

## Field values MadGraph writes

- **`SCALUP`** is `sqrt(max(q2fact(1), q2fact(2)))` (`unwgt.f:752`), the larger
  factorisation scale, which is what the Les Houches accord defines. It is not
  `μR`. The two coincide whenever the clustering puts both scales on the same
  vertex, so reading `SCALUP` as `μR` is a hazard, not a MadGraph defect. Ours
  is the same quantity (`lhef/build.rs::scalup`); see
  [record scales](../../scales-pdf/record-scales.md).
- **`AQCDUP`** is `g*g/4d0/3.1415926d0` (`unwgt.f:760`), and `AQEDUP` the same
  with `gal(1)` (`:761`). The truncated π biases both by `+1.7e-8` relative, in
  one direction. That is a defect of the field
  ([MadGraph defects](../../validation/madgraph-defects.md)); our writer does
  not reproduce it.
- **Second beam**: its transverse momentum components are written `-0.0`.
- **`<generator>`**: MadGraph's tag is single-quoted, and events can carry
  `<mgrwt>` and `<rwgt>` blocks. Banners are not reliably well-formed XML, which
  is why our reader runs with `check_end_names = false`.
- **Status 2**: MadEvent writes an `ISTUP = 2` record for a propagator of the
  event's configuration when `cut_bw` (`myamp.f:76`) leaves it flagged `OnBW`;
  `addmothers.f:253` then gives it its daughters' summed momentum, its
  virtuality as the mass, and the daughters' mother pointers. MadGraph 3.6.6
  fixed a serious bug that wrote wrong intermediate particles to the event file.
  vibegraph's reconstruction of these records is
  [resonance records](../../events/resonance-records.md).
- **Colour labels**: only the connectivity is physical, and MadGraph relabels
  the same connectivity differently across subprocesses. The rule is stated
  once in [LHEF record conventions](../../events/lhef-record-conventions.md).
- **Parsing**: `lhe_parser.Particle.parse` sets every field through
  `float(value)` and casts only `pid` back to `int`, so `color1` and `color2` are
  floats in Python; the writer prints them with `%4d`.

## MadGraph as a format oracle

Our events are not MadGraph's events (no shared random numbers), so no
per-event content comparison is meaningful; distribution-level comparison is
[the samples gate](../../validation/samples-gate.md). What MadGraph can check is
the format: `banked_files_round_trip_byte_for_byte`
(`tests/validate_lhef.rs:157`) parses each banked file into our records, writes
it back, and requires identical bytes. It counts the converted runs separately
and requires at least one, a hadron-collider `<init>` and coloured incoming
legs, because a corpus of pass-through runs would only show that reader and
writer are inverse[^validate-lhef].

`the_round_trip_is_sensitive_to_every_convention_sensitive_field` (`:269`)
mutates a parsed file eight ways (`MOTHUP` dropped, `MOTHUP` order swapped,
`ISTUP` sign flipped on incoming legs, `ICOLUP` slots exchanged, incoming
momenta crossed to outgoing, the momentum tuple rotated, the mass replaced by
the momentum's invariant, `SPINUP` zeroed) and requires each to break the round
trip. When first run (commit `04d5da4`) the round trip covered 20 banked runs,
198 747 events and 1 020 299 particle lines, byte-identical, and all eight
mutations fired[^n23-e3].

What these tests cannot detect: which event was generated (they re-emit
MadGraph's values), whether our sample is distributed like MadGraph's, colour
label integers, which helicity an event should carry, and `SCALUP` as `μF`
against `μR` on real kinematics (pinned only by a hand-built unit test and by
the scale replay on MadGraph's 2 → 6 runs).

[^lhef-mod]: `vibegraph-lib/src/lhef/mod.rs`, module docs "There is more than one dialect".
[^mg-lhe-parser]: `madgraph/various/lhe_parser.py:149` at `b7687064`.
[^mg-lhe-event]: `madgraph/various/lhe_parser.py:2606` at `b7687064`.
[^mg-banner-init]: `madgraph/various/banner.py:1096` and `:1106` at `b7687064`.
[^validate-lhef]: `vibegraph-lib/tests/validate_lhef.rs`, module docs and lines 157–263.
[^n23-e3]: Note 23, the LHEF writer outcome (2026-07-28).
