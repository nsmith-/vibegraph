# amplitude-signs log

## 2026-10-15
* **Creation**: Sprint folder created.
* **Plan**: seven draft PRs, #29–#35, claim the ten items (see `sprint.md`).
* **Amendment** (planner-approved): PR G extended from `g g > t t~ g` to every
  `pp_to_ttx_0j1j_mlm` `@1` subprocess (`gu_to_ttxu`, `gux_to_ttxux`,
  `uux_to_ttxg`, and `ug_to_ttxu` as asked), each with a colour falsifier and a
  MadEvent-width diagnostic, to close the σ sprint's `@1` |M|² hypothesis.
* **Environment**: MadGraph runs only through `mg5_pinned.sh` (pinned 3.7.1,
  `madgraph-oracle-pinning`); the packaged 3.5.7 supplies Python and gfortran.
  The cloud disk filled once (a 15 GB debug target); cargo runs since use
  `CARGO_INCREMENTAL=0` and no debuginfo.
* **G1 checked**: `standalone_jamps` re-run by the manager on 556ec90, 15 passed
  with the reported numbers; CI green on cfd44f2 including `banked`.
  `gg-ttxg-no-amplitude-gate` closed.
* **G-H checked**: `standalone_jamps` re-run on df93d59, 16 passed. Found 1–2
  (stale row lists) fixed by the manager; Found 3–4 and G1 Found 1 filed as
  `standalone-colour-falsifiers-not-permanent`,
  `standalone-table-without-test-is-never-run` and
  `standalone-spacelike-width-convention-unbanked`.
