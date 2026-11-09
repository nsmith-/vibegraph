---
type: Session Report
title: "I-C1 report: the cut_decays window rule is not IR-safe on the every-pairing |M|²"
description: "Stopped before code: an exempt identical quark can go collinear with a beam while its pair stays on the pole, and the other pairings give |M|² ∝ 1/pT² (probe: |M|²·pT² constant to 1e-4; no pole for z > d d~). Led to D2."
status: draft
tags: [cuts, decay-chains, identical-particles, ir-safety, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-08}]
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/71b5302e", title: "71b5302e, the commit probed"}
---
Manager's spot check: the probe log's ε = 1e-2 and 1e-3 rows for all three
same-flavour cases and both distinct-flavour controls, which match the figures
below. The probe source is in the scratchpad only. The figures were not
re-run.


I did not implement the window rule. I stopped as your dispatch asked ("a case where the window rule is not IR-safe, stop and report"). The brief and the backlog item both claim the rule is IR-safe because "a pair containing a soft quark has m → 0 and is never in the window". That argument covers only the soft limit. It does not cover an exempt decay product going **collinear with a beam**: the pair can stay exactly on the pole while that quark's pT goes to zero. In the other kept pairings, the same quark is a massless core quark. Their g → q q̄ initial-state splitting then gives |M|² ∝ 1/pT², so σ diverges logarithmically. This is H-C2's divergence again. It survives in the window rule's own region, and a measurement below shows it survives even when every other pairing is outside its window.

**Branch state:** `sprinti-c1` is unchanged at `71b5302e`. There are no commits and nothing new to push. `git diff --stat` is empty and there are no untracked files: the scratch probe was removed from `vibegraph-lib/tests/`. Nothing under `research/kb/` or the manifest was touched. No heavy job was run: no sweep, no MadGraph launch, no workspace test or `validate.sh`. The only build was one integration test (lean settings, 3 jobs, about 4.5 min). `target/` is 801 MB.

### 1. Every `from_decay` reader (scope item 1)
I re-read the pinned source in the worktree's submodule copy (`grep -n "from_decay\|do_cuts" setcuts.f cuts.f …`). The list matches H-C2's table:
- `do_cuts` is set at `setcuts.f:202-203`.
- Single-leg class cuts: `:269`. Per-PDG cuts: `:322`, which reads `cut_decays` and `from_decay` directly.
- ΔR: `:344`. Pair mass: `:392`. Per-PDG pair mass: `:441`, `:448`.
- The `smin` terms: `:538`, `:579-582`, `:610`, `:643`, `:683`.
- The grouping-consistency check: `:714`, `:741`.
- `setxqcuts`: `:943`, `:951`.
- Merging-cut membership: `setcuts.f:245`, and `cuts.f:553`, `:565`, `:652`, `:667`, `:676` (ktdurham and ptlund, both refused here).
- `ptll` and `mmnl` do not read `do_cuts`.
- One point for any later per-point design: the `smin` and xqcut readers are compile-time floors. A per-point exemption would need every floor taken over the weakest pairing, as H-K1's UnionBounds does for groups.

### 2. The measurement (the oracle)
**Probe.** A scratch integration test, not committed: `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic1/ic1_window_probe.rs`. To rerun it, copy it to `vibegraph-lib/tests/` and run:
```
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=3 cargo test -p vibegraph-lib --test ic1_window_probe -- --nocapture
```
The log is `/tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/ic1/ic1-probe2.log`. It ran on `71b5302e` with the default SM: m_Z = 91.188, Γ_Z = 2.441404, window ±36.62 GeV.

**Kinematics.** Process `g g > z u u~, z > u u~`, final order [u, ū, u, ū], with the coherent |M|² summed over all four pairings.
- Leg 0 (u) has E₀ fixed and pT = ε, along beam a (+z).
- Leg 1 (ū) has a fixed direction, and its energy is set so that m₀₁ = M_Z exactly. Pairing {0,1} is on the pole, so it is the closest pairing under any ranking, and the window rule exempts legs 0 and 1.
- Legs 2 and 3 carry the rest. At every ε they pass `ptj 20`, `etaj 5` and `drjj 0.4` (pT ≥ 22.4, |y| ≤ 2.08, ΔR₂₃ ≥ 2.7).
- So every point passes the window rule's cuts.
- `*` in the output marks a pair mass inside the window.

**Case 3: the only in-window pairings have leg 0 as a decay product.** The pairings where leg 0 is a core quark, {2,3} and {2,1}, are both outside the window.
```
--- m23 = 180 th1 = 2.6 e0 = 40
eps      1 |M|^2 5.840452e-3 |M|^2*eps^2 5.840452e-3 ... m01 91.19* m23 180.00 m03 117.15* m21 143.59 pT2 24.02 pT3 22.67 dR23 4.44
eps    0.1 |M|^2 1.214399e-1 |M|^2*eps^2 1.214399e-3 ... m01 91.19* m23 180.00 m03 117.15* m21 142.88
eps   0.01 |M|^2 1.168819e1  |M|^2*eps^2 1.168819e-3 ... m01 91.19* m23 180.00 m03 117.15* m21 142.81
eps  0.001 |M|^2 1.168707e3  |M|^2*eps^2 1.168707e-3 ... m01 91.19* m23 180.00 m03 117.15* m21 142.81
```
|M|²·ε² is constant to 0.01% over two decades, so |M|² ∝ 1/pT₀².

**Case 2: the double-window region.** Here m₂₃ = 111.19 and m₂₁ = 86.7 are also inside the window, but {0,1} is still the closest pairing. The coefficient is about 6× larger: 6.538e-2 at ε = 0.01 and 6.518e-2 at ε = 0.001.

