---
type: Validation Methodology
title: Localising an amplitude disagreement with a bit-exact oracle
description: "Match parameter provenance, then compare per-diagram x per-helicity complex amplitudes, fit per-diagram signs (least squares, then a GF(2) solve over diagram features), and verify a fix on the dumps."
status: draft
tags: [amplitudes, debugging, oracle, madgraph, signs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n12-intro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L11-L19", title: "Note 12 — the 2→6 continuum bug"}
  - {id: n12-hard, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L20-L36", title: "Note 12 — why it was hard"}
  - {id: n12-false, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L88-L108", title: "Note 12 — false leads"}
  - {id: n12-tool, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L109-L134", title: "Note 12 — the instrument that worked"}
  - {id: n12-lessons, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/12-helas-continuum-bugfix-journey.md#L135-L164", title: "Note 12 — lessons"}
  - {id: n19-v5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L148-L710", title: "Note 19 V5 — rooting-soundness probes (methodology only)"}
  - {id: n28-s5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2305-L2327", title: "Note 28 S5 — the W-current defect localised to eleven diagrams"}
  - {id: n39-found, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/39-vector-vertex-signs.md#L124-L155", title: "Note 39 §4 — per-diagram sign recovery and the GF(2) solve"}
  - {id: code-probe, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L3358-L3366", title: "probe_process_diagrams"}
  - {id: code-compare, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/compare_amps.py", title: "validation/madgraph/compare_amps.py"}
  - {id: code-signs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/root_diagram.rs#L1100-L1245", title: "root_diagram.rs — vector-vertex signs and fermi_sign assembly"}
---

# Localising an amplitude disagreement with a bit-exact oracle

The binding rule is in [`AGENTS.md`](../../../AGENTS.md) ("Amplitude
disagreements: bit-exact oracle first"). This concept carries the method in
working detail and the cases that established it.

## The order of work

1. **Make the oracle bit-exact before debugging physics.** Both sides must read
   the same masses, widths, couplings and momenta: the same `param_card.dat`
   (the amplitude tables carry MadGraph's), exactly on-shell points (a printed
   LHE point is off shell by ~1e-10, enough for two programs to differ through
   gauge-dependent terms), and the same width convention on spacelike lines
   (MadEvent zeroes them; `output standalone` does not). The 2→6 continuum hunt
   lost sessions chasing a 1% residual through a 7e-4 mass systematic and a
   reference computed with wrong parameters[^n12-lessons]. Couplings can be
   checked by name before any amplitude: [the coupling oracle](coupling-oracle.md).
2. **Go straight to per-diagram × per-helicity complex values**, and per flow
   where colour is involved. Total `|M|²`, two-helicity ratios and scalar
   projections are underdetermined and each misled at least once (compensating
   factors `1.25 × 0.80 = 1.00`)[^n12-lessons].
3. **Fit the per-diagram structure.** With MadGraph's per-diagram values in
   hand, fit one constant per diagram (or `Σ_d c_d · ours_d = MG` over every
   entry). If every residual is at rounding level and every `c_d = ±c`, the
   defect is a pure per-diagram sign pattern and the magnitudes are right.
4. **Solve for the rule rather than guessing it** (below).
5. **Verify the candidate fix arithmetically on the dumped amplitudes** — flip
   or rotate the identified classes, recontract, compare — before writing
   production code.
6. **Run the controls that were not in the fit**, and the production gates.

## The instruments

- **MadGraph side, per process**: `build_amplitude.sh` awk-patches
  `matrix1_orig.f` to expose `AMP()`/`JAMP()` through a COMMON block (f2py);
  `gen_amplitude_tables.py` banks the result per row (see
  [the amplitude oracle](amplitude-oracle.md)). For processes no MadEvent row
  carries, `gen_standalone_jamps.py` does the same from `output standalone`.
- **This crate's side**: the `#[ignore]`d probe
  `helas::eval::run::tests::probe_process_diagrams`, selected by
  `VG_PROBE_NAME=<name>`, dumps the per-diagram array plus a propagator-content
  signature per diagram:

  ```
  VG_PROBE_NAME=<name> cargo test -p vibegraph-lib --features extended-validation \
      --lib helas::eval::run::tests::probe_process_diagrams -- --ignored --nocapture
  ```

- **Matching**: `validation/madgraph/compare_amps.py` (a diagnosis tool, not a
  gate; it reads the probe dump and MadGraph's values at CSV point 0) pairs the
  per-diagram rows by full-helicity-vector overlap and reports each pair's complex
  ratio. For `NCOLOR > 1` it runs the same matcher over the per-flow JAMPs, and
  there it takes the identity pairing whenever that fits at least as well as any
  other, since both sides order flows by the colour basis's sorted keys. That rule exists because the
  `[flow × helicity]` JAMP matrix of a tree-level all-gluon MHV process is rank 1
  (Parke–Taylor: every colour-ordered partial carries the same ⟨ij⟩⁴), so a greedy
  max-overlap matcher pairs its rows arbitrarily; that artefact once produced a
  false "the NCOLOR=6 bases are not 1:1" finding. Once a pairing is established
  it is banked (`MG_DIAGRAM_ORDER`), never re-searched per run.

## Per-diagram sign recovery and the GF(2) solve

The vector-vertex sign work shows the full method[^n39-found]:

1. A least-squares fit `Σ_d c_d · (our diagram d's per-flow values) = MG JAMPs`
   over all entries; every residual came out at 1e-15 with every `c_d = ±c`.
2. Each probe's required flip pattern, together with "no relative change" for
   every diagram of every committed `amplitude_oracle` row, becomes a linear
   system over GF(2) in per-diagram features (vertex class counts, class counts
   off the anchor, the class at the anchor, the source output particle,
   fermion-line classes, propagator species).
3. The solution space either has no role-free member (as here: no per-vertex or
   per-propagator constant works), or fixes the rule up to choices the data
   leave open. Controls outside the system decide those choices.
4. Candidates that fit the system can still fail outside it. A per-vertex
   constant (triple-gluon `+1`, γWW/ZWW `−1` in both roles) fixed every probe but
   broke the enforced `gg_to_gg_cg`; the electroweak extension of the solution
   kept every committed row but broke Standard-Model `w+ w- > w+ w- z` (1.5
   against 3.7e-12). A candidate that fixed only the pure-gluon rows ("contact
   −1 at the anchor only") was falsified by quark-anchored rows the first probe
   set lacked.
5. A prediction is worth more than a fit: `t t~ > g g NP<=1` in `vg_cHG` had
   nothing fitted to it, missed by 7.6e-2 with the first two items and agreed to
   5.7e-16 with the third.

The rule this produced lives in `helas/eval/root_diagram.rs`:
`yang_mills_vvv_sign` (a −1 per colourless VVV vertex other than the anchor),
`vector_contact_sign`, and `gluon_scalar_current_sign`, all keyed to
`Diagram::anchor` and multiplied into `fermi_sign` in `compile_single_diagram`.
The physics is in [vector-vertex signs](../amplitudes/vector-vertex-signs.md);
the rooting-invariance argument is in
[rooting invariance and the anchor](../amplitudes/rooting-invariance-and-anchor.md).

## Worked cases

**The 2→6 continuum** (`u u~ > c c~ e+ e- mu+ mu-`, 579 diagrams). It started at
a 2.26e10 disagreement and ended at 2.14e-13; it was six independent physics
defects plus two defects in the oracle itself, each masking the next. The error
staircase (2.26e10 → 6.95e6 → 7.26e3 → 3.96e1 → 2.66e1 → 2.14e-13) was the
map[^n12-intro]. The per-diagram dump showed every one of 579 magnitudes already
exact and the residual as three phase clusters (528 diagrams at `+i`, 48 at `−i`,
3 Higgs diagrams at `−1`); rotating the classes on the dumped data gave 2.3e-14
before any production change[^n12-tool].

**The W-current amplitude** (`u d > e+ e- u d QCD=0`, 35 diagrams). Per diagram,
over 48 (point, helicity) rows, every diagram paired with exactly one MadGraph
graph at normalised overlap 1.00000, modulus 1.000000, residual ≤ 3.5e-15 — so no
diagram was individually wrong. The fitted constant took both `+i` and `−i`;
flipping exactly eleven diagrams (MadGraph graphs 1–8, 17, 18, 19) and
recontracting through `CF` took the worst `|M|²` deviation over all 74 points
from 5.1e1 to 4.1e-14[^n28-s5]. The fix is the fermion-line sign's mixed-line
arm; see [the fermion-line sign](../amplitudes/fermion-line-sign.md).

**Rooting dependence.** When a re-rooted diagram disagreed with its canonical
rooting, temporary per-diagram probes with per-locus sign counters showed every
failure was an exact `−1` (ratio `−1.0000` at every helicity), and attributed it
vertex by vertex. Two attribution rules came out of it: count a convention sign
once per *vertex* (a chiral vertex's `ProjM + ProjP` terms flip together, so a
per-term product cancels on even-term vertices), and verify factorability first
(every re-rooting changed each diagram by exactly a global ±1 over 1583
diagram × root × helicity checks before any extraction was designed)[^n19-v5].
The derivation itself belongs to
[rooting invariance](../amplitudes/rooting-invariance-and-anchor.md).

## False leads and why each failed

- **Ward identities as a correctness oracle.** A chirality error distributed
  gauge-consistently passed every photon Ward test, and partial fixes broke them.
  Ward tests check consistency, not correctness[^n12-false].
- **Fixes validated on hand-built diagrams.** A hand-built diagram encodes its
  author's rooting, not production's; one fix was byte-identical on the
  production per-diagram oracle. Drive the production compile and compare to
  the reference.
- **By-hand censuses.** "All uux diagrams have 2 fermion propagators" ignored the
  VVS vertices (the 3 Higgs diagrams have 0 fermion and 1 scalar propagator),
  which is precisely why the bug hit that process. Machine-check every count;
  note 29's own write-up repeated the mistake and had to correct it against
  `channel_counts`.
- **Plausible mechanisms that match the data.** "The bra projects on the wrong
  side of the slash" was falsified because `γ^μ P_L = P_R γ^μ` makes the fix
  either bit-identical or Ward-breaking.
- **Explicit signs that were already implicit.** "Each fixed vertex needs a
  Denner −1" broke exactly the odd-count rows: `gR/gL` is real and negative, so
  the projector conjugation already supplied the sign.

## Why disagreements hide

- **Gauge cancellation amplifies.** The continuum's coherent sum is ~3e-3 of the
  incoherent sum, so unit-modulus per-diagram phase errors inflate `|M|²` by
  orders of magnitude, and a 26.6× `|M|²` error can be phases alone[^n12-hard].
- **Uniform errors are invisible.** A phase shared by every diagram of a process
  cancels in `|M|²` and is absorbed by the oracle's fitted `G`; it detonates only
  when diagram classes mix. Normalise each chain type against the reference
  individually.
- **Vector currents wash out conventions.** Fermion-line orientation conventions
  can leave vector currents bit-identical to MadGraph while flipping chiral
  pieces.
- **Sign bugs cluster at duality boundaries**: bra/ket flow, particle/antiparticle
  slot under crossing, index variance.

Every oracle used here has a blind spot; the catalogue is
[oracle blind spots and non-vacuity](oracle-blind-spots-and-non-vacuity.md).

[^n12-intro]: Note 12, introduction.
[^n12-hard]: Note 12, "Why it was hard".
[^n12-false]: Note 12, "False leads, and what killed them".
[^n12-tool]: Note 12, "The instrument that worked". The bespoke probe scripts it names were replaced by `probe_process_diagrams` and `compare_amps.py`.
[^n12-lessons]: Note 12, "Lessons learned".
[^n19-v5]: Note 19 V5, the locus (b) probes and the factorability sweep.
[^n28-s5]: Note 28 S5, "The measurement".
[^n39-found]: Note 39 §4.
