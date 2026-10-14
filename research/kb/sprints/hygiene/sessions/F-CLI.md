---
type: Session Brief
title: "F-CLI: CLI and validation-report fixes"
description: "Fix R-F's 15 triaged findings in vibegraph-cli and validation-report: untested check-events complaints, exit-code-only refusal tests, loose CLI σ checks, unnamed-file errors and stringly-typed report state."
status: draft
agent: feature-dev (Opus)
depends_on: [triage]
closes: [no-network-variable-read-two-ways]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-F.1, .3, .5, .6, .7, .8, .11 (the CLI and report sites: `assets.rs`, `validation-report/src/main.rs`), .16, .17, .18, .21, .22, .23, .24, .25, .26 (`sessions/R-F-report.md`).

**Claimed items closed here:** no-network-variable-read-two-ways.

**Notes:**
- **R-F.23:** serde enums must accept exactly the strings the row writers emit (`vibegraph-lib/tests/common/report.rs`). Run the collator over a real report directory before and after, and diff the rendered report: it must be identical.
- **R-F.21's schema check** fails loudly on a mismatch. Show that it does.
- **Banked gates:** the CLI's extended-validation tests your changes touch (`cli_integrate`, …).
