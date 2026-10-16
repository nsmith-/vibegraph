---
type: Validation Gate
title: Which gate pins which convention-sign channel
description: "Each factor of convention_sign fires on a named process (asserted), each one dropped fails named MadGraph rows (measured), and the relabelling, renumbering and rooting gates check the graph-only reading; what none of them can see."
status: draft
tags: [amplitudes, signs, coverage, rooting, relabelling, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-16}]
sources:
  - {id: b-report, resource: "../sprints/amplitude-signs/sessions/B-report.md", title: "B′ report: per-factor mutations, the relabelling test, the acceptance runs"}
  - {id: bh-report, resource: "../sprints/amplitude-signs/sessions/B-H-report.md", title: "B-H report: mutations of the firing test, the relabelling test and the live-parity cancellation"}
  - {id: t1-report, resource: "../sprints/amplitude-signs/sessions/T1-report.md", title: "T1 report: the triple-gluon derivation and mutations M1–M4"}
  - {id: code-firing, resource: "vibegraph-lib/src/helas/eval/root_diagram.rs", title: "named_processes_exercise_every_convention_factor"}
  - {id: code-relabel, resource: "vibegraph-lib/src/helas/eval/relabelling.rs", title: "relabelling_preserves_per_diagram_amplitudes"}
  - {id: code-renumber, resource: "vibegraph-lib/src/helas/eval/renumbering.rs", title: "renumbering_preserves_signs_and_amplitudes"}
  - {id: code-rooting, resource: "vibegraph-lib/src/helas/eval/rooting_soundness.rs", title: "helas/eval/rooting_soundness.rs"}
  - {id: r1-report, resource: "../sprints/amplitude-signs/sessions/R1-report.md", title: "R1 report: the mutations re-measured on 0ec027a"}
  - {id: n19-v5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/19-validation-pass-plan.md#L148-L710", title: "Note 19 V5 — the rooting-soundness gate"}
  - {id: n24-gux, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L683-L703", title: "Note 24 P1 — the q̄ g row"}
  - {id: n24-mut, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L756-L809", title: "Note 24 P1 — mutation experiments"}
  - {id: n29-f10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L830-L976", title: "Note 29 §F.10 — which rows vary which channel"}
---

# Which gate pins which convention-sign channel

A convention sign that no gated process varies is not pinned by any comparison
against MadGraph: a sign shared by every diagram of a process is absorbed into
the amplitude oracle's fitted `G`, and `|M|²` cannot see it. The
[`AGENTS.md`](../../../AGENTS.md) rule — convention claims are hypotheses,
pinned by a test that would fail if they were false — is applied here three
ways: a test asserting that each factor fires on a named process, each factor
dropped against the MadGraph suites, and three invariance gates for the claim
that every factor reads the graph alone. The factors themselves are
[convention signs](../amplitudes/convention-sign-inventory.md).

## The channels in `fermi_sign`

```
fermi_sign = diagram.sign                           // Wick parity × fermion-line sign
           * convention_sign(diagram, model, chain) // propagator_phase_sign
                                                    //   × vertex_root_signs
                                                    //   × triple_gluon_sign
           * tree.live_reversed_parity()            // cancels the runtime parity
```

`spine_sign_from_flow` (the rooted-tree derivation of the line sign) is a
`debug_assert` cross-check on every live tree and the subject of
`fermion_line_sign_matches_the_rooted_tree_derivation`. See
[the fermion-line sign](../amplitudes/fermion-line-sign.md) and
[vector-vertex signs](../amplitudes/vector-vertex-signs.md).

## Each factor fires: the non-vacuity test

`named_processes_exercise_every_convention_factor` (default suite) counts, per
named process, the diagrams on which a factor is `−1`, and asserts each count is
`> 0`[^code-firing]:

| factor | process | what fires |
|---|---|---|
| `propagator_phase_sign`, scalar line | `b b~ > h h` | the s-channel Higgs of the triple-Higgs diagram |
| `propagator_phase_sign`, crossed fermion line | `e+ e- > ta+ ta- h` | the tau propagator when the Higgs comes off the final-state pair |
| `vertex_root_signs`, reversed bilinear | `e+ e- > mu+ mu-` | the initial electron line, read against its UFO order |
| `vertex_root_signs`, tensor path | `lt~ lt > qt qt~ NP<=1 NPGG<=1` (toy, `restrict_tensor`) | the tensor contact as the amplitude, on a crossed line |
| `triple_gluon_sign` | `g g > t t~` | the s-channel gluon |

Each part, set to `+1` on its own, fails the test (B-H report)[^bh-report]. The
test calls each factor directly, so it cannot see `convention_sign` drop a factor
from its product; that is the MadGraph rows' to see (next section). The `−1`
values are not its business either.

## Each factor dropped: the MadGraph rows

Each factor of `convention_sign` was set to `+1` on its own and the two
MadGraph suites run (B′; re-measured on `0ec027a` in the R1 review, which
reproduces B′'s counts and every row B′ names). Every `standalone_jamps` failure is a per-flow disagreement. An
`amplitude_oracle` row runs its per-diagram and per-flow checks before the
`CONFIG_AMP_SIGNS` tripwire, so a row that fails only the tripwire agrees with
MadGraph up to a sign shared by all its diagrams; those are counted apart. Each
factor varies within at least the rows of the first two columns[^b-report][^r1-report]:

| dropped | `standalone_jamps` fails | `amplitude_oracle` fails per diagram or per flow | `amplitude_oracle` fails only on `CONFIG_AMP_SIGNS` |
|---|---|---|---|
| every vertex reading | 17 rows (`ee_to_ee_4f`, `uux_to_ggg`, `tata_to_ttxh`, …) | 13 rows: `ee_to_ee`, `ee_to_wpwm`, `ee_to_wp0wm`, `ee_to_wp0wmt`, `ee_to_mumua`, `ee_to_mumu_4f`, `ee_to_mumu_tata_qcd0`, `ll_to_qqx_toy_yukawa`, `tata_to_ttx_tensor4f`, `ud_to_epemud_qcd0`, `uux_to_uux` and the two 2→6 rows | 15 rows (`ee_to_mumu`, `ee_to_ttx`, `ee_to_zh`, `uux_to_mumu`, …) |
| the tensor-path sign only | `tata_to_ttxa_tensor4f` | `ll_to_qqx_toy_tensor`, `tata_to_ttx_tensor4f` | none |
| the scalar-line phase | 25 rows (`bbx_to_hh`, `hh_to_hh_chbox`, `zz_to_ttx`, the `O_W` rows, …) | 9 rows: `bbx/uux_to_ccx_emmm_qcd0`, `gg_to_gg_cg`, `qqx_to_o8o8_toy_dcolor`, `gg_to_ttx_smlimit_qcd2`, `wpwm_to_wpwmz_cw`, `ee_to_mumu_tata_qcd0`, `ll_to_qqx_toy_yukawa`, `tata_to_ttx_tensor4f` | none |
| the crossed-line fermion phase | 22 rows (`ee_to_ttxh`, `wpwm_to_epem`, `wph_to_tbx`, …) | 9 rows: `gg_to_ttx` and its two SM-limit rows, `ee_to_tatah`, `ee_to_mumua`, `ee_to_mumu_tata_qcd0`, `ud_to_epemud_qcd0` and the two 2→6 rows | none |
| the triple-gluon sign | `uux_to_ggg`, `ug_to_ug`, `gg_to_uux`, `ttx_to_gg_chg`, the five `t t~ + jet` rows, `uux_to_ttxg_4f` | `gg_to_ttx`, `gg_to_ttx_smlimit`, `gg_to_ttx_smlimit_qcd2` | none |

The relabelling test passes under every one of these mutations: each is still
a function of the graph.

**The fermion-line sign** (on the diagram, not in `convention_sign`):

- Its two arms are checked on `u d > e+ e- u d QCD=0`, whose 35 diagrams split 24
  / 9 / 2 by where the internal fermion propagator sits (a mixed quark line, the
  crossed lepton line, none); `spine_sign_separates_mixed_line_and_crossed_line_propagators`
  asserts the split and the signs, with `g g > t t~` as the control that a
  crossed line's −1 does not count propagators.
- In Bhabha and `u u~ > u u~` the pairing sign and the line sign cancel exactly,
  so those rows pin the product, not either factor[^n29-f10].
- The four ℓℓj partonic rows do **not** pin the line sign: it fires on all four
  diagrams of `g q → ℓℓq`, so dropping it is a global sign there. That mutation
  failed seven `|M|²` rows and two per-diagram rows, none of them ℓℓj[^n24-mut].
  A slot rule wrong only for `g u~` was caught by `gux_to_epemux` alone[^n24-gux].

## The graph-only reading: three invariance gates

| gate | holds | catches | blind to |
|---|---|---|---|
| `relabelling_preserves_per_diagram_amplitudes` (default suite) | twelve processes against the same process with its legs listed in another order, each diagram paired with its partner by a label-free graph signature and compared per helicity at a generic point, up to one sign per process; at least two diagrams compared per pair | a sign keyed to the vertex holding a particular leg: a `−1` wherever leg 0's vertex has a fermion leg flips diagrams of `w+ e- > z ve`, `w+ a > e+ ve a` and the toy `st lt > st lt`[^bh-report] | a wrong function of the graph (every factor mutation above passes it); a dependence on a vertex no listed relabelling moves; multi-flow processes |
| `renumbering_preserves_signs_and_amplitudes` (default suite) | every census diagram renumbered twice (once with leg 0's vertex forced off index 0): exact `fermi_sign` per colour chain, the same canonical form, and on subprocesses of at most 40 diagrams per-helicity per-flow amplitudes to 1e-10 | a sign read off a vertex or propagator *index* | a wrong function of the graph |
| `rooting_soundness` | \|M\|² under re-rooting against the production rooting's own value | a kernel that is not rooting-invariant; a live parity that is not cancelled | any error shared by every rooting, which includes every convention-sign error |

### The rooting-soundness check

`helas/eval/rooting_soundness.rs` (a module of the library) re-roots diagrams
through a test-only override hook and requires `|M|²` to match the production
rooting's value, which [the amplitude oracle](amplitude-oracle.md) pins against
MadGraph[^n19-v5]. Per-diagram root isolation runs for processes up to 40
diagrams, whole-process re-rooting for the two 579/615-diagram 2 → 6 processes,
at `REL_TOL = 1e-10`: re-rooting reassociates momentum sums, and the measured
floor is 2.2e-11 (`e+ e- > ta+ ta- H`), far below any O(1) sign error. The
convention sign reads no root, so what the module tests is the kernels'
rooting invariance and the cancellation of the live tree's reversed parity in
`fermi_sign`[^code-rooting]. That cancellation dropped is seen by
`literal_sigma_currents_are_rooting_invariant` in the default suite and by 3 of
the sweep's 270 re-rootings, all on `b b~ > c c~ e+ e- mu+ mu- QCD=0`; the
relabelling and renumbering gates pass it (B-H report)[^bh-report]. The
MadGraph suites see it too: 6 `standalone_jamps` rows (`ee_to_ee_4f`,
`tata_to_ttxh`, `ttx_to_gg_chg`, …) and 7 `amplitude_oracle` rows per diagram
or per flow (`ee_to_ee`, `uux_to_uux`, `bbx_to_ccx_emmm_qcd0`, …), with 24 more
on `CONFIG_AMP_SIGNS` only (R1 review)[^r1-report].

- `root_override_hook_is_transparent` runs in the default suite: an explicit
  `VtxIdx(0)` override reproduces the default `|M|²` bit for bit on
  `e+ e- > mu+ mu-` and Bhabha, where the canonical root is vertex 0.
- `literal_sigma_currents_are_rooting_invariant` runs in the default suite;
  `momentum_slashed_chain_is_rooting_invariant`,
  `four_fermion_currents_are_rooting_invariant` and
  `tensor_four_fermion_currents_are_rooting_invariant` are
  `extended-validation` tests. Each re-roots a few diagrams whose currents a
  structure-specific defect would make rooting-dependent.
- `all_rootings_preserve_amplitude` is `#[ignore]` for cost:

  ```
  RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib \
      --lib helas::eval::rooting_soundness::all_rootings_preserve_amplitude \
      -- --ignored --nocapture --test-threads=1
  ```

  It passes with 0 failures; the re-rooting count grows with the validated
  process list (270 at the last recorded run).

## What no gate here sees

- **Which factor carries the triple-gluon `−1`.** The sign is derived: it is the
  slot exchange the all-incoming crossing gives every charged triple-vector
  vertex inside its kernel, which the self-conjugate gluon vertex does not get
  ([vector-vertex-signs](../amplitudes/vector-vertex-signs.md)). Both MadGraph
  gates are blind to where it sits. Moved into the colour (`f_to_traces`
  negated, `triple_gluon_sign` at `+1`), all 49 tests of `standalone_jamps`
  (its 45 table rows and four checks) and all 49 of `amplitude_oracle` pass; moved into the coupling (`GC_10` negated),
  every SM row passes and only SMEFTsim rows, whose triple-gluon coupling has
  another name, fail. `amplitude_oracle` compares `gg_to_ttx` per flow, not per
  diagram, although its table banks `AMP()`. The colour and the rest are pinned
  separately by `helas::eval::triple_gluon` on `g g > t t~`, with
  `e+ e- > w+ w-` as the colourless control[^t1-report].
- **The triple-gluon predicate is a proxy** for "no conjugate pair among the
  legs": a colourless vertex of three self-conjugate vectors would need `−1` by
  the derivation and gets none; no model in reach has one.
- **The per-process sign is free.** Every oracle here and `|M|²` are blind to a
  sign shared by all diagrams of a process; `CONFIG_AMP_SIGNS` banks the one the
  convention gives as a tripwire.
- **The tensor-path sign** is pinned on (initial, crossed) line pairs only. A
  cyclic structure on two mixed lines is unmeasured.
- **No committed row** reaches a scalar⊗scalar four-fermion contact rooted in a
  tree (`O_ledq`-type) or a VSS vertex.
- Majorana lines (none in the SM UFO) and charge-flow phrasings that agree on
  every tree diagram are untestable with the banked set; see
  [the charge-flow phase study](../amplitudes/charge-flow-phase-study.md).

[^b-report]: B′ report, "Mutations (each factor pinned by MadGraph)" and "Relabelling test".
[^bh-report]: B-H report, "Mutations".
[^r1-report]: R1 report, "Mutations".
[^t1-report]: T1 report, "Mutations" M2 and M3.
[^code-firing]: `vibegraph-lib/src/helas/eval/root_diagram.rs` tests.
[^code-rooting]: `vibegraph-lib/src/helas/eval/rooting_soundness.rs`, module doc and `REL_TOL`.
[^n19-v5]: Note 19 V5.
[^n24-gux]: Note 24 P1, the `q̄ g` gap.
[^n24-mut]: Note 24 P1, mutation experiments 3 and 4.
[^n29-f10]: Note 29 §F.8 and §F.10, hostile cases H2–H5.
