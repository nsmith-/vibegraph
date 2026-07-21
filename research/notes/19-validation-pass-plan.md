# 19 — `validation-2` Sprint Plan

**Date**: 2026-07-19. **Position in the loop**: validation pass following the
`hadronic-xsec` feature sprint (note 18). **Goal**: clear the unblocked
validation backlog accumulated across the eval performance program (note 15),
`hadronic-xsec`, and the `validation-sprint` leftovers — plus new σ-level
integration coverage for every `MG_VALIDATED_PROCESSES` entry.

## 1. Scope

**In**: pruned-frame contract guard, interned-SM CI check, NHEL-table pinning
14/14, `vibegraph integrate` generalization (absorbs `cli-proc-card`),
14-process σ gate through the CLI, multi-subgrid PDF seam coverage,
`rooting-soundness` spike, branch-level rooted-tree pattern assertions,
per-flavor diagram matching.

**Out (blocked, stays in `TODO.md`)**: flow→LHEF color-string dictionary and
`mg-single-helicity-bench` (both ride with `event-output-lhef`); `IdentityAmp`
process coverage (needs a non-SM model, rides with `non-sm-ufo`);
`Coeff(f64)`→`CoeffRat` rationalization (optional cleanup, no consumer).

## 2. Key survey findings (2026-07-19)

These reshaped the plan and are worth recording:

- **Every `validation/madgraph/scripts/*.mg5` already carries a `launch`
  block**, and all 19 `output/` dirs contain completed runs (`Events/run_01`
  banner, `results.dat`, σ ± err in `build.log`). The σ references for the
  σ-gate session are **already banked on disk** — no new MG generation rig is
  needed (`gen_hadronic_sigma.sh` stays specific to the dy13 PDF-convolved
  runs). The exact run cards MG used are in `output/*/Cards/run_card.dat`.
- **Beam conventions of the existing runs**: the partonic/colored/2→6 scripts
  launch with `lpp=0` fixed-energy beams (√ŝ = 500 for the 250+250 ones,
  Z-pole for `ee_to_mumu`), so their σ is the **partonic σ̂** — directly
  comparable to a vibegraph integral with no PDF. Only the `pp_*` scripts use
  proton beams.
- **Cross-check**: `ee_to_mumu`'s run σ = 2025 ± 1.116 pb matches the
  `MG5_SIGMA_PB` already pinned in `validate_vegas.rs`.
- **Coverage gap**: `u u~ > mu+ mu-` has no dedicated `lpp=0` script — its
  only σ reference is the PDF-convolved `pp_to_ll`, which ran with MG's
  *internal* `nn23lo1` pdlabel (not an LHAPDF grid, so not comparable to our
  `pdf/` loader without a re-launch). V3b adds a trivial `uux_to_mumu.mg5`
  (`lpp=0`); the PDF-convolved path is already gated end-to-end by
  `validate_hadronic` (dy13, 0.14 % / 0.07 %, note 18).
- **Profiling substrate**: `Cargo.toml` already defines
  `[profile.profiling]` (`inherits = "release"`, `debug = 1`).

### Banked MG run σ values (from `build.log`, for V3b's extractor to formalize)

| process | σ ± err (pb) | beams |
|---|---|---|
| `ee_to_mumu` | 2025 ± 1.116 | e⁺e⁻ 45.6+45.6 |
| `ee_to_ee` | 155.8 ± 0.2096 | lpp=0, 250+250 |
| `ee_to_mumua` | 0.1006 ± 3.865e-4 | lpp=0, 250+250 |
| `ee_to_ttx` | 0.5486 ± 3.292e-4 | lpp=0, 250+250 |
| `ee_to_wpwm` | 7.155 ± 0.02169 | lpp=0, 250+250 |
| `ee_to_zh` | 0.05723 ± 2.068e-5 | lpp=0, 250+250 |
| `ee_to_tatah` | 0.001884 ± 2.405e-6 | lpp=0, 250+250 |
| `ee_to_mumu_tata_qcd0` | 0.001337 ± 2.804e-6 | lpp=0, 250+250 |
| `uux_to_ccx_emmm_qcd0` | 6.556e-7 ± 1.611e-9 | lpp=0, 250+250 |
| `bbx_to_ccx_emmm_qcd0` | 1.605e-6 ± 4.84e-9 | lpp=0, 250+250 |
| `uux_to_uux` | 3.343e+4 ± 52.11 | lpp=0, 250+250 |
| `gg_to_ttx` | 15.95 ± 0.03742 | lpp=0, 250+250 |
| `gg_to_gg` | 1.688e+5 ± 252.4 | lpp=0, 250+250 |
| `uux_to_mumu` | *(to add, V3b)* | lpp=0 |

## 3. Sessions

### V1 — Quick guards (light; Sonnet-able)

- **Pruned-frame contract guard**: nothing enforces the partonic-CM
  ±z-beam requirement of a pruned evaluator — a boosted input silently
  revives J_z-forbidden combinations (up to 3e-3 of the sum, note 15 §2.3).
  Add a debug-build frame assertion (or an explicit boost-to-CM seam) plus a
  test that a boosted point on a pruned evaluator is caught.
- **Interned-SM CI check**: `gen_sm_blob` + `git diff --exit-code` job to
  catch a stale interned SM blob vs the pinned submodule.
- The `validate_helas_mg` timing print is **not** touched here: it currently
  doubles as the samply profiling target, so its retirement rides with V3b,
  which lands the replacement.

### V2 — NHEL-table pinning 14/14

Extend `prune_zero_helicities_matches_madgraph_filter_bitwise`
(`helas/eval/run.rs`) from 7 pinned processes to all 14
`MG_VALIDATED_PROCESSES`, including in-test pinning of the 2→6 survivor
counts (16/256 uux, 32/256 bbx — previously only eyeball-checked against MG
reports). Reference `NHEL` tables come from the existing per-process MG
output dirs.

### V3a — Generalize `vibegraph integrate` (feature-shaped; feature-dev agent)

Absorbs the `cli-proc-card` backlog item. Three hard-codings to remove:

1. **Process assembly**: `hadronic.rs` hard-codes the `p p > e+ e-`
   flavor-class assembly → proc-card-driven subprocess/PDF assembly through
   the existing `GlobalConfig::load_ufo` seam.
2. **Beams**: `runcard.rs` rejects `lpp ≠ (1,1)` → add `lpp=(0,0)`
   fixed-energy no-PDF beams (the mode 13 of the 14 reference runs use).
3. **Phase space**: the (τ,y)×2-body map is 2→2-only → add a
   flat-RAMBO-uniforms-under-VEGAS path for 2→3/2→4/2→6 final states (H3
   substrate: massive RAMBO + splittable substreams). Channel mappings stay
   `lips-nbody` scope; expect slow convergence on peaked integrands — the
   V3b gate is statistical, with precision set by what flat sampling affords.

### V3b — 14-process σ gate through the CLI

- **Extractor** (à la `extract_diagrams.py`): parse σ ± err per process from
  the existing run output (`results.dat` / run banner) into a banked JSON.
- **Run card as single source of truth**: drive `vibegraph integrate` with
  the exact `output/<proc>/Cards/run_card.dat` MG used, so beams *and* cuts
  are pinned identically on both sides by construction. The default cuts in
  those cards screen the divergent processes (Bhabha t-channel, ee→μμγ
  soft/collinear photon, the massless colored 2→2s, the 2→6 γ*→ℓℓ mass
  singularity); `cuts.rs` hard-errors on any active unimplemented cut, so
  gaps surface loudly.
