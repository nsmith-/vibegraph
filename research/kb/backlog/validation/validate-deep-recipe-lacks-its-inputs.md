---
type: Backlog Item
title: "validate-deep's catch-all cannot satisfy its own gated tests' inputs"
description: "The --ignored catch-all runs without LHAPDF_DATA_PATH and without the MLM/kT dumps, so environment gaps read as failures and the per-event MLM scale oracle silently never runs."
area: validation
state: open
priority: low
closes_when: "The validate-deep recipe sets LHAPDF_DATA_PATH and SYSTEMATICS_CMS_PDFS, fetches or builds the dumps the MLM and kT oracles need (or skips them by name with a recorded reason), and runs to a clean exit on a fresh host."
blocked_by: []
opened: 2026-11-09
tags: [validation, long-tier, validate-deep, ci]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: iz2, resource: "../../sprints/v02-last-refusals/sessions/I-Z2-report.md", title: "I-Z2 report"}
---
In sprint I's run on 2b26c109:
- `the_pdfas_member_reads_its_own_coupling_and_clamp` passed in its own task
  but failed in the catch-all, where `LHAPDF_DATA_PATH` is not set.
- `matched_scales_reproduce_madevents_event_by_event` and
  `the_jet_memo_is_the_channel_s_restricted_jet_count` panic on the absent
  `output/ktdump/dumps`, which the bundle does not ship.
- `madgraphs_module_on_other_files` needs `SYSTEMATICS_FILES`.

Related: [long-tier-gates-never-run-in-ci](long-tier-gates-never-run-in-ci.md).
