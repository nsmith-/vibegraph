---
type: Session Report
title: "T1 report: the triple-gluon −1 is the slot exchange every other triple-vector vertex gets from the crossing"
description: "Per diagram, helicity and flow against MadGraph, the triple-gluon diagram's colour coefficients and coupling are MadGraph's, so its −1 is on the Lorentz side. The all-incoming crossing exchanges the W⁺/W⁻ slots of every charged triple-vector vertex, VVV1 gives that kernel −1 against MadGraph, and the rest of convention_sign is set against kernels that carry it. A gluon is self-conjugate, so the triple-gluon kernel gets no exchange and triple_gluon_sign supplies it. The constant is kept, its doc states the derivation, and four tests pin colour, the rest and the colourless control separately."
status: draft
tags: [amplitude, sign-convention, colour, triple-gluon, crossing, derivation, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: code, resource: "https://github.com/nsmith-/vibegraph/commit/9549fcb", title: "docs(amplitudes): derive the triple-gluon sign and pin its factors against MadGraph"}
---
Item: `triple-gluon-sign-calibrated-not-derived`. Built on `3bbe4eb`. The
constant, its predicate and every number the evaluator produces are unchanged.
What changed is the doc of `triple_gluon_sign` (and its line in
`convention_sign`'s doc), its visibility (`pub(super)`, for the tests), and a new
test module, `helas::eval::triple_gluon`. The stop rule did not fire (see
"Stop rule").

## Correction to the brief

- **The sign is not in the colour.** The brief, the item and the old doc all
  place it in "how the colour-ordered decomposition of `f^{abc}` meets the vertex
  slots". It is not there. Every colour coefficient of the traced diagram equals
  MadGraph's JAMP coefficient term by term. The coupling is MadGraph's too. The
  `−1` is on the Lorentz side, and the colourless triple-vector vertices carry
  the same `−1` inside their kernels.
- **None of the brief's four candidate factors is the source.** `colorize` /
  `f_to_traces`, the order of `f(1,2,3)` against `VVV1`'s slots, `GC_10` and
  MadGraph's JAMP decomposition all agree with MadGraph (numbers below). The
  source is a fifth factor, the crossing's slot exchange at a vector vertex.
- **The committed falsifiers run on `g g > t t~`, not `u u~ > g g`.** The
  tests need MadGraph's per-graph `AMP()` from a committed table. No committed
  table banks `AMP()` for `u u~ > g g` or `g g > u u~`. The banked
  `validation/madgraph/amplitudes/gg_to_ttx.json` does, from MadGraph's Fortran.
  The `u u~ > g g` trace itself was done, on MadGraph's own ALOHA routines
  (below). The dump "with the −1 removed" is the same run: the probe prints
  each factor of `fermi_sign` separately.

## How the trace was done (instruments in the scratchpad, not committed)

- **MadGraph side, `u u~ > g g`.** No MadGraph run was possible or needed:
  there is no `gfortran` here, and the brief forbids regenerating outputs.
  ALOHA's Python writer, run from the pinned submodule
  (`aloha.create_aloha.AbstractALOHAModel('sm')`, `write(..., 'Python')`),
  generated `VVV1_1`, `FFV1_0`, `FFV1_1` and `FFV1_2`, and
  `aloha/template_files/wavefunctions.py` gave `ixxxxx`, `oxxxxx` and `vxxxxx`.
  I followed the call sequences of `P1_qq_gg/matrix1_orig.f` and
  `P1_gg_qq/matrix1_orig.f` from the `pp_to_jj` work area (JAMP coefficients
  included), with `GC_10 = −G` and `GC_11 = iG` at the table's `aS`. On the
  committed `standalone/gg_to_uux.json` (3 points × 16 helicities) the
  reconstructed `g g > u u~` JAMPs reproduce MadGraph's Fortran JAMPs with worst
  relative deviation `0.0`. They are bit for bit identical, because the routines
  are the same arithmetic. `u u~ > g g` then uses the same routines at the same
  momenta.
- **MadGraph side, `g g > t t~` and `e+ e- > w+ w-`.** These come from the
  committed amplitude tables, which bank Fortran `AMP()` and `JAMP()` on 6
  points each.
- **This side.** A throwaway `#[ignore]` probe in `root_diagram.rs`'s tests,
  since removed, dumped for every diagram:
  - the rays in slot order;
  - its colour contributions, from `colorize_process`;
  - each factor of `fermi_sign` (`Diagram::sign`, `propagator_phase_sign`,
    `vertex_root_signs`, `triple_gluon_sign`, the live tree's reversed parity);
  - the per-flow value of the diagram compiled alone, per banked helicity and
    point.

  A script paired the diagrams with the graphs, fitted the one process constant
  `G` on the diagram sums, and divided out each factor.

## The numbers

In each table, *ours / (G·c·AMP)* is the diagram's whole per-flow value against
MadGraph's `c_{fk}·AMP(k)`, under the one fitted `G`. *Colour* is our colour
coefficient against MadGraph's JAMP coefficient (MadGraph's includes its fermion
factor). *Raw kernel* is the diagram's value with the colour, `fermi_sign` and
the live parity divided out, over `G·AMP(k)`. Its sign is meaningful within a
process only. The last five columns are the factors of `fermi_sign`: *sign* =
`Diagram::sign`, *prop* = `propagator_phase_sign`, *vroot* =
`vertex_root_signs`, *live* = the live tree's reversed parity (it cancels the
runtime's), and *tg* = `triple_gluon_sign`.

**`u u~ > g g`** (ALOHA-Python MadGraph, the `gg_to_uux` points), `G = +1.0i`:

| diagram | rays (slot ← leg) | graph | ours / (G·c·AMP) | colour ours / MG | raw kernel | sign | prop | vroot | live | tg |
|---|---|---|---|---|---|---|---|---|---|---|
| 0, s-channel g | `u~ u g ← 2 1 p`; `g g g ← 3 4 p` | AMP(1) | 1.000000000000000 (worst 3.3e-16) | `−i/−i`, `+i/+i` | +1 | +1 | +1 | −1 | −1 | −1 |
| 1, quark exchange | `u~ u g ← p 1 3`; `← 2 p 4` | AMP(3) | 1.000000000000000 (2.4e-15) | `1/1` | −1 | −1 | +1 | +1 | −1 | +1 |
| 2, quark exchange | `← p 1 4`; `← 2 p 3` | AMP(2) | 1.000000000000000 (4.5e-16) | `1/1` | −1 | −1 | +1 | +1 | −1 | +1 |

**`e+ e- > w+ w-`** (Fortran `AMP()`, `amplitudes/ee_to_wpwm.json`), `G = −1.0i`:

| diagram | rays (slot ← leg) | graph | ours / (G·c·AMP) | colour ours / MG | raw kernel | sign | prop | vroot | live | tg |
|---|---|---|---|---|---|---|---|---|---|---|
| 0, s-channel γ | `e+ e- a ← 1 2 p`; `a W- W+ ← p 3 4` | AMP(1) | 0.999999999999995 (7.3e-16) | `1/−1` | +1 | +1 | +1 | −1 | −1 | +1 |
| 1, s-channel Z | `e+ e- Z ← 1 2 p`; `W- W+ Z ← 3 4 p` | AMP(2) | 1.000000000000000 (3.4e-15) | `1/−1` | +1 | +1 | +1 | −1 | −1 | +1 |
| 2, ν exchange | `e+ ve W- ← 1 p 3`; `ve~ e- W+ ← p 2 4` | AMP(3) | 1.000000000000000 (1.1e-15) | `1/−1` | +1 | −1 | +1 | +1 | −1 | +1 |

**`g g > u u~`** (ALOHA-Python MadGraph, bit-exact on the committed JAMPs), `G = −1.0i`:

| diagram | rays (slot ← leg) | graph | ours / (G·c·AMP) | colour ours / MG | raw kernel | sign | prop | vroot | live | tg |
|---|---|---|---|---|---|---|---|---|---|---|
| 0, s-channel g | `g g g ← 1 2 p`; `u~ u g ← 3 4 p` | AMP(1) | 0.999999999999990 (5.8e-15) | `−i/+i`, `+i/−i` | −1 | −1 | +1 | +1 | +1 | −1 |
| 1, quark exchange | `u~ u g ← 3 p 1`; `← p 4 2` | AMP(2) | 0.999999999999990 (2.3e-15) | `1/−1` | −1 | −1 | −1 | +1 | +1 | +1 |
| 2, quark exchange | `← p 4 1`; `← 3 p 2` | AMP(3) | 1.000000000000002 (7.1e-15) | `1/−1` | −1 | −1 | −1 | +1 | +1 | +1 |

**`g g > t t~`** (Fortran `AMP()`, `amplitudes/gg_to_ttx.json`), `G = −1.0i`: the
same factors as `g g > u u~` on every diagram, worst entries 3.7e-16, 1.7e-16
and 5.7e-16.

The rays of the colourless `w+ w- > u u~` (`standalone/wpwm_to_uux.json`,
gated, single flow, no banked `AMP()`) were dumped too. Its s-channel γ and Z
have `sign −1, prop +1, vroot +1, live +1`, with rays `a W- W+ ← p 2 1` and
`u~ u a ← 3 4 p`. Its d exchange has `sign −1, prop −1, vroot +1, live +1`,
with rays `u~ d W+ ← 3 p 1` and `d~ u W- ← p 4 2`. That is `g g > u u~`'s
pattern diagram for diagram, fermion rays included.

## The derivation

1. **Colour is MadGraph's.** Our `(diagram, flow)` coefficients for
   `u u~ > g g` are `(−i, +i)` on `T(3,4,2,1)`/`T(4,3,2,1)` for the s-channel
   gluon, and `1` for each quark exchange. These are `P1_qq_gg`'s JAMP
   coefficients exactly: `JAMP(1) = −i·AMP(1) + AMP(2)`,
   `JAMP(2) = +i·AMP(1) + AMP(3)`. In `g g > u u~` and `g g > t t~` every one
   of our coefficients is MadGraph's times `−1`, on every diagram alike. That
   `−1` is MadGraph's fermion factor, which it writes into its coefficients
   (`i·AMP(1) − AMP(2)`). By hand, MadGraph's `color_amp.add_vertex` puts the
   output leg first, so the triple-gluon vertex is `f(e,3,4)`. The
   quark–gluon vertex `T(3,2,1)` on `[u~, u, g]` reads `T(e,2,1)` in the
   all-outgoing crossing. So `f^{e34}T^e_{21} = −i[T³,T⁴]_{21}`, which is
   `−i` on `T(3,4,2,1)`. Here `convert_expr` transposes the
   `T(e,1,2)` that feyngraph's slot assignment gives into the same `T(e,2,1)`,
   and leaves `f` alone. `f_to_traces` is MadGraph's expansion. Nothing in
   colour differs.
2. **`f(1,2,3)` and `VVV1` see the same slot order.** Our triple-gluon rays are
   `[3, 4, p]` (`u u~ > g g`) and `[1, 2, p]` (`g g > u u~`). MadGraph's are
   `[p, 3, 4]`: `VVV1P0_1(W3, W4)` puts the off-shell gluon in slot 1, and
   `add_vertex` puts it first in the colour. That is a cyclic rotation. Any
   permutation would act on `f` and `VVV1` together, since both are read
   positionally and both are antisymmetric, so their product cannot change.
3. **`GC_10` is MadGraph's.** It is `−G` on both sides, from the same UFO and the
   same card. MadGraph passes it unmodified (`VVV1P0_1(..., GC_10, ...)`), and
   every raw-kernel ratio above has modulus 1.
4. **The difference is the crossing at a vector vertex.** feyngraph reads every
   interaction slot incoming (`Prop`'s doc: the slot particles "are all read
   incoming"). MadGraph reads every leg outgoing. So each leg stands in its
   antiparticle's slot, and the two slots of every conjugate pair hold each
   other's legs.
   - In `e+ e- > w+ w-` our `a W- W+` rays are `[p, 3, 4]`: the final `W⁺`
     (leg 3) stands in the `W⁻` slot. MadGraph's call is
     `VVV1_0(W5, W4, W3, −GC_3)`, with leg 3 in the `W⁺` slot. (`−GC_3 = GC_4`
     is MadGraph merging equal couplings, not a sign.) `VVV1` is odd under the
     exchange of two slots and their momenta, so the charged vertex's kernel is
     `−1` against MadGraph's `COUP·L`.
   - At a fermion vertex the same exchange is undone: the colour by
     `convert_expr`'s transpose, the bilinear by the bra/ket binding and
     `vertex_root_signs`.
   - At the triple-gluon vertex no slot changes particle, because a gluon is
     its own antiparticle.
5. **The rest of the convention is set against kernels that carry the
   exchange.** `e+ e- > w+ w-` and `u u~ > g g` carry the same `sign`, `prop`,
   `vroot` and `live` diagram for diagram: `+1, +1, −1, −1` on the s-channel and
   `−1, +1, +1, −1` on the t-channel. MadGraph agrees with the first with no
   vertex sign. Its raw s/t kernel ratio is `+1`: the charged vertex's `−1`
   matches the t-channel fermion propagator's `−1` (this side's `−i` against
   ALOHA's `+i`). In `u u~ > g g` the raw s/t ratio is `−1`, because the
   triple-gluon kernel has no exchange. `w+ w- > u u~` and `g g > u u~` repeat
   the comparison on a crossed line. Both have the same factors diagram for
   diagram, the first agrees with MadGraph with no vertex sign, and the second
   needs `−1` on the triple-gluon diagram.

**The convention, stated.** Against MadGraph's all-outgoing reading, every
triple-vector vertex in this crate's reading carries `−1`. A charged one carries
it inside its kernel, from the `W⁺`/`W⁻` slot exchange the crossing makes and
`VVV1`'s antisymmetry. The triple-gluon vertex, whose legs are self-conjugate,
carries it in `triple_gluon_sign`. The rest of `convention_sign` assumes it. The
oracle sees only signs relative within a process, so "the rest assumes it" is a
statement about the whole set of factors, not about one of them. By the vertex
identity `n_qg + n_ggg ≡ n_ext_g (mod 2)` the same convention can be written as
`−1` per quark–gluon vertex, up to a sign shared by the process, which the old
doc already noted.

## Stop rule

It did not fire.

- The derived rule counts the triple-vector vertices whose slots the crossing
  does not exchange, those with no conjugate pair among their legs. In every
  model in reach (SM, SMEFTsim, the two toy UFOs), the triple-vector vertices
  are `g g g` (the SM's `V_36`, and SMEFTsim's `V_9` with its `O_G` structures),
  `a W- W+` and `W- W+ Z`. The toy UFOs have no vector triple vertex. So the
  derived rule and `is_yang_mills_vvv && is_coloured` count the same vertices
  everywhere.
- No row changes: the constant, the predicate and the tree are untouched.
- The derived rule differs from the code only for a colourless triple-vector
  vertex of three self-conjugate legs, which no model here has. I did not
  change the predicate for an unmeasured class. The doc says so, and it is
  under Found.

## Tests added (`vibegraph-lib/src/helas/eval/triple_gluon.rs`)

The tests read the committed `amplitudes/gg_to_ttx.json` and `ee_to_wpwm.json`.
They check the transcribed `g g > t t~` JAMP coefficients against the banked
`JAMP()` rebuilt from the banked `AMP()` at `1e-12` of each entry's scale, and
check that the colour basis keys are MadGraph's `flow_structures` in order.

| test | asserts | blind to |
|---|---|---|
| `the_triple_gluon_diagram_agrees_with_madgraph` | every `g g > t t~` diagram's per-flow value is `G·c·AMP` under one `G` (`\|G\|=1`), at 1e-12; the triple-gluon diagram's sign is `−1` | a sign shared by every diagram; how the value splits into factors |
| `the_triple_gluon_diagram_has_madgraphs_colour` | our colour coefficient over MadGraph's JAMP coefficient is one constant of modulus 1 over every `(diagram, flow)`, at 1e-15 | a colour sign common to every diagram; anything outside colour |
| `the_triple_gluon_diagram_reads_minus_one_outside_its_colour` | with colour and `triple_gluon_sign` divided out, the quark exchanges are `K·AMP` under one `K` and the triple-gluon diagram is `−K·AMP(1)`, at 1e-12 | the split among kernel, coupling, propagator phases and the other factors of `fermi_sign`; a flip of `triple_gluon_sign` alone, which the first test catches |
| `a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel` | on `e+ e- > w+ w-`, with colour divided out, all three diagrams are `K·AMP` under one `K`; each final-state W on a triple-vector vertex stands in its antiparticle's slot (the crossing's exchange), never in its own | the split inside the rest; vertices other than `a W- W+` and `W- W+ Z`; a crossing change that moves a gluon slot (none can) |

Every test also inherits the oracle's own blind spots: one process each, six
banked points, and only the helicities those points carry.

## Mutations

Each mutation was applied to the committed tree by a scratch driver
(`mutate.py`, `run_mutations.sh`), then run through the new tests,
`standalone_jamps` and `amplitude_oracle`, then reverted. Every cargo call ran
with `CARGO_BUILD_JOBS=3 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`. After the last revert `git diff --stat` showed only
the session's own changes.

**M1: flip the constant** (`triple_gluon_sign` returns `+1`).

```
$ cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --nocapture --test-threads=1
test ...the_triple_gluon_diagram_agrees_with_madgraph ... g g > t t~, each diagram against G·c·AMP: worst 4.039e-1, G = -0.000000000000000-0.628217558524257i
...panicked at vibegraph-lib/src/helas/eval/triple_gluon.rs:395:5:
|G| = 0.6282175585242568
test result: FAILED. 3 passed; 1 failed
$ cargo test -p vibegraph-lib --test standalone_jamps -- --test-threads=1
test gg_to_ttxg ... FAILED      test gg_to_uux ... FAILED      test gu_to_ttxu ... FAILED
test gux_to_ttxux ... FAILED    test ttx_to_gg_chg ... FAILED  test ug_to_ttxu ... FAILED
test ug_to_ug ... FAILED        test uux_to_ggg ... FAILED     test uux_to_ttxg ... FAILED
test uux_to_ttxg_4f ... FAILED
test result: FAILED. 39 passed; 10 failed
$ cargo test -p vibegraph-lib --test amplitude_oracle -- --test-threads=1
failures: gg_to_ttx, gg_to_ttx_smlimit, gg_to_ttx_smlimit_qcd2
[gg_to_ttx] per-flow JAMPs disagree with MadGraph beyond the per-diagram constant (max element-wise deviation 5.877e-1) ...
test result: FAILED. 46 passed; 3 failed
```

The colour test and the rest test pass, as they must: the rest divides the
triple-gluon sign out, so a flip of the constant alone cannot move it. The
failing set is B′'s.

**M2: move the `−1` into the colour** (`f_to_traces` gives
`+2i·Tr(abc) − 2i·Tr(cba)`, and `triple_gluon_sign` returns `+1`). Every diagram
keeps its value.

```
$ cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --nocapture --test-threads=1
test ...the_triple_gluon_diagram_agrees_with_madgraph ... g g > t t~, each diagram against G·c·AMP: worst 1.720e-15, G = 0.000000000000000-0.999999999999999i
...panicked at vibegraph-lib/src/helas/eval/triple_gluon.rs:400:5:
assertion `left == right` failed
  left: 1.0
test ...the_triple_gluon_diagram_has_madgraphs_colour ... g g > t t~, colour over MadGraph's coefficient: [(0, 0, Complex { re: 1.0, im: 0.0 }), (0, 1, Complex { re: 1.0, im: 0.0 }), (1, 0, Complex { re: -1.0, im: -0.0 }), (2, 1, Complex { re: -1.0, im: -0.0 })]
...panicked at vibegraph-lib/src/helas/eval/triple_gluon.rs:424:9:
diagram 1, flow 0: colour coefficient over MadGraph's is -1+-0i, against 1+0i on diagram 0, flow 0
test result: FAILED. 2 passed; 2 failed
$ cargo test -p vibegraph-lib --test standalone_jamps -- --test-threads=1
test result: ok. 49 passed; 0 failed
$ cargo test -p vibegraph-lib --test amplitude_oracle -- --test-threads=1
test gg_to_ttx ... [gg_to_ttx] 'g g > t t~' NGRAPHS=3 NCOLOR=2: |M|² max_rel grid 3.18e-15 / event 1.53e-15, per-diagram 0.00e0 (not banked), per-flow 1.87e-15, ...
test result: ok. 49 passed; 0 failed
```

Both MadGraph gates pass. They cannot see which factor carries the sign. The
colour test catches the move. The totals test fails only on its literal
`triple_gluon_sign == −1`; its values agree at 1.7e-15. `amplitude_oracle`
compares `gg_to_ttx` per flow and not per diagram ("not banked"), although its
table carries `AMP()`.

**M3: move the `−1` into the coupling** (the evaluated `GC_10` is negated in
`EvaluatedModel::from_model_card`, and `triple_gluon_sign` returns `+1`). In the
SM `GC_10` is the triple-gluon coupling, used at tree level by `V_36` and nowhere
else, so every SM diagram keeps its value.

```
$ cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --nocapture --test-threads=1
test ...the_triple_gluon_diagram_agrees_with_madgraph ... g g > t t~, each diagram against G·c·AMP: worst 1.720e-15, G = 0.000000000000000-0.999999999999999i
...panicked at vibegraph-lib/src/helas/eval/triple_gluon.rs:400:5:   (the literal triple_gluon_sign == −1)
test ...the_triple_gluon_diagram_reads_minus_one_outside_its_colour ... g g > t t~, colour and triple-gluon sign divided out, against AMP: quark exchanges K = 0.000000000000000+0.999999999999999i (worst 1.758e-15), triple-gluon diagram -0.000000000000000+1.000000000000000i
...panicked at vibegraph-lib/src/helas/eval/triple_gluon.rs:458:5:
the triple-gluon diagram, colour and triple-gluon sign divided out, is not −K·AMP(1) (K = 0.000000000000000000000000000000001605652286243322+0.9999999999999989i): worst 4.961e-1
test result: FAILED. 2 passed; 2 failed
$ cargo test -p vibegraph-lib --test standalone_jamps -- --test-threads=1
test ttx_to_gg_chg ... FAILED   test uux_to_ttxg_4f ... FAILED
test result: FAILED. 47 passed; 2 failed
$ cargo test -p vibegraph-lib --test amplitude_oracle -- --test-threads=1
failures: gg_to_ttx_smlimit, gg_to_ttx_smlimit_qcd2
test result: FAILED. 47 passed; 2 failed
```

The colour test passes and the rest test catches the move. Every SM row of
both gates passes (`gg_to_uux`, `uux_to_ggg`, `ug_to_ug`, `gg_to_ttx`, the
`t t~ + jet` rows): they cannot see it either. The four rows that fail are
SMEFTsim rows. There the triple-gluon coupling is not named `GC_10`, so the
mutation only drops the sign, as M1 does. That is a limit of the mutation, not
of the test.


**M4: attribute the sign to every triple-vector vertex** (`triple_gluon_sign`
drops `is_coloured`, so `a W- W+` and `W- W+ Z` take `−1` too).

```
$ cargo test -p vibegraph-lib --lib helas::eval::triple_gluon -- --nocapture --test-threads=1
test ...a_charged_triple_vector_vertex_carries_its_sign_in_its_kernel ... e+ e- > w+ w-, each diagram against G·c·AMP: worst 3.985e-1, G = 0.000000000000000-0.601459031929820i
...panicked at vibegraph-lib/src/helas/eval/triple_gluon.rs:478:5:
a diagram of e+ e- > w+ w- disagrees with MadGraph's c·AMP: worst 3.985e-1 (G = 0.000000000000000019205461964210726-0.6014590319298199i)
test result: FAILED. 3 passed; 1 failed
$ cargo test -p vibegraph-lib --test standalone_jamps -- --test-threads=1
FAILED: aem_to_wmve, ee_to_wpwmwpwm_cw, ee_to_wpwmz, wmwp_to_emep, wpem_to_zve, wpwm_to_epem,
        wpwm_to_epemz, wpwm_to_uux, wpwm_to_uuxg, wpwm_to_wpwma_cw, wpwm_to_wpwmz_cw
test result: FAILED. 38 passed; 11 failed
$ cargo test -p vibegraph-lib --test amplitude_oracle -- --test-threads=1
failures: ee_to_wp0wm, ee_to_wp0wmt, ee_to_wpwm, ud_to_epemud_qcd0, wpwm_to_wpwmz_cw
test result: FAILED. 44 passed; 5 failed
```

This closes the gap the vector-vertex concept records ("no mutation has been
run that puts a sign on a colourless vector vertex alone"): the charged
triple-vector vertex's sign is pinned by 11 standalone rows and 5 amplitude rows,
and by the control test.

## Gates

Every cargo call ran with `CARGO_BUILD_JOBS=3 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, on the tree of `9549fcb`.

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace --no-fail-fast`: exit 0, with 38 suites, 1438
  passed, 0 failed and 16 ignored (summed `test result` lines). The four new
  tests are among them.
- `cargo test -p vibegraph-lib --test standalone_jamps --test amplitude_oracle
  -- --nocapture --test-threads=1`: exit 0. `amplitude_oracle`: `test result:
  ok. 49 passed; 0 failed`. `standalone_jamps`: `test result: ok. 49 passed; 0
  failed`, with `KNOWN_DISAGREEMENT` empty. Rows the derivation leans on:
  - `gg_to_uux` worst flow 7.555e-15;
  - `uux_to_ggg` 3.880e-16;
  - `ug_to_ug` 3.383e-16;
  - `wpwm_to_uux` 1.860e-15;
  - oracle `gg_to_ttx` per-flow 1.87e-15;
  - oracle `ee_to_wpwm` per-diagram 1.05e-15.

  `gg_to_uux` and `wpwm_to_uux` are B′'s numbers to the digit.
- New tests (`cargo test -p vibegraph-lib --lib helas::eval::triple_gluon --
  --nocapture`):
  - `g g > t t~`, each diagram against `G·c·AMP`: worst 1.720e-15,
    `G = −0.999999999999999i`;
  - colour over MadGraph's coefficient: `−1` on all four `(diagram, flow)`
    entries;
  - rest: quark exchanges `K = +0.999999999999999i` (worst 1.758e-15), and the
    triple-gluon diagram `−1.000000000000000i`, which is `−K`;
  - `e+ e- > w+ w-`: totals worst 1.046e-15 and rest worst 1.046e-15.
- `pixi run --skip-deps validate`: `VALIDATE EXIT 0`, with 63 suites, 1823 passed, 0 failed and 84 ignored. The
  cell line reads `64 rows × 4 categories = 256 cells: 203 measured in the
  layers this run drove (197 ✅, 6 ⚠️, 13 ⏳, 40 — / uncovered)`, followed by
  `the measured cells are the declared cells`. No pre-change report was on
  disk in this worktree, so I did not diff the cells. No evaluator number can
  move, because the change is a doc comment, a visibility and a test module.
  The ✅/⚠️ split differs from B′'s 196/7 by F2's flip of `wpwm_to_wpwmz_cw`,
  which landed after B′.
- `pixi run -e kb kb-lint` and `python3 scripts/kb.py index --check`: `619 concepts, 207 backlog items; 0 errors, 2 warnings` (the two pre-existing `state: blocked` warnings), and the index check exits 0.

## Found

- **The kb concepts still say the sign is in the colour, or underived.** These
  need the change:
  - `amplitudes/vector-vertex-signs`: "Its sign belongs to the colour-ordered
    decomposition…", and its "Open" item;
  - `amplitudes/convention-sign-inventory`: the triple-gluon paragraph, and the
    blind-spot bullet "has no derivation";
  - `validation/convention-channel-coverage`: "calibrated, not derived";
  - `validation/bit-exact-amplitude-debugging`: "takes a calibrated `−1`".

  To close: rewrite those passages from "The derivation" above.
- **The backlog item's `closes_when` is met, but I left the file in place.**
  The brief `sessions/T1.md` links it by relative path, so deleting it fails
  `kb-lint` ("link … does not resolve"), and I may not edit the brief. To
  close: at close-out, delete
  `backlog/validation/triple-gluon-sign-calibrated-not-derived.md` and point the
  brief's link at this report.
- **The predicate is a proxy.** `triple_gluon_sign` counts
  `is_yang_mills_vvv && is_coloured`. The derived rule counts triple-vector
  vertices with no conjugate pair among their legs. They part only on a
  colourless vertex of three self-conjugate vectors, which is predicted to need
  `−1` and gets none. A W-pair structure even under the W exchange would also
  take `−1` by this account. There is none in reach: a scratch check
  (`vvv_swap.py`, a numeric parity test at random momenta with momentum
  conservation) finds every triple-vector structure on a W-pair vertex of the
  SM and SMEFTsim UFOs odd under the exchange. That covers `VVV1`–`VVV7` and
  `VVV9` on `a W- W+` and `W- W+ Z`; `VVV3` vanishes identically. To close: a
  toy-UFO standalone row with a neutral antisymmetric triple-vector vertex
  against MadGraph `AMP()`, then either a predicate that reads the legs'
  self-conjugacy or a documented refusal.
- **No committed table banks `AMP()` for `u u~ > g g` or `g g > u u~`.**
  `gen_standalone_jamps.py`'s `bank_amps` asserts a single-flow process
  ("bank_amps needs a single-flow process"), so `gg_to_uux` banks JAMPs only,
  and the in-tree falsifiers had to use `g g > t t~`. To close: bank the JAMP
  coefficient matrix for multi-flow rows, rebank `gg_to_uux` with `AMP()`, and
  point the `triple_gluon` tests at it as well.
- **ALOHA's Python writer is a MadGraph-side instrument that needs neither
  `gfortran` nor a MadGraph run.** On the `gg_to_uux` points it reproduced
  MadGraph's Fortran JAMPs bit for bit (worst deviation `0.0`), since the
  routines are the same arithmetic. For a diagnosis where the call sequence is
  known (from a banked `matrix1_orig.f`), it gives per-graph `AMP()` on a
  machine with no Fortran toolchain. To close: commit the generator and
  wavefunction shim beside `compare_amps.py` with a known-answer check against
  one committed table.
