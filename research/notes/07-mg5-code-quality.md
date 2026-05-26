# MadGraph5 Code Quality Review

**Status:** Reference material — no action items. Code quality assessment; motivates design patterns for vibegraph.

> **Source material**: UpdateNotes.txt (2633 lines, covering v1.0.0 through v3.7.1, 2011–2026) plus
> direct code inspection of `madgraph/core/{base_objects,diagram_generation,color_algebra}.py`,
> `aloha/{aloha_lib,aloha_writers,create_aloha}.py`, `madgraph/various/{misc,banner,lhe_parser,rambo}.py`.

---

## Summary of Strengths

- **Symbolic color algebra** (`color_algebra.py`): Uses Python's `fractions.Fraction` throughout, so
  color-factor reductions are exact rational arithmetic — no floating-point error in the SU(3) reduction
  chain.  Standard identities (trace cyclicity, Fierz, T-chain contraction, f/d replacement rules)
  are fully implemented.
- **Diagram-tag deduplication** (`diagram_generation.py`): `DiagramTag` encodes each diagram as a
  recursively-nested canonical tuple (depth-ordered chains), giving an O(1) identity check between
  diagrams instead of the O(n!) explicit symmetry-orbit search used in earlier versions.
- **Robust LHE I/O** (`lhe_parser.py`): `EventFile` transparently handles plain text, `.gz`, and
  size > 4 GB files; banner parsing is integrated; the `FourMomentum` class provides kinematics helpers.
- **RAMBO phase-space generator** (`various/rambo.py`): Clean Python transcription of the
  Ellis–Kleiss–Stirling RAMBO algorithm with the correct `(2N−4) log(ET) + Z[N]` massless weight
  formula and a Newton–Raphson loop for the massive rescaling factor.
- **Helicity recycling** (introduced v2.9.0): benchmark-based selection of non-zero helicity channels
  at run-time gives ~2× LO speed-up; the fallback to full sum is guarded by model checks.
- **Modular UFO / ALOHA pipeline**: ALOHA routines are generated symbolically from the UFO Lorentz
  structure then emitted to Fortran/C++/Python, keeping physics separate from output language.
- **Extensive regression coverage**: The UpdateNotes log shows ~400 distinct bug entries since 2011,
  implying a mature CI/regression testing culture.

---

## Summary of Weaknesses

- **Color-matrix fragility**: Color matrix bugs recur across multiple major versions (v1.5.8, v2.1.2,
  v2.2.1, v2.9.25, v3.6.2, v3.6.7), covering epsilon_ijk assignments, scalar-octet symmetries,
  interference-term signs, and sextet matrices.  Each fix is patch-local and not backed by a
  systematic algebraic test suite.
- **Helicity-recycling brittleness**: The optimization introduced in v2.9.0 generated seven distinct
  correctness bugs by v3.6.x: wrong-helicity-set selection during scans, BSM models with ≥10
  couplings, models with GC in name, spin-2/3/2 particles, edge cases with propagator-free diagrams,
  cases with all-zero amplitudes at one scan point.
- **Phase-space integrator numerical hazards**: Multiple bugs around BW mapping, T-channel ordering,
  threshold kinematics, and "conflicting BW" configurations; many existed for 5–10 years before
  discovery.
- **RAMBO overflow-check sign error** (`various/rambo.py` line 218): `iwarn[4] > 5` should be
  `iwarn[4] < 5` — the massive-particle overflow warning never fires.
- **LHE output correctness**: At least six distinct bugs in which particle masses, color flows,
  polarization tags, or propagator records were written incorrectly to event files.
- **Python version instability**: 30+ separate Python 2→3 migration bugs appeared between 2018–2023,
  indicating that the codebase was not written with version-agnostic semantics.
- **Long-lived latent bugs**: Several bugs (e.g., FKS shower-scale selection, b-quark color
  assignment, decay-syntax ordering) survived 5–10 years before discovery, suggesting gaps in
  integration-level testing against physics observables.

---

## Bug Categories