**Case 1** (m₂₃ = 160): 1.2201e-2 at ε = 0.01 and 1.2222e-2 at ε = 0.001.

**Control: distinct flavour, `g g > z u u~, z > d d~`, same kinematics.** Leg 0 is a d, which is a decay product in the only pairing. |M|² stays finite (1.94e-3 and 1.94e-3; 4.43e-3 and 4.43e-3), and |M|²·ε² falls as ε² (1.9e-7, then 1.9e-9). The 1/pT² term exists only where the exempt leg is a core quark of another pairing.

**What the oracle does and does not show.** It shows the integrand is unbounded on the region the rule accepts. The measure goes as pT dpT, so σ goes as ∫dpT²/pT², which diverges. The coefficient is roughly suppressed by the off-shell Breit–Wigner of the other pairings: in case 3 the singular term equals the regular one near pT ≈ 0.5 GeV. My rough estimate is a few 1e-4 of σ per unit of ln pT², which is the size of the ~0.2% interference scale the gate would read. So a 5-seed σ comparison at 0.2% could pass by accident: the channels' spacelike floor would cap the logarithm silently. That is the "confidently wrong σ" case. The oracle does not size what a VEGAS run would report.

### 3. Consequences for the brief's scope
- **Item 3 is false as stated.** The claim to prove, "it holds for every exempted leg", fails for any exempt identical massless quark that is a core parton in another kept pairing, through the initial-collinear limit. The soft half of the argument is correct.
- **Item 4 would hide the divergence.** Restating `refuse_ir_unsafe_exemption` over the exempting pairing's own diagrams would accept these points, because the 1/pT² comes from the *other* pairings' diagrams, which stay in the coherent |M|². I-C2's guard read over every kept diagram (`refuse_unregulated_limits` with the exempt legs as `uncut_decay`) refuses it correctly. Applied to the window rule's per-pairing masks, that guard refuses every identical-quark card the rule was meant to fix. H-C3's Found 2 ("rebuild the guard over the exempting pairing's diagrams") is therefore unsound: it would remove a correct refusal.
- **No kinematic rule fixes this on the every-pairing |M|².** Any rule that leaves an identical massless quark entirely uncut has this problem. The observable "four identical-flavour quarks, two of them uncut" is IR-divergent at tree level. MadGraph's number is finite only because it drops the other pairings. A regulator on the exempt legs (a tiny pT or a rapidity maximum) makes σ finite but logarithmically dependent on an arbitrary parameter, and still unlike MadGraph.

### Options for the user (not decided by me)
- (a) **Keep today's rule** (cut every leg that is not a decay product in every pairing), the registered −26% / −6.8% deviation. It is IR-safe. Fix the item's and the docs' "IR-safe" claim.
- (b) **Refuse** `cut_decays = F` cards with identical massless partons across a decay, under the hard-error rule. This refuses the CMS `g g > z j j, z > j j` style card.
- (c) **Per-pairing |M|²**: H-C2's option 2, the pairing-weighted cut w_p = |M_p|²/Σ|M_q|², or compat site (d). This is the only route to MadGraph's number, and it needs the evaluator split. It is out of this sprint's scope.

### Corrections to the brief and the item
1. The IR-safety premise ("a pair containing a soft quark has m → 0") covers soft limits only. It is repeated in `cut-decays-identical-quarks-cut-a-stricter-observable`, in the `madgraph-defects.md` row ("The intended fix is a kinematic, IR-safe rule …") and in D5. All of these need correcting.
2. "MadGraph 150.61 at 5 seeds" (H-K1 §5b) was run with LHAPDF 247000, not `nn23lo1`. The gate's local MadGraph runs would carry H-D2's PDF offset (+0.32% on Drell-Yan) against it, so they are not directly comparable to that number.
3. The `ir-unsafe-exemption-guard-sees-coloured-legs-only` item, as written, needs a rule that exempts by pairing. With that rule shelved, its test case ("exempts a massless colourless core leg and is refused") cannot be built from a real card.

### Gates
None were run, since no source changed. `git diff --stat` on `sprinti-c1` is empty, and HEAD is still `71b5302e`.

### Found
1. **The window rule is IR-divergent through the initial-collinear limit of an exempt identical quark** (§2). What is wrong: the item's `closes_when` cannot be met as written. What would close it: the user choosing among (a), (b) and (c) above, then rewriting both items and the `madgraph-defects` row to match.
2. **I-C2's guard may already cover the colourless half of the guard item for the fixed mask.** `refuse_unregulated_limits` reads every kept diagram's `MasslessLimit`s, and an exempt (`uncut_decay`) leg regulates nothing in `limit_regulated`, so photons and massless leptons are covered whenever their limit is one `diagram_massless_limits` recognises. That makes `massless_coloured_core` / `refuse_ir_unsafe_exemption` partly redundant. This is inferred from the code and not measured. What would close it: a unit test with a synthetic `ForcedResonances` that exempts a massless lepton which is a core leg with an `InitialCollinear` or `FinalCollinear` limit, checking that `compile_with` refuses it. Then decide whether to keep or delete the coloured-only guard.
3. **H-C3's Found 2 recommendation is unsound** (§3, item 4). It should not be carried into any future design, and the guard item's text inherits it ("or is restated over the exempting pairing's |M|² support").
4. **The probe could be banked** as a regression test that the coherent |M|² of `g g > z u u~, z > u u~` has a 1/pT² pole for an on-pole pair's quark (and the distinct-flavour control has none). This would pin the reason any future exemption rule must be refused. What would close it: commit a trimmed version of `ic1_window_probe.rs` asserting that |M|²·ε² is constant to 1e-3 over ε = 1e-2 to 1e-3, and goes to 0 for `z > d d~`.