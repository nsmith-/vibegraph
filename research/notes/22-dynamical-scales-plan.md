# `dynamical-scales` — running couplings and the per-event scale (sprint plan)

**✅ CLOSED 2026-07-27, merged to `main` (ff, HEAD `3a98900`).** D1–D4 landed;
D5 stayed optional and was not needed. Close-out with session outcomes at the
end of this note.

Feature sprint, 4 sessions + 1 optional (D1–D4, D5 optional). Sequenced **before**
Sprint B `event-output-lhef`, because the per-event renormalisation scale and
`αs` at that scale are LHE `<event>`-line fields (`SCALUP`, `AQCDUP`), so B would
otherwise have to write placeholders and revisit the writer.

**What it unblocks.** The three QCD `validate_sigma` rows (`gg_to_ttx`,
`uux_to_uux`, `gg_to_gg`) are `Plan::Info` today purely because vibegraph
evaluates `αs` at the param-card value while MadGraph runs it to a per-event
scale — the "running-αs gap". This sprint closes it and promotes them to hard
σ gates, and gives the hadronic path a physical `μF`.

---

## 1. What MadGraph actually does (survey, 2026-07-26)

Read off the MG5 3.5.7 LO template in `.pixi/envs/madgraph/MG5_aMC/Template/LO/`
and the banked runs in `validation/madgraph/output/*/`. Every claim below is
checkable against a file cited here; the sessions turn each into a test.

### 1.1 `αs` is MG's own RGE, not the PDF's

`Source/alfas_functions.f` — `ALPHAS(Q)` runs `asmz` down/up with
`NEWTON1`, an `nloop`-order β-function Newton solve (`TOL = 5d-4`) with fixed
flavour thresholds `CMASS = 1.42`, `BMASS = 4.7`, `ZMASS = 91.188`, switching
`nf = 5 → 4 → 3` at the b- and c-thresholds. Even with an LHAPDF grid in play,
the matrix element's `αs` comes from *this* routine, not from `alphasPDF`.

`asmz` and `nloop` come from (`Source/setrun.f:129`, `Source/PDF/pdfwrap.f`):

| beams | `asmz` | `nloop` |
|---|---|---|
| `lpp = 0` (no PDF) | param card `aS` (via `G²/16π`) | 2 |
| `lpp ≠ 0` | **the PDF label's tabulated value**, e.g. `nn23lo1 → 0.130` | 2 (default; some labels override) |

The second row is a trap worth naming: with a PDF in use MG **overrides the param
card's `aS(M_Z)`** (`setrun.f:132–136`, "A PDF is used, so alpha_s(MZ) is going to
be modified"). A hadronic QCD gate that used the card's `0.118` would be wrong
before any running is considered.

Per event, MG recomputes `G = √(4π·αs(μR))` and every `aS`-dependent coupling
(`update_as_param`). Our [`ParameterSet::recompute`] + coupling re-evaluation is
the same operation, already implemented at the model layer.

### 1.2 The scale itself

`SubProcesses/setscales.f` implements `dynamical_scale_choice` 1–5 as closed
forms over the final-state momenta (total `E_T`; Σ transverse mass; Σ`m_T`/2;
`√ŝ`; decaying-particle mass). The **default `-1`** is not in that file: it is the
kT-clustering scale — cluster the event down to a 2→2 topology and read the scale
off the central vertex (`SubProcesses/reweight.f:551 setclscales`, with
`cluster.f`). Reproducing the general clustering is a subproject of its own and is
**out of scope**; what is in scope is the set of cases the banked runs pin, where
clustering is degenerate.

There is one factorisation scale **per beam** — `q2fact(1)`, `q2fact(2)`, with
independent `fixed_fac_scale1` / `fixed_fac_scale2` guards; the older single
`fixed_fac_scale` fills in for whichever of them a card leaves alone.

