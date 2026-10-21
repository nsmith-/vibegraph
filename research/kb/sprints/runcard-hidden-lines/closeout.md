---
type: Sprint Record
title: "Run-card hidden lines and the systematics warning"
description: "Run-card lines tagged '# hidden_parameter' are read as MadGraph reads them, and a card asking for systematics weights warns once that none are written."
closed: 2026-10-20
status: draft
tags: [runcard, parser, systematics, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/58", title: "nsmith-/vibegraph#58"}
  - {id: banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L2900-L2902", title: "MadGraph RunCard.read: strip '#', then '!', split at the last '='"}
---
One PR ([nsmith-/vibegraph#58](https://github.com/nsmith-/vibegraph/pull/58))
closing `runcard-hidden-parameter-lines-dropped` and
`systematics-request-dropped-silently`. Sessions: [F1](sessions/F1-report.md)
(feature-dev, Sonnet) and [HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet).

## What was established

- **Hidden lines.** `split_line` cuts a line at its first `#` or `!`, as
  MadGraph's `RunCard.read` does (`banner.py:2900-2902` at b7687064, read by the
  manager). `<v> = <name> # hidden_parameter` now reads as `<v> = <name>`, and
  `False = pdfwgt # hidden_parameter` on an MLM card reaches the matching
  weight. The split at the first `=` (MadGraph: the last) is left to
  `runcard-value-with-equals-dropped`.
- **No tracked card reads differently.** No committed run card carries
  `# hidden_parameter`. Which of the survey's 66 CMS cards now meet a new
  refusal (an unimplemented cut behind a hidden line) is unmeasured: the cards
  are on the CERN host.
- **Systematics.** With `use_syst` on and any `B_SYST` field off its default,
  one warning names the fields and says the `<rwgt>` variation weights are not
  written; σ and events are unchanged. The field list is derived from
  `FIELD_CLASSES`, so a new `B_SYST` field warns without a second edit, and
  each of the seven is tested alone.
- **Test capture.** One crate-private `#[cfg(test)]` `test_capture` module
  replaces the private subscribers in `progress.rs` and `runcard/classes.rs`,
  with `tracing`'s callsite-interest cache handled once.

The manager re-ran the new tests from the committed tree at F1 and after HY1.

## Backlog

- Deleted: `runcard-hidden-parameter-lines-dropped`,
  `systematics-request-dropped-silently`.
- Filed: `budget-test-keeps-private-capture-subscriber` (hygiene, low).

## Process notes

The HY1 brief named `budget.rs` among the subscriber copies, though the sprint
keeps out of `budget.rs` while its design session is open; the manager
reverted that part.
