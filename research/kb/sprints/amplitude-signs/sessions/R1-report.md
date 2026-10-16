---
type: Session Report
title: "R1 report: independent review of the ten convention concepts"
description: "Every claim of the ten concepts B-H and B-D rewrote around convention_sign, checked against the code, the tests (run), the MadGraph tables, ALOHA and the archived notes, with the per-factor mutation tables re-measured. Findings first, then the fixes, the stamps and what is left."
status: draft
tags: [review, sign-convention, kb, amplitudes, mutations, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: brief, resource: "R1.md", title: "R1 brief"}
  - {id: b-report, resource: "B-report.md", title: "B′ report"}
  - {id: bh-report, resource: "B-H-report.md", title: "B-H report"}
  - {id: bd-report, resource: "B-D-report.md", title: "B-D report"}
  - {id: t1-report, resource: "T1-report.md", title: "T1 report"}
---
Reviewed on `amp-signs/concept-review-a` at `0ec027a` (main's `5e6d2b0` plus
the brief). No code was changed; every mutation and probe below was applied to
the tree by a driver, run and reverted, and `git diff --quiet` held after each.

## How the claims were checked

- **Code claims**: read at `0ec027a` (`root_diagram.rs`, `root_lorentz.rs`,
  `diagram_eval.rs`, `kernel.rs`, `diagram.rs`, `compile.rs`, `lower.rs`,
  `run.rs`, `topo.rs`, the test files).
- **Test claims**: every cited test run (commands and results under "Runs").
- **Mutation tables**: re-measured by `target/r1/mutate.py` (each factor set
  to `+1`, T1's M2–M4, the live-parity cancellation, the Dirac-content
  exemption), against `standalone_jamps`, `amplitude_oracle` and the lib tests
  the concepts name.
- **Census claims**: re-counted by machine (a temporary lib test for the
  four-fermion census; the banked tables read with `python3`; a scratch numeric
  check of every triple-vector Lorentz structure's parity under the W slot
  exchange).
- **Upstream claims**: read in `research/refs/mg5amcnlo` (3.7.1):
  `aloha/create_aloha.py`, `aloha/aloha_object.py`, `models/import_ufo.py`,
  `aloha/aloha_fct.py`, the banked `matrix1_orig.f`.
- **Archived-note claims**: read in `research/kb/history/notes/`. The note
  permalinks pin `787070e`, which this shallow clone does not hold; the archived
  copies carry two more frontmatter lines than the originals, and with that
  offset every cited range in the ten concepts starts at its section heading
  and ends at the next one. The `facts/` files are not archived, so the
  `fact-line-sign` source could not be opened.
