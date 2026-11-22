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

## What changed

- **Grammar** (`diagrams/check.rs`). `NAME^2` and the `aEW`/`aS` aliases reach
  `SupportedProcess::squared_orders` as written. `Unsupported::SquaredOrder` and
  `Unsupported::WeightedOrder` are gone. Two refusals are new:
  `ChainSquaredOrder` (a squared order anywhere on a decay-chain line, MadGraph's
  own `do_add` error) and `NegativeSquaredOrder` (a negative value under `==` or
  `>`; see the design note below). The module table and the CLI and guide docs
  are updated.
- **Enumeration** (`diagrams/squared.rs`, `diagrams/mod.rs`).
  `SquaredRequest::resolve` resolves aliases against the model, checks names,
  lets the leftmost constraint win and counts negatives. The implied amplitude
  bounds are added when a line has no `<=`/`=`/`==` order, and the automatic
  WEIGHTED search is off. `SquaredRequest::filter` is
  `apply_squared_order_constraints` on each subprocess, after the s-channel
  filters. `WEIGHTED==n` / `WEIGHTED>n` are MadGraph's `constrained_orders`
  filter on the weighted order: their implied squared order is vacuous, so they
  fall out with no split. Lines sharing a process number must share their
  squared orders (`MixedSquaredOrders`).
- **The split** (`helas/eval/{lower,compile,run}.rs`). `lower_split_flows`
  builds one JAMP per `(order class, flow)` pair that has a contribution, in
  class-major order, MadGraph's `JAMP(flow, M)`. `AmplitudeEvaluator` carries
  `jamp_slots`, and `jamp_cf` holds the colour factor where the selection keeps
  the two classes' term and zero where it drops it. Every existing contraction
  path (`eval_m2`, the lane path, per-helicity |M|², the reweight Gram) reads
  `jamp_cf` unchanged. A selection that keeps every pair compiles exactly as
  before, unsplit. Under a split, `JAMP2(i)` is MadGraph's
  `Σ_{M,N chosen} |Re J(i,M) J(i,N)*|`. The class vectors and the selection are
  public (`order_classes`, `jamp_slots`, `n_jamps`, `jamp_cf`), so PR 10 can
  build on this graded evaluation.
- **Signed |M|²**. Helicity filtering and helicity selection read `|T_h|`, as
  MadGraph does (`DABS(TS(I))`). The budget's stopping test reads `Δσ/|σ|`.
  `generate` refuses a selection that keeps an interference term without the
  squares that bound it (`GenerateError::SignedMatrixElement`): unweighting
  writes no negative-weight events. `integrate` reports the signed σ.
- **Artifacts keep their bytes.** `DiagramSet::squared_orders` is
  `#[serde(skip)]`, and so are the two new `ProcessFacts` fields. Both artifact
  readers derive them again from the recorded card
  (`squared::annotate_sets`, `config::set_interference_facts`). A recorded set
  that the line's filter would change is refused. Cards without squared orders
  therefore serialize exactly as on main: the sets and the pinned enumeration
  digests are unchanged, as is every grid sha256 below.
  `ENUMERATION_VERSION` and the format versions are not bumped.
- **Run-card defaults** (`runcard/process_defaults.rs`, `config.rs`). The
  interference defaults from `banner.py:4969-4988` apply, and so does the
  single-colour block's return to `sde_strategy = 1`. `bounds_qcd_at_zero` reads
  the implied `QCD` bound of a squared-only line.

**Design note, `NegativeSquaredOrder`.** A negative value is resolved against
each subprocess's surviving diagrams. Under `<=` the lowest-order diagram always
pairs with itself, so re-deriving the selection from the recorded diagrams gives
the same target. Under `==` or `>` that diagram can be dropped, and the target
cannot then be recovered from the artifact. Supporting those two operators needs
the resolved target serialized, which means a format bump.

## Oracles, finest first

### 1. Per class, per helicity, per flow against MadGraph 3.5.7 standalone

`validation/madgraph/gen_squared_order_jamps.py` generates each row with
`output standalone` and patches `matrix.f` to copy out `JAMP(NCOLOR,NAMPSO)`.
It banks, per point and helicity, every `JAMP(flow, M)` and every `RES(I)`,
together with `AMPSPLITORDERS`, `SQSPLITORDERS`, `CHOSEN_SO_CONFIGS` and the
param card, in `validation/madgraph/squared_orders/*.json`. The test is
`vibegraph-lib/tests/squared_orders.rs`. It is hermetic and runs in
`cargo test`.

