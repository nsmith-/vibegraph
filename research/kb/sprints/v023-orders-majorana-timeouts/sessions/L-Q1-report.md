---
type: Session Report
title: "L-Q1 report: squared-order constraints by coupling-order split"
description: "MadGraph's squared-order semantics with citations, the split of each amplitude by coupling-order class with only the selected class pairs in |M|², the per-class oracle against MadGraph 3.5.7 standalone, σ seeds, the bit-for-bit check and the campaign cards."
status: draft
tags: [squared-orders, smeft, coupling-orders, process-grammar, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
Branch `sprintl-squared-orders` off main b2434a0. Closes
`squared-order-constraints-refused` (its file is deleted in the final commit).

## MadGraph's semantics (pinned 3.7.1 at `research/refs/mg5amcnlo`)

Every rule below was read in the pinned checkout before the code was written;
the 3.5.7 used for the oracles here has the same text in every cited function
(diffed: `apply_squared_order_constraints`, `set_chosen_SO_index`, the order
block of `extract_process`, `matrix_madevent_group_v4.inc`,
`split_orders_helping_functions.inc`), except that 3.5.7 also adds the implied
squared order of an amplitude `==`/`>` on a decay-chain core
(`avoid_squared_orders`, 3.7.1 only), which is irrelevant here since squared
orders are refused on chains.

**Parsing** (`madgraph/interface/madgraph_interface.py`).
- The order regex matches the rightmost constraint first and strips it
  (`:4883-4886`, `:4967-4968`), so in every dict the *leftmost* constraint on a
  name wins.
- Aliases (`:4891-4908`): on a model with `EW` and no `QED`, `QED`→`EW`,
  `QED^2`→`EW^2`; `aEW` → `EW^2` with the value doubled when the model has `EW`,
  else `QED^2` doubled; `aS` → `QCD^2` doubled. An alias applies only if the
  model does not itself define an order of that name.
- A squared constraint (`NAME^2`): the base must be a model order or `WEIGHTED`
  (`:4928-4932`); types `==`, `<=`, `=`, `>` (`:3036`, `:4933-4935`); `=` is
  read as `<=` (`:4936-4939`).
- An amplitude `==` adds the name to `constrained_orders`, to `orders` as an
  upper bound, and — if no squared constraint on the name was met yet — a
  squared order `(2n, '==')`; an amplitude `>` adds `constrained_orders` and a
  squared order `(2n, '>')` but nothing to `orders` (`:4953-4965`).
- If `orders` is empty and squared orders exist (LO), every squared name gets an
  amplitude bound: its value when non-negative and not `>`, else 99
  (`:4987-4999`). So `NP^2==1` alone generates at `NP<=1`, `NP^2==2` at `NP<=2`.
- `split_orders` = the squared-order names (plus perturbation orders), sorted by
  hierarchy with `WEIGHTED` first (`:5262-5273`).
- At most one negative value across squared orders (`:5329-5332`), and across
  amplitude and squared orders together (`do_add`, `:3341-3346`).
- Decay chains: "Decay processes cannot specify squared orders constraints"
  (`:3301-3304`). A `1 → n` process may, with a warning (`:3348-3354`).
- Any squared order means `orders` is non-empty, so the automatic WEIGHTED
  search does not run (`diagram_generation.py:1971-1975`).

**Generation** (`madgraph/core/diagram_generation.py:856-899`, called at `:808`
for a top-level amplitude).
- `filter_constrained_orders` keeps a diagram with order `== n` / `> n` for
  each amplitude `==`/`>` (`base_objects.py:2897-2911`).
- Then, iterated to a fixed point, `apply_positive_sq_orders` keeps a diagram
  only if some diagram of the list (itself included) makes with it a term the
  positive constraints all accept (`base_objects.py:2882-2895`), the test being
  `pass_squared_order_constraints` on the summed orders: `==` equal, `<=` at
  most, `>` strictly above (`base_objects.py:2659-2676`). Orders absent from a
  diagram are 0; `WEIGHTED` is `Σ hierarchy·power` (`calculate_orders`,
  `base_objects.py:2638-2657`).
- At most one negative constraint is resolved last into
  `target = min + min + 2(−v − 1)` over the survivors' values of the order, and
  the list filtered once more against it alone; the target replaces the
  negative value in `squared_orders` (`base_objects.py:2863-2880`,
  `diagram_generation.py:884-897`).
- So an amplitude `==`/`>`'s implied squared order is vacuous: every kept
  diagram has order `n` (or `> n`), every pair sums to `2n` (or `> 2n`).

**Matrix element** (`madgraph/iolibs/export_v4.py`, templates).
- The JAMPs are split per amplitude split-order class: `JAMP(flow, M)` where `M`
  indexes the distinct vectors of the diagrams' orders in `split_orders`
  (`helas_objects.py:4996-5035`, `get_JAMP_lines_split_order`,
  `export_v4.py:1497`).
- `set_chosen_SO_index` (`export_v4.py:1171-1203`) marks each squared split
  order (a sum of two class vectors) chosen iff it passes *every* user squared
  constraint (with the resolved target); the same three comparisons.
- MadEvent: |M|² of a helicity is `Σ_M Σ_N [chosen(M+N)] Re Σ_ij CF_ij
  JAMP(j,M) JAMP(i,N)*` (`matrix_madevent_group_v4.inc:358-372`); `JAMP2(i)`
  sums `|Re JAMP(i,M) JAMP(i,N)*|` over the chosen pairs (`:380-388`). `AMP2`
  is unchanged (`get_amp2_lines`, `export_v4.py:1390-1444`). The helicity sum
  is `ANS = Σ |T_h|` with the sign of a helicity drawn `∝ |T_h|`
  (`:136-147`, `:196-207`): unbiased for a signed integrand, noisier per point.
- Standalone: `RES(I)` per squared split order, `ANS(0) = Σ_chosen ANS(I)`
  (`matrix_standalone_splitOrders_v4.inc`, `SMATRIX_SPLITORDERS`).
- Run-card defaults: a process whose `nice_string` prints `^2` (any squared
  order but `WEIGHTED`) gets `use_syst = F`, `systematics_program = none`,
  `dynamical_scale_choice = 3`, `sde_strategy = 2` (`banner.py:4969-4988`);
  the single-colour block that follows can set `sde_strategy` back to 1
  (`banner.py:4993-5014`).

Several squared constraints at once select a pair only when every one accepts
it, in generation and at the matrix element alike.

<!-- RESULTS -->
