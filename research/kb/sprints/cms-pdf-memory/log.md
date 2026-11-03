# cms-pdf-memory log

## 2026-11-03
* **Creation**: Sprint folder created.
* Plan posted: draft PR #86 claims `pdf-members-exceed-cms-memory-slot`. Its sibling, #87 (`grid-worker-fixes`), claims the other two sprint-F items.
* The user answered D1–D4, each with the recommended option ([D1](decisions/D1-lossless-knots-first.md)): lossless knots first, up to 2× time, below 1.5 GB, a CI bytes-per-member guard.
* M-P0 is folded into M-P1 as its first step, with a stop-and-report condition, because the technique is already decided.
* Reference data in the container: refdata-13, the SM UFO at b7687064, and NNPDF23. The 43 CMS sets go to `/home/user/cms-pdfs`: only the members the list reads, archives checked against `pdfsets.json`.