```
$ PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH python validation/madgraph/gen_squared_order_jamps.py all --work …/sa
wrote …/ee_to_ttx_np1.json: 4 points x 16 helicities, classes [[0], [1]], chosen [False, True, False]
wrote …/ee_to_ttx_np2.json: 4 points x 16 helicities, classes [[0], [1], [2]], chosen [False, False, True, False, False]
wrote …/uux_to_uux_qcd2.json: 4 points x 16 helicities, classes [[0], [2]], chosen [False, True, False]

$ cargo test -p vibegraph-lib --test squared_orders -- --nocapture
ee_to_ttx_np1: G = -0.000000000000000 -1.000000000000000i, worst JAMP 1.519e-15, worst RES 3.195e-15, worst |M|² 4.265e-15
ee_to_ttx_np2: G = 0.000000000000000 -1.000000000000000i, worst JAMP 1.556e-15, worst RES 3.152e-15, worst |M|² 1.158e-15
uux_to_uux_qcd2: G = -0.000000000000000 -1.000000000000004i, worst JAMP 1.716e-15, worst RES 1.009e-14, worst |M|² 1.635e-15
test a_rotated_class_assignment_is_seen ... ok
test squared_order_classes_match_madgraph_per_helicity_and_flow ... ok
```

Diagram counts against `NGRAPHS` are 36, 148 and 6. The class vectors and
split-order names match MadGraph's both ways. The two ee→tt̄ rows use the
SMEFTsim `restrict_massless` card. `uux_to_uux_qcd2` is the QCD case: QCD and
electroweak diagrams, two colour flows, and `QCD^2==2`, so the selection is the
interference alone.

The first `uux` run disagreed at 3.3e-4. MadGraph's standalone output keeps
t-channel widths, while this crate zeroes them as MadEvent does. The row now
zeroes every width on both sides, which is the existing convention in
`gen_standalone_jamps.py`.

**What it cannot detect:** the process-wide phase `G`; two diagrams of one
class that compensate at every banked point; helicities and regions the four
points per row do not visit; and the split `JAMP2` used for colour selection,
which standalone output does not expose. That last one is tested only against
its own formula.

**What it detects that a full-|M|² row cannot:** a diagram in the wrong class
when the per-flow sum is preserved, and a class pair wrongly kept or dropped.
`a_rotated_class_assignment_is_seen` pairs MadGraph's classes with ours
rotated by one, leaving every flow's sum unchanged, and requires the comparison
to fail (worst JAMP > 1e-3) on every table.

### 2. Generation-time filter

`tests/diagrams.rs` covers `squared_order_constraints_select_diagrams_as_madgraph_does`,
`weighted_exact_and_lower_bounds_filter_the_weighted_order` and
`a_squared_order_no_term_meets_leaves_no_diagrams`. On u u~ > u u~:
`QCD^2==2` keeps 6 diagrams, `QCD^2==4` keeps 2, `aS=1` gives `QCD^2<=2` with 6,
the leftmost-wins case `aS<=2 QCD^2==4` gives `QCD^2<=4`, and `QCD^2<=-1`
resolves to `QCD^2<=0` with 4. `QED^2==4` alone keeps 4 through the implied
bound. `WEIGHTED==4` and `WEIGHTED>2` keep 4; `WEIGHTED==2` and `WEIGHTED<=2`
keep 2.

Unit tests in `squared.rs` pin the pair selection (`==`, `<=`, strict `>`, all
constraints jointly) and `is_signed`. `check.rs` tests pin the grammar
refusals, `generate.rs` pins the signed-generation refusal, and
`diagrams/artifact.rs` pins that the selection survives a write and read, which
would fail without the re-derivation.

These counts are checked against MadGraph only through the 36/148/6 `NGRAPHS`
above; the u u~ counts are hand-derived from MadGraph's rule.

### 3. σ, five seeds each (release build, 160000 × 8, `ee_to_ttx_smeft` run and param cards)