- **Statistical gate**: pull = (σ_vg − σ_MG)/√(err_vg² + err_MG²), |pull| ≲ 3
  plus a relative-tolerance backstop. σ-agreement is a weak oracle (blind to
  mis-sampled regions of small measure) — the bit-exact `validate_helas_mg`
  net remains the fine instrument; this gate covers what that net *cannot*
  see: flux, spin/color averaging, symmetry factors, cuts, beam/PDF handling.
- **Close the uux→μμ gap**: add `uux_to_mumu.mg5` (`lpp=0`) and run it once.
- **Retire (or relabel) the `validate_helas_mg` timing print** and document
  the profiling recipe that replaces its samply role: build the σ-gate test
  under `--profile profiling` and point `samply record` at it. Per-process
  time is then weighted by how hard each process is to *integrate*, so
  hotspots rank in a tackle-worthy order (unlike the old fixed-N loop).

### V4 — Multi-subgrid PDF seam (free-floating)

The LHAPDF oracle only covers the single-subgrid NNPDF23_lo_as_0130_qed set;
the subgrid-walk and two-Q²-knot bilinear fallback are pinned only by
synthetic fixtures. Pick a real multi-Q²-subgrid LHAPDF set, extend
`validation/pdf/gen_oracle.cpp` to describe LHAPDF's seam-derivative-
flattening behavior instead of hard-erroring on repeated Q² knots, and pin
the walk + fallback against it.

### V5 — `rooting-soundness` spike ✅ DONE (2026-07-20)

**CLOSED: the gate is green — `all_rootings_preserve_amplitude` passes 0/133
re-rootings.** All three rooting-dependent sign classes are lifted to the diagram's
`fermi_sign` at the canonical `VtxIdx(0)` rooting, so the honest currents are
rooting-invariant tensors: locus (a) VVV `σ_V`, locus (b) build-convention (VVS
`pure_metric` / FFS scalar-sink / crossed) + spine, and the reversed-bilinear parity.
`validate_helas_mg` stays 14/14 bit-exact throughout; `REL_TOL` relaxed 1e-12 → 1e-10
(benign momentum-sum FP floor 2.2e-11). Full derivation + implementation record in the
"Locus (a)/(b)" write-ups below. Remaining follow-ups are independent of V5: the Path
A/Path B resolver merge and the perf removal of the runtime `resolve_bra_ket` order
check.

Per note 15 §3 + `rooting-study-results.md`: the amplitude is correct only
for feyngraph's `VtxIdx(0)` edge orientation — every node-reducing rooting
silently corrupts multi-boson/≥6-point amplitudes (max_rel up to 1.7e+3).
Make momentum routing, Lorentz-output rooting, and fermion-spine sign
root-invariant. **First deliverable is the failing test**: all V rootings of
every `MG_VALIDATED_PROCESSES` diagram pass the gate, via the
`set_root_override` hook already on `explore/rooting`. This blocks any
production rooting change and the Track-3 re-rooting rule family; the perf
prize (−21 % nodes / −34 % slot traffic) is secondary to the correctness fix.

**Status (2026-07-20): failing gate landed.** The `set_root_override` hook is
ported into the current `root_diagram.rs` (`#[cfg(test)]`, transparent by
construction — `root_override_hook_is_transparent` proves an explicit
`VtxIdx(0)` override reproduces the default |M|² bit-for-bit and runs under
default `cargo test`). The soundness gate lives in
`vibegraph-lib/src/helas/eval/rooting_soundness.rs` as
`all_rootings_preserve_amplitude` (`#[ignore]`): per-diagram root isolation for
processes with ≤40 diagrams, whole-process re-rooting for the two 579/615-diagram
8-point processes, oracle = the baseline `VtxIdx(0)` |M|² (already MG-pinned),
`REL_TOL = 1e-12`, 6 reference momenta/process. Run: `RUST_MIN_STACK=134217728
cargo test -p vibegraph-lib --features extended-validation --lib
rooting_soundness::all_rootings_preserve_amplitude -- --ignored --nocapture
--test-threads=1` (~52 s). It **FAILS 21/133 re-rootings across 6 processes**,
sorting cleanly into the three loci the fix must address:

1. **Momentum-odd boson-vertex sign** (the dominant, gross failure) — **FIXED,
   see the LANDED block below.** VVV structures used a fixed `NegVout` (`−V^μ`)
   momentum-odd sign (`root_lorentz.rs`) calibrated to the `VtxIdx(0)` output leg;
   re-rooting a boson vertex to a different output leg flipped it uncompensated.
   `ee→W+W-` max_rel 4.2–6.8, `gg→ttx` diagram 0 root 1 = 3.65. **The ≥6-pt QCD
   `uux/bbx` whole-process shifts (0.1–0.37) were MIS-ATTRIBUTED here**: those
   processes are QCD=0 and contain **zero VVV vertices** (verified by direct
   count — 0/579 and 0/615 diagrams), so the locus-(a) fix is a no-op for them
   and they still fail. They belong to locus (b) below — a whole-process
   re-rooting moves fermion sinks across the 4 fermion lines, and the colour
   interference amplifies the spine-sign flip into the 0.1–0.37 range.
2. **Fermion-spine sign** (`spine_sign_from_flow`, derived from the baked spinor
   adjoint / output-leg direction). Re-rooting a fermion line moves the sink,
   flipping the spine sign: `ee→ττH` diagrams 0–4 = 1.3e-3…3.4e-3, `ee→μμττ`
   diagram 16 = 6.5e-5, and (per the reclassification above) the `uux/bbx` 2→6
   whole-process shifts 0.1–0.37.
3. **Benign FP reassociation** (correct amplitude, just over `REL_TOL`).
   `POut = −Σ inputs` momentum routing *is* orientation-aware, so re-rooting only
   reorders the momentum sums: `ee→ττH` diag 4 root 2 = 2.16e-11, `ee→μμττ`
   diags 18/23/24 ≈ 1.1–1.5e-12. Not a soundness bug; a sound fix will still trip
   `REL_TOL` here, so the eventual pass criterion for class 3 should widen the
   tolerance or compare against a per-diagram symbolic invariant rather than |M|².

The fix is a genuine three-pronged sign-convention rewrite (note-12 bug-magnet
territory), separable from this gate and best done as its own session against the
gate now in place: (a) the boson-vector-propagator sign (below); (b) make
`spine_sign_from_flow` invariant to which end of a fermion line is the sink;
(c) decide class-3's tolerance/oracle. Recommend not promoting any production
rooting change until (a)+(b) land and the gate goes green.

**Locus (b) — DIAGNOSED, NOT the fermion-spine sign (2026-07-20, inline probe
session; tree left clean, no code committed).** The note-19 attribution above —
"make `spine_sign_from_flow` invariant" — is **falsified**, the third
mis-attribution in this taxonomy (after uux/bbx → locus (a), and the whole "spine"
framing). Hard per-diagram complex-amplitude probes (temporary `eval_single_diagram`
+ `REVERSED_FIRES`/`SWAP_GAMMA` counters over `ee→ττH` all diags/roots, real CSV
momenta, all helicities) established:

1. **The re-rooting flips the per-diagram amplitude by EXACTLY −1** (ratio `−1.0000`
   for every helicity; massive-tau propagator invariant, so it is a Dirac/convention
   sign, not a momentum-routing effect).