- **Recorded measurements** (a session's or a note's number, made with an
  instrument that is not committed: B′'s GF(2) solve and 506-subprocess
  comparison, note 19's 1583/96 480 checks, the rooting study's tables, note
  38's adjudications) were checked against their record, not re-measured. A
  concept that states one as a current measurement, or that rests a current
  claim on one, gets no stamp.

## Runs (unmutated tree)

`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
throughout.

| command | result |
|---|---|
| `cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture --test-threads=1` | `ok. 49 passed` (45 table rows and 4 checks) |
| `cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture --test-threads=1` | `ok. 49 passed` |
| `cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --nocapture` | `ok. 4 passed`; T1's printed numbers to the digit |
| `cargo test -p vibegraph-lib --lib helas::eval::root_diagram -- --nocapture` | `ok. 7 passed; 1 ignored` |
| `cargo test -p vibegraph-lib --lib helas::eval::relabelling -- --nocapture` | `ok. 1 passed`; 12 pairs, every one `flipped against the rest: []` |
| `cargo test -p vibegraph-lib --lib helas::eval::renumbering -- --nocapture` | `ok. 2 passed`: `3730 diagrams, 10454 (renumbering, chain) signs, 4038 amplitude comparisons, 6378 renumberings with leg 0's vertex off index 0, 0 failures` |
| `cargo test -p vibegraph-lib --lib helas::eval::rooting_soundness -- --nocapture` | `ok. 2 passed; 1 ignored` (`literal_sigma_currents_are_rooting_invariant`, `root_override_hook_is_transparent`) |
| `RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib --lib helas::eval::rooting_soundness::all_rootings_preserve_amplitude -- --ignored --nocapture --test-threads=1` | `rooting-soundness sweep: 270 re-rootings, 0 failures across 0 processes` |
| `cargo test -p vibegraph-lib --test gluon_parke_taylor -- --nocapture` | `ok. 2 passed` (5 gluons: worst flow 2.146e-14; 6 gluons: 7.998e-15) |

## Findings, per concept (written before any edit)

Verdicts: **holds** (checked, true as written), **wrong** (checked, false or
stale as written; fixed unless marked otherwise), **imprecise** (true in
substance, misleading as written; fixed), **record** (a recorded measurement,
faithful to its source, not re-measured), **unchecked** (could not be checked
here). Mutation results are in the next section and cited by their ids.

### `amplitudes/convention-sign-inventory`

| # | claim | how checked | verdict |
|---|---|---|---|
| 1.1 | `fermi_sign = diagram.sign × convention_sign × tree.live_reversed_parity()` in `compile_single_diagram` | read `root_diagram.rs:1163-1189` | holds |
| 1.2 | ALOHA: `+i` on spin-0 and spin-½ propagators, `−i(g − qq/m²)` on spin-1, the amplitude closed with `−i` | `create_aloha.py:336-346` (`complex(0,1)` for spins 1 and 2), `aloha_object.py:1697` (`VectorPropagator = complex(0,1)*(-1*Metric + …)`), `create_aloha.py:412` (`complex(0,-1)` when no leg is off shell) | holds |
| 1.3 | every propagator here carries `−i` (`propagate_core`) | `kernel.rs:60-115` (`ri(-F::one())` on fermion, vector and scalar) | holds |
| 1.4 | the arms table's functions and readings | read `propagator_phase_sign`, `vertex_root_signs`, `VertexInfo::{reversed_sign, build_sign, tensor}`, `term_reversed_parity`, `RootedTerm::build_sign` / `add_cut_line` (`−1` per crossed line of a cyclic term), `triple_gluon_sign`, `live_reversed_parity` | holds |
| 1.5 | "pinned by" counts: 25 rows without the scalar term, 22 without the crossed-line term, 17 standalone and 28 oracle rows without the vertex reading, the tensor-path and triple-gluon rows | re-measured, MS, MC, MV-all, MT, M1 | see Mutations; the 28 is **wrong as described** (1.6) |
| 1.6 | "every listed failure is a per-diagram or per-flow disagreement with MadGraph" | MV-all's 28 oracle failures classified from their messages (`target/r1/aofail.py`); the oracle runs the per-diagram and per-flow checks before the `CONFIG_AMP_SIGNS` tripwire and returns on the first violation | **wrong**: 15 of the 28 fail only on `CONFIG_AMP_SIGNS`, a whole-process flip |
| 1.7 | the vertex reading: `ψ̄₁Γψ₂ = ψ̄₂(CΓᵀC⁻¹)ψ₁`, `−Γ` for `γ^μ`, `σ^{μν}`, `+` for `1`, `γ⁵`, `P_χ`; a vertex whose terms disagree is refused (`uniform_sign`) | algebra; `diagram_eval.rs:301` | holds. `term_reversed_parity` counts a `Gamma`/`Sigma` whose own row index is the leg; no structure in the SM, SMEFTsim or the toy UFOs puts a projector or `Identity` at the row leg before a `Gamma` (grep of the four `lorentz.py`), so "per bilinear" and "per op" agree in reach |
| 1.8 | the triple-gluon paragraph | T1 report, `triple_gluon_sign`'s doc, the `triple_gluon` tests (run), M1–M4 | holds |
| 1.9 | "`−1` per quark–gluon vertex is the same convention up to a sign common to the process" | gluon-end counting: `n_qg + n_ggg ≡ n_ext_g (mod 2)` needs every other vertex to carry an even number of gluons. SMEFTsim's UFO has `g g g g g` (`O_G`), `g g g H` (×2), `g g g H H`, `g g g H1` and `q q̄ g H` vertices (vertices.py census) | **imprecise**: true in the SM; false for a model with those vertices, and no row reaches one |
| 1.10 | GF(2) solve: 3495 diagrams, consistent with the reversal parity, unique minimal solution `−1` on the triple-gluon vertex, next weight 9 | B′ report "How it was found" | record |
| 1.11 | "compared, per (diagram, colour chain), with the measured signs on 506 subprocesses (7347 signs) and differed on none" | B′ report step 4: the comparison is the old convention times W1's and F1's measured fixes against the new one, on the renumbering census plus the verified set | **wrong**: most of the 506 carry no MadGraph measurement; it is a comparison with the previous convention |
| 1.12 | `e+ e- > e+ e-` carries `c = (−1, −1, +1, +1)`; `g u~ > e+ e- u~` flips `T(1,5,2) → T(1,2,5)` and `+1 → −1` | `amplitudes/ee_to_ee.json`, `gu_to_epemu.json`, `gux_to_epemux.json` `jamp_coefficients` and `flow_structures` | holds |
| 1.13 | "`g g > t t~` is uniform with crossed top lines carrying 0, 1 and 1 propagators, which is what pins the crossed line's propagator independence" | T1's factor table (`g g > t t~` as `g g > u u~`): `prop` is `+1, −1, −1` and `tg` `−1, +1, +1`, so `fermi_sign` is uniform because the crossed line's propagator phases are offset by the triple-gluon sign; only `Diagram::fermion_line_sign` is propagator-independent on a crossed line | **wrong** (stale): in this convention a crossed line's propagators each take `−1` |
| 1.14 | `CONFIG_AMP_SIGNS` banks the per-process sign as a tripwire | `amplitude_oracle.rs:614, 1931-1970` | holds |
| 1.15 | the "How a new sign gets found" account (`u d > e+ e- u d QCD=0`, three fitted candidates each falsified) | note 28 §S5 (archived) | record |
| 1.16 | Open items and links | the three backlog items exist; `kb-lint` resolves the links | holds |
| 1.17 | sources: `code-*` cite this repo's code at `5d4a346` | `git merge-base --is-ancestor 5d4a346 HEAD` fails: a PR #30 branch commit; `root_diagram.rs` has changed since (T's doc of `triple_gluon_sign`, `triple_gluon.rs` is new). `workflow/knowledge-bundle`: "Our own code is cited by path" | **wrong**: cited by path instead |

