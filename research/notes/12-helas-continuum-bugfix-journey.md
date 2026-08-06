# 12 — The 2→6 continuum bug: six root causes and the per-diagram oracle

Post-mortem of the HELAS evaluator's continuum |M|² bug (2026-06-12 → 2026-07-06,
branch `fix/helas-continuum-fermion-flow`). The headline symptom: `u u~ > c c~ e+ e-
mu+ mu-` (QCD=0, 579 diagrams) disagreed with MadGraph by **2.26×10¹⁰**; it ended at
**2.14×10⁻¹³**, with all four `validate_helas_mg` processes enforced at machine
precision. What looked like one bug was six independent physics defects plus two
defects in the validation setup itself, each masking the next.

## Why it was hard

- **Gauge cancellation as an error amplifier.** The continuum's coherent sum is
  ~3×10⁻³ of the incoherent sum, so per-diagram errors of unit modulus (pure phases!)
  inflate |M|² by orders of magnitude. Conversely a 26.6× |M|² error can be caused by
  *nothing but phases* — every magnitude exact.
- **Per-process-uniform errors are invisible.** Any deviation shared by all diagrams
  of a process (a global phase, a per-chain-type phase when all diagrams have the same
  chain content) cancels in |M|². Such bugs pass every 2→2 and 2→4 test and only
  detonate when diagram *classes* mix (uux: 2-fermion-chain continuum + 1-scalar-chain
  Higgs class).
- **Conventions that wash out of vector currents.** Fermion-line orientation and
  crossing conventions leave vector currents bit-identical to MadGraph while flipping
  chiral pieces — the W-array cross-checks passed while the amplitude was wrong.
- **The oracle itself was contaminated, twice** (see fixes 6 and 7 below). Until it
  was bit-exact, real signals were indistinguishable from mass/parameter systematics.

## Root causes, in the order they were fixed

uux `max_rel_diff` vs MadGraph after each stage in parentheses.

1. **Off-shell momentum routing** (2.26e10 → 6.95e6, commits around `c2e7007`).
   Three conflated bugs: `evaluate_propagation` flipped momentum on every propagator
   (mis-routing every >1-vertex line → spurious poles); flow-in currents must subtract
   the boson momentum (`fvixxx`: `fi.p − v.p`) where flow-out adds it; a leg-index
   crash. Also the VVS off-shell vector current (`MetricVout`, commit `99e4589`).
2. **Flow-typed fermion slots** (with a QCD=0 validation-harness mismatch accounting
   for most of the remaining 4e6). `WaveformSlot::Fermion` split into
   `FermionIn`/`FermionOut`; the old "adjoint on demand" `.bar()`/`.unbar()` coercions
   corrupted `GammaJout` (the propagator `(q̸+m)` does not commute with the adjoint).
3. **Flow-driven dispatch** (7.26e3 → 3.96e1, commit `82c5e81`, 2026-06-14). External
   slots were all built flow-in and silently dualized at consumption; bra/ket must be
   picked by *physical* flow (`is_incoming == is_particle`), with UFO i/j selecting
   only which leg is input vs output. Plus the FFS Higgs-current momentum sign
   (`fo.p + fi.p` → `fo.p − fi.p`, commit `9d47e24`), caught by a 2→5 Ward test.
4. **Initial-state spine sign** (3.77e1 → 2.66e1; ee→μμττ 2.02e1 → 5.7e-3, commit
   `6a6d920`). A fermion line joining the two incoming legs needs a crossing sign.
5. **Crossed-line conjugation** (ee→μμττ 0.2–0.57% → 1.8e-14, commit `8cd2ff0`,
   2026-07-06 "S19"). feyngraph binds outgoing legs in the all-incoming (crossed)
   convention: a final-state pair evaluates `ū₁Γv₂` where the vertex is defined
   `ū₂Γv₁`. By `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁` this is exact for γ^μ, conjugates chiral
   projectors (`P_χ→P_χ̄`, no sign) on gamma chains, and negates scalar bilinears.
   Crossing inverts slot identity and flow *together*, so no amount of flow-vs-slot
   inspection can detect it — it required an explicit per-leg `crossed` bit
   (`LegFlow`) threaded through the bake.
6. **Chain-phase normalization + spine-sign parity** (2.66e1 → 2.14e-13, commit
   `51ed7d7`, 2026-07-06 "S20"). The V-propagator carried −i/D (bit-matched to MG),
   the F-propagator no i, the S-propagator −i/D — so every fermion or scalar chain
   deviated by ×i per chain relative to MadGraph. Uniform per process until uux mixed
   the 2-F-chain continuum with the 1-S-chain ZZH Higgs class. Fix: F-prop
   → −i(q̸+m)/D, S-prop → 1/D. And the spine sign of fix 4 is −1 *per internal
   fermion propagator* (flip iff odd count), not once per line — indistinguishable on
   1-prop spines, wrong for uux's 2-prop u-spine (48 diagrams).

Two **oracle defects** fixed along the way, without which none of the endgame was
possible:

6′. **UFO restrict-card parameters discarded** (commit `c8ee1bd`): `restrict_default.dat`
   was used for vertex pruning only, so Rust ran physical lepton masses against MG's
   massless-baked matrix elements. Fixing it (and evaluating with each process's
   actual `param_card.dat`) took ee→μμ / pp→ll from 6.7e-4 "agreement" to 1e-14
   bit-match — and proved the uux residual was real physics, not mass systematics.