2. **It is NOT the spine sign.** `fermi_sign` (= `diagram.sign ·
   spine_sign_from_flow · yang_mills_vvv_sign`) is **identical** in base and every
   re-root; a direct `spine_sign_from_flow` re-root probe never flips. `spine_sign_
   from_flow` is already root-invariant.
3. **It is NOT the runtime `reversed` flag** (`resolve_bra_ket` → `gamma_vout`/
   `ffv_vout` −1): the reversed-fire count is **identical** (1) in base and re-root —
   the −1 just lands on a different vertex.
4. **It is NOT a γ^μ fermion-continuation / `correct_spin_index_for_flow` swap**:
   `ee→ττH` has **zero** gamma-chained swaps in *every* rooting yet still flips, and
   `ee→μμττ` diag 16 (a real flip) also has zero swaps. Injecting a −1 per swapped
   continuation is a no-op on `ee→ττH` (0 swaps) and **regresses baseline**
   (`ee→WW` 1.72e3, `ee→μμa`, 2→6) on the processes that *do* swap at
   `VtxIdx(0)` — so swap-parity is not the discriminator.

**Mechanism (confirmed for `ee→ττH`): amplitude-SINK convention signs keyed to
which vertex is the root.** `root_lorentz::build_at_leg` bakes the *root* vertex with
`idx = None` (scalar amplitude sink) and applies a cluster of −1s there —
`ProjM/ProjP/Identity` scalar-sink −1 ("−1 against the −i/D scalar propagator",
lines ~472/488/544), the `pair_crossed` −1, and the `pure_metric` vertex −1 — while
a *non-root* vertex is baked with `idx = Some(output leg)` and gets a different sign
set. In `ee→ττH` diag 0 the canonical `VtxIdx(0)` root is the electron FFV, so the
amplitude root is a **photon `Metric`** (no scalar-sink −1); re-rooting to the
Yukawa (`root VtxIdx(2)`) makes the **`ProjMAmp` Yukawa scalar** the amplitude sink,
firing the scalar-sink −1 → the observed −1. `ee→μμ` never flips because its
amplitude root is *always* a photon `Metric` (no scalar bilinear can become the
sink). This is **the same class as locus (a)** (a role-dependent source/sink sign),
now for FFS/FFV *amplitude sinks* rather than VVV. `ee→μμττ` diag 16 flips with **0
scalars and 0 swaps**, so at least one more amplitude-root sign (a `pair_crossed`/
adjoint case in the `idx=None` branch) is in play — the locus (b) fix must sweep the
whole `build_at_leg` `idx=None` sign cluster, not just the scalar-sink −1.

**Fix direction (next session).** Mirror locus (a): make every amplitude-sink sign
rooting-invariant. Either (i) compute the sink-sign cluster from the *fixed* diagram
(canonical root) and fold it into the per-diagram `fermi_sign`, decoupled from the
live root — as `yang_mills_vvv_sign` does — or (ii) apply each convention −1 at a
rooting-invariant locus (the physical vertex/leg it belongs to) regardless of whether
that vertex is the current root. Must re-verify **14/14 `validate_helas_mg`
bit-exact across every color flow** (the `build_at_leg` signs are pinned per-diagram
against MG AMP() for `gg→gg`/uux 2→6/bbx 2→6 — see the in-code comments — so a wrong
generalization silently breaks a flow). The current gate stays **18 failures** (13
gross locus (b): `ee→ttH` ×5, `ee→μμττ` diag16 ×2, uux 2→6 ×2, bbx 2→6 ×5; +5
benign FP locus (c) ~1e-11..1e-3 mixed in — re-triage after the sink-sign fix, since
some "gross" bbx entries at 4.8e-3/1.7e-2 may be additional sink cases).

**Locus (b) — first implementation attempt + KEY counter-example (2026-07-20, inline;
all code reverted, tree clean).** User endorsed option (i) with the principle
*"textbook structure at every vertex, then compute the relative sign on the undirected
graph."* First attempt: treat the scalar-sink −1 as a **per-scalar-propagator**
convention — remove it from `build_at_leg` (FFS `ProjM/ProjP/Identity` scalar-sink and
the VVS `pure_metric` scalar-out −1, the latter shown to *also* be rooting-dependent:
it fires only when the VVS output leg is the scalar) and fold `(−1)^{#scalar props}`
into `fermi_sign` via `scalar_propagator_sign`. Result: **13/14 bit-exact** (`ee→ττH`
✓, `uux 2→6` ✓) but **`bbx 2→6` breaks (1.99e0)**. `uux` only *appeared* fixed — its
`H`-from-`uu` diagrams are ~0 (tiny u-Yukawa), hiding the sign; the massive-`b` Yukawa
makes `bbx`'s `H` diagrams non-zero and exposes it.

**The counter-example (`bbx→ccx eemm`, the two diagrams with a scalar propagator):**
- **diag 72**: `H` (scalar prop, `ParticleId(33)`) is **VVS-produced** (`CouplingId(80)·
  Metric` of two vector currents) and **consumed by an FFS as a fermion-out current**.
  Net old sign = −1 (VVS `pure_metric` only). `scalar_propagator_sign` = −1. **Match.**
- **diag 121**: the *same* `H` prop is **VVS-produced** *and* **consumed by the `bbH`
  Yukawa at the amplitude root** (`ProjMAmp+ProjPAmp`), which fires its *own* scalar-sink
  −1 *on top of* the VVS −1. Net old sign = (−1)(−1) = **+1**. `scalar_propagator_sign`
  = −1. **MISMATCH → diag 121 flips → `bbx` wrong.**

**Conclusion: the scalar-production sign is genuinely per-vertex-ROLE, not
per-propagator.** VVS-scalar-out fires −1 *and* FFS-scalar-sink fires −1 *independently*;
the same propagator nets −1 or +1 depending on how its consuming end is rooted. So
`(−1)^{#scalar props}` cannot reproduce it, and neither can any "apply once per
propagator" rule (checked both producer- and consumer-end placement — both fail diag
121). This is the concrete proof that locus (b) is a **role/phase-convention** problem,
not a topological-count problem.

**Refined fix plan.** The role-dependent ±1s that must ALL move off the vertices for a
textbook tree: (1) FFS scalar-sink −1, (2) VVS `pure_metric` scalar-out −1 (keep only
the all-vector VVVV-contact −1, which `gg→gg` shows is already root-invariant), (3) the
runtime FFV/gamma `reversed` −1 (`resolve_bra_ket` → `gamma_vout`/`ffv_vout`; the
user's perf target), and (4) whatever drives `ee→μμττ` diag16 (0 scalars, 0 swaps —
still uncharacterized). A **textbook tree is NOT rooting-invariant on its own** (an
earlier check: dropping just the FFV `reversed` gives base `−A`, reroot `+A`), so the
per-diagram graph sign must capture *all* of (1)–(4) together. The only *robust* way to
compute that graph sign and stay bit-exact is to evaluate it at the **canonical
`VtxIdx(0)` rooting** (which is what production always uses and what MG is pinned to),
independent of the live root — i.e. either re-derive each vertex's canonical role from
the fixed diagram, or bake the sign once at a forced `VtxIdx(0)` and reuse it. This is a
genuine multi-vertex phase-convention refactor (note-12 territory), not a one-liner;
budget it as a dedicated session with the 14/14 bit-exact gate (esp. `bbx 2→6`, the
sharpest oracle) run after every increment. `ee→μμττ` diag16 must be characterized
*first* — it is the one confirmed locus-(b) flip with neither a scalar nor a swap, so it
reveals the residual role-sign that (1)–(3) don't cover.