### 1. Physics-Correctness Bugs

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| 3.7.1 | Bug in EW Sudakov: wrong color matrix entry AND wrong value of α_s | EW Sudakov corrections (NLO EW) | Test color matrix entries independently; test α_s coupling value at each vertex |
| 3.6.7 | Color matrix wrong for interference-term computation (introduced 3.6.2) | Processes with explicit interference terms between different coupling combinations | Test color matrix for every pair of diagrams contributing to an interference term |
| 3.6.5 | cudacpp: color of events wrongly selected when writing LHE (GPU path) | GPU/SIMD event generation | Test that color-flow selection is deterministic and reproduces CPU result |
| 3.6.4 | Critical FxFx merging bug (introduced 3.3.1): too many negative-weight events and high-pT jets | NLO+PS FxFx merging with ≥2 jets | Test that merged sample weight distribution has physical negative-weight fraction |
| 3.5.8 | Scale choice asymmetric in presence of mirror processes → asymmetric z→−z production (introduced 3.2.0) | LHC processes with mirror subprocesses | Test forward-backward symmetry of inclusive η distributions for symmetric initial states |
| 3.5.4 | Virtual amplitude wrongly accounted for when coupling combinations have different sign | NLO processes where virtual has >1 coupling combination (BSM/EFT, not plain QCD to SM) | Test that virtual contribution sums correctly over all coupling-combination signs |
| 3.5.4 | NLO EW at lepton colliders: PS points where real-emission failed but Born passed were discarded entirely | NLO EW at e+e− with initial-state FKS | Test PS-point acceptance logic: a real-emission failure should not discard Born contribution |
| 3.5.2 | 2→1 process at NLO (e.g. p p > Z [QCD]) returned absurdly large cross-section on some compilers | NLO QCD for 2→1 process | Test 2→1 cross-sections: must be finite and match analytic NWA expectation |
| 3.3.2 | Processes with external photons + only QCD corrections were broken | External photons, QCD corrections only | Test p p > γ j cross-section |
| 3.3.0 | montecarlocounter: wrong handling of Born not strictly positive (interference-only Born) | Interference-only contributions at NLO | Test that interference-only Born gives correct sign in NLO weight |
| 2.9.25 | Sextet color matrix wrong for 2→≥4 processes (present since some earlier version) | Sextet (color-6) representation in final state, ≥4 final-state particles | Test all color-6 traces for 2→4; verify Tr(6⊗6̄) identity |
| 2.9.22 | Helicity recycling: code picks wrong amplitude after scan point with all-zero then single-helicity ME | Sequential parameter scan across helicity-zero and non-zero points | Test that helicity recycling gives same result as full sum after re-enabling previously-zero helicity |
| 2.9.12 | BSM/EFT with >9 couplings per vertex: coupling assigned to wrong Lorentz structure | SMEFTatNLO or any model where a single vertex has ≥10 couplings | Test vertex coupling-to-Lorentz mapping for all vertex types when coupling count ≥ 10 |
| 2.9.12 | Helicity recycling incorrectly discards certain BSM interactions with ≥10 couplings | Same as above | Test that all coupling combinations contribute to helicity sum |
| 2.9.12 | Helicity recycling: scan point with all-zero ME causes subsequent points to permanently discard that ME | Benchmark scan with a zero ME at any scan step | Test resumption: zero ME at step i must not suppress non-zero ME at step i+1 |
| 2.9.11 | Wrong color assignment for b-quark in 5-flavor scheme (present since 2.3.1); caused Herwig crash | 5FS processes using auto-defined `j` and `b` multiparticles | Test b-quark color-flow tags in LHE output for 5FS processes |
| 2.9.9 | MLM generation with mixed EW/QCD: wrong power of α_s in scale factor (introduced 2.9.0) | MLM-matched generation of mixed EW+QCD processes | Test that α_s reweighting factor has correct power for EW diagrams |
| 2.9.9 | Wrong zero result in presence of conflicting Breit-Wigner propagators | S-channel resonance + conflicting BW | Test near-resonance processes do not spuriously return zero |
| 2.9.7 | 10-year-old FKS importance-sampling bug: wrong pick of shower scale (weighted average instead of random pick) | All NLO+PS runs; observable impact in matching region | Test shower-scale distribution is uniform over FKS configurations, not skewed to mean |
| 2.9.5 | Factorization scale was dynamic for lpp=2/3/4 despite fixed-scale setting | Effective photon approximation (EPA) beams | Test that fixed-scale flag correctly freezes factorization scale for both beams independently |
| 2.9.1.2 | Wrong BW phase-space mapping when conflicting resonance is followed by massless propagator (introduced 2.8.2) | p p > Z h, Z > e+e−, h > b b̄ γ type processes | Test cross-section near a resonance pole when a massless propagator follows |
| 2.8.2 | "Set width to zero" actually set it to 1e-6 × width (introduced 2.6.4) | Narrow-width / zero-width processes | Test exact zero-width behavior; verify propagator denominator = M² − q² (no fake width) |
| 2.8.3 | Potential wrong result for polarized sample with ≥3 polarized particles, ≥2 identical | p p > W+{0} W+{T} + jets with decays | Test polarized cross-section sums match unpolarized total |
| 2.7.3 | Longitudinal polarization: deviation at large invariant mass due to off-shell effects | Longitudinal vector boson production at high invariant mass | Test that LO longitudinal ε_L·p → 0 in massless limit is not violated |
| 2.7.1.2 | Wrong cross-section for polarized samples with identical polarized particles that are decayed | e.g. p p > j j W+{0} W+{T}, W → ℓν | Test symmetry factor handling for polarized identical-particle amplitudes |
| 2.6.7 | Wrong sign of interference term for gluon-like particles (introduced 2.6.2) | Gluon-like particles (non-QCD gluon in BSM models) | Test interference term sign for any octet particle interfering with SM gluon |
| 2.6.2 | NLO+PS: asymmetry in distributions for processes with identical QCD-charged particles in Born | Born processes with same-flavor quarks (e.g. u u > ...) | Test forward-backward asymmetry is zero for symmetric production |
| 2.4.0 | MLM α_s reweighting not fully applied on some events (impact within scale uncertainty) | MLM matching | Test α_s reweighting completeness on a sample of events |
| 2.3.2 | Wrong color-flow assignment in presence of ε_ijk color structure | RPV SUSY, diquark models, any ε_ijk vertex | Test color-flow tags for processes with ε_ijk vertices; ensure Pythia8 accepts them |
| 2.2.1 | Bug preventing LO event generation due to wrong color-flow treatment | Any LO process (broad regression) | Test that LO color flow never prevents event generation |
| 2.1.2 | Wrong color-flow for ε_ijk structures (earlier instance of 2.3.2 pattern) | Same as 2.3.2 above | |
| 1.5.11 | Critical: lepton cuts also applied to neutrinos in grouped-subprocess mode → wrong σ for multi-W processes | Processes with >1 W decaying leptonically, group_subprocesses=True | Test that lepton pT cuts are NOT applied to neutrinos |
| 1.5.9 | Bug in symmetric-diagram identification: wrong propagators in event file | Processes like p p > ZZj, Z → l+l− | Test propagator PDGs in event file against expected resonance topology |
| 1.5.8 | Critical ALOHA bug (introduced 1.5.0): wrong HELAS routine for Lorentz expressions containing P(μ,i)² | Models with momentum-squared Lorentz structures; none of default SM/MSSM affected | Test any model vertex with P(-1,1)**2 or P(μ,i)**2 type term |
| 1.4.3 | ALOHA: wrong sign convention for ε_ijk; wrong IO fermion ordering in conjugate routines for Majorana/fermion-momentum vertices | Majorana particles, RPV vertices | Test conjugate routine result against manual Fierz identity |
| 1.3.12 | Only leading color flows included in event output (singlet flows from octets previously leaked) | Color-octet resonances | Test that no singlet color tag appears for a gluon propagator |
| 1.2.2 | ALOHA symmetries created non-gauge-invariant result for scalar octet | Scalar color-octet particle | Test Ward identity check for scalar-octet vertex |
| 1.5.7 | Bug for 2→1 process with outcoming fermion as final state (introduced 1.5.0) | 2→1 with outgoing fermion, e.g. u ū → e+ (EFT 4-fermion) | Test 2→1 process cross-section with outcoming fermion |