**Correction (D2): where `scalefact` lands is not uniform.** `cuts.f:1220` calls
`set_ren_scale` only when `fixed_ren_scale` is false, so a *fixed* scale never
sees `scalefact` at all. A dynamic 1–5 scale sees it once, and `set_fac_scale`
squares that already-multiplied value into `q2fact`, so `μF` carries exactly one
factor too. Under `-1` it is `reweight.f` that applies it — once to `scale`
(`:1148`), once to each `q2fact` (`:1118`) — except in the branch where no colour
line reaches the beams, where `q2fact(2) = scalefact²·q2fact(1)` is built from an
already-scaled `q2fact(1)` and beam 2 picks the factor up **twice** (`:1215`).
Every banked run has `scalefact = 1.0`, so none of this is pinned by reference
data; it is pinned only against a reading of the Fortran, in `coupling::scales`'
unit tests.

All **20** banked runs use `dynamical_scale_choice = -1`, `fixed_ren_scale =
False`, `fixed_fac_scale = False`, `scalefact = 1.0`.

### 1.3 What `-1` collapses to, measured on the banked events

Everything below is now pinned per-event over all 20 runs by D2's
`validate_scales.rs`; the original entries are superseded where they disagree.

The whole of `-1` for these runs turns on one function, `djb`
(`Source/kin_functions.f:397`) — the scale a single leg carries with respect to
the beams. **With a PDF on either beam it is the transverse mass squared,
`(E − p_z)(E + p_z)`; with neither beam carrying a PDF it is `E²`.** That switch,
not the beam energy, is what makes fixed-beam runs look constant.

