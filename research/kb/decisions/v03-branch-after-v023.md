---
type: Design Decision
title: The v0.3 line splits from main at the v0.2.3 tag
description: "release/v0.2 is cut from the v0.2.3 tag and main becomes v0.3 development, where lane-batched integration lands; lane work that keeps f64 output bit for bit lands on main before then."
decided: 2026-11-22
decided_by: human:nsmith-
status: stable
tags: [release, branching, v0.3, simd, lanes]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: lanes, resource: "../performance/simd-lane-evaluation.md", title: "Lane-batched evaluation of eval_m2"}
  - {id: sprint-l, resource: "../sprints/v023-orders-majorana-timeouts/sprint.md", title: "Sprint L plan"}
---
**The split.** `release/v0.2` is cut from the v0.2.3 tag. After that, `main`
is v0.3 development. v0.2.x patch releases are tagged on `release/v0.2`: a fix
lands on `main` first and is cherry-picked across, unless the code has
diverged so far that the fix is written for `release/v0.2` directly.

**Why then.** A branch costs a second set of merges and a second `banked` run
per change, so it is cut only when the two lines must differ. The first change
that cannot keep v0.2's banked σ lines bit for bit is the switch to
lane-batched integration (`lane-batching-not-in-production`). It changes the
order of sampler draws, so every banked σ line moves within its error, and the
integrate artifact probably has to record its sampler kind. Splitting at
v0.2.3 also keeps squared orders and Majorana support, the large correctness
changes of sprint L, on a single line.

**What lands on main before the split.** Every lane prerequisite whose
`closes_when` requires byte-identical output at `F = f64`:
- `real-has-no-lane-masks-or-bin-search`, starting with its design session with the user;
- `sampler-hardwired-to-vegas-grid`;
- `per-point-chain-f64-only`;
- `per-point-branches-f64-only`;
- `sampler-draw-not-lane-generic`.

Jet clustering, PDF lookup and per-lane αs follow the same rule. Each lands
before the split only if it is byte-identical at f64.

**Consumers.** simshop and the CERN host's batch run pinned release binaries,
so they stay on v0.2.x until a v0.3 release exists. The branch's CI and
ruleset setup is `release-branch-ci-and-ruleset-missing`.
