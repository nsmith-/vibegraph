---
type: Design
title: "LHEF reader and writer: records, emit, parse, build, and the source-preserving round trip"
description: "The lhef module's layers: quick-xml owns the document, fixed-format records are written by hand, and parsed blocks keep their source text so both MadGraph dialects round-trip."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [events, lhef, io, round-trip, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n23-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L383-L561", title: "Note 23 E3, the LHEF writer"}
  - {id: n27-b7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L1039-L1124", title: "Note 27 B7, the source-text-preserving round trip"}
  - {id: n27-findings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L1223-L1243", title: "Note 27 §7 findings register"}
  - {id: n24-gates, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1954-L1975", title: "Note 24 P4 gate (a), the corpus guard"}
  - {id: mg-lhe-parser, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/lhe_parser.py", title: "MadGraph lhe_parser.py (delivered-file formats)"}
---

# LHEF reader and writer

`vibegraph-lib/src/lhef/` reads and writes Les Houches event files (LHE
version 3.0, `LHE_VERSION`). The field conventions — what each column holds —
are [events/lhef-record-conventions](lhef-record-conventions.md); how the
accept/reject weights become a file's events is
[events/lhef-weight-strategy](lhef-weight-strategy.md).

## Layers

| module | role |
|---|---|
| `record` | the `<init>` and `<event>` blocks as data (`LheInit`, `LheEvent`, `WeightStrategy`, `BlockSource`); field names follow the accord's common blocks (`IDBMUP`, `XWGTUP`, `ICOLUP`, …) |
| `write` | `LheWriter::begin` / `begin_with_blocks` → `write_event`* → `finish`, streaming; `generator_element`, `mg_run_card`, `initrwgt_block`, `rwgt_block` |
| `parse` | `LheFile::parse`, reading the same layout back |
| `build` | assembling a record from what the generator produced: `SubprocessRecord::new` once per subprocess, `event` / `event_with_intermediates` per accepted event, `relabelled` for flavour-group members, `scalup`, `pt_clust_scales` |
| `emit` | `UnweightStrategy` (`Buffer`, `StochasticRounding`), `EventSource`, `EmitPlan` |
| `resonance` | which propagators become status-2 records ([events/resonance-records](resonance-records.md)) |
| `observables` | a parsed event read back as the named kinematic and categorical quantities the `samples` gate compares |

`build` takes momenta as every external leg in `[E, px, py, pz]`, **incoming
first with physical signs**; the writer permutes to the accord's
`px py pz E`[^n23-e3].

## XML by library, records by hand

`quick-xml` (0.32) owns the *document*: the `<LesHouchesEvents>` root,
`<header>`, `<init>`, `<event>`, and skipping past banner `CDATA` cards and
per-event `<mgrwt>`/`<rwgt>` blocks. It does not own the bodies of `<init>`
and `<event>`, which are Fortran fixed-format numeric records, parsed and
written column by column. The reader runs with `check_end_names = false`:
banners in the wild are not reliably well formed, and a banner defect must not
cost the events. Content the writer authors (the `<generator>` element, the
header comment, `<MGRunCard>`) goes through the XML writer so it is escaped;
`<MGRunCard>` is element text with only `<`, `>`, `&` escaped, not CDATA,
because Pythia 8.312 drops CDATA content (see [events/pythia-interop](pythia-interop.md)).
Content merely carried through is re-emitted verbatim[^n23-e3].

## The layout is MadGraph's delivered file, not its Fortran

The delivered `unweighted_events.lhe` is rewritten by
`madgraph/various/lhe_parser.py`, not left as `Source/rw_events.f`'s
`(i2,i5,e16.7e3,3e15.7)`, which writes the intermediate per-channel files. The
writer emits the Python formats[^mg-lhe-parser]:

```text
init beam line:    "%d %d %e %e %d %d %d %d %d %d"
init process line: "%e %e %e %d"
event line:        "%2d %6d %+13.7e %14.8e %14.8e %14.8e"
particle line:     " %8d %2d %4d %4d %4d %4d %+13.10e %+13.10e %+13.10e %14.10e %14.10e %10.4e %10.4e"
```

The Fortran and Python formats disagree on every line (two exponent digits,
not three; nine significant digits in the scale fields, not seven). The values
in a banked file's scale fields still carry only seven significant digits,
because they came through the Fortran first, so they end in two padding zeros.
The widths never actually pad, so the layout is in effect one space between
columns. The line-by-line detail of MadGraph's output path is
[references/codebases/madgraph-lhe-output](../references/codebases/madgraph-lhe-output.md).

## Two dialects, and the source-preserving round trip

MadGraph writes the delivered file twice: Fortran first, then a Python pass that
reformats only the fields it had to parse. When unweighting is all it needs,
every field except the rescaled weight passes through in the Fortran spelling
(`0.25000000000E+03`, a lifetime and helicity of `0.` and `1.`, an info line
without padding). Which dialect arrives depends on whether a systematics pass
forced a full parse (keyed off `use_syst` and the beams; when measured, the
converted runs were the 8 proton-beam runs and 6 MadGraph 3.5.7 evidence runs,
14 of 34), and nothing in the file says which it got[^n27-b7][^n27-findings].

So `parse` keeps each block's record lines as a `BlockSource` (one owned
string per `<init>` and per `<event>`) beside the decoded values, and `write`
hands a line back **verbatim only after decoding it again and finding it to
spell the record being written**[^n27-b7]:

- reuse is checked, not flagged — no mutation tracking, no dialect enum; a
  caller who edits a field gets that line in this writer's layout and the rest
  of the block in the file's own spelling;
- a block whose record-line count no longer matches (a leg added or dropped)
  drops its source whole, and so does a reordering of legs
  (`observables::canonical`);
- a record *built* rather than read carries no source, so everything this crate
  generates is written in the layout above.

`LheInit` and `LheEvent` compare by value with a hand-written `PartialEq`: the
source says how one file spelled a record, not what the record is. Owned
strings rather than spans were measured to be the right trade: with the source
carried *and* the gate's second, source-dropped pass, the 34-file corpus of the
time took 23.7 s and 853 MB peak RSS, against 22.3 s and 724 MB with the source
dropped at parse and no second pass. The whole second pass therefore costs about
what the reformatting it replaces cost, so carrying a verified line is cheaper
than formatting thirteen fields; the extra memory is the owned text of the two
200k-event Drell–Yan banks. Spans would have put a lifetime on
`LheFile`[^n27-b7].


## A file is a lossy record of the run

`<init>` cross sections carry seven significant digits, `XWGTUP` eight,
momenta eleven. Re-serialising what was read reproduces the file exactly;
recovering the *run* is good only to those precisions[^n23-e3].

## The round-trip gate, and why it states its split

`validate_lhef::banked_files_round_trip_byte_for_byte` parses every banked
`Events/*/unweighted_events.lhe.gz` into our record types, writes it back, and
requires identical bytes. Because pass-through files would round-trip whatever
our columns were, the gate re-serialises every run a second time with the source
dropped, requires at least one to still reproduce MadGraph's bytes, and prints
the own-layout / pass-through split. The corpus is discovered, so it guards its
own coverage: it requires at least one run with a hadron-collider `<init>`
(proton beam ids and an LHAPDF id in `PDFSUP`) and one with colour lines on an
incoming leg[^n24-gates]. `the_round_trip_is_sensitive_to_every_convention_sensitive_field`
mutates a parsed banked file eight ways (`MOTHUP` dropped or swapped, incoming
`ISTUP` sign, `ICOLUP` slots exchanged, incoming momenta crossed, momentum tuple
rotated, mass replaced by the invariant, `SPINUP` zeroed) and requires each to
break the round trip[^n23-e3]. Thresholds and current counts are
[validation/event-output-gates](../validation/event-output-gates.md).

What the round trip provably cannot see: any physics field filled with a wrong
number (it re-emits MadGraph's values), whether our events are distributed
like MadGraph's, and colour-line integers (it re-emits the integers it read).

## Colour-line labels are not comparable across implementations

Only the induced connectivity of colour lines is physical. vibegraph's tags
equal MadGraph's 501-based numbering on most subprocesses, but four
gluon-initiated ones (`gg_to_gg`, `gg_to_ttx`, both `gg_bbx`) are
relabellings of the same connectivity. Any byte-level comparison of our `.lhe`
with MadGraph's must normalise colour labels first; the `samples` gate and
`color_flow_tags_oracle` compare connectivity, not integers
([validation/colour-oracles](../validation/colour-oracles.md))[^n23-e3].

Event *content* is never compared with MadGraph's event by event: the two
generators do not share a random stream, so such a comparison would pair
unrelated phase-space points. Distribution-level agreement is the `samples`
gate.

[^n23-e3]: Note 23 E3 outcome: layers, XML split, conventions, blind spots.
[^n27-b7]: Note 27 B7 outcome: BlockSource, checked reuse, the dialect split and its cost.
[^n27-findings]: Note 27 §7, finding 6 (two LHE dialects keyed off `use_syst` and beams).
[^n24-gates]: Note 24 P4 gate (a), the discovered corpus and its coverage guard.
[^mg-lhe-parser]: MadGraph `madgraph/various/lhe_parser.py`, the delivered file's format strings.
