---
type: Session Report
title: "D1b report: integrate and generate split by data dependency"
description: "D1 revised after the user's gridpack decision: run-card fields classified by stage, GenerateSettings, artifact versions 13/14, the new generate CLI and its migration, choices restated, and a four-step PR split."
status: draft
tags: [artifact, generate, run-card, pdf, param-card, design, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-21}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
This session was design only. No Rust was written and no cargo command was
run. The decision entry in `log.md` (a08b780) is binding, and the revision
follows it. The work is in two concepts, both `status: draft`:

- **New: `research/kb/pipeline/integrate-generate-split.md`.** The principle,
  the classification of run-card fields by stage, `GenerateSettings`, the
  version steps, both command lines afterwards, the migration list, the open
  choices and the PR split.
- **Revised: `research/kb/pipeline/run-inputs-in-the-artifact.md`.** What the
  artifact records for the PDF and the param card.
  - The param card is integrate-only. `generate` rebuilds a `ParamCard` from
    the recorded moved parameters and binds the model with it. That rebind
    reproduces every external parameter exactly.
  - The checks `generate` makes are now asset checks only.
  - The CVMFS layout measured in the log replaces my assumption, including
    the `SetIndex` line with a trailing space.
  - The `PdfInput::Unrecorded` upgrade and the write-down rule are gone.

## Classification

The table is in the split concept, §1. Each `FIELD_CLASSES` row gains a stage
column. **Generate** gets 15 fields:

- `nevents`, `iseed`, `event_norm`, `use_syst`;
- `systematics_program`, `systematics_arguments`, and the five `sys_*`;
- `clusinfo`, `time_of_flight`, `boost_event`, `lhe_version`.

**Integrate** gets everything else, including the benign bookkeeping rows.

`use_syst` is the only field read at both stages. Integrate records the
resolved `alpsfact`, which `use_syst` forces to 1. `use_syst` itself goes into
`GenerateSettings`.

These rows are ambiguous and need the user's confirmation:

- `use_syst`
- `iseed`
- `clusinfo`
- `bias_*`
- the bookkeeping rows

## Restated choices

The table in the split concept, §5, has every remaining choice in one place,
each with my recommendation and the alternative. Ten are new:

- **N1.** Record the proc card as verbatim text, re-parsed at `generate`.
- **N2.** Keep the generate settings inside the artifact.
- **N3.** Accept the classification above.
- **N4.** `--seed` is the only seed, and `iseed` is recorded but unread.
- **N5.** Add a `--event-norm` flag.
- **N6.** Carry the systematics strings verbatim.
- **N7.** Raise the artifact floor at each version step, and delete the old
  readers and guards.
- **N8.** Keep `--ufo-dir` on `generate`.
- **N9.** Write the `<MGRunCard>` banner from the settings as used.
- **N10.** Move the sampling driver into `vibegraph-lib` in H1.

Of the original choices:

- C1, C4–C10 and C12 still apply unchanged.
- C2 still applies. The moved-parameter list is now also what `generate`
  binds from.
- C3 and C11 are gone, because no artifact below the floor is replayed or
  written.

## Proposed split

1. **#59 and #66 merge first.**
   - #59's test that `generate` reads the run card first becomes moot.
   - #66's `event_norm` read moves into `GenerateSettings` in H1.
2. **H1, gridpack `generate`.** It adds the stage column, `RunCard::split`,
   `GenerateSettings` and artifact version 13 (`proc_card`, an
   integrate-only `run_card`, `generate`). It raises the floor to 13 and
   brings the new `generate` CLI and the library entry point. It closes no
   item yet; see Found.
3. **H2, the PDF taken from the run card.** It adds `RunInputs` at version 14
   and raises the floor to 14. `params` is written as `[]`, and a non-empty
   list is refused. It closes `pdf-set-not-taken-from-run-card`.
4. **The param-card PR.** It fills `params` with no version bump and closes
   `cli-has-no-param-card-option`. It can run in parallel with H2 after H1.

## Commit

The commit is on `sprintd-pdf-from-run-card`, after a08b780. `git log -1`
gives the hash.

## Found

- **No backlog item covers H1.** The work is that `generate` takes only the
  artifact plus generate-time inputs, and the run card splits by stage. The
  evidence is the user's decision of 2026-10-21. An item is closed by
  `generate` refusing a proc card or `--run-card` as an argument error, and by
  a test that every `PARAM_DEFAULTS` name has a stage.
- **Item `cli-has-no-param-card-option` has an obsolete scope line.** It says
  `generate` uses the same card or refuses, which the decision supersedes.
  The `closes_when` clause "`generate` refuses a different one" no longer
  applies. Amending the item text closes this.
- **The generation driver lives in the CLI crate.** It is in
  `vibegraph-cli/src/generate.rs` (`generate_sample`,
  `generate_proton_sample`). The decision's "library generate API" therefore
  does not exist yet. N10 decides whether H1 moves the driver or a follow-up
  does.
- **Several guards become dead code once the floor reaches 13.** They are
  `SCALE_DRAW_VERSION`, `MULTIPLICITY_VERSION`, `MERGED_CHANNEL_VERSION`,
  `version_for` and the `refuse_stale_artifact_on_*` functions, together
  with their tests. H1 deletes them under N7. Under the alternative they stay
  as dead branches.
- **`pipeline/integrate-artifact.md` is stale**, as D1 already found: it
  says version 11. H1 rewrites its "What generate checks" section anyway.
