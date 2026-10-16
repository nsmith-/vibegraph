---
type: Session Brief
title: "B′: one root-independent role-sign convention"
description: "Derive the class-b role signs so a diagram's sign product no longer depends on its root, accept it on a relabelling-invariance test plus every MadGraph per-diagram table, and land it; or stop and return the three fitted rules."
status: draft
agent: validation-dev (Opus)
depends_on: [W1, F1, C2]
closes: [wpwm-to-epem-neutrino-exchange-sign]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
User decision (sprint log, 2026-10-15): W2 and F2's sign work run as this one
session. Read the [W1](W1-report.md), [F1](F1-report.md), [C1](C1-report.md)
and [C2](C2-report.md) reports first; then
[rooting invariance and the anchor](../../../amplitudes/rooting-invariance-and-canonical-form.md),
[vector-vertex signs](../../../amplitudes/vector-vertex-signs.md) and
[fermion-line sign](../../../amplitudes/fermion-line-sign.md).

**The problem.** The class-b convention signs (`yang_mills_vvv_sign`,
`vector_contact_sign`, `gluon_scalar_current_sign` in `root_diagram.rs`;
`vertex_contact_sign` and the `build_child` arms in `root_lorentz.rs`) are read
at `Diagram::anchor`, the vertex holding leg 0, and their product depends on the
reference root. Three role signs are missing or wrong; each has a fitted rule
that fixes every table, and none is derived:
1. W1: −1 on FFV/FFS-anchored diagrams when leg 0 is a colourless vector.
2. F1: the colourless all-vector contact sign by arity, (−1)^(n−3) at the
   anchor and (−1)^(n−2) as a source, wrong at n = 5.
3. F1: colourless momentum-bearing VVS terms take no role sign; they need the
   pure-metric VVS treatment (sink or scalar output −1, vector output none).
Two of them show as relabelling dependence (signs that change when legs 0 and 1
swap), which MadGraph's `AMP()` never has.

**The work.**
1. **Acceptance test first (falsifier).** A hermetic relabelling-invariance
   test: per-diagram values of a process and its leg-permuted relabelling, paired
   by diagram, equal up to one global sign per process. Cover W1's and F1's
   relabelling pairs (`w+ e- > z ve` vs `e- w+ > ve z`, `w+ h > t b~` vs
   `h w+ > t b~`, `w+ w- > w+ w- z` vs `w- w+ > w- w+ z`) and enough controls.
   It must fail on today's code, for exactly those diagrams.
2. **Derive** the role-sign convention once, so the sign product is the same at
   every root, for every vertex class: fermion lines, all-vector contacts by
   arity, VVS per term (metric and momentum), coloured and colourless. State the
   derivation in the code's comments in its own terms. Keep C2's two arms green.
3. **Accept** only if all of these hold:
   - the relabelling test passes;
   - every MadGraph per-diagram table agrees, including W1's 11 rows (22 failing
     processes plus 17 controls), F1's 8 rows and C's 8 rows;
   - every committed `amplitude_oracle` and `standalone_jamps` row is unchanged
     digit for digit (sorted diff of the printed lines);
   - `rooting_soundness`, including the `#[ignore]`d `all_rootings_preserve_amplitude`
     sweep, and `renumbering` pass.
   Then turn W1's and F1's informational tables into tests, and drop
   `wpwm_to_epem` from `KNOWN_DISAGREEMENT`. The five-vector row's per-graph gate,
   partition pairing and manifest cell are F2's, on PR #33.
4. **Stop rule.** If no derived convention meets item 3 within this session,
   stop. Commit the relabelling test as informational, and report the three
   fitted rules (A′) with their falsifiers for the user's approval. Do not land
   fitted rules.

Invoke the `extended-validation` skill and run what it names for a
sign-convention change, including `pixi run --skip-deps validate` with every
enforced cell unmoved. Report any banked σ row the change moves.