### `amplitudes/fermion-line-sign`

| # | claim | how checked | verdict |
|---|---|---|---|
| 2.1 | `Diagram::sign = fermion_pairing_sign × fermion_line_sign`, set in `from_view`; the line sign reads only ends and `P` | `diagram.rs:357-362, 518-578` | holds |
| 2.2 | the line-class table and code excerpt | `diagram.rs:544-555` | holds |
| 2.3 | the slot-binding table (initial / mixed / crossed legs under the two crossings) and `mixed_line_final_legs` | `external_leg`, `mixed_line_final_legs`, `correct_spin_index_for_flow`'s `(ket slot, bra slot)` | holds |
| 2.4 | `CΓᵀC⁻¹` for `γ^μ P_χ` is `−γ^μ P_χ̄`; `chiral_correction` per vertex; the reversal sign is `vertex_root_signs`' | algebra; `root_lorentz.rs:870-948` | holds |
| 2.5 | `(−1)^P` is the propagator phase (`+i` against `−i`) | 1.2, 1.3 | holds |
| 2.6 | a crossed line: `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁`, its propagators counted by `propagator_phase_sign` | `propagator_phase_sign`; note 12 root cause 5 | holds |
| 2.7 | pins: `tata_to_ttxh` (14 diagrams), `tata_to_ttxhh`, `bbx_to_hh`, `every_line_takes_the_propagator_sign`, `qqx_to_o8o8_toy_dcolor` | tables' `n_graphs` (14, 96, 3); tests run; MS fails `qqx_to_o8o8_toy_dcolor` (B′, and here) | holds |
| 2.8 | "`b b~ > c c~ e+ e- mu+ mu- QCD=0`: a **mixed** gauge/Yukawa `b` line … is bit-for-bit only with the −1" | the `b` line runs initial to initial; "mixed" here means mixed vertex content, while the concept defines a mixed line as initial–final. The fact file it cites is not archived | **imprecise** wording; the measurement is unchecked (fact file absent) |
| 2.9 | "Alternatives that fit some data and are false": the first, "−1 per propagator on a crossed final–final line", moves `ee_to_mumu_tata_qcd0` diagrams 0–15 against 17–24 and `g g > t t~`'s t/u against s | note 28 §S5–S6; `propagator_phase_sign` gives exactly that `−1` today, and both rows pass (oracle run) | **wrong** as a statement about the convention: the factor is in the code, beside the other factors of `convention_sign` (on `g g > t t~` it is the triple-gluon sign that restores uniformity, 1.13). It was falsified as one factor added to the anchor-read signs then in use (note 28 §S6 gives both rows) |
| 2.10 | the Dirac-content exemption: fits `qt qt~ > o8 o8` only beside a missing scalar-line `−1`; with it `ta+ ta- > t t~ h h` has 24 wrong signs of 96 | MF-exempt and MF-exempt+MS (Mutations); the 24/96 count is PR #13's record (`7f523ad` is not in this clone; `validation/toy-ufo-models` and `oracle-blind-spots-and-non-vacuity` carry the same number) | see Mutations; 24/96 record |
| 2.11 | "How it is checked": `ud_to_epemud_qcd0`, `ee_to_ee` and the uux/bbx 2→6 classes are "Per-diagram MadGraph rows" | oracle output: `ud_to_epemud_qcd0` `per-diagram 0.00e0 (not banked)`, NCOLOR=2 (its table banks `AMP()` but has no `jamp_coefficients`); the 2→6 tables bank flows only (`amp_table: false`) | **wrong**: those three are compared per flow; only `ee_to_ee` is per diagram |
| 2.12 | caveat: "the absolute per-diagram pattern of `u d > e+ e- u d` is pinned only by its `amplitude_oracle` row. If that row were demoted, the split test alone would pass with the line sign off by a global sign" | as 2.11; a global sign is invisible to the oracle too | **wrong**: the row pins the pattern per flow; the split test checks only that the classes differ |
| 2.13 | the 24 / 9 / 2 split and the `g g > t t~` control | `spine_sign_separates_mixed_line_and_crossed_line_propagators` (run) | holds |
| 2.14 | `P = V − 1` makes the two phrasings identical on tree lines | arithmetic | holds |
| 2.15 | sources: `code-*` at `5d4a346`; `fact-line-sign` | as 1.17; the fact file is not archived | **wrong** (path); fact unchecked |

