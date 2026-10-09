---
type: Session Brief
title: "F-F: library I/O fixes"
description: "Fix R-F's 7 triaged library findings in config, cache, lhef/emit and artifact, and close the artifact version-arm and run-card Opaque-default items."
status: draft
agent: feature-dev (Opus)
depends_on: [triage]
closes: [artifact-reader-arm-names-format-version, runcard-opaque-defaults-unverified]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [fix protocol](fix-protocol.md). The findings are listed in
[triage.md](../triage.md), "Fix here, by session"; read each in its review
report.

**Findings:** R-F.9, .10, .11 (the library sites: `validation.rs`, `cache/mod.rs`, `cache/resolve.rs`), .12, .13, .14, .20 (`sessions/R-F-report.md`).

**Claimed items closed here:** artifact-reader-arm-names-format-version, runcard-opaque-defaults-unverified.

**Notes:**
- **R-F.20:** its literal-9 sites go with artifact-reader-arm-names-format-version.
- **runcard-opaque-defaults-unverified:** the item names the fix (normalise the Python reprs). Its pinning test's known-mismatch set must end empty.
- **Banked gates:** `validate_lhef`, if `lhef/emit.rs` changes.
