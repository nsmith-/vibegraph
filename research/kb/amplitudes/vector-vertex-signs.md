---
type: Physics Convention
title: Vector-vertex convention signs
description: "MadGraph reads a vertex's momenta outgoing and vibegraph incoming, so a vertex reads (−1)^{n_P}·ε_slot against MadGraph's COUP·L. An all-boson vertex whose legs are all self-conjugate takes −1 iff its Lorentz terms carry an odd number of momenta (boson_momentum_sign): the triple-gluon vertex, the scalar–three-gluon contacts of O_HG/O_HGtil (an ALP's a·GG̃), O_G's five-gluon contact. A charged VVV's W-slot exchange cancels its momentum sign in the kernel. Even-parity contacts and VVS vertices take none; a scalar line beside one takes its propagator phase. Why, and what pins it."
status: draft
tags: [sign-conventions, vector-vertices, qcd, electroweak, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
sources:
  - {id: b-report, resource: "../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report: no role is needed; the triple-gluon constant; the mutations"}
  - {id: t1-report, resource: "../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report: the triple-gluon −1 is the slot exchange every other triple-vector vertex gets from the crossing"}
  - {id: code-tg, resource: "vibegraph-lib/src/helas/eval/triple_gluon.rs", title: "helas::eval::triple_gluon (the factor-by-factor tests against MadGraph AMP())"}
  - {id: code-rootdiag, resource: "vibegraph-lib/src/helas/eval/root_diagram.rs", title: "boson_momentum_sign, momentum_parity_sign, propagator_phase_sign, and the vertex-class unit tests"}
  - {id: mg-vxxxxx, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/template_files/aloha_functions.f#L798-L873", title: "HELAS vxxxxx: the stored momentum is nsv·p, nsv = −1 on an initial leg"}
  - {id: ha2-report, resource: "../sprints/cms-gridpack-coverage/sessions/H-A2-report.md", title: "H-A2 report: every all-boson vertex of self-conjugate legs takes (−1)^{n_P} against MadGraph"}
  - {id: code-standalone, resource: "vibegraph-lib/tests/standalone_jamps.rs", title: "standalone_jamps (per-flow JAMPs against MadGraph standalone)"}
  - {id: code-pt, resource: "vibegraph-lib/tests/gluon_parke_taylor.rs", title: "gluon_parke_taylor (hermetic MHV check)"}
  - {id: r1-report, resource: "../sprints/amplitude-signs/sessions/R1-report.md", title: "R1 report: the u u~ > g g probe and the vertex censuses"}
  - {id: n39-wrong, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L16-L53", title: "Note 39 §1 (the Ward-identity failure of u u~ > g g)"}
  - {id: n39-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L73-L123", title: "Note 39 §3 (the standalone JAMP oracle)"}
  - {id: n35-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L432-L474", title: "Note 35 §3.5 (the ALOHA VVV normalisation)"}
---

# Vector-vertex convention signs

A diagram's `fermi_sign` carries one sign for boson vertices:
`boson_momentum_sign` (`helas/eval/root_diagram.rs`)[^code-rootdiag], `−1` per
all-boson vertex whose legs are all self-conjugate and whose Lorentz terms carry
an odd number of momenta[^ha2-report]:

| vertex | `n_P` per term | conjugate pair | convention sign | against MadGraph's `COUP · L` |
|---|---|---|---|---|
| triple-gluon VVV (`VVV1`) | 1 | no | `−1` | the momentum sign, which nothing else supplies |
| `O_HG`'s and `O_HGtil`'s Higgs–three-gluon contacts (`O_HGtil`'s through `Epsilon`; an axion-like particle's `a·GG̃` contact has the same structure) | 1 or 3 | no | `−1` | as the triple-gluon vertex |
| `O_G`'s five-gluon contact | odd | no | `−1` | as the triple-gluon vertex |
| `O_HG`'s Higgs–four-gluon contact; the Higgs–gluon-pair VVS; `O_G`'s four-gluon contacts; the SM four-gluon contact | 0 or 2 | no | none | agrees |
| VVS, colourless (ZZH, the loop-induced γZH and γγH) | even | no | none; a scalar line at it takes `−1` as every internal scalar line does | agrees |
| charged VVV (γW⁺W⁻, ZW⁺W⁻, SMEFT VVV, `O_W`'s triple-gauge structures) | odd | yes | none | the slot exchange's `−1` cancels the momentum sign in the kernel |
| colourless contacts of four or five vectors (WWZZ, WWAA, WWWW, `O_W`'s five-vector contacts) | — | yes | none | agrees |

The other factors of `fermi_sign` are in
[convention-sign-inventory](convention-sign-inventory.md).

## The derivation: `(−1)^{n_P} · ε_slot`

ALOHA evaluates every vertex as `COUP · L` with every momentum read outgoing:
HELAS stores an external momentum as `nsv·p`, with `nsv = −1` on an initial
leg[^mg-vxxxxx]. The kernels here read every momentum incoming: an off-shell leg's `P` is minus
the sum of the inputs (`LorentzEvalNode::POut`). So each `P` of a Lorentz term is
minus MadGraph's, and a term with `n_P` momenta is `(−1)^{n_P}` times
MadGraph's. The incoming reading also puts each leg in its antiparticle's slot
of the UFO vertex. That exchanges the two slots of every conjugate pair and
multiplies the structure by its parity `ε_slot` under the exchange. Per vertex,
then, this side reads `(−1)^{n_P} · ε_slot` times MadGraph's `COUP · L`[^ha2-report].

- **All legs self-conjugate.** Nothing is exchanged, so `ε_slot = +1` and the
  vertex reads `(−1)^{n_P}`. `boson_momentum_sign` supplies `−1` where `n_P` is
  odd.
- **A conjugate pair.** The kernel already evaluates the exchanged structure. On
  `e+ e- > w+ w-` the `a W- W+` rays are `[p, 3, 4]`: the final W⁺ stands in the
  W⁻ slot, where MadGraph's `VVV1_0(W5, W4, W3, …)` puts it in the W⁺ slot.
  `VVV1` is odd under exchanging two slots with their momenta. So is every
  triple-vector structure the SM and SMEFTsim put on a W pair (`VVV1`–`VVV7` and
  `VVV9` on `a W- W+` and `W- W+ Z`, by a numeric parity check at random
  momenta; `VVV3` vanishes identically). So `ε_slot = −1 = (−1)^{n_P}`, and the
  charged vertex agrees with MadGraph with no factor: `e+ e- > w+ w-` matches
  MadGraph's `AMP()` diagram for diagram under one constant[^t1-report]. At a
  fermion vertex the slot exchange is undone, in the colour by `convert_expr`'s
  transpose of each `T` and in the bilinear by the slot binding
  `vertex_root_signs` reads.
- **Parity, not count.** In a Lorentz-scalar term of an all-boson vertex every
  vector leg and every `P` carries one free index, and `Metric` and `Epsilon`
  contract an even number, so `n_P` has the parity of the vector legs in every
  term. Only the parity is uniform within a vertex, not the count. SMEFTsim's
  Higgs–three-gluon vertex mixes terms with 1 and 3 momenta (`O_HG` beside the
  loop-induced `gHgg` structures), and its Higgs–four-gluon vertex mixes terms
  with 0 and 2. `momentum_parity_sign` counts per term, a `P(μ,i)**2` counting
  twice. A vertex whose terms disagree on the parity is not such a scalar, and
  it is refused with `RootLorentzError::UnsupportedVertex` naming the vertex.
- **Colour plays no part.** A colourless vertex of self-conjugate bosons with odd
  `n_P` takes `−1` too (a 2HDM `A Z h` vertex would be one). No model in reach
  has one in unitary gauge, so that case is derived, not measured
  ([colourless-self-conjugate-vvv-sign-unoracled](../backlog/validation/colourless-self-conjugate-vvv-sign-unoracled.md)).

A vector line beside any of these has no phase to make up: ALOHA's vector
propagator carries `−i(g − qq/m²)/D`, the phase of the evaluator's own. What
differs at a VVS vertex is the scalar line beside it, whose propagator ALOHA
writes `+i/D` against the evaluator's `−i/D`. That `−1` belongs to the line
(`propagator_phase_sign`), whichever vertex produces or consumes it[^b-report].

ALOHA's VVV normalisation is uniform: it emits `VERTEX = −i·COUP·L` for every VVV
structure, so SMEFTsim's real `GC_7 = G` against the SM's `i·G` does not change
the convention[^n35-e2].

## The triple-gluon vertex

The triple-gluon `−1` is the momentum sign, and it is not in the colour factor
or the coupling[^t1-report]:

- **Colour.** Every colour coefficient of the triple-gluon diagram equals
  MadGraph's JAMP coefficient term by term (`u u~ > g g`: `(−i, +i)` on
  `T(3,4,2,1)`/`T(4,3,2,1)`, as in `P1_qq_gg`'s `JAMP(1) = −i·AMP(1) + AMP(2)`).
  On `g g > u u~` and `g g > t t~` every coefficient is MadGraph's times `−1` on
  every diagram alike, which is MadGraph's fermion factor riding in its
  coefficients. `f` passes through `convert_expr` untouched, and `f_to_traces`
  is MadGraph's `f = −2i Tr(abc) + 2i Tr(cba)`.
- **Slot order.** The triple-gluon rays (`[3, 4, p]` in `u u~ > g g`) and
  MadGraph's (`[p, 3, 4]`, `VVV1P0_1(W3, W4)`) differ by a permutation that acts
  on `f` and on `VVV1` together, since both are read positionally and both are
  antisymmetric; their product cannot change.
- **Coupling.** `GC_10 = −G` on both sides, from the same UFO and card, and
  MadGraph passes it unmodified.

The GF(2) solve over every MadGraph-verified row (3495 diagrams) gave the same
constant as the unique minimal one once the propagator phases and the bilinear
readings were fixed[^b-report].

Where every other vertex carries an even number of gluons, as in the SM, only
the triple-gluon sign's ratio to the quark–gluon vertex is observable. Counting
gluon line ends, `n_qg + n_ggg ≡ n_ext_g (mod 2)`. So a process's external
gluons fix the parity of the two counts together, and `−1` per quark–gluon
vertex is the same convention up to a sign common to the process. Vertices of
odd gluon number break that identity: `O_G`'s five-gluon contact and `g g g H`
from `O_HG`/`O_HGtil`, both banked (below), and `g g g H H` and `q q̄ g H` from
`O_HG` and the dipole operators, in no row. Beside the banked ones the two
phrasings differ diagram by diagram, and MadGraph agrees with the momentum
rule, not with `−1` per quark–gluon vertex.

The relative sign it sets is visible without MadGraph. With the triple-gluon
vertex at `+1`, the `u u~ > g g` helicities with equal gluon helicities, which
the Ward identity sets to zero, come out at `|M|² ≈ 105` at
`√s = 500`[^n39-wrong]. On the current code with the sign set to `+1` they read
`9.6e1` at `√s = 500`, `cos θ = 0.3`, against `0` (to `3e-31`) with
it[^r1-report].

### Details that matter

- **The rule reads the momentum parity, not the arity.** A contact of four or
  more vectors can carry momenta (`O_G`'s four- and five-gluon structures,
  SMEFTsim's `VVVV2`/`VVVV3`, `O_W`'s five-vector structures). Of these, the
  self-conjugate ones with odd parity (`O_G`'s five-gluon contact) take `−1`, and
  the even ones (`O_G`'s four-gluon contacts) take none. The W-pair contacts have
  a conjugate pair and take none.
- **A pure SM-gluon process cannot see the triple-gluon sign.** On a tree of
  three- and four-gluon vertices with `n` external gluons, `V₃ + 2V₄ = n − 2`, so
  the triple-gluon count has the same parity in every diagram and the sign is
  global. A quark line, a Higgs–three-gluon contact or `O_G`'s five-gluon contact
  beside a gluon vertex is what pins it.

## What pins it, and what each gate cannot see

`validation/madgraph/gen_standalone_jamps.py` generates a process with MadGraph
`output standalone` (pinned 3.7.1), patches `MATRIX` to copy `AMP()` and
`JAMP()` into a COMMON block, and dumps per RAMBO point and per helicity the
colour-summed |M|² and every JAMP, and, on a single-flow row that asks for them
(`bank_amps`), every AMP, at the same param card and momenta as ours. Standalone output keeps the width of a spacelike propagator
while MadEvent (and this crate) zero it (`zerowidth_tchannel`). Through a
massive t-channel line that is a 0.1–0.5% per-helicity difference, not a sign,
so such rows compare with every width zero on both sides (`zero_widths`)[^n39-oracle].
A spacelike Higgs between two of SMEFTsim's loop-induced Higgs–gluon vertices
leaves a `Γ_H/M_H`-sized residual (1e-9 of the largest amplitude on
`g g > h g g`), and those rows run at zero widths too[^ha2-report].

| gate | what it pins | blind to |
|---|---|---|
| `tests/standalone_jamps.rs`[^code-standalone] (hermetic, committed tables in `validation/madgraph/standalone/`) | per-helicity, per-flow JAMPs and \|M\|² against MadGraph standalone, one fitted `G` with `\|G\| = 1`, at 1e-12; on the rows of `ROUNDED_COUPLING_GRAPHS` the named graphs built on MadGraph's seven-digit `GC_303`/`GC_299` are divided by their registered modulus deviation first, and every graph of such a row must also match on its own scale (`GRAPH_RATIO_TOL = 1e-10`). Vector-vertex rows: `gg_to_ggg`, `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`, `ee_to_wpwmz`, `wpwm_to_wpwm`, `ttx_to_gg_chg`, the odd- and even-momentum contacts `gg_to_hg_chg` (`O_HG`'s Higgs–three-gluon contact, `−1`), `gg_to_hg_chgtil` (`O_HGtil`'s, through `Epsilon`, `−1`), `gg_to_hgg_chg` (`O_HG`'s Higgs–four-gluon contact, `+1`) and `gg_to_ggg_cg` (`O_G`'s five-gluon contact, `−1`, beside its four-gluon contacts), the W-pair rows (`wpwm_to_epem`, `wmwp_to_emep`, `wpwm_to_uux`, `wpem_to_zve`, `aem_to_wmve`, `wpwm_to_epemz`, `wpwm_to_uuxg`), the VVS rows `wph_to_tbx`, `hwp_to_tbx` and `zz_to_ttx` (with `hh_to_ttx`, which has no vector vertex, as their control), `O_W`'s rows, and the `t t~ + jet` rows | the phase of `G`; compensating errors in diagrams that reach the same flows with the same weight; kinematics outside the banked points (three per row, four on two rows, eight on one) |
| `tests/gluon_parke_taylor.rs`[^code-pt] (hermetic, no MadGraph) | 5 and 6 gluons, every MHV and anti-MHV configuration: `J_σ·⟨σ1σ2⟩…⟨σnσ1⟩` constant over flows and `\|J\|·\|cyclic\|/\|⟨ij⟩\|⁴` constant over configurations; tolerance 1e-10 in units of the point's largest flow | the triple-gluon sign (global on a pure-gluon process); global phase and normalisation; NMHV; anything with a quark |
| `tests/amplitude_oracle.rs` | per-diagram and per-flow values on the banked rows | anything those rows do not reach |
| `root_diagram::tests` (hermetic, no MadGraph)[^code-rootdiag] | the rule per vertex class: `the_triple_gluon_vertex_takes_minus_one`, `a_charged_triple_vector_vertex_takes_none` (γW⁺W⁻, ZW⁺W⁻), `an_odd_momentum_contact_takes_minus_one` (Higgs–three-gluon under `vg_cHG` and `vg_cHGtil`, five-gluon under `vg_cG`), `an_even_momentum_vertex_takes_none` (Higgs–four-gluon, Higgs–gluon-pair, the SM and `O_G` four-gluon contacts), `a_vertex_of_mixed_momentum_parity_is_refused` (the refusal reaches `compile_single_diagram`); each check fails if the model lacks the vertex it names | whether the rule is MadGraph's (the standalone rows' job); colour (every odd-parity vertex they reach is coloured) |

Dropping the triple-gluon `−1` fails `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`,
`ttx_to_gg_chg`, the five `t t~ + jet` rows and `uux_to_ttxg_4f` in
`standalone_jamps`, and `gg_to_ttx` with its two SM-limit rows in
`amplitude_oracle`[^b-report]. Restricting the rule to three-leg vertices fails
`an_odd_momentum_contact_takes_minus_one` and all four contact rows
(`gg_to_hg_chg`, `gg_to_hg_chgtil`, `gg_to_hgg_chg`, `gg_to_ggg_cg`). Dropping the
self-conjugate condition, which puts `−1` on the charged triple-vector vertices
too, fails `a_charged_triple_vector_vertex_takes_none`,
`a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel` and 9
`standalone_jamps` W-pair rows[^ha2-report]; the same mutation on the
three-vector predicate failed 11 `standalone_jamps` rows and 5 `amplitude_oracle`
rows (`ee_to_wpwm`, `ee_to_wp0wm`, `ee_to_wp0wmt`, `ud_to_epemud_qcd0`,
`wpwm_to_wpwmz_cw`)[^t1-report].

Neither MadGraph gate sees which factor carries the triple-gluon sign. Moving it
into the colour (`f_to_traces` negated, `boson_momentum_sign` at `+1`) or into the
coupling (`GC_10` negated) keeps every SM row of both suites green. The tests of
`helas::eval::triple_gluon`[^code-tg] pin the factors separately on the banked
`g g > t t~` and `e+ e- > w+ w-` `AMP()` tables:

| test | asserts | blind to |
|---|---|---|
| `the_triple_gluon_diagram_agrees_with_madgraph` | each diagram is `G·c·AMP` under one `G` with `\|G\| = 1`, at 1e-12; `boson_momentum_sign` is `−1` on the triple-gluon diagram | a sign shared by every diagram; how the value splits into factors |
| `the_triple_gluon_diagram_has_madgraphs_colour` | our colour coefficient over MadGraph's is one unit constant over every `(diagram, flow)` | a colour sign common to every diagram; anything outside colour |
| `the_triple_gluon_diagram_reads_minus_one_outside_its_colour` | with colour and `boson_momentum_sign` divided out, the quark exchanges are `K·AMP` and the triple-gluon diagram `−K·AMP(1)` | the split among kernel, coupling, propagator phases and the rest of `fermi_sign`; a flip of `boson_momentum_sign` alone |
| `a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel` | on `e+ e- > w+ w-`, colour divided out, every diagram is `K·AMP` under one `K`; each final-state W stands in its antiparticle's slot | the split inside the rest; vertices other than `a W- W+` and `W- W+ Z` |

Each also inherits the oracle's limits: one process, six banked points and the
helicities they carry. Which gate pins which
factor across the whole convention is
[validation/convention-channel-coverage](../validation/convention-channel-coverage.md);
the oracle family is [validation/amplitude-oracle](../validation/amplitude-oracle.md).

## Open

- **A colourless vertex of self-conjugate bosons with odd `n_P`** takes `−1` by
  the derivation; no row measures it, and reinstating a colour condition would
  fail no test
  ([colourless-self-conjugate-vvv-sign-unoracled](../backlog/validation/colourless-self-conjugate-vvv-sign-unoracled.md)).
- **The W-pair structures' oddness** under the slot exchange rests on scratch
  parity checks (T1's, and an independent one in the R1 review, which also finds
  `VVV5` odd on SMEFTsim's `W1` pairs), not a committed test. A vertex with a
  conjugate pair whose `ε_slot` differs from `(−1)^{n_P}` would come out wrong
  with no refusal: the code assumes the identity rather than computing
  `ε_slot` from the structure.
- **The fermion dipole.** The naive `(−1)^{n_P}` would also put `−1` on a
  one-momentum fermion vertex (`σ^{μν}q_ν`, SMEFTsim's `P(-1,3)·γγ` FFV
  structures). The dipole rows (`ee_to_ttx_dipole`, `ll_to_qqx_toy_dipole`, the
  `ctZ`/`ctA` chains of `ee_to_ttx_smeft`) agree with MadGraph with no such
  factor. Whether the bilinear's slot reversal supplies the cancelling `ε_slot`
  is not established; `boson_momentum_sign` does not count fermion
  vertices[^ha2-report].
- **`g g g H H` and `q q̄ g H`** (vertices of odd gluon number from `O_HG` and the
  dipole operators) are in no row.
- **No committed table banks `AMP()` for `u u~ > g g` or `g g > u u~`**, so the
  factor tests run on `g g > t t~`
  ([standalone-amps-single-flow-only](../backlog/validation/standalone-amps-single-flow-only.md)).
- **A VSS vertex** (scalar–scalar–vector, one momentum) appears in no model in
  reach. With every leg self-conjugate (a 2HDM `A Z h`) the rule gives it `−1`;
  with a charged scalar pair the slot exchange cancels the momentum sign and it
  takes none. Either way each scalar line beside it takes `−1`.
- **`wpwm_to_wpwmz_cw`** (`O_W` through a five-vector vertex) agrees with
  MadGraph per graph and is gated, with only the twelve `GC_303` graphs'
  modulus exempted by name ([the amplitude oracle](../validation/amplitude-oracle.md)).

[^b-report]: B′ report: "Correction to the brief", "The derivation", the mutation table and the blind spots.
[^t1-report]: T1 report: "The numbers", "The derivation", "Stop rule" and mutations M1–M4.
[^code-tg]: `vibegraph-lib/src/helas/eval/triple_gluon.rs`.
[^code-rootdiag]: `vibegraph-lib/src/helas/eval/root_diagram.rs`: `boson_momentum_sign`, `momentum_parity_sign`, `propagator_phase_sign`, and the unit tests named above.
[^mg-vxxxxx]: `aloha/template_files/aloha_functions.f`, `vxxxxx`, at mg5amcnlo `b7687064`.
[^ha2-report]: H-A2 report: §1 (the rule), §2 (unit tests and mutations), §3 (the four rows and the zero-width residual), §6 (the dipole).
[^code-standalone]: `vibegraph-lib/tests/standalone_jamps.rs`.
[^code-pt]: `vibegraph-lib/tests/gluon_parke_taylor.rs`.
[^n39-wrong]: Note 39 §1.
[^r1-report]: R1 report, probes P0 and M1.
[^n39-oracle]: Note 39 §3. Its per-row before/after table and the `p p > j j` σ figures are as of 2026-09-25 and are not repeated here.
[^n35-e2]: Note 35 §3.5, the ALOHA VVV normalisation.
