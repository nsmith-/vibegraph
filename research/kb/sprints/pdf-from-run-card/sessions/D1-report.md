---
type: Session Report
title: "D1 report: the run inputs the integrate artifact records"
description: "Design for artifact version 13 (PDF lhaid, set, member, per-file SHA-256; moved param-card values), generate's checks, lhaid lookup over four tiers; twelve open choices for the user."
status: draft
tags: [pdf, lhapdf, param-card, artifact, design, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
---
Design only; no Rust written, no cargo run. The concept is
[run inputs in the artifact](../../../pipeline/run-inputs-in-the-artifact.md)
(`research/kb/pipeline/run-inputs-in-the-artifact.md`, `type: Design`,
`status: draft`). It is under `pipeline/`, beside the integrate-artifact
concept, because the artifact contract is what both PRs share; the lookup
half amends two `tooling/` concepts, which it links. It runs to about 290
lines against the brief's 200-line aim, mostly the built-in `pdlabel` table
and the choices table.

## Open choices for the user

These are C1 to C12 in the concept. The recommended option comes first.

1. **PDF checksum.** Per-file SHA-256 of `<set>.info` and the member `.dat`
   that was read. The alternatives are a digest of the whole set directory, or
   the archive digest, which does not exist on CVMFS.
2. **Param-card record.** Record only the externals that moved from the
   restriction default. With no card, an empty card or a card equal to the
   defaults, the list is `[]`, and the artifacts are byte-identical. The
   alternatives are every external's value, or the card file's digest.
3. **Old hadronic artifacts (version 12 and earlier).** Refuse them. The
   alternative is to replay the recorded set without a checksum.
4. **Built-in `pdlabel`s.** Admit only `nn23lo1`. The alternative also admits
   `nn23lo`, on the evidence that the names match.
5. **Pinned sets.** The pin also carries `set_index` and `num_members`, so the
   default lhaid needs no index fetch. The alternative is to always go through
   an index.
6. **Index across tiers.** Use the first index found, and fall through to the
   next one when the lhaid is in no known set. The alternative is the first
   index only.
7. **`$VIBEGRAPH_PDF_DIR`.** Keep it as tier 0, and tests use it. The
   alternative is to retire it, and tests set `$LHAPDF_DATA_PATH`.
8. **`validation/pdf` dev fallback.** Remove it. The alternative is to keep it.
9. **Checksum mismatch in `generate`.** Refuse at the first tier that holds
   the set. The alternative is to search later tiers for a copy that matches.
10. **Prompt for an unpinned set.** It says the size and SHA-256 are unknown.
    The alternative is a HEAD request first, to get the size.
11. **Write-down.** No-PDF, default-card artifacts stay at version 12 or
    earlier, following the "write the oldest version" rule. The alternative is
    to always write version 13.
12. **Testing the download tier.** Test it at library level only, through an
    injected `MapFetch`. The alternative adds a CLI test through a base-URL
    override and a local server.

## Contradictions with the items, the brief and the code

- **`nn23lo1` is not an exact equivalent of 247000.** Item
  `pdf-set-not-taken-from-run-card` says "`nn23lo1` is
  NNPDF23_lo_as_0130_qed, 247000". It is the same fit but not the same
  numbers. MadGraph's built-in label reads its own
  `NNPDF23_lo_as_0130_qed_mem0.grid` through `NNPDFDriver`. The kb concept
  `scales-pdf/pinned-pdf-set.md` records a 9.8% shift in a `b b̄` σ between
  the two. The design still admits `nn23lo1`, because refusing it would refuse
  every proton run without a run card. It logs a note when it does. No other
  label has an exact equivalent either.
- **MadGraph only runs member 0.** At LO, MadGraph passes `lhaid` to
  `copy_lhapdf_set`, which raises unless the `lhaid` is a key of
  `pdfsets.index`, that is, a SetIndex (`madevent_interface.py` L6114–6116,
  `common_run_interface.py` L4566–4572). The item's rule,
  member = lhaid − SetIndex, therefore extends MadGraph. A non-zero member can
  only be tested against a synthetic set.
- **MadGraph's `get_pdf_id` table is stale.** In today's `pdfsets.index`,
  10000 is `CoSMAO22Proton`, not cteq6m, and 10041 is absent. Adopting the
  table wholesale would load the wrong set.
- **Item `cli-has-no-param-card-option` cites the wrong line.** It points to
  `ufo/mod.rs:562` for `from_model_card`, which is at line 540; line 562 is
  in `from_model`.
- **The brief's claim about `scripts/acceptance.sh` is wrong.** The brief
  says the script uses `--pdf-dir`. It passes no PDF flag: it unsets
  `VIBEGRAPH_PDF_DIR` and relies on the cache and the download. It still has
  to change, to also unset `LHAPDF_DATA_PATH`.
- **The artifact already records the old run's PDF.** The brief asks whether
  the PDF an old run loaded (`--pdf-set` member 0) is recorded. It is: every
  version from 3 to 12 has `pdf_set` and `pdf_member`. The upgrade carries
  them as `PdfInput::Unrecorded`.

## Commit

The commit is on `sprintd-pdf-from-run-card`; `git log -1` gives the hash.

## Found

- **`pipeline/integrate-artifact.md` is stale.** It says
  `FORMAT_VERSION = 11` and that files are version 9, 10 or 11. The code is
  at 12, with `grid_rule`. Rewriting the concept's version text to match the
  code closes this.
- **Large sets may not download.** `HttpFetch` caps a response at 256 MiB
  and holds the whole archive in memory. Every set is downloaded whole,
  although a run reads one member. An unpinned CMS set (325300, 325500,
  306000, 325100) may hit the cap. Measuring those archives' sizes, and
  raising the cap or streaming to disk, closes this.
- **The CVMFS layout is unverified.** The design assumes that
  `/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current/` holds
  `pdfsets.index` and `<set>/` directories. Nobody could check that from this
  container. A listing taken on a CVMFS host closes this.
- **`ParamCard` silently drops entries the model does not have.** Entries
  that name no external parameter of the model are ignored without a word. A
  CMS card with a misspelled block would run at the defaults. Deciding
  between a warning and a refusal, with a test, closes this.
- **`RunCard::pdfsup()` writes ids that now name other sets.** It writes
  10000 and 10041 for `cteq6_m` and `cteq6_l`. Once those labels are refused
  on proton beams, this survives only on fixed-energy cards, where nothing
  reads a density. It is low priority. Deciding whether `PDFSUP` should keep
  copying MadGraph or write 0 closes it.