7′. **The MG probe reference was always massive-τ** ("S18"): the generated Fortran's
   `SETPARA` ignores the runtime card — `param_read.inc` statically includes the
   param card baked at generation time. Every per-diagram comparison before this
   carried an O((m_τ/E)²) ≈ 3e-4 pollution floor, and one "critical helicity-spectrum
   over-constraint" alarm was purely this mismatch.

## False leads, and what killed them

- **"Bra projects on the wrong side of the slash"** (S13d): plausible mechanism,
  polarisation-dependent, matched the data — falsified because `γ^μP_L = P_Rγ^μ`
  makes the proposed fix either bit-identical to the existing code or
  Ward-breaking. The primitive convention was fine; the *rooting* had to compensate.
- **The hoist validated on hand-built diagrams** (S16): passed its unit checks, was
  byte-identical on the production per-diagram oracle — the hand-built diagram's
  rooting was not the rooting production takes. Repeated theme: S17's re-rooting
  convention was exactly backwards, discovered only by the oracle firing on the
  wrong diagram subset.
- **"Each fixed vertex needs a Denner −1"** (S18): implementing the explicit signs
  broke exactly the odd-count rows. The sign was already carried implicitly — gR/gL
  is real negative, so the projector conjugation supplied it.
- **"All uux diagrams have 2 fermion propagators"** (S19): a by-hand census that
  ignored VVS vertices. The 3 Higgs diagrams have 0 F-props and 1 S-prop, which is
  precisely why the per-chain phase bug hit uux and nothing else.
- **Ward identities as a correctness oracle**: the chirality error was
  gauge-*consistently* distributed, so the full (wrong) family passed every photon
  Ward test, and any partial fix broke them. Ward tests consistency, not correctness.

## The instrument that worked

The endgame (S18–S20) was decided by one tool: a **per-diagram, per-helicity complex
amplitude dump on both sides, at bit-matched parameters**.

- MG side: `build_amplitude.sh` awk-patches `matrix1_orig.f` to expose the `AMP`
  array through a COMMON block (`wrappers/*_amp_probe.f`, f2py);
  `probe_amp.py`/`probe_uux_amp.py` dump `[n_diagrams × n_helicities]` arrays.
- VG side: `#[ignore]` probes in `helas/eval/run.rs` (`probe_eemumutata_diagrams`,
  `probe_uux_diagram_classes`) dump the same array plus a propagator-content
  signature per diagram.
- Matchers: `compare_full_hel.py` (25×16 ratio table for ee→μμττ),
  `compare_uux_amps.py` (matches 579 diagrams by log-|amp| helicity fingerprint,
  cost ~1e-31 — unambiguous).

_These per-process probes/matchers were later generalized into one
process-parameterized pair — the `probe_process_diagrams` test (`VG_PROBE_NAME`)
and `validation/madgraph/compare_amps.py` — during `mg-validation-coverage`; the
bespoke scripts named above no longer exist._

For uux this instantly produced the whole diagnosis: **every one of 579 diagram
magnitudes already exact; the residual was three phase clusters** (528 diagrams at
+i, 48 u-spine at −i, 3 Higgs at −1). The candidate fix was then verified
*arithmetically on the dumped amplitudes* (rotate the classes, recompute |M|²,
2.3e-14) before a line of production code changed.

## Lessons learned

1. **Make the oracle bit-exact before debugging physics.** Chasing a 1% residual
   through a 7e-4 mass systematic and a wrong-parameter reference wasted sessions.
   Parameter provenance on both sides is part of the experiment.
2. **Per-diagram × per-helicity is the decisive granularity.** Total |M|², 2-helicity
   ratios, and 2-hel scalar projections were all underdetermined and each actively
   misled at least once (compensating factors: 1.25×0.80 = 1.00 "correct" cells).
3. **Ward/gauge tests can't see convention errors that are applied consistently.**
   They ruled suspects out but never localized anything.
4. **Never validate a fix through hand-built test diagrams alone** — they encode the
   test author's rooting, not production's. Drive the production bake and compare to
   the reference.
5. **Uniform deviations are latent bugs.** Any per-chain-type or per-class phase that
   happens to be uniform across a process's diagrams is invisible there and fatal
   the first time classes mix. Normalize every chain type against the reference
   individually (the V/F/S phase ledger), not the process total.
6. **Magnitudes exact ≠ done.** Under strong cancellation the phases carry the
   physics.
7. **Verify candidate fixes numerically on dumped data before coding them.** The
   S20 class-rotation check turned "plausible" into "proven" in minutes.
8. **Machine-check census claims** (diagram class counts, which vertices sit on
   fermion lines). Both by-hand counts made during this hunt were wrong.
9. **Bugs come in layers; the error-magnitude staircase is the map.** Each fix
   exposed the next: 2.26e10 → 6.95e6 → 7.26e3 → 3.96e1 → 2.66e1 → 2.14e-13.
10. **Sign/convention bugs cluster at duality boundaries** — flow (bra/ket), crossing
    (particle/antiparticle slot), variance (index position). Each was hand-reconciled
    at a different code site; the typed unification (note 11, TODO
    `lorentz-eval-node-2level`) exists to make this class unrepresentable.

## Where things stand

`validate_helas_mg` (all ENFORCED): ee→μμ 4.2e-14, pp→ll 2.1e-14 (color factor 3),
ee→μμττ 1.8e-14, uux 2→6 2.1e-13 (color factor 9). Production fermion chain
= `i·fvixxx`; all chain types phase-aligned with MadGraph (uniform −i per diagram,
drops out of |M|²). Known non-oracle'd edges: multi-color-flow processes (NCOLOR≥2),
and the massive-fermion reversed spine (the parity sign assumes the massless
S(−q) = −S(q) identity).
