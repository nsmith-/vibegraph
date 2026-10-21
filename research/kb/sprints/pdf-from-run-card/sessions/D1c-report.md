---
type: Session Report
title: "D1c report: the user's answers on the run-input choices folded in"
description: "All twelve C-choices settled in both concepts: generate-read PDF files recorded in the LHE header, every built-in pdlabel refused with lhapdf/247000 as the default card, LHAPDF_DATA_PATH (with LHAPATH) replaces VIBEGRAPH_PDF_DIR."
status: draft
tags: [pdf, lhapdf, run-card, artifact, design, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
This session was design only: no Rust was written and no cargo command was
run. It folds the answers in the log entries f923548 and 2edb34b into the two
concepts:

- `research/kb/pipeline/run-inputs-in-the-artifact.md`
- `research/kb/pipeline/integrate-generate-split.md`

Both concepts now mark C1–C12 as settled.

## What changed

- **C1.** The integrate artifact keeps the per-file SHA-256s of the files
  `integrate` read. Files that only `generate` reads are recorded by
  `generate`. Where and how is under "Where the C1 checksums live" below
  (run-inputs §3a).
- **C4.** Every built-in `pdlabel` is refused wherever a parton density is
  read, `nn23lo1` included. The refusal message tells the user to set
  `pdlabel = lhapdf` and an `lhaid`, and adds the 247000 hint for `nn23lo1`.
  Fixed-energy and decay cards are not refused, since the label is inert
  there (`for_decay` sets `lpp = 0`).
  - The default card becomes `pdlabel = lhapdf` and `lhaid = 247000`. The
    departure from MadGraph is documented in three places: the comment on the
    `PARAM_DEFAULTS` rows, `docs/src/cli/overview.md:55`, and
    `run-card/run-card-parser-and-defaults.md`.
  - `pdf_label_alpha_s` and its table become unreachable and are removed. On
    proton beams, αs always comes from the PDF set's own grid.
  - `PDFSUP` is unchanged.
- **C7.** `LHAPDF_DATA_PATH` replaces `VIBEGRAPH_PDF_DIR`, read as a
  `:`-separated list. `LHAPATH` is honoured only when `LHAPDF_DATA_PATH` is
  unset, which is what LHAPDF 6.5.3 does (`src/Paths.cc` lines 20–28). The
  reason: a user's working LHAPDF setup then resolves the same sets here as it
  does in their MadGraph.
  - LHAPDF's install-prefix entry and its trailing-`::` rule are not copied:
    vibegraph has no LHAPDF installation, and empty entries are skipped.
  - Run-inputs §7 lists every place `VIBEGRAPH_PDF_DIR` appears: code, tests,
    `scripts/acceptance.sh:183`, `scripts/demo_pp.tape:29`, docs and concepts.
    CI never sets the variable.
- **C8, C9.** The `validation/pdf` fallback is removed, and a checksum
  mismatch is refused, never searched past.
- **C2, C3, C5, C6, C10–C12.** These are recorded as settled. C3 is carried
  out by N7, and C11 is satisfied because every artifact is written at the
  current version.
- **Tests.** New tests cover the label refusals and their acceptance on
  fixed-energy beams, the environment-variable parsing, the absence of a
  `validation/pdf` fallback, the default card's αs and banner, and the
  `<vibegraphInputs>` block.
- **PR split.**
  - C7 and C8 move into H1, because H1 already rewrites every `generate`
    call; this avoids migrating the tests twice.
  - The C4 work and the central entry of the header block go into H2.
  - The systematics entries go into whichever PR first evaluates PDF
    systematics.

## Where the C1 checksums live

They go in a `<vibegraphInputs>` block in the event file's header, beside
`<MGRunCard>`:

- one `<pdf>` entry per set and member read: `use` (central or systematics),
  `set`, `lhaid`, `member`, `tier`, and a `<file name sha256>` element per
  file;
- for a set fetched at generate, its `archive_sha256`;
- the SHA-256 of the grid file `generate` read.

How each entry is checked:

- **Central entry.** It repeats the digests `generate` has just verified
  against the artifact.
- **Systematics entries.** They are recorded, not compared, since nothing
  earlier holds a reference digest for them.
  - A pinned set fetched at generate is verified against its pin before
    extraction, so its members are verified transitively.
  - If the user refuses consent for a systematics set, the run is refused
    before any event is written.
- **The `tier` field** names the tier, never a path.

There is no separate record file. Whether to add one is the new choice N11.
No systematics are evaluated today, so until they are, the block holds only
the central entry and the grid digest.

## The `nn23lo1` migration list

The full list is in the split concept, §4. In summary:

- **Code:**
  - `runcard.rs:615-618` (defaults) and `:289` (doc);
  - `coupling/alphas.rs`: L279-293 and L241, plus the tests at L564/L649,
    which become refusal tests;
  - `runcard/matching.rs:264`;
  - `cache/pinned.rs:32`.
- **Tests whose proton cards omit `pdlabel`.** Their αs source moves from the
  `nn23lo1` table to the 247000 grid; the densities do not change.
  - `vibegraph-cli/tests/`: `cli_generate_proton`, `cli_integrate`,
    `cli_reweight_proton`, `cli_decay_chain_events`, `cli_hard_errors`,
    `validate_samples_proton`;
  - `vibegraph-lib/tests/`: `validate_hadronic`, `decay_chain_ladder`,
    `diagram_channel`, `validate_alphas`.

  Any number pinned from a default-card proton QCD run moves. I have not
  established which pins those are; H2 lists each one with its old and new
  value. `cli_decay.rs:475` needs only a comment change.
- **Validation cards:**
  - `onshell_uu_run_card.dat` is the only proton `nn23lo1` card a test reads
    (the ungated `uu_tt` row). It gets re-carded, with a banking request to
    regenerate the MadGraph row; until then, the test overrides the two lines
    in the copy it writes.
  - `onshell_ee`, `onshell_t_bev`, `decay_h4l_cuts` and `decay_tbev_cuts` are
    inert and stay as they are.
  - The `manifest.toml:629` rationale is reworded.
- **Docs:** `docs/src/cli/overview.md:55`,
  `docs/src/guide/10-hadronic.md:37`,
  `docs/src/guide/11-unweighting-events.md:145`.
- **Concepts:** `pinned-pdf-set`, `alpha-s-sources`, `field-classification`,
  `run-card-parser-and-defaults`, `lhef-record-conventions`.

## The log's reasoning about the gates is wrong

The log says no gate's comparison changes because the banked references used
MadGraph's internal `nn23lo1` grid and the gates already compare LHAPDF 247000
against them. The conclusion holds, but the premise is wrong. Every banked
proton σ and sample row was carded at `pdlabel = lhapdf`, `lhaid = 247000` on
the MadGraph side: `manifest.toml:151` records that the four `nn23lo1` rows
were re-carded at cut 5, and `madgraph-oracle-pinning.md` says the same.
vibegraph reads those same cards. So both sides already use LHAPDF 247000 and
the grid's αs. The only proton comparison whose MadGraph side read `nn23lo1`
is the ungated `uu_tt` row. The split concept §4 states the corrected
reasoning.

## Still open

N1–N10 from D1b, unchanged, with my recommendations as the working
assumption. Two choices are new in this pass:

- **N11**: record generate-read PDF files in the header block only (my
  recommendation), or also in a sidecar file.
- **N12**: a set `$VIBEGRAPH_PDF_DIR` draws one warning naming
  `LHAPDF_DATA_PATH` (my recommendation). The alternatives are to ignore it
  silently or to refuse.

## Commit

The commit is on `sprintd-pdf-from-run-card`, after 2edb34b. `git log -1`
gives the hash.

## Found

- **Some test pins will move, and I have not identified which.** Proton test
  cards that omit `pdlabel` switch their αs source from the `nn23lo1` table
  (0.130, two-loop) to the 247000 grid. Any σ or weight pinned from such a
  card on a QCD process changes. Running the hermetic and banked suites in H2
  and listing each moved pin closes this.
- **The `uu_tt` MadGraph row needs rebanking.** It was banked with
  MadGraph's internal `nn23lo1` (`onshell_uu_run_card.dat`), and vibegraph
  will refuse that card. Re-carding the file to `lhapdf`/247000 and
  regenerating the row through `gen_onshell_veto.sh` and
  `bank_seeded_sigma.sh` closes this. It is a banking request.
- **The 2026-10-21 log entry misstates the references.** It says the banked
  references used MadGraph's internal `nn23lo1` grid; they used LHAPDF 247000
  (see above). Correcting the log entry closes this.
