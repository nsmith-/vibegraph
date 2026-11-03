# cms-pdf-memory log

## 2026-11-03
* **Creation**: Sprint folder created.
* Plan posted: draft PR #86 claims `pdf-members-exceed-cms-memory-slot`. Its sibling, #87 (`grid-worker-fixes`), claims the other two sprint-F items.
* The user answered D1–D4, each with the recommended option ([D1](decisions/D1-lossless-knots-first.md)): lossless knots first, up to 2× time, below 1.5 GB, a CI bytes-per-member guard.
* M-P0 is folded into M-P1 as its first step, with a stop-and-report condition, because the technique is already decided.
* Reference data in the container: refdata-13, the SM UFO at b7687064, and NNPDF23. The 43 CMS sets go to `/home/user/cms-pdfs`: only the members the list reads, archives checked against `pdfsets.json`.
* M-P1 dispatched (performance-dev, Opus) with M-P0 folded in.
* M-P1 checked (428c4d8). Coefficient and knot storage that the new layout drops made up 83% of the peak, so the stop condition did not fire. Two of the three copies of each member were redundant: `coeffs` and a second copy of the grid.
  - Re-run by the manager:
    - the default `pdf_member_bytes` test passes;
    - one interleaved pair at `-j 4` on the item's command: new 0.696 GB, 11.1 s wall; baseline 3.606 GB, 14.8 s wall;
    - the two LHE files are byte-identical, 9.26 M `<wgt>` lines.
  - Taken from the report as stated: the bit-for-bit harness found 0 of 404 M values different across 984 members, and the CMS gate gave 4 × 9.26 M weights exact.
  - Corrections to the brief: each member held six `f64` per knot, not five; the PDF readings are about 3 s of the roughly 24 s user time, not the dominant cost.
  - The bit-level oracle lives only in the scratchpad. M-V1 brings it into the tree.
* M-V1 dispatched (validation-dev, Opus).
* M-V1 checked (300674a). The manager re-ran `pdf::table_oracle` over the 880 CMS members: 413,847,296 values compared, 0 differ.
  - Taken from the report: peak RSS at most 0.701 GiB at `-j 1` and `-j 4` across five runs each, measured with its own wrapper; the CMS gate 4 × 9.26 M weights exact.
  - Every clause of `closes_when` holds, so the item's file is deleted here.
  - Found, filed by the manager: `pdf-grid-oracle-blind-to-the-seam-rule`, `fetch-downloads-do-not-retry`, `pdf-reading-builds-cubic-per-flavour`. `resolve_mom` is already filed.
  - M-HY1 dispatched.