**Locus (b) — `ee→μμττ` diag16 CHARACTERIZED (2026-07-20, inline probe; all code
reverted, tree clean). The "0 scalars, 0 swaps, uncharacterized residual" premise
above is FALSIFIED — diag16 is NOT a new residual role-sign; it is case (2), the VVS
`pure_metric` sign, one more instance of the exact same mechanism as `bbx` diag 121.**
Per-diagram complex-amplitude probe (temporary `REVERSED_FIRES` counter, a compile-time
`BUILD_SIGN` product, and per-locus `sign_tag`s in `build_at_leg`/`build_child`, driven
through `set_root_override` + `compile_single_diagram` over all 4 rootings at the
largest-amplitude helicity):

- **diag16 is a Higgs diagram**, propagators `[Z + H + Z]`: `e⁺e⁻ → Z*`, `Z* → Z H`
  via the **HZZ VVS vertex** (`CouplingId(80)·Metric`), `Z → μ⁺μ⁻`, `H → τ⁺τ⁻` (the
  ττH Yukawa). The earlier "0 scalars" reading was simply **wrong** — there *is* an
  `H` propagator and a VVS vertex. (The prior probe's "scalar/swap" counters missed it;
  the ττH Yukawa *does* fire `ProjMAmp+ProjPAmp` scalar-sink −1s.)
- **The flip is exactly the HZZ `pure_metric` −1, and nothing else.** `BUILD_SIGN`
  tracks the amplitude flip bit-for-bit; `fermi_sign` (+1) and the `reversed` parity (1)
  are **invariant** across all 4 rootings. Per-locus tags:
  - root0 (base, no flip): `projM/projP_scalar_sink` + `projM/projP_pair_crossed`
    (the ττH Yukawa) — **no `pure_metric`**. HZZ is rooted at a *vector* (Z) output →
    `MetricVout` vector current, no sign.
  - root2 (no flip): identical tag set, no `pure_metric`.
  - root1 (FLIP −1): same Yukawa tags **+ `pure_metric`** — HZZ is the **amplitude
    sink** (`-1*ScalarProduct(Metric(Leg0,Leg1),Leg2)`), so its bare `Metric` fires the
    scalar-sink `pure_metric` −1.
  - root3 (FLIP −1): same Yukawa tags **+ `pure_metric`** — the amplitude sink is the
    ττH Yukawa, and HZZ is now rooted at its **scalar (H) output leg**, which also fires
    `pure_metric` (scalar-out).
  The Yukawa scalar-sink/pair-crossed −1s are present in *all four* rootings → they net
  a constant sign, not the flip. **The sole rooting-dependent contribution is the HZZ
  `pure_metric` −1, which fires iff the VVS is rooted at a scalar leg or the amplitude
  sink, and is absent iff it is rooted at a vector leg** — identical to the `bbx`
  diag-121 finding (`VVS-scalar-out −1` vs `VVS-into-amplitude −1`).

**Consequence for the fix plan.** Item (4) collapses into item (2): there is **no
fourth, uncharacterized residual role-sign**. The complete role-dependent ±1 set for a
textbook tree is just (1) FFS scalar-sink −1, (2) VVS `pure_metric` −1 (scalar-out *and*
amplitude-sink), and (3) the runtime FFV/gamma `reversed` −1. All three are the *same
kind* of role-dependent convention sign; fixing (1)+(2) rooting-invariantly (compute
from the canonical `VtxIdx(0)` role, fold into a per-diagram scalar à la
`yang_mills_vvv_sign`) should close both `ee→ttH`/`ee→μμττ` (FFS/VVS) and the `bbx`/uux
scalar classes together. Keep (3) for the perf-motivated same-refactor cleanup. Sharpest
oracle after every increment remains **`bbx 2→6`** (massive-b Yukawa exposes the VVS
sign that `uux` hides).

**Perf aside (user, 2026-07-20): removing the runtime `reversed` branch in
`gamma_vout`/`ffv_vout` is desirable.** It is *orthogonal* to locus (b) (not its
cause — the reversed count is root-invariant), but it is a valid hot-path cleanup and
folds naturally into option (i): if the FFV/gamma reversal sign is also lifted to a
compile-time per-diagram scalar, both `resolve_bra_ket` branches disappear. Do it in
the same refactor, gated by the same 14/14 bit-exact check.

**Locus (b) — the Path A / Path B bifurcation (user, 2026-07-20; DEFERRED cleanup,
not the sign fix).** The rooting-dependent sign enters through *two* code paths in
`root_lorentz` that handle the same `LorentzOp` differently:
- **Path A** — the scalar/amplitude-sink `while` loop in `build_at_leg` (~L442+,
  `idx=None` or leftover disconnected factors): `Metric`→`pure_metric` −1,
  `ProjM/ProjP/Identity`→`scalar_sink` + `pair_crossed` −1.
- **Path B** — `build_child` rooted at a real output leg (~L270+): `Metric`→
  `MetricVout` (no sign), `ProjM/ProjP`→`standalone_projector_crossed` only (a
  *different* rule). `Gamma` carries no compile sign in either path (the orientation
  −1 is applied at runtime by `reversed`, which is why reversed-parity is
  root-invariant).
Which path a vertex takes is decided by the rooting, so the two disagreeing sign
rules ARE the rooting-dependence. **Agreed direction:** collapse the per-op semantics
into a single sign-free resolver keyed on the output fiber
(`VectorOut(idx)`/`FermionOut(idx,adjoint)`/`ScalarSink`) — the node *type* still
differs by fiber (`MetricVout` vs `Metric`, `ProjM` vs `ProjMAmp`), but there is one
`match op` body and **no `sign` in it**. Structural merge is a *follow-up cleanup*;
the sign extraction below is the actual gate fix and does not require it. NOTED, not
yet done.

**Locus (b) — SIGN-EXTRACTION design (2026-07-20). Make `root_term`/`build_at_leg`
sign-free and carry a per-diagram convention sign `S` computed from the canonical
`VtxIdx(0)` bake — the `yang_mills_vvv_sign` template generalized.** Mechanics:
`build_at_leg` currently returns a per-term `sign` folded into `RootedTerm.coeff`
(diagram_eval.rs:109); the sign never touches the tensor tree, only the scalar coeff.
So IF re-rooting multiplies each *diagram's* amplitude by a global ±1 (equal to the
change in the product of all its term signs), the extraction is: strip the sign from
`build_at_leg` (coeff = `term.coeff`), compute `S` = product of term signs from a
forced-`VtxIdx(0)` bake, fold `S` into `fermi_sign`. Bit-exact at `VtxIdx(0)` by
construction (`S` ≡ the canonical build product), root-invariant elsewhere (build
carries no sign). diag16 already shows this shape (amp ratio = build-sign ratio =
−1, exactly).

