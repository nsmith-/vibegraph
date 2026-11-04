# cms-gridpack-coverage log

## 2026-11-04
* **Creation**: Sprint folder created by the sprint-H manager session. Items 3 and 4 filed as `xpt-nonpositive-refused` and `pt-min-pdg-refused`.
* Reference data in the container: refdata-13, the PDF sets (`validation/pdf/fetch.sh`), and the `mg5amcnlo` submodule at b7687064.
* Plan posted (sprint.md). Questions to the user on items 2 and 5.
* The user answered: D1 (item 2) merge rounding-level pairs by raising GROUP_REL_TOL to about 1e-9 and split physical ones; D2 (item 5) decide after MadGraph is measured ([D1](decisions/D1-near-degenerate-merge-rounding.md)).
* Worktrees `wt-{cuts,massive-incoming,decay-duplicate,g0}` were created from `sprinth-plan` (9ba3d96) with refdata-13, NNPDF23 and NNPDF31, and the submodule's content. pixi is not installed here, so the gates run as plain cargo and `validation/validate.sh`.
* Dispatched: H-C1 (feature-dev, items 3 and 4), H-M1 (feature-dev, item 6), H-D1 (validation-dev, item 5, measurement only per D2) and H-G0 (validation-dev, the colour-basis diagnosis, read-only). H-N1 (item 2) waits for `colour-split-groups`, because both edit the degenerate-groups check in `derive_flavor_groups`.
* The user: MadGraph runs that need no banking happen in this container, not on the M3. The pinned submodule's `bin/mg5_aMC` runs here (tested with an `output` of `p p > z j j, z > j j`), and gfortran is installed. LHAPDF is not, so MadGraph uses `nn23lo1`. All four sessions were told. Only references committed to the banked bundle stay M3 requests.
