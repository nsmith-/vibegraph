---
type: Physics Convention
title: Vector-vertex convention signs
description: "A colourless vector vertex (VVV, VVS, a four- or five-vector contact, O_W's included) carries no convention sign; a scalar line beside one takes its propagator phase; the triple-gluon vertex takes a calibrated −1; why, and what pins it."
status: draft
tags: [sign-conventions, vector-vertices, qcd, electroweak, madgraph-comparison]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: b-report, resource: "../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report: no role is needed; the triple-gluon constant; the mutations"}
  - {id: code-rootdiag, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/src/helas/eval/root_diagram.rs", title: "triple_gluon_sign, is_yang_mills_vvv, is_coloured, propagator_phase_sign"}
  - {id: code-standalone, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/tests/standalone_jamps.rs", title: "standalone_jamps (per-flow JAMPs against MadGraph standalone)"}
  - {id: code-pt, resource: "https://github.com/nsmith-/vibegraph/blob/5d4a34672e784f8b5fc985e82bd656232ae07caa/vibegraph-lib/tests/gluon_parke_taylor.rs", title: "gluon_parke_taylor (hermetic MHV check)"}
  - {id: n39-wrong, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L16-L53", title: "Note 39 §1 (the Ward-identity failure of u u~ > g g)"}
  - {id: n39-oracle, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L73-L123", title: "Note 39 §3 (the standalone JAMP oracle)"}
  - {id: n35-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L432-L474", title: "Note 35 §3.5 (the ALOHA VVV normalisation)"}
---

# Vector-vertex convention signs

A diagram's `fermi_sign` carries one sign for vector vertices, and only for the
coloured three-vector one (`helas/eval/root_diagram.rs`)[^code-rootdiag]:

| vertex | convention sign |
|---|---|
| colourless VVV (γWW, ZWW, SMEFT VVV, `O_W`'s triple-gauge structures) | none |
| colourless contact of four or five vectors (WWZZ, WWAA, WWWW, `O_W`'s five-vector contacts) | none |
| gluon contact (any structure) | none |
| VVS, colourless or coloured (ZZH, effective ggH, `O_HG`, the loop-induced γZH and γγH) | none; a scalar line at it takes `−1` as every internal scalar line does |
| triple-gluon VVV | `−1` (`triple_gluon_sign`) |

The other factors of `fermi_sign` are in
[convention-sign-inventory](convention-sign-inventory.md).

## Why a colourless vector vertex carries no sign

ALOHA evaluates every vertex as `COUP · L` and closes the amplitude with `−i`
whichever vertex closes it, and its vector propagator carries
`−i(g − qq/m²)/D`, the phase of the evaluator's own. The kernels build each
vector vertex's current as the honest tensor contraction, the same at every
output leg. So a vertex with no fermion bilinear has nothing to read against a
UFO order, and a vector line beside it has no phase to make up: the ratio to
MadGraph at that vertex is `+1`, in every role and at every multiplicity. The
same holds for VVS. What differs there is the scalar line beside the vertex,
whose propagator ALOHA writes `+i/D` against the evaluator's `−i/D`; that `−1`
belongs to the line (`propagator_phase_sign`), whichever vertex produces or
consumes it[^b-report].

ALOHA's VVV normalisation is uniform: it emits `VERTEX = −i·COUP·L` for every VVV
structure, so SMEFTsim's real `GC_7 = G` against the SM's `i·G` does not change
the convention[^n35-e2].

## The triple-gluon `−1`

The triple-gluon vertex's Lorentz structure and coupling normalisation are the
colourless VVV's. Its sign belongs to the colour-ordered decomposition of its
antisymmetric `f^{abc}`: `f_to_traces` matches MadGraph's
`f = −2i Tr(abc) + 2i Tr(cba)` term for term, so the `−1` sits in how the trace
basis meets the vertex slots, which no session has traced. It is calibrated,
not derived: it is the unique minimal solution of the per-diagram sign system
over every MadGraph-verified row (3495 diagrams) once the propagator phases and
the bilinear readings are fixed[^b-report].

Only its ratio to the quark–gluon vertex is observable. In a process the
external gluons fix the parity of the triple-gluon count and the quark–gluon
count together, so `−1` per quark–gluon vertex is the same convention up to a
sign common to the process.

The relative sign it sets is visible without MadGraph: with the triple-gluon
vertex at the colourless VVV's sign, the same-helicity `u u~ > g g` amplitudes,
which the Ward identity sets to zero, come out at `|M|² ≈ 105` at
`√s = 500`[^n39-wrong].

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
  beside a gluon vertex is what pins it.

## What pins it, and what each gate cannot see

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
| `tests/standalone_jamps.rs`[^code-standalone] (hermetic, committed tables in `validation/madgraph/standalone/`) | per-helicity, per-flow JAMPs and \|M\|² against MadGraph standalone, one fitted `G` with `\|G\| = 1`, at 1e-12 (1e-7 on the rows carrying `GC_303`/`GC_299`). Vector-vertex rows: `gg_to_ggg`, `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`, `ee_to_wpwmz`, `wpwm_to_wpwm`, `ttx_to_gg_chg`, the W-pair rows (`wpwm_to_epem`, `wmwp_to_emep`, `wpwm_to_uux`, `wpem_to_zve`, `aem_to_wmve`, `wph_to_tbx`, `hwp_to_tbx`, `zz_to_ttx`, `hh_to_ttx`, `wpwm_to_epemz`, `wpwm_to_uuxg`), `O_W`'s rows, and the `t t~ + jet` rows | the phase of `G`; compensating errors in diagrams that reach the same flows with the same weight; kinematics outside three points |
| `tests/gluon_parke_taylor.rs`[^code-pt] (hermetic, no MadGraph) | 5 and 6 gluons, every MHV and anti-MHV configuration: `J_σ·⟨σ1σ2⟩…⟨σnσ1⟩` constant over flows and `\|J\|·\|cyclic\|/\|⟨ij⟩\|⁴` constant over configurations; tolerance 1e-10 in units of the point's largest flow | the triple-gluon sign (global on a pure-gluon process); global phase and normalisation; NMHV; anything with a quark |
| `tests/amplitude_oracle.rs` | per-diagram and per-flow values on the banked rows | anything those rows do not reach |

Dropping the triple-gluon `−1` fails `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`,
`ttx_to_gg_chg`, the five `t t~ + jet` rows and `uux_to_ttxg_4f` in
`standalone_jamps`, and `gg_to_ttx` with its two SM-limit rows in
`amplitude_oracle`[^b-report]. No mutation has been run that puts a sign on a
colourless vector vertex alone. Which gate pins which
factor across the whole convention is
[validation/convention-channel-coverage](../validation/convention-channel-coverage.md);
the oracle family is [validation/amplitude-oracle](../validation/amplitude-oracle.md).

## Open

- **The triple-gluon `−1` has no derivation.** Following the trace basis through
  `colorize` for one `u u~ > g g` diagram would place it against the
  quark–gluon vertex.
- **A VSS vertex** (scalar–scalar–vector) appears in no model in reach. The
  convention predicts no vertex sign and `−1` for each scalar line beside it.
- **`wpwm_to_wpwmz_cw`'s amplitude cell** (`O_W` through a five-vector vertex)
  agrees with MadGraph at 1.14e-11, the `GC_303` precision, and is still
  informational; its per-graph banking is tracked as
  [wpwmz-cw-ow-five-vector-residual](../backlog/validation/wpwmz-cw-ow-five-vector-residual.md).

[^b-report]: B′ report: "Correction to the brief", "The derivation", the mutation table and the blind spots.
[^code-rootdiag]: `vibegraph-lib/src/helas/eval/root_diagram.rs`: `triple_gluon_sign`, `is_yang_mills_vvv`, `is_coloured`, `propagator_phase_sign`.
[^code-standalone]: `vibegraph-lib/tests/standalone_jamps.rs`.
[^code-pt]: `vibegraph-lib/tests/gluon_parke_taylor.rs`.
[^n39-wrong]: Note 39 §1.
[^n39-oracle]: Note 39 §3. Its per-row before/after table and the `p p > j j` σ figures are as of 2026-09-25 and are not repeated here.
[^n35-e2]: Note 35 §3.5, the ALOHA VVV normalisation.