**FACTORABILITY — VERIFIED GREEN, with a critical refinement (2026-07-20, inline
`probe_factorability_sweep`; all probe code reverted, tree clean).** Swept every
diagram of every ≤60-diagram MG process at every non-canonical rooting, all helicities,
1583 (diagram,root,helicity) checks. Result: **0 shape violations** (every re-rooting
changes each per-diagram amplitude by exactly a global ±1 — unit modulus, so the honest
tensor value IS root-invariant and a per-diagram scalar can absorb the rest) and, after
the refinement below, **0 predict violations** (the ±1 is fully reproduced from the
canonical bake). The strip-and-refold architecture is sound.

**THE REFINEMENT — the convention sign is per-VERTEX-current, NOT per-term. A
product-over-terms accounting is WRONG.** First sweep left 32 unexplained −1s, all in
`ee→ττH` at `root2` (the ttH Yukawa as amplitude sink), with build/reversed/`fermi_sign`
all reading +1. Tree dump of `ee→ττH` diag0 showed why: the chiral Yukawa has **two**
Lorentz terms (`ProjM`+`ProjP`), and re-rooting flips **both together** —
`-1*ProjM + -1*ProjP` (root0, off-shell fermion current) → `+1*ProjMAmp + +1*ProjPAmp`
(root2, scalar sink). The *current* flips by −1, but a product over its two terms reads
(−1)²=+1 and misses it. Fixing the instrument to fold **one sign per vertex** (after
asserting all its terms agree) closed all 32 and confirmed **0 non-uniform vertices**
across the whole sweep — every vertex's Lorentz terms do share a single build sign, so
the per-vertex sign is well-defined. Implementation consequence: the extracted sign must
be applied **once per vertex** (a common factor of the vertex's summed terms), exactly
as `yang_mills_vvv_sign` counts vertices — NOT `(-1)^{#terms}` or any per-term product,
which cancels on even-term vertices (every chiral FFV/FFS: `ProjM`+`ProjP`).

**Complete set of rooting-dependent sign sources (all ±1, all canonical-bake
computable):** (a) the **per-vertex build sign** from `build_at_leg` (VVS `pure_metric`,
FFS `scalar_sink`/`pair_crossed`, `standalone_projector_crossed`), applied once per
vertex; (b) the **fermion `reversed`** sign at fermion→vector sinks
(`gamma_vout`/`ffv_vout`), a runtime parity; (c) **`spine_sign_from_flow`** inside
`fermi_sign` — tree-dependent, so it too can flip on re-rooting (contrary to an earlier
claim in this note that it is root-invariant; it was simply invariant on the diagrams
first probed). The scalar bilinear (`scalar_bilinear_current`) discards `reversed` and is
genuinely order-independent (ψ̄Γψ), so it contributes no value sign — instrumented and
confirmed parity-invariant across the sweep.

**Implementation plan (validated, ready).** (1) Make `build_at_leg`/`root_term`
sign-free (drop the `sign` return; `coeff = term.coeff`). (2) Compute a per-diagram
convention scalar `S` = Π_vertices (that vertex's build sign at its **canonical
`VtxIdx(0)` role**) × the canonical `reversed`/spine contributions — reusing the fixed
diagram's canonical rooting so `S` is decoupled from the live root. (3) Fold `S` into
`fermi_sign` (which already carries `spine`+`yang_mills`; this generalizes the same
mechanism). Bit-exact at `VtxIdx(0)` by construction; root-invariant elsewhere. Gate:
14/14 `validate_helas_mg` bit-exact across every color flow after each increment,
sharpest oracle `bbx 2→6`. Extend the sweep past the ≤60-diagram cap to the `bbx`/uux
2→6 topologies before declaring done (they were skipped here for cost, but carry the VVS
+ FFS scalar classes the fix most needs to get right).

**Locus (b) — INCREMENT 1 IMPLEMENTED: per-vertex build sign + canonical spine
extracted (2026-07-20, on branch validation-2, NOT yet committed).** Changes:
- `RootedTerm` gains a `build_sign: i8` field; `root_term` sets `coeff = term.coeff`
  (sign-free) and `build_sign` from `build_at_leg`'s returned sign. The honest tensor
  `tree` now carries no rooting-convention −1.
- `VertexTerm::build_sign()` / `VertexInfo::build_sign()` return the vertex's common
  sign, **asserting per-vertex uniformity** (panics if a vertex's terms disagree — the
  factorization guard).
- `DiagramEvalTree::build_convention_sign()` = Π over vertex nodes of `VertexInfo::
  build_sign()`.
- `root_tree` split into `root_tree_at(diagram, model, chain, root)`; `compile_single_
  diagram` builds the **canonical `VtxIdx(0)`** tree (identical to the live tree in
  production, where `choose_root == VtxIdx(0)`; rebuilt only under the soundness-test
  override) and reads *both* `spine_sign_from_flow` **and** `build_convention_sign` off
  it, folding them into `fermi_sign` alongside `diagram.sign`/`yang_mills_vvv_sign`.

**Why bit-exact in production:** moving a per-vertex ±1 from a term `coeff` to the
diagram's `fermi_sign` is exact — ±1 multiply has no rounding, and pulling a current's
common −1 out of its summed terms (`(−a)+(−b) = −(a+b)`) and out of a product of
currents is exact in IEEE. **Verified: `validate_helas_mg` 14/14 bit-exact**, all
`max_rel_diff` unchanged (gg→gg 8.25e-14, bbx 5.63e-14, ee→ttH 3.86e-13, …).

**Soundness gate: 18 → 8 failures.** Every per-diagram gross sign flip is gone. The 8
remaining: **4 benign FP** (`ee→ττH` diag4 root2 2.16e-11 — already class-(c) pre-fix;
`ee→μμττ` ×3 at ~1e-12, just over the strict `REL_TOL=1e-12`) and **4 gross `bbx 2→6`**
— but only in *whole-process* re-rooting (bbx >40 diagrams).

**Locus (b) — INCREMENT 2 CHARACTERIZED, the reversed residual (2026-07-20; probe
reverted, tree clean).** With build+spine now canonical, the only remaining
rooting-dependent sign is the runtime `reversed`. Confirmed on `bbx`: **96480
(diagram,root,helicity) checks, 0 shape violations (every re-rooting is exactly ±1, bbx
IS factorable), and the ±1 equals the *combined* gamma+ffv reversed parity with 0
mismatches.** Split counter: `gamma_vout` and `ffv_vout` each fire ~75k times and each
drive **576 of the 1152** flips. **Subtlety for the fix:** `gamma_vout` reversed is a
pure −1, but `ffv_vout` reversed *also swaps `gl↔gr`* (`-(jr·gl + jl·gr)` vs
`jl·gl + jr·gr`) — a value change, not just a sign. Yet the net per-diagram effect is
*still* exactly ±1 (= reversed parity), so the swap's value effect is root-invariant in
aggregate and only the parity matters. Therefore reversed CANNOT be extracted by naively
stripping the runtime branch (that would drop the swap and corrupt values); the fix must
make each fermion→vector sink use its **canonical** reversed flag. Design: bake the
canonical reversed bool per `GammaVout`/`FfvVout` sink at compile time (from the
canonical tree's baked adjoints — `resolve_bra_ket`'s `fo`/`fi` typing is order-
independent; only the `reversed` bool depends on operand order, which the rooting flips),
and have the runtime use the baked flag instead of re-deriving it from live operand
order. This keeps the swap+sign canonical and doubles as the perf-motivated removal of
the runtime `resolve_bra_ket` order check. This is a hot-path change (touches
`gamma_vout`/`ffv_vout` and the fused/SIMD paths) — a distinct increment, gated by the
same 14/14 bit-exact + soundness checks. The 4 benign-FP soundness failures are a
separate class-(c) `REL_TOL` question, independent of increment 2.

**Locus (b) — INCREMENT 2 IMPLEMENTED: reversed-bilinear parity extracted; ROOTING-
SOUNDNESS GATE NOW GREEN (2026-07-20, branch validation-2, NOT yet committed).**
Chosen approach: a **per-diagram scalar**, *not* a per-node baked flag. A per-node flag
can't be made canonical because a vertex is a fermion→vector sink (`GammaVout`, carrying
`reversed`) under one rooting but a fermion-continuing current (`GammaIout`/`GammaOout`,
no `reversed`) under another — live nodes have no canonical counterpart. So, mirroring
`build_convention_sign`:
- `RootedTerm` gains `reversed_sign: i8`; `term_reversed_parity(term, idx, flows)`
  computes it analytically from the *corrected* output leg + baked leg adjoints: a
  `Gamma` is a `GammaVout` sink unless rooted at one of its own fermion legs, and the
  runtime `resolve_bra_ket` reads `reversed = true` iff the first operand (UFO row index
  `i`) is a ket. `build_at_leg` returns it as a third tuple element.