| run | beams | `-1` collapses to |
|---|---|---|
| `gg_to_gg`, `gg_to_ttx`, `uux_to_uux` | `lpp = 0` | `(djb₃·djb₄)^¼ = √ŝ/2 = 250` — a t-channel merges a beam with a leg first, and `djb = E²` in the partonic CM makes every vertex the same number. `uux_to_uux` is the exception that pins the tie-break: flavour locks each leg to one beam, and in **10 of its 10000 events** both locked pairs are crossed, so `cluster.f`'s `1 + 1e-6` inflation survives and `SCALUP` reads `250.0001` |
| `ee_to_ee`, `ee_to_wpwm` | `lpp = 0` | same form, `250` every event — but with colourless beams, which changes which vertex the scale is read off |
| `ee_to_mumu`, `ee_to_ttx`, `ee_to_zh`, `ee_to_tatah`, `uux_to_mumu` | `lpp = 0` | `√(djb(Σp_out)) = √ŝ = 500` every event: no diagram lets a beam reach the final state, so it collapses to one propagator. `ee_to_ttx` is the discriminating row — a *coloured* final state that still takes this branch, because the central line is a `γ/Z` |
| `pp_to_bb`, `pp_to_bb_qcd2` | `lpp = 1` | `(djb₃·djb₄)^¼ = √(m_T(b)·m_T(b̄))`, e.g. `5.1491350` on the first event |
| `pp_to_ll`, `pp_to_ll_qcd0` | `lpp = 1` | `√(djb(p₃+p₄)) = m(ℓℓ)`, and `μF` on **both** beams is the same number |
| `pp_to_llj{,_qcd2_qed2}`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`, `bbx_to_ccx_emmm_qcd0`, `uux_to_ccx_emmm_qcd0` | both | no closed form — the general clustering |

**Correction: "`lpp = 0` ⇒ constant scale" is false.** `bbx_to_ccx_emmm_qcd0` has
`lpp = 0` and a fixed `√ŝ = 500` and still shows 8720 distinct `SCALUP` values
over 10k events; `ee_to_ttx` has `lpp = 0` and sits at `500`, not `250`. The
constancy is a property of a 2→2 with equal-mass legs, not of the beams.

**Correction: `pp_to_bb_qcd2` does not take the `mt2last` branch.** For its `g g`
channel a beam–leg merge wins the clustering outright, so the geometric-mean
comment at `reweight.f:1012` is not what produces `5.1491350`; the two routes
agree because the legs are equal-mass. Which also means the *form* of the
geometric mean is unpinned by any banked run: for a 2→2 the two transverse momenta
are equal and opposite, so equal-mass legs make `(djb₃·djb₄)^¼` and either leg's
own `√djb` the same number.

So the three fixed-beam QCD σ rows need only a *correct constant* `αs(250)`, not
the full dynamic machinery. The dynamic path is what the hadronic rows and the LHE
`SCALUP` field need.

### 1.4 The oracle this sprint gets to use

Every banked run has `Events/run_01/unweighted_events.lhe.gz` with 10k events.
Each `<event>` line carries `SCALUP`, `AQEDUP`, **`AQCDUP`** alongside the
momenta. That is a **per-event, finest-level oracle** for all three new
quantities — scale function, `αs(μ)`, and the `μF` fed to the PDF — not a σ-level
backstop. Per AGENTS.md's validation regime this is the gate to build first; σ
agreement is the coarse confirmation afterwards.

**Correction (D2): `<mgrwt>` exists in only 6 of the 20 runs**, not all of them —
it is written under `use_syst`, which is set in `pp_to_bb`, `pp_to_bb_qcd2`,
`pp_to_ll`, `pp_to_ll_qcd0`, `pp_to_llj`, `pp_to_llj_qcd2_qed2`. Those six give a
direct per-event `μR` (`<rscale>`, `E15.8`) and per-beam `μF` (`<pdfrwt>`:
flavour, `x`, `μF`, also `E15.8`). The other fourteen are pinned by `SCALUP`
(`e15.7`) plus `AQCDUP`.

**Correction (D2): `SCALUP` is the factorisation scale, not `μR`.** `unwgt.f:686`
fills it with `sqrt(max(q2fact(1), q2fact(2)))`. It doubles as `μR` only where the
clustering reads both off the same vertex — true in 18 of the 20 runs and false in
the two `2→6` ones, which is the mechanism behind D1's asserted
`SCALUP_IS_THE_RENORMALISATION_SCALE` partition. Independently of any scale field,
`AQCDUP` recovers `μR` to about `1e-6` relative (`dαs/αs ≈ −0.1·dQ/Q` with seven
printed digits), which is the second oracle D2 uses for the fourteen runs without
`<mgrwt>`.

(`AQEDUP = 7.5467710e-3 = 1/132.507` in every run: **α_EW does not run** in MG's
LO path. Restricting dynamic running to QCD, as proposed, matches MG exactly.)

---

## 2. Design decision: where the running coupling multiplies in

The proposal was to have each `Diagram` carry its coupling orders `n` and to
scale diagram `d` by `(αs(μ)/αs(μ₀))^{n_d/2}` "right before summing diagrams".
**That seam no longer exists.** `lower_flows` (`helas/eval/lower.rs:58`) builds
one JAMP per colour flow as `Σ_d coeff_d · amp_d` and the whole thing is then
hash-consed, constant-folded, helicity-expanded and CSE'd into a single arena;
per-diagram identity is gone by the time anything is summed. Restoring it means
splitting the root into per-(flow, order) JAMPs — doable (the partition is on
`terms` in `lower_flows`, CSE below is unaffected), but it perturbs the compiled
core, the JAMP dumps, and the ZEROAMP/helicity pruning that the whole MG gate
rides on.

**Recommended instead: rescale the constant pool.** `Folded::pools()`
(`helas/eval/fold.rs:497`) resolves the two numeric pools — `pool_c` couplings,
`pool_f` masses/widths/coeffs, plus the folded composites replayed through
`const_ast` — from an `EvaluatedModel`, and `BoundAmplitude` owns the result. A
scale change is exactly a change of those pools. Two implementations, and the
plan takes both:

1. **Reference path** — re-run `EvaluatedModel::recompute("aS")` + `pools()`.
   Exact by construction, general, and slow enough to want off the hot path.
2. **Fast path** — tag each pool entry with its power of `G`. Every SM tree
   coupling is a monomial in `G` (`GC_10 = -G`, `GC_12 = i·G²`, …) and a product
   of monomials is a monomial, so the tag propagates through `const_ast`; a *sum*
   of unequal powers is not, and is detected rather than assumed. Then a scale
   change is `consts[i] *= r^{n_i}` with `r = G(μ)/G(μ₀)` — a handful of
   multiplies, no allocation, no re-evaluation.

Path 2 is validated **against path 1** at random `αs` (bit-exact or it falls back),
which is the strongest available oracle and costs nothing to keep running. Where
the tag cannot be established the entry falls back to path 1 — never to a guess.

This choice also means **per-diagram coupling orders are not required** for the
feature: the power of `G` lives in each vertex's coupling constant, and a
diagram's total power emerges from the product automatically. D5 keeps the
diagram-order bookkeeping as an *independent cross-check* of the tags (and for
its own downstream uses), not as a dependency.

**Threading constraint.** `VegasGrid::adapt_parallel` exists and the pools are
read on the hot path, so mutable per-event pools must live in per-thread state
(alongside `ScratchSpace`), not behind a shared `&`. Settle this in D3, not later.

---

## 3. Sessions

| Session | Scope | Gate |
|---|---|---|
| **D1 `alphas-running`** | Port `ALPHAS`/`NEWTON1` (`nloop`-loop β-function, `TOL = 5e-4` Newton iteration, `c`/`b` thresholds, `nf` switching) into `pdf/alphas.rs` or a new `coupling/` module. Add the `asmz`/`nloop` source rule of §1.1, including the PDF-label override, driven off the run card (`lpp`, `pdlabel`, `lhaid`). | **(a)** bit-level vs a compiled `alfas_functions.f` driver (new pixi task in the `madgraph` env) over a `Q` grid straddling both thresholds and both sides of `M_Z`, at `nloop ∈ {1,2}`; **(b)** the banked LHE `AQCDUP` field for all 5 QCD runs, to its printed precision |
| **D2 `scale-choice`** | `ScaleChoice` compiled from the run card (`dynamical_scale_choice`, `fixed_ren_scale`, `fixed_fac_scale{,1,2}`, `scale`, `dsqrt_q2fact1/2`, `scalefact`): closed forms 1–5 + fixed + the `-1` cases §1.3 pins. Everything else returns an explicit `Unsupported` error — **no silent fallback**, since a wrong scale is a smooth σ shift, exactly the failure class the sampler sprint's lesson warns about. | Per-event replay of the banked `.lhe`: recompute `μR` (and per-beam `μF` from `<pdfrwt>`) from the event momenta and match `SCALUP`/`<rscale>` for every event of `gg_to_gg`, `gg_to_ttx`, `uux_to_uux`, `pp_to_bb_qcd2`, `pp_to_ll_qcd0`. Any run whose `-1` branch we do not implement must be *listed as unsupported* in the test, not skipped silently |
| **D3 `scale-aware-couplings`** | §2: `EvaluatedModel::set_alpha_s` (reference path) + `G`-power tagging of `pool_c`/`pool_f`/`const_ast` composites with verified fallback (fast path) + per-thread pool ownership + `BoundAmplitude` API for a per-event scale. | **(a)** fast path bit-exact vs reference path over ≥100 random `αs`, all 14 MG processes; **(b)** at `αs = αs_ref`, `|M|²` bit-identical to today — `validate_helas_mg` untouched at 1e-12; **(c)** a synthetic non-monomial coupling triggers the fallback (negative test); **(d)** `eval_strategies` shows no regression at fixed scale, and the per-event rescale cost is reported |
| **D4 `dynamic-scale-in-integrator`** | Plumb D1–D3 into `FixedBeamIntegrand` and `DrellYanIntegrand`: per point, momenta → `μR`, `μF`; `μF` into the PDF calls, `μR` into the coupling rescale, both before the ME. Short-circuit the constant-scale case (`lpp = 0`) so fixed-beam runs pay nothing per event. | **(a)** `gg_to_ttx`, `uux_to_uux`, `gg_to_gg` σ rows `Plan::Info → Plan::Gate` vs banked MG σ, pull-based, **over a seed sweep** (not a fixed seed — note 21 close-out); **(b)** DY σ re-validated with a *dynamic* `μF`: the banked 0.14%/0.07% numbers will move, and the new numbers are re-derived honestly rather than the tolerance loosened |
| **D5 `diagram-coupling-orders`** *(optional, parallel)* | `Diagram.orders` populated in `from_view` from feyngraph's per-diagram order, plus a model-level notion of *which* orders run (derive it: an order whose couplings depend transitively on `aS` — `ParameterSet::rdeps` already has the graph). `coupling_orders.py` parsing already exists (`ufo/mod.rs:41`, hierarchy for the WEIGHTED filter) and only needs extending. | **(a)** feyngraph's per-diagram order == Σ over the diagram's vertices of the UFO coupling orders, for every diagram of all 14 processes; **(b)** the D3 `G`-power tags agree with the per-diagram QCD orders — two independently-derived answers to the same question |

**Order:** D1, D2, D3 are mutually independent and can run in parallel; D4 needs
all three. D5 pairs with D3 (its gate (b) is a D3 cross-check) but blocks nothing.

---

## 4. Risks and things to not get wrong

- **The `aS(M_Z)` override (§1.1).** Silent, ~10% on a QCD σ, and invisible to
  every existing test. D1's gate (b) catches it because `AQCDUP` is banked.
- **`-1` in general needs clustering.** We implement the degenerate cases and
  refuse the rest. The refusal must be loud: an unimplemented scale that quietly
  returns `√ŝ` would produce a plausible, wrong, smooth σ.
- **DY must *not* move (correction, D2).** The banked hadronic σ reference — the
  0.14% / 0.07% numbers — was generated with
  `validation/madgraph/dy13_default_run_card.dat` and `dy13_mmll_run_card.dat`,
  and **both set `fixed_ren_scale = True` and `fixed_fac_scale = True` at
  `91.188`**. Honouring the fixed branches therefore leaves those numbers exactly
  where they are; anything that moves them is a bug in the fixed branch, not an
  expected re-derivation. `validate_scales.rs` asserts both cards still compile to
  constants. Note that `vibegraph-lib/tests/data/run_card_dy.dat` disagrees with
  them (`fixed_ren_scale = False`, `fixed_fac_scale = True`) — asserted, not
  silently aligned, so a future disagreement says which card it came from.
- **The fixed branches are load-bearing.** They are not a rarely-taken path: the
  cards behind the banked hadronic reference take them, while *no* banked LHE run
  does. So the fixed branches and the `-1` branches are pinned by disjoint
  evidence, and neither gate covers the other.
- **Hot-path cost.** Per-event coupling rescale sits inside the VEGAS loop.
  Measure it (`eval_strategies`, `profile-sigma`); the fast path exists so that
  the answer is "negligible", but that must be shown, not assumed.
- **Parallelism.** Pools become per-event mutable state; they must be per-thread
  (§2) or `adapt_parallel` silently races.
- **Scale ≠ only `αs`.** The same choice drives `μF`; treat them as one object
  with two outputs (MG allows them to differ per beam — `q2fact(1)`, `q2fact(2)`).
- **`pdlabel = lhapdf` (for D4).** D1's `RunningAlphaS::from_run_card` refuses it,
  because MadGraph then links `alfas_functions_lhapdf.f` and forwards to
  `alphasPDF`. Both `dy13` cards and `run_card_dy.dat` use it. DY at `qcd0` has no
  `αs` in the matrix element, so this only bites if D4 constructs the coupling
  unconditionally rather than on demand.
- **`μF ≥ 2 GeV` is an event veto, not a scale (for D4).** `reweight.f:1185` makes
  `setclscales` *fail* — rejecting the phase-space point — when a hadronic beam's
  `q2fact` falls below `4`. `coupling::scales` reports the scale and does not
  implement the veto; a hadronic run that can reach `μF < 2` needs it applied
  alongside the cuts or its σ will differ from MadGraph's.

## 5. Out of scope

- General kT clustering for `dynamical_scale_choice = -1` (its own subproject; it
  is also what MLM matching would need, so it belongs with that if ever).
- Running `α_EW` — MG's LO path does not run it (§1.4), so neither do we.
- Scale-variation systematics (`use_syst` reweighting, the `<rwgt>` block); the
  banked events carry them and they are a natural later feature, but nothing here
  depends on them.
- Squared-order splitting of the amplitude (per-(flow, order) JAMPs). §2 explains
  why the sprint avoids needing it.

## 6. Execution notes

Same regime as the last three sprints: **`feature-dev`** agent (Opus; never
general-purpose), one session per agent, worktrees pre-created off `main` by hand
with the validation data COW-cloned, hard `cd`-verify before each agent acts.
Every session stays behind the 14-process `validate_helas_mg` bit-exact net.

---

## Sprint close-out (2026-07-27) ✅

All four required sessions landed on branch `dynamical-scales` (HEAD `5b1258f`)
and fast-forwarded onto `main` (HEAD `3a98900`). D5 was not needed — §2 explains
why per-diagram coupling orders are not a dependency: the power of `G` lives in
each vertex's coupling and a diagram's total power emerges from the product.

### Session outcomes

**D1 `alphas-running` ✅** — `coupling::alphas` ports `ALPHAS`/`NEWTON1`
bit-exactly against MG's own Fortran (792-point grid, `nloop` 1–3) and resolves
`asmz`/`nloop` from the run card, gated on the `AQCDUP` field of 180k banked
LHE events.

**D2 `scale-choice` ✅** — `coupling::scales` compiles a run card into `μR` and a
**per-beam** `μF`: the fixed branches, `dynamical_scale_choice` 1–5, and the
`-1` clustering cases that collapse to a closed form (a t-channel 2→2 with
equal-`djb` legs; an s-channel-only tree at any multiplicity), with everything
else an explicit refusal. Gated per event on the banked LHE at the fields' own
printed precision — 400k comparisons over 140k events in 14 runs, worst 1.000
of budget — and the 6 runs needing the general kT clustering are asserted as
refused, not skipped. D2's corrections to the survey are folded into
§1.2/§1.3/§1.4/§4 above; the one that mattered downstream is that the banked
hadronic σ reference cards fix *both* scales at 91.188, so those numbers must
not move — and they did not.

**D3 `scale-aware-couplings` ✅** — `ScaleAwareAmplitude`
(`helas/eval/rescale.rs`) owns a bound amplitude's constant pools and moves them
to a per-event `αs` two ways: re-evaluating the model
(`EvaluatedModel::set_alpha_s` + `Folded::pools`, exact for any parameter graph)
or scaling each entry by `rⁿ` from its power of `G`, read symbolically off the
UFO expressions and propagated through the folded constant subgraph (`Mul` adds
exponents, `Add` requires them equal). A sum of unequal powers, a function of
`G`, or any other `aS`-driven parameter goes untagged and sends the whole
amplitude down the reference path. Gated entry by entry, scaling against
reference, at 100 random `αs` for all 14 amplitude-gate processes
(`validate-scale-couplings`): every pool entry tagged, no process needs the
fallback, worst 5 ulp — bit equality is unreachable, the two paths being
different floating-point routes to the same value rather than the same
expression — and the pools return bit-for-bit to the bound ones at the card's
own `αs`, which is what leaves `validate_helas_mg` untouched. Only 1–3 entries
per process carry a power of `G` (11 of 14 carry none), so `set_alpha_s` costs
12.2 ns against `gg_to_gg`'s 1.56 µs per point, and 3.2 ns on an amplitude with
no strong coupling. Pools are per-thread by ownership (`fork`), never shared
mutably.

**D4 `dynamic-scale-in-integrator` ✅** — D1–D3 wired into `FixedBeamIntegrand`
and `DrellYanIntegrand`. Per point the momenta give `μR` and a per-beam `μF`;
`μF` feeds the two PDF calls separately (`q2fact(1)`/`q2fact(2)` can differ),
`μR` feeds the coupling move, both before the matrix element. The
`ClusterTopology` the `-1` scale needs is **derived** from the process rather
than declared per run: `coupling::topology` reads the t-channel from the
diagrams' momentum routing (`Prop::is_spacelike` — vertex adjacency alone would
miss `e⁺e⁻ → μ⁺μ⁻τ⁺τ⁻`'s `ZZ` diagram), the beam↔leg merge mask from vertex
adjacency, colour from the model, and `isjet` from `maxjetflavor`; it reproduces
every declaration `validate_scales.rs` makes by hand for the processes the σ
gate covers. Short-circuits: a fully fixed prescription resolves once at setup
and reads no kinematics, a fixed-beam process whose matrix element carries no
strong coupling compiles no prescription at all (which is also what keeps
Drell-Yan away from D1's `pdlabel = lhapdf` refusal), and `αs(μR)` is memoised
across repeats. Cost `probe_scale_cost`: ~100 ns/point against a 0.5–1.7 µs
matrix element.

### Headline

**The three QCD σ rows are now hard GATEs**: `gg_to_ttx` pull +0.20 (rel
4.9e-4), `gg_to_gg` −0.64 (2.0e-3), `uux_to_uux` −0.63 (1.6e-3), each stable
over five seeds with the deviation shrinking as the budget grows
(`probe_qcd_seed_stability`). Drell-Yan did not move, as §4 requires — both
reference cards fix both scales at 91.188, now asserted in
`validate_hadronic.rs` rather than assumed.

**Two MadGraph defects found first-hand**, both written up in note 07:
`unwgt.f:694-695` truncates π to eight digits when filling the LHE `AQCDUP` and
`AQEDUP` fields while `g` was built from full-precision π — a systematic
`+1.7e-8` baked in before printing, so it is not an artifact of the seven-digit
output format and would survive widening the field; and the `-1` scale MadGraph
writes as `SCALUP` is the *factorisation* scale (`unwgt.f:686`,
`sqrt(max(q2fact(1), q2fact(2)))`), which parts company with `μR` on ≥2→3
topologies — inverting `αs` against `AQCDUP` puts the true `μR` at 0.50–1.00 of
the printed `SCALUP` on the two 2→6 runs.

### Found along the way — the missing identical-particle symmetry factor

Promoting `gg_to_gg` uncovered a **missing final-state identical-particle
symmetry factor** in `FixedBeamIntegrand`: `dΦ_n` counts both orderings of the
two outgoing gluons, so its σ was exactly twice MadGraph's. It is the only
MG-validated process with a repeated outgoing particle, and its `Plan::Info`
status had been hiding the factor of 2 behind the running-αs gap.
`final_state_symmetry_factor` now derives `1/Π_s n_s!` from the evaluator's
outgoing legs — but it landed as a per-integrand scalar, which is the wrong
home; the follow-up (make the factor a property of the phase-space map, which
is what over-counts) is filed in TODO.md as `identical-particle-permutation`.

### Deferred / open at close (carried forward in TODO.md, not regressions)

| Item | Why it is open | Where |
|---|---|---|
| General kT clustering for `dynamical_scale_choice = -1` | Out of scope by §5, and a subproject in its own right (it is also what MLM matching needs). 6 banked runs are asserted as refused rather than skipped. **This is now a hard prerequisite for gating any QCD process beyond 2→2** — the short-circuit that lets `e⁺e⁻ → μ⁺μ⁻a` integrate without a prescription stops covering it the moment the matrix element carries a strong coupling | `coupling/scales.rs`, `validate_scales.rs` |
| `uux_to_uux` ~0.15% negative mean | Stable over five seeds and shrinking with budget, so sampling rather than a defect, but larger than the seed spread alone explains. The spacelike collinear region a single-rung t-channel spine under-resolves is where to look — pairs with note 21's deferred multi-rung spine | `validate_sigma.rs` |
| `μF ≥ 2 GeV` event veto | `reweight.f:1185` makes `setclscales` *fail* below it — a veto, not a scale. Not implemented; bites nothing today (the QCD gate rows have no PDF, the hadronic reference cards fix μF at 91.188), but a hadronic run with a dynamic μF reaching below 2 GeV will disagree with MG | `coupling/scales.rs` (§4 above) |
| Per-event scale cost ~100 ns/point | 6–20% of the matrix element depending on process — reported, not hidden. `ScaleChoice::clustered` heap-allocates its beam–leg candidate `Vec` once per event; that is the obvious first cut | `coupling/scales.rs`, `validate_sigma.rs` `probe_scale_cost` |
| Per-lane scales | With pool mutation, `eval_m2_lanes` can only batch points sharing one `αs`. Nothing needs it today (the integrands are scalar), but a SIMD-batched dynamic-scale integrator would need the scaling fused into the constant loads instead | `helas/eval/rescale.rs` |
| `ee_to_wpwm` topology mask | D4's derivation and D2's hand declaration disagree on which beam pairs with which W. Unpinned either way — with colourless beams the tie-break never reaches the scale — so both pass. Matters only if that mask is ever made load-bearing | `coupling/topology.rs` vs `validate_scales.rs` |
| `run_card_dy.dat` vs the `dy13` cards | The test fixture sets `fixed_ren_scale = False` where both banked reference cards set it True. Asserted as a known discrepancy rather than silently aligned, since a banked σ depends on those cards | `validate_scales.rs` |
