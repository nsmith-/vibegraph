---
type: Session Report
title: "K-O1 report: negative UFO coupling orders and the gluon-free WEIGHTED trial"
description: "Coupling orders load signed and are spent as MadGraph spends them, vertex by vertex in its reduction order; the automatic WEIGHTED search trials each subprocess with its final-state gluons removed. SMEFT_alpha loads and matches MadGraph per diagram and per helicity."
status: draft
tags: [ufo, coupling-orders, weighted, smeft, smeftsim, madgraph-parity, report]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-17}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-negative-orders`, branch `sprintk-negative-orders`,
based on `main` at `703f57f`. MadGraph is the pinned submodule
(`research/refs/mg5amcnlo`, `VERSION` 3.7.1), run through
`validation/madgraph/mg5_pinned.sh` from the main checkout's `madgraph` pixi
environment. Scratch work sits in `/home/user/wt/sprintk-negative-orders-mg/`.
The brief named MadGraph 3.5.7; the pinned checkout every generator here runs is
3.7.1.

| commit | what |
|---|---|
| `ee1eb01` | signed UFO coupling orders and the gluon-free WEIGHTED trial (checkpoint) |
| `de997b7` | spend coupling-order bounds in MadGraph's reduction order; toy model counts |

## How MadGraph treats a negative order

1. **Loading.** `models/import_ufo.py:1854` copies `coupling.order` into the
   interaction as written; `Interaction.filter` (`base_objects.py:725-734`) asks
   only that each power is an `int`. No clamp.
2. **A diagram's orders.** `Diagram.calculate_orders` (`base_objects.py:2638-2657`)
   sums each order's signed powers over the vertices, and WEIGHTED as Σ
   hierarchy × power; both can fall below a positive-only sum.
3. **Interaction WEIGHTED and the search's model inputs.**
   `Interaction.get_WEIGHTED_order` (`:893-899`) is the same signed sum over
   `max(n − 2, 1)`; `get_max_WEIGHTED` (`:1481`) takes the largest and
   `find_optimal_process_orders` truncates it with `int()`
   (`diagram_generation.py:1999`). `get_particles_hierarchy` (`:1432-1479`)
   reads only the order *names*, so SMEFT_alpha's `{NP: 1, QED: -1}` triple-Higgs
   vertex joins the QED class.
4. **The bounds: a running budget, not a total.** `reduce_orders`
   (`diagram_generation.py:1048-1083`) subtracts each new vertex's signed orders,
   and its WEIGHTED, from what the process allows as `reduce_leglist`
   (`:958-1046`) builds the diagram, and drops it the moment a remaining budget is
   below zero; a negative bound is ignored. Nothing is clamped, but the check is
   per prefix: a diagram whose total is within the bound is dropped when its
   negative vertex is added after a prefix that is not. With non-negative orders
   prefix and total agree, which is why no banked row can see it.
5. **The reduction order is a property of the tree.** A combination needs at
   least one leg made in the step before (`LegList.can_combine_to_1`,
   `base_objects.py:2250-2256`; uncombined legs lose the mark, `merge_comb_legs`,
   `diagram_generation.py:1224-1231`), so each vertex is made at its height above
   the external legs. The closing vertex needs two fresh legs (`can_combine_to_0`,
   `base_objects.py:2258-2279`) or is the identity joining the last two
   (`:579-589`, merged into `ref_dict_to0` at `:1176-1179`): the root is the tree's
   centre. A decay closes on its initial leg (`:2267-2274`). Within a step the
   vertices are checked in leg order, which stays sorted by the smallest external
   leg number below each (`combine_legs`, `diagram_generation.py:1114-1153`).

## What changed

- `ufo/couplings.rs`: `Coupling.orders` is `BTreeMap<String, i64>`; a negative
  power loads. bincode writes a non-negative `i64` with the same bytes as a
  `usize`, so model digests and every artifact written before are unchanged.
- `ufo/topo.rs`: `NegativeOrders`. feyngraph counts powers as `usize`; each
  negative power goes to it as 0 and is recorded per feyngraph vertex, and a
  diagram's signed power is feyngraph's count plus the recorded part.
- `ufo/weighted.rs`: interaction WEIGHTED, the cap and `maximum_weighted` are
  signed, truncated towards zero as Python's `int` truncates.
- `diagrams/selector.rs`: an order some interaction carries negatively is bounded
  on the signed sum through a custom selector function; every other order keeps
  feyngraph's own (pruning) selection, so Standard-Model paths are untouched.
- `diagrams/reduction.rs` (new): MadGraph's running budget in its reduction order
  (height, then smallest leg number, the centre vertex last; a decay's initial-leg
  vertex last). `generate_sets_inner` applies it to every `<=`/`=` bound and the
  WEIGHTED bound when the model carries any negative power.
- `diagrams/mod.rs`: the WEIGHTED filter is the signed sum; the automatic search
  runs MadGraph's gluon-free trials (below); a new refusal
  `GluonTrialWithoutQcd` names the case MadGraph answers with `KeyError`.
- Signed orders through `Diagram::coupling_orders`, the decay-chain overall-order
  check, `qcd_order`, and the CLI's `enumerate`/`inspect` JSON.
- `validation/ufo/vibegraph_toy_negorder_UFO` (authored, README entry added): a
  scalar cubic coupling at `QED=1` and again at `{NP: 1, QED: -1}`.
- `tests/negative_orders.rs`: the signed load, and 22 diagram counts against
  MadGraph. `tests/weighted_search.rs`: the gluon test runs.

## The gluon-free trial

`find_optimal_process_orders` (`diagram_generation.py:2015-2108`), reproduced as
`GluonFreeTrials` when PDG 21 is in the lowest particle class and a final-state
leg can be a gluon: per initial-state assignment, each final-state assignment
(one per sorted content) is trialled with its gluons removed, at the bound
lowered by the QCD hierarchy per gluon (a negative trial bound bounds nothing);
legs from particle ids alone; a crossing that failed is skipped unless
s-channels are forbidden; a two-leg trial passes on a particle-antiparticle pair
unless an s-channel is required. When the gluon condition does not hold the
search trials the whole process, as before: MadGraph's trials then differ only in
ignoring polarizations and skipping failed crossings, which can move only the
bound a refusal quotes.

## Oracles

Scratch work: `/home/user/wt/sprintk-negative-orders-mg/`. `vg/count.sh` runs
`vibegraph enumerate` per process; `ko1-mg-counts.sh` runs one pinned-MadGraph
`generate` per process (`mgcounts/ko1-counts.txt`); `standalone/ko1_rows.py`
banks standalone tables through `gen_standalone_jamps.py` with scratch rows, a
scratch Rust test (kept beside them as `ko1_scratch_amps.rs`, never committed)
dumps this side's per-diagram values at the same points and helicities, and
`standalone/ko1_match.py` pairs them graph by graph.

**Diagram counts, 32 processes, all equal to MadGraph.** The 22 toy processes
are the table in `tests/negative_orders.rs` (three of them MadGraph refusals).
SMEFT_alpha, unrestricted:

| process | MadGraph | vibegraph |
|---|---|---|
| `h h > h h NP=1` | 31 | 31 |
| `h h > h h` | 16 | 16 |
| `z z > h h NP=1` | 78 | 78 |
| `z z > h h` | 6 | 6 |
| `e+ e- > z h h NP=1` | 838 | 838 |
| `e+ e- > z h h` | 126 | 126 |
| `e+ e- > z h h QED=2 NP=1` | 168 | 168 |
| `e+ e- > z h h QED=1 NP=1` | 8 | 8 |
| `g g > h h NP=1` | 18 | 18 |
| `e+ e- > mu+ mu-` | 10 | 10 |

and under the σ card's restriction (below): `e+ e- > z h h NP=1` 334/334,
`e+ e- > z h h` 44/44, `e+ e- > z h h QED=1 NP=1` 6/6.

Non-vacuity, measured: with the signed *total* as the filter (commit `ee1eb01`)
eight toy rows differed (`fw~ fw > xw xw QED=0` 1 against MadGraph's refusal,
`WEIGHTED=1` likewise, `fw~ fw > xw xw xw QED=1` 15/12, `QED=0` 3/refused,
`xw xw > xw xw QED=0` 9/6, `xw xw > xw xw xw QED=1` 115/94,
`vw vw > xw xw xw QED=1` 15/12, `fw~ fw > fw~ fw xw xw QED=2` 52/38), and so
did the automatic search on SMEFT_alpha (`h h > h h` 22/16, `z z > h h` 8/6).
All agree after `de997b7`.

**Per-diagram × per-helicity complex amplitudes against MadGraph standalone.**
SMEFT_alpha's default card sets every Wilson coefficient to zero, which leaves
every NP diagram at exactly zero; both sides set `NEWCOUP` 1-15 to distinct
values between ±0.52 and ±2.9 (Λ = 1 TeV).

| process | graphs paired 1:1 | global G | worst graph / largest | worst total / largest |
|---|---|---|---|---|
| toy `fw~ fw > xw xw` (3 points × 4 helicities) | 4/4 | −i, \|G\|−1 = 0 | 3.0e-16 | 5.2e-16 |
| SMEFT_alpha `z z > h h NP<=1` (3 × 9, zero widths) | 78/78 | +i, \|G\|−1 = 1.1e-11 | 2.9e-11 | 1.4e-10 |
| SMEFT_alpha `e+ e- > z h h NP<=1` (2 × 12) | 838/838 | +i, \|G\|−1 = −2.1e-11 | 1.2e-10 | 2.3e-10 |
| the same at 200 RAMBO points | 838/838 | +i, \|G\|−1 = −2.4e-11 | 1.8e-10 | 1.9e-10 |

Every graph off by more than 1e-12 on its own scale is off by a modulus factor of
exactly 1 − 1.22e-8 per rounded coupling (2.44e-8 for two), phase exact: the
seven-digit coupling literals MadGraph's Fortran writes, the class of `GC_303`.
MadGraph graphs of 1e-17 to 1e-19 that are exactly zero here are cancellation
residues. The vev-insertion graph of `z z > h h` (|amp| up to 0.377) agrees to
1.5e-15 on its own scale.

**σ (substituting for GluGluH_HToUpsilon).** The Upsilon gridpacks' model is
`onia_sm`; it is not in the CMS model store and its card was out of reach (the
`cms-sw/genproductions` repository is not indexed by code search and could not
be attached here). In its place: SMEFT_alpha restricted by a card with
cHBox = −0.52, cHDD = 0.69, cH = −0.86 and every other coefficient zero
(`SMEFT_alpha_UFO-ko1`), `e+ e- > z h h NP=1` at √s = 1 TeV, MadGraph's default
run card for it, the restriction's own parameters on both sides.

- MadGraph (MadEvent, `nb_core 1`): **117.07 ± 0.149 ab** (`results.dat`
  `0.11707E-03 0.14859E-06` pb).
- vibegraph, five seeds (default convergence, target 0.1%): 116.933 ± 0.080,
  116.979 ± 0.080, 116.955 ± 0.078, 116.800 ± 0.078, 116.986 ± 0.080 ab.
  Unweighted mean **116.931**, seed spread 0.076, χ²/dof 0.94; mean − MadGraph =
  −0.139 ab (−0.12%), pull −0.91 against the error of the mean and MadGraph's
  error combined. About 32 s per seed.
- What the vev insertion carries here: with cH moved to 0.17874 (its only
  entries in this process are the `{NP: 1, QED: -1}` coupling `GC_5284`, set to
  zero by that value; `dlam` otherwise enters only the four-Higgs `GC_5181`),
  one seed gives 97.402 ± 0.066 ab: the vev-insertion diagrams raise σ by 20%
  of the no-insertion value, about 130 times the comparison's resolution. At
  `NP=0` (one seed) σ = 119.76 ± 0.08 ab.

**Banked rows.** `cargo test --workspace`: 63 suites, 2117 passed, 0 failed,
27 ignored (the first attempt died on a full disk and was rerun).
`cargo clippy --workspace --all-targets -- -D warnings` and
`cargo fmt --all --check` are clean. `validate.sh --skip-deps` was not run locally: CI's `banked` job runs
`pixi run --skip-deps validate` on the PR head; the result is the CI banked run
on this branch's final head.

**What each oracle cannot see.** The counts check numbers, not the diagram sets:
two compensating errors (one diagram dropped, another kept) would pass; the
per-diagram oracle closes that for the three processes it covers. The per-diagram
pairing is free in its global phase and pairs by best match, so it cannot see a
swap between two diagrams with identical values at every banked point. The toy
is a single-flow, colourless model: colour and the gluon-free trial are
exercised only by the SMEFTsim `e+ e- > z g g` test and the banked SM rows. The
σ comparison has 0.15% resolution and sees the vev diagrams only through their
interference on one card. The reduction-order rule is pinned by counts on trees
of up to six external legs; a rooting difference that changes no count there
is invisible.

## Found

1. **The SMEFT_alpha oracles are scratch, not banked.** Per-diagram tables
   and σ above live in the scratch directory. Closing: vendor SMEFT_alpha (or
   a SMEFTsim v2 A-model) under `validation/ufo` with its licence, and add
   `zz_to_hh` / `ee_to_zhh` rows to `gen_standalone_jamps.py` with the
   Wilson-coefficient card edits, plus a σ row.
2. **The toy's amplitudes are not a banked standalone row.** The 4-graph
   table matched at 3e-16, but a row needs a `restrict_*` card for the toy.
   Closing: add `restrict_full.dat` (its own param card) and the row.
3. **MadGraph's gluon-free trial raises `KeyError` whenever the gluon is in
   the lowest class and the model has no QCD hierarchy**, even with no gluon
   in the final state (`diagram_generation.py:2058` evaluates
   `nglue * order_hierarchy['QCD']` unconditionally). vibegraph refuses only
   when a gluon is actually removed. Evidence: reading only. Closing: decide
   whether to mirror it; no model in reach is affected.
4. **The automatic WEIGHTED search ignores polarizations in MadGraph's trials
   but not in ours when no gluon is removed** (`g0_g_ttx`: bound 2 against our
   4, refusal text only). Closing: trial every process MadGraph's way.
5. **Timing of the gluon-free trials, measured: no regression.** Standard
   Model, `vibegraph -q enumerate -j 2 --svg none`, the same `dev` profile
   built on `main` at `703f57f` and on this branch, two repetitions each
   (both cards run the gluon-free trial path):

   | card | main (s) | branch (s) | diagrams / subprocesses (both) |
   |---|---|---|---|
   | `p p > j j j j` | 6.49, 5.92 | 6.00, 7.04 | 8560 / 275 |
   | `p p > a j j j j` | 45.42, 47.77 | 48.11, 46.14 | 44744 / 274 |

   Nothing to file.
6. **Dangling links after the items' deletion** (`kb-lint` errors, outside
   this session's edit scope):
   `backlog/feature/ufo-parameter-value-expressions-refused.md:24` and
   `validation/cms-gridpack-refusals-202a540.md:39` link
   `negative-ufo-coupling-orders-refused.md`.
