---
type: Backlog Item
title: "The default -j takes every core even on a shared batch node"
description: "-j defaults to one thread per core, which on an HTCondor worker means the whole machine; it ignores the CPU affinity mask, a cgroup CPU quota and OMP_NUM_THREADS."
area: feature
state: open
priority: low
closes_when: "The default thread count is the minimum of the affinity mask, any cgroup CPU quota and OMP_NUM_THREADS when set, and logs which limit applied; a unit test covers each source."
blocked_by: []
opened: 2026-11-13
tags: [cli, batch, threads]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
---
Reported by the batch-proxy design session. HTCondor sets `OMP_NUM_THREADS`
to the job's `request_cpus` and confines it with cgroups. A job that forgets
`-j` starts one thread per physical core and oversubscribes the node. The
batch side always passes `-j`, so this only makes a forgotten flag harmless.

Check what the thread-pool crate already honours (the affinity mask, for
example) before adding detection by hand.
