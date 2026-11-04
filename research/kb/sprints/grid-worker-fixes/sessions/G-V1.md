---
type: Session Brief
title: "G-V1: the ttx MLM row's own events against MadEvent's"
description: "Generate the pp_to_ttx_0j1j_mlm row's own process, now possible, and gate its record against MadEvent's banked run_01 as population fractions; the first test that generates that row."
status: draft
agent: validation-dev (Opus)
depends_on: [G-F2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker` (draft PR #87). Start
at the commit that adds this brief; G-F2's commit, e9041eb, is below it. Read
the [G-F2 report](G-F2-report.md) and G-F2's banked CLI test
`a_quark_jet_top_pair_is_generated_across_a_mixed_flavour_group`. Then read
`matched_sample_record_fractions_against_madevent`
(`vibegraph-cli/tests/cli_generate_proton.rs`, about line 1694), and Found 1
and Found 4 of `research/kb/sprints/sigma-ttx-mlm-at1/sessions/V2-report.md`.
Pixi is not installed. The CLI package is `vibegraph`. This brief may be
wrong: if the code says otherwise, say so in the report.

## Why

The `pp_to_ttx_0j1j_mlm` row's process (`p p > t t~ @0`,
`p p > t t~ j @1`, with `j` including the gluon) was refused at generate
until G-F2, and no test has ever generated it (G-F2's Found 1). The banked
MadEvent sample `validation/madgraph/output/pp_to_ttx_0j1j_mlm/Events/run_01`
is in refdata-13. Never write into it.

## Task

- Generate the row's own process from its proc and run cards with
  vibegraph.
- Compare the record against MadEvent's `run_01` as population fractions,
  each a two-sample binomial comparison within five standard deviations, in
  the style of the llj test. Fractions:
  - the share of 1-jet (`@1`) events;
  - among those, the gluon-jet and quark-jet shares, and the quark flavour
    shares where the counts allow;
  - the initial-state shares (gg, qg, qq̄);
  - the share listing the top resonances, if both sides write them;
  - from `<mgrwt>`, the emission-scale share and the long-density-chain
    share.
- Before you enforce anything:
  - V2 records a known lab-against-CM clustering difference: about +1% of σ
    on `gg`, and a residual excess. Estimate whether it can move any fraction
    by more than about 1σ at your sample size, and say so.
  - A failing fraction is a finding to diagnose, not a threshold to widen.
- **Placement.** Put it in the banked layer if a run fits the budget the
  other banked CLI tests take (measure it). Otherwise make it an `#[ignore]`
  test with a pixi task, the way the oracle-layer test is. Record the wall
  time either way.
- Also confirm that the c-member line change G-F2 reports on `llj` and
  `ll_0j2j` (`u` becomes `c`) moves no banked gate: run the matched-sample
  and LHEF gates that read those rows.
- Gates: fmt, clippy (default and extended features), the default
  `cargo test`, and the banked tests you touched.

**Report.** `G-V1-report.md` (type Session Report), with commands, the
fraction table with counts and pulls, and Found. Commit as described in your
agent definition. Do not push.
