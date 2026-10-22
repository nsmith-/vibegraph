---
type: Session Report
title: "HY1 report: hygiene review of the ckm_no_b_mass restriction and its banking plumbing"
description: "Review of F1 (9dc76b4) and V1 (768a207): shared banked-card test helpers, named thresholds, a MadGraph anchor that can see its exclusion widen, an empty VG_MG5_ROOT treated as unset."
status: draft
tags: [hygiene, model, restriction, sm, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-20}]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-20}
sources:
  - {id: 481c979, resource: "https://github.com/nsmith-/vibegraph/commit/481c979", title: "hygiene(ckm_no_b_mass): shared banked-card helpers, named thresholds, a tighter MadGraph anchor, empty VG_MG5_ROOT"}
---
Filed by the manager from the session's final message (the session returned
its report rather than committing it). Scope: `git diff origin/main...HEAD`
(F1 `9dc76b4`, V1 `768a207`). Worktree `/home/user/wt/G`; logs `ckmG-HY1-*`
in the session scratchpad.

## Findings (written before any edit)

| id | pt | site | evidence | disposition |
|---|---|---|---|---|
| H1 | 1 | `validation/madgraph/gen_couplings.py` | `os.environ.get("VG_MG5_ROOT", default)` returns `''` for a set-but-empty variable; `mg5_pinned.sh` uses `${VG_MG5_ROOT:-default}` | **fixed** 481c979 (`get(...) or default`; checked before/after by AST for unset, empty, a path) |
| H2 | 1/4 | `tests/sm_ckm_no_b_mass.rs`, `tests/interned_sm_rows.rs` | the same param-card join and parse, and the `ParamNature::External` destructure, copied | **fixed**: `common::banked_param_card`, `common::lha_slot` |
| H3 | 1 | `sm_ckm_no_b_mass.rs` | bare `1e-9`, `1e-10`, `1e-13` literals | **fixed**: `MERGE_PAIR_REL`, `ZERO_RULE_BAND_TOP`, `PYTHON_REL_TOL`, each justified once |
| H4 | 2 | `madgraph_default_couplings_hold_outside_the_b_and_ckm_sectors` | its last assert passed with the exclusion widened to every coupling below `GC_5` | **fixed**: asserts ≥ 25 tables and ≥ 49 compared couplings and a non-empty exclusion |
| H5 | 1 | `ufo/identity.rs` | comment said "all nine variants" | **fixed** |
| H6 | 1/4 | `src/bin/gen_sm_blob.rs` | printed a literal "+ 9 restrict cards" | **fixed**: counts `ALL` filtered by `ships_with_madgraph` |
| H7 | 1 | doc headers in `validation-report/src/manifest.rs`, `gen_sm_blob.rs` | overflowing lines | **fixed** (reflow) |
| H8 | 2 | `manifest_rows_naming_the_interned_sm_resolve` | runs on an empty set until banking adds a row | leave; it should assert a non-zero count in the banking commit |
| H9 | 3/4 | `common::load_row_model` beside `model_for_row` | the cached key path delegates to the uncached one; only synthetic rows call the latter | leave |
| H10 | 4 | `SMRestrict::ships_with_madgraph` | one predicate; every list goes through `ALL` | leave; a second vendored card would want an exhaustive match |
| H11 | cross | param-card join in six other test files | pre-existing | Found |
| H12 | cross | `gen_reweight_oracle.py` | the same empty-variable defect as H1 | Found |
| H13–H15 | – | history/plan references, backlog paths, new `pub` items | none, or each justified | not findings |

## Mutations

Driver `ckmG-HY1-mutate.sh`, files restored by `cp` then `touch`. The H4
mutation (anchor skips every coupling named below `"GC_5"`): the old test
passes 9/9 (blind), the new one fails at its `compared.len() >= 49` assert;
9 pass after restore. The refactors (H2, H3) are output-identical: the
`--nocapture` counts match V1's gate line for line.

## Gate

`cargo fmt --all --check` 0; `cargo clippy --workspace --all-targets -- -D warnings` 0;
the same with `--all-features` 0; `cargo test --workspace` 0 (lib 1009, cli 106,
`interned_sm_rows` 4, `sm_ckm_no_b_mass` 9, unchanged); `cargo doc -p vibegraph-lib`
8 pre-existing warnings, none in touched files; `python3 -m py_compile gen_couplings.py`;
`python3 scripts/kb.py lint` 0 errors. The manager re-ran `sm_ckm_no_b_mass` (9) and
`interned_sm_rows` (4) and fmt from the committed tree.

## Found

1. The banked param-card join is copied in six other test files
   (`coupling_oracle`, `validate_scale_couplings`, `standalone_jamps`,
   `polarization_frame`, `validate_hadronic`, `amplitude_oracle`), and
   `PYTHON_REL_TOL` in two; `common` now has the helper.
2. `gen_reweight_oracle.py` reads an empty `VG_MG5_ROOT` as a path.
3. "nine restrict cards" in `model/restriction-semantics.md` and
   `model/ufo-parsing.md` (fixed by the manager in this PR).
4. H8's sweep should assert a non-zero row count once banking adds the rows.