---

### 2. Numerical Precision / Stability

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| RAMBO code bug | `iwarn[4] > 5` should be `< 5` in massive-particle overflow check (line 218 of rambo.py) — overflow warning never fires | N-body phase space with very high total weight (high multiplicity, large ECM) | Test that overflow warning fires when wt > 174 for massive particles |
| 2.9.23 | Wrong pseudorapidity formula in `FourMomentum` class | Any analysis using η from LHE parser | Test η = −ln tan(θ/2) against known 4-vector values; test collinear limit |
| 2.9.21 | Numerical inaccuracy when boosting to CM frame for non-Lorentz-invariant amplitude | Non-Lorentz-invariant amplitudes (uncommon) | Test that boost to CM frame preserves invariant mass to machine precision |
| 2.9.18 | Threshold bias: initial integration grid biased toward θ = π for processes near kinematic threshold → wrong helicity identified as zero | Processes with √s very close to ΣM_final | Test cross-section reproducibility (5 seeds should agree to statistical error) near threshold |
| 2.9.3 | T-channel with massive initial state and √s ≈ M_initial: wrong boundary conditions on t-channel variable → strange angular distributions | Heavy initial state particles, low CM energy | Test t-channel propagator invariant is correctly bounded by M² |
| 3.3.0 | Integer overflow in NLO interface when very high precision requested | Requesting very small (< 0.001%) integration accuracy | Use 64-bit integers or arbitrary-precision counters for event/iteration counts |
| 3.0.1 | Major NLO bug: phase-space region where ξ_FKS ≠ ξ_norm × ξ̂ had wrong prefactor (processes with cuts and/or massive final states) | NLO+PS with kinematic cuts, especially massive final states | Validate FKS subtraction prefactor against analytic soft/collinear limits |
| 2.8.0 | Numerical issue for low invariant mass in T-channel: spurious configurations generated | Low-Q² T-channel exchange | Test that T-channel invariant is always ≤ 0 (spacelike); flag positive values |
| 2.6.0 | NaN produced for some integration channels in complicated high-multiplicity processes | High-multiplicity (≥6 final-state particles) | Test that phase-space weight is finite and positive; propagate NaN-checks through |
| 2.3.2 | Instabilities in real-emission ME for processes with jets at Born level but no generation cut | e.g. single-top t-channel NLO | Test real-emission ME for vanishing soft/collinear limits; check sum rules |
| 2.2.2 | Integration grid bug (introduced 2.1.2): 3-orders-of-magnitude bias in γγ→μ+μ− via EPA | Effective Photon Approximation | Test cross-section for γγ→μ+μ− against analytic QED result |
| 2.2.2 | NaN from NLO virtual: now skip and continue instead of propagating NaN | Rare numerical instability at NLO PS point | Test that individual NaN PS-point events are dropped, not accumulated |
| 2.1.0 | Wrong cross-section/width for ECM < 1 GeV (broad; also affects decay widths for M < 1 GeV) | Low-energy processes or light particle widths | Guard against ECM ≲ 1 GeV; test known analytic results for light meson widths |
| 2.0.2 | NLO fixed-order: wrong cross-section when PS generation is inefficient (conflicting BW) | Conflicting Breit-Wigner channels at NLO | Test NLO cross-section with processes having two competing s-channel resonances |
| 1.3.31 | Overflow in RAMBO for very high multiplicity (standalone mode) | N > ~50 massless particles | Test RAMBO weight for N = 10, 20; check weight is finite |
| 1.2.1 | Phase-space generation crashed for s-channel mass > √s | Resonance above kinematic threshold | Test graceful handling: BW integration should zero out when M_BW > √s |

