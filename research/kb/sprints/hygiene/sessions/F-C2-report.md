---
type: Session Report
title: "F-C2 report: a structural reweight guard on $ processes"
description: "reweight-forbidden-onshell-guard-is-dead closed by option (b): OnShell::Forbidden deleted, ReweightPlan::new takes a required veto list and refuses a non-empty one, ForbiddenSChannel names the ids."
status: draft
tags: [hygiene, fix, reweight, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: ed5b166, resource: "https://github.com/nsmith-/vibegraph/commit/ed5b166", title: "refuse a $ process through a required veto argument"}
---
The dev agent's report, condensed by the manager.

## Fixed

**reweight-forbidden-onshell-guard-is-dead** (`ed5b166`), by the user's option
(b):
- **`OnShell::Forbidden` is deleted** with its dead reader. The `OnShell` doc
  now says the `$` veto lives per subprocess in `crate::onshell`.
- **`ReweightPlan::new(sets, forbidden_onshell: &[i64], …)`** refuses a
  non-empty list first, as `ReweightError::ForbiddenSChannel { ids }`; the
  message names the PDG ids.
- **Callers:** both CLI `reweight_plan` callers (fixed-energy and proton)
  pass `forbidden_onshell(parsed, model)`. The CLI card-level refusal stays
  and now prints the ids. `reweight_mg_oracle` passes each row's
  `forbidden_onshell_ids`.
- **New test** `a_process_with_an_onshell_veto_is_refused`: `e+ e- > mu+ mu- $ z`
  gives `[23]` and is refused; with `&[]` the plan builds. A temporary
  `if false &&` mutation of the guard makes it fail.

## Gate (agent, at ed5b166)

- **fmt** and **clippy, both configurations:** pass.
- **`cargo test --workspace`:** 1353 passed, 16 ignored (+1 test).
- **`cargo doc`, per package:** 9 lib and 1 bin warnings, unchanged.
- **Banked:** `reweight_mg_oracle` 1 passed; `cli_reweight_proton` 2 passed.

## Found

1. **`backlog/feature/reweight-forbidden-schannel-and-as-refused.md:18`**
   cites the old variant shape and line. Close-out.
2. **This is an API break** for library callers of `ReweightPlan::new` and
   `ForbiddenSChannel`. All callers are in-tree and updated, and no book page
   names them.

## Brief corrections

- **The error variant changed shape** to carry the ids, which the brief did
  not say.
- **A second CLI caller** (the proton path, ~:1785) was not listed.

## Manager check (2026-10-10)

- **Commit and trailer:** one commit, `Assisted-by` only; 5 files.
- **Deletion:** no `OnShell::Forbidden` remains.
- **Gates:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  exits 0. `cargo test -p vibegraph-lib --lib reweight::` passes 33.
  `cargo test -p vibegraph --bins` passes 95.
