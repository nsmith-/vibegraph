---
type: Backlog Item
title: "The kT dump generator demands a byte-identical MadEvent replay, so its gates only run on the banking host"
description: "gen_kt_cluster_dumps.sh refuses a replay that differs from the bank in any byte; on Linux 2 of 70001 lines differ at 1.4e-11, so validate-kt-cluster cannot run off macOS."
area: validation
state: open
priority: medium
closes_when: "The replay check compares numerically at a tolerance set by the print precision with measured headroom, normalises MadGraph's development-version banner line, and validate-kt-cluster runs on a Linux host with LHAPDF."
blocked_by: []
opened: 2026-11-09
tags: [validation, kt-clustering, reproducibility, long-tier]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: iz2, resource: "../../sprints/v02-last-refusals/sessions/I-Z2-report.md", title: "I-Z2 report"}
---
I-Z2's probe ran `VG_KT_STAGE=extract … gen_kt_cluster_dumps.sh ee_to_ttx`
on Linux x86 with the pinned MadGraph 3.7.1. It is the same run as the bank,
but 2 of 70001 event lines differ in the last printed digit (for example
`-0.72372879882E+02` against `…881E+02`). The banner also differs, because the
local MadGraph prints "UNKNOWN DEVELOPMENT VERSION".

AGENTS.md ("ULP exactness is never the target") says not to pin float results
to exact bits across platforms. The same dumps feed
`validate_mlm_dumps::matched_scales_reproduce_madevents_event_by_event`, the
finest per-event scale oracle, which also could not run in sprint I.
