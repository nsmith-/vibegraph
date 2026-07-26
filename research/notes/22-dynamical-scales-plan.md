# `dynamical-scales` — running couplings and the per-event scale (sprint plan)

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
`√ŝ`; decaying-particle mass), each then multiplied by `scalefact`. The **default
`-1`** is not in that file: it is the kT-clustering scale — cluster the event down
to a 2→2 topology and use the central vertex's transverse mass
(`SubProcesses/reweight.f:551 setclscales`, with `cluster.f`). Reproducing the
general clustering is a subproject of its own and is **out of scope**; what is in
scope is the set of cases the banked runs pin, where clustering is degenerate.

All 19 banked runs use `dynamical_scale_choice = -1`, `fixed_ren_scale = False`,
`fixed_fac_scale = False`, `scalefact = 1.0`.

### 1.3 What `-1` collapses to, measured on the banked events

| run | beams | `SCALUP` over 10k events |
|---|---|---|
| `gg_to_gg`, `gg_to_ttx`, `uux_to_uux` | `lpp = 0`, `ebeam = 250` | **exactly `250.0`, every event** (`αs = 0.1113305`) |
| `pp_to_bb_qcd2` | `lpp = 1` | varies per event; one event checked exactly: `5.1491350` = `√(m_T(b)·m_T(b̄))`, `m_T = √(m²+p_T²)` — the "s-channel QCD → geometric average of the transverse masses" branch (`reweight.f:1012` comment) |
| `pp_to_ll_qcd0` | `lpp = 1` | varies per event; formula to be pinned in D2 (colour-singlet central line) |

So with no PDF the scale is just the beam energy (`= √ŝ/2`), constant — which
means the three fixed-beam QCD σ rows need only a *correct constant* `αs(250)`,
not the full dynamic machinery. The dynamic path is what the hadronic rows and
the LHE `SCALUP` field need.

### 1.4 The oracle this sprint gets to use

Every banked run has `Events/run_01/unweighted_events.lhe.gz` with 10k events.
Each `<event>` line carries `SCALUP`, `AQEDUP`, **`AQCDUP`** alongside the
momenta, and the `<mgrwt>` block adds `<rscale>` and per-beam
`<pdfrwt>` (`x`, `μF`). That is a **per-event, finest-level oracle** for all three
new quantities — scale function, `αs(μ)`, and the `μF` fed to the PDF — not a
σ-level backstop. Per AGENTS.md's validation regime this is the gate to build
first; σ agreement is the coarse confirmation afterwards.

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
- **DY moves.** `μF` becomes dynamic where it was fixed at `91.188`; the 0.14%
  agreement was partly luck (the Z peak sits at the fixed scale). Expect the
  number to change and re-derive it.
- **Hot-path cost.** Per-event coupling rescale sits inside the VEGAS loop.
  Measure it (`eval_strategies`, `profile-sigma`); the fast path exists so that
  the answer is "negligible", but that must be shown, not assumed.
- **Parallelism.** Pools become per-event mutable state; they must be per-thread
  (§2) or `adapt_parallel` silently races.
- **Scale ≠ only `αs`.** The same choice drives `μF`; treat them as one object
  with two outputs (MG allows them to differ per beam — `q2fact(1)`, `q2fact(2)`).

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
