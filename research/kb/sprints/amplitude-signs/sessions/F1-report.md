---
type: Session Report
title: "F1 report: wpwm_to_wpwmz_cw is two missing signs, one of them not O_W's"
description: "Diagnosis. Per graph against MadGraph's AMP(), the 222-graph row disagrees by a relative −1 on exactly eight diagrams: O_W's two five-vector contacts, and six diagrams where the loop-induced photon–Z–Higgs vertex (VVS4, SMHLOOP) produces the off-shell Higgs. Flipping both classes takes |M|² from 2.79e1 to 1.1e-11 in production; the remainder is MadGraph's seven-digit GC_303. Neither is W1's leg-0 anchor rule. A diagram pairing derived from the AMP() values makes the configuration partitions identical."
status: draft
tags: [amplitude, sign-convention, smeft, five-vector, vvs, config-groups, diagnosis, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: tables, resource: "https://github.com/nsmith-/vibegraph/commit/367c70b", title: "validation: O_W probe rows and tables for gen_standalone_jamps"}
  - {id: merge, resource: "https://github.com/nsmith-/vibegraph/commit/5bbf434", title: "Merge of the Q1 branch (bank_amps, per-graph gate)"}
---
Diagnosis only. Nothing under `vibegraph-lib/src` is committed. Committed with
this report:

- the merge of `origin/amp-signs/same-flavour-4f-contact` into this branch, as
  a merge commit (`5bbf434`), for Q1's `bank_amps` route;
- eight informational `gen_standalone_jamps.py` rows and their JAMP-only tables
  under `validation/madgraph/standalone/` (`*_cw.json`). No test reads them yet.

Every MadGraph run went through `mg5_pinned.sh` from `gen_standalone_jamps.py`;
all eight generation logs read `VERSION 3.7.1`. The SMEFTsim models were staged
by `stage_models` alone (extracted from `build.sh` with `sed`, run under
`pixi run -e madgraph`). Output: `staged model: SMEFTsim_topU3l_MwScheme_UFO
(converted from Python 2)`.

## Correction to the brief and the backlog item

Both say the residual belongs to "O_W's five-vector and momentum-bearing
contact structures". Half of that is wrong.

- **O_W's four-vector contacts are innocent.** `w+ w- > w+ w- NP<=1` and
  `w+ w- > z z NP<=1` agree per diagram and per flow (3.4e-13 and 9.2e-14).
  In the 222-graph row every four-vector-contact diagram agrees, as anchor and
  as source.
- **Half of the defect is not O_W's.** It is the loop-induced photon–Z–Higgs
  vertex `V(a,Z,H)` with Lorentz structure `VVS4` and coupling `GC_303 =
  2i·gHza/vevhat`, coupling order `SMHLOOP`. SMEFTsim carries it at every
  restriction, and the interned SM model does not. That is why the
  Standard-Model `w+ w- > w+ w- z` agrees and this row does not. Its photon–photon
  twin (`GC_299`, gHaa) behaves the same.
- **The five-vector part is right.** O_W's two five-vector contacts (`VVVVV27`
  with `GC_183`, and `VVVVV52` with `GC_181`) carry the wrong sign.
- **The row cannot reach 1e-12 after a fix.** MadGraph's Fortran writes
  `GC_303` at seven significant digits (`KNOWN_FORTRAN_DEVIATIONS` in
  `coupling_oracle.rs` already lists `wpwm_to_wpwmz_cw`/`GC_303` at 1.2e-8). With
  both signs fixed, production measures |M|² 1.14e-11, at the same level as
  the informational `ee_to_zh_smeft` (6.00e-11). The item's `closes_when`
  ("flipped from info to gate") cannot be met as written; see the
  recommendation.

## Root-cause hypothesis

Two class-b convention signs are missing. Each is set by a vertex's class and
role, in the family of note 39's items, and each is distinct from W1's leg-0
anchor rule.

**1. The colourless five-vector contact.** Today a colourless all-vector
contact of any arity ≥ 4 nets −1 as the anchor and +1 as a source. That net is
the kernel's `vertex_contact_sign` −1 (`root_lorentz.rs`), cancelled off the
anchor by `vector_contact_sign` (`root_diagram.rs`). For the five-vector,
MadGraph needs the opposite sign in both roles: +1 as the anchor and −1 as a
source.

