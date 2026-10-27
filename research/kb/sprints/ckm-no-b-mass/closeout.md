---
type: Sprint Record
title: "The built-in SM gains the ckm_no_b_mass restriction"
description: "The CMS card is vendored byte-exact and loads as sm-ckm_no_b_mass; its first MadGraph row exposed a CKM conjugation bug (η → −η on charged vector currents), fixed and gated with G = +i."
closed: 2026-10-22
status: draft
tags: [model, restriction, ckm, sm, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2 report"}
  - {id: v3, resource: "sessions/V3-report.md", title: "V3 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/62", title: "nsmith-/vibegraph#62"}
---
One PR ([nsmith-/vibegraph#62](https://github.com/nsmith-/vibegraph/pull/62),
merged 2026-10-22) closing `sm-lacks-ckm-no-b-mass-restriction`. Sessions:
[F1](sessions/F1-report.md) (feature-dev, Opus), [V1](sessions/V1-report.md)
(validation-dev, Opus), [HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet),
[V2](sessions/V2-report.md) (validation-dev, Sonnet) and
[V3](sessions/V3-report.md) (validation-dev, Opus). The M3 banked the two rows.

## What was established

- **The card.** MadGraph 3.7.1 does not ship `restrict_ckm_no_b_mass.dat`; CMS
  genproductions does, and all 88 surveyed gridpacks carry one copy (sha256
  `ad6e3eaf…1b05`). It differs from `restrict_ckm.dat` in `MB = 0`, `ymb = 0`
  and `DECAY 15 = 2.27e-12`, so `WTau` survives restriction (the parameter is
  `WTau`, not the brief's `WTA`). It is vendored byte-exact as
  `SMRestrict::CkmNoBMass`; `sm_interned_blob` asserts MadGraph's tree does not
  carry a card of that name. *Manager-verified:* the committed card's sha256.
  The user decided it needs no licence of its own (`licensing.md`).
- **Couplings before banking.** No coupling table existed at `ckm` or
  `no_b_mass` (all 25 interned-SM tables are at `restrict_default`), so F1 and V1
  pinned the card by composition with this crate's own restrictions and against
  MadGraph only outside the b and CKM sectors. Restriction diffs and
  one-parameter probes both missed joint dependencies (GC_13/14/28/29,
  `yb·CKM`); NaN poisoning does not.
- **The interaction count is unitary gauge.** MadGraph lists 67 interactions at
  this card; this crate has 133 vertices = 67 + 24 with a ghost + 42 with a
  Goldstone and no ghost (50 carry a Goldstone, 8 of them a ghost too). The
  count test goes through the predicate diagram enumeration filters on
  (`Vertex::is_physical`). A Feynman-gauge MadGraph build would give 109.
  *Manager-verified:* `import_ufo.py:1241-1255` read; the suites re-run.
- **A CKM conjugation bug, found by the new row and fixed.** The bound `u b~ W`
  vertex carried `GC_43 ∝ CKM1x3` where MadGraph's carries
  `GC_102 ∝ conj(CKM1x3)`: the amplitude constant came out
  `G = i·V_ub/conj(V_ub) = 0.6733 − 0.7394i` (fit and arithmetic agree to 3e-15).
  That is the model at η → −η, which `|M|²` sees where CKM pairings interfere:
  3.7e-2 on `u c~ > b b~ d d~` and 5.3e-4 on its `QCD=0` form, at five RAMBO
  points. A charged vector current now carries its Hermitian conjugate's
  coupling (`UFOModel::bound_coupling`). After the fix `|M|²_new(+η)` equals
  `|M|²_old(−η)` bit for bit on all six probed processes (V3's claim). A test
  checks all 18 W–quark couplings against the Lagrangian; reverting the fix fails
  it and the oracle. *Manager-verified:* `G = i·e^{2i·arg V_ub}` matches;
  `amplitude_oracle` 51/51 and `coupling_oracle` 50/50 pass.
- **The rows.** `ubx_to_epve_ckm_no_b_mass` (G = +i, `|M|²` max_rel 8.1e-14 grid,
  per-diagram 3.3e-16) and `bbx_to_tapta_ckm_no_b_mass` (G = +i; `ymb = 0`
  leaves 2 diagrams where the default restriction has 3) gate on diagrams and
  amplitudes; banked on the M3 (`866c884`), bundled since refdata-11.

The manager re-ran `sm_ckm_no_b_mass` (6, then 9), `interned_sm_rows` (4) and
`coupling_oracle` (48, then 50) from the committed tree after each session.

## Backlog

- Deleted: `sm-lacks-ckm-no-b-mass-restriction`.
- Filed during the sprint: `banked-param-card-join-copied-in-tests`,
  `gen-reweight-oracle-empty-mg5-root` (HY1 Found 1, 2).
- Drafted for filing at close-out: `ckm-orientation-unpinned-on-cp-sensitive-process`
  (V3 Found 1), `charged-non-vector-vertices-bind-own-coupling` (V3 Found 2),
  `self-conjugate-complex-chiral-coefficient-unpinned` (V3 Found 3),
  `conjugate-vector-current-fallback-silent` (V3 Found 4),
  `ckm-sector-test-docs-predate-the-ckm-rows` (V3 Found 5, 6),
  `coupling-dependence-inferred-by-restriction-diff-or-probe` (F1 Found 3, V1
  Found 3), `interned-sm-row-sweep-passes-on-no-rows` (HY1 H8 and Found 4).
- Not filed:
  - the "nine restrict cards" kb lines (F1 Found 1, HY1 Found 3): the manager
    fixed them in this PR;
  - the card's licence (F1 Found 2): settled by the user;
  - the `mv`-restore mtime trap in mutation drivers (F1 Found 4): a process
    note for the dev-agent prompts or `workflow/hygiene-review.md`, not
    backlog work;
  - V1 Found 1, 4, 5 (generated-card print precision, `<model>-None` naming,
    three generators reading only `restrict_default`): nothing wrong today, and
    each closes in a line when the case arises;
  - V1 Found 2 (`aS` in a hadronic run's banked card is the PDF set's): recorded
    in the doc of `interned_sm_defaults_are_madgraphs_generated_param_card`;
  - V2 Found 1, 2 (the count test's interned-SM branch unexercised, an info
    pair hidden behind "no count"): closed by the bank, both rows gate;
  - V2 Found 3 (ghost vertices have no MadGraph oracle at tree level): an
    inherent limit, no reference exists to bank.

## Process notes

The M3's first banking run stopped at the count check (67 against 133). That
stop was correct: V2 found the count is unitary gauge, not a defect. The
conjugation bug was invisible to every earlier row because the CKM is real at
every restriction validated before; it took the first complex coupling table to
see it.