---

### 3. Process-Specification / Model-Loading Bugs

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| 2.9.18 | Second model not correctly imported when exporting code with two different models (Python 3 only) | Two-model export (e.g. SM + EFT plugin) | Test that a second imported model does not overwrite first model's particles/couplings |
| 2.9.12 | Models with >9 couplings per interaction: coupling assigned to wrong Lorentz structure | SMEFTatNLO, any dense-vertex BSM model | Test coupling↔Lorentz-structure mapping for vertex with up to 15 coupling entries |
| 2.9.10 | Helicity recycling incorrectly applied to spin-2 and spin-3/2 models (no optimization implemented for these) | RS graviton, goldstino models | Disable helicity recycling for non-standard spins; test full helicity sum instead |
| 2.9.18 | "cot" function in UFO model inconsistently defined before this fix (may break old models) | UFO models using `cot` in Lorentz expression | Test cot(x) = cos(x)/sin(x) identity at several kinematic points |
| 2.9.0 | Python 3 incompatibility in many UFO models; `auto_convert_model` introduced as workaround | Any UFO model written against Python 2 | Test UFO parsing for both Python 2 and Python 3 style files; reject invalid files loudly |
| 2.8.0 | Width of T-channel propagators now auto-set to zero (previously nonzero causing BW-like distortion) | T-channel propagator with nonzero width in param_card | Test that T-channel propagators are spacelike (q²<0); assert width=0 for them |
| 2.6.3 | Restriction model with opposite-sign couplings merged incorrectly for some vertices | Model restriction with sign-changing couplings | Test restricted model gives same cross-section as full model for the surviving diagrams |
| 2.5.5 | ALOHA C++ output wrong in presence of form-factor | UFO models with form-factor in Lorentz structure | Test C++ and Fortran ALOHA output agree on same phase-space point |
| 2.1.1 | Variable name collision: UFO model variable names could collide with internal MG5 names (introduced prefix `mdl_`) | Models with short parameter names (e.g. `g`, `m`) | Test that model parameter names are unique after prefixing; test known collisions |
| 1.5.8 | Critical: ALOHA wrote wrong routine for Lorentz expressions containing P(μ,i)² (introduced 1.5.0) | Any non-SM model with momentum-squared in vertex | Test ALOHA-generated routines against known analytic vertex for P²-dependent coupling |
| 1.5.5 | ALOHA crash for pseudo-scalar 3-boson interaction (introduced 1.5.4) | Pseudo-scalar triple-boson vertex | Test ALOHA generation for ε_{μνρσ} p₁^μ p₂^ν A^ρ B^σ C type vertex |
| 1.5.1 | UFO model with mass name matching another parameter name (case-insensitive) caused wrong results | Models generated by FeynRules with non-unique names | Test that parameter name uniqueness is checked during UFO load |
| 1.4.3 | ALOHA: wrong sign for ε_ijk; wrong fermion IO ordering for Majorana conjugate routines (present since ALOHA v1.0) | Any model with Majorana particles or fermion-momentum Lorentz structures | Test ε_ijk sign in generated ALOHA routine: ε_123 = +1, ε_132 = −1 |
| 1.2.2 | ALOHA symmetry reduction created non-gauge-invariant result for scalar octet | Scalar color-8 particle (e.g. sgluon) | Test gauge invariance check (Ward identity) for scalar-octet processes |
| 2.9.21 | Rare ALOHA-generated code did not compile | Specific Lorentz structure combinations (version-dependent) | Run compilation test for all ALOHA routines generated from a test UFO model |

---

