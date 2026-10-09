---
type: Physics Convention
title: Vector-vertex convention signs
description: "The colourless VVV source sign, the four-vector contact sign (per vertex) and the gluon–scalar current sign, read at Diagram::anchor; why they are graph invariants; the open W-pair case."
status: draft
tags: [sign-conventions, vector-vertices, qcd, electroweak, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n39-wrong, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L16-L53", title: "Note 39 §1 (what was wrong)"}
  - {id: n39-rule, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L54-L72", title: "Note 39 §2 (the rule)"}
  - {id: n39-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L73-L123", title: "Note 39 §3 (the standalone JAMP oracle)"}
  - {id: n39-found, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L124-L155", title: "Note 39 §4 (how the rule was found; falsified candidates)"}
  - {id: n39-open, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L156-L171", title: "Note 39 §5 (open)"}
  - {id: n39-gates, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L172-L208", title: "Note 39 §6 (gates and their blind spots)"}
  - {id: n35-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L432-L474", title: "Note 35 §3.5 (C1 colour, E2 the contact sign)"}
  - {id: n16-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/16-color-flow-design.md#L19-L36", title: "Note 16 outcome (the gg→gg contact phase)"}
  - {id: code-rootdiag, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs", title: "yang_mills_vvv_sign, vector_contact_sign, gluon_scalar_current_sign, anchor_rooted_outputs"}
  - {id: code-rootl, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_lorentz.rs", title: "build_at_leg: the per-vertex contact −1"}
  - {id: code-standalone, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/standalone_jamps.rs", title: "standalone_jamps (per-flow JAMPs against MadGraph standalone)"}
  - {id: code-pt, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/gluon_parke_taylor.rs", title: "gluon_parke_taylor (hermetic MHV check)"}
---

# Vector-vertex convention signs

Besides the fermion signs, a diagram's `fermi_sign` carries three
convention signs for vector vertices. All three are read at the
[anchor](../amplitudes/rooting-invariance-and-anchor.md) rooting, in
`helas/eval/root_diagram.rs`[^code-rootdiag]:

| function | factor | fires on |
|---|---|---|
| `yang_mills_vvv_sign` | −1 each | **colourless** Yang–Mills VVV vertex (three vector legs, a `P` in its structure, no coloured leg) that is not the anchor |
| `vector_contact_sign` | −1 each (cancels the kernel's −1) | all-vector vertex of ≥ 4 legs that is **coloured**, or is **not the anchor** |
| `gluon_scalar_current_sign` | −1 each | two-vector–one-scalar vertex with a coloured leg whose anchor-rooted output is the **scalar** |

The kernel side: `LorentzEvalTree::build_at_leg` (`root_lorentz.rs`) gives every
all-vector vertex of four or more legs a −1, once, as a property of the
**vertex** (≥ 4 legs, all spin 1), whatever operators a particular term
contains[^code-rootl]. Net effect:

| vertex | as the anchor (amplitude sink) | as a source (off the anchor) |
|---|---|---|
| colourless VVV (γWW, ZWW, SMEFT VVV) | +1 | −1 |
| triple-gluon VVV | +1 | +1 |
| colourless contact (WWZZ, WWAA, WWWW, SMEFT multi-vector) | −1 | +1 |
| gluon contact (any structure) | +1 | +1 |
| coloured VVS (effective ggH, `O_HG`), scalar output | — | −1 |
| coloured VVS, sink or gluon output | +1 | +1 |

These sit beside the fermion-line signs and the build/reversed-bilinear signs
in the same `fermi_sign`; the whole inventory is
[convention-sign-inventory](../amplitudes/convention-sign-inventory.md).

## Why a triple-gluon vertex takes no source sign

The honest vector current from a VVV vertex is rooting-invariant. A colourless
VVV needs a −1 relative to it at a vector output (source) leg: the vertex's
Lorentz structure is antisymmetric. A triple-gluon vertex is antisymmetric in
colour as well (`f^{abc}`), and the colour decomposition carries that half of
the antisymmetry. With a −1 at every gluon source, every process that puts a
triple-gluon source beside a quark-line anchor (`u u~ > g g`, `u g > u g`,
`u u~ > g g g`, `u u~ > t t~ g`, `t t~ > g g`) gets the wrong relative sign
between its gluon-exchange and quark-exchange diagrams. That error is visible
without MadGraph: the same-helicity `u u~ > g g` amplitudes, which the Ward
identity sets to zero, come out at `|M|² ≈ 105` at `√s = 500`[^n39-wrong].

The gluon contact follows the triple-gluon vertex (no sign in either role).
The colourless contact keeps the kernel's −1 only as the anchor
(`w+ w- > w+ w-` and `a a > w+ w-` pin that). The coloured VVS vertex needs −1
exactly when the anchor-rooted tree has it produce the off-shell scalar
current, the same scalar-output sign the colourless pure-metric VVS takes
through its build sign; it sets the relative sign of Higgs and gluon exchange.

### Why compensating errors hid this

In a process with only gluon vertices, flipping "every gluon source, every
gluon contact, every scalar source" is a global sign. A wrong triple-gluon
source sign therefore compensated a wrong gluon-scalar sign in every
pure-gluon process with Higgs exchange (`gg_to_gg_cg`), and a wrong contact
sign in `g g > g g`. The fitted global constant `G` of `gg_to_gg` and
`gg_to_gg_cg` flipped from −i to +i when the rule was fixed, and nothing else
in the enforced amplitude oracle moved. A gluon-only process cannot pin these
signs; a quark-anchored gluon source can[^n39-wrong].

## Why it is a graph invariant

Each factor is a function of three things only[^n39-rule]:

- the vertex's particle content (spins, arity, colour);
- whether the vertex is `Diagram::anchor`, the first lowest-arity vertex, itself
  a graph invariant;
- the particle on the vertex's path to the anchor (`anchor_rooted_outputs`, a walk
  over the undirected graph from the anchor).

None reads a vertex index or the live evaluation rooting, so every numbering
and every re-rooting of one diagram gets the same factor. The evaluator roots
for performance elsewhere, and the honest currents handle the tensor
contraction root-invariantly while this scalar carries the antisymmetric-vertex
sign; their product reproduces the anchor-rooted amplitude for every rooting.
`helas::eval::renumbering` and `helas::eval::rooting_soundness` check that.

## Details that matter

- **`is_yang_mills_vvv` tests the arity.** A contact of four or more vectors
  can carry momenta (SMEFTsim's `VVVV2`/`VVVV3`, the five-vector structures of
  `O_W`). Without the `len() == 3` test such a vertex would take both the
  contact sign and the Yang–Mills source sign off the anchor.
- **The contact −1 is per vertex, not per term.** The four-gluon vertex and the
  field-strength operators sharing its legs sit in one interaction whose
  structures range over pure metrics, momentum products and Levi-Civita
  tensors; some `O_Gtil` terms carry no `Metric` at all. A per-term test ("no
  `P`, no `Gamma`") gave one vertex different signs. Pinned by
  `four_vector_contact_sign_is_uniform_over_its_structures`[^n35-e2].
- **The contact sign is real.** An amplitude-rooted pure-metric contact lowers
  as a real −1 times a plain `Metric`, the same as the scalar-output branch.
  A contact has no propagator on its line to carry the per-diagram `−i` that
  cancels in |M|² elsewhere, so a `−i` there is a physical +90° phase against
  MadGraph[^n16-outcome]. Where each factor of `i` lives is
  [global-phase-i-counting](../amplitudes/global-phase-i-counting.md).
- **ALOHA's VVV normalisation is uniform.** ALOHA emits `VERTEX = −i·COUP·L` for
  every VVV structure, so SMEFTsim's real `GC_7 = G` against the SM's `i·G` does
  not change the sign convention[^n35-e2].

## How the rule was found, and what it rules out

The method generalises; it is
[validation/bit-exact-amplitude-debugging](../validation/bit-exact-amplitude-debugging.md)
applied to signs[^n39-found]:

1. **Per-diagram sign recovery.** Least-squares fit
   `Σ_d c_d · (our diagram d's per-flow values) = MadGraph JAMPs` over every
   entry. Every residual came out at 1e-15 with every `c_d = ±c`: the
   discrepancies were pure per-diagram signs.
2. **Solve over GF(2).** Each probe's required flip pattern, plus "no relative
   change" for every diagram of every committed amplitude-oracle row, is a
   linear system over GF(2) in per-diagram features (vertex class counts, counts
   off the anchor, class at the anchor, source output particle, fermion-line
   classes, propagator species). **No role-free solution exists**: no
   per-vertex or per-propagator constant works. The solution space fixes the
   three factors up to choices that controls decided.
3. **Predict, then check.** The gluon–scalar sign was forced only by
   `gg_to_gg_cg`, entangled with the triple-gluon sign. `t t~ > g g NP<=1`
   (`vg_cHG`), with `O_HG` as a scalar source beside the top-line anchor, was
   not fitted: without the gluon–scalar sign it misses by 7.6e-2, with it it
   agrees to 5.7e-16.

Falsified candidates:

- **"Contact −1 only at the anchor"** fixes the pure-gluon rows and leaves
  `u u~ > g g`, `u g > u g`, `u u~ > g g g` and `t t~ > g g` wrong. The gluon
  source sign is a separate defect.
- **"σ_V constant per vertex"**, a fixed sign per vertex type in both roles
  (for example `σ_V = (−1)^(number of Yang–Mills VVV at index ≥ 1, ggg
  included)`): triple-gluon +1 and γWW/ZWW −1 everywhere, with every contact
  +1, fixes every probe, including `w+ w- > e+ e-`, but breaks the enforced
  `gg_to_gg_cg`. Adding the gluon–scalar sign and the VVS signs the GF(2)
  solution then asks for keeps every committed row but breaks the SM
  `w+ w- > w+ w- z` (1.5 against 3.7e-12). A per-vertex constant cannot be the
  rule.
- **Redefining the anchor** as leg 0's vertex was not adopted; the anchor stays
  the first lowest-arity vertex.

## The oracle and what each gate cannot see

`validation/madgraph/gen_standalone_jamps.py` generates a process with MadGraph
`output standalone` (pinned 3.7.1), patches `MATRIX` to copy `AMP()` and
`JAMP()` into a COMMON block, and dumps per RAMBO point and per helicity the
colour-summed |M|², every JAMP and every AMP, at the same param card and
momenta as ours. Standalone output keeps the width of a spacelike propagator
while MadEvent (and this crate) zero it (`zerowidth_tchannel`). Through a
massive t-channel line that is a 0.1–0.5% per-helicity difference, not a sign,
so such rows compare with every width zero on both sides (`zero_widths`)[^n39-oracle].

| gate | what it pins | blind to |
|---|---|---|
| `tests/standalone_jamps.rs`[^code-standalone] (hermetic, committed tables in `validation/madgraph/standalone/`) | per-helicity, per-flow JAMPs and \|M\|² against MadGraph standalone; one fitted `G`, `\|G\| = 1`, tolerance 1e-12. Rows: `gg_to_ggg`, `uux_to_ggg`, `ug_to_ug`, `ee_to_wpwmz`, `wpwm_to_wpwm`, `ttx_to_gg_chg`, the tau Yukawa rows, `bbx_to_hh`; `wpwm_to_epem` as a two-way known disagreement | the phase of `G`; compensating errors in diagrams that reach the same flows with the same weight; kinematics outside three points |
| `tests/gluon_parke_taylor.rs`[^code-pt] (hermetic, no MadGraph) | 5 and 6 gluons, every MHV and anti-MHV configuration: `J_σ·⟨σ1σ2⟩…⟨σnσ1⟩` constant over flows and `\|J\|·\|cyclic\|/\|⟨ij⟩\|⁴` constant over configurations; tolerance 1e-10 in units of the point's largest flow | global phase and normalisation; per-configuration phase; NMHV; anything with a quark (the triple-gluon source beside a quark, and the gluon–scalar sign, are invisible) |
| `tests/amplitude_oracle.rs` | per-diagram and per-flow values on the banked rows | anything those rows do not reach: no enforced row has a colourless contact at the anchor |

Mutation results, each against the three suites[^n39-gates]. They were measured
when `standalone_jamps` held its first six rows and `wpwm_to_epem`; the tau
Yukawa and `bbx_to_hh` tables came later and were not part of the sweep:

| mutation | fails |
|---|---|
| drop `vector_contact_sign` | `gg_to_ggg`, `uux_to_ggg`, `ee_to_wpwmz`; both Parke–Taylor tests; oracle `gg_to_gg`, `gg_to_gg_cg` |
| triple-gluon vertex keeps the source sign | `uux_to_ggg`, `ug_to_ug`, `ttx_to_gg_chg`; six-gluon Parke–Taylor; oracle `gg_to_gg`, `gg_to_gg_cg` |
| drop `gluon_scalar_current_sign` | `ttx_to_gg_chg`; oracle `gg_to_gg_cg` |
| colourless contact treated like a gluon one | `wpwm_to_wpwm` only |
| contact −1 at the anchor only, nothing else | `uux_to_ggg`, `ug_to_ug`, `ttx_to_gg_chg` |

**The colourless contact at the anchor is pinned only by `wpwm_to_wpwm`** in
`standalone_jamps`. Which gate covers which sign channel across the whole
inventory is [validation/convention-channel-coverage](../validation/convention-channel-coverage.md);
the oracle family is [validation/amplitude-oracle](../validation/amplitude-oracle.md).

## Open

- **`w+ w- > e+ e-`** (also `w- w+ > e- e+` and `w+ w- > u u~`): the t-channel
  neutrino (down-quark) exchange has the wrong sign relative to the γ/Z
  s-channel. The crossings `e+ e- > w+ w-` and `u u~ > w+ w-` agree, as do
  `w+ w- > w+ w-` and `w+ w- > w+ w- z`, so it needs a W pair at the anchor
  *and* a final-state fermion line; the QCD analogue `g g > u u~` agrees. A
  constant sign on the γWW/ZWW vertex fixes it and breaks
  `w+ w- > w+ w- z`. The next place to look is the final–final fermion line
  with chiral `FFV2` vertices, per diagram against `AMP()`. Tracked as
  [wpwm-to-epem-neutrino-exchange-sign](../backlog/validation/wpwm-to-epem-neutrino-exchange-sign.md)[^n39-open].
- **`wpwm_to_wpwmz_cw`** (`O_W` through a five-vector vertex) still disagrees
  (|M|² `max_rel` 2.79e1). The SM part of the same process agrees to 3.7e-12,
  so the residual is `O_W`'s five-vector and momentum-bearing contact
  structures; its `amplitudes` cell is informational. Tracked as
  [wpwmz-cw-ow-five-vector-residual](../backlog/validation/wpwmz-cw-ow-five-vector-residual.md).

[^n39-wrong]: Note 39 §1. Defect 2 of note 38 (`u u~ > t t~ g NP<=1`, `vg_c4q`) was the triple-gluon source sign, not the four-quark contacts.
[^n39-rule]: Note 39 §2.
[^n39-oracle]: Note 39 §3. Its per-row before/after table and the `p p > j j` σ figures are as of 2026-09-25 and are not repeated here.
[^n39-found]: Note 39 §4. Note 38 §4 S1's "contact −1 only at the anchor" (with the anchor at leg 0's vertex) and note 19 V5's σ_V rule are the falsified candidates.
[^n39-open]: Note 39 §5.
[^n39-gates]: Note 39 §6, the mutation table.
[^n35-e2]: Note 35 §3.5, the per-term-to-per-vertex contact sign and the ALOHA VVV normalisation. Its "`is_yang_mills_vvv` lacks an arity check" is fixed.
[^n16-outcome]: Note 16 outcome: the amplitude-root pure-metric contact was lowered with a `−i` kernel (`Op::MetricNegI`, since removed); it is now a real −1 and a plain `Metric`.
[^code-rootdiag]: `vibegraph-lib/src/helas/eval/root_diagram.rs`: `is_yang_mills_vvv`, `is_coloured`, `is_vvs`, `anchor_rooted_outputs`, and the three sign functions.
[^code-rootl]: `vibegraph-lib/src/helas/eval/root_lorentz.rs`, `build_at_leg`: "An all-vector contact of four or more legs carries the −1 as a property of the vertex".
[^code-standalone]: `vibegraph-lib/tests/standalone_jamps.rs`, `KNOWN_DISAGREEMENT`.
[^code-pt]: `vibegraph-lib/tests/gluon_parke_taylor.rs`.
