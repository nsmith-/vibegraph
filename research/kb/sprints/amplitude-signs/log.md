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
* **Approval** (human:nsmith-, relayed by the planner): sign fixes verified
  against MadGraph per-diagram dumps, plus controls outside their fit, land
  without a separate design approval. In the user's words, "I accept skipping
  design approval for sign fixes when they align with madgraph". Convention
  changes still need sign-off.
* **Decision** (human:nsmith-): W2 and F2 run as one derivation session (B′).
  It states the role-sign convention root-independently and is accepted on a
  hermetic relabelling-invariance test plus every MadGraph per-diagram table. If
  it does not close in one session, it stops and the three fitted rules (A′)
  come back for approval.
* **Decision** (human:nsmith-): C2 lands C1's two derived fixes (the Identity
  crossed-line −1, the derivative all-scalar sign) before B′; B′ builds on them
  and keeps C's rows green.
* **Approval** (human:nsmith-): `ee_to_ee_4f`'s per-pairing diagram count is
  registered as a counting convention, worded as Q1 proposed. The emphasis is
  that the standalone gate requires the declared diagram groups to partition
  vibegraph's diagrams into exactly MadGraph's graphs. A backlog item for
  MadGraph-style grouping of a diagram's contributions is to be filed.
* **Approval** (human:nsmith-): `wpwmz-cw-ow-five-vector-residual`'s
  `closes_when` is restated (MadGraph's seven-digit `GC_303`: the twelve graphs'
  modulus exempted by name, and the MadEvent amplitudes cell flipped to gate
  with the cause attributed). It is registered as a deviation, as for `ee_to_ee_4f`.
* **Process**: Q1 stamped its own report `verified`; the stamp is the manager's,
  and later briefs say so. The manager's re-run confirmed the report.
* **Not filed**: J-H's duplicated `ExtLegInfo` literals outside the change, too
  small for an item.
* **Decision** (human:nsmith-): MadGraph's seven-digit UFO literals (`GC_303`,
  and `GC_299` by the same mechanism) are classified under the defect policy as
  an approximation this crate improves on at negligible cost: a registered
  deviation, named in the affected rows and catalogued (PR #33).
* **Answer** to the user on the per-pairing split: it is made by vibegraph in
  `ufo/topo.rs::build_feyngraph_model` (one feyngraph vertex per pairing), not
  by feyngraph. feyngraph's one-`spin_map`-per-vertex API made it the easy route,
  and the evaluator relies on one set of fermion lines per diagram (one Fermi
  sign, one wavefunction typing, one rooting). Filed as
  `madgraph-style-diagram-grouping` (PR #35).