Every colourless all-vector vertex in reach fits one pattern. Arity 3
(`VVV5/7/9`) nets +1 as the anchor and −1 as a source. Arity 4 (`VVVV5/7/8/11/14/15`)
nets −1 and +1. Arity 5 (`VVVVV27/52` here, and the WWWW-photon `VVVVV32/57`)
needs +1 and −1. So the sign is (−1)^(n−3) as the anchor and (−1)^(n−2) as a
source.

This is a fitted pattern, not a derivation. O_W's structures also have odd
momentum count at arity 3 and 5 and even count at arity 4, so "arity parity"
and "momentum-count parity" cannot be told apart on these vertices.

**2. The colourless momentum-bearing VVS vertex.** A pure-metric `VVS` term
takes the kernel's `pure_metric` −1 per term when it sinks into the amplitude
or produces the scalar, and none when it produces a vector (`root_lorentz.rs`,
the `Metric` arm of the scalar root). A `VVS` term that carries momenta takes
no build sign in any role. Today only the coloured case is compensated, by
`gluon_scalar_current_sign`, which fires on the scalar-output role alone.

The colourless `VVS4` vertices (`GC_303` photon–Z–Higgs and `GC_299`
photon–photon–Higgs) need −1 at the sink and as a scalar source, and +1 as a
vector source. That is exactly the pure-metric term's role pattern. The
`is_coloured` restriction in `gluon_scalar_current_sign` is where the
colourless case falls out.

Note the coloured and colourless cases then differ at the sink. Note 39 says
the gluon–gluon–Higgs vertex takes no sign as the sink, and the colourless one
here needs −1. I did not re-measure the coloured case.

**Why neither is W1's class.**

- W1's rule fires only where the anchor has a fermion leg and leg 0 is a
  colourless vector. No diagram of `wpwm_to_wpwmz_cw` has a fermion.
- The five-vector sign is relabelling-invariant. The diagram is a single
  vertex, always its own anchor, and it reads −1 under both `w+ w- > w+ w- z`
  and `w- w+ > w- w+ z`.
- The VVS4 sign does move under relabelling, but only because the vertex's
  role moves. Under `w- w+ > w- w+ z`, the six graphs whose `V(a,Z,H)` produced
  a photon current now produce the Higgs current, and the six whose vertex
  produced the Higgs now produce a photon. The −1 follows the role: AMP(59–61)
  and AMP(65–67) swap signs between the two labellings. With candidate 2 both
  labellings agree with MadGraph on all 222 diagrams under the same global −i.
  The anchor-dependence is the symptom of a missing role sign, not a
  leg-0 rule.

## Evidence chain

**Parameter provenance.** Both sides bind the param card in each table,
MadGraph's own `Cards/param_card.dat` for the `vg_cW` card, with every width
zeroed (`zero_widths`), as for every standalone row with a spacelike massive
line. Couplings by name: `coupling_oracle` lists `wpwm_to_wpwmz_cw`/`GC_303` at
1.2e-8 as a MadGraph Fortran deviation. The per-graph modulus measured here is
0.999999988030 on every `GC_303` graph and 0.999999972 on every `GC_299` graph,
and 1.000000000 elsewhere, which is that deviation.

**The probe.** A temporary `#[ignore]` test (reverted; patch in the scratchpad
as `F1/F1-probe-and-candidate.patch`) compiles each enumerated diagram alone,
as a one-diagram `DiagramSet` through `AmplitudeEvaluator::compile`. That is
Q1's route, and it reads production, contacts included. It also dumps each
vertex's interaction, Lorentz structures, couplings, anchor flag and
anchor-rooted output. Bridge: the sum of the one-diagram evaluators against
the full production evaluator is 1.87e-15 on the 222-graph row.

**The pairing is a bijection from the values alone** (scratch `F1-pairing.py`).

1. Each of our diagrams goes to the MadGraph graphs whose `c_k·AMP_k` has
   overlap 1 − 1e-10 with it over every (point, helicity) entry.
2. Where several graphs qualify, the one whose ratio has unit modulus is
   chosen. This happens for the photon and Z s-channel exchanges of one
   topology, which have `q² = s` at every RAMBO point and so are exactly
   proportional. The chosen graph is unique in every case.
3. Checked afterwards: the coupling of MadGraph's amplitude-closing call
   appears among our diagram's couplings, for every pair.

Results:

