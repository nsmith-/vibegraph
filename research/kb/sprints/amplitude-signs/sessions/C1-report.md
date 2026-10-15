---
type: Session Report
title: "C1 report: the Identity crossed-line sign and the derivative scalar sign are both missing a −1"
description: "Eight new standalone rows, compared per MadGraph graph through the production evaluator. A bare `Identity` Yukawa rooted as a fermion current on a crossed line, and every all-scalar vertex that carries `P`, come out at exactly −1 against MadGraph. The SM `FFS4` spelling and every plain `SSS1`/`SSSS1` agree. One candidate rule per item flips exactly the 34 affected diagrams of seven rows, SMEFTsim predictions outside the toy fit included, and brings every row to ≤1.8e-14. No production sign changed; the seven disagreeing rows land informational."
status: draft
tags: [amplitude, sign-convention, fermion-flow, scalar, smeftsim, toy-ufo, diagnosis, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: rows, resource: "https://github.com/nsmith-/vibegraph/commit/fea4aef", title: "test(amplitudes): standalone falsifiers for the Identity crossed-line and derivative scalar signs"}
  - {id: merge, resource: "https://github.com/nsmith-/vibegraph/commit/992c7d5", title: "Merge of amp-signs/same-flavour-4f-contact (Q1's bank_amps and per-graph gate)"}
  - {id: merge2, resource: "https://github.com/nsmith-/vibegraph/commit/2d1a7e7", title: "Merge of amp-signs/same-flavour-4f-contact at b9604e8 (diagram-to-graph MG_GRAPH_DIAGRAMS, shared coefficient parser)"}
---
Diagnosis and oracle only. No production code changed. The arms in
`root_lorentz.rs` are exactly as on `main`. Items:
`standalone-identity-crossed-line-sign-unpinned` and
`derivative-scalar-contact-sign-unoracled`. C2 makes the fix.

## What was done

- **Merged Q1, twice.** I merged `amp-signs/same-flavour-4f-contact` into this
  branch with merge commits, no rebase: first at `a7a37c3` (`992c7d5`), then at
  the hygiene-reviewed tip `b9604e8` (`2d1a7e7`), as the manager asked. This
  branch needs Q1's `bank_amps` generator option and its per-graph gate. The
  rows here use the new `MG_GRAPH_DIAGRAMS` shape: entry `d` is our diagram
  `d`'s 0-based graph. They also use the shared JAMP(1) coefficient parser.
  Rebanking all eight tables with that parser reproduced them byte for byte.
  The only merge conflicts were generated KB indexes, which I regenerated, and
  `standalone_jamps.rs`, which I ported to the new shape.
- **Staged the models.** I ran only `stage_models` from `build.sh`, extracted
  with `sed` together with `model_card_dir`, under `pixi run -e madgraph`. I
  restaged after each model edit.
- **Generated eight standalone rows.** Each one went through
  `gen_standalone_jamps.py` → `mg5_pinned.sh`. Every generation log reads
  `VERSION 3.7.1`. Every row banks `AMP()` and its JAMP coefficients.
- **Extended the standalone gate.** `standalone_jamps.rs` now has a test, a
  `MG_GRAPH_DIAGRAMS` pairing and (for the seven disagreeing rows) a
  `KNOWN_DISAGREEMENT` entry for each new row. It can also load a model
  directory and restrict card that a table records itself (`"model": "ufo"`),
  for a card no manifest row carries.
- **Added to the toy UFO.** A vertex `V_ststst` with `SSS1` (`GC_S3`, order
  `NP`) and the derivative `SSSD = P(-1,1)*P(-1,2) + P(-1,1)*P(-1,3) +
  P(-1,2)*P(-1,3)` (`GC_S3D`, new order `NPD`). It also gets parameters `gs3`
  and `gs3d` (TOYCOUP 12, 13) and a card `restrict_dscalar.dat` (`ysl`, `gs3`,
  `gs3d` on). Every existing toy card gets `gs3 = gs3d = 0`, and `restrict_all`
  gets both non-zero.
- **Added a SMEFTsim card**, `validation/madgraph/cards/smeft/restrict_vg_cHbox.dat`
  (cHbox 0.5, cHDD 0.6, the shipped values, all else zero), for the derivative
  Higgs self-couplings `SSS2`/`SSS3`/`SSSS2`/`SSSS3`.

