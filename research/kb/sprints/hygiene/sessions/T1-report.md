---
type: Session Report
title: "T1 report: tooling and CI fixes"
description: "Five script and workflow fixes, one commit each; four items meet closes_when, and the acceptance.yml change awaits the next refdata release."
status: draft
tags: [hygiene, tooling, ci, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: f578bde, resource: "https://github.com/nsmith-/vibegraph/commit/f578bde", title: "profile.sh: forward harness arguments separately from samply's"}
  - {id: e098b1b, resource: "https://github.com/nsmith-/vibegraph/commit/e098b1b", title: "madgraph generators: C++ runtime by platform; gen_higgs_window refuses non-3.5.7"}
  - {id: 608e7ae, resource: "https://github.com/nsmith-/vibegraph/commit/608e7ae", title: "host_info: CPU and memory block from /proc on Linux"}
  - {id: 19c8282, resource: "https://github.com/nsmith-/vibegraph/commit/19c8282", title: "release.yml C dependencies; extended-validation skill regeneration trigger"}
  - {id: a4db76f, resource: "https://github.com/nsmith-/vibegraph/commit/a4db76f", title: "acceptance.yml: v* releases and manual dispatch only"}
---
The dev agent's report, condensed by the manager. Commands and outputs are
the agent's except where the manager check says otherwise.

## Items

| Item | Commit | closes_when |
|---|---|---|
| profile-script-forwards-one-filter | f578bde | met |
| madgraph-generators-hardcode-lcxx-and-bypass-pin | e098b1b | met |
| host-info-null-cpu-block-on-linux | 608e7ae | met |
| ci-and-agent-skill-docs-stale | 19c8282 | met |
| acceptance-yml-fails-on-refdata-releases | a4db76f | pending: only a published `refdata-*` release can show it |

- **profile.sh:** usage `profile.sh <test-name> [test-filter] [-- harness-args...] [--samply samply-args...]`.
  The old `-- samply-args` position now means harness arguments. Nothing in
  the repository used it, and the pixi tasks pass the test name only.
  Demonstrated by stubbing `cargo` and `samply` and printing their argument
  vectors.
- **MadGraph generators:** all five `-lc++` sites now use `mes_ldflags`.
  `gen_higgs_window.sh` keeps the packaged mg5_aMC, because its reference is a
  3.5.7 measurement, and refuses any other version through
  `require_packaged_version` (the item's "refuse" alternative). Routing it
  through `mg5_pinned.sh` would have made the recorded `mg_version` false.
- **host_info.py:** `/proc` is read only when `sysctl` returns nothing, so
  macOS output is unchanged. Linux adds `family`, `model_id` and `stepping`.
  `frequency_hz` stays null there.
- **release.yml:** the comment names zstd-sys, ring and libmimalloc-sys. The
  item named two; `Cargo.lock` shows a third `cc` user.
- **acceptance.yml:** the job runs on
  `workflow_dispatch || startsWith(github.event.release.tag_name, 'v')`. It
  uses the tag prefix rather than `prerelease`, so a `v*` release candidate
  still runs.

## Method

The agent read the five items, edited each site and demonstrated each change
with stubs on `PATH`. `shellcheck` 0.11, `actionlint` 1.7 and `ruff` were
installed user-locally. No cargo or MadGraph was run.

## Manager check (2026-10-09)

Re-run from the committed tree:

- `shellcheck scripts/profile.sh`: clean.
- `actionlint` on `acceptance.yml` and `release.yml`: clean.
- `bash -n` on the three generators: clean.
- The `host_info.host_block()['cpu']` output is filled as reported.
- `mes_ldflags` gives `-lstdc++` on this host and `-lc++` under a stub
  `uname` printing Darwin.
- `grep` shows no `-lc++` left in the three generators outside comments.
- Every commit carries only an `Assisted-by` trailer.

## Found

- **The bundle concept `tooling/instruction-level-profiling.md:51`** still
  says `scripts/profile.sh` forwards only one filter. Update it at close-out,
  together with the new usage. (Found by the manager.)
- **`gen_higgs_window.sh` writes `mg_version` 3.5.7 as a literal** rather than
  reading it from `MGMEVersion.txt`. It is now guarded. Low priority.
- **`host_info.py:22,31` trip `ruff` PLW1510** (`subprocess.run` without
  `check=`). This predates T1. It matters only if ruff becomes a gate.

## Brief corrections

- The `-- samply-args` syntax in the brief and the item is superseded, as
  above.
- `ci-and-agent-skill-docs-stale` missed `libmimalloc-sys`.