| card | seeds 20261101–05 (pb) | mean | spread | quoted/√5 | χ²/dof |
|---|---|---|---|---|---|
| `NP^2==1` | −2.00432 −2.00398 −2.00429 −2.00418 −2.00457 (each ±0.00058) | −2.004268 | 2.2e-4 | 2.6e-4 | 0.14 |
| `NP^2==2` | 3.6823 3.6821 3.6826 3.6822 3.6830 (±0.0011) | 3.682440 | 3.7e-4 | 4.9e-4 | 0.11 |
| `NP<=1 NP^2==2` | 3.6769 3.6767 3.6772 3.6768 3.6776 (±0.0011) | 3.677040 | 3.7e-4 | 4.9e-4 | 0.11 |
| `NP^2==0` | 0.55029 0.55007 0.55019 0.55024 0.55029 (±0.00016) | 0.550216 | 9.2e-5 | 7.2e-5 | 0.33 |
| `NP<=1` (banked capstone) | 2.22284 2.22281 2.22308 2.22281 2.22331 (±0.00066) | 2.222970 | 2.2e-4 | 3.0e-4 | 0.11 |

- **Internal closure.** SM + interference + |NP¹|² = 0.550216 − 2.004268 +
  3.677040 = 2.222988 pb, against 2.222970 pb for the unsplit `NP<=1`: a
  difference of 8e-6 relative. This is a σ-level check that the classes
  partition |M|². It is blind to a common factor on every class.
- **Against banked MadGraph.** `NP^2==0` at 0.550216 reads +2e-6 relative
  against the banked `ee_to_ttx_smlimit` MadGraph 0.550210 pb (a different
  restrict card with the same SM limit). The capstone reads +3.0e-4 against
  2.2223 ± 0.00053 pb.
- **No MadGraph σ for the two new rows yet.** The 3.5.7 MadEvent in this
  container fails in every survey: `gensym` stops on
  `Fortran runtime error: Missing comma between descriptors (I1$)` at
  `symmetry.f:234` (`ValueError: could not convert string to float: '#7'` in
  `gen_ximprove.py:162`). The references have to be banked at 3.7.1 in any
  case. The request is below.
- **The quoted errors look about 3× conservative.** χ²/dof sits near 0.1 on all
  five rows, including the already-banked capstone, so this is a property of
  the existing estimator and not of this change.

### 4. Nothing else moves: main b2434a0 against the branch, bit for bit

Both sides ran release builds of `vibegraph integrate` on each row's own banked
cards with `--seed 20261122 --fixed-budget`: `ee_to_mumu` (20000 × 5),
`dy13_default` (proton beams, 20000 × 5) and `ee_to_ttx_smeft` (`NP<=1`,
40000 × 5).

```
main   ee_to_mumu      σ = 2.0238 ± 0.0022 nb  9dd7444274328815
main   dy13_default    σ = 923.4 ± 4.9 pb      dc65dd14276a9b82
main   ee_to_ttx_smeft σ = 2.2220 ± 0.0014 pb  9ee5b2b0962ac671
branch ee_to_mumu      σ = 2.0238 ± 0.0022 nb  9dd7444274328815
branch dy13_default    σ = 923.4 ± 4.9 pb      dc65dd14276a9b82
branch ee_to_ttx_smeft σ = 2.2220 ± 0.0014 pb  9ee5b2b0962ac671
```

The last column is the first 16 hex digits of `sha256(grid.bin.zst)`, so the
grid files are identical. The branch binary was built from `c04a946` plus the
call-site fix. The later commits change tests, the diagram-artifact facts and a
type alias, none of which reaches an integrate artifact. The main build and its
worktree were deleted afterwards.

### 5. Gates

```
cargo test --workspace                              67 targets ok: 2251 passed, 0 failed, 26 ignored
cargo clippy --workspace --all-targets -D warnings  EXIT 0
cargo fmt --all --check                             clean
cargo test --features extended-validation --test color_cf_oracle --test color_flow_tags_oracle
                                                    112 passed; 186 passed
pixi run kb-lint                                    0 errors (5 pre-existing warnings)
```

`amplitude_oracle` and `standalone_jamps`, the hermetic amplitude gates, run
inside the workspace suite and pass. `validate_sigma` and the rest of the banked
layer are left to CI's `banked` job, as the brief directs.