**Correction to the brief.** `validation/ufo/vibegraph_toy_UFO` is *authored*.
It has no `SHA256SUMS` (`validation/ufo/README.md`: authored directories carry
none; only the vendored SMEFTsim does). There was no manifest to update. I
updated the README's description of the toy model instead.

### Rows

| row | process | model / card | leg 0 | what it roots | NGRAPHS = ours |
|---|---|---|---|---|---|
| `ee_to_ttxh` | `e+ e- > t t~ h` | `sm` | e⁺ | control: `FFS4` (ProjM+ProjP) current on the crossed top line | 5 = 5 |
| `ee_to_ttxh_identity` | `e+ e- > t t~ h NP<=1 SMHLOOP=0` | SMEFTsim `vg_cbH` | e⁺ | `FFS2 = Identity(2,1)` current on the crossed top line | 5 = 5 |
| `ll_to_qqxs_toy_yukawa` | `lt~ lt > qt qt~ st NP<=3 NPCP<=3` | toy `yukawa` | lt~ | `Identity` and `Gamma5` currents on the crossed qt line, uncrossed on the lt line | 40 = 40 |
| `ll_to_ss_toy_dscalar` | `lt~ lt > st st NP<=2 NPD<=1` | toy `dscalar` | lt~ | `SSS1`, `SSSD` as scalar current | 4 = 4 |
| `sl_to_sl_toy_dscalar` | `st lt > st lt NP<=2 NPD<=1` | toy `dscalar` | st | `SSS1`, `SSSD` as amplitude | 4 = 4 |
| `ttx_to_hh_chbox` | `t t~ > h h NP<=1` | SMEFTsim `vg_cHbox` | t | `SSS1`/`SSS2`/`SSS3` as scalar current | 17 = 17 |
| `ht_to_ht_chbox` | `h t > h t NP<=1` | SMEFTsim `vg_cHbox` | h | `SSS1`/`SSS2`/`SSS3` as amplitude | 17 = 17 |
| `hh_to_hh_chbox` | `h h > h h NP<=1` | SMEFTsim `vg_cHbox` | h | `SSSS1`/`2`/`3` contacts; `SSS2`/`SSS3` at one end of each exchange | 32 = 32 |

**Leg 0.** Leg 0 is a fermion or a scalar everywhere, never a colourless
vector, so W1's leg-0-vector anchor sign stays out of the measurement. The
scalar-leg-0 rows (`sl_to_sl`, `ht_to_ht`, `hh_to_hh`) are what put the
derivative vertex at the anchor, which makes it the amplitude root. The
fermion-leg-0 rows make it a current.

**Widths.** `zero_widths` is set wherever there is a spacelike massive line:
`sl_to_sl`, `ttx_to_hh`, `ht_to_ht` and `hh_to_hh`. Before I set it, the first
`ttx_to_hh` run had a 1.4e-3 residual on its top-exchange diagrams, which is
the width.

**`SMHLOOP=0`.** At first `ee_to_ttxh_identity` had eight graphs. The three
extra ones are SMEFTsim's loop-induced h-γγ/h-Zγ vertices. On their graphs the
per-diagram ratio drifted by 2.8e-8 and 1.2e-8. That is the seven-digit Fortran
literal defect `validation/madgraph-defects` already records for `gHza`, and
`gHaa` has it too. I excluded the order rather than match a rounded reference.

## Instruments

The per-diagram values come from **production**, not from the colour-free
probe. Each diagram is compiled alone as a one-diagram `DiagramSet` through
`AmplitudeEvaluator::compile`, evaluated with `eval_amplitude`. This is the
same device the committed per-graph gate uses. A temporary `#[ignore]` test
(not committed) dumped those values, their sum, the full evaluator's amplitude
and MadGraph's `AMP()`·coefficient and JAMP to JSON, per (point, helicity).

**Probe bridge.** The per-diagram sum against the full evaluator agrees to
≤ 2.95e-16 of the largest JAMP on every row. That covers the item's "compare
the per-diagram sum with the production |M|²", and it holds the contact
diagrams too, which `run_config_amps` cannot read out.

**Pairing.** Each of our diagrams is paired with the MadGraph graph it equals
up to a unit-modulus constant (residual < 1e-10, ‖r|−1| < 1e-10), by bipartite
matching. The census is machine-checked: NGRAPHS equals our count on all eight
rows, and the pairing is one-to-one. In the SMEFTsim rows some graphs are equal
at every point; see "Blind spots".