| process | diagrams = graphs | overlap-1 ties resolved | relative −1 on |
|---|---|---|---|
| `w+ w- > w+ w- z NP<=1` | 222 | 38 | d0, d1 (five-vector); d102–104, d148–150 (`V(a,Z,H)` → H) |
| `w+ w- > w+ w- a NP<=1` | 222 | 44 | d0, d1 (five-vector); d102–107, d148–153 (`V(a,Z,H)`, `V(a,a,H)` → H) |
| `e+ e- > w+ w- w+ w- NP<=1` | 688 | 0 | d0–d3 (five-vector as source) |
| `w+ w- > a z NP<=1` | 14 | 0 | d3 (`V(a,Z,H)` → H) |
| `a z > w+ w- NP<=1` | 14 | 0 | d3 (`V(a,Z,H)` the anchor) |
| `a a > w+ w- NP<=1` | 14 | 0 | d3 (`V(a,a,H)` the anchor) |
| `w+ w- > w+ w- NP<=1` | 25 | 10 | none |
| `w+ w- > z z NP<=1` | 14 | 0 | none |

The per-diagram residual of every pair is at most 2.9e-14 on the 222-graph row
and 1.5e-13 on `e+ e- > w+ w- w+ w-`. Every relative constant is exactly ±1
(asserted). So the defect is a pure per-diagram sign pattern, and the
magnitudes are right.

The `V(a,Z,H)` and `V(a,a,H)` diagrams that produce a vector current (`>a`,
`>Z`) take +1: d96–98 and d142–144 in the 222-graph row, and d408–452 in
`e+ e- > w+ w- w+ w-`.

**Arithmetic verification on the dumps** (scratch `F1-verify.py`). Flip the
classes, recontract, fit G, compare per flow:

| `w+ w- > w+ w- z` | \|G\| − 1 | worst flow |
|---|---|---|
| as dumped | 6.50e-3 | 2.40e-1 |
| five-vector flipped | −4.86e-4 | 1.36e-3 |
| VVS4 → H flipped | 6.98e-3 | 2.39e-1 |
| both | −4.3e-12 | 1.97e-11 |
| both, `GC_303`'s measured 1.2e-8 divided out | 4.4e-16 | 2.0e-14 |

`w+ w- > a z` with the VVS4 flip and `GC_303` divided out: 6.7e-15.

**Production check of the candidate.** The two rules went in temporarily as
one more factor in `compile_single_diagram`'s `fermi_sign`, switched by an
environment variable (`F1_RULE`), so one build ran every combination.

- `five`: −1 per colourless five-vector vertex, in either role.
- `vvs`: −1 per colourless `VVS` vertex whose every structure carries a `P`,
  when it is the anchor or its anchor-rooted output is the scalar.

`cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture`, the
`wpwm_to_wpwmz_cw` line (MadEvent table, grid and event points):

| rule | \|M\|² grid / event | per-flow | JAMP2 | \|G\| − 1 |
|---|---|---|---|---|
| none | 2.79e1 / 1.05e1 | 7.48e-1 | 2.55e1 | 1.7e-2 |
| `five` | 8.88e-4 / 2.01e-3 | 2.05e-3 | 2.01e-3 | 5.3e-4 |
| `vvs` | 2.80e1 / 1.05e1 | 7.48e-1 | 2.55e1 | 1.8e-2 |
| `five vvs` | 1.14e-11 / 2.11e-11 | 2.53e-11 | 1.63e-11 | 6.2e-12 |

The other 48 rows' printed lines were identical under every rule, digit for
digit (sorted-line diff). Every run reported `test result: ok. 49 passed`. The
row is declared `info`, so it passes in all four.

`standalone_jamps` (the eleven existing rows): `test result: ok. 11 passed`
under every rule, each printed line unchanged.

Production per flow on the new probe tables (`five vvs`, against the same MadGraph tables):

| process | none | `five vvs` |
|---|---|---|
| `w+ w- > w+ w- z` | 2.40e-1 | 1.97e-11 |
| `w+ w- > a z` | 1.41e-2 | 8.5e-11 |
| `a z > w+ w-` | 8.95e-2 | 5.4e-10 |
| `e+ e- > w+ w- w+ w-` | 3.68e-1 | 6.4e-10 |
| `w+ w- > w+ w- a` | 8.16e-1 | 3.4e-10 |
| `a a > w+ w-` | 2.41e-2 | 3.3e-10 |
| `w+ w- > w+ w-`, `w+ w- > z z` | 3.4e-13, 9.2e-14 | unchanged |

