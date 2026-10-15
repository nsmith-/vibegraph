---
type: Session Report
title: "J-H report: hygiene review of the jioxxx chirality and longitudinal test"
description: "Findings of the four-point hygiene review of PR #34 (test_eval_jioxxx and the jioxxx reference port), written before any edit, with their dispositions."
status: draft
tags: [hygiene, helas, review, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-15}
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/f9b0b05", title: "PR #34 tip reviewed"}
---
Review of `git diff main...HEAD` on `amp-signs/jioxxx-chiral-oracle`
(`vibegraph-lib/src/helas/eval/run.rs`, `vibegraph-lib/src/helas/vertex.rs`).
This first version of the report holds the findings as they stood before any
code edit. Fix and gate sections follow in later versions.

## Findings (before any edit)

| id | point | site | evidence | confidence | disposition |
|---|---|---|---|---|---|
| H1 | 2 | `run.rs` ALOHA cross-check, `match (m_in != 0.0, case.current_terms)` then `if let Some(aloha)` | The check is optional per case and nothing counts how many ran. A defect that makes the match fall through (a changed guard, a renamed term) leaves the ALOHA comparison, the only independent pin of the longitudinal denominator, silently empty while the test stays green. Mutation: `(true, [("FFV2", c)])` to `(false, ...)`. | suspected: settle by running the mutation | fix: count the comparisons and assert the independently derived number (2 structures x 2 propagators x 4 helicity pairs = 16) |
| H2 | 1, 4 | `run.rs` `test_eval_jioxxx` (326 lines, `awk` over the fn) and the two tests after it | The fn builds four `ExtLegInfo` literals and the e+e- current diagram inline. The same 4-node current diagram and the same two `ExtLegInfo` literals recur in `test_eval_ffv2_4_3` and `test_longitudinal_z_current_transverse_for_massless_fermions` (`grep -n "ExtLegInfo {"`: 2299, 2306, 2553, 2560, 2720, 2727). The full-amplitude diagram is a 40-line inline closure. | checked | fix: shared `fermion_leg` and `s_channel_current_diagram` helpers, amplitude-diagram builder out of the test; identity oracle is the printed residuals |
| H3 | 1 | `run.rs` comments in `test_eval_jioxxx` | (a) the `ref_amp` comment is hard-wrapped mid-sentence ("... to pin. Read reversed, the line couples" runs past its neighbours); (b) "Both comparisons are relative to the largest reference over the helicities" now sits above only the current scale, `amp_scale` is computed far below. | checked | fix |
| H4 | 1, 2 | `run.rs` closures `helas_gc` and `aloha_coupling` | Both repeat `evaluated.coupling(model.coupling_id(c).unwrap()).im`. The `.im` silently drops a real part, so the claim that the SM FFV couplings are pure imaginary is an unchecked convention. | checked (duplicate); suspected (real part is exactly 0 for GC_3/50/51/58/59): settle with an assert | fix: one closure that asserts the real part vanishes |
| H5 | 3 | `vertex.rs` `pub fn jioxxx`/`iovxxx`, `pub use` in `helas/mod.rs` | `grep -rln "jioxxx\|iovxxx" --include=*.rs .` lists `vibegraph-lib/tests/helas_kernel_composition.rs`, an integration target, and `docs/src/guide/04-helicity-amplitudes.md:44` links the rustdoc page. Both need `pub`. | checked | leave |
| H6 | 1 | `ffv2_3`/`ffv4_3` and `ffv_chiral_weights` duplicating something in the tree | `grep -rn "fn ffv2_3\|fn ffv4_3"` gives only `vertex.rs:132,170`, both `#[cfg(test)]` and unchanged by the PR. `grep -rn '"FFV4"\|"FFV5"' src tests` finds no other chiral-weight table. | checked | not reproduced |
| H7 | 1 | history references in the diff | `git diff main...HEAD -U0 -- src \| grep -E '\bT[0-9]\b\|Phase [0-9]\|used to\|no longer\|the old\|session'` is empty | checked | not reproduced |
| H8 | 2 | M5 (`chiral_weighted`) not reached by this test | stated blind spot in the brief and in the J1 report; S5 shows `helas::` tests catch it | checked | leave |
| H9 | 1 | `research/kb/amplitudes/wavefunctions-and-propagators.md:108-115` | The page says `jioxxx` divides by complex `m^2 - i m Gamma` and that `test_eval_jioxxx` uses massless fermions and cannot tell the forms apart. Both statements are false after this PR (`vertex.rs:61-67`; the massive regime). | checked | Found (no kb edits allowed) |

Backlog check: `grep -rln "run.rs\|vertex.rs" research/kb/backlog` lists
`hygiene/comment-leftovers-after-hygiene-sprint`,
`validation/qcd-2to3-rows-unbanked` and two others; none names a line of
`test_eval_jioxxx` or `jioxxx`, so none is a finding here.
