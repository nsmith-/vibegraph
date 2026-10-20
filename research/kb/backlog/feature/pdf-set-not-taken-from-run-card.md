---
type: Backlog Item
title: The PDF set comes from --pdf-set, not the run card's lhaid
description: "integrate loads --pdf-set (default NNPDF23) member 0 whatever lhaid and pdlabel say, so every CMS card silently runs the wrong PDF; the card should choose the set and member, with no PDF flags."
area: feature
state: open
priority: high
closes_when: "integrate and generate take no PDF option; the run card picks the set and member (pdlabel = lhapdf: the set whose SetIndex range holds lhaid, member lhaid − SetIndex; a built-in pdlabel: its LHAPDF equivalent, or a refusal); lookup goes $LHAPDF_DATA_PATH, then CVMFS lhapdfsets, then the cache, then an HTTPS download with consent; the artifact records lhaid, set and member, and generate reads them from it; tests cover a non-zero member, a lhaid in no known set, each lookup tier and the default card."
blocked_by: []
opened: 2026-10-20
tags: [pdf, lhapdf, run-card, cli, cvmfs, network-consent, cms, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, silent physics change 1"}
---
**Measured.** `p p > e+ e-` gives 690.095053 ± 11.511817 pb with no `lhaid`,
with `325300 = lhaid` and with `230000 = lhaid`; every run logs NNPDF23.
Every LO CMS card names another set: 325300 (41), 325500 (17), 306000 (6) and
325100 (1). On card 075 the PDF alone is a factor of 1.5
([survey](../../validation/cms-gridpack-survey-2026-10.md)).

**Today.**
- The set is `--pdf-set` (`integrate.rs:349`, default
  `DEFAULT_PDF_SET`), and the member is the constant `PDF_MEMBER = 0`
  (`integrate.rs:66`).
- The run card sets only αs. A built-in `pdlabel` takes αs(MZ) from a table
  (`coupling/alphas.rs:279`), and `pdlabel = lhapdf` reads the loaded set's αs
  grid. `lhaid` appears only in an error message.
- A built-in `pdlabel` with a non-default `--pdf-set` therefore mixes one set's
  αs with another set's densities.
- The `.info` `SetIndex` is not parsed.
- Downloads go over HTTPS from `lhapdfsets.web.cern.ch`, but only for the one
  SHA-256-pinned set ([pdf-set distribution](../../tooling/pdf-set-distribution.md)).

**Decision (user, 2026-10-20).** The run card is the input:
- `--pdf-set` and `--pdf-dir` are removed from `integrate` and `generate`.
- An HTTPS download is trusted, so any LHAPDF set may be fetched with consent,
  not only pinned ones. A set that is pinned keeps its digest check.
- A CVMFS LHAPDF location (`/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current`)
  is searched before any download.
- `lhaid` resolves to a set name through LHAPDF's `pdfsets.index`, read from
  the same tiers.

The [network-consent](../../tooling/network-consent.md) and
[asset-resolution](../../tooling/asset-resolution.md) concepts change with it.

**Open questions for the implementer:**
- Whether the artifact also records a digest of the loaded set files, so a
  CVMFS set that changes under a saved grid is caught.
- Which built-in `pdlabel`s have an exact LHAPDF equivalent. `nn23lo1` is
  NNPDF23_lo_as_0130_qed, 247000; refuse the rest until each is checked.
- Tests and `scripts/acceptance.sh` pass `--pdf-dir` today, and will need an
  environment variable instead.