Every residual left is the `GC_303`/`GC_299` transcription; the pair table
shows unit modulus elsewhere.

**Predictions, made after both rules were formed:** `w+ w- > w+ w- a NP<=1`
and `a a > w+ w- NP<=1`. Their tables were generated after the rules. Under
the rules every diagram has relative sign +1 (14 and 1 diagrams flip without
them).

What was fitted:

- The five-vector rule took its anchor role from the 222-graph row and its
  source role from `e+ e- > w+ w- w+ w-`.
- The VVS rule took the scalar-source role from `w+ w- > a z` and the
  222-graph row, the vector-source role from the 222-graph row, and the sink
  role from `a z > w+ w-`.

**Relabelling.** `w- w+ > w- w+ z NP<=1` at the table's points with the legs
permuted (`1,0,3,2,4`) and `z a > w- w+` (`1,0,3,2`). MadGraph's `AMP()` is
relabelling-invariant (W1).

- Baseline: our per-graph signs move on exactly the twelve `V(a,Z,H)` graphs
  whose role moves, and the five-vector stays −1.
- With `five vvs`: no relative −1 on any of the 222 graphs or the 14 graphs,
  with the same global phase (−i) in both labellings.

## What would falsify it

- **A five-vector contact in another role pattern.** A colourless five-vector
  that agrees as the anchor or disagrees as a source under the rule. The rule
  rests on the 222-graph row (anchor), `e+ e- > w+ w- w+ w-` (source) and
  `w+ w- > w+ w- a` (anchor, a prediction).
- **A coloured five-vector.** `O_G`'s five-gluon contact under `vg_cG`, for
  example `g g > g g g NP<=1`. Gluon contacts take no contact sign at arity 4,
  and the rule is stated for colourless contacts only, so a five-gluon contact
  agreeing today without a −1 is expected. One agreeing only with a −1 means
  the arity rule is wider than measured.
- **A colourless VVS vertex mixing pure-metric and momentum structures**, at
  the sink or producing the scalar. SMEFTsim's `V_55` (`W- W+ H`:
  `VVS1, VVS3, VVS4`) under a `cHW`/`cHWB` card is one. The per-vertex
  candidate fires only when every structure carries a momentum. A mixed vertex
  would then sign its metric terms −1 and its momentum terms +1. That relative
  sign inside one MadGraph routine is almost certainly wrong. A per-term rule
  (below) predicts −1 on both. `w+ w- > h z NP<=1` or `w+ w- > h a` under
  such a card decides it.
- **A coloured VVS at the sink.** If `g g > g h NP<=1` under `vg_cHG` turns
  out to need −1 at the gluon–gluon–Higgs sink after all, the colourless and
  coloured rules collapse into one.
- **A banked σ row moving.** Predicted: none (below).

## Fix recommendation for F2

1. **Five-vector.** In `root_diagram.rs`, give a colourless all-vector contact
   with five legs one more −1, in either role. The smallest change counts such
   vertices in `vector_contact_sign`'s parity. A more principled form makes the
   kernel `vertex_contact_sign` −1 (`root_lorentz.rs`) an arity sign,
   (−1)^(n−3), with `vector_contact_sign` unchanged. That form must be checked
   against the gluon contacts: `vector_contact_sign` already cancels the kernel
   sign on every coloured contact, so a coloured five-gluon contact would then
   carry +1. That is unmeasured.
   Doc comment: the anchor/source pattern of colourless all-vector vertices is
   (−1)^(n−3) / (−1)^(n−2) at arities 3, 4 and 5, measured per graph against
   MadGraph.

2. **VVS.** Sign colourless momentum-bearing `VVS` structures like pure-metric
   ones: −1 at the amplitude sink and producing the scalar, none producing a
   vector.
   - Per-term form (preferred): in `root_lorentz.rs`'s scalar root, apply the
     once-per-term −1 to a colourless `VVS` term whether or not it carries `P`.
     Coloured vertices keep `gluon_scalar_current_sign` as today. The kernel
     may not see colour; if it does not, the diagram-level form below is the
     fallback.
   - Diagram-level form (what was measured): the `vvs` predicate above, as a
     sibling of `gluon_scalar_current_sign`. It is exact on every vertex in
     reach and wrong-by-construction on a mixed vertex.
   - Prefer the form that is right on a mixed vertex, and add the
     `w+ w- > h z` probe of the falsifier before landing either.