- `VertexTerm/VertexInfo::reversed_sign()` (per-vertex common, asserted uniform) →
  `DiagramEvalTree::reversed_convention_sign()` = Π over vertices.
- `compile_single_diagram` folds **`canonical.reversed_convention_sign() ·
  live.reversed_convention_sign()`** into `fermi_sign`: the runtime `resolve_bra_ket`
  still applies the *live* parity `P_l` (keeping `ffv_vout`'s `gl↔gr` swap intact — the
  swap's value effect is root-invariant, only its parity flips), and this factor multiplies
  by `P_l·P_c` so the net is `P_c` (canonical). **In production `live == canonical`, so
  the factor is `P_c² = +1` — the runtime reversed branch is untouched and the amplitude
  is bit-identical.** The runtime kernel is NOT modified (removing the runtime
  `resolve_bra_ket` order check for perf is a *separate* follow-up, not needed for
  root-invariance).

**Validation.** `validate_helas_mg` **14/14 bit-exact** (all `max_rel_diff` unchanged).
Rooting-soundness gate **8 → 5 → 0 gross**: the `bbx 2→6` whole-process failures dropped
from 1.67e-2/3.11e-2 to 1.04e-12 (benign FP). The analytic `term_reversed_parity` is
validated by the gate itself — bbx fixed, no new failures, so `P_l(analytic) ==
P_runtime` everywhere. The 5 residuals were all pure double-precision momentum-sum
reassociation (≤2.2e-11); `REL_TOL` raised 1e-12 → **1e-10** (well above the 2.2e-11
floor, enormously below any O(1) sign error), and **`all_rootings_preserve_amplitude` now
passes** (0/133 re-rootings deviate). Its `#[ignore]` is kept only for the slow full
sweep; the "currently FAILS by design" framing is removed from the module + test docs.

**Status: locus (b) CLOSED.** Every rooting-dependent sign — build-convention (VVS/FFS,
increment 1), spine (increment 1), and reversed-bilinear (increment 2) — is now lifted to
`fermi_sign` at the canonical `VtxIdx(0)` rooting; the honest currents are rooting-
invariant tensors. Remaining validation-pass items (V6 branch asserts ✅ done 2026-07-21,
V7 per-flavor, the deferred Path A/Path B resolver merge, and the perf removal of the
runtime `resolve_bra_ket` order check) are independent of the sign work.

**Locus (a) — FIXED AND LANDED (2026-07-20).** The two probe write-ups below are
kept for the derivation record but are now *superseded by the implementation*. The
fix is the note's recommended architecture, with the σ_V formula pinned down:

- **Rooting mechanics → sign-neutral.** `root_lorentz::vector_out_node` now always
  emits the honest contravariant current `MetricVout` (`+V^μ`); the `NegVout`
  branch is gone. `Op::NegVout` / `Instr::NegVout` / `LorentzEvalNode::NegVout` and
  the `kernel::neg_vout*` routines are **deleted** across the eval pipeline (op,
  layout, lower, analysis, run, kernel, egraph grammar) — the entangled
  role-dependent sign is removed, not special-cased. The honest current is
  rooting-invariant by construction.
- **Antisymmetric vertex sign → rooting-invariant scalar `σ_V`**, in
  `root_diagram::yang_mills_vvv_sign`: `σ_V = (−1)^(number of Yang-Mills VVV
  vertices at diagram indices 1..)`, folded into the per-diagram `fermi_sign`. A
  "Yang-Mills VVV" is an all-vector vertex whose Lorentz structure carries a `P`
  op (`is_yang_mills_vvv`); the VVVV contact is excluded (all-vector but
  momentum-free — its −1 is the pure-metric factor already applied symmetrically).

*Why this is the correct σ_V and why it is provably safe.* Production roots at
`VtxIdx(0)`, so the VVV vertices at indices `1..` are exactly the ones that were
*sources* (rooted at a vector output) and each fired one `NegVout` = −1; the root
vertex (index 0) was the sink and fired none. Computing the sign from the **fixed
diagram** (vertex 0 = canonical root) rather than the live rooting decouples it
from the root choice. At production rooting, `σ_V·(honest current)` reproduces the
old `NegVout` amplitude **bit-for-bit** — the −1 is exact in FP and moving it from
inside the current to the diagram-level coeff is linear — so all 14
`validate_helas_mg` processes and *every* colour flow stay bit-exact by
construction (confirmed: `gg→gg` n_flows=6 8.25e-14, `uux→uux` n_flows=2 5.61e-14,
`gg→ttx` 1.89e-15, all unchanged). For any other rooting the honest current is
invariant and `σ_V` is fixed, so the product is root-invariant.

*Gate result.* `all_rootings_preserve_amplitude` drops 21→18 failures: **every
locus-(a) failure is eliminated** (`ee→WW` 4.2–6.8 → pass, `gg→ttx` diag0 3.65 →
pass). The 18 that remain are locus (b) (`ee→ττH`, `ee→μμττ`, and the reclassified
`uux/bbx` 2→6) and locus (c) (benign FP ~1e-11), both separate sessions.

*This resolves the "opposite intrinsic signs" puzzle the probes hit.* The two
baselines demand opposite signs (EW-VVV −1, `gg→ttx` ggg +1) **not** because of an
intrinsic per-species colour sign, but because the canonical `VtxIdx(0)` root lands
on the ggg vertex in `gg→ttx` diag0 (that VVV is the sink, 0 sources → +1) while it
lands on the FFV vertex in `ee→WW` (the VVV is a source, 1 source → −1). The
falsified `CAND_A`/`CAND_C` are the two constant-per-VVV rules that ignore the
`[root is VVV]` term; `σ_V = (−1)^(#VVV at idx 1..)` carries it exactly. (The colour
factorisation is genuinely rooting-invariant and untouched — the fix lives entirely
on the Lorentz side, as the root-cause analysis below concluded.)

────────────────────────────────────────────────────────────────────────

**Locus (a) — mechanism NAILED numerically (2026-07-20, second probe session).**
The first-session "arithmetic probe" reading below (kept for the record but now
partly *superseded*) held that the −1 is a mislocated *vector-propagator* sign,
fixable by making all vector-output currents share one momentum-flow sign. **That
blanket framing is falsified.** Hard per-diagram complex-amplitude numerics (via a
temporary `eval_single_diagram` probe over `ee→WW` diag0 and `gg→ttx` diag0, all
helicities, real CSV momenta) established:

1. **The re-rooting flips the per-diagram amplitude by EXACTLY −1** (ratio
   `−1.0000` for every helicity, both processes) — a clean global per-diagram sign,
   not a partial-term effect. Universally: **VVV source-mode = −(honest Yang-Mills
   current); VVV sink-mode = +(honest)** — the −1 is purely the `NegVout` node
   (`= MetricVout × −1`, momentum unchanged, same contravariant `Vector` storage, so
   it carries *no* variance/propagator role and can be folded into a coeff).
2. **The sign is NOT a function of leg position.** A baseline VVV-vertex scan across
   all 14 MG processes shows source VVVs at `out_leg=2` using `NegVout` (−1) and
   matching MG (the uux/bbx EW VVVs) — yet `gg→ttx` needs **+1** at position 2. Any
   `f(internal_leg_position)` rule is therefore ruled out. (The internal-leg position
   *is* rooting-invariant, which is why it looked promising — but it doesn't set the
   sign.)
3. **The needed sign is an intrinsic per-vertex `σ_V`**: MG wants `σ_V·(honest)`,
   with **`σ_V = −1` for EW-VVV (γWW/ZWW), `σ_V = +1` for the color-antisymmetric
   ggg (f^{abc}) vertex.** Confirmed by experiment `CAND_C` (add −1 to *every*
   VVV sink): it sends `ee→WW` root0-vs-root1 to ratio +1 **and leaves the `ee→WW`
   MG baseline bit-unchanged** (`max_rel 4.24e-14`), while **regressing `gg→ttx`
   baseline to 3.72** — precisely because the ggg sink must stay +1. Symmetrically,
   `CAND_A` (drop `NegVout` → all vector-outputs +honest) fixes the ratios but
   **breaks `ee→WW` baseline to 1.72e3**, since EW-VVV genuinely needs −honest.
   So neither a uniform source change nor a uniform sink change can work — the two
   baselines *demand opposite intrinsic signs*, resolvable only per-vertex.
4. This is the **color↔Lorentz antisymmetry interplay** (note-16 §6 `gg_to_gg`-VVVV
   class): the physical vertex is `f^{abc}·V^{μνρ}` (antisymmetric × antisymmetric =
   symmetric); vibegraph factorizes color and Lorentz and roots them separately, and
   the ggg color `f` supplies the extra sign that the EW-VVV (trivial color) lacks.
   It is **diagram/color-flow-specific** — `gg→ttx` diag0 fails one re-rooting, but
   `uux→uux` (also ggg) and `gg→gg` (VVVV+ggg) do *not* appear in the 21 failures at
   all. So `σ_V` is not simply "ggg ⇒ +1" globally; the correct rule must track the
   color-flow sign under rooting, not just the vertex species.

**Correct fix direction (for the next implementation session):** apply
`σ_V·(honest)` per VVV vertex in **both** source and sink modes, with `σ_V`
determined by the vertex's color-structure antisymmetry / the rooted color-flow
sign — *not* by rooting orientation or leg position. **Any candidate must
re-validate bit-exact across all 14 `validate_helas_mg` processes AND every color
flow** (the `gg→ttx`/`uux→uux`/`gg→gg` NCOLOR≥2 flows are the trap — a fix green on
the rooting gate can still silently break an unexercised flow). *Falsified, do not
repeat:* blanket `NegVout` removal (`CAND_A`, breaks EW), blanket sink −1 (`CAND_C`,
breaks ggg), blanket propagator −1 (breaks FFV-source), position-based `f`.

**Recommended architecture — sign-neutral rooting + pre-rooting `σ_V` (parallel to
color).** The cleanest form of the fix *separates the two concerns the current code
entangles.* Today the Lorentz sign is not derived *after* rooting — it is derived
*from* the rooting: `NegVout(−1)`-if-source / `+1`-if-sink reads the vertex's
antisymmetric sign off its rooted *role*, so it only lands at the baseline root.
Instead:

1. **Rooting mechanics → sign-neutral.** Always build the honest current
   (`MetricVout`/`+textbook`); drop the role-dependent `NegVout`. The honest current
   is *already* rooting-invariant — plain tensor contraction with consistent momenta,
   no added sign — which is exactly why `CAND_A` (drop `NegVout`) drove the `ee→WW`
   and `gg→ttx` root0-vs-root1 ratios to `+1`. (Verified only for those two; the
   multi-internal 2→6 VVVs still need checking under honest rooting.)
2. **Antisymmetric vertex sign → computed pre-rooting**, from the vertex's fixed
   ray/leg ordering (the *same* input `slot_indices` consumes on the color side), and
   applied as a rooting-invariant scalar `σ_V` on the vertex — architecturally the
   mirror of how color already derives its factor from the undirected topology.

This *deletes* the entangled `NegVout`/sink-sign logic instead of adding a second
special case, and makes `total = σ_V·honest` root-invariant by construction. **The
real work is `σ_V`'s formula, and it is NOT free:** it is a *permutation parity of
that instance's leg→slot assignment* (diagram/flow-specific — `gg→ttx` diag0 fails a
re-rooting, `uux→uux` (also ggg) fails none), the same kind of quantity
`slot_indices` computes (`3/3̄` transpose, imaginary-`f` sign). It also can't be
*only* the color parity — EW-VVV has trivial color yet needs `σ_V = −1`, so the
Lorentz structure carries an intrinsic antisymmetric sign that must be *combined*
with the color parity. Deriving that combined parity (best shared with / cross-checked
against the color `slot_indices` machinery) is the substance; the decomposition only
makes the surrounding code clean. Implementation-wise, `σ_V` is a per-vertex scalar:
source mode has `result_leg_idx`, but since `σ_V` is now rooting-*invariant* it is
cleanest to compute it once per vertex from the diagram at `root_diagram::bake_node`
(where the ray/prop/color-flow info lives) and fold it into the vertex coeff for both
the `OffShellCurrent` and `ContractAmplitude` cases.