### 4. Phase-Space / Integration Bugs

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| 3.5.9 | Fixed factorization scale: some beams kept dynamic in asymmetric-beam context | Asymmetric beam energies with fixed_fact_scale=T | Test that σ is independent of factorization scale when fixed-scale mode is requested |
| 3.5.9 | Wrong scale assignment when beam#2 is dynamic but beam#1 is static | Mixed beam-type runs | Test scale assignment for each beam independently |
| 2.9.18 | Initial BW integration grid biased toward θ = π near threshold: accidental helicity removal | √s ≈ ΣM threshold | Test that σ(seed=1) ≈ σ(seed=2) ≈ … to within statistical error near threshold |
| 2.9.3 | T-channel BW boundary conditions wrong for massive initial state with √s ≈ M_initial | Heavy initial-state particle (e.g. top-quark collider) | Test t-channel kinematic bounds: t ∈ [t_min, 0] where t_min includes initial-state mass |
| 2.9.1.2 | Wrong BW phase-space mapping for conflicting resonance followed by massless propagator → zero or biased σ (introduced 2.8.2) | e.g. ZH production with Z → ll and H → bb̄γ | Test cross-section is nonzero and unbiased for mixed massive+massless decay topology |
| 2.9.0 | T-channel ordering: four strategies implemented; wrong strategy selection was the default for many processes before v2.9.0 | T-channel dominated processes (VBF, DIS, single-top) | Test T-channel phase-space coverage: 1D projections should be smooth and cover full range |
| 2.8.0 | Numerical issue at low T-channel invariant mass: spurious PS configurations | Low-Q² T-channel (photon exchange near threshold) | Assert Q² > 0 for timelike and Q² < 0 for spacelike propagators |
| 2.6.0 | NaN in some integration channels for high-multiplicity processes | ≥6 final-state particles | Propagate NaN-check through weight computation; discard with warning, not silently |
| 2.3.2 | Small improvement: real-emission ME instability for processes with jets at Born level but no generation cut | e.g. single top t-channel NLO | Test that soft/collinear limits of real-emission ME satisfy known sum rules |
| 2.2.2 | Integration grid (introduced 2.1.2) biased γγ→μ+μ− cross-section by 3 orders of magnitude via EPA | EPA processes with photon beam | Test cross-section against known analytic QED result for γγ→ℓ+ℓ− |
| 2.1.0 | Critical: Wrong cross-section for ECM < 1 GeV; also affects narrow decay widths for M < 1 GeV | Low-energy processes or light particles | Guard: assert ECM ≥ some minimum threshold; test known light-meson widths |
| 2.0.2 | NLO fixed-order: wrong σ when PS generation is inefficient due to conflicting BW | NLO, conflicting BW (present since 2.0.0) | Test NLO PS weight normalizes correctly even with competing s-channel resonances |
| 1.5.1 | Phase-space integration broken for multibody decay processes (introduced 1.5.0) | 1→N decay for N ≥ 3 | Test 1→3, 1→4 decay phase-space generator gives correct invariant-mass distributions |
| 1.4.8 | BW kinematic impossibility not detected: wasted computing on impossible channels | Processes with conflicting BWs and tight cuts | Test that integration channels where BW mass is kinematically unreachable are skipped |
| 1.3.11 | BW used for shat even when BW mass > √s | Any BW resonance above threshold | Assert: BW sampling only applied when M_BW < √s_hat |
| 1.2.1 | Phase-space generation crashed for s-channel mass > √s | Heavy resonance above collider energy | Test graceful zero-cross-section, not crash, when all s-channel masses > √s |
| RAMBO | Massive Newton–Raphson: limited to 6 iterations; no error raised after non-convergence, just print statement | Very tight mass sum near ECM (ΣM → ECM) | Test convergence for ΣM/ECM = 0.999; raise an exception on non-convergence |
| RAMBO | Overflow check for massive particles has sign error (`> 5` instead of `< 5`): warning never fires | High multiplicity, high energy | Fix check; test that overflow warning fires for wt > 174 |

---

### 5. Diagram Generation Bugs

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| 1.5.9 | Symmetric-diagram identification assigned wrong propagators to event file (distributions unaffected, but propagator records wrong) | Processes like p p > ZZj, Z → ll | Test propagator PDG codes in event record against expected topology |
| 1.4.7 | Non-identical matrix elements combined for complicated multiprocess (e.g. p p > l vl l vl l vl) → wrong resonance in event file | High-multiplicity same-flavor lepton production | Test that matrix-element grouping only merges provably identical MEs |
| 1.3.27 | Amplitude incorrectly flagged as mirrored | Some multi-subprocess processes | Test that mirrored amplitude set is correct by comparing with explicit crossing |
| 1.3.22 | Wrong ordering of s-channel propagators for certain multiprocess cases (introduced 1.3.18) | Hard stop (crash) — process-dependent | Test s-channel propagator ordering for processes with ≥2 competing s-channels |
| 1.3.12 | Decay-chain with different decays giving identical final states: only one decay chain was chosen (all 3 combinations should be generated) | SUSY cascades with multiple identical final states | Test that all distinct decay permutations are generated |
| 1.3.12 | Small bug in fermion flow determination for multi-fermion vertices | 4-fermion vertices | Test fermion flow arrows are consistent around every vertex |
| 1.3.3 | Diagram symmetry wrong for processes with no 3-vertex-only diagrams | Pure 4-point contact vertices | Test symmetry factor for a process consisting entirely of 4-point vertices |
| 1.1.1 | Replaced P(μ)=−P(μ) id=0 placeholder vertex produced by diagram generation: it appeared in drawing/HELAS objects | Algorithm-level (always present) | Test that no vertex in a generated diagram has the "null" interaction id |
| 2.3.2 | HELAS: wavefunction appearance order wrong in HELAS diagrams when extra WFs created during Majorana fermion-flow fix | Majorana particles | Test HELAS wavefunction list is correctly ordered after Majorana fermion-flow fix |
| 1.5.10 | ALOHA: wrong wavefunction order (introduced 1.5.7) caused crash | Any vertex (broad regression) | Test wavefunction order for every particle spin combination |
| 1.5.10 | Crash in conjugate routine for massless propagator (introduced 1.5.9) | Massless propagator + conjugate routine | Test conjugate routine for massless vector, massless fermion |

---