3. **Gating, honestly.** `GC_303`'s seven-digit literal keeps the full row at
   ~2e-11. Either of these lets the row gate:
   - (a) Gate the standalone `wpwm_to_wpwmz_cw` table per graph (`bank_amps`)
     and exempt the twelve `GC_303` graphs' modulus by name, the way
     `KNOWN_FORTRAN_DEVIATIONS` names the coupling.
   - (b) Add an `SMHLOOP=0` standalone row (`w+ w- > w+ w- z NP<=1 SMHLOOP=0`;
     the parser's support for that order is unchecked) that gates the
     five-vector at 1e-12. Keep the amplitude row informational, attributed
     the way `ee_to_zh_smeft`'s cell is.
   - The backlog item's `closes_when` needs restating either way.

4. **Rows to add as tests.** The eight tables committed here:
   - every one but `wpwm_to_wpwm_cw` and `wpwm_to_zz_cw` disagrees before the
     fix;
   - the four that carry `GC_303`/`GC_299` cannot pass 1e-12 without the
     exemption of 3(a).
   Also a hermetic relabelling-invariance check on
   `w+ w- > w+ w- z` / `w- w+ > w- w+ z`, which fails today on the twelve
   `V(a,Z,H)` graphs and needs no MadGraph.

5. **Controls F2 must run, outside the fit:**
   - `standalone_jamps`;
   - `amplitude_oracle` (every other line unchanged);
   - `helas::eval::renumbering`;
   - `rooting_soundness`;
   - `probe_bridge`;
   - `pixi run --skip-deps validate`.

## The partition-pairing route

Yes: the per-diagram `AMP()` data give a pairing derived from MadGraph's own
values, not fitted to `config_groups`.

- **The same numbering as MadEvent.** The standalone export's
  amplitude-closing `CALL` lines are textually identical to the MadEvent
  `matrix1_orig.f` of the amplitude row (`diff`, no output). The only
  difference anywhere in the HELAS calls is the width argument spelling
  (`ZERO` against `FK_ZERO`). So the standalone graph index is the amplitude
  row's graph index.
- **The partition.** Under the derived pairing, our 21 `config_groups` mapped
  to MadGraph indices equal MadGraph's 21 `AMP2` accumulators exactly (set
  equality, scratch check).
- **It is not fitted to that check.** The pairing reads only `|ratio| = 1`
  and the overlap. The photon/Z ties it resolves fall in *different*
  accumulators (AMP 15–21 against 22–28, and 32–38 against 39–45), so the
  partition check is not blind to that resolution. The other five ties
  (AMP 49–53 against 54–58) sit in no accumulator, so the partition cannot see
  them.

The derived order is our diagram → MadGraph graph, 0-based, which is the
`MG_DIAGRAM_ORDER` convention:

```
0, 1, 52, 51, 50, 49, 48, 56, 55, 57, 54, 53, 108, 107, 106, 105, 104, 112, 111, 113, 110, 109, 151, 150, 152, 149, 148, 188, 187, 191, 190, 189, 193, 192, 196, 195, 194, 201, 198, 197, 200, 199, 203, 202, 206, 205, 204, 208, 207, 211, 210, 209, 216, 213, 212, 215, 214, 218, 217, 221, 220, 219, 37, 34, 33, 36, 35, 32, 31, 44, 41, 40, 43, 42, 39, 38, 47, 46, 45, 20, 17, 16, 19, 18, 15, 14, 27, 24, 23, 26, 25, 22, 21, 30, 29, 28, 4, 3, 2, 7, 6, 5, 10, 9, 8, 13, 12, 11, 93, 90, 89, 92, 91, 88, 87, 100, 97, 96, 99, 98, 95, 94, 103, 102, 101, 76, 73, 72, 75, 74, 71, 70, 83, 80, 79, 82, 81, 78, 77, 86, 85, 84, 60, 59, 58, 63, 62, 61, 66, 65, 64, 69, 68, 67, 141, 138, 137, 140, 139, 146, 143, 142, 145, 144, 147, 134, 135, 136, 131, 132, 133, 124, 121, 120, 123, 122, 129, 126, 125, 128, 127, 130, 117, 118, 119, 114, 115, 116, 159, 156, 155, 158, 157, 154, 153, 166, 163, 162, 165, 164, 161, 160, 169, 168, 167, 180, 177, 176, 179, 178, 185, 182, 181, 184, 183, 186, 173, 174, 175, 170, 171, 172
```

