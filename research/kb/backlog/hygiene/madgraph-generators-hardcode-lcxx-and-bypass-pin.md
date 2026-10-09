---
type: Backlog Item
title: Three MadGraph generators hard-code -lc++, and one bypasses the 3.7.1 pin
description: "gen_hadronic_sigma, gen_higgs_window and gen_pta_windows append -lc++ on every platform, which fails the madevent link on Linux; gen_higgs_window also runs mg5_aMC from PATH (3.5.7)."
area: hygiene
state: open
priority: low
closes_when: "The three scripts choose the C++ runtime by platform as madevent_seeds.sh does, and gen_higgs_window.sh either runs through mg5_pinned.sh or states that it is a 3.5.7 defect study and refuses another version."
blocked_by: []
opened: 2026-10-09
tags: [madgraph, tooling, linux, version-pin]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: seeds, resource: "../../../../validation/madgraph/madevent_seeds.sh", title: "madevent_seeds.sh mes_ldflags (~:26-34), the platform switch"}
  - {id: higgs, resource: "../../../../validation/madgraph/gen_higgs_window.sh", title: "gen_higgs_window.sh ~:68, ~:124"}
---
Three generators append `-lc++` to `LDFLAGS` unconditionally:

- `validation/madgraph/gen_hadronic_sigma.sh:92`
- `gen_higgs_window.sh:68, :124`
- `gen_pta_windows.sh:143, :288`

`-lc++` names libc++, the macOS runtime. On Linux the runtime is libstdc++,
and naming the other one fails the link. `build.sh`, `madevent_seeds.sh`
(`mes_ldflags`) and `time_stages.py` already switch by platform.
`generate-hadronic-sigma` is a dependency of `pixi run validate-hadronic`, so
on Linux that task fails whenever it regenerates.

`gen_higgs_window.sh:68` also runs `mg5_aMC` from `PATH` (the packaged 3.5.7)
rather than `mg5_pinned.sh`. Its committed `higgs_window_reference.json`
records `mg_version` 3.5.7, so a rerun meant as a 3.7.1 reference would
silently be 3.5.7. `pta_window_reference.json`'s 3.5.7 rows are labelled by
version and selected by version in `validate_samples.rs` (~:2230), so they are
deliberate. Found by Phase 2 drafter D15.
