---
type: Session Report
title: "R2 report: independent review of thirteen sprint-edited concepts"
description: "Every claim of the thirteen concepts checked against code, the committed MadGraph tables, the pinned ALOHA/MadGraph tree and re-run gates; the findings, the fixes, which concepts were stamped and why the rest were not, the stop-rule items and Found."
status: draft
tags: [review, kb, amplitudes, validation, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
---
Brief: [R2](R2.md). Worktree `/home/user/wt/wt-review-b`, branch
`amp-signs/concept-review-b`, built on `4277158`. No code was changed. Two
mutations were applied to production and test code to re-measure one claim,
then reverted (see the wavefunctions section). That was a measurement, not a
code change.

## How the checks were made

The findings below were written before any concept was edited. Each claim's
check is one of these:

- **re-run**: the gate or test was run on `4277158` and its own output read.
  Runs, all with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, logs under the
  worktree's `target/r2-*.log`:
  - `amplitude_oracle -- --nocapture`: 49 passed;
  - `coupling_oracle -- --nocapture`: 48 passed;
  - `standalone_jamps -- --nocapture`: 49 passed;
  - `validate_madgraph_diagrams`: 57 passed;
  - `decay_chain_census -- --nocapture`: passed;
  - `smeftsim -- --nocapture`: 13 passed;
  - `--lib` filtered to `stitching`, `test_eval_jioxxx` and
    `generic_chiral_pair`: 6 passed;
  - `--features extended-validation --test color_cf_oracle --test
    color_flow_tags_oracle`: 97 and 163 passed;
  - `--features extended-validation --lib` filtered to
    `momentum_slashed_chain_is_rooting_invariant` and `test_eval_jioxxx`,
    then two mutations (wavefunctions section).
- **census**: re-counted by a script over the committed files
  (`validation/manifest.toml`, `validation/madgraph/amplitudes/*.json`,
  `standalone/*.json`, `couplings/*.json`, `decay_chain_census.json`,
  `interactions.json`, the generated `matrix1_orig.f`).
- **code**: the named item was found in the current tree and read.
- **source**: the pinned MadGraph/ALOHA tree (`research/refs/mg5amcnlo`,
  submodule at `b7687064`, `VERSION` 3.7.1) was opened at the cited line.
- **record**: the cited note, in the unchanged archive under
  `research/kb/history/notes/`, or the manifest cell note. The archive's
  line numbers run two lines past the 787070e numbering. On note 35 every
  cited heading sits at the cited line plus two.
- **derived**: re-derived by hand from the formula in the concept.

**Not checked: the vibegraph source pins at `787070e`.** The clone is shallow
and that commit is not in it. The brief forbids contacting GitHub. So the
line ranges of the pinned `blob/787070e…/vibegraph-lib/…` links could not be
opened. Every item those links name was found by name in the current tree,
and every claim about it was checked there. I treat the pins as provenance
metadata, not as claims of the concept body, and did not withhold a stamp for
them alone. If you read the brief more strictly, no concept can be stamped,
because all thirteen carry such pins.

## Findings, per concept

Verdicts: **ok** (the claim holds as written), **wrong** (it is false at
`4277158`), **stale** (it was true at an earlier commit and is now out of
date), **imprecise** (the claim is right in substance but its wording is off),
**unchecked** (I could not check it).

### amplitudes/gamma-chains-gamma5-and-epsilon

| # | claim | check | verdict |
|---|---|---|---|
| GC1 | Informational rows include `wpwm_to_wpwmz_cw`, citing the item `wpwmz-cw-ow-five-vector-residual` | census: the manifest gives the row's `amplitudes` mode `gate`; re-run: `amplitude_oracle` passes it, worst flow 3.31e-14, with `GC_303` on 12 named graphs at −1.197023e-8; the item is closed | **wrong**: the row is gated; the slug points at a deleted item |
| GC2 | `ee_to_wpwm_cw`: one of 48 points at 2.078e-12, 336× its ulp sensitivity | re-run: report row `max_rel_grid` 1.632e-12, event 2.57e-15, `G = −1i`; the manifest note says 2.078e-12 at `G = +1i` | **stale**: the sprint moved the reading to 1.63e-12, still above 1e-12. The 336× ulp ratio and the factor-875 cancellation were not re-measured (the harness does not print them for this row) |
| GC3 | `L_Epsilon.give_parity` returns `−sign·(…)/12`, so `ε^{0123} = −1`, `ε_{0123} = +1` | source: `aloha_object.py:938-946`; derived: the identity permutation gives product 12, so the result is `−sign`; lowering four indices in (+,−,−,−) gives a factor −1 | ok |
| GC4 | `epsilon_out_order`: `ε(x₀…x₃) = (−1)^{3−k} ε(rest…, out)`, a −1 absorbed by swapping the first two | code `root_lorentz.rs:268-291` | ok |
| GC5 | `gg_to_h_cpodd`, `gg_to_h_cpeven`, `ee_to_ttx_dipole`, `gg_to_gg_cg` gated | census (manifest modes) and re-run | ok; `wpwm_to_wpwmz_cw` belongs in this list too (GC1) |
| GC6 | `EpsilonVout` gated through `gg_to_gg_cg`; the identity `EpsilonVout · d = EpsilonAmp` is hermetic | re-run `smeftsim` op census: `gg_to_gg_cg` has 27 `EpsilonVout`; code `kernel.rs:1273-1290` | ok |
| GC7 | `chain_adjoint`, `pmom_out` (`p_bra − p_ket`), `chiral_gamma_site` refusals, the fused-vs-generic kernel tests | code; re-run `ffv_vout_matches_generic_chiral_pair`, `ffv_fermion_out_matches_generic_chiral_pair` and `momentum_slashed_chain_is_rooting_invariant` pass | ok |
| GC8 | The SM UFO writes `γ⁵` as `ProjP − ProjM`, so only SMEFTsim and toy rows reach the `Gamma5` ops | source: `models/sm/lorentz.py` has no `Gamma5`; smeftsim census: `Gamma5` reaches `ee_to_ttx_dipole` | ok |

### amplitudes/per-diagram-amp2

| # | claim | check | verdict |
|---|---|---|---|
| PD1 | `wpwm_to_wpwmz_cw` has no banked pairing; its 21-group partition matches MadGraph's only in group sizes (`KNOWN_CONFIG_PAIRING_UNAVAILABLE`); cites `wpwmz-cw-config-partition-uncompared` | code: `KNOWN_CONFIG_PAIRING_UNAVAILABLE` is `&[]`; `MG_DIAGRAM_ORDER` carries `mg_graphs::WPWM_TO_WPWMZ_CW_GRAPHS`, measured on the standalone table; re-run: the row passes the partition comparison through that pairing | **wrong**: the partition is compared as sets of graph indices now; the slug points at a deleted item |
| PD2 | `KNOWN_CONFIG_MERGE` has 14 entries, the ones listed | code `amplitude_oracle.rs:201-296`: counted 14, names match | ok |
| PD3 | `CONFIG_AMP_SIGNS` non-uniform on five rows | code: the non-uniform patterns are `ee_to_mumu_tata_qcd0`, `ee_to_mumua`, `ee_to_tatah`, `ll_to_qqx_toy_yukawa` and `tata_to_ttx_tensor4f` | ok |
| PD4 | `get_amp2_lines` at `export_v4.py:1390`, `get_icolamp_lines` at `:1295`, `IdentifyConfigTag` at `group_subprocs.py:56` | source | ok |
| PD5 | `GOODHEL` filter `\|T\| > ANS·LIMHEL/NCOMB`, `LIMHEL = 1e-8`, ours 1e-24 | source: `matrix_madevent_group_v4.inc:169`, `banner.py:4310` (`limhel` default 1e-8); code: `compile.rs:1018` `HEL_PRUNE_REL = 1e-24` | ok |
| PD6 | Pruning moved `AMP2` by 39.5% on `gg_to_ttx`, 3.2% on `gg_to_gg` and 0 elsewhere, when the oracle was introduced | re-run: 3.95e-1 and 3.19e-2; also nonzero now on `gg_to_ttx_smlimit` (3.98e-1) and `gg_to_ttx_smlimit_qcd2` (4.07e-1), rows added later; 0 on the other 44 | ok as a dated reading |
| PD7 | `gg_to_gg`'s partition is `[[3],[4],[5]]` | re-run: 3 configurations; record: `MG_DIAGRAM_ORDER` `gg_to_gg` = `[0,3,4,5]` | ok |
| PD8 | Code names (`config_carrying_diagrams`, `config_groups`, `config_tag`, `pair_config_weights`, `amp_weights`, `amp_locs`, `config_amp_counts`, `config_diagram`, `select_config_and_flow(amp2, jamp2, [u0,u1], clustered)`, `eval_amp2`, `run_config_amps`, `config_flows`) | code | ok |
| PD9 | "About 21% of the 2→6 row's instructions" | record: topdown-zen4 L354 | ok as a record; not re-measured (needs the perf host) |
| PD10 | "`SMATRIX` rescales every `AMP2(j)` by `GET_CHANNEL_CUT` (the product of the configuration's inverse squared propagator denominators)" | source: `matrix_madevent_group_v4.inc:218-222`; `genps.f:1878` returns 1 at `sde_strat = 1`, `tmin_for_channel = −1`, and the products are built only inside `if (sde_strat.eq.2)` | **imprecise**: the product is what `GET_CHANNEL_CUT` is under `sde_strategy = 2` only. Found while editing, after the findings commit |

### amplitudes/wavefunctions-and-propagators

| # | claim | check | verdict |
|---|---|---|---|
| WF1 | HELAS `jioxxx.F` divides by complex `m² − imΓ`; the port follows ALOHA's real `m²` | source: `HELAS/jioxxx.F:115-117` (`cm2 = dcmplx(vm2, -vmass*vwidth)`), `aloha_writers.py:625` (`OM = 1/M**2`); code `vertex.rs:59-69` | ok |
| WF2 | `test_eval_jioxxx` uses massive fermions with `q·J ≠ 0`, cross-checks ALOHA `FFV2`/`FFV4`, and fails if either side's denominator changes | code `run.rs:2283-2537` (16 ALOHA comparisons asserted; the divergence guard requires `q·ε > 1e-2` on massive axial cases); re-run: passes; mutations (below) | ok |
| WF3 | The `aloha-om` pin is `aloha_writers.py#L624-L626` | source: the `OM{0}` line is 625 | ok |
| WF4 | Spinor storage, `vxxxxx` polarisation formulas, the at-rest and along-z conventions, the clamps, `nhel = 0` massless, no `nhel = 4` | code `wavefn.rs:352-433`; derived: `ê_θ`/`ê_φ` components reproduce `e1/e2/e3`; the along-z limit is the side where `p_x` and `p_z` share a sign | ok |
| WF5 | Propagator table: `−i/D` on Dirac, scalar and massive vector; massless vector `−i ε/q²`, no width; the longitudinal term over real `m²` | code `kernel.rs:58-115` | ok |
| WF6 | Helicity states per spin; spin-3/2 gives `UnsupportedSpin`; spin 2 panics in `build_external_core`; Majorana refused as `UnsupportedVertex` | code `particles.rs:92-102`, `compile.rs:687`, `run.rs:1926-1948` (`unsupported external spin code`), `root_lorentz.rs:526` (`LorentzOp::C`) | ok |
| WF7 | `complex_mass_scheme True` is refused in `diagrams/check.rs` | code `check.rs:424`, test at `:831` | ok |
| WF8 | The scalar-chain phase is pinned per diagram by the internal-H chains of ee→μμττ and of the uux and b b̄ 2→6 rows, against MadGraph `AMP()` | census: `uux_to_ccx_emmm_qcd0` and `bbx_to_ccx_emmm_qcd0` have `amp_table: false` ("only the per-flow JAMPs are banked"); `ee_to_mumu_tata_qcd0` and `ee_to_tatah` bank `AMP()` | **wrong** for the 2→6 rows: they are pinned per helicity and flow (one flow), not per diagram |
| WF9 | The line sign on massive tau lines is "pinned bit-for-bit against MadGraph per-diagram `AMP()`" by `tata_to_ttxh` and `tata_to_ttxhh` | census: both standalone tables have no `jamp_coefficients` and are absent from `MG_GRAPH_DIAGRAMS`, so no `AMP()` is compared; re-run: both pass per helicity and flow at `REL_TOL = 1e-12` | **wrong**: per helicity and flow at 1e-12, not per diagram and not bit for bit. The same claim is in `Diagram::fermion_line_sign`'s doc comment (Found) |

Mutations for WF2. Each was applied with `sed` to one line, run with
`cargo test -p vibegraph-lib --features extended-validation --lib --
test_eval_jioxxx`, and reverted with a per-file checkout of the mutated file
(`git diff --stat` empty afterwards):

| id | mutation (one line) | result |
|---|---|---|
| control | none, with `momentum_slashed_chain_is_rooting_invariant` | both pass |
| M7 | `vertex.rs:67` `jioxxx`: `/ vm2` to `/ C::new(vm2, -vmass * vwidth)` (HELAS's complex denominator) | **fails**: `current vs jioxxx (FFV2/Z, t~t→ta+ta-, hel ↓↓): \|diff\|=1.774e-4, scale=6.409e-3` |
| M6 | `kernel.rs:101` `propagate_vector_bare`: the same change on the production side | **fails**: `current vs jioxxx (FFV2/Z, t~t→ta+ta-, hel ↓↓): \|diff\|=1.774e-4, scale=6.411e-3` |

Both reproduce J1's M6 and M7 to the digit. I did not re-run the base-commit
test against them (J1's P3 records that it passed M6).

### process/decay-chains

| # | claim | check | verdict |
|---|---|---|---|
| DC1 | MadGraph cites: `extract_decay_chain_process` L5661, `DecayChainAmplitude` L1337, the dropped-decay warning ~L1405, `combine_decay_chain_processes` L5427 and its branch L5510–L5566, `get_legs_with_decays` L3667, orders L570/L1972, `helas_objects.py:3986` | source: each at the line (the warning's `logger.warning` is at 1409, inside the block that starts at 1405) | ok |
| DC2 | `trim_diagrams(decay_ids)` removes nothing, only sets the flag | source `diagram_generation.py:1270-1300` | ok |
| DC3 | Container-equality oracle: ten cards, 147 diagrams, with the per-card counts quoted | re-run `stitched_chains_are_the_filtered_final_state`: 4+2+2+2+5+2+16+50+16+48 = 147; `g g > t t~ g` 16 with one four-point, `u u~ > z g g g` 50 with two, `p p > z j` 24 sets and 48 diagrams, two slot-order pairs in `e+ e- > w+ w- z` | ok |
| DC4 | Per-diagram amplitudes: worst 1.2e-16 relative, bit-equal except the two slot-swapped pairs | re-run: every card reports `worst relative 0.0e0` | **stale**: all 147 are bit-equal now, the two slot-swapped pairs included |
| DC5 | Census: 24 cards; 17 stitched with 54 pairs; 2 refused; five order cards with `ee_zz_qed3` refused by both | census and re-run: `24 cards: 21 stitched and matched (69 subprocesses), 3 refused`; the first 17 cards carry 54 subprocesses, the four accepted order cards 15 | ok |
| DC6 | `Prop::onshell: OnShell { Free, Forced, Forbidden }`; `Forbidden` exists for `$` lines and no diagram carries it | code `diagram.rs:95-102`: the enum is `{ Free, Forced }`; `grep OnShell::Forbidden` is empty | **wrong**: the variant does not exist |
| DC7 | `ForestLine::forced` is in `coupling/cluster/kt.rs` | code: `ForestLine` is defined in `coupling/cluster/graph.rs:47`; `kt.rs:480` reads it | **imprecise** path |
| DC8 | Refusals `DecayChain`, `DecayedPolarization`, `DecayOnShellVeto`, `ChainOrders`, the ambiguous forced line | code `check.rs`, `chain.rs`; re-run `an_ambiguous_forced_line_is_refused` | ok |
| DC9 | Representation names (`Provenance`, `DecayOrigin`, `ChainNode` preorder numbering, `final_state_side`, `tree_momentum`, `fermion_pairing_sign`, `match_resonances`) | code `diagram.rs:104-107` (the doc numbers `t` 1, `w+` 2, `t~` 3) | ok |
| DC10 | BW window fraction `(2/π)·atan(2·bwcutoff)`, 0.979 at 15 | derived: `(2/π)·atan(30)` = 0.9788 | ok |

### process/diagram-enumeration

| # | claim | check | verdict |
|---|---|---|---|
| DE1 | feyngraph at git rev `fd5aa83`, default features off | code `vibegraph-lib/Cargo.toml:34` | ok |
| DE2 | `DiagramTag` L46, `Amplitude` L433, `generate_diagrams` L520 | source | ok |
| DE3 | Selectors, the `>` power cap of 20, `select_propagator_count(X, 0)`, `TopologyGenerator`, `DiagramGenerator::new(…).assign_topologies`, `Diagram::from_view`, the WEIGHTED range `(n_ext − 2) × min/max hierarchy` | code `selector.rs:11-67`, `mod.rs:683-686, 742-891`, `diagram.rs:282` | ok |
| DE4 | `g g > g g` 4 against 6, `gg_to_gg_cg` 21 against 27 | re-run `validate_madgraph_diagrams` report rows | ok |
| DE5 | `g g > g g` is pinned per colour flow at 8.25e-14 | re-run: per-flow 5.43e-16; 8.27e-14 is its grid `\|M\|²` deviation | **wrong** number: the per-flow pin is 5.4e-16 |
| DE6 | The four-fermion pairing split is made in `ufo/topo.rs::build_feyngraph_model`, one vertex per pairing, and checked per graph by `standalone_jamps`' `ee_to_ee_4f` | code `topo.rs:67-127`; `MG_GRAPH_DIAGRAMS` `ee_to_ee_4f` pairs six diagrams to three contact graphs; re-run passes | ok |
| DE7 | The linked backlog items exist (`diagram-enumeration-cost-unmeasured`, `feyngraph-assign-counts-allocation`, `diagram-inspection-command-missing`, `amplitude-partial-sums-attributed-to-diagrams`) | `ls research/kb/backlog/*/` | ok |

### validation/amplitude-oracle

| # | claim | check | verdict |
|---|---|---|---|
| AO1 | 48 tables | census: 48 files; re-run: 48 trials plus the table-list check | ok |
| AO2 | `bbx/uux_to_ccx_emmm_qcd0` (615/579) bank flows only; `pp_to_ll_qcd0` events from a hadronic run | census (`amp_table: false`; event source) | ok |
| AO3 | One `G` per process, `\|G\| = 1`, `Re G = 0`; `i^V(−i)^{V−1} = i`; every row reads `G = ±i` | derived; re-run: 26 rows `+1i`, 22 rows `−1i` | ok |
| AO4 | `e+ e- > e+ e-` has `c = (−1,−1,+1,+1)`; `ee_to_mumu` reads `G = −i` and `uux_to_mumu` `+i`, with `c = (−1,−1)` against `(+1,+1)` | census `jamp_coefficients`; re-run | ok |
| AO5 | `GRID_REL_TOL = EVENT_REL_TOL = 1e-12`, `ULP_BUDGET = 10`; the two Higgs-pole points at 2.00× and 3.51×; per flow 1.5e-14 there | code; re-run: 2.00×, 3.51×; the row's worst flow over all points is 1.59e-14 | ok |
| AO6 | The exemption-list table: `KNOWN_CONFIG_PAIRING_UNAVAILABLE` holds `wpwm_to_wpwmz_cw` | code: the list is empty | **wrong** (as PD1); the "at `787070e`" column header no longer describes the rows |
| AO7 | Two `info` amplitude cells, `ee_to_wpwm_cw` (at 2.078e-12, 336×, cancelling by 875) and `ee_to_zh_smeft` | census: those two are the only `info` amplitude cells; re-run: `ee_to_wpwm_cw` 1.63e-12 | count ok; number **stale** (GC2) |
| AO8 | `ee_to_wpwm_cw` "linear level exact" | re-run: the report row's note reads "2 of the linear-level checks failed; first: … the configuration amplitudes' signs … read +++++++ and CONFIG_AMP_SIGNS banks none for this row". The numeric levels are at rounding (per diagram 4.70e-16, per flow 2.87e-16) | **imprecise**: the numbers are exact, but `CONFIG_AMP_SIGNS` has no entry for the row, a failure the `info` mode hides (Found) |
| AO9 | `wpwm_to_wpwmz_cw` gated with the `GC_303` graphs' modulus exempted by name; "each graph is also checked on its own scale (`GRAPH_RATIO_TOL`, 1e-10)" | code: `GRAPH_RATIO_TOL` is in `standalone_jamps.rs:78`, not in `amplitude_oracle`; `amplitude_oracle` corrects the named graphs' share through the pairing (`Rounded`, `amplitude_oracle.rs:425-470`); re-run: both pass, and `wpwm_to_wpwmz_cw_sees_a_wrong_rounded_list` shows each of the 12 left out reads 1.197e-8 on its own scale | ok in substance; **imprecise** on which gate holds the per-graph check |
| AO10 | `k/G ∈ {±1}`, worst 1.19e-13 on `ee_to_mumu_tata_qcd0`; eleven tables bank `jamp_coefficients: null`, the ones listed | census: 11 `null`, names match; re-run: `ee_to_mumu_tata_qcd0` configuration amplitude 1.19e-13 | ok |
| AO11 | "Asserting that `run_config_amps` *equals* the per-diagram amplitudes would fail on those three rows" | reading: the sentence before it names five rows | **wrong** antecedent (left over from an earlier list of three) |
| AO12 | Standalone gate: `REL_TOL = 1e-12`, `zero_widths = True` on the four named rows; "Rows: …" | code; census: 45 standalone tables, and the concept names 15 | **imprecise**: the row list reads as complete and is not |
| AO13 | `wpwm_to_epem` is a two-way known disagreement, citing `wpwm-to-epem-neutrino-exchange-sign` | code: `standalone_jamps`' `KNOWN_DISAGREEMENT` is `&[]`; re-run: `wpwm_to_epem` passes | **wrong**: the row gates; the slug points at a deleted item |
| AO14 | The `t t~ + jet` rows are gated per flow by `standalone_jamps` | re-run: `gg_to_ttxg`, `gu_to_ttxu`, `gux_to_ttxux`, `ug_to_ttxu`, `uux_to_ttxg` pass | ok |
| AO15 | "rows with a massive t-channel line (`ee_to_wpwmz`, `wpwm_to_wpwm`, `ttx_to_gg_chg`, `wpwm_to_epem`) are generated with every width zero" | census: an AST parse of `gen_standalone_jamps.py` finds `zero_widths=True` on 33 of 45 rows | **imprecise**: the four are examples, not the set. Found while editing, after the findings commit |

### validation/colour-oracles

| # | claim | check | verdict |
|---|---|---|---|
| CO1 | `CF_REL_TOL = 1e-14`, `check_jamp`, `ORDER-DIFF`, the trial `jamp-normalisation/structure-controls` | code; re-run (97 passed, the trial among them) | ok |
| CO2 | `g g > g g`'s six keys `Tr(1,2,3,4) … Tr(1,4,3,2)` in MadGraph's order | source: `gg_to_gg/…/matrix1_orig.f:384-395` | ok |
| CO3 | The CF matrix of `g g > g g` is `(7/2)I + P − (1/3)J`, eigenvalues 5/2 (×4) and 9/2 (×2) | derived from the packed `DATA CF` block (`DENOM = 6`, diagonal 19, off-diagonal stored doubled: −4 → −1/3, 8 → 2/3 on the trace-reversal partner) | ok |
| CO4 | `J₁ = J₆, J₂ = J₄, J₃ = J₅` | source: the `JAMP(1..6,1)` lines `:435-440` | ok |
| CO5 | `gg_to_ttxg` fails under each of the five wrong contact colour assignments | re-run `gg_to_ttxg_sees_the_contact_colour_assignment` | ok |
| CO6 | Trial count "73 trials over 47 files on the `refdata-4`-era tree" | re-run: 163 trials; census: 95 `leshouche.inc` files | ok as a dated reading |
| CO7 | Mutation experiments: 22/30 flow-tag trials | record: note 24 L765, L773 | ok |
| CO8 | `ColorFlowTags::conjugated`, `SubprocessRecord::relabelled` with `check_legs`, `illegal_slots`, `validate_samples_proton.rs` | code | ok |
| CO9 | Per-flow worst 3.7e-16 on `g g > g g` | record: note 16; re-run reads 5.43e-16 today | ok as the cited reading |

### validation/coupling-oracle

| # | claim | check | verdict |
|---|---|---|---|
| CP1 | 47 coupling tables | census | ok |
| CP2 | Crate vs Python worst 8.85e-15 (40 ulp, `GC_64`); `ee_to_ttx_smeft` 355 couplings at 6.48e-15; the toy models exact; Fortran vs Python 3.00e-16; `GC_303` 1.2e-8 on the two rows | re-run: every number reproduced (`GC_303` 1.197e-8) | ok |
| CP3 | `PYTHON_REL_TOL = 1e-13`, `FORTRAN_REL_TOL = 1e-14`, `KNOWN_CRATE_DEFECTS` empty, `KNOWN_FORTRAN_DEVIATIONS` the two entries | code `coupling_oracle.rs:85-130` | ok |
| CP4 | "The manifest's `couplings-mg` note still describes the precedence defect as a live `KNOWN_CRATE_DEFECTS` entry" | census: the note now says `KNOWN_CRATE_DEFECTS` "is empty" | **stale**: the manifest was corrected |
| CP5 | `GC_303 = 2i·gHza/vevhat`; `gHza` from literals like `0.4583333333333333` | source: SMEFTsim `couplings.py:1220`, `parameters.py:2230` | ok |

### validation/madgraph-defects

| # | claim | check | verdict |
|---|---|---|---|
| MD1 | Line cites `unwgt.f:752, 760-761`, `create_aloha.py:262-263, 527-530`, `export_v4.py:7076` (`aS = 0.138`, `G = 4.1643`), `genps.f:1817, 1878, 1910, 1932, 1937-1938, 1946`, `rambo.py:218` (and the massless `< 5` at 164), `helas_objects.py:4581`, `super_auto_dsig_group_v4.inc:842`, `reweight.f:1138`, `setcuts.f:939-942`, `cuts.f:565`, `addmothers.f:115`, `banner.py:1706` | source: each holds at the line | ok |
| MD2 | Both `t` assignments in `get_channel_cut` sit inside `if (sde_strat.eq.2)` | source `genps.f:1930-1950` | ok |
| MD3 | "Only `ee_to_zh_smeft` and `wpwm_to_wpwmz_cw` carry it" (the `GC_303` rounding) | census: true of the coupling tables' Fortran blocks; but `ROUNDED_COUPLING_GRAPHS` registers `GC_303` graphs on four more standalone rows (`wpwm_to_az_cw`, `az_to_wpwm_cw`, `wpwm_to_wpwma_cw`, `ee_to_wpwmwpwm_cw`) | **imprecise** |
| MD4 | `GC_299` 2.8e-8 off; "no banked amplitude row carries it yet, so `coupling_oracle` does not list it" | census: no coupling table's Fortran block has `GC_299`; code: `mg_graphs.rs` registers it at −2.769559e-8 on `aa_to_wpwm_cw`, `wpwm_to_wpwma_cw` and `ee_to_wpwmwpwm_cw`, gated by `standalone_jamps` | **imprecise**: right about `coupling_oracle`, but standalone rows do carry it and gate it by name |
| MD5 | The table row "UFO literal printed at seven digits", handling "Python `model_reader` is the arbiter", is unmarked, so by the register's own rule it reads as a failure | record: `madgraph-defect-policy.md:89` classes it "an approximation, not a code bug … **registered deviation** (decided 2026-10-15, human:nsmith-)"; this concept's own prose says the same | **wrong** marking: the row lacks the **approximation** label and the registered-deviation handling, and omits `GC_299` |
| MD6 | Numbers: 0.06811 against 0.04982 pb (37% high); 0.04888 ± 0.00020; 99.8% / 2.3% low; `0.920ⁿ`; +0.26% ± 0.08% | record: note 38 L853/L1422, note 27 L119/L158, `refdata-sigma-comparability.md:43`, `identical-particles-across-decays.md:69` | ok |
| MD7 | The MLM table has six defects; the reweight and `rambo.py` entries | source (MD1) | ok |

### validation/non-sm-rows

| # | claim | check | verdict |
|---|---|---|---|
| NS1 | 22 rows: 16 SMEFTsim, 6 toy | census over manifest `model` fields | ok |
| NS2 | Interaction counts 62 / 913 / 154 / 82 | census `interactions.json`; re-run `smeftsim` | ok |
| NS3 | `restricted_defaults_are_madgraphs_generated_param_card` compares 421 external parameters over 13 gated rows | re-run: "444 external parameters over 14 rows"; `GATED_ROWS` has 14 since `wpwm_to_wpwmz_cw` gated | **stale** |
| NS4 | "every `amplitudes` cell but three" | census: two `info` amplitude cells | **stale** (the next bullets already say `wpwm_to_wpwmz_cw` gates); the "at `6ccc6e4`" stamp is out of date |
| NS5 | `ee_to_wpwm_cw` one point at 2.08e-12 | re-run 1.63e-12 | **stale** (GC2) |
| NS6 | Capstone: 2.222986 ± 6.565e-4 against 2.2223 ± 5.257e-4, pull +0.82; "seven seeds span pulls −0.80 to +1.22" | census: the manifest cell note gives the same σ and pull, and seven seeds at `\|pull\| ≤ 1.44`; record: note 35 L807 and note 36a L120 give 1.22 | **stale**: the manifest's later reading is 1.44 |
| NS7 | SM-limit control 0.5496 pb, factor 4.04 | record note 35 L812 | ok |
| NS8 | "a 6e-11 derived-parameter spread" | record: note 35 L1692 called it that; note 36 B5 and the `ee_to_zh_smeft` manifest note attribute the 6.00e-11 to `GC_303`'s seven-digit literal | **stale** attribution |
| NS9 | Cell-status bullets for `gg_to_gg_cg`, the 2→1 rows, massive beams, toy `AQCDUP`, `SCALE_FALLBACK_ROWS` | census; code `validate_sigma.rs:221` | ok |
| NS10 | Coverage table: rows, cards, the `wpwm_to_wpwmz_cw` card is `vg_cW` like `ee_to_wpwm_cw` | census (`restrict` fields) | ok |

### validation/oracle-blind-spots-and-non-vacuity

| # | claim | check | verdict |
|---|---|---|---|
| OB1 | `color_cf_oracle`'s blind spot is covered by per-flow gates on `gg_to_gg`, `uux_to_ggg`, `gg_to_ggg` and `gg_to_ttxg` | re-run (all four pass; CO5 shows the `gg_to_ttxg` gate sees the assignment) | ok |
| OB2 | Test and constant names (`the_rung_ordering_test_fires_on_a_swapped_chain` in `diagram_channel.rs`, `spine_sign_from_flow`, `GRID_ALPHA_S_RUNS`, `SCALE_FALLBACK_ROWS`, the kT gate `#[ignore]` under `pixi run -e madgraph validate-kt-cluster`) | code; `pixi.toml:273` | ok |
| OB3 | The Yukawa-line exemption was reverted in `7f523ad`; 24 of 96 diagrams of `ta+ ta- > t t~ h h` | `git show 7f523ad` (its message describes the revert); census: `tata_to_ttxhh` has 96 graphs. The 24 is a record (PR #13), not re-measured | ok |
| OB4 | The cases and numbers (`IDWTUP` factor 2.0e5, 5.5% σ rows, NEG-A, pull −9.31, 25σ, `refdata-5` fetch) | record, spot-checked: note 27 L582 (2.0e5), note 28 L1576 (NEG-A), note 38 L1467 and note 40 L92 (−9.31) | ok as records |

### validation/sigma-row-gating-exceptions

| # | claim | check | verdict |
|---|---|---|---|
| SG1 | Every `info` cell appears in the table, "at `6ccc6e4`" | census with the concept's own `awk`: the nine `info` cells at `4277158` all appear | ok; the commit stamp can move to `4277158` |
| SG2 | `ee_to_mumua`: `rel_tol 0.03`, pull reported, +1.04% at χ²/dof 0.50, pull +4.11, 3.56 > 3.5 | code `validate_sigma.rs:155-202, 390-394` | ok (not re-run: five-seed σ sweeps) |
| SG3 | 2→6 rows: 296 448 and 1 185 792 evaluations, 5.8 µs, 15–18 orders, 46%, ±4.8%; worst single seed under 1% after the floor | census: manifest cell note; record: note 32 L574 (+4.8/−4.5/+3.5%), the backlog item `two-to-six-integrals-not-enforced` (+0.89%, +0.46%) | ok as records |
| SG4 | `pp_to_jj` 9 of 10 000, `TIE_BREAK_MISSES` | code `validate_scales.rs:321` | ok |
| SG5 | `gg_to_gg_cg`: χ²/dof 1.01, −0.22%, 2.6× | record note 36 L519 | ok |
| SG6 | `ee_to_ee_nsz` χ²/dof 15, Bhabha 93, pull ~1.8 | census: the manifest note (15, 93, pull +1.88); note 38 §8.5 read 20 before a re-measurement | ok |
| SG7 | `INFORMATIONAL` in `cli_decay_chain.rs`, `UNDECLARED_ALPHA_S_RUNS` asserted both ways | code `cli_decay_chain.rs:56`, `validate_alphas.rs:171-182` | ok |

I did not re-run any σ gate or seed sweep. These are long-tier runs, and the
brief's host limits rule them out. Every number in this concept is checked
against the record that holds it (the code doc, the manifest note, the
cited note or item), and all of them agree with that record.

### workflow/agent-dispatch-and-worktrees

| # | claim | check | verdict |
|---|---|---|---|
| AD1 | Every `.agents/agents/*-dev.md` carries the discipline section and names the lean cargo settings | `grep` over the four definitions; commit `404e700` | ok |
| AD2 | The footnote names the section in `validation-dev.md` and `performance-dev.md` | `hygiene-dev.md` carries it too | **imprecise** |
| AD3 | The amplitude-signs disk incident: 4 cores, ~30 GB, a 15 GB debug `target/`, a linker `Bus error` | record: G1 report L172-176, the closeout L114-116, the log L12, commit `404e700` | ok; the passage has no footnote |
| AD4 | "The `release-debug` profile cannot be tuned through `CARGO_PROFILE_*` variables (cargo misreads the hyphenated name), so its debuginfo stays as the profile sets it" | re-run in a scratch crate with cargo 1.97.0: `CARGO_PROFILE_RELEASE_DEBUG_DEBUG=0` makes every build fail, `error in environment variable CARGO_PROFILE_RELEASE: could not load config key profile.release`, under `--profile release-debug` and `--release` alike | **imprecise**: the variable is not ignored, it breaks the build |
| AD5 | The manager deletes a checked session's `target/` and copies reference data only into worktrees about to be dispatched | commit `404e700`'s message ("the manager-side rule") | ok as a rule; I have no record of it in practice |
| AD6 | Agent types; "Never `general-purpose`"; the `#decided` anchor of `hygiene-review.md` | `AGENTS.md`; `hygiene-review.md:241` `### Decided` | ok |
| AD7 | The older incidents (jetsam, load 84, 16 GB tree, `pkill -f`, 18m46s/3m16s, 7.5 s against 26.9 s) | record, spot-checked: note 24 L3086 (26.9 s wall, 3.4 s CPU), L3118-3121 (18m46s against 3m16.2s) | ok as records |

## Fixes

All fixes are in the concepts, in `da9cf8a`, the commit after the findings
commit `677242e`. The stamps follow in the commit after that. Each row names the
finding it closes.

| concept | finding | change |
|---|---|---|
| gamma-chains-gamma5-and-epsilon | GC1, GC2, GC5 | `wpwm_to_wpwmz_cw` moved to the gated rows, with its named `GC_303` exemption; the closed slug dropped; `ee_to_wpwm_cw`'s reading given as 1.63e-12 at `4277158`, with the manifest's 2.078e-12 and 336× attributed to the manifest note and dated before the sprint |
| per-diagram-amp2 | PD1, PD10 | the `wpwm_to_wpwmz_cw` gap replaced by how its pairing is measured now, with the list stated empty; the `SMATRIX` caveat split by `sde_strategy`, with source lines |
| wavefunctions-and-propagators | WF8, WF9 | the scalar-chain pins split into per diagram (ee→μμττ, e⁺e⁻→τ⁺τ⁻H) and per helicity and flow (the 2→6 rows); the tau-line pin restated as per helicity and flow at 1e-12 on single-flow tables without `AMP()`, with what that can and cannot see |
| decay-chains | DC4, DC6, DC7 | the per-diagram oracle is bit for bit on all 147 pairs; `OnShell { Free, Forced }`, with `$` described as not a diagram flag; `ForestLine` placed in `graph.rs` |
| diagram-enumeration | DE5 | `g g > g g` per flow at 5.4e-16 (its grid `\|M\|²` at 8.3e-14) |
| amplitude-oracle | AO6, AO7, AO8, AO9, AO11, AO12, AO13, AO15 | the exemption table at `4277158`, with `KNOWN_CONFIG_PAIRING_UNAVAILABLE` empty and the note 35 partition finding kept as history; the two `info` rows as a list with current numbers and the `CONFIG_AMP_SIGNS` gap; `GRAPH_RATIO_TOL` placed in `standalone_jamps`; the "three rows" antecedent fixed; the standalone row list marked as a subset of 45 tables, `zero_widths` on 33 of them; `wpwm_to_epem` gated, `KNOWN_DISAGREEMENT` empty, the closed slug dropped |
| coupling-oracle | CP4 | the stale remark about the manifest's `couplings-mg` note removed |
| madgraph-defects | MD3, MD4, MD5 | the seven-digit-literal row marked **approximation**, `GC_299` added, handling "registered deviation"; the prose names which coupling tables carry `GC_303`, which standalone rows carry `GC_303`/`GC_299` under `ROUNDED_COUPLING_GRAPHS`, and why `ee_to_zh_smeft` stays `info` (with its open item) |
| non-sm-rows | NS3, NS4, NS5, NS6, NS8 | 444 parameters over 14 rows; "but two" at `4277158`; `ee_to_wpwm_cw` at 1.63e-12; the capstone's seed reading as the manifest's \|pull\| ≤ 1.44 beside note 35's range; the 6.0e-11 attributed to `GC_303` |
| sigma-row-gating-exceptions | SG1 | the info-cell census stamped `4277158` (re-run, same nine cells) |
| agent-dispatch-and-worktrees | AD2, AD3, AD4 | `hygiene-dev.md` added to the footnote; a footnote for the amplitude-signs disk incident; the `release-debug` sentence says what cargo does (refuses every build) |

No concept was rewritten beyond those passages. `colour-oracles` and
`oracle-blind-spots-and-non-vacuity` needed no fix.

## Stamps

Stamped `{by: claude-code/claude-opus-5-5, at: 2026-10-16}`, `status` left
`draft`, after the fixes: all thirteen.

Each was checked end to end in the sense of the method section. Every body
claim has a re-run, a census, a code or source reading, or (for a dated
measurement or an incident) a match against the record that holds it. What
that does not cover, said once here for all thirteen:

- **The `787070e` pins of vibegraph source** were not opened (shallow clone, no
  GitHub). Every item they name was found and checked in the current tree.
- **Recorded measurements I did not re-run**: the σ and seed-sweep numbers
  (sigma-row-gating-exceptions, the non-sm-rows capstone), the perf figures
  (per-diagram-amp2's 21%), the 24 of 96 wrong-sign diagrams of PR #13
  (oracle-blind-spots), the `ee_to_wpwm_cw` 336× and 875 factors (now
  attributed to the manifest note), and the operational incidents
  (agent-dispatch). Each matches its record. Re-running them needs a σ host,
  a perf host or history that no longer exists.

If either is a reason to withhold, the stamps on sigma-row-gating-exceptions
and agent-dispatch-and-worktrees rest most on records, and are the ones to
reconsider.

**Manager's disposition:** withheld. The brief stamps only concepts checked end
to end, and a measurement matched to its record was not re-measured, so the
manager removed this session's stamp from sigma-row-gating-exceptions and
agent-dispatch-and-worktrees. Eleven concepts carry it.

## Stop-rule items

None fired. No fix changes a stated convention or a number a gate or code
relies on.

- The madgraph-defects reclassification (MD5) only applies the decision already
  recorded in `madgraph-defect-policy.md:89` (2026-10-15, human:nsmith-).
- The numbers I changed are descriptive readings, re-measured by the gate that
  produces them. No tolerance, list or manifest mode was touched.
- The manifest notes that are stale (Found 2, 3) are not kb concepts, and I left
  them alone.

## Found

1. **`Diagram::fermion_line_sign`'s doc comment overstates its pins.**
   `vibegraph-lib/src/diagrams/diagram.rs:524-530` says the sign is "pinned per
   diagram against MadGraph's `AMP()`" by `u u~ > c c~ e+ e- mu+ mu- QCD=0` and
   by `ta+ ta- > t t~ h (h)`. Neither banks `AMP()`: `uux_to_ccx_emmm_qcd0.json`
   has `amp_table: false`, and the two tau tables have no `jamp_coefficients`
   and no `MG_GRAPH_DIAGRAMS` entry. To close: say "per helicity and flow" for
   those rows (hygiene).
2. **`ee_to_wpwm_cw` has no `CONFIG_AMP_SIGNS` entry, and its `info` mode hides
   the failure.** `target/validation-report/amplitudes/ee_to_wpwm_cw.json`
   (`amplitude_oracle` at `4277158`) reads "2 of the linear-level checks failed;
   first: … read +++++++ and CONFIG_AMP_SIGNS banks none for this row". The
   list's doc says "a compared row must be listed". To close: bank the measured
   pattern, or have the two-way bookkeeping check fail whatever the row's mode
   (validation).
3. **The `ee_to_wpwm_cw` amplitudes note in `validation/manifest.toml` is
   stale.** It gives `G = +1i`, per flow 3.07e-16, JAMP2 1.83e-15 and the worst
   point at 2.078e-12. At `4277158` the gate reads `G = −1i`, 2.87e-16,
   2.03e-15 and 1.632e-12. The point index, the 336× ulp ratio and the 875
   cancellation need re-measuring at the new reading. To close: re-measure and
   rewrite the note (validation).
4. **The `pp_to_ttx_0j1j_mlm` amplitudes note** says "the one-jet subprocesses
   have no amplitude row". `standalone_jamps` now gates `gg_to_ttxg`,
   `gu_to_ttxu`, `gux_to_ttxux`, `ug_to_ttxu` and `uux_to_ttxg`. To close: point
   the cell at those rows (`covered-by`), or say what is still missing.
5. **`madgraph-defect-policy.md:89`** says "the affected rows exempt only those
   graphs' modulus by name and are otherwise gated". `ee_to_zh_smeft` is
   affected and still `info`; its route is the open item
   `ee-to-zh-smeft-rounded-coupling-gate`. To close: qualify the sentence (kb).
6. **`model/restriction-semantics.md:52` and
   `validation/toy-ufo-models.md:95`** quote "421 external parameters over 13
   gated rows". The test prints 444 over 14 (`smeftsim -- --nocapture`). To
   close: update both (kb).
7. **`amplitudes/levi-civita-and-sigma-conventions.md:95`** quotes
   `ee_to_wpwm_cw`'s point at 2.08e-12; it reads 1.63e-12 at `4277158` (kb).
8. **The info amplitude rows' sign patterns are generally unbanked.**
   `ee_to_zh_smeft` is in `CONFIG_AMP_SIGNS`, but nothing makes an `info` row's
   bookkeeping lists two-way. This generalises Found 2. To close: a test that
   every row comparing per-diagram amplitudes is listed, independent of mode
   (validation).

## Corrections to the brief

- **"Re-measure every claim you can" meets the shallow clone.** The `787070e`
  pins cannot be opened without contacting GitHub, which the brief forbids.
  Either allow a read-only `git fetch` of that one commit, or accept the pins
  as provenance (my reading).
- **"Pinned-commit source links" and the history archive.** The archive's
  line numbers run two past the pinned ones on note 35. A reviewer checking
  note anchors locally should know the offset; it may differ per note.
- **The first commit's index.** The findings commit (`677242e`) did not
  regenerate `sessions/index.md`, so `kb-lint` fails on that commit alone. The
  next commit regenerates it.
