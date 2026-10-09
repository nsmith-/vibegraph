---
type: Validation Gate
title: Which gate pins which convention-sign channel
description: "Guard processes asserted to exercise every sign branch, mutation experiments per row, and the all-rootings soundness check; which channels have no varying instance."
status: draft
tags: [amplitudes, signs, coverage, rooting, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n19-v5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/19-validation-pass-plan.md#L148-L710", title: "Note 19 V5 — the rooting-soundness gate"}
  - {id: n19-v6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/19-validation-pass-plan.md#L711-L750", title: "Note 19 V6 — branch-level coverage and the guard census"}
  - {id: n24-rows, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L665-L682", title: "Note 24 P1 — rows enforced and channel counts"}
  - {id: n24-gux, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L683-L703", title: "Note 24 P1 — the q̄ g row"}
  - {id: n24-mut, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L756-L809", title: "Note 24 P1 — four mutation experiments"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L834-L854", title: "Note 24 P1 — what the llj rows pin and do not"}
  - {id: n29-f1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L353-L437", title: "Note 29 §F.1 — the pinned-convention inventory"}
  - {id: n29-f10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L830-L976", title: "Note 29 §F.10 — which rows vary which channel"}
  - {id: n29-f13, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/29-v01-validation-sprint-plan.md#L1096-L1152", title: "Note 29 §F.13 — row 4b's attribution was wrong"}
  - {id: code-guard, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/root_diagram.rs#L1393-L1470", title: "channel_counts and mg_guard_processes_exercise_every_convention_channel"}
  - {id: code-fermi, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/root_diagram.rs#L1204-L1248", title: "compile_single_diagram — fermi_sign assembly"}
  - {id: code-rooting, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/src/helas/eval/rooting_soundness.rs", title: "helas/eval/rooting_soundness.rs"}
  - {id: n39-gates, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/39-vector-vertex-signs.md#L172-L208", title: "Note 39 §6 — vector-vertex sign mutations"}
---

# Which gate pins which convention-sign channel

A convention sign that no gated process varies is not pinned by any comparison
against MadGraph: a global sign is absorbed into the amplitude oracle's fitted
`G`, and `|M|²` cannot see it. The [`AGENTS.md`](../../../AGENTS.md) rule —
convention claims are hypotheses, pinned by a test that would fail if they were
false — is applied here in three ways: guard processes asserted to exercise
each channel, mutation experiments run against the whole suite, and the
all-rootings soundness check. The inventory of the signs themselves is
[convention signs](../amplitudes/convention-sign-inventory.md).

## The channels in `fermi_sign`

`compile_single_diagram` (`helas/eval/root_diagram.rs`) assembles each
diagram's sign as

```
fermi_sign = diagram.sign                         // Fermi permutation + fermion-line sign
           * fermion_current_line_sign(reference)
           * yang_mills_vvv_sign(diagram, model)
           * vector_contact_sign(diagram, model)
           * gluon_scalar_current_sign(diagram, model)
           * reference.build_convention_sign()
           * reference.reversed_convention_sign() * tree.reversed_convention_sign()
```

where `reference` is the tree rooted at `Diagram::anchor`. The fermion-line sign
is carried by `Diagram::sign` through `Diagram::fermion_line_sign`;
`spine_sign_from_flow` (the rooted-tree derivation) survives only as a
`debug_assert` cross-check and in
`fermion_line_sign_matches_the_rooted_tree_derivation`. See
[the fermion-line sign](../amplitudes/fermion-line-sign.md) and
[vector-vertex signs](../amplitudes/vector-vertex-signs.md).

## The guard test

`mg_guard_processes_exercise_every_convention_channel` (default suite) uses
`channel_counts`, which counts per process the diagrams firing each of four
channels on the anchor-rooted tree, and asserts each is `> 0` on a named
process[^n19-v6]:

| channel | guard process | why that one |
|---|---|---|
| Yang–Mills VVV source sign | `e+ e- > W+ W-` | the γ/Z → W⁺W⁻ vertex is a non-anchor source |
| fermion-line sign | `e+ e- > e+ e-` | Bhabha's s-channel has a crossed line |
| build −1, pure-metric arm | `g g > g g` | no fermion or scalar externals, so the four-gluon contact is the only build sign — the branch of the original VVVV phase bug |
| build −1, scalar-bilinear arm | `e+ e- > ta+ ta- H` | its build sign comes only from the τ Yukawa |
| reversed-bilinear parity | `e+ e- > mu+ mu-` | never fires on pure-gauge processes |

Production roots at `canonical_root` (the anchor); re-rooting no longer corrupts
amplitudes, because every rooting-dependent sign is read off the anchor-rooted
tree. The guard is non-vacuity only; per-channel properties live in
`yang_mills_vvv_sign_fires_only_for_source_vvv`,
`spine_sign_separates_mixed_line_and_crossed_line_propagators`,
`fermion_line_sign_matches_the_rooted_tree_derivation`, and the
`root_lorentz::tests::test_root_vvs_metric{,_scalar_out}` primitives. The VVS
pure-metric −1 with the *scalar* leg as output appears only in the 2 → 6 Higgs
classes; it is pinned at the primitive level by `test_root_vvs_metric_scalar_out`
and bit-for-bit by `u u~ / b b~ > c c~ e+ e- mu+ mu-` in the amplitude oracle.

**What the guard does not cover.** `channel_counts` predates the three
vector-vertex signs (`vector_contact_sign`, `gluon_scalar_current_sign`, and the
colourless-only restriction of the VVV sign). Those are pinned instead by
mutation against `standalone_jamps`, `gluon_parke_taylor` and
`amplitude_oracle`[^n39-gates]:

| mutation | fails |
|---|---|
| drop `vector_contact_sign` | `gg_to_ggg`, `uux_to_ggg`, `ee_to_wpwmz`; both Parke–Taylor tests; oracle `gg_to_gg`, `gg_to_gg_cg` |
| triple-gluon vertex keeps the source sign | `uux_to_ggg`, `ug_to_ug`, `ttx_to_gg_chg`; six-gluon Parke–Taylor; oracle `gg_to_gg`, `gg_to_gg_cg` |
| drop `gluon_scalar_current_sign` | `ttx_to_gg_chg`; oracle `gg_to_gg_cg` |
| colourless contact treated like a gluon one | `wpwm_to_wpwm` only |

The colourless contact at the anchor is pinned only by the standalone
`wpwm_to_wpwm` table; no enforced amplitude-oracle row reaches it.

## Which channels have a varying instance

Per-diagram, a channel is pinned against MadGraph only where it varies across
the diagrams of one process; a uniform sign is absorbed. Measured over the guard
processes and the hostile cases[^n29-f10]:

- Seven of nine processes carry a `fermi_sign` that is a pure global sign
  (`g g > g g`, `g g > t t~`, Bhabha, `e+ e- > mu+ mu-`, `e+ e- > W+ W-`,
  `u u~ > u u~`, `u u~ > d d~`). The observable content concentrates in
  `e+ e- > ta+ ta- h` (4 : 1) and `u d > e+ e- u d QCD=0` (18 : 17). Uniformity
  still refutes rules that predict variation: `g g > t t~`'s crossed top line
  carries 0, 1, 1 propagators with a uniform sign, which pins the crossed arm's
  propagator-independence.
- In Bhabha and `u u~ > u u~`, `diagram.sign` and the fermion-line sign cancel
  exactly, so those processes pin the product, not either factor.
- **The fermion-line sign's two arms** are both checked on
  `u d > e+ e- u d QCD=0`: the line sign is −1 on 11 diagrams (`{4…11, 18, 21,
  22}`, no mixed line carrying a propagator) and +1 on 24, matching
  `channel_counts`. (The neutral-current/charged-current 24/11 split there is a
  different partition, a Wick-pairing effect.)
- **The standalone-projector-crossed −1** is checked on `e+ e- > ta+ ta- H`: its
  build sign comes from the `ta ta H` `FFS4` vertex rooted at a fermion output,
  not from the scalar-sink arm.
- **The scalar-sink `ProjM/ProjP/Identity` −1 at a scalar or amplitude sink** had
  no varying instance anywhere in the banked set when this was measured, and the
  guard comment on its fourth assertion still attributes `e+ e- > ta+ ta- H`'s
  build sign to that arm, which does not fire there[^n29-f13]. The Standard-Model
  Yukawa and Higgs self-coupling rows added since (`tata_to_ttxh`,
  `tata_to_ttxhh`, `bbx_to_hh` in `standalone_jamps`, after the all-scalar vertex
  was given the scalar-sink sign) may vary it; that has not been re-measured.
- **The reversed-bilinear parity and the crossed-pair −1** (`pair_crossed`) have
  no varying instance in the banked set, so neither is checked per diagram. The
  parity is unchecked in principle at the `fermi_sign` level, because its live and
  anchor factors are equal on every production compile and their product is `+1`;
  the runtime reversed-FFV parity is exercised on coloured spines by
  `uux_to_epemg` and `ddx_to_epemg`[^n24-rows].
- **Row-by-row coverage is uneven.** The four ℓℓj partonic rows do **not** pin
  the fermion-line sign: it fires on all four diagrams of `g q → ℓℓq`, so dropping
  it is a global sign; that mutation failed 7 `|M|²` rows and two per-diagram rows,
  none of them ℓℓj[^n24-mut]. A slot rule wrong only for `g u~` was caught by
  `gux_to_epemux` alone, which had no other detector[^n24-gux].

Majorana lines (none in the SM UFO) and charge-flow phrasings that agree on every
tree diagram are untestable with the banked set; see
[the charge-flow phase study](../amplitudes/charge-flow-phase-study.md).

## The rooting-soundness check

`helas/eval/rooting_soundness.rs` (a module of the library, not an integration
test) re-roots diagrams through a test-only override hook and requires `|M|²` to
match the production rooting's value, which [the amplitude
oracle](amplitude-oracle.md) already pins against MadGraph[^n19-v5]. Per-diagram
root isolation runs for processes up to 40 diagrams, whole-process re-rooting
for the two 579/615-diagram 2 → 6 processes, at `REL_TOL = 1e-10`: re-rooting
reassociates momentum sums, and the measured floor is 2.2e-11
(`e+ e- > ta+ ta- H`), far below any O(1) sign error.

- `root_override_hook_is_transparent` runs in the default suite: an explicit
  override to the canonical root reproduces the default `|M|²`.
- `all_rootings_preserve_amplitude` is `#[ignore]` (oracle layer for cost):

  ```
  RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib \
      --lib helas::eval::rooting_soundness::all_rootings_preserve_amplitude \
      -- --ignored --nocapture --test-threads=1
  ```

  It passes with 0 failures; the re-rooting count grows with the validated
  process list (270 at the last recorded run).

*Blind to*: any error shared by every rooting, since its oracle is this crate's
own production value; and any sign whose live and anchor factors coincide.

[^n19-v5]: Note 19 V5. Its rooting is now the anchor rather than `VtxIdx(0)`, and the line sign is carried by the diagram.
[^n19-v6]: Note 19 V6.
[^n24-rows]: Note 24 P1, rows enforced.
[^n24-gux]: Note 24 P1, the `q̄ g` gap.
[^n24-mut]: Note 24 P1, mutation experiments 3 and 4.
[^n29-f10]: Note 29 §F.8 and §F.10, hostile cases H2–H5.
[^n29-f13]: Note 29 §F.13 item 3.
