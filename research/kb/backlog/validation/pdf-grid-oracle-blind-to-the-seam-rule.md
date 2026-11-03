---
type: Backlog Item
title: "The LHAPDF grid oracle cannot see which band reads a Q² seam"
description: "validate_pdf_grid probes NNPDF31_lo_as_0130 member 0, whose duplicated seam rows are bit-identical, so a member read through the wrong band at an interior seam passes it; the lower-band rule is pinned against LHAPDF only on synthetic bands."
area: validation
state: open
priority: medium
closes_when: "The LHAPDF grid oracle probes a set whose seam rows differ (HERAPDF20_NNLO_EIG or an MSHT20 member) at interior Q² seams, and the interior-seam mutant of select_band fails it."
blocked_by: []
opened: 2026-11-03
tags: [pdf, lhapdf, oracle, seams, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: mv1, resource: "../../sprints/cms-pdf-memory/sessions/M-V1-report.md", title: "cms-pdf-memory M-V1 report, Found"}
---
Under the interior-seam mutant `partition_point(<=).min(len-1)` in
`select_band`, the in-tree table oracle over the CMS sets finds 1.88 M
differences, all on HERAPDF20, MSHT20, NNPDF30 and a synthetic set. NNPDF31
and NNPDF40 show none, and NNPDF31_lo member 0 is what `validate_pdf_grid`
compares against LHAPDF. So the oracle against LHAPDF itself cannot tell
which band reads a seam. The table oracle compares the crate against its own
earlier algorithm, so it pins stability, not LHAPDF's rule.