**Arm reach.** A temporary env-gated `eprintln!` in the `Identity` arm of
`build_child` and at the top of `build_at_leg` for all-scalar vertices, plus a
marker in `compile_single_diagram` separating the live tree from the
anchor-rooted reference tree, which is where `build_convention_sign` is read.
It is not committed. Patches: scratchpad `C1/C1-trace.patch`,
`C1/C1-trace-and-candidate.patch`.

## Item 1: `Identity` rooted as a fermion current on a crossed line

**The arm is reached (machine).** In `ee_to_ttxh_identity`, our diagrams 0–3
(the Higgs off t or t̄ through `FFS2`) each print `identity-arm idx=0 wrapped=1
crossed=true` or `idx=1 wrapped=0 crossed=true`. They print it once in the live
tree and once in the anchor tree (live root ≠ anchor for all five). Diagram 4
(Higgsstrahlung) prints nothing.

In `ll_to_qqxs_toy_yukawa`, every even diagram 0–18 has one `crossed=true` hit:
the scalar emitted off the qt pair through `FFS1 = Identity`, interaction 4.
The odd diagrams 1–19 emit through `FFS5 = Gamma5` (interaction 5) and have no
`Identity` hit. Diagrams 20–39 emit off the initial lt line, and their only
hits are `crossed=false`.

**Per diagram × per helicity** (48 entries per row; ratio = ours / MadGraph's
`c_k·AMP_k`, residual = max over entries of the deviation from a constant ratio):

| row | diagrams | ratio | residual |
|---|---|---|---|
| `ee_to_ttxh` (SM `FFS4`) | all 5 | `+i` | ≤ 3.1e-15 |
| `ee_to_ttxh_identity` | Higgsstrahlung (AMP 1) | `+i` | 3.0e-15 |
| | Higgs off top, `Identity` (AMP 2–5) | **`−i`** | ≤ 3.1e-15 |
| `ll_to_qqxs_toy_yukawa` | `Identity` crossed-current, 10 diagrams | **`−i`** | ≤ 6.2e-16 |
| | the other 30 (`Gamma5` crossed, `Identity`/`Gamma5` on the lt line, vector) | `+i` | ≤ 5.6e-16 |

Every diagram is individually exact in modulus and shape. The defect is a pure
per-diagram sign. Over the 50 diagrams of the three rows, the flipped set is
exactly the set with an `Identity`-arm hit where `standalone_projector_crossed`
is true in the anchor tree. Each such diagram has one hit.

**Candidate rule.** The `Identity` arm takes the same
`standalone_projector_crossed` −1 as `ProjM`, `ProjP` and `Gamma5`. The
reversed bilinear reads `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁`, and `C·1ᵀ·C⁻¹ = 1`, exactly as
for `γ⁵` and `P_χ`. The amplitude-root arm (`scalar_root`) already applies
`pair_crossed` to `Identity`. Flipping the `Identity` arm like `ProjM + ProjP`
fixes it.

**Arithmetic verification on the dumps.** Flip the hit diagrams, then refit:

| row | before: G, worst flow, worst graph | after |
|---|---|---|
| `ee_to_ttxh_identity` | G = −0.0091−0.8262i, 3.98e-1, 1.05e0 | G = +i (|G|−1 = −2.2e-15), flow 1.91e-15, graph 4.12e-15 |
| `ll_to_qqxs_toy_yukawa` | G = 0.043+1.031i, 7.41e-2, 5.30e-2 | G = +i (|G|−1 = −4.4e-16), flow 3.31e-16, graph 4.07e-16 |

**The same rule in production (temporary, reverted).** I added the three-line
check to the `Identity` arm, rebuilt and re-dumped. Both rows reproduce the
"after" column to the printed digit.

**Controls.**

- **SM `FFS4` spelling.** `ee_to_ttxh` agrees today, per graph at 4.05e-15
  (G = +i). That row is committed and gated.
- **The control can see the convention.** I mutated the `ProjM`/`ProjP` arms
  to drop their crossed flip (temporarily, reverted) and re-dumped. The SM row
  then fails in exactly the SMEFTsim pattern: AMP 2–5 at ratio +1.2003 against
  AMP 1 at −1.2003 under the fitted G, flow 3.72e-1. `ee_to_ttxh_identity` was
  unchanged by that mutation, to the digit. So the two spellings differ by
  exactly the projector flip.