How F2 should bank it:

- Bank `AMP()` for the standalone `wpwm_to_wpwmz_cw` row (`bank_amps`, with
  `MG_GRAPH_DIAGRAMS` one diagram per graph from this order), so the pairing
  is gated by the per-graph level rather than asserted.
- Then enter the same order in `amplitude_oracle`'s `MG_DIAGRAM_ORDER` and drop
  `KNOWN_CONFIG_PAIRING_UNAVAILABLE`.
- The exemption is two-way against the *MadEvent* table, which still banks no
  `AMP()`, so F2 must point its pairing at the standalone measurement
  explicitly.
- Size: `bank_amps` at all four points is about 10 MB of JSON. One or two
  points (2.4–4.8 MB) keep the pairing over-determined, since the photon/Z
  ties are resolved by modulus, not by kinematics.

## Banked σ rows a fix would move

None measured or predicted.

- `wpwm_to_wpwmz_cw` has no σ or sample cell (its run card is refused).
- `ee_to_zh_smeft` (σ gated) carries `V(a,Z,H)` only as a vector source, where
  the rule does not fire. Its amplitude line was unchanged under the candidate.
- `ee_to_ttx_smeft` and the other SMEFTsim σ rows reach neither a five-vector
  nor a colourless momentum `VVS`. Every other amplitude row was unchanged to
  the printed digit.

## Gates

Every cargo invocation ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the committed tree
(probe and candidate reverted; `git diff --stat vibegraph-lib/src` empty).

- `cargo test --workspace`: exit 0; summed `test result` lines give 1396
  passed, 0 failed, 16 ignored.
- No Rust source changed, so `fmt`/`clippy` and `pixi run --skip-deps
  validate` were not rerun for this commit. The production measurements above
  ran with the temporary candidate, not on the committed tree.

## Blind spots

- **Global phase.** The fitted G (−i here) is blind to a sign common to every
  diagram. Both rules are fixed only up to that, as W1's are.
- **Arity or momentum parity.** O_W's colourless structures have odd momentum
  count exactly at odd arity, so the five-vector rule cannot tell "arity" from
  "momentum parity". A structure with an odd momentum count at arity 4 would.
- **Mixed VVS vertices.** Not reached: the `vg_cW` card zeroes `V_55`'s
  momentum structures.
- **Coloured counterparts.** The five-gluon contact and the gluon–gluon–Higgs
  sink were not re-measured.
- **Points.** Three or four RAMBO points per table at one √s. The probes
  compare per graph, where the kinematics only have to separate graphs; the
  s-channel photon/Z ties show where they cannot.
- **The pairing's tie resolution reads modulus.** If two graphs were both
  proportional and of equal modulus, the pairing would be ambiguous (asserted
  absent here: exactly one unit-modulus candidate per diagram).

## Found

- **The loop-induced `V(a,Z,H)`/`V(a,a,H)` vertices are a class no gated row
  reaches in a sign-bearing role.** Evidence: the six probe rows above. Today
  only `ee_to_zh_smeft` carries `GC_303`, as a vector source. What would close
  it: the `vvs` rule plus the `a z > w+ w-` and `a a > w+ w-` rows gated per
  graph.
- **`GC_299` belongs beside `GC_303` in `KNOWN_FORTRAN_DEVIATIONS`.** The
  per-graph modulus is 0.999999972 (2.8e-8) on every `GC_299` graph. No banked
  amplitude row carries it, so `coupling_oracle` does not list it. What would
  close it: a coupling-oracle row for a process with photon–photon–Higgs.
- **A colourless mixed `VVS` probe is missing** (see the falsifier). What
  would close it: `w+ w- > h z NP<=1` (or `> h a`) under a card with `cHW`,
  `cHB` and `cHWB`, with per-graph `AMP()`.
- **The five-gluon contact is unpinned.** What would close it:
  `g g > g g g NP<=1` under `vg_cG`. It is multi-flow, so it needs a per-graph
  route the single-flow `bank_amps` does not have.
- **`gen_standalone_jamps.py --amps` does not record the graph's HELAS call.**
  Matching needed `matrix.f` parsed separately. What would close it: bank the
  closing routine and coupling name beside each `AMP()`.
- **The relabelling oracle still has no hermetic test.** W1 found the same.
  Here it is also the cheapest detector of a missing role sign: the baseline
  fails it on twelve graphs of `w+ w- > w+ w- z`.