### 6. I/O / Output Format Bugs

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| 3.6.6 | Serious LHE bug: intermediate particles wrongly written to event file | Processes with intermediate resonances | Test that every status-2 particle in LHE output corresponds to an actual diagram propagator |
| 3.6.5 | cudacpp: color of events wrongly selected when writing LHE (GPU path) | GPU event generation | Test CPU ↔ GPU color-flow agreement event-by-event |
| 3.5.14 | Sextet (color-6) particles incorrectly written in LHE output | Color-sextuplet model | Test that PID and color tags for sextet particles are correct in LHE output |
| 2.9.23 | Pseudorapidity formula wrong in `FourMomentum` class of lhe_parser | Any analysis computing η from LHE | Test η(px=0, py=0, pz=p, E=p) = ∞; test η for massive particle against known formula |
| 2.6.3.2 | g b initial state: mass in LHE file not always correctly assigned (momentum was correct) | 5-flavor scheme, gluon+b initial state | Test M² = E²−p⃗² for every particle in LHE output; flag M² < 0 |
| 2.3.2 | Momenta precision in LHE file written by MadSpin was insufficient | MadSpin-decayed events | Test 4-momentum conservation to double precision in every event |
| 2.2.3 | Internal PDF sets "nn23lo" and "nn23lo1" associated with wrong LHAPDF ID in LHE file banner | Running with built-in NNPDF sets | Test that lhapdf_id in banner matches the PDF set used for event generation |
| 2.2.0 | SPINUP field in NLO LHE file was 0 instead of 9 (sum over helicities) | NLO events | Test that SPINUP=9 for all NLO events (sum over helicities), SPINUP=±1 for polarized |
| 2.2.0 | Color-flow in presence of two ε_ijk structures: Pythia8 could not handle the format | RPV SUSY, diquark models | Test that color-flow tags generated for ε_ijk processes satisfy Les Houches color-flow convention |
| 2.1.2 | LHE output for 1→N events: `<init>` block and mother IDs were set to LHC defaults | 1→N decay process | Test that mother IDs in 1→N events correctly identify the single initial particle |
| 2.0.2 | `<init>` block: wrong format for >100 subprocesses or PDF ID > 10^6 | Complicated multiprocess generation | Test `<init>` block parsing with 200 subprocess entries and large PDF IDs |
| 1.5.9 | Wrong propagator PDGs in event file for symmetric diagrams | Symmetric final states (ZZj, etc.) | Test all propagator PDGs against expected resonance topology |
| 1.4.2 | Matching broken for >9 final-state particles due to buffer size in event output | ≥10 final-state particles | Test event parsing and matching for 10-particle events |
| lhe_parser.py | `color2` field parsed as `float` (never cast to `int`) — causes subtle integer-comparison bugs | Any LHE file with color tags | In vibegraph, store color tags as integers; test round-trip integer↔string |

---

### 7. Crashes / Regressions