- **No enforced row moved.** With the candidate (both items) in production,
  `amplitude_oracle` gave `49 passed; 0 failed`. That includes
  `bbx_to_h_identity`, `ll_to_qqx_toy_yukawa` and `qqx_to_o8o8_toy_dcolor`. All
  eleven pre-existing `standalone_jamps` rows printed the same G and worst
  residuals as without it. The existing rows never reach the changed arms (the
  items said so, and the trace confirms it on the new rows), so this is a
  no-collateral check, not a constraint on the rule.

**What would falsify the candidate.** Any of these would:

- a row where an `Identity` current on a crossed line agrees with MadGraph
  unflipped;
- an `Identity` crossed-current diagram whose ratio is not ±1 times the
  process's G;
- a gated row moving when the arm is flipped.

None was found. The mixed-line case is not tested: an `Identity` current whose
wrapped leg is crossed but whose output continues to an uncrossed partner.
That cannot occur on one line, because `crossed` is a property of the line.

## Item 2: all-scalar vertices that carry `P`

**The arm is reached (machine).** `build_at_leg` prints `all-scalar n_ops=2`
(one line per term) with `sign0=1.0` for every `SSSD`, `SSS2`, `SSS3`, `SSSS2`
and `SSSS3` occurrence: three terms for `SSSD`/`SSS2`, six for `SSS3`.
`SSS1`/`SSSS1` print `n_ops=0 sign0=-1.0`. The root is `Some(2)` (current, the
output a scalar slot) in `ll_to_ss` and `ttx_to_hh`. It is `None` (amplitude)
in `sl_to_sl`, `ht_to_ht` and the `hh_to_hh` contacts.

**Per diagram × per helicity:**

| row | diagrams | ratio | residual |
|---|---|---|---|
| `ll_to_ss_toy_dscalar` (current) | `SSS1`, two lepton exchanges | `+i` | ≤ 2.8e-15 |
| | `SSSD` (AMP 2) | **`−i`** | 4.2e-19 |
| `sl_to_sl_toy_dscalar` (amplitude) | `SSS1`, two lepton exchanges | `−i` | ≤ 4.3e-16 |
| | `SSSD` (AMP 3) | **`+i`** | 2.1e-15 |
| `ttx_to_hh_chbox` (current) | 15 `SSS1` and top-exchange diagrams | `−i` | ≤ 4.7e-14 |
| | `SSS3` (AMP 2), `SSS2` (AMP 3) | **`+i`** | ≤ 3.3e-15 |
| `ht_to_ht_chbox` (amplitude) | 15 | `−i` | ≤ 3.2e-15 |
| | `SSS3` (AMP 9), `SSS2` (AMP 10) | **`+i`** | ≤ 3.8e-16 |
| `hh_to_hh_chbox` | 18 (`SSSS1` contacts, exchanges through `SSS1` only) | `−i` | ≤ 2.2e-15 |
| | `SSSS3`, `SSSS2` contacts (AMP 2, 3) and the 12 exchanges with one `SSS3`/`SSS2` | **`+i`** | ≤ 5.5e-15 |

The pattern is the same at every root. A P-carrying all-scalar vertex is off by
−1 whether it is a current or the amplitude, and at three or four legs. The
plain partner on the same vertex agrees. The flipped set is exactly the
diagrams with one `n_ops>0` all-scalar hit, 20 diagrams over the five rows. No
diagram here has two.

**Candidate rule.** Every all-scalar vertex term takes `vertex_contact_sign`'s
scalar-sink −1, whatever operators it carries. That means dropping
`&& term.ops.is_empty()` from its all-scalar condition. The −1 is against the
`−i/D` scalar propagator, so it is a property of the vertex sinking into a
scalar, not of the term's content. A `P` contraction (`scalar_root`'s `P` arm,
or a `Metric` of two `P`s) carries no sign of its own, so nothing double-counts.

**Arithmetic verification on the dumps.** Flip the derivative diagrams:

