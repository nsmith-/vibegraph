---
type: Codebase Survey
title: MadGraph5 code-quality review and UpdateNotes bug catalogue
description: "MadGraph's strengths and recurring defect classes, a deduplicated catalogue of UpdateNotes v1.0.0–v3.7.1 fixes by category, source observations, and the tests each class suggests."
resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/UpdateNotes.txt"
status: draft
tags: [madgraph, code-quality, bugs, testing, external-code]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n07, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L10-L67", title: "Note 07, scope, strengths and weaknesses"}
  - {id: n07-cat, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L70-L254", title: "Note 07, bug categories 1–7"}
  - {id: n07-tests, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L257-L364", title: "Note 07, implications for unit tests"}
  - {id: n07-src, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/07-mg5-code-quality.md#L416-L750", title: "Note 07 appendix, source observations"}
  - {id: mg-updatenotes, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/UpdateNotes.txt", title: "UpdateNotes.txt (2633 lines, v1.0.0 to v3.7.1)"}
  - {id: mg-rambo, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/rambo.py#L215-L220", title: "rambo.py, massive overflow check"}
  - {id: mg-color, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/color_algebra.py#L365", title: "color_algebra.py, rule_eps_aeps_nosum"}
  - {id: mg-alohalib, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/aloha_lib.py#L132", title: "aloha_lib.py, known_fct"}
  - {id: mg-createaloha, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/create_aloha.py#L52-L123", title: "create_aloha.py, module constants and prop_lib"}
  - {id: mg-lheparser, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/lhe_parser.py#L133-L140", title: "lhe_parser.py, Particle.parse"}
  - {id: diagrams-json, resource: "validation/madgraph/diagrams.json", title: "MadGraph diagram counts per reference row"}
---

A reading of MadGraph5_aMC@NLO's `UpdateNotes.txt` (2633 lines, v1.0.0 to
v3.7.1, 2011–2026) and of `madgraph/core/{base_objects,diagram_generation,color_algebra}.py`,
`aloha/{aloha_lib,aloha_writers,create_aloha}.py` and
`madgraph/various/{misc,banner,lhe_parser,rambo}.py`[^n07]. It catalogues the
defect classes MadGraph itself has fixed, as a checklist of what a generator
like vibegraph should test. Defects this project found first-hand in MadGraph
(the truncated π in `AQCDUP`, seven-digit literals in the Fortran writer, the
injected `aS`, an uninitialised `t` in `genps.f`) are in
[MadGraph defects](../../validation/madgraph-defects.md), not here. The
codebase itself is [the MadGraph survey](madgraph5-amcnlo.md).

## Strengths

- **Exact colour algebra.** `color_algebra.py` uses `fractions.Fraction`
  throughout, so SU(3) reductions are exact; trace cyclicity, Fierz, T-chain
  contraction and the f/d replacement rules are implemented.
- **Canonical diagram tags.** `DiagramTag` encodes a diagram as a nested
  canonical tuple, so duplicate detection is a hash lookup rather than an
  explicit symmetry-orbit search.
- **LHE I/O.** `EventFile` reads plain, `.gz` and >4 GB files and parses the
  banner.
- **RAMBO.** `various/rambo.py` transcribes Ellis–Kleiss–Stirling with the
  `(2N−4) log(ET) + Z[N]` massless weight and a Newton–Raphson massive rescale.
- **Helicity recycling** (v2.9.0) filters vanishing helicities at run time for
  about a 2× LO speed-up ([helicity recycling](../papers/helicity-recycling-mg5.md)).
- **ALOHA** keeps physics separate from the output language.
- **About 400 distinct bug entries since 2011** (note 07's count, not
  recounted at the pin), a sign of an active regression culture.

## Recurring weaknesses

- **Colour matrices**: fixes in v1.5.8, 2.1.2, 2.2.1, 2.3.2, 2.9.25, 3.6.2,
  3.6.7 and 3.7.1, covering ε_ijk, scalar octets, interference signs and
  sextets, each patched locally without a systematic algebraic test.
- **Helicity recycling**: seven distinct correctness bugs from v2.9.0 to 3.6.x
  (scan points, BSM vertices with ≥ 10 couplings, `GC` in model names, spin 2
  and 3/2, propagator-free diagrams, all-zero scan points).
- **Phase-space mapping**: Breit–Wigner mapping, t-channel ordering, threshold
  kinematics and conflicting resonances; several lived 5–10 years.
- **Event-file content**: at least six bugs writing wrong masses, colour flows,
  polarization tags or propagator records.
- **Python 2 → 3**: 30+ migration bugs between 2018 and 2023.
- **Long-lived latent bugs** (FKS shower-scale pick, b-quark colour,
  decay-syntax ordering) that observable-level integration tests would have
  caught.

## Catalogue (LO-relevant entries)

Each entry is a version from `UpdateNotes.txt` and what was wrong. NLO-, MadSpin-,
cluster- and GPU-only entries are summarised at the end.

| Class | Version: defect |
|---|---|
| Colour | 1.3.12: singlet flows from octets leaked into event colour; 2.1.2 and 2.3.2: wrong colour flow for ε_ijk; 2.2.1: wrong colour-flow treatment blocked LO event generation; 2.6.7: wrong interference sign for gluon-like BSM octets (from 2.6.2); 2.9.11: wrong b-quark colour in 5FS `j`/`b` multiparticles (from 2.3.1); 2.9.25: sextet colour matrix wrong for 2 → ≥ 4; 3.6.7: colour matrix wrong for interference terms (from 3.6.2); 3.7.1: EW Sudakov colour entry and α_s value |
| ALOHA | 1.2.2: symmetry reduction broke gauge invariance for a scalar octet; 1.3.21: four-fermion operator; 1.4.3: wrong ε_ijk sign and fermion order in Majorana conjugate routines; 1.5.5: crash on a pseudo-scalar triple-boson vertex; 1.5.8: wrong routine for `P(μ,i)²` structures (from 1.5.0); 1.5.10: wrong wavefunction order, conjugate-routine crash for massless propagators; 2.5.5: C++ output wrong with form factors; 2.9.21: uncompilable output for some structures |
| Model loading | 1.5.1: mass name colliding case-insensitively with another parameter; 2.1.1: UFO names colliding with MadGraph internals (hence the `mdl_` prefix); 2.6.3: restriction merged opposite-sign couplings; 2.9.0: Python-3-incompatible UFOs (`auto_convert_model`); 2.9.12: with > 9 couplings per vertex a coupling was attached to the wrong Lorentz structure; 2.9.18: `cot` defined inconsistently; second model in a two-model export not imported |
| Diagrams | 1.1.1: placeholder `id=0` vertex leaked into HELAS objects; 1.3.3: symmetry factor for contact-only processes; 1.3.12: fermion flow at multi-fermion vertices, only one of several identical-final-state decay chains generated; 1.3.22: s-channel propagator order; 1.3.27: amplitude wrongly flagged as mirrored; 1.4.7: non-identical matrix elements grouped; 1.5.9: wrong propagators for symmetric diagrams in the event file; 2.3.2: HELAS wavefunction order after the Majorana flow fix |
| Phase space | 1.2.1: crash for an s-channel mass above √s; 1.3.11: BW used for ŝ with `M > √s`; 1.4.8: unreachable BW channels not skipped; 1.5.1: multibody decay phase space (from 1.5.0); 2.1.0: wrong σ for E_CM < 1 GeV; 2.2.2: EPA grid bias of three orders of magnitude (from 2.1.2); 2.6.0: NaN in high-multiplicity channels; 2.8.0: spurious low-Q² t-channel configurations, t-channel widths now forced to zero; 2.8.2: "zero width" set to `1e-6 × width` (from 2.6.4); 2.9.0: t-channel ordering strategies, wrong default before; 2.9.1.2: wrong BW map for a conflicting resonance followed by a massless propagator; 2.9.3: t-channel bounds with a heavy initial state; 2.9.9: spurious zero with conflicting BWs; 2.9.18: initial grid biased to θ = π near threshold, so a helicity was wrongly pruned |
| Scales and couplings | 2.4.0: MLM α_s reweighting incomplete; 2.9.5: fixed μF still dynamic for `lpp = 2/3/4`; 2.9.9: wrong α_s power in MLM scale factor for mixed EW+QCD; 3.5.8: scale choice asymmetric for mirror processes (from 3.2.0); 3.5.9: fixed-scale and mixed-beam scale assignment |
| Cuts | 1.5.11: lepton cuts applied to neutrinos in grouped subprocesses (wrong σ for multi-W) |
| Polarization | 2.7.1.2 and 2.8.3: identical polarized particles (decayed, or ≥ 3 polarized with ≥ 2 identical); 2.7.3: longitudinal deviation at large invariant mass |
| Event file | 1.4.2: buffer limit at > 9 final-state particles; 2.0.2: `<init>` format for > 100 subprocesses or PDF id > 10⁶; 2.1.2: 1 → N `<init>` and mothers set to LHC defaults; 2.2.0: NLO `SPINUP` 0 instead of 9, two-ε_ijk colour flows Pythia 8 could not read; 2.2.3: wrong LHAPDF id for built-in `nn23lo` sets; 2.6.3.2: wrong mass for g b initial states; 2.9.23: wrong pseudorapidity in `FourMomentum`; 3.5.14: sextets written wrongly; 3.6.6: wrong intermediate particles written |
| Numerics | 1.3.31: RAMBO overflow above ~50 massless particles; 2.9.21: boost to the CM frame inaccurate for non-Lorentz-invariant amplitudes; 3.3.0: integer overflow at very small requested accuracy |

Outside LO scope, summarised: FKS and FxFx defects (2.9.7 shower-scale pick by
weighted average instead of random, ten years old; 3.0.1 wrong ξ prefactor;
3.3.1–3.6.4 FxFx negative weights), NLO virtuals and EW (3.5.2, 3.5.4, 3.3.0,
3.3.2), MadSpin crashes and ignored benchmarks (2.3.2, 2.5.3, 2.9.24, 3.5.9),
cluster and multicore infinite loops (2.6.0, 2.8.3, 2.9.0, 2.9.14), GPU colour
selection (3.6.5), and a duplicate `dilog` in loop-induced code (3.6.7)[^n07-cat].

## Source observations at the pin

- `various/rambo.py:218` tests `iwarn[4] > 5` where its siblings test `< 5`,
  so the massive-particle overflow warning never fires[^mg-rambo]:

  ```python
  if(wt > 174  and iwarn[4] > 5):
  ```

  The massive Newton–Raphson stops after six iterations and only prints on
  non-convergence.
- `color_algebra.py:365`: `rule_eps_aeps_nosum = True # This is not compatible with LC rules.`
  `T.complex_conjugate()` reverses both the generator chain and the
  fundamental-index pair, `T(a,b,c,i,j)* = T(c,b,a,j,i)`; `f` expands to
  `−2i Tr(a,b,c) + 2i Tr(c,b,a)` and `d` to `2 Tr(a,b,c) + 2 Tr(c,b,a)`.
- `lhe_parser.py:137`: `Particle.parse` sets every field with `float(value)`
  and casts only `pid` to `int`, so `color1`/`color2` stay floats; `EventFile`
  consumes the banner in `__init__`, so reading events and banner needs a seek.
- `aloha_lib.py:132`: `known_fct` includes `cot`, whose definition changed in
  v2.9.18. The global `KERNEL` computation object is shared across routines and
  must be `clean()`ed between processes; `add_function_expression` evaluates
  with `eval()` on trusted UFO input.
- `create_aloha.py:52–53`: `_conjugate_gap = 50` and `_spin2_mult = 1000` are
  fixed naming offsets; `AbstractRoutineBuilder.prop_lib` (line 123) is a
  class-level cache shared by every instance.

## Tests the catalogue suggests

These are the checks each defect class argues for. Which of them vibegraph runs
is in the [validation layers](../../validation/validation-layers.md) and the
[process manifest](../../validation/process-manifest.md).

- **Colour**: exact-arithmetic factors for every supported structure (T, Tr,
  f, d, ε_ijk, sextets), the Casimir and Fierz identities, the sign of colour
  factors between diagrams of different coupling orders.
- **Gauge invariance**: replace an external `ε^μ` by `k^μ` and require the
  amplitude to vanish.
- **Helicity sum**: `Σ_hel |M|²` equal to the traced spin sum.
- **Fermion flow**: consistent arrows at every vertex, Dirac and Majorana.
- **Diagram counts**: against MadGraph's own census, e.g. 2 for
  `e+ e- > mu+ mu-` (γ and Z) and 6 for `g g > g g` (`validation/madgraph/diagrams.json`),
  no duplicate tags, mirrored amplitudes checked by explicit crossing.
- **Couplings**: each coupling of a vertex with ≥ 10 coupling entries mapped to
  its own Lorentz structure; parameter names unique after any prefixing;
  `cot(x) = cos(x)/sin(x)`.
- **Phase space**: BW sampling only when the pole is reachable; conflicting
  resonances give a small nonzero σ; every LO phase-space weight finite and
  positive; every t-channel invariant spacelike; near-threshold σ consistent
  across five or more seeds; RAMBO's `(Σp)² = s` and finite weights at
  N = 10, 20.
- **Cuts**: lepton cuts not applied to neutrinos.
- **Event file**: momentum conservation, `M² = E² − p²` per particle, valid
  mother pointers, colour lines that close, integer colour tags through a round
  trip, `<init>` with > 100 processes, η correct at the beam axis.
- **Robustness**: every iteration loop bounded with an error on
  non-convergence, NaN weights caught and discarded with a warning, zero-event
  and zero-diagram requests handled without a crash.

[^n07]: Note 07, header, strengths and weaknesses; the UpdateNotes line count and the source lines re-checked at `b7687064`.
[^n07-cat]: Note 07, bug categories 1–7, deduplicated.
[^mg-rambo]: `madgraph/various/rambo.py:215–220` at `b7687064`.
