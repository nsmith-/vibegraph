---
type: Session Report
title: "C1 report: the MLM notes, the ensemble figure, the census links and the attribution rule"
description: "V1's Found 5-8 closed on the PR: the five MLM manifest notes re-recorded against refdata-10, probe_llj_seed_ensemble's forty-seed figure replaced by the 20-seed reading, the remaining concept pages repointed at the 2026-10 census, and a first-seed-sigma rule for drift attribution."
status: draft
agent: claude-code (Sonnet)
tags: [sigma-gate, calibration, seed-sweep, refdata, manifest, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: v1, resource: "V1-report.md", title: "V1 report (Found 5-8)"}
  - {id: census, resource: "../../../validation/seed-headroom-census-2026-10.md", title: "Seed-sweep headroom of the σ gates (2026-10)"}
---
Worktree `/home/user/wt/sigma-calibration-rerecord`, branch
`sigma-calibration-rerecord`, base `474948e` (tree `1db7000`). No tolerance,
budget or gate code changed.

## 1. MLM manifest notes (Found 5)

`validate-mlm-sigma` runs `sigma_llj_xqcut_only_vs_madevent`,
`sigma_llj_mlm`-prefixed tests (`_mlm_` and `_mlm_alps2_`),
`sigma_ll_0j2j_mlm_vs_madevent` and `sigma_ttx_0j1j_mlm_vs_madevent`; its only
dependency is `fetch-pdf`, so under `--skip-deps` it regenerates nothing. V1
measured the first two (359 s and 114 s alone); the other three ran together,
the same cargo command restricted with `--exact`:

```
$ cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_hadronic -- --ignored --nocapture --exact sigma_llj_mlm_alps2_vs_madevent sigma_ll_0j2j_mlm_vs_madevent sigma_ttx_0j1j_mlm_vs_madevent
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 1846.37s
```

HEAD `474948e`, tree `1db7000` (the working tree also held this session's
comment-only edit to `validate_hadronic.rs`, made while cargo was compiling;
the edit changes comments and one report-cell note string). Log
`rerecord-C1-mlm3.log`. V1's two measurements are on tree `3f9a26f`
(`rerecord-V1-sigma_llj_mlm_vs_madevent.log`,
`rerecord-V1-sigma_llj_xqcut_only_vs_madevent.log`).

| row | this side | seed record | rel | pull | seed χ²/dof |
|---|---|---|--:|--:|--:|
| `pp_to_llj_mlm` (V1) | 268.7417 ± 0.1155 (10 seeds) | 268.1360 ± 0.5932 | +0.23% | +1.00 | 0.81 |
| `pp_to_llj_xqcut_only` (V1) | 212.7019 ± 0.1527 (5) | 212.3040 ± 0.4441 | +0.19% | +0.85 | 1.52 |
| `pp_to_llj_mlm_alps2` | 240.9774 ± 0.1439 (5) | 240.4980 ± 0.5242 | +0.20% | +0.88 | 0.26 |
| `pp_to_ll_0j2j_mlm` @0 / @1 / @2 / total | 665.4279 / 268.8866 / 132.4486 / 1066.763 | 665.0684 / 268.6159 / 131.5321 / 1065.220 | +0.05% / +0.10% / **+0.70%** / +0.14% | +0.49 / +0.36 / +1.90 / +1.34 | 0.94 / 0.68 / 0.97 / 0.39 |
| `pp_to_ttx_0j1j_mlm` @0 / @1 / total | 513.1846 / 579.1429 / 1092.328 | 513.1058 / 578.3374 / 1091.440 | +0.02% / +0.14% / +0.08% | +0.31 / +0.70 / +0.75 | 0.85 / 0.60 / 0.77 |

All gates pass. The `0j2j` and `ttx_0j1j` notes were not refdata-9 only: the
`ttx_0j1j` note already said refdata-10 (2026-10-17) but its figures
(`@1` 578.7796, total 1091.999) are from an earlier tree; they are re-recorded
too. `0j2j` `@2` reads +0.70% (pull +1.90), where the old note had +0.32%
against 21 MadEvent directories; the note's registered deviations (about
0.9 pb, 0.7% of `@2`) are now the whole of the offset, and the gate's 3σ on
`@2` is 1.45 pb (1.1%). The note's "inside 3 sigma only while this side's
error stays near 0.66 pb" is restated against the measured 0.38 pb.

Edits: the five `integrals` notes in `validation/manifest.toml`. The per-event
(dump) parts of the notes are refdata-9 measurements of the oracle layer and
were left.

## 2. `probe_llj_seed_ensemble` figure (Found 7)

The forty-seed claim ("2x at 150k and 5x at 600k"), taken on an earlier
sampler, is dropped from the `RECARDED_ROWS` comment and the `pp_to_llj` cell
note. Both now state V1's measurement: twenty seeds at 150k read one seed's
spread at 1.01 of its quoted error, against 0.70 over the gate's five (V1
report §4, CLI run seeds 20261101-20, binary from `5782665`, library source
equal at tree `340683c`). No ensemble was re-run.

## 3. Census links (Found 8)

`seed-sweeps-and-budget-ladders.md` (both links; the `ud_to_epemud_qcd0`
sentence now reads `3.80e-3`, 2.6× inside `rel_tol`, against the refdata-10
record, naming the three commits V1 bisected instead of "the note-34 draw
commits"), `gate-thresholds.md`, `reference-row-rationale.md` (same `3.80e-3`
figure) point at the 2026-10 page. `samples-gate.md` keeps the 2026-09 link
for `P_FLOOR` (`1.573e-4`): that page is the only one that measures it, and the
2026-10 page says it does not re-measure samples; the sentence now says so. The
`3.80e-3` and 2.6× are the `ud_to_epemud_qcd0` row of the 2026-10 page.

## 4. Process rule (Found 6)

Added to the "Protocol for an attribution measurement" section of
`seed-sweeps-and-budget-ladders.md`: a claim that a commit moved the sampling
stream carries a first-seed σ at that commit and at its parent; dating by when
comments were written is not evidence. Case cited: V1's bisection of the 108
first-parent commits (the 2026-09 census's three commits were bit-identical to
their parents).

## Gates

See the commit message and the reply; commands and results:

```
$ cargo fmt --all --check                                  exit 0
$ cargo clippy --workspace --all-targets -- -D warnings    exit 0
$ cargo test --workspace      40 binaries: 1538 passed, 0 failed
$ pixi run --skip-deps kb-index   wrote 0 index files
$ pixi run --skip-deps kb-lint    798 concepts, 270 backlog items; 0 errors, 4 warnings
```

The four lint warnings are `state: blocked` without `blocked_by` on backlog
items (V1 recorded three; the planner's backlog edits added the fourth).
