---
type: Backlog Item
title: "The gpdb text-output parser keeps the last multiplicity line per tag, under-recording per-group σ"
description: "parse_vg_output dedups multiplicity lines by tag, last wins (parse.py:733); since per-group lines repeat a tag, text mode would record one group's σ. #106 fixed only the ldjson path."
area: validation
state: open
priority: low
closes_when: "parse_vg_output sums a tag's per-group lines (errors in quadrature) as parse_vg_ldjson does, or the text path is dropped, with a test on a text output that repeats a tag."
blocked_by: []
opened: 2026-11-07
tags: [gpdb, cms, tooling, parser]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: #106 and its Found"}
---
Since #103/#104 a run reports its multiplicities per flavour group, so a tag
repeats. #106 made `parse_vg_ldjson` sum a tag's groups with errors in
quadrature. The text path, `parse_vg_output`
(`validation/cms_gridpacks/parse.py:704`), still deduplicates with
`seen[mm["tag"]] = mm` (`:733`): the last line per tag wins, so a text-mode run
would record one group's σ for the tag. The campaigns run in ldjson mode, so
nothing recorded so far is affected (**inferred** from #106's crash being in the
ldjson path).
