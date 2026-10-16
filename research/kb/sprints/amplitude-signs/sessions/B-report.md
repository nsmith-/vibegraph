---
type: Session Report
title: "B′ report: the convention sign is a product of local constants, read off the graph"
description: "The class-b role signs read at Diagram::anchor are replaced by one root-free convention sign: −1 per internal scalar line and per fermion line of a crossed line (the propagator phase against ALOHA's), each vertex's reversed-bilinear parity as the amplitude root, and −1 per triple-gluon vertex. It reproduces the old convention on all 506 census subprocesses where W1's and F1's rules do not fire and their measured fixes where they do; the relabelling test passes on all 12 pairs; all 46 standalone rows (27 new) agree with MadGraph. The anchor is gone. The fitted G of 19 amplitude rows changes sign and nothing else they print moves."
status: draft
tags: [amplitude, sign-convention, rooting, relabelling, fermion-flow, smeftsim, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: relabel, resource: "https://github.com/nsmith-/vibegraph/commit/6f2df1e", title: "test(amplitudes): relabelling invariance of per-diagram amplitudes, informational"}
  - {id: fix, resource: "https://github.com/nsmith-/vibegraph/commit/f1bb5bb", title: "fix(amplitudes): a convention sign read off the graph, not off the anchor rooting"}
---
Item: `wpwm-to-epem-neutrino-exchange-sign`. Built on `d6f2d41`. A derived
convention passed every acceptance check, so it landed; nothing is offered
under the stop rule.

## Correction to the brief

The brief asks for role signs whose product is the same at every root. The
data say more: no role is needed at all.

- **The kernels are honest.** No kernel applies a build sign; the build and
  reversed signs only ever entered `fermi_sign`. The live-rooted kernel value is
  rooting-invariant, which is what `rooting_soundness` measures (the anchor
  reading was constant across its re-rootings).
- **MadGraph's `AMP()` is rooting-invariant too.** ALOHA evaluates every vertex
  as `COUP·L`, closes the amplitude with `−i` and gives each propagator a fixed
  phase (`aloha/create_aloha.py`: `+i` for spin 0 and ½, `−i(g − qq/m²)` for
  spin 1). W1 saw the consequence: `AMP()` does not change under relabelling.
- So the ratio of the two is a product of one constant per line and one per
  vertex, with no root in it. A δ-consistent set of role signs is the same thing
  written differently; the anchor rule just read a non-δ-consistent set at one
  labelled vertex.

Three points of the brief change with that:

- **C2's two arms are subsumed, not kept.** The `Identity` crossed-line −1 and
  the all-scalar −1, like every other tree-path build sign (pure-metric VVS,
  scalar-sink, crossed pair, standalone projector, contact −1), are removed.
  Their rows stay green: C's 8 standalone rows agree to ≤1.8e-14 (below). The
  scalar-sink −1s were the scalar propagator's phase seen from its ends.