| Version | Description | Trigger | vibegraph test implication |
|---------|-------------|---------|--------------------------|
| 3.6.7 | Loop-induced crash: `dilog` function defined twice (introduced 3.6.6) | Loop-induced process generation | Test that function names in generated code are globally unique |
| 3.5.9 | Crash in multi_run when MadSpin active and BR ≠ 1 during LHE file merge | Multi-run with decay | Test merge of multiple LHE files with non-unit branching ratio |
| 2.9.24 | MadSpin ignored benchmark (compiler-dependent, present since v2.0.0 with GCC13+) | Any MadSpin run with GCC13+ | Test that MadSpin correctly uses the active benchmark (not the default) |
| 2.9.22 | Helicity recycling: rare crash for case without propagator in diagram | Diagrams with contact interactions only | Test helicity recycling on a pure 4-point contact interaction |
| 2.9.19 | Regex bug in Fortran writer (long-latent) | Specific patterns in model parameter names | Test Fortran code generation for parameter names containing digits and operators |
| 2.9.18 | Critical: second model not imported correctly when using two models (Python 3 only) | Two-model export | Integration test: export process using two models, check second model parameters present |
| 2.9.15 | Helicity recycling: rare incorrect-format generated code → crash | Specific helicity configuration during recycling | Test generated code compiles and runs for all helicity configurations |
| 2.9.14 | Crash in multicore mode when 0 jobs needed (introduced 2.9.13) | Empty job list after pre-filtering | Test graceful handling of zero-job multicore run |
| 2.9.0 | Potential infinite loop with Python 3 | Python 3 runtime | Test all loops have termination conditions that work with Python 3 iterator semantics |
| 2.8.3 | Infinite loop issues during Python 2→3 migration | Python 3 `filter`/`map` return iterators, not lists | In Rust: prefer exhausting iterators eagerly to avoid lazy-evaluation-related infinite loops |
| 2.6.0 | Infinite loop with condor cluster when `cluster_temp_path` used | Cluster mode with temp path | Test that retry logic has explicit iteration limit |
| 2.5.3 | Infinite loop in MadSpin when extracting t-channel invariants | MadSpin, t-channel invariants | Test t-channel invariant extraction with known pathological kinematics |
| 2.3.2 | Various MadSpin crashes | Multiple triggers | Test MadSpin on processes with identical final-state particles |
| 2.2.3 | Segfault in FxFx merging (bug #1406000) | FxFx with complicated process | Test FxFx merge with ≥3 jets and >1 multiplicity sample |
| 1.5.10 | Crash in conjugate routine for massless propagator (introduced 1.5.9) | Massless vector/fermion propagator | Test conjugate routine for photon, gluon propagators |
| 1.5.7 | Crash for wwww / zzww interactions in v4 model | v4 model with quartic gauge vertices | Test quartic vector-boson vertices (WWWW, WWZZ, ZZZZ, WWZA, etc.) |
| 1.4.4 | Buffer overflow in `gen_ximprove` when #configs > #diagrams (introduced 1.4.3) | Conflicting BWs | Test that integration config count always ≤ diagram count |
| 1.4.2 | Wrong vertex correction to ensure >9 final-state particles work in matching | ≥10 final states | Test event output for 10-particle events |
| 1.3.21 | ALOHA bug for four-fermion operator (four-fermion contact interaction) | 4-fermion operators (e.g. Fierz-equivalent operators) | Test ALOHA output for 4-fermion Lorentz structure against known result |
| 1.3.20 | Hard stop in `myamp.f` for certain many-subprocess cases with different propagators | Multi-subprocess with distinct propagators | Test that subprocess directory can handle mixed propagator types |

---

## Implications for vibegraph Unit Tests

### Physics-Correctness Tests

- **Color algebra completeness**: For every color structure your generator supports (T, Tr, f, d,
  ε_ijk, sextets), implement an exact-arithmetic color factor test comparing the symbolic reduction
  to tabulated values.  Test all SU(3) quadratic Casimir identities:
  `Tr(T^a T^b) = δ^ab/2`, `Σ_a T^a_{ij} T^a_{kl} = δ_{il}δ_{kj}/2 − δ_{ij}δ_{kl}/(2N_c)`.
- **Color matrix sign for interference**: Explicitly test sign of color factors when two diagrams with
  different coupling orders interfere (the v3.6.2/3.7.1 bug category).
- **Symmetry factors for identical particles**: Test that the symmetry factor of a diagram with k
  identical final-state particles equals 1/k!; test for 2, 3, and 4 identical particles.
- **Ward identity / gauge invariance**: Replace every external polarization vector ε^μ with k^μ and
  verify the amplitude vanishes.  Test in Unitary and Feynman gauge.
- **2→1 amplitude finite and correct**: Test that 2→1 processes give finite cross-section equal to
  NWA result.
- **Fermion flow orientation**: Test that fermion number flows consistently from each incoming fermion
  to exactly one outgoing fermion line, for both Dirac and Majorana fermions.
- **Helicity sum completeness**: For any process, `Σ_{helicities} |M|²` must equal the
  spin-averaged/summed result computed by tracing Dirac structures.
- **Lepton ≠ neutrino for cuts**: Test that phase-space cut logic applies pT_min to charged leptons
  but NOT to neutrinos.

### Numerical Precision / Stability Tests

- **RAMBO invariant mass conservation**: After generating N momenta, verify `(Σp_i)² = s` to 1e-10
  relative precision.
- **RAMBO weight range**: For N ≥ 5 and large ECM, test that weight (log-scale) stays in (−180, 174).
  Test that the overflow/underflow warnings fire correctly.
- **RAMBO massless limit**: For M_i → 0, massive RAMBO should approach massless result continuously.
- **Newton–Raphson convergence**: Test that the massive rescaling Newton–Raphson converges in ≤ 6
  iterations for ΣM/ECM ≤ 0.99; raise an explicit error for non-convergence.
- **Phase-space coverage**: For 2→2 and 2→3, generate 10^5 events and check that the 1D projections
  of cosθ, φ, invariant masses are flat / follow expected distributions (no grid bias near threshold).
- **Near-threshold helicity test**: For processes with √s close to ΣM, run 5 times with different
  seeds; σ must agree within 2σ statistical uncertainty.
- **η formula**: Test `η(px, py, pz, E)` against known values: η=0 for pz=0, η=∞ for massless
  particle along beam axis, η=−ln tan(θ/2) for general angle.
- **T-channel spacelike**: Assert q² < 0 for every T-channel propagator in generated events.

### Process-Specification / Model-Loading Tests

- **UFO round-trip**: Load a UFO model, export it to an intermediate representation, reload it, and
  verify all particles, couplings, and Lorentz structures are identical.
- **Parameter name uniqueness**: After adding the `mdl_` prefix (or your equivalent), assert no two
  parameters share a name.
- **High-coupling-count vertex**: Test that a vertex with exactly 10 or more coupling entries has each
  coupling correctly mapped to its Lorentz structure.
- **`cot` function consistency**: Verify `cot(x) ≡ cos(x)/sin(x)` at several values including near x=0.
- **T-channel width auto-zero**: Verify that any T-channel propagator has its width set to zero before
  phase-space integration.
- **UFO compatibility test**: Attempt to load both a Python-2-style and Python-3-style UFO model;
  expect clean error on incompatible model.

### Phase-Space / Integration Tests

- **BW sampling only below threshold**: When generating s-channel Breit-Wigner phase space, verify
  that the BW peak is not sampled for M_BW > √s.
- **Conflicting BW zero-cross-section avoidance**: For a process where two BWs cannot be simultaneously
  on-shell, test that the integration returns a small but nonzero result (not exactly zero).
- **Multi-channel sampling coverage**: Test that each active integration channel is sampled at least
  once in 1000 events; no channel should produce zero weight for a kinematically allowed process.
- **Phase-space weight positivity**: For LO processes, every event weight should be positive (no NaN,
  no negative weight from PS alone).

### Diagram Generation Tests

- **Diagram count for known processes**: Test that `e+ e− → μ+ μ−` generates exactly 1 diagram (1
  s-channel photon/Z), `gg → gg` generates exactly 4 tree-level diagrams (s, t, u channels plus
  contact), etc.
- **No duplicate diagrams**: For each generated amplitude, assert that the diagram-tag set has no
  repeated entries.
- **Mirrored amplitude identification**: Test that each mirrored amplitude is correctly identified by
  explicit crossing symmetry.
- **Fermion flow for Majorana**: Test that fermion number flow is consistent for a Majorana fermion
  vertex; the direction should flip at a Majorana propagator.
- **4-point contact vertices**: Test that a pure 4-point interaction (quartic gauge coupling)
  generates exactly 1 diagram topology.

### I/O / Output Format Tests

- **LHE 4-momentum conservation**: In every output event, test `|Σp_incoming − Σp_outgoing|/E_CM < 1e-8`.
- **LHE M² = E² − p²**: For every particle in every event, test `|M²_computed − M²_stated| < 1e-6 GeV²`.
- **LHE mother consistency**: For every particle with status=2 (propagator), verify that its mother
  indices point to valid particles in the event record.
- **LHE color-flow validity**: For each event, verify that color tags form valid color-singlet flows
  (no dangling color lines).
- **Propagator PDG in event**: Every status-2 (intermediate) particle must have a PDG code matching
  an actual particle in the model, not an artifact.
- **Integer color tags**: Color fields must round-trip as integers through LHE parse/write.
- **`<init>` block**: Test LHE `<init>` block parsing for >100 subprocesses, PDF ID > 10^6, and
  mixed beam types.
- **SPINUP field**: For summed-helicity events, SPINUP must be 9.

### Crashes / Regression Tests

- **No duplicate function names**: In any generated code (Fortran, C++, or Rust modules), assert that
  all function names are globally unique.
- **Zero-event handling**: Test that a run requesting 0 events completes gracefully with no crash.
- **Empty subprocess list**: Test that generating a process that yields 0 diagrams returns an
  appropriate error, not a crash.
- **Large parameter scan**: Test a scan with 50 parameter points; ensure cross-section values are
  reproducible and that helicity state is correctly reset between points.
- **Iteration limit guard**: Every Newton–Raphson or adaptive-MC iteration loop must have an explicit
  maximum iteration count and must raise an error (not silently continue) on non-convergence.
- **NaN propagation**: Test that a NaN weight at any single phase-space point is caught and discarded
  with a warning, not propagated to the cross-section sum.

---

## Appendix: Code-Level Observations from Source Survey

### `madgraph/various/rambo.py` — Direct Bug Found

```python
# Line 218 — WRONG (overflow warning for massive particles never fires):
if(wt > 174  and iwarn[4] > 5):   # <--- should be iwarn[4] < 5
```

Compare with the correct massless case at line 164:
```python
if wt > 174 and iwarn[2] < 5:     # correct
```

### `madgraph/core/color_algebra.py` — Observations

- Uses exact `fractions.Fraction` arithmetic throughout — ideal for correctness.
- `Epsilon.rule_eps_aeps_nosum` is flagged as "not compatible with LC rules" in a comment —
  indicates a known consistency issue between simplification strategies.
- `T.complex_conjugate()` reverses both the generator index chain and the fundamental-index pair
  separately: `T(a,b,c,i,j)* = T(c,b,a,j,i)`.  Verify this is the correct Hermitian conjugate
  convention for your metric signature.
- `f.simplify()` expands f(a,b,c) = −2i Tr(a,b,c) + 2i Tr(c,b,a) (correct SU(N) identity).
- `d.simplify()` expands d(a,b,c) = 2 Tr(a,b,c) + 2 Tr(c,b,a) (correct).

### `madgraph/various/lhe_parser.py` — Observations

- `Particle.parse()` sets all fields via `setattr(key, float(value))`, then casts `pid` to `int`.
  `color1` and `color2` remain as `float` — integer equality checks on color tags could silently
  fail for large color indices.  Recommendation: cast color tags to `int` explicitly.
- `EventFile` handles `.gz` transparently; the banner is consumed in `__init__`, which means
  seeking back to position 0 is needed after construction if both banner and events are required —
  potential subtle bug for callers that don't know about this.
- `FourMomentum` pseudorapidity bug fixed in v2.9.23; ensure your implementation uses
  `η = -ln(tan(θ/2))`, not a naive `atanh(pz/E)` (which diverges for massless particles at
  exactly pz = E).

### `aloha/aloha_lib.py` — Observations

- The `KERNEL` global object (`Computation()`) holds all symbolic variables and is shared across
  all ALOHA evaluations in a process.  Its `clean()` method must be called between independent
  processes; failure to do so causes expression cross-contamination.
- The `add_function_expression` method silently evaluates numeric expressions via `eval()` —
  input sanitation is absent; only safe because inputs come from trusted UFO files.
- `known_fct` list (line 132) includes `cot` but the `cot` definition changed in v2.9.18; ensure
  your Lorentz expression evaluator uses `cot(x) = cos(x)/sin(x)` consistently.

### `aloha/create_aloha.py` — Observations

- `_conjugate_gap = 50`, `_spin2_mult = 1000` are module-level magic constants controlling ALOHA
  conjugate and spin-2 naming; they are not configurable and could cause silent naming collisions
  for models with many Lorentz structures.
- `AbstractRoutineBuilder.prop_lib` is a class-level dict shared across all instances — a
  singleton cache that can cause cross-contamination when generating routines for different models
  in the same session.

---

*Document generated from UpdateNotes.txt v1.0.0–v3.7.1 (2011–2026) and source survey of key MadGraph5 Python modules.*
