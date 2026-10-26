---
type: Backlog Item
title: "Four concepts describe code that sprint D changed"
description: "integrate-artifact.md says version 13 (code: 14); asset-resolution.md names the removed resolve_pdf_set_dir; flavour-groups.md and mixed-multiplicity-integrand.md omit the mass split."
area: hygiene
state: open
priority: low
closes_when: "The four concepts describe the code on main: artifact version 14, the cache::lhapdf tiers, and the split by outgoing masses before flavour grouping."
blocked_by: []
opened: 2026-10-26
tags: [kb, docs, stale]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: d1, resource: "../../sprints/pdf-from-run-card/sessions/D1-report.md", title: "pdf-from-run-card D1 report, Found 1"}
  - {id: f2, resource: "../../sprints/pdf-from-run-card/sessions/F2-report.md", title: "pdf-from-run-card F2 report, Found 2"}
  - {id: fm-hy1, resource: "../../sprints/flavour-mixed-masses/sessions/HY1-report.md", title: "flavour-mixed-masses HY1 report, Found 3 and 4"}
  - {id: fm-f1, resource: "../../sprints/flavour-mixed-masses/sessions/F1-report.md", title: "flavour-mixed-masses F1 report, Found 2"}
---
Checked on `main` at `cfd9f7a`:

- [`pipeline/integrate-artifact.md`](../../pipeline/integrate-artifact.md) says
  `FORMAT_VERSION = 13` (:25, :45); `vibegraph-lib/src/artifact.rs` has 14, and
  the run inputs v14 records are not described.
- [`tooling/asset-resolution.md`](../../tooling/asset-resolution.md) :84 names
  `assets::resolve_pdf_set_dir`, which the PDF-from-run-card PR removed with
  `Source::DataPath`. Its PDF half should describe `cache::lhapdf::{Tiers, locate_set}`.
- [`hadronic/flavour-groups.md`](../../hadronic/flavour-groups.md) rule 2 (:41-44)
  presents `UnequalFinalMasses` as the card's fate. A card is now split by
  outgoing masses first (`multiplicity::split_by_outgoing_masses`), so the
  refusal is the decomposition's invariant. The same paragraph should say that
  MadGraph keys on the mass parameter's name and this crate on its value
  (`group_subprocs.py:485`), which agree except where distinct parameters share
  a value, where merging is exact.
- [`phase-space/mixed-multiplicity-integrand.md`](../../phase-space/mixed-multiplicity-integrand.md)
  :25-33 has one part per multiplicity ("`part_results` gives σ per
  multiplicity"), and :108 describes `MultiplicityChannel`'s `group` without the
  offset across mass parts.
