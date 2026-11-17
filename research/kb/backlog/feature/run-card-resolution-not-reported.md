---
type: Backlog Item
title: "The resolved run card and each key's classification are not reported before integrate"
description: "Only the integrate artifact records each run-card key's resolved value and FIELD_CLASSES class, so card-reading tools (simshop's form and clone, CMS card audits) re-implement the parser and drift."
area: feature
state: open
priority: medium
closes_when: "One command (enumerate --run-card's ldjson result, or a cards subcommand) reports, without sampling, every key of the given run card and every key it leaves at its default: the resolved value, MadGraph's LO default, whether the card moves it, and its FIELD_CLASSES class (consumed, ignored-benign, ignored-physics refused off default), plus the defaults the process implies (maxjetflavor, decay-cut removal) and warnings; a test pins the report against RunCard parsing for the committed validation cards and one refused card."
blocked_by: []
opened: 2026-11-17
tags: [cli, run-card, ldjson, madgraph-parity, ui, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by simshop, the CMS web UI that runs vibegraph through batch-proxy.
vibegraph already resolves this. The parser resolves each run-card name against
MadGraph's LO defaults
([run-card/run-card-parser-and-defaults](../../run-card/run-card-parser-and-defaults.md)),
and `FIELD_CLASSES` gives every name one class
([run-card/field-classification](../../run-card/field-classification.md)).
But the result appears only in the integrate artifact's resolved run card
([pipeline/integrate-artifact](../../pipeline/integrate-artifact.md)).
`enumerate --run-card` reads the card only to preview flavour groups.

So every tool that reads cards ahead of a job keeps its own copy of the parser,
the defaults and the "is this key off" rules:
- simshop's New sample form;
- simshop's clone from a CMS gridpack, which uses a defaults table built from
  the host survey;
- the CERN host's refusal classification.

Where a copy disagrees with vibegraph, the user learns it only when a batch
integrate starts.

Per key, the report would give:
- the resolved value and the default;
- whether the card moves the key off its default;
- the class, plus the refusal integrate would emit for an ignored-physics key
  off its default;
- the defaults the process implies (`maxjetflavor` for an incoming b, cuts
  removed for a decay, `auto_ptj_mjj` under matching);
- warnings such as `xqcut > 0` with `ickkw = 0`.

Keys the parser does not recognise should be listed, not dropped. That covers a
CMS gridpack's `$DEFAULT_PDF_SETS` placeholder.

This is the run-card half of
[`integrate-has-no-check-only-mode`](integrate-has-no-check-only-mode.md), and
it can ship first. It is the per-card counterpart of
[`run-card-classification-table-missing`](run-card-classification-table-missing.md),
the static per-release table. One implementation could serve both.
