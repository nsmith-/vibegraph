---
type: Physics Convention
title: Vector-vertex convention signs
description: "Against MadGraph every triple-vector vertex carries −1: a charged one (γW⁺W⁻, ZW⁺W⁻) in its kernel, from the W⁺/W⁻ slot exchange of the all-incoming crossing, and the self-conjugate triple-gluon vertex in triple_gluon_sign. Contacts and VVS vertices carry no convention sign, and a scalar line beside one takes its propagator phase. Why, and what pins it."
status: draft
tags: [sign-conventions, vector-vertices, qcd, electroweak, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: b-report, resource: "../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report: no role is needed; the triple-gluon constant; the mutations"}
  - {id: t1-report, resource: "../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report: the triple-gluon −1 is the slot exchange every other triple-vector vertex gets from the crossing"}
  - {id: code-tg, resource: "vibegraph-lib/src/helas/eval/triple_gluon.rs", title: "helas::eval::triple_gluon (the factor-by-factor tests against MadGraph AMP())"}
  - {id: code-rootdiag, resource: "vibegraph-lib/src/helas/eval/root_diagram.rs", title: "triple_gluon_sign, is_yang_mills_vvv, is_coloured, propagator_phase_sign"}
  - {id: code-standalone, resource: "vibegraph-lib/tests/standalone_jamps.rs", title: "standalone_jamps (per-flow JAMPs against MadGraph standalone)"}
  - {id: code-pt, resource: "vibegraph-lib/tests/gluon_parke_taylor.rs", title: "gluon_parke_taylor (hermetic MHV check)"}
  - {id: r1-report, resource: "../sprints/amplitude-signs/sessions/R1-report.md", title: "R1 report: the u u~ > g g probe and the vertex censuses"}
  - {id: n39-wrong, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L16-L53", title: "Note 39 §1 (the Ward-identity failure of u u~ > g g)"}
  - {id: n39-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L73-L123", title: "Note 39 §3 (the standalone JAMP oracle)"}
  - {id: n35-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L432-L474", title: "Note 35 §3.5 (the ALOHA VVV normalisation)"}
---

# Vector-vertex convention signs

A diagram's `fermi_sign` carries one sign for vector vertices, and only for the
coloured three-vector one (`helas/eval/root_diagram.rs`)[^code-rootdiag]. Against
MadGraph's reading every triple-vector vertex carries `−1`; a charged one carries
it inside its kernel, so `convention_sign` has no factor for it[^t1-report]:

| vertex | convention sign | against MadGraph's `COUP · L` |
|---|---|---|
| charged VVV (γW⁺W⁻, ZW⁺W⁻, SMEFT VVV, `O_W`'s triple-gauge structures) | none | `−1`, in the kernel (the crossing's slot exchange) |
| colourless contact of four or five vectors (WWZZ, WWAA, WWWW, `O_W`'s five-vector contacts) | none | agrees with no vertex sign |
| gluon contact | none | agrees with no vertex sign on the four-gluon contacts (SM and `O_G`); `O_G`'s five-gluon contact is in no row |
| VVS, colourless or coloured (ZZH, effective ggH, `O_HG`, the loop-induced γZH and γγH) | none; a scalar line at it takes `−1` as every internal scalar line does | agrees with no vertex sign |
| triple-gluon VVV | `−1` (`triple_gluon_sign`) | `+1` in the kernel (no slot exchanged), so the constant supplies it |

The other factors of `fermi_sign` are in
[convention-sign-inventory](convention-sign-inventory.md).

## Why `convention_sign` has no factor for a colourless vector vertex

ALOHA evaluates every vertex as `COUP · L` and closes the amplitude with `−i`
whichever vertex closes it, and its vector propagator carries
`−i(g − qq/m²)/D`, the phase of the evaluator's own. The kernels build each
vector vertex's current as the honest tensor contraction, the same at every
output leg. So a vertex with no fermion bilinear has no bilinear to read against
a UFO order, and a vector line beside it has no phase to make up: whatever the
vertex's kernel reads against MadGraph, it reads the same in every role and at
every multiplicity. What differs at a VVS vertex is the scalar line beside it,
whose propagator ALOHA writes `+i/D` against the evaluator's `−i/D`; that `−1`
belongs to the line (`propagator_phase_sign`), whichever vertex produces or
consumes it[^b-report].

The kernel's own reading is not `+1` at a charged triple-vector vertex. feyngraph
reads every interaction slot incoming and MadGraph reads every leg outgoing, so
each leg stands in its antiparticle's slot of the UFO vertex, and the two slots
of a conjugate pair hold each other's legs. On `e+ e- > w+ w-` the `a W- W+` rays
are `[p, 3, 4]`: the final W⁺ stands in the W⁻ slot, where MadGraph's
`VVV1_0(W5, W4, W3, …)` puts it in the W⁺ slot. `VVV1` is odd under exchanging
two slots with their momenta, as is every triple-vector structure the SM and
SMEFTsim put on a W pair (`VVV1`–`VVV7` and `VVV9` on `a W- W+` and `W- W+ Z`,
by a numeric parity check at random momenta; `VVV3` vanishes identically), so
the charged vertex's kernel reads `−1` against MadGraph's `COUP · L`. The rest of
`convention_sign` is set against kernels that carry it: `e+ e- > w+ w-` and
`u u~ > g g` carry the same `Diagram::sign`, propagator phases, bilinear
parities and live parity diagram for diagram, and the colourless one agrees with
MadGraph with no vertex sign[^t1-report]. At a fermion vertex the same exchange
is undone, in the colour by `convert_expr`'s transpose of each `T` and in the
bilinear by the slot binding `vertex_root_signs` reads.

The contacts and VVS vertices of the table agree with MadGraph with no vertex
sign on the rows below. The slot exchange has not been traced through their
structures one by one.

ALOHA's VVV normalisation is uniform: it emits `VERTEX = −i·COUP·L` for every VVV
structure, so SMEFTsim's real `GC_7 = G` against the SM's `i·G` does not change
the convention[^n35-e2].

## The triple-gluon `−1`

A gluon is its own antiparticle, so the crossing exchanges none of the
triple-gluon vertex's slots, and its kernel gets none of the `−1` the charged
vertices' kernels carry. `triple_gluon_sign` supplies it. The `−1` is not in the
colour factor or the coupling[^t1-report]:

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
readings were fixed[^b-report]; the slot exchange is why.

**The predicate is a proxy.** `triple_gluon_sign` counts the vertices that are
Yang–Mills triple-vector and coloured. The derived rule counts the triple-vector
vertices with no conjugate pair among their legs. The two part only on a
colourless vertex of three self-conjugate vectors, which no model in reach has:
the SM carries `g g g`, `a W- W+` and `W- W+ Z`; SMEFTsim those and eleven more
on its auxiliary `W1`/`Z1` fields (`a W1- W1+`, `W- W1+ Z1`, …), each with a
charged pair; the toy UFOs no triple-vector vertex. The derivation predicts `−1`
there and the code gives `+1`.

Only its ratio to the quark–gluon vertex is observable where every other vertex
carries an even number of gluons, as in the SM: counting gluon line ends,
`n_qg + n_ggg ≡ n_ext_g (mod 2)`, so a process's external gluons fix the parity
of the two counts together, and `−1` per quark–gluon vertex is the same
convention up to a sign common to the process. SMEFTsim breaks the identity with
vertices of odd gluon number: `O_G`'s five-gluon contact, and `g g g H`,
`g g g H H` and `q q̄ g H` from `O_HG` and the dipole operators. There the two
phrasings differ diagram by diagram, so the convention's choice is observable,
and no committed row reaches such a vertex.

The relative sign it sets is visible without MadGraph: with the triple-gluon
vertex at the colourless VVV's sign, the `u u~ > g g` helicities with equal gluon
helicities, which the Ward identity sets to zero, come out at `|M|² ≈ 105` at
`√s = 500`[^n39-wrong]. On the current code, with `triple_gluon_sign` set to
`+1`, they read `9.6e1` at `√s = 500`, `cos θ = 0.3`, against `0` (to `3e-31`)
with it[^r1-report].

### Details that matter

- **`is_yang_mills_vvv` tests the arity.** A contact of four or more vectors
  can carry momenta (`O_G`'s four- and five-gluon structures, SMEFTsim's
  `VVVV2`/`VVVV3`, `O_W`'s five-vector structures), so "all vector legs and some
  `P`" does not separate the triple vertex from a contact; counted as a
  triple-gluon vertex such a contact would take the `−1`. Pinned by
  `a_momentum_bearing_vector_contact_is_not_a_triple_vector_vertex`, which also
  asserts that `vg_cG` carries momentum-bearing four- and five-vector contacts,
  so the arity test does work there.
- **A pure-gluon process cannot see it.** On a tree of three- and four-gluon
  vertices with `n` external gluons, `V₃ + 2V₄ = n − 2`, so the triple-gluon count
  has the same parity in every diagram and the sign is global. A quark line
  beside a gluon vertex is what pins it. With `O_G`'s five-gluon contact a
  pure-gluon process can see it too (above).

## What pins it, and what each gate cannot see

`validation/madgraph/gen_standalone_jamps.py` generates a process with MadGraph
`output standalone` (pinned 3.7.1), patches `MATRIX` to copy `AMP()` and
`JAMP()` into a COMMON block, and dumps per RAMBO point and per helicity the
colour-summed |M|² and every JAMP, and, on a single-flow row that asks for them
(`bank_amps`), every AMP, at the same param card and momenta as ours. Standalone output keeps the width of a spacelike propagator
while MadEvent (and this crate) zero it (`zerowidth_tchannel`). Through a
massive t-channel line that is a 0.1–0.5% per-helicity difference, not a sign,
so such rows compare with every width zero on both sides (`zero_widths`)[^n39-oracle].

| gate | what it pins | blind to |
|---|---|---|
| `tests/standalone_jamps.rs`[^code-standalone] (hermetic, committed tables in `validation/madgraph/standalone/`) | per-helicity, per-flow JAMPs and \|M\|² against MadGraph standalone, one fitted `G` with `\|G\| = 1`, at 1e-12; on the rows of `ROUNDED_COUPLING_GRAPHS` the named graphs built on MadGraph's seven-digit `GC_303`/`GC_299` are divided by their registered modulus deviation first, and every graph of such a row must also match on its own scale (`GRAPH_RATIO_TOL = 1e-10`). Vector-vertex rows: `gg_to_ggg`, `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`, `ee_to_wpwmz`, `wpwm_to_wpwm`, `ttx_to_gg_chg`, the W-pair rows (`wpwm_to_epem`, `wmwp_to_emep`, `wpwm_to_uux`, `wpem_to_zve`, `aem_to_wmve`, `wpwm_to_epemz`, `wpwm_to_uuxg`), the VVS rows `wph_to_tbx`, `hwp_to_tbx` and `zz_to_ttx` (with `hh_to_ttx`, which has no vector vertex, as their control), `O_W`'s rows, and the `t t~ + jet` rows | the phase of `G`; compensating errors in diagrams that reach the same flows with the same weight; kinematics outside the banked points (three per row, four on two rows, eight on one) |
| `tests/gluon_parke_taylor.rs`[^code-pt] (hermetic, no MadGraph) | 5 and 6 gluons, every MHV and anti-MHV configuration: `J_σ·⟨σ1σ2⟩…⟨σnσ1⟩` constant over flows and `\|J\|·\|cyclic\|/\|⟨ij⟩\|⁴` constant over configurations; tolerance 1e-10 in units of the point's largest flow | the triple-gluon sign (global on a pure-gluon process); global phase and normalisation; NMHV; anything with a quark |
| `tests/amplitude_oracle.rs` | per-diagram and per-flow values on the banked rows | anything those rows do not reach |

Dropping the triple-gluon `−1` fails `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`,
`ttx_to_gg_chg`, the five `t t~ + jet` rows and `uux_to_ttxg_4f` in
`standalone_jamps`, and `gg_to_ttx` with its two SM-limit rows in
`amplitude_oracle`[^b-report]. Putting `−1` on the charged triple-vector
vertices too (`triple_gluon_sign` without `is_coloured`) fails 11
`standalone_jamps` rows (`wpwm_to_epem`, `wpwm_to_uux`, `ee_to_wpwmz`, …) and 5
`amplitude_oracle` rows (`ee_to_wpwm`, `ee_to_wp0wm`, `ee_to_wp0wmt`,
`ud_to_epemud_qcd0`, `wpwm_to_wpwmz_cw`)[^t1-report].

Neither MadGraph gate sees which factor carries the triple-gluon sign. Moving it
into the colour (`f_to_traces` negated, `triple_gluon_sign` at `+1`) or into the
coupling (`GC_10` negated) keeps every SM row of both suites green. The tests of
`helas::eval::triple_gluon`[^code-tg] pin the factors separately on the banked
`g g > t t~` and `e+ e- > w+ w-` `AMP()` tables:

| test | asserts | blind to |
|---|---|---|
| `the_triple_gluon_diagram_agrees_with_madgraph` | each diagram is `G·c·AMP` under one `G` with `\|G\| = 1`, at 1e-12; `triple_gluon_sign` is `−1` on the triple-gluon diagram | a sign shared by every diagram; how the value splits into factors |
| `the_triple_gluon_diagram_has_madgraphs_colour` | our colour coefficient over MadGraph's is one unit constant over every `(diagram, flow)` | a colour sign common to every diagram; anything outside colour |
| `the_triple_gluon_diagram_reads_minus_one_outside_its_colour` | with colour and `triple_gluon_sign` divided out, the quark exchanges are `K·AMP` and the triple-gluon diagram `−K·AMP(1)` | the split among kernel, coupling, propagator phases and the rest of `fermi_sign`; a flip of `triple_gluon_sign` alone |
| `a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel` | on `e+ e- > w+ w-`, colour divided out, every diagram is `K·AMP` under one `K`; each final-state W stands in its antiparticle's slot | the split inside the rest; vertices other than `a W- W+` and `W- W+ Z` |

Each also inherits the oracle's limits: one process, six banked points and the
helicities they carry. Which gate pins which
factor across the whole convention is
[validation/convention-channel-coverage](../validation/convention-channel-coverage.md);
the oracle family is [validation/amplitude-oracle](../validation/amplitude-oracle.md).

## Open

- **A colourless triple-vector vertex of three self-conjugate vectors** is
  predicted to need `−1` and gets none from the predicate
  ([colourless-self-conjugate-vvv-sign-unoracled](../backlog/validation/colourless-self-conjugate-vvv-sign-unoracled.md)).
- **The W-pair structures' oddness** under the slot exchange rests on scratch
  parity checks (T1's, and an independent one in the R1 review, which also finds
  `VVV5` odd on SMEFTsim's `W1` pairs), not a committed test.
- **Vertices of odd gluon number besides `q q̄ g` and `g g g`** (SMEFTsim's
  five-gluon `O_G` contact, `g g g H`, `g g g H H`, `q q̄ g H`) are in no row;
  `triple_gluon_sign` does not count them.
- **No committed table banks `AMP()` for `u u~ > g g` or `g g > u u~`**, so the
  factor tests run on `g g > t t~`
  ([standalone-amps-single-flow-only](../backlog/validation/standalone-amps-single-flow-only.md)).
- **A VSS vertex** (scalar–scalar–vector) appears in no model in reach. The
  convention predicts no vertex sign and `−1` for each scalar line beside it.
- **`wpwm_to_wpwmz_cw`** (`O_W` through a five-vector vertex) agrees with
  MadGraph per graph and is gated, with only the twelve `GC_303` graphs'
  modulus exempted by name ([the amplitude oracle](../validation/amplitude-oracle.md)).

[^b-report]: B′ report: "Correction to the brief", "The derivation", the mutation table and the blind spots.
[^t1-report]: T1 report: "The numbers", "The derivation", "Stop rule" and mutations M1–M4.
[^code-tg]: `vibegraph-lib/src/helas/eval/triple_gluon.rs`.
[^code-rootdiag]: `vibegraph-lib/src/helas/eval/root_diagram.rs`: `triple_gluon_sign`, `is_yang_mills_vvv`, `is_coloured`, `propagator_phase_sign`.
[^code-standalone]: `vibegraph-lib/tests/standalone_jamps.rs`.
[^code-pt]: `vibegraph-lib/tests/gluon_parke_taylor.rs`.
[^n39-wrong]: Note 39 §1.
[^r1-report]: R1 report, probes P0 and M1.
[^n39-oracle]: Note 39 §3. Its per-row before/after table and the `p p > j j` σ figures are as of 2026-09-25 and are not repeated here.
[^n35-e2]: Note 35 §3.5, the ALOHA VVV normalisation.