**Root cause CONFIRMED — color and Lorentz use *different* rootings.** The color
factorization is **rooting-invariant**: `colorize_process`/`colorize_diagram`
(`helas/color/colorize.rs`) read the owned `Diagram`'s fixed `rays`/`props`
(`slot_indices`, vertices in `VtxIdx` order) and **never** call `choose_root` /
`root_tree` / `set_root_override` (grep across `helas/color/` is empty). Rooting only
drives the *Lorentz* `root_tree` walk; it does not touch the diagram's rays/props, so
the color coefficient is the same number for every root. `lower_flows` then forms each
JAMP as `Σ colorcoeff · (sym·fermi) · amp_{d,chain}` = **rooting-invariant color factor
× rooted Lorentz amplitude**. For `M_f = colorcoeff_f · amp_f` to be root-invariant
with `colorcoeff_f` fixed, `amp_f` *must* be root-invariant — but the VVV Lorentz sign
flips (source − / sink +), so the product flips by −1. That is exactly the measured
per-diagram −1. It also explains why `σ_V` is color-tied: the physical `f^{abc}·V^{μνρ}`
splits its antisymmetry between the (invariant) color `f` — including the `3/3̄` slot
transpose and imaginary-`f` coefficient sign `slot_indices` documents — and the Lorentz
`V^{μνρ}`; trivial-color EW-VVV puts the whole sign on Lorentz (−honest = `NegVout` at
baseline), ggg lets the color `f` carry part (Lorentz +honest). **Fix belongs on the
Lorentz side** (make the VVV Lorentz sign a rooting-invariant function of the vertex's
color/particle content, so `colorcoeff × amp` is root-independent and still matches MG);
rooting the *color* to match would only cancel one orientation-dependence with another —
the color factor is a genuine algebraic invariant and must stay one.

