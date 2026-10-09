---
type: Backlog Item
title: Code comments cite MadGraph line numbers from before the 3.7.1 pin
description: "Comments in validate_scales/alphas/sigma, cuts.rs and runcard/matching.rs cite unwgt.f, cuts.f, setcuts.f and kin_functions.f lines from an older release, or the wrong subroutine."
area: hygiene
state: open
priority: low
closes_when: "Every citation listed in the body names the line (and subroutine) at the pinned b7687064, ideally as a permalink."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, madgraph, citations, hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L750-L761", title: "unwgt.f at the pin: SCALUP (sscale) at :752, AQCDUP (aaqcd) at :760"}
---
Each was read against the pinned tree (b7687064, the 3.7.1 tag) on 2026-10-09.
Paths are under `vibegraph-lib/`.

- **`unwgt.f:686`, SCALUP** (now `:752`): `tests/validate_scales.rs:271`, `tests/validate_alphas.rs:249`. The module-doc site `validate_scales.rs:9` is in [validate-scales-module-doc-stale](validate-scales-module-doc-stale.md).
- **`unwgt.f:694`, AQCDUP** (now `:760`): `tests/validate_scales.rs:1317`, `tests/validate_alphas.rs:379`, `tests/validate_sigma.rs:3149`.
- **`src/cuts.rs`** (module doc `:12-38`, comments `:1060-1061`):
  - `cuts.f:219-221`: the squaring is `:225`.
  - `cuts.f:429`: the `r2 < r2min` test is `:443`.
  - `cuts.f:312`: the `nincoming = 2` ŝ guard is `:310`.
  - `setcuts.f:345`: `drjj` is stored at `:346`.
  - `kin_functions.f:180`: `DELTA_PHI` starts at `:164`, and the clamp is `:181-182`.
- **`src/runcard/matching.rs:73`**: `myamp.f:343-351` is labelled `setxqcuts`. Those lines are in `set_peaks` (`myamp.f:207-595`). `setxqcuts` is `setcuts.f:892`.

Spot-checked and still right at the pin: `kin_functions.f:42`/`:95`,
`setcuts.f:212`/`:217`, and `unwgt.f:741`/`:838`. Found by drafter D2 and
verifiers V3, V11 and V12.