- **F1's arity rule is not a rule.** With no role reading, every colourless
  vector vertex (triple, four- and five-vector, O_W's included) carries no sign.
  The kernel's contact −1 and its anchor cancellation were the artefact.
- **"Unchanged digit for digit" cannot hold for the sign of the fitted `G`.** A
  convention without a labelled anchor changes the sign shared by every diagram
  of some processes. On 19 amplitude rows and 10 pre-existing standalone rows the
  printed `G` changes sign and no other printed number moves (diffs below).
  `CONFIG_AMP_SIGNS`, which banks the sign of each configuration amplitude
  against its diagram, flips as a whole on 11 rows and is re-banked. Nothing
  downstream reads either: `|M|²`, `eval_amp2` and every σ are blind to it.

## The derivation

`fermi_sign = Diagram::sign × convention_sign × (live reversed parity)`. The last
factor cancels the parity the runtime `resolve_bra_ket` applies on the live
tree. `convention_sign` (`helas/eval/root_diagram.rs`) is a product of three
factors:

| factor | value | from |
|---|---|---|
| `propagator_phase_sign` | −1 per internal scalar line, and per internal fermion line of a crossed line | ALOHA: fermion `+i(q̸+m)/D`, scalar `+i/D`, vector `−i(g−qq/m²)/D`. Here (`propagate_core`): `−i` for all three. A line with an initial-state end takes its fermion phases as `Diagram::fermion_line_sign`'s `(−1)^P` |
| `vertex_root_signs` | each vertex's reversed-bilinear parity, read with the vertex as the amplitude root | `ψ̄₁Γψ₂ = ψ̄₂(CΓᵀC⁻¹)ψ₁`, with `CΓᵀC⁻¹ = −Γ` for `γ^μ` and `σ^{μν}` and `+Γ` for `1`, `γ⁵`, `P_χ`. A cyclic four-fermion term on the tensor path takes the sign that path gives its amplitude root (−1 per crossed line) |
| `triple_gluon_sign` | −1 per triple-gluon vertex | the one constant the algebra above does not fix (see blind spots) |

With `Diagram::sign` (pairing parity, `(−1)^P` on uncrossed lines, −1 per crossed
line), every fermion propagator takes −1 and every crossed line −1. Each factor
reads only the graph, so the product is the same at every rooting and every
labelling. The `Diagram::anchor` concept is removed. The renumbering test now
forces leg 0's vertex off index 0.

### How it was found (instruments in the scratchpad, not committed)

1. **δ census** (`BP-probe.rs`, `bp_delta_census`). For each internal line `e`
   between `A` and `B`, the old role ratio `δ(v,e) = b(v, out e)·b(v, sink)` must be
   equal at both ends for the product to be rooting-invariant. 270 diagrams. The
   mismatched classes:
   - a colourless vector between a two-fermion vertex (δ = +1) and any boson
     vertex (VVV, VVS, contact: δ = −1): W1's case;
   - a fermion line of a crossed line between FFS (δ = −1) and FFV (δ = +1):
     W1's `w+ h > t b~`;
   - an internal Higgs between `ggH` (δ = −1) and FFS (δ = +1);
   - a colourless momentum-bearing VVS (δ(V) = +1 against −1): F1's VVS4;
   - a vector four-fermion current beside a gluon emission (`u u~ > t t~ g`, c4q).

   Everything else is consistent.
2. **GF(2) solve over role features** with root-invariance equations and the
   MadGraph-verified targets. The targets are old × W1-B × F1 rules, on 3495
   diagrams of every amplitude-oracle row, every standalone table and W1's 40
   probes. Result: consistent, 9 free variables.
3. **Role-free solve**: target = Π κ(vertex class) × Π g(line class), up to one
   sign per process. Consistent once the vertex class carries its sink reversal
   parity. With the derived line constants fixed (scalar −1, crossed fermion
   −1) and the vertex sign set to its reversal parity, **the unique minimal
   solution is one constant: −1 on the triple-gluon vertex.** The next smallest
   is weight 9 (colourless VVV, five-vector and every FFV −1). It is the same
   convention up to the per-process identity Π_v(−1)^{#gluon legs} =
   (−1)^{#external gluons}.
4. **Production comparison per (diagram, colour chain)** (`bp_compare_chains`).
   Old against new × W1-B × F1, on the renumbering census plus the verified set:
   **506 subprocesses, 7347 signs, 0 subprocesses with a relative difference.**
   A first pass with chain 0 throughout flagged 4 diagrams of
   `u u~ > t t~ g NP<=1` (c4q). Under the real colour chains they agree, so that
   flag was the instrument's. The new `uux_to_ttxg_4f` table confirms it: both
   conventions agree with MadGraph there (1.0e-15).

## Relabelling test

`helas::eval::relabelling::relabelling_preserves_per_diagram_amplitudes`
(hermetic). It pairs the two labellings' diagrams by a label-free graph
signature, compares per helicity at a generic point, and requires one sign per
process. Twelve pairs. Committed informational first (`6f2df1e`, two-way list).

Before (`6f2df1e`, `cargo test -p vibegraph-lib --lib relabelling_preserves -- --nocapture`):

```
w+ e- > z ve / e- w+ > ve z: 3 diagrams compared, flipped against the rest: [1]
w+ h > t b~ / h w+ > t b~: 3 diagrams compared, flipped against the rest: [0]
z h > t t~ / h z > t t~: 3 diagrams compared, flipped against the rest: [0]
w+ w- > w+ w- z NP<=1 / w- w+ > w- w+ z NP<=1: 222 diagrams compared, flipped against the rest: [142, 143, 144, 148, 149, 150]
a z > w+ w- NP<=1 / z a > w- w+ NP<=1: 14 diagrams compared, flipped against the rest: []
w+ w- > e+ e- / w- w+ > e- e+: 3 diagrams compared, flipped against the rest: []
e+ e- > w+ w- / e- e+ > w- w+: 3 diagrams compared, flipped against the rest: []
w+ a > e+ ve a / a w+ > ve e+ a: 7 diagrams compared, flipped against the rest: [6]
e+ e- > ta+ ta- h / e- e+ > ta- ta+ h: 5 diagrams compared, flipped against the rest: []
u u~ > z g / u~ u > g z: 2 diagrams compared, flipped against the rest: []
h h > t t~ / h h > t~ t: 3 diagrams compared, flipped against the rest: []
st lt > st lt NP<=2 NPD<=1 / lt st > lt st NP<=2 NPD<=1: 4 diagrams compared, flipped against the rest: []
```

It fails on exactly W1's diagrams: the W exchange of `w+ e- > z ve`, and the
s-channel boson against the two quark exchanges of `w+ h`/`z h`. The listed set
is the minority. It also fails on six of F1's `V(a,Z,H)` graphs, and on a
diagram of `w+ a > e+ ve a` that no earlier session had relabelled.

After: every pair `flipped against the rest: []`, `test result: ok. 1 passed`,
with `KNOWN_RELABELLING_DEPENDENT` empty.

## Acceptance checks

Every cargo call ran with `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the committed tree.

**`standalone_jamps`** (`cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture --test-threads=1`):
`test result: ok. 46 passed; 0 failed`, with `KNOWN_DISAGREEMENT` empty. Rows:

- **W1's 11, and `wpwm_to_epem`**, worst flow:

  | row | worst flow |
  |---|---|
  | `wpwm_to_epem` | 9.423e-15 |
  | `wmwp_to_emep` | 1.409e-15 |
  | `wpwm_to_uux` | 1.860e-15 |
  | `wpem_to_zve` | 3.147e-15 |
  | `aem_to_wmve` | 8.997e-16 |
  | `wph_to_tbx` | 1.025e-14 |
  | `hwp_to_tbx` | 2.670e-14 |
  | `zz_to_ttx` | 9.044e-15 |
  | `hh_to_ttx` | 1.193e-15 |
  | `gg_to_uux` | 7.555e-15 |
  | `wpwm_to_epemz` | 4.557e-14 |
  | `wpwm_to_uuxg` | 3.336e-14 |

- **F1's 8**:
  - `wpwm_to_wpwm_cw` 3.421e-13 and `wpwm_to_zz_cw` 9.186e-14, at `REL_TOL` 1e-12.
  - The six that carry `GC_303`/`GC_299` gate at 1e-7, a new
    `SEVEN_DIGIT_COUPLING_ROWS` list: the coupling's own 1.2e-8 and 2.8e-8, with
    a factor of three to spare. Measured worst flow: `wpwm_to_az_cw` 8.458e-11,
    `az_to_wpwm_cw` 5.359e-10, `aa_to_wpwm_cw` 3.342e-10, `wpwm_to_wpwmz_cw`
    1.971e-11, `wpwm_to_wpwma_cw` 3.378e-10, `ee_to_wpwmwpwm_cw` 6.377e-10.
    Before the fix they read 1.413e-2, 8.954e-2, 2.413e-2, 2.395e-1, 8.162e-1 and
    3.679e-1.
- **C's 8**, worst graph:

  | row | worst graph |
  |---|---|
  | `ee_to_ttxh` | 4.048e-15 |
  | `ee_to_ttxh_identity` | 4.119e-15 |
  | `ll_to_qqxs_toy_yukawa` | 4.066e-16 |
  | `ll_to_ss_toy_dscalar` | 2.318e-15 |
  | `sl_to_sl_toy_dscalar` | 4.257e-16 |
  | `ttx_to_hh_chbox` | 1.596e-14 |
  | `ht_to_ht_chbox` | 3.423e-15 |
  | `hh_to_hh_chbox` | 1.810e-15 |

  These are C2's numbers to the digit.
- **Two new MadGraph rows** (generated here through `gen_standalone_jamps.py` →
  `mg5_pinned.sh`, both logs `VERSION 3.7.1`):
  - `uux_to_ttxg_4f`, `u u~ > t t~ g NP<=1` under `vg_c4q`, 4 flows: worst flow
    1.001e-15. A vector four-quark contact produces the quark current a gluon
    emission closes.
  - `tata_to_ttxa_tensor4f`, `ta+ ta- > t t~ a NP<=1 SMHLOOP=0` under
    `vg_cleQt3`, with `AMP()` banked: worst graph 5.105e-15 over 1536 entries. It
    puts the tensor contact on an internal line, and gives the pairing
    `TATA_TO_TTXA_TENSOR4F_GRAPHS`, unique at unit modulus. Until now the tensor
    contact on an internal line was checked against MadGraph only through
    `|M|²`, in an uncommitted measurement.

**Pre-existing standalone rows, digit for digit.** I ran the same suite under
the old convention (a temporary switch, removed) and sorted the printed lines.
With the sign of `G`'s components masked, the diff lists only the rows that
failed before. Every one of the 20 pre-existing rows prints identical numbers.
`G` changes sign on 10 of them: `bbx_to_hh`, `gg_to_ggg`, `gux_to_ttxux`,
`hh_to_hh_chbox`, `ht_to_ht_chbox`, `ll_to_ss_toy_dscalar`,
`sl_to_sl_toy_dscalar`, `ttx_to_hh_chbox`, `ug_to_ttxu` and `uux_to_ttxg`.

**`amplitude_oracle`** (`cargo test -p vibegraph-lib --test amplitude_oracle -- --nocapture --test-threads=1`):
`test result: ok. 49 passed; 0 failed`. Sorted lines, ANSI and `finished in`
stripped, old (the switch) against new, with `(G = ±1i` normalised:

```
$ diff BP-oracle-old2.norm BP-oracle-final.norm
7c7
<   [wpwm_to_wpwmz_cw] ... |M|² max_rel grid 2.79e1 / event 1.05e1, per-diagram 0.00e0 (not banked), per-flow 7.48e-1, JAMP2 2.55e1 (G = ±1i, |G|-1 = 1.7e-2, ...
---
>   [wpwm_to_wpwmz_cw] ... |M|² max_rel grid 1.14e-11 / event 2.11e-11, per-diagram 0.00e0 (not banked), per-flow 2.53e-11, JAMP2 1.63e-11 (G = ±1i, |G|-1 = 6.2e-12, ...
```

That row is informational, and the change is F1's prediction to the printed
digit. Without the normalisation, `G` flips sign on 19 rows:

- `bbx_to_h_identity`, `ddx_to_epemg`, `ee_to_mumu_tata_qcd0`, `ee_to_mumua`;
- `ee_to_wp0wm`, `ee_to_wp0wmt`, `ee_to_wpwm`, `ee_to_wpwm_cw`;
- `gg_to_ttx`, `gg_to_ttx_smlimit`, `gg_to_ttx_smlimit_qcd2`, `gu_to_epemu`,
  `gux_to_epemux`;
- `p3r3_to_p3r3_toy_epsilon`, `p3r3_to_p3r3_toy_sextet`, `qqx_to_o8o8_toy_dcolor`;
- `ud_to_epemud_qcd0`, `uux_to_epemg`, `uux_to_ztg`.

`CONFIG_AMP_SIGNS` was re-banked on the 11 rows where it measured a whole-pattern
flip, for example `ee_to_wpwm` `---` → `+++` and `ee_to_mumua` `----++++` →
`++++----`.

**`rooting_soundness`** (`RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib --lib helas::eval::rooting_soundness::all_rootings_preserve_amplitude -- --ignored --nocapture --test-threads=1`):
`rooting-soundness sweep: 270 re-rootings, 0 failures`, `test result: ok`. The
non-ignored tests ran in the workspace suite.

**`renumbering`** (`cargo test -p vibegraph-lib --lib helas::eval::renumbering -- --nocapture`):

```
3730 diagrams, 11681 propagators (4750 not self-conjugate), 0 failures
3730 diagrams, 10454 (renumbering, chain) signs, 4038 amplitude comparisons, 6378 renumberings with leg 0's vertex off index 0, 0 failures
test result: ok. 2 passed; 0 failed
```

**Lints and the workspace:**

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0.
- `cargo test --workspace --no-fail-fast`: exit 0, 38 suites, 1431 passed, 0
  failed, 16 ignored (summed `test result` lines). That is C2's 1406, plus 27 new
  standalone tests, the relabelling test and the factor-coverage test, less four
  removed tests:
  - `yang_mills_vvv_sign_fires_only_for_source_vvv`;
  - `mg_guard_processes_exercise_every_convention_channel`;
  - `four_vector_contact_sign_is_uniform_over_its_structures`;
  - `the_anchor_is_the_first_lowest_arity_vertex`.

  Their subjects no longer exist. `named_processes_exercise_every_convention_factor`
  is the new non-vacuity check, and the momentum-contact predicate test now runs
  on `vg_cG`.

**`pixi run --skip-deps validate`**: `VALIDATE EXIT 0`, 63 suites, 1816 passed, 0
failed, 84 ignored. Cell line: `64 rows × 4 categories = 256 cells: 203 measured
in the layers this run drove (196 ✅, 7 ⚠️, 13 ⏳, 40 — / uncovered)`, and `the
measured cells are the declared cells`. These are C2's counts. The first attempt
was stopped by the 30-minute background limit while still compiling, and was
rerun to completion.

## Mutations (each factor pinned by MadGraph)

Each factor of `convention_sign` was dropped behind a temporary switch (since
removed), one at a time. The table lists the suites each mutation fails. Every
oracle failure listed is a per-diagram or per-flow disagreement with MadGraph,
not only a re-banked sign pattern.

| dropped | `standalone_jamps` fails | `amplitude_oracle` fails (physics) |
|---|---|---|
| every vertex sign | 17 rows (`ee_to_ee_4f`, `uux_to_ggg`, `tata_to_ttxh`, …) | 28 rows, e.g. `ee_to_ee`, `ee_to_wpwm`, `ud_to_epemud_qcd0`, `uux_to_uux` |
| tensor-path sign only | `tata_to_ttxa_tensor4f` | `ll_to_qqx_toy_tensor`, `tata_to_ttx_tensor4f` |
| scalar-line phase | 25 rows (`bbx_to_hh`, `hh_to_hh_chbox`, `zz_to_ttx`, the O_W rows, …) | `bbx/uux_to_ccx_emmm_qcd0`, `gg_to_gg_cg`, `qqx_to_o8o8_toy_dcolor`, … |
| crossed-line fermion phase | 22 rows (`ee_to_ttxh`, `wpwm_to_epem`, `wph_to_tbx`, …) | `gg_to_ttx`, `ee_to_tatah`, `ud_to_epemud_qcd0`, … |
| triple-gluon sign | `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`, `ttx_to_gg_chg`, the five `t t~ + jet` rows, `uux_to_ttxg_4f` | `gg_to_ttx`, `gg_to_ttx_smlimit`, `gg_to_ttx_smlimit_qcd2` |

The relabelling test passes under every mutation. Each factor is a function of
the graph, so it cannot see one.

## Banked σ rows moved

None. The baseline is C2's after-run report (scratchpad `C2-report-after`, the
tree this branch builds on). I diffed every per-cell JSON under
`target/validation-report/{diagrams,amplitudes,integrals,samples}` with the
duration fields removed:

```
diagrams 56 cells; differ []
amplitudes 48 cells; differ ['ee_to_wpwm_cw.json', 'wpwm_to_wpwmz_cw.json']
integrals 52 cells; differ []
samples 53 cells; differ []
```

Both amplitude cells are informational:

- `wpwm_to_wpwmz_cw`: `max_rel_grid` 27.94 → 1.14e-11, `per_flow` 0.748 →
  2.53e-11, `jamp2` 25.5 → 1.63e-11.
- `ee_to_wpwm_cw`: only its note, which quotes the configuration sign pattern
  (`-------` → `+++++++`, a global flip).

Every σ, error, pull and sample statistic is identical, as it must be: `|M|²`
changes only on processes where W1's or F1's rule fires, and no banked σ row is
one of them.

## The hygiene findings H11 and H13 (C-H report)

- **H11: the mixed-sign refusal no longer depends on the root.** I derived this
  and did not run it. The probe vertex puts `Identity(2,1)` beside
  `P(-1,3)*Gamma(-1,2,1)` under one coupling order.
  - The build signs are gone, so only the reversed-bilinear parity can be refused,
    plus the tensor-path sign on cyclic terms.
  - `vertex_root_signs` now reads *every* vertex as the amplitude root, in every
    diagram, at every production rooting. Its refusal is therefore a function of
    the graph: which external leg the bilinear's row slot leads to.
  - The live tree's parity check reads the same input bindings at a scalar- or
    vector-output current as at the sink, and is `+1` at a fermion-output current.
    So it refuses only where the sink reading already does.
  - For this vertex the refusal is physics: under a reversed reading the `γ^μ`
    term changes sign (`Cγ^{μT}C⁻¹ = −γ^μ`) and the `1` term does not, so the
    vertex really has no single sign. Where no bilinear is reversed (both parities
    `+1`) it compiles, with the sign the convention gives every vertex.
  - Expected: `st > lt lt~` compiles because its line binds the row slot to a bra,
    not because of where it is rooted.
  - Falsifier: the C-H probe, rerun on this tree, refusing or compiling the same
    process differently under `set_root_override`. It was not run, because the
    banked layer was running on this tree.
- **H13: not done.** The four arms of `build_child` lost their crossed-line flips
  here, and what is left is the shared pick-the-wrapped-index step. The
  projector arms' chirality correction still differs from the `Identity` and
  `Gamma5` arms. Unifying them is a pure refactor with nothing left to derive, so I
  left it for the hygiene pass.

## Blind spots

- **The triple-gluon constant is calibrated, not derived.** Its Lorentz
  structure and coupling normalisation are the colourless VVV's. `f_to_traces`
  matches MadGraph's `f = −2i Tr(abc) + 2i Tr(cba)` term for term. So the sign
  sits in how the colour-ordered decomposition meets the vertex slots, which I
  did not trace. It is equivalent to −1 per quark–gluon vertex, up to a
  process-global sign.
- **The per-process global sign is free and is now different.** `|M|²`, every σ
  and the relabelling test are blind to it, and the fitted `G` absorbs it. No
  labelling-free rule reproduces the old one. The best external-leg parity
  predictor matches 445 of 463 subprocesses.
- **The tensor-path sign is the kernel's own.** It is pinned on (initial,
  crossed) line pairs only: the toy tensor row, `tata_to_ttx_tensor4f` and
  `tata_to_ttxa_tensor4f`. A cyclic structure on two mixed lines (a Fierzed
  Bhabha-like tensor contact) is unmeasured.
- **Scalar⊗scalar tree-rooted four-fermion contacts** (`O_ledq`-type) now take
  no vertex sign. The old scalar-sink signs would have given them `(−1)·(crossed)`
  per bilinear. No committed row reaches one, so neither reading is checked.
- **VSS vertices** (scalar–scalar–vector) appear in no model in reach. The
  convention predicts no vertex sign for them, and −1 for the scalar lines
  beside them.
- **The relabelling test sees only labelling dependence.** That covers anchor-
  or leg-keyed rules, and only where a relabelling moves the vertex in question.
  It covers single-flow pairs only, and is blind to a wrong function of the graph
  (the mutation table above).
- **`rooting_soundness` no longer tests a convention.** The convention reads no
  root, so the gate tests only the kernels' rooting invariance and the
  live-parity cancellation.
- **Points.** Three or four RAMBO points per standalone row at one √s, as
  before.

## Found

- **The kb concepts describe the anchor convention.** These are wrong now:
  - `amplitudes/rooting-invariance-and-anchor`;
  - `amplitudes/vector-vertex-signs`;
  - `amplitudes/fermion-line-sign` (its reversal account of `(−1)^P`);
  - `amplitudes/convention-sign-inventory`;
  - `validation/convention-channel-coverage`, including the test names it cites.

  To close: rewrite them around `convention_sign` and its three factors, from
  this report.
- **The triple-gluon −1 has no derivation.** Evidence: blind spots above. To
  close: follow the trace basis through `colorize` for one `u u~ > g g` diagram
  and write down where the `−1` against the quark–gluon vertex enters.
- **No row reaches a scalar⊗scalar tree-rooted four-fermion contact, a mixed-line
  tensor contact or a VSS vertex.** To close: one standalone row each, for
  example `e+ e- > b b~ NP<=1` under an `O_ledq` card, the tensor Bhabha
  analogue, and a toy charged-scalar row.
- **`wpwm_to_wpwmz_cw`'s amplitude cell is ready for F2.** It agrees at 1.14e-11
  (`GC_303`-limited). Its per-graph banking, pairing and manifest cell are F2's.
- **The probe instruments are worth committing.** The δ census, the
  per-(diagram, chain) old/new comparison and the table pairing by value
  (scratchpad `BP-probe.rs`) are each a few dozen lines. A chain-0 shortcut gave
  a false difference once; the committed version should use the colour basis's
  chains.