*Landed this session:* only the `P { leg } => "P{leg}"` render tweak in
`root_lorentz.rs::render` (keeps future `DiagramEval` traces legible). All probe
harness (`eval_single_diagram` visibility, `dump_rootings`/`probe_diagram_ratios`/
`scan_vvv_positions` tests, `VVV_DEBUG`/`CAND_*` env hooks) was reverted; trivially
reconstructable from this note. The failing gate stays red by design.

*Superseded first-session reading (kept for context):* rendered the `ee→W+W-` VVV
diagram rooted both ways and read *output-leg mode = −textbook via `NegVout`*,
*sink mode = +textbook*; concluded the −1 was "a vector-propagator sign misattributed
to the VVV output" and proposed attributing it to the vector `Propagate` / a shared
vector-output sign. Point (3) above shows that blanket framing fails (`gg→ttx` needs
+1, `ee→WW` needs −1 for the *same* propagator species), so the −1 is a per-vertex
color-tied sign, not a propagator sign.

### V6 — Branch-level coverage (after V5; same code territory) ✅ DONE (2026-07-21)

Rooted-tree pattern assertions per MG-pinned convention: every "pinned by
process X" claim gets a test that fails if X stops exercising that branch —
an unexercised branch silently drifting out of sync with its exercised
sibling is exactly the failure mode that produced the `gg_to_gg` VVVV bug
(note 16 §6).

**Outcome.** The rooting-convention signs V5 lifted into `fermi_sign` are the
branches that matter here (each depends on the rooted output leg; a refactor
that stopped a process exercising one would rot silently against its
still-exercised sibling — exactly the note-16 §6 mode). A census over the
default-suite processes (temporary probe, per-diagram counts at the canonical
`VtxIdx(0)` rooting) mapped which process fires each channel:

| channel | fires in (small suite) | guard process chosen |
|---|---|---|
| VVV `σ_V` (`yang_mills_vvv_sign<0`) | `e+e-→W+W-` (2), `gg→gg` (3) | `e+e-→W+W-` |
| spine (`spine_sign_from_flow<0`) | most fermion procs; `e+e-→e+e-` (2, crossed) | `e+e-→e+e-` |
| build `−1`, **VVVV pure-metric** (`build_convention_sign<0`) | **only `gg→gg`** (1) | `g g > g g` |
| build `−1`, FFS/crossed scalar-sink | `e+e-→ta+ta-H` (4) | `e+e-→ta+ta-H` |
| reversed-bilinear (`reversed_convention_sign<0`) | most fermion procs; `e+e-→mu+mu-` (2); **never** `gg→gg`/`gg→ttx` | `e+e-→mu+mu-` |

The single-process branch is the VVVV pure-metric build sign — `g g > g g` is
the *only* small process with no fermion or scalar externals, so its 4-gluon
contact is the sole source of a build sign there, and it is precisely the
note-16 §6 branch. New default-suite test
`mg_guard_processes_exercise_every_convention_channel` (`root_diagram.rs`)
asserts each channel's count `>0` for its guard process; the deeper per-channel
properties (VVV +1-uniformity + VVVV-not-counted; spine vs the `spin_map`
oracle) stay in their dedicated tests, which the census doc cross-references.

**One sub-branch no default-suite process reaches**: the VVS pure-metric `−1`
with the *scalar* leg as output (H produced from two vector chains), which only
appears in the 2→6 H classes. Pinned cheaply at the primitive level by
`root_lorentz::tests::test_root_vvs_metric_scalar_out` (the same VVS `Metric`
as `test_root_vvs_metric`, rooted at the scalar leg instead of the amplitude),
and bit-for-bit by `u u~/b b~ > … QCD=0` in `tests/validate_helas_mg.rs`. So no
slow 2→6 compile is needed in the default suite.

### V7 — Per-flavor diagram matching (optional tail)

`madgraph-diagram-cmp-per-flavor`, design moved here from `TODO.md`. An
independent, verification-heavy refactor (Python extractor + Rust matching +
JSON regen). Background: the `validate_madgraph_diagrams` reference count
uses the representative subprocess's true `NGRAPHS` from `matrix1_orig.f`,
not `MAPCONFIG(0)` (which counts the integration-channel *union* across a
P-class — e.g. 2672 vs the actual 2316 for `u u~ > u u~ l+ l- l+ l-`).
**Remaining gap**: `count_mg_style_topologies`
(`vibegraph-lib/tests/validate_madgraph_diagrams.rs`) collapses vibegraph
subprocesses into coarse particle-type classes and compares one
representative per class against the summed `total_diagrams` — fragile,
since it assumes vibegraph's first-enumerated subprocess matches MadGraph's
`matrix1` representative.

Design for the refinement (validates *all* variants, incl. the 40 of the
qq4l class):

- **Robust flavor source — the matrix-file header, not `IDUP`.** Each
  `SubProcesses/P*/matrix<N>_orig.f` carries
  `C     Process: u u~ > u u~ e+ e- e+ e- QCD=0 @1` comment lines — one per
  concrete flavor process sharing that variant's `NGRAPHS` (u/c and e/mu are
  grouped). Parse these directly: it avoids reverse-engineering MG's fragile
  `matrix<N> ↔ IDUP(I,J,K)` 3-index mapping in `leshouche.inc`.
  `extract_diagrams.py` grows a per-concrete-process
  `{in:[pdg…], out:[pdg…], ngraphs}` list (name→PDG via a bounded SM dict:
  the full token set is `a b b~ c c~ d d~ e± g h mu± s s~ t t~ ta± u u~ w± z`).
- **Rust side**: key each MG entry and each vibegraph subprocess by
  `(sorted initial PDGs, sorted final PDGs)`; compare per-subprocess
  (`set.diagrams.len()` vs `ngraphs`).
- **Known risk to resolve first**: this exposes whether vibegraph enumerates
  the *same set* of concrete subprocesses as MG's `C Process:` union — i.e.
  whether the multiparticle `p`/`l` definitions and flavor-symmetry pruning
  align. Validate on a small process (`pp_to_ll`) before the qq4l class; a
  set mismatch is a real finding, not a test bug, and needs physics judgment
  (note-12 territory: MG-convention reconciliation is a bug magnet).

## 4. Sequencing & mechanics

- **Order**: V1 → V2 → V3a → V3b; V4 free-floating (independent of
  everything); V5 → V6; V7 last, dropped to the next pass if the sprint runs
  long. V5 is cleanly separable into its own spike if needed.
- **Branch**: `validation-2`, sessions land as one commit(-set) each, merged
  to `main` at close-out.
- **Agents**: one session per agent — `validation-dev` for V1/V2/V3b/V4/V5/
  V6/V7, `feature-dev` for V3a; light sessions (V1) may run Sonnet.
  Worktrees managed manually with cd-verification.
- **Gate**: everything stays behind the 14-process `validate_helas_mg` net;
  V3b adds the σ-level net on top.