## Campaign cards

Not done here. The 24 cards are not listed anywhere in the repository. GitHub
code search through `gh` returns nothing for `^2` patterns, and only
raw.githubusercontent.com is reachable, which needs exact paths. The request
below asks vg-validator to rerun them.

## Long-run requests (vg-validator)

1. **σ references at MadGraph 3.7.1** for two new rows. Branch
   `sprintl-squared-orders`; head SHA in the final message. Scripts:
   `validation/madgraph/scripts/ee_to_ttx_np1.mg5` and `ee_to_ttx_np2.mg5`
   (the capstone's script with the process line changed). Five fresh-directory
   MadEvent seeds each under the seed policy. Expected answers: −2.00427 pb
   (NP^2==1) and 3.68244 pb (NP^2==2), each ±0.03%. MadEvent's own errors will
   be larger here, because it samples the sign of a helicity rather than summing
   signed helicities. Once banked, register `ee_to_ttx_np1` / `ee_to_ttx_np2`
   rows with gated `integrals` (rel_tol on the capstone's model, 0.002) and
   `amplitudes` covered by the `squared_orders` tables.
2. **Campaign cards.** Rerun the 24 v022-1009 cards refused on squared orders,
   including the three `M3^2==1` / `T6^2==1` ones, with the branch binary:
   `enumerate`, then `integrate --check`. Expected result: none refused on
   squared orders. Any that are refused again should name `ChainSquaredOrder`,
   `NegativeSquaredOrder` or an unrelated refusal. Interference-only cards
   integrate, and `generate` on them refuses with `SignedMatrixElement`.

## Corrections to the brief

- The brief asks for σ against MadGraph with 3.5.7 here and points to 3.7.1 for
  banking. 3.5.7's MadEvent cannot run in this container (the `gensym` failure
  above), so every MadGraph σ for the new rows goes to vg-validator.
- `WEIGHTED==`/`>` and `aEW`/`aS` needed no split. Their implied squared order
  is vacuous given the diagram filter (`WEIGHTED`), or they are plain squared
  orders (`aEW`/`aS`). All four are supported.
- `aEW`/`aS` are now refused by name only on a model that defines an order of
  that name (`ResolveError::AliasIsAModelOrder`). MadGraph would read such a
  constraint as an amplitude order instead.

## Found

1. **Unweighting cannot write negative-weight events.** `generate` refuses any
   signed squared-order selection, which includes every `NP^2==1` card.
   MadGraph writes such samples with signed weights. *Closes when:* the
   unweighting accepts `|w|/w_max`, writes the sign into the event, and
   `validate_unweighting` gains a signed row.
2. **Implied squared orders are missing from the run-card interference
   defaults.** MadGraph prints the squared order that an amplitude `==`/`>`
   implies in `nice_string`, so `p p > t t~ QED==0` (no run card) also gets
   `use_syst = F`, `dynamical_scale_choice = 3` and `sde_strategy = 2`
   (`banner.py:4972-4988`, `base_objects.py:4090-4094`). This crate's default
   card does not, which is a pre-existing silent difference on default cards.
   *Closes when:* `prints_squared_orders` also counts non-WEIGHTED `==`/`>`
   amplitude orders on non-chain lines, with a `dump-runcard-defaults` row.
3. **Negative squared values under `==`/`>` are refused**
   (`NegativeSquaredOrder`). *Closes when:* the resolved target is serialized
   with the set, behind a format bump.
4. **KB concepts are stale.** `process/supported-card-check.md`,
   `process/proc-card-grammar.md`, `model/coupling-orders.md`,
   `model/smeftsim-topu3l.md`, `process/decay-chains.md` and
   `validation/non-sm-rows.md` still describe squared orders as refused. Their
   links to the deleted item now point here. This is close-out work.
5. **The σ quoted errors look about 3× conservative** on these five
   fixed-beam SMEFT rows (χ²/dof ≈ 0.1 over five seeds, the banked capstone
   included). This is worth a 20-seed spread measurement before any tolerance
   is tightened on them.
6. **MadEvent 3.5.7 fails in the container.** Its `gensym` stops on the `$`
   edit descriptor at runtime (`symmetry.f:234`). This affects any session that
   wants a local 3.5.7 MadEvent σ.