### `amplitudes/rooting-invariance-and-canonical-form`

| # | claim | how checked | verdict |
|---|---|---|---|
| 3.1 | the sign table and code excerpt | `compile_single_diagram` | holds |
| 3.2 | the live reversed parity is the only rooting-dependent sign, applied twice | `live_reversed_parity` doc and the runtime `resolve_bra_ket`; ML (Mutations) | holds |
| 3.3 | kernels rooting-invariant: 1583 checks, 96 480 checks (note 19 §V5) | archived note 19 lines 410-416, 483-490 | record (measured on the 2026-07 code; the current measurement is the sweep, 3.4) |
| 3.4 | `rooting_soundness` 270 re-rootings, 0 failures | sweep run | holds |
| 3.5 | "worst 2.2e-11 on the census" | `rooting_soundness.rs:46-56` and note 19 say the MG-validated suite, not the census | **imprecise** |
| 3.6 | stitched diagrams rebuild both signs; `g b > w- t, t > w+ b` | `chain.rs:670`; note 38 §D2 | holds (code) / record (the case) |
| 3.7 | "On a census of every rooting of 2553 (diagram, chain) pairs the tree derivation never varied with the rooting" | note 38 §S1: that census ran the line rule then in force, which still read the line's Dirac content | record, attributed as if current; reworded. The current check is `fermion_line_sign_matches_the_rooted_tree_derivation` (every rooting, eight processes; run) |
| 3.8 | `canonical_root` and the tie-break | `root_diagram.rs:783-797` | holds |
| 3.9 | canonical form, `CanonicalDiagram` fields, `provenance` excluded, `Prop::particle` at `endpoints[1]`; 2656 / 3397 | `diagram.rs:195-244, 640-704`, `Prop` doc; note 38 §D2 | holds / record |
| 3.10 | container-equality blind spots; the `w+ w- z` slot-order case | `stitching.rs:209-232`; note 38 §D2 | holds |
| 3.11 | the gates table (sweep scope, renumbering's 40-diagram cap, relabelling's twelve pairs, the debug assertion) | code and runs | holds |
| 3.12 | sources: `code-*` at `5d4a346` | as 1.17 | **wrong** (path) |

### `amplitudes/vector-vertex-signs`

