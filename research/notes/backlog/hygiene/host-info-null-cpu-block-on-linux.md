---
type: Backlog Item
title: host_info.py records no CPU or memory on Linux
description: validation/madgraph/host_info.py reads only macOS sysctl, so the CPU block of mg_timings.json and timings.json is null on Linux hosts.
area: hygiene
state: open
priority: medium
closes_when: On Linux, host_info.py fills the CPU block from /proc/cpuinfo (model name, family/model/stepping, logical CPUs) and memory from /proc/meminfo, with the macOS path unchanged.
blocked_by: []
opened: 2026-09-26
tags: [tooling, timing, host, linux]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1000-L1003", title: "TODO.md entry T093"}
---
`validation/madgraph/host_info.py` builds the host block from `sysctl` keys only
(`machdep.cpu.brand_string`, `hw.logicalcpu`, `hw.perflevel0.logicalcpu`,
`hw.memsize`, …; `host_info.py:20-57`). On Linux these return nothing, so the
CPU block of `mg_timings.json` and `timings.json` is null. On the Cascade Lake
run it was filled in by hand. Read `/proc/cpuinfo` (model name, family / model /
stepping, logical CPUs) and `/proc/meminfo` there.

Detail: [mg-comparison-cascade-lake-results.md §5](../../mg-comparison-cascade-lake-results.md).