| row | before (G, worst flow, worst graph) | after |
|---|---|---|
| `ll_to_ss_toy_dscalar` | 0.9086i, 1.90e-1, 1.79e-1 | +i, 2.68e-15, 2.32e-15 |
| `sl_to_sl_toy_dscalar` | −0.99995i, 7.31e-3, 9.07e-3 | −i, 2.77e-16, 4.26e-16 |
| `ttx_to_hh_chbox` | −1.1067i, 8.06e-2, 2.24e-1 | −i, 1.79e-14, 1.60e-14 |
| `ht_to_ht_chbox` | −0.9675i, 1.43e-1, 3.11e-1 | −i, 6.49e-16, 3.28e-15 |
| `hh_to_hh_chbox` | −0.7394i, 4.25e-1, 2.61e-1 | −i, 4.74e-15, 1.81e-15 |

The same rule in production (temporary, reverted) reproduced the first four to
the printed digit. `hh_to_hh_chbox` was banked after that run and was verified
on the dump only.

**A prediction, not a fit.** The rule was formed on the two toy rows. The three
SMEFTsim rows were generated afterwards. They put a real model's
`SSS2`/`SSS3`/`SSSS2`/`SSSS3` (O_HDD, O_Hbox) at both roots and at four legs,
and each one agreed with the toy rule.

**Controls.** Every `SSS1`/`SSSD`-free diagram, and every `SSS1`/`SSSS1`
diagram (the pinned scalar-sink −1), agrees in all five rows at both roots. The
enforced `qqx_to_o8o8_toy_dcolor` (`SSS1`), `tata_to_ttxh` and `tata_to_ttxhh`
(`HHH`, `HHHH`) are unmoved under the candidate, as listed above.

**What would falsify the candidate.** Any of these would:

- a P-carrying all-scalar structure whose diagram agrees unflipped;
- a vertex mixing a plain and a derivative term under one coupling whose
  per-diagram ratio is not a constant (that would say the −1 is per term, not
  per vertex);
- a non-symmetric derivative structure on distinct scalars that disagrees in
  magnitude, which would be a leg-slot or `POut` problem rather than a sign.

## Land status

The SM control `ee_to_ttxh` is gated. The seven others are listed in
`KNOWN_DISAGREEMENT`, each with its pattern. They are measured in full and fail
when they start agreeing, so C2's fix has to remove them. Each row's per-graph
level is live, so after the fix the rows gate per MadGraph graph at
`REL_TOL = 1e-12` (measured after-values above: worst 1.8e-14).

## Gates

Every cargo invocation ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the committed tree
(production code identical to `main`).

