---
type: Session Brief
title: "K-H2: hygiene pass on the diagram, colour and grouping"
description: "Review sprint K's merged the diagram, colour and grouping PRs (#146, #147, #150, #153) for maintainability, test non-vacuity, visibility and reusable abstractions; write findings first, then fix them with mutation evidence, σ bit for bit."
status: draft
agent: hygiene-dev (Opus)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Hygiene pass of [sprint K](../sprint.md) (plan step 4), on branch `sprintk-hyg-diagrams` off
main 92dee0d7. It covers these merged PRs. Read each squash commit's diff
(`git show <sha>`) and its session report under `sessions/`:

- #146 `82ed311c`: the coloured-scalar seagull colour structure (K-S1).
- #147 `15de3a8f`: split subprocess groups by outgoing masses (K-X1).
- #150 `0340a897`: stitch a decay chain once where identical decays meet (K-D1).
- #153 `74596ba3`: MLM matched reweighting with no panic, vanishing groups (K-M1).

No sibling pass runs now; use `CARGO_BUILD_JOBS=3` (one core stays for the sprint
session). Main already carries the other three hygiene passes: #157 (K-H1)
moved the automatic WEIGHTED search out of `generate_undecayed` into
`search_weighted` and made `diagrams/chain.rs` and `diagrams/reduction.rs` call
`ufo::weighted::order_weight`; #159 (K-H3) added `refuse_emitted_fault` in
`generate.rs`. Review #146/#147/#150/#153's code as it stands on main, and don't
re-open what those passes settled (their reports are under `sessions/`).
Stay inside the files these PRs touched, plus their direct callers and tests.
Do not edit code another cluster owns (model loading: #141/#144/#145;
diagrams/colour/grouping: #146/#147/#150/#153; run card/CLI/batch:
#143/#152/#156; integration: #142/#155) except where a finding needs it, and
then say so.

## Scope

1. **Findings first.** Before editing anything, write the findings section of
   your report and commit it. For each finding give the file:line, what is
   wrong, the class (dead or duplicated code, a vacuous or tautological test,
   a public item that should be private, a hand-kept list that should be
   derived, a missing reusable abstraction, a comment that violates AGENTS.md's
   comment rules, a panic or unwrap on a fallible path), and the fix you
   propose. Rank them, and mark which ones you will fix here.
2. **Test non-vacuity.** For every test these PRs added, make one mutation of
   the code it guards that it should catch, run it, and record whether it
   failed. Any test that survives its mutation is a finding: strengthen it and
   show it now fails under the same mutation.
3. **Fix** the findings you marked. Behaviour is unchanged: σ and grid sha256
   are bit for bit against main on `pp_to_llj_mlm`, `pp_to_ttx_0j1j_mlm` and `dy13_default`
   (`integrate --fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`, main
   built in a separate target dir and deleted afterwards).
4. **Physics or performance findings** (a wrong convention, a missing oracle,
   a slow path) are not yours to fix: list them under Found.

No seed sweeps, MadGraph runs or local `validate`: CI's banked job is the
banked gate. The gates are fmt, clippy `-D warnings`, `cargo test --workspace`
and kb-lint.
