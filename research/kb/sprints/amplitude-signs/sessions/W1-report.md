---
type: Session Report
title: "W1 report: the w+ w- > e+ e- sign is a leg-0 anchor sign"
description: "Diagnosis. When external leg 0 is a colourless vector, a diagram anchored at a two-fermion vertex has the wrong sign relative to one anchored at a boson vertex. One factor fixes all 22 failing probe processes, including 7 predictions outside the fit. The rule is not derived, because the class-b sign product depends on the reference root."
status: draft
tags: [amplitude, sign-convention, standalone, electroweak, diagnosis, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/84289cd", title: "Sprint plan-of-work commit this diagnosis ran on"}
---
Diagnosis only. No source change is committed. Committed with this report:
eleven informational `gen_standalone_jamps.py` rows and their tables under
`validation/madgraph/standalone/`. They are fixed data for W2, and no test reads
them yet. Every MadGraph run was the pinned 3.7.1 (`mg5_pinned.sh`, through
`gen_standalone_jamps.py`), and every generation log reads `VERSION 3.7.1`.

## Correction to the brief and the backlog item

The item frames the defect as "a W pair at the anchor and a final-state fermion
line". That is too narrow, and part of it is wrong:

- **It is not about the W pair.** One colourless vector at leg 0 is enough:
  `a e- > w- ve`, `w+ a > e+ ve`, `w+ z > e+ ve`, `z z > t t~` and
  `z h > t t~` all fail.
- **The line need not be final-state.** It fails with a mixed line too:
  `w+ e- > z ve`.
- **It is not about the neutrino exchange.** The Higgs exchange fails as well:
  `w+ h > t b~`, `z z > t t~`.
- **The crossings that agree have a different leg 0.** `e- w+ > ve z` agrees
  and `w+ e- > z ve` does not. `h w+ > t b~` agrees and `w+ h > t b~` does not.
  These are the same graph set; only the label on leg 0 changes.

## Root-cause hypothesis

Every class-b convention sign (`yang_mills_vvv_sign`, `vector_contact_sign`,
`gluon_scalar_current_sign`, `build_convention_sign`,
`reversed_convention_sign`, `fermion_current_line_sign`) is read at
`Diagram::anchor`. For a tree of three-point vertices, the anchor is the vertex
that holds leg 0.

The product of these signs, S(d, r), is not independent of the reference root r.
The probe dumped S at every root of every diagram:

- On a WWV + lepton-line diagram, S changes by the Yang–Mills source −1 between
  the VVV root and the FFV root.
- On a crossed top line through FFV and FFS, S changes by −1 between the FFV
  root and the FFS root. At the FFV root, `standalone_projector_crossed` gives −1.
  At the FFS root, the scalar-sink −1 and the crossed-pair −1 cancel.

The anchor rule therefore picks one root per diagram. The calibration rows never
fixed the relative sign between two kinds of diagram in the same process:

- a diagram anchored at a two-fermion vertex;
- a diagram anchored at a boson vertex (VVV, VVS, VVVV, VVSS);

when leg 0 is a colourless vector. Every amplitude-oracle and standalone row had
one of these instead:

- leg 0 a fermion, so every anchor is a fermion vertex;
- leg 0 a gluon, where `g g > u u~` agrees;
- a pure-boson process, where every anchor is a boson vertex.

Measured: with a colourless vector at leg 0, the fermion-vertex-anchored
diagrams need a −1 relative to the boson-vertex-anchored ones. With a Higgs or a
gluon at leg 0 they need none.

This is a rule that fits the data, not a derivation. A principled fix would make
S(d, r) independent of r. No such per-vertex-role assignment exists in the
feature set below (see "Blind spots").

## Evidence chain

**Parameter provenance.** Both sides read the same `param_card` (each table
carries MadGraph's). Every width is zeroed on both sides, except in
`g g > u u~`, which has no massive t-channel line.

**Per-diagram dumps.** The probe is a temporary `#[ignore]` test, now reverted.
Its patch is in the scratchpad as `W1-probe-and-candidate.patch`. It lowers each
diagram through `compile_diagram_ast` and runs the production `BoundAmplitude`
on the same points.

Bridge (the diagram sum against the production evaluator, per helicity, relative
to the largest entry):

| process | bridge |
|---|---|
| `wpwm_to_wpwmz` | 1.8e-12 |
| `wpwm_to_wpwm` | 2.1e-14 |
| every other single-flow probe | ≤ 3.7e-15 |

So the test-only lowering agrees with the production `Program` on every probe.

MadGraph side: `gen_standalone_jamps.py --amps`, run on 46 tables:

- 40 processes;
- 6 relabelled copies (below).

**Step 1–2: the defect is a pure per-diagram sign.** Every vibegraph diagram
pairs with exactly one MadGraph `AMP()`:

- overlap 1.000000000;
- |c| = 1.000000000000;
- residual ≤ 1.6e-13 for `wpwm_to_wpwmz` and ≤ 7.1e-15 for every other process.

The ratio c is ±i throughout. For the rows named in the brief, with s and t the
photon/Z s-channel and the fermion-exchange diagrams:

| process | c (s, s, t) | worst flow before |
|---|---|---|
| `w+ w- > e+ e-` | (+i, +i, −i) | 6.29e0, \|G\| 2.577 |
| `w- w+ > e- e+` | (+1, +1, −1) relative | 3.87e0, \|G\| 2.168 |
| `w+ w- > u u~` | (+1, +1, −1) relative | 4.85e0, \|G\| 1.902 |

Controls that agree today (worst flow, \|G\| = 1):

| process | worst flow |
|---|---|
| `e+ e- > w+ w-` | 2.1e-15 |
| `u u~ > w+ w-` | 1.0e-13 |
| `w+ w- > w+ w-` | 1.3e-13 |
| `w+ w- > w+ w- z` | 3.7e-12 |
| `g g > u u~` | 7.6e-15, 2 flows |

**Step 3: GF(2) solve.** Unknowns are per-diagram features:

- vertex class × role (anchor, or the spin of the anchor-rooted output);
- fermion-line class × propagator parity;
- leg-0 species × anchor class.

Each process gets its own free global sign. The equations are:

- the needed flip pattern of every probe;
- "no relative change" for every diagram of every committed Standard-Model
  `amplitude_oracle` and `standalone_jamps` row (31 processes, re-enumerated).

The system has 43 features and 1613 equations. Results:

- **The only single-feature solution is `leg0 colourless vector × anchor FFV`.**
  The weight-4 equivalent is `leg0 colourless vector × anchor ∈ {VVV, VVS,
  VVVV, VVSS}`. It differs from the single-feature rule by a sign per process.
- **No solution exists without a leg-0 feature.** The minimal core that forces
  one is `h w+ > t b~` alone. Without it, "anchor ∈ {VVV, VVS, VVVV, VVSS}"
  solves.
- **No role-free assignment exists** (one constant per vertex class), with or
  without line features. This agrees with note 39 §4.3.

**Step 4: arithmetic verification on the dumps.** Recipe: flip, recontract,
fit G, compare per flow and per helicity |M|² (scratchpad `W1-rule.py`,
outputs `W1-ruleA.txt` and `W1-ruleB.txt`). Rule B is "−1 on a diagram whose
anchor has a fermion leg, when leg 0 is a colourless vector".

Failing probes, worst flow before → after rule B, every one with \|G\| =
1.000000000000 after:

| process | before | after |
|---|---|---|
| `w+ w- > e+ e-` | 6.29e0 | 9.4e-15 |
| `w+ w- > e- e+` | 4.60e0 | 4.3e-15 |
| `w- w+ > e- e+` | 3.87e0 | 1.4e-15 |
| `w- w+ > e+ e-` | 1.24e1 | 3.8e-14 |
| `w+ w- > u u~` | 4.85e0 | 1.8e-15 |
| `w+ w- > d d~` | 2.51e1 | 1.3e-14 |
| `w+ w- > ve ve~` | 3.31e0 | 1.8e-15 |
| `w+ w- > t t~` | 9.42e0 | 1.6e-14 |
| `w+ z > e+ ve` | 1.76e1 | 2.1e-15 |
| `w+ a > e+ ve` | 2.15e0 | 2.7e-15 |
| `w+ a > t b~` | 1.50e1 | 5.8e-15 |
| `w+ e- > z ve` | 1.08e1 | 3.1e-15 |
| `a e- > w- ve` | 3.92e0 | 9.0e-16 |
| `w+ h > t b~` | 7.63e0 | 1.0e-14 |
| `z z > t t~` | 4.20e0 | 8.8e-15 |
| `w+ w- > e+ e- z` (16 diagrams) | 1.43e2 | 4.3e-14 |

The probes this rule must leave alone are unchanged: `e+ e- > w+ w-`,
`e- e+ > w- w+`, `u u~ > w+ w-`, `e+ ve > w+ z`, `e- w+ > ve z`,
`h w+ > t b~`, `h h > t t~`, `t t~ > h h`, `t b~ > w+ h`, `e- ve~ > w- h`,
`a a > e+ e-`, `z z > e+ e-`, `w+ w- > h h`, `w+ w- > w+ w-`,
`w+ w- > w+ w- z` and `g g > u u~`.

**Predictions.** These processes were generated after the rule was formed:

| process | before | after |
|---|---|---|
| `a w+ > t b~` | 8.64e0 | 1.5e-14 |
| `z w+ > t b~` | 1.59e1 | 3.0e-14 |
| `z h > t t~` | 3.32e0 | 1.8e-15 |
| `z w+ > e+ ve` | 8.34e0 | 2.3e-15 |
| `w+ a > e+ ve a` (7 diagrams) | 1.70e0 | 6.1e-15 |
| `w+ w- > u u~ g` (7 diagrams, a final-state gluon) | 6.65e1 | 3.1e-14 |

Two controls not in the fit are unchanged: `h z > t t~` (1.1e-15) and
`t t~ > z z` (3.2e-14).

**Production check of the candidate.** The candidate went in temporarily as a
factor in `compile_single_diagram`'s `fermi_sign`, switched by an environment
variable, so one build ran baseline, B and A. Results:

- `cargo test -p vibegraph-lib --test standalone_jamps --test amplitude_oracle`:
  - Baseline: 10/10 and 49/49 pass.
  - B: 49/49 pass. `wpwm_to_epem` now fails only as "listed as disagreeing but
    now agrees: G = +1.0000000000000024i, worst flow 9.423e-15, worst |M|²
    1.494e-15". The printed lines of every other row are identical to baseline,
    digit for digit.
  - A: the same `wpwm_to_epem` result. It also turns the fitted G of
    `wpwm_to_wpwm` and `wpwm_to_wpwmz_cw` from −i to +i; nothing else changes.
- B, `helas::eval::renumbering`: 2/2 pass.
- B, `rooting_soundness::all_rootings_preserve_amplitude`: "270 re-rootings,
  0 failures", pass.

**A finer oracle: relabelling.** Six pairs P1/P2 differ only by swapping
legs 0 and 1 (and legs 2 and 3 for `w+ e- > z ve` → `e- w+ > ve z`). Each P2
was evaluated at P1's points with the momenta and helicities permuted. MadGraph's
`AMP()` of every graph is unchanged: ratio +1.000000, residual ≤ 2.6e-15.

Vibegraph's value for the same diagram flips sign (−1.000000) exactly where the
anchor moves to a different vertex class, and nowhere else:

| pair | diagram | anchor in P1 → P2 |
|---|---|---|
| `w+ e- > z ve` → `e- w+ > ve z` | the W exchange | VVV → FFV |
| `w+ h > t b~` → `h w+ > t b~` | the two quark exchanges | FFV → FFS |
| `z h > t t~` → `h z > t t~` | the two top exchanges | FFV → FFS |

`w+ a`/`a w+ > t b~`, `w+ z`/`z w+ > e+ ve` and `w+ w-`/`w- w+ > e+ e-` keep
their anchor classes, and every ratio is +1.

This is the anchor-dependence of S seen directly. It needs no MadGraph: the
two labellings disagree with each other.

**Step 5: banked σ rows the fix moves.** None.

- The only banked σ or sample row with a colourless vector at leg 0 is
  `w+ w- > w+ w- z NP<=1`. Every anchor there is a boson vertex, so B does not
  fire.
- `decay_widths.rs` compares |M|² only. Its `z > e+ e- mu+ mu-` row has
  FFV anchors throughout, so B is a global sign there.
- Decay-chain stitched diagrams take leg 0 from the production process.
- Every hadronic, MLM and grammar row is parton- or lepton-initiated.

## What would falsify it

- A process with a colourless vector at leg 0 where B leaves a per-flow
  residual above rounding. The likeliest places:
  - the anchor is a ≥4-point contact holding leg 0 beside a fermion-vertex
    anchor elsewhere (only exercised inside `w+ w- > e+ e- z`);
  - a non-SM colourless VVS or VVV (SMEFT `O_W`, `O_HW`, a photon–Z–Higgs
    vertex) at the anchor;
  - a coloured vector that is not a gluon.
- A leg-0-Higgs or leg-0-gluon process that needs the flip. Then the rule's
  restriction to colourless vectors is wrong.
- A leg-0-fermion process whose fermion-vertex-anchored diagrams interfere
  wrongly with others. Not seen: every anchor there is a fermion vertex.

## Fix recommendation for W2

In `vibegraph-lib/src/helas/eval/root_diagram.rs`, `compile_single_diagram`,
multiply one more graph-invariant factor into `fermi_sign`:

```rust
/// −1 when external leg 0 is a colourless vector and the anchor vertex has a
/// fermion leg; +1 otherwise.
fn leg0_vector_fermion_anchor_sign(diagram: &Diagram, model: &UFOModel) -> i8 {
    let leg0 = model.particle(diagram.legs[0].particle);
    let anchor = model.vertex_def(diagram.vertex(diagram.anchor()).interaction);
    let fermion_anchor = anchor.particles.iter().any(|&p| model.particle(p).spin % 2 == 0);
    if leg0.spin == 3 && leg0.color.abs() == 1 && fermion_anchor { -1 } else { 1 }
}
```

This is the exact predicate tested above. It reads only labelled legs and
`Diagram::anchor`, so it survives renumbering and re-rooting. Prefer this form
(B) to the boson-anchor form (A): B changes no committed row's fitted G, while
A flips the G of `wpwm_to_wpwm` and `wpwm_to_wpwmz_cw`.

Its doc comment should say what it compensates for: the relative class-b sign
between fermion-vertex anchors and boson-vertex anchors under a colourless-vector
leg 0. Then:

1. Remove `wpwm_to_epem` from `KNOWN_DISAGREEMENT`.
2. Add `#[test]`s for the eleven new standalone tables, each expected to agree
   under B. Before B, `hwp_to_tbx`, `hh_to_ttx` and `gg_to_uux` agree, and the
   other eight disagree.

Controls W2 must run, outside the fit:

- `standalone_jamps`, including all eleven new rows;
- `amplitude_oracle` (every row's printed line unchanged);
- `helas::eval::renumbering`;
- the `rooting_soundness` sweep;
- `decay_widths`;
- `pixi run --skip-deps validate`.

Worth adding:

- A **relabelling-invariance test**, hermetic and needing no MadGraph: per
  diagram, P1 against P2 with permuted momenta and helicities. Under B it holds
  per diagram for `w+ h`/`h w+` and `z h`/`h z`. For `w+ e-`/`e- w+` it holds
  only up to a per-process global −1, so that test must compare up to one sign
  per process.
- One SMEFT probe with a colourless vector at leg 0, for example
  `w+ w- > e+ e- NP<=1` with `vg_cW`.

## Blind spots

- **Per-process global sign.** The rule is fixed only up to a sign per process.
  A and B differ exactly there. |M|², the fitted G (free in sign) and every σ
  are blind to it. The relabelling oracle sees it.

  Neither A nor B makes per-diagram values invariant under relabelling:
  - B leaves a global −1 between `w+ e- > z ve` and `e- w+ > ve z`;
  - A leaves one between `w+ h > t b~` and `h w+ > t b~`, and between
    `z h > t t~` and `h z > t t~`.

  The GF(2) solve with the relabelled pairs joined under one global sign still
  has solutions, but every one has a leg-0 feature. The lightest (weight 3) is
  B plus a global −1 on every process whose leg 0 is a colourless fermion.
  Physically harmless; it is noted so that nobody reads relabelling-invariance
  into the fix.
- **Feature set.** "No leg-0-free solution" holds only for the features listed
  above, plus anchor-propagator species and where each line closes. A richer
  feature set might express the rule without leg 0.
- **SM only.** The committed rows of the GF(2) system are the Standard-Model
  ones. SMEFT and toy-model `amplitude_oracle` rows enter only through the
  production run, all 49 pass, and none of them has a colourless vector at
  leg 0.
- **Few points.** There are three RAMBO points per table, and one √s per table.

## Found

- **The class-b sign product S(d, r) depends on the reference root, and the
  anchor hides this.**
  - Evidence: the per-root dump above shows the Yang–Mills −1 and the
    crossed-line FFS signs vary with the root, and the relabelling pairs flip
    diagram signs where MadGraph does not. `rooting_soundness` cannot see it,
    because it re-roots the live tree but always reads the signs at the same
    anchor.
  - What would close it: either a convention whose class-b product is equal at
    every root (a redesign of `yang_mills_vvv_sign`, the FFS crossed-pair /
    `standalone_projector_crossed` signs and the VVS pure-metric sign together),
    or a hermetic relabelling-invariance gate that pins the anchor-dependence
    wherever it is used.
- **No hermetic relabelling-invariance test exists.** Swapping two initial legs
  of a process changes per-diagram signs today (measured above). A test needs
  only `BoundAmplitude` and permuted momenta.
- **The backlog item's premise is narrower than the defect.** Leg 0 is the
  trigger, not "a W pair at the anchor and a final-state line" (see the
  correction above). The item text and note 39 §5 should say so at close-out.
- **`gen_standalone_jamps.py --amps` has no option for points from another
  table.** The relabelling check needed it, and a scratch wrapper did the job.
  An option would make the relabelling oracle a one-liner.