All four gates ran on the final merged tree. The tree was `fea4aef`'s content
before commit, after the `b9604e8` merge.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished`, exit 0.
- `cargo test --workspace`: exit 0, 38 suites, 1404 passed, 0 failed, 16 ignored
  (summed `test result` lines).
- `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`:
  `19 passed; 0 failed`. The new gated row prints
  ```
  ee_to_ttxh: 48 flow entries, G = Complex { re: 3.932458812023934e-18, im: 1.0000000000000029 }, worst flow 1.883e-15, worst |M|² 2.701e-15
  ee_to_ttxh: 240 graph entries, worst graph 4.048e-15
  ```
  The seven informational rows print `informational, disagrees as listed`:
  `ee_to_ttxh_identity` worst graph 1.047e0, `ll_to_qqxs_toy_yukawa` 5.297e-2,
  `ll_to_ss_toy_dscalar` 1.785e-1, `sl_to_sl_toy_dscalar` 9.067e-3,
  `ttx_to_hh_chbox` 2.236e-1, `ht_to_ht_chbox` 3.114e-1, `hh_to_hh_chbox`
  2.606e-1. The eleven pre-existing rows print the same G and worst residuals
  as before this session, for example `ee_to_ee_4f` worst graph 2.981e-14 and
  `bbx_to_hh` worst flow 2.327e-15.
- `pixi run --skip-deps validate`: exit 0, 63 suites, 1789 passed, 0 failed,
  84 ignored. Report: `64 rows × 4 categories = 256 cells: 203 measured in the
  layers this run drove (196 ✅, 7 ⚠️, 13 ⏳, 40 — / uncovered)` and `The
  measured cells are exactly the cells the manifest declares`. These are the
  same cell counts P1 and Q1 report, so no enforced cell moved. The same run on
  the tree before the second merge gave identical totals.

## Blind spots

- **The global phase of each row.** A sign shared by every diagram is invisible,
  as for every standalone row. The candidate rules move one diagram class
  against the rest, which is visible. A rule that flipped *every* diagram of a
  process would not be.
- **Symmetric derivative structures.** `SSSD`, `SSS2`, `SSS3`, `SSSS2` and
  `SSSS3` are all symmetric in their legs, and on momentum conservation each
  reduces to a function of the `p_i²`. So these rows cannot see two things:
  - a wrong leg-slot assignment of an individual `P(-1,k)`;
  - a `POut` sign error confined to cross terms.

  Both would need a non-symmetric `P(-1,1)*P(-1,2)` on three distinct scalars.
  The ratios are exactly unit modulus, which rules out a `POut` sign error in
  the symmetric combination itself.
- **Fixed-√s collinearity.** In `ll_to_ss_toy_dscalar` and the `hh_to_hh`
  contacts, the derivative graph is proportional to its plain partner at every
  point (fixed s, on-shell legs). The pairing there rests on the coupling
  magnitude (|r| = 1), not on shape. The t-channel rows (`sl_to_sl`, `ht_to_ht`)
  are not degenerate.
- **Equal-valued graphs in the SMEFTsim rows.** For example, `AMP(2) = AMP(4)` in
  `ht_to_ht` (`GC_1025·GC_1029` in either order), and O_Hbox's cubic shift
  against its Yukawa shift. These are paired in matching order. A swap inside
  such a set is invisible, and every member of a set carries the same ratio.
- **Mixed-structure vertices.** Every derivative structure here is a separate
  interaction, with its own coupling order. A vertex that combines a plain and a
  derivative structure under one order panics today (see Found). So whether
  MadGraph's per-vertex reading matches a per-term −1 is untested.
- **Points.** There are three to eight RAMBO points per row at one √s. The
  comparison is exact algebra, but a kinematic coincidence is not excluded.
- **`Identity` reached only as a fermion current from a crossed external leg.**
  The `idx < 0` case, an `Identity` reached through a summed spinor index
  (gamma-chained), is excluded by `standalone_projector_crossed` and was not
  exercised.

## Found

- **A vertex mixing `Identity` with `Gamma5` (or a plain with a derivative cubic
  scalar) under one coupling-order tuple panics at compile.**
  `VertexInfo::build_sign` asserts that every term of a vertex carries the same
  rooting sign, and today the two arms disagree. I reproduced it on scratch
  copies of the toy UFO, with `GC_LP`/`GC_QP` moved to order `NP` in one copy
  and `GC_S3D` moved to `NP` in another:
  - `lt~ lt > qt qt~ st NP<=3`, `st lt > st lt NP<=3` and `lt~ lt > st st NP<=3`
    each give `compile: panic a vertex's couplings carry mixed rooting-convention
    signs`;
  - `lt~ lt > qt qt~ NP<=3`, which has amplitude roots only, compiles.

  With both candidate rules applied, all three compile. A user UFO with a
  CP-violating Yukawa `y(1 + iκγ⁵)` written under one order, or with a
  composite-Higgs `(∂h)²h` beside `h³`, hits this today. To close it: C2's fix
  makes the signs uniform. Separately, a model-load or compile-time refusal (not
  an `assert!`) for any future mixed-sign case would help, and a toy row with a
  mixed-structure vertex should be banked against MadGraph.
- **Seven-digit literals reach `gHaa` as well as `gHza`.** `GC_299 =
  4i·gHaa/vevhat` sits 2.8e-8 off on `e+ e- > t t~ h NP<=1`'s h-γγ graph,
  measured as a per-diagram ratio drift of 2.8e-8 against 1.2e-8 for `GC_303`.
  `validation/madgraph-defects` and `coupling-oracle` name only `gHza`/`GC_303`.
  To close it: add `GC_299` to the coupling oracle's listed Fortran deviations,
  if any gated row reaches it, and to the defect's text.
- **No non-symmetric derivative scalar oracle.** See Blind spots. To close it:
  a toy row with `P(-1,1)*P(-1,2)` on three distinct scalars, rooted at each
  slot.
- **The standalone per-diagram dump is not a committed tool.** This session,
  like Q1, wrote a throwaway dump of production per-diagram values against
  `AMP()`. To close it: a committed `#[ignore]` dump in `standalone_jamps.rs`
  (key → JSON) and a pairing script (`compare_amps.py` reads only MadEvent work
  areas). The scratch versions are `C1/match.py`, `C1/pair3.py` and
  `C1/verify.py` in this session's scratchpad.
