---
type: Backlog Item
title: "The README's demo GIFs show the old command line"
description: "assets/demo_ee.gif and demo_pp.gif were last rendered 2026-08-11; the tapes now use generate <ARTIFACT> --seed and LHAPDF_DATA_PATH, which the recordings do not show."
area: hygiene
state: open
priority: low
closes_when: "Both GIFs are re-rendered with vhs from the current scripts/demo_{ee,pp}.tape and show the current integrate and generate commands."
blocked_by: []
opened: 2026-10-26
tags: [docs, readme, demo]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f2, resource: "../../sprints/gridpack-generate/sessions/F2-report.md", title: "gridpack-generate F2 report, Found 1"}
---
`git log -1 -- assets/demo_ee.gif assets/demo_pp.gif` gives `7ddcba2`
(2026-08-11). The tapes were migrated by the gridpack-generate PR (#69):
`scripts/demo_ee.tape:44` runs `vibegraph generate ee_to_mumu/grid.bin.zst
--nevents 10000 --seed 1`, and `demo_pp.tape:29` exports `LHAPDF_DATA_PATH`.
The PDF-from-run-card PR (#60) then removed `integrate --pdf-set/--pdf-dir`.
The README embeds both GIFs (`README.md:114`, `:162`)
([F2](../../sprints/gridpack-generate/sessions/F2-report.md)).
