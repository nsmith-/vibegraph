---
type: Session Brief
title: "F-C2: a structural reweight guard on $ processes"
description: "Close reweight-forbidden-onshell-guard-is-dead by option (b): delete the never-assigned OnShell::Forbidden and give ReweightPlan::new a required forbidden_onshell argument that refuses a vetoed process."
status: draft
agent: feature-dev (Opus)
depends_on: [F-C]
closes: [reweight-forbidden-onshell-guard-is-dead]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
---
Follow the [fix protocol](fix-protocol.md). Read the item
(`python3 scripts/kb.py backlog --item reweight-forbidden-onshell-guard-is-dead`)
and [F-C's report](F-C-report.md), "Stopped", for the analysis.

**Decision (user, 2026-10-10): option (b).**

1. **Delete `OnShell::Forbidden` and its doc** (`diagrams/diagram.rs`), and
   every match arm that handles it.
2. **`ReweightPlan::new` takes a required argument** carrying the `$`-vetoed
   s-channel ids for the process, a `&[i64]` or a small named type. It
   refuses a non-empty list with an error that names the veto. The CLI's
   card-level refusal (`vibegraph-cli/src/generate.rs` ~:1056) stays, and the
   CLI passes `forbidden_onshell_ids(card)`.
3. **Update every caller:** the CLI call (~:1086) and its test (~:2335),
   `tests/reweight_mg_oracle.rs`, and any other `ReweightPlan::new` site.
4. **Tests:**
   - a library `ReweightPlan` built with a non-empty veto list is refused,
     with the error matched by variant;
   - an empty list builds as before.

   Show that the refusal test fails when the check is removed.

**Gate:** the protocol's gate, plus the banked `reweight_mg_oracle` and the
CLI's `cli_reweight_proton` (`-p vibegraph`).