| # | claim | how checked | verdict |
|---|---|---|---|
| 4.1 | the table: only the coloured VVV takes a convention sign; charged VVV `−1` in the kernel; contacts and VVS none | code; T1; M1, M4 | holds |
| 4.2 | "gluon contact (any structure): agrees with no vertex sign" | rows reaching gluon contacts: `gg_to_ggg`, `uux_to_ggg`, `gg_to_gg_cg` (four-gluon, `O_G`); SMEFTsim's five-gluon `O_G` contact is in no row | **imprecise**: the five-gluon contact is unmeasured |
| 4.3 | ALOHA reads every vertex `COUP·L` and closes with `−i`; the vector propagator's phase is the evaluator's | 1.2, 1.3 | holds |
| 4.4 | `VVV1`–`VVV7` and `VVV9` on `a W- W+` and `W- W+ Z` odd under the slot exchange, `VVV3` vanishing | re-measured: a scratch numeric check (random momenta, momentum conserved, slots and momenta exchanged; sanity-checked on a known even, odd and mixed structure) over the SM and SMEFTsim `lorentz.py`/`vertices.py`: all odd, `VVV3` zero; also odd on SMEFTsim's `W1` pairs (`VVV5`) | holds |
| 4.5 | "the SM and SMEFTsim carry `g g g`, `a W- W+` and `W- W+ Z`" | vertex census: SMEFTsim's UFO also has eleven triple-vector vertices with its auxiliary `W1`/`Z1` fields (custom propagators, refused on an internal line); each has a charged pair, so none is a colourless vertex of three self-conjugate vectors | **imprecise**; the conclusion stands |
| 4.6 | the colour, slot-order and coupling evidence for the triple-gluon `−1` | T1's numbers; the committed `triple_gluon` tests (run) reproduce the `g g > t t~` and `e+ e- > w+ w-` ones to the digit | holds (`u u~ > g g` ALOHA trace: record) |
| 4.7 | `|M|² ≈ 105` at `√s = 500` for same-helicity `u u~ > g g` with the triple-gluon vertex at the colourless VVV's sign | probe P0/M1 (Mutations): with `triple_gluon_sign` at `+1`, the helicities with equal gluon helicities read `9.604e1` at `cos θ = 0.3`, against `0` (to `2.6e-31`) unmutated | holds in substance; the value is point-dependent |
| 4.8 | `is_yang_mills_vvv` tests the arity; `a_momentum_bearing_vector_contact_is_not_a_triple_vector_vertex` | code; test run | holds |
| 4.9 | `V₃ + 2V₄ = n − 2` on a tree of three- and four-gluon vertices | counting (`3V₃ + 4V₄ = n + 2I`, `I = V − 1`) | holds; with `O_G`'s five-gluon contact a pure-gluon process can see the sign (1.9) |
| 4.10 | `gen_standalone_jamps.py` dumps "every JAMP and every AMP" | `gen_standalone_jamps.py:93` (`bank_amps`, single-flow rows only) | **imprecise** |
| 4.11 | `zero_widths` for spacelike widths | `gen_standalone_jamps.py:92, 133-146` | holds |
| 4.12 | gate table: `standalone_jamps` "at 1e-12 (1e-7 on the rows carrying `GC_303`/`GC_299`)" | `standalone_jamps.rs:26-35, 63-77`: the seven-digit list is gone; the named graphs of `ROUNDED_COUPLING_GRAPHS` are divided by their registered deviation, everything else at `1e-12`, each graph at `GRAPH_RATIO_TOL = 1e-10` | **wrong** (stale) |
| 4.13 | gate table's vector-vertex rows: "the W-pair rows (`…`, `wph_to_tbx`, `hwp_to_tbx`, `zz_to_ttx`, `hh_to_ttx`, …)" | `hh_to_ttx` has no vector vertex; `zz_to_ttx`, `wph_to_tbx`, `hwp_to_tbx` have no W pair (their vector vertices are VVS and FFV). They are W1's rows | **imprecise** |
| 4.14 | "kinematics outside three points" | point counts of the 45 standalone tables: 42 with 3, 2 with 4, 1 with 8 | **imprecise** |
| 4.15 | Parke–Taylor test description | `gluon_parke_taylor.rs` doc and `REL_TOL = 1e-10`; run | holds |
| 4.16 | the drop-the-triple-gluon and M4 failure lists | M1, M4 | see Mutations |
| 4.17 | "Moving it into the colour or into the coupling keeps every SM row of both suites green" | M2, M3 | see Mutations |
| 4.18 | the four `triple_gluon` tests and their blind spots | `triple_gluon.rs`; runs; M1 | holds |
| 4.19 | sources: `code-rootdiag`, `code-standalone`, `code-pt` at `5d4a346`; `code-tg` at `9549fcb` | as 1.17 (`9549fcb` is PR #39's branch commit) | **wrong** (path) |

### `validation/convention-channel-coverage`

| # | claim | how checked | verdict |
|---|---|---|---|
| 5.1 | the `fermi_sign` block | 1.1 | holds |
| 5.2 | `named_processes_exercise_every_convention_factor`: the five rows | `root_diagram.rs:1302-1356`; test run | holds |
| 5.3 | "Each part, set to `+1` on its own, fails the test" | MS, MC, MV-rev, MT, M1 (Mutations) | see Mutations |
| 5.4 | the dropped-factor table, and "Every failure listed is a per-diagram or per-flow disagreement" | as 1.5, 1.6 | see Mutations; **wrong** as 1.6 |
| 5.5 | the relabelling test passes under every factor mutation | the relabelling test ran under each (Mutations) | see Mutations |
| 5.6 | the fermion-line arms (24/9/2, the `g g > t t~` control); Bhabha and `u u~ > u u~` | test; note 29 §F.10 | holds / record |
| 5.7 | the ℓℓj rows do not pin the line sign; seven `|M|²` and two per-diagram rows; `gux_to_epemux` alone caught a `g u~` slot rule | note 24 §P1 (archived, mutation experiments 3 and 4) | record |
| 5.8 | the invariance gates table: relabelling "at least two diagrams compared per pair" | `relabelling.rs:301-302` | holds |
| 5.9 | renumbering "per-helicity per-flow amplitudes to 1e-10" | `renumbering.rs:194-197, 337`: on subprocesses of at most 40 diagrams | **imprecise** (the cap is unstated here) |
| 5.10 | rooting soundness: 40-diagram cap, the two 579/615-diagram processes whole-process, `REL_TOL = 1e-10`, floor 2.2e-11 | `rooting_soundness.rs:46-66, 287-327`; table `n_graphs` | holds |
| 5.11 | the live-parity cancellation dropped: `literal_sigma_currents_are_rooting_invariant` fails, 3 of 270 sweep re-rootings fail, all on `bbx`; relabelling and renumbering pass | ML (Mutations) | see Mutations |
| 5.12 | "`momentum_slashed_chain_is_rooting_invariant`, `four_fermion_currents_are_rooting_invariant`, `tensor_four_fermion_currents_are_rooting_invariant` and `literal_sigma_currents_are_rooting_invariant` run in the default suite" | `rooting_soundness.rs:405, 509, 664`: the first three are `#[cfg(feature = "extended-validation")]`; the default run lists two tests | **wrong** |
| 5.13 | "Moved into the colour … every row of `standalone_jamps` (49) and `amplitude_oracle` (49) passes" | M2; 49 is the test count of each (45 table rows and 4 checks; 48 rows and a coverage test) | see Mutations; **imprecise** count wording |
| 5.14 | `amplitude_oracle` compares `gg_to_ttx` per flow although its table banks `AMP()` | oracle output (`per-diagram 0.00e0 (not banked)`); table `amp_table: true`, no `jamp_coefficients` | holds |
| 5.15 | sources at `5d4a346` | as 1.17 | **wrong** (path) |

### `amplitudes/charge-flow-phase-study`

| # | claim | how checked | verdict |
|---|---|---|---|
| 6.1 | the study's question, bar, rule, results table, `n − k = 0`, corrections | archived note 29 §F.1–F.15 | record (dated in the text) |
| 6.2 | "the per-propagator −1 is not a repackaged propagator −i: the propagator's −i is applied in the kernel and is present separately" | `fermion-line-sign` (2.5): against MadGraph the `−1` is the ratio of the kernel's `−i` to ALOHA's `+i` | **imprecise**: read beside the current account it contradicts it; both are true (the `−1` is not the kernel's `−i` itself, it is the ratio of the two phases) |
| 6.3 | the W1 paragraph: a gluon contact takes no sign, an exchange diagram `−1` at each of two triple-gluon vertices, so the same net sign; no colourless-VVV, contact, pure-metric-VVS or scalar-sink sign exists; no root or anchor read | code (`convention_sign` and its three factors; `grep` for the removed names returns only `history/` and sprint files) | holds |
| 6.4 | `amplitude_oracle` asserts `|Im(k/G)| < LINEAR_REL_TOL` and banks `CONFIG_AMP_SIGNS` | `amplitude_oracle.rs:1903-1925, 614` | holds |

### `amplitudes/evaluator-architecture`

| # | claim | how checked | verdict |
|---|---|---|---|
| 7.1 | the three stages and their entry points | `compile.rs:127`, `run.rs:241, 358` | holds |
| 7.2 | compile order: 1 root each diagram, 2 colourize, 3 lower, 4 fold | `AmplitudeEvaluator::compile`: `colorize_process` runs first, then `root_amplitudes` roots each `(diagram, colour chain)` the basis names (the chain selects each vertex's colour structure), then `lower_flows`, `lower::optimize`, `Folded::build` | **wrong** order |
| 7.3 | the per-flow JAMP formula, the single-flow special case, sharing | `lower.rs` doc and `lower_flows` | holds |
| 7.4 | the helicity-expanded program is built lazily "on first use of `eval_m2`" | `folded_hel()` readers: `eval_m2`, `eval_jamp2`, `eval_amp2`, `eval_hel_m2`, `eval_hel_jamps` and others in `run.rs` | **imprecise** |
| 7.5 | the rooting example and `canonical_root` | note 10 §5.4; `test_walk` output; code | holds (the example roots at the muon vertex for illustration; production roots that diagram at vertex 0) |
| 7.6 | the convention paragraph (B-D's) | 1.1, 3.2 | holds |
| 7.7 | bind and the read-outs | `run.rs:241-255, 340-360, 416, 479` | holds |
| 7.8 | `code-*` sources at `787070e` with line ranges | not in this clone | unchecked (the named items exist) |

### `amplitudes/fermion-flow-and-crossing`

| # | claim | how checked | verdict |
|---|---|---|---|
| 8.1 | adjoint-typed slots; `ExtLegInfo::adjoint` (`incoming == is_particle`) | `diagram_eval.rs:38-55` | holds |
| 8.2 | `resolve_bra_ket` and `reversed` | `kernel.rs:553-571` | holds |
| 8.3 | the off-shell current table (momentum routing) | `kernel.rs` docs of `gamma_iout`, `gamma_oout`, `off_shell_fermion_current`, `propagate_core` | holds |
| 8.4 | the crossed bit, `LegAdjoint { adjoint, crossed }`, mixed lines | `root_lorentz.rs:40-57`, `external_leg` | holds |
| 8.5 | the `CΓᵀC⁻¹` table | algebra (each row); `chiral_correction`, `sigma_chained`, `add_cut_line` | holds |
| 8.6 | "`collect_fermion_pairs` tags each open fermion end with its vertex slot and closes them by the vertex's own pairing" | no such function (`git log -S` shows it removed in `3652d48`); the pairs are `closed_fermion_pairs(group, out_slot)` (`root_diagram.rs:688`) | **wrong** (stale name) |
| 8.7 | "every gated four-fermion diagram is a single contact" | `uux_to_ttxg_4f` (a contact producing the current a gluon emission closes) and `tata_to_ttxa_tensor4f` (the tensor contact on an internal line) are gated per flow and per graph | **wrong** (stale) |
| 8.8 | `code-rootlorentz` title names `standalone_projector_crossed` and `pair_crossed`; `code-mixed` names `collect_fermion_pairs` | none of the three exists | **wrong** (stale titles at `787070e`) |

### `amplitudes/four-fermion-vertices`

| # | claim | how checked | verdict |
|---|---|---|---|
| 9.1 | MadGraph's `get_sign_flow` excerpt, its `expected = {1: 2, 2: 3}` for four fermions, the parity loop at `import_ufo.py:1893-1918`; `get_fermion_flow` at `aloha_fct.py:26` | submodule (3.7.1) `import_ufo.py:1878-1918`, `aloha_fct.py:26` | holds |
| 9.2 | one feyngraph vertex per pairing; `fermion_flow`, `flow_groups`, `build_feyngraph_model`, `Vertex::flow_group`, `VertexInfo::from_ufo` | `topo.rs:67, 206, 360`; `diagram_eval.rs` | holds |
| 9.3 | `ee_to_mumu_4f`: `V_729`'s structures, MadGraph's `−GC_54`, `−GC_29` on the `(1,4)(2,3)` calls | `matrix1_orig.f:431-435`, SMEFTsim `vertices.py` `V_729` | holds |
| 9.4 | census: 21 structures, 15/6; 80 of 1985 interactions mix pairings, "every one same-flavour (`X̄ X X̄ X`)" | temporary lib test (P0) over `ParsedModel::parse(SMEFTsim)`: `interactions 1985 … four-fermion structures 21 mixed 80 same-flavour 60`; the other 20 are `t̄ b b̄ t` (4), `ū d d̄ u` (8), `c̄ s s̄ c` (8) | **wrong**: 60 of the 80 are same-flavour |
| 9.5 | the `ee_to_ee_4f` counting convention, 22 diagrams against 19 graphs | `standalone_jamps.rs:96-141` (22 entries, 19 graphs); log 2026-10-15 approval; Q1 report | holds |
| 9.6 | the cyclic tensor path (`cyclic_tensor_term`, `FierzOut`/`FierzOutRev`, `SigmaOut`, `Multivector`) | `root_lorentz.rs` | holds |
| 9.7 | blind spot: "SMEFTsim writes the tensor operator with its aligned structures at exactly `−2×` the reversed ones (`c₁₀₅₂ = −2 c₁₀₄₉` under `vg_cleQt3`)" | SMEFTsim `V_747` (`ta+ ta- t~ t`): `GC_1053` on the aligned `FFFF8`/`FFFF21` is `−2×` `GC_1050` on the reversed `FFFF5`/`FFFF6`/`FFFF19`/`FFFF20`. `GC_1049` is `cleQt3Im·ytau/(2Λ²)`, zero under `vg_cleQt3` (`cleQt3Im = 0`) | the ratio holds; the coupling labels are **wrong** |
| 9.8 | the tensor-path line sign: `RootedTerm::build_sign`, `−1` per crossed line, `+1` tree-rooted, read in `vertex_root_signs` | code | holds |
| 9.9 | its adjudication table (`ta+ ta- > t t~ a` 0.25000 ± 3e-6; toy `lt~ lt > qt qt~ vt`; `e+ e- > mu+ mu- a` under `vg_c4l`) as the evidence for that sign | note 38 §S1 (`2d99872`): those numbers adjudicated a different rule, whether a line closed at a fermion-output four-fermion current takes the *line* sign, under the anchor convention | **imprecise** attribution; the current pins are MT's rows |
| 9.10 | `uux_to_ttxg_4f` agrees per flow | standalone run (1.001e-15) | holds |
| 9.11 | `code-current-line` at `5d4a346` | as 1.17 | **wrong** (path) |

### `performance/diagram-rooting`

| # | claim | how checked | verdict |
|---|---|---|---|
| 10.1 | `canonical_root` (code excerpt), `choose_root`'s override in test builds | `root_diagram.rs:783-835` | holds |
| 10.2 | "`all_rootings_preserve_amplitude` re-roots every diagram at every vertex across the MG-validated processes" | `rooting_soundness.rs:287-327`: per diagram only for processes of at most 40 diagrams; the two 2→6 processes are re-rooted whole-process | **imprecise** |
| 10.3 | `REL_TOL = 1e-10`, 2.2e-11 worst | code doc | holds |
| 10.4 | the headroom table, the per-process extremes, the floor | archived `rooting-study-results.md:224-266` and its per-process tables | record (and the concept says the command no longer reproduces) |
| 10.5 | "Feyngraph's `VtxIdx(0)` is already the vertex adjacent to the lowest-index external leg, so 'lowest-leg anchor' reproduces it on every process" | note 38 §S1: true on 2475 of 2517 census diagrams, false on 42 (`g g > g g g` 6, `g g > t t~ g` 1, `W+ W- > W+ W- Z NP<=1` 35), where leg 0 sits on a four-point contact | **imprecise**: true on the study's 14 processes, not in general |
| 10.6 | `ULP_BUDGET`; the amplitude gate at 1e-12 | `amplitude_oracle.rs:174-178` | holds |
| 10.7 | the "Why it is sound" paragraph (B-D's) | 1.1, 3.2 | holds |
| 10.8 | `eval_strategies` figures | bench record (M3 Max) | record |
| 10.9 | `root-diagram`, `rooting-soundness` sources at `787070e` | not in this clone | unchecked (items exist) |
