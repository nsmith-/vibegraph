---
type: Session Report
title: "L-D2 report: complex couplings on two-fermion contacts and self-conjugate dipoles bind as MadGraph's"
description: "bound_coupling returns a coupling and a sign; a charged two-fermion vertex binds its Hermitian partner's coupling times the conjugation sign, and a self-conjugate one keeps its own coupling with the sign of the rooting's projector exchange. The brief's self-conjugate rule (the Dirac-conjugation sign) broke five enforced SMEFTsim rows and was replaced. Eight committed toy rows fail on main and pass; every other committed table is bit-identical; card 6's subprocess σ goes from 1.668 to 1.009 of MadGraph's; HC_NLO_X0 matches MadGraph on main and with the fix."
status: draft
tags: [ufo, couplings, cp-violation, hermitian-conjugate, dim6top, madgraph-parity, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Branch `sprintl-cp-odd-couplings` off main `c095a1cf` (brief at `2210e855`), merged
with `origin/main` `27beee97` (#213) in the final commit. Worktree
`/home/user/wt/sprintl-cp-odd-couplings`; every scratch artifact is under
`/home/user/wt/sprintl-cp-odd-couplings-mg/` (listed at the end). The final commit
deletes `imaginary-couplings-on-contact-and-self-conjugate-vertices-wrong-phase`
and removes it from the `blocked_by` of `eft-cards-with-imaginary-coefficients-unscreened`
and `tzq-eft-cpv-sigma-above-madgraph`; `half-percent-residual-after-hard-survey-134-967`
and the latter mention it in prose only (backticked slug, not a link), left as
they are. `tzq-eft-cpv-sigma-above-madgraph` is not deleted (the manager closes it
on the validator's σ run).

## Summary

- **The brief's rule is right for charged vertices and wrong for self-conjugate
  ones.** Implemented as written (every two-fermion vertex binds its partner's
  coupling times the Dirac-conjugation sign, a self-conjugate vertex being its own
  partner), it closed the dim6top-shaped toy rows and broke five enforced SMEFTsim
  rows (`bbx_to_ttx_tbdipole`, `ttx_to_gg_ctg`, `gg_to_ttx_ctg`, `gg_to_ttxh_ctg`,
  `ttx_to_wpwm_ctw`, worst flow 1.1e-2 to 8.8e-2). See "The rule".
- **The rule shipped.** A charged vertex whose terms all carry a gamma binds the
  partner's coupling times the sign relating the term's conjugate to the partner's
  term (the brief's class A, extended to the charged `FFVS` contacts). A
  self-conjugate two-fermion vertex keeps its own couplings, with sign `−1` exactly
  on a two-gamma chain whose chirality is written asymmetrically with projectors:
  that is the structure the rooting evaluates in its place. Yukawas, `Gamma5`
  chains and chirality-symmetric chains keep `+1`.
- **Committed rows.** Eight per-graph × per-helicity rows on a new authored toy UFO
  with dim6top's Lorentz structures and coupling shapes, banked from MadGraph 3.7.1:
  every one fails on main (worst graph 2.8e-3 to 1.03) and passes with the fix
  (worst graph ≤ 4.7e-15).
- **Sign pins.** Dropping the sign in `bound_coupling` fails two unit tests and all
  eight rows; dropping it in the evaluator fails all eight rows.
- **Bit identity.** Every other committed standalone table (89 rows: SMEFTsim's
  CP-odd dipole rows, CKM, MSSM, Majorana, the toy models) gives bit-identical
  flows and per-diagram amplitudes on main and with the fix.
- **Artifacts.** No artifact embeds bound couplings, and enumeration output is
  unchanged (bit-identical `diagrams.bin.zst`). A stale integrate artifact is
  refused by its amplitude probe. `ENUMERATION_VERSION` is not bumped.
- **σ.** Card 6's `g u > e+ ve b b~ d mu+ mu-` at card 6's point: 1.511 ± 0.014 fb
  against MadGraph's 1.497 ± 0.008 (1.009; main read 2.497, 1.668). Card 7's
  `b b~` channel: 5.900 ± 0.046 ab against 5.68 ± 0.035 (1.039; main 265.4, 46.7),
  with the same ~5% excess at ctXI = 0 on main and with the fix, and a χ²/dof of
  order 1e7 on both: not a converged reading (Found 2). At fixed momenta that
  channel's core agrees per flow to 5.1e-15 with the fix and 3.46 on main.
- **HC_NLO_X0** matches MadGraph per diagram on main and with the fix, which agree
  bit for bit; the HC reweight to cosa = 1/√2 agrees with the direct evaluation to
  4.3e-9 (the LHEF's printed precision) on both.
- **Licence.** Neither dim6top_LO_UFO nor HC_NLO_X0_UFO carries any licence; neither
  is vendored.

## What changed

- `vibegraph-lib/src/ufo/mod.rs`
  - `BoundCoupling { id, sign }`, returned by `UFOModel::bound_coupling`.
  - `conjugate_couplings`: any vertex with exactly two fermion slots. Self-conjugate
    vertices take the branch described below whatever their bosons. Charged vertices
    take the partner exchange wherever every term carries a gamma (scalars beside
    the vector allowed; charged `FFS` still excluded), the normal-form match allowing
    an overall sign (`form_sign`).
  - `even_chain_chirality_flipped`: the structure with `ProjM ↔ ProjP` in every term
    whose chain carries an even, non-zero number of `Gamma`s and no `Sigma`.
  - Unit tests: `a_gamma5_dipole_on_a_self_conjugate_vertex_keeps_its_sign`
    (replaces `a_self_conjugate_dipole_vertex_has_no_partner`), and on the toy model
    `a_charged_contact_binds_minus_its_partners_coupling`,
    `a_charged_contact_with_a_scalar_binds_its_partners_coupling` and
    `a_projector_written_cp_odd_dipole_term_enters_negated`.
    `the_partner_exchange_is_limited_to_gamma_currents` now asserts that SMEFTsim's
    `t b W H` contacts bind a partner whose bound value equals their own on a
    real-coefficient card.
- `vibegraph-lib/src/helas/eval/diagram_eval.rs`: `VertexTerm::from_ufo_read` takes
  the `BoundCoupling` and negates every Lorentz term's coefficient where its sign is
  `−1`. The coupling id stays the IR's coupling symbol, so the coupling's parameter
  dependence is unchanged.
- `vibegraph-lib/src/{proton.rs, reweight/poly.rs, ufo/report.rs}`: read `.id` (the
  vanishing check, the polynomial support and the parameter report do not depend
  on the sign).
- `validation/ufo/vibegraph_toy_cpodd_UFO/` (authored, MIT OR Apache-2.0; README
  entry in `validation/ufo/README.md`): `ut`, `dt`, `wt±`, `zt`, `ht`; the
  self-conjugate dipole `V_ututzt` (FFV1, FFV11, FFV9), the charged contact pair
  `V_utdtzwp`/`V_dtutzwm` (FFVV3/FFVV1, `c^I ∓ i c`), the self-conjugate scalar contact
  `V_ututzht` (FFVS6, FFVS4), the charged scalar contact pair
  `V_utdtwph`/`V_dtutwmh` (FFVS5/FFVS2), and the Yukawa `V_ututht` (Identity `−i y`,
  Gamma5 real `y_a`: the HC `X0 t t~` shape). Structures copied from dim6top's
  `lorentz.py`.
- `validation/madgraph/gen_standalone_jamps.py`: eight rows (the generation recipe);
  tables `validation/madgraph/standalone/*_toy_cpodd.json`.
- `vibegraph-lib/tests/standalone_jamps.rs`: the eight `#[test]`s and their
  `MG_GRAPH_DIAGRAMS` entries (identity pairings, read from the probe below).

## The rule

### What the brief's rule did to the enforced rows

Implemented exactly as the brief states it (self-conjugate vertex = its own partner,
sign of the Dirac conjugation by the normal form), a scratch print of every changed
binding (`ld2_print_changed_bindings`, since removed) showed it negating SMEFTsim's
self-conjugate `FFV2` (`[γ^μ, p̸]γ⁵`, real `cbBIm`/`ctGIm`-type couplings) and the
`t~ t g g` and `t~ t W- W+` contacts. The full suite:

```
$ cargo test --release -p vibegraph-lib --test standalone_jamps      (/home/user/wt/logs/ld2-sj1.log)
test bbx_to_ttx_tbdipole ... FAILED   worst flow 2.355e-2
test gg_to_ttx_ctg ... FAILED         worst flow 2.649e-2
test gg_to_ttxh_ctg ... FAILED        worst flow 8.797e-2
test ttx_to_gg_ctg ... FAILED         worst flow 4.024e-2
test ttx_to_wpwm_ctw ... FAILED       worst flow 1.127e-2, worst graph 3.746e-3
test result: FAILED. 79 passed; 5 failed; 1 ignored
```

The same run passed all four toy rows of the time. So the Dirac-conjugation sign is
not what a self-conjugate vertex needs.

### Why the self-conjugate sign is the projector exchange

The measured facts:

| vertex (model) | term | coupling | needs |
|---|---|---|---|
| `t~ t g/a/Z` (dim6top) | FFV9 `[γ^μ,p̸](P_L−P_R)` | real | `−1` (L-D1's dumps; toy rows) |
| same | FFV11 `[γ^μ,p̸]` (projectors at equal weight) | imaginary | `+1` |
| `b~ b a/Z`, `t~ t g` (SMEFTsim) | FFV2 `[γ^μ,p̸]γ⁵` | real | `+1` (enforced rows) |
| `t~ t g g` (SMEFTsim) | FFVV3 `[γ^μ,γ^ν]` | real | `+1` (enforced rows) |
| `t~ t g H` (dim6top) | FFVS4 (FFV9 beside a scalar) | real | `−1` (scratch, below) |
| `t~ t g g` (dim6top) | FFVV2 | | `−1` (scratch, below) |
| `X0 t t~` (HC), `ut ut~ ht` (toy) | Identity, Gamma5 | imaginary, real | `+1` |

FFV9 and FFV2 are the same matrix up to sign, so a rule that reads the couplings
or the Dirac conjugate cannot tell them apart. The difference is how the rooting
writes them: `chiral_correction` (`helas/eval/root_lorentz.rs`) exchanges a
projector reached through a gamma on a line read against the vertex's arrow, which
is right through one gamma and leaves the chirality of a two-gamma chain flipped,
while a `Gamma5` node is never conjugated. So for a self-conjugate vertex, which
MadGraph and this crate read as the same interaction, the coupling stays and the
sign is the one relating the projector-exchanged structure to the written one. This
is a statement about what the rooting evaluates, checked on every row above; it is
not derived from the Lagrangian. The cleaner fix would be to make the rooting
treat a projector-written and a `Gamma5`-written two-gamma chain alike; that
touches the W-dipole machinery the charged exchange relies on (Found 1).

For a charged vertex the brief's rule stands: FFV10 ↔ FFV4 with `+1` (SMEFTsim's
enforced W-dipole rows, unchanged), FFVV3 → `−`FFVV1 (L-D1's dumps and the toy
contact rows), and, extended here, FFVS5 ↔ FFVS2 with `+1` (dim6top's `t b W H`
contact and the toy scalar contact rows).

## The committed rows

Model `validation/ufo/vibegraph_toy_cpodd_UFO`, no restrict card, √s = 600 GeV, 3
RAMBO points each, every NHEL entry, MadGraph 3.7.1 (`research/refs/mg5amcnlo`).
Recipe: `VG_MODELS_DIR=<staged copy> python validation/madgraph/gen_standalone_jamps.py
<key> --work <dir>` under the madgraph env (scratch driver `/home/user/wt/logs/ld2-mg.sh`,
final banking `/home/user/wt/logs/ld2-mg4.log`, all eight regenerated from scratch on
the final model).

| row | process | classes |
|---|---|---|
| `udx_to_wpz_toy_cpodd` | `ut dt~ > wt+ zt` | A contact (FFVV1 on `[dt~ ut zt wt-]`), B dipole ×3 terms |
| `dux_to_wmz_toy_cpodd` | `dt ut~ > wt- zt` | A contact (the other half), B dipole |
| `uux_to_zz_toy_cpodd` | `ut ut~ > zt zt` | B dipole on both vertices, 18 diagrams |
| `zu_to_wpd_toy_cpodd` | `zt ut > wt+ dt` | A, B on a line from an initial to a final `ut` |
| `uux_to_zh_toy_cpodd` | `ut ut~ > zt ht` | B beside a scalar (FFVS4), Yukawa 1 + γ⁵ (real) |
| `hu_to_zu_toy_cpodd` | `ht ut > zt ut` | the same, crossed |
| `udx_to_wph_toy_cpodd` | `ut dt~ > wt+ ht` | charged FFVS contact (FFVS2 bound), Yukawas |
| `dux_to_wmh_toy_cpodd` | `dt ut~ > wt- ht` | the other half |

**Fail on main.** Production source reverted to `c095a1cf`, tests and tables as
committed (`/home/user/wt/logs/ld2-toy8.sh`, log `…-mg/ab/main_toy8.log`):

```
$ cargo test --release -p vibegraph-lib --test standalone_jamps toy_cpodd -- --nocapture --test-threads=1
dux_to_wmh_toy_cpodd: 36 flow entries, G = (-0.882130, -0.470509), worst flow 3.838e-3; 108 graph entries, worst graph 2.962e-3
dux_to_wmz_toy_cpodd: 108 flow entries, G = (0.939548, 0.311521), worst flow 1.260e-1; 540 graph entries, worst graph 1.102e-1
hu_to_zu_toy_cpodd: 36 flow entries, G = (0.0, 0.835121), worst flow 5.549e-1; 504 graph entries, worst graph 5.559e-1
udx_to_wph_toy_cpodd: 36 flow entries, G = (0.882151, -0.470532), worst flow 3.685e-3; 108 graph entries, worst graph 2.839e-3
udx_to_wpz_toy_cpodd: 108 flow entries, G = (-0.945376, 0.288562), worst flow 1.053e-1; 540 graph entries, worst graph 9.652e-2
uux_to_zh_toy_cpodd: 36 flow entries, G = (0.0, -0.822226), worst flow 5.590e-1; 504 graph entries, worst graph 5.572e-1
uux_to_zz_toy_cpodd: 108 flow entries, G = (0.0, 0.032894), worst flow 9.675e-1; 1944 graph entries, worst graph 1.033e0
zu_to_wpd_toy_cpodd: 108 flow entries, G = (0.928313, -0.358642), worst flow 7.526e-2; 540 graph entries, worst graph 1.446e-1
test result: FAILED. 0 passed; 8 failed
```

The probe (`probe_graph_ratios`) on main shows the shape per class: the contacts
alone off by a pure phase (`udx_to_wph`: ratio (−0.8824, −0.4706)/i against the rest
at +i), the dipole's CP-odd diagrams and the scalar contact's at the opposite sign
(`uux_to_zh` diagrams 1, 6, 7, 10, 13 at −i against +i).

**Pass with the fix** (`…-mg/ab/fix_toy8.log`):

```
dux_to_wmh_toy_cpodd: G = (1.3e-18, 1.0), worst flow 4.960e-16, worst graph 4.673e-16
dux_to_wmz_toy_cpodd: G = (-5.8e-19, -1.0), worst flow 2.707e-16, worst graph 2.737e-16
hu_to_zu_toy_cpodd: G = (3.3e-20, -1.0), worst flow 1.717e-16, worst graph 1.728e-16
udx_to_wph_toy_cpodd: G = (-9.0e-18, 1.0), worst flow 5.521e-16, worst graph 5.646e-16
udx_to_wpz_toy_cpodd: G = (-3.7e-17, -1.0), worst flow 2.844e-16, worst graph 3.253e-16
uux_to_zh_toy_cpodd: G = (0.0, 1.0), worst flow 2.288e-16, worst graph 2.309e-16
uux_to_zz_toy_cpodd: G = (-1.8e-18, -1.0), worst flow 3.166e-15, worst graph 4.721e-15
zu_to_wpd_toy_cpodd: G = (3.9e-17, 1.0), worst flow 6.048e-16, worst graph 5.295e-16
test result: ok. 8 passed; 0 failed
```

Every diagram of every row pairs one to one with its MadGraph graph at overlap
1.000000000000 and ratio ±i (`…-mg/ab/ext_fix3.log`, `probe_wph_fix.log`).

## The sign pins

`/home/user/wt/logs/ld2-mut.sh`, log `/home/user/wt/logs/ld2-mut.log` (at the
six-row stage; the two charged-scalar rows were added after, and fail on main as
shown above):

```
== mutation a: evaluator drops the sign
-                if coupling.sign < 0 {
+                if false && coupling.sign < 0 {
test result: FAILED. 0 passed; 6 failed    (every *_toy_cpodd row)
== mutation b: bound_coupling drops the sign
-            Some(couplings) => couplings[&key],
+            Some(couplings) => BoundCoupling { sign: 1, ..couplings[&key] },
test ufo::tests::a_charged_contact_binds_minus_its_partners_coupling ... FAILED
    left: BoundCoupling { id: CouplingId(6), sign: 1 }  right: BoundCoupling { id: CouplingId(6), sign: -1 }
test ufo::tests::a_projector_written_cp_odd_dipole_term_enters_negated ... FAILED
    V_ututzt#3 FFV9  left: BoundCoupling { id: CouplingId(4), sign: 1 }  right: … sign: -1
test result: FAILED. 214 passed; 2 failed
test result: FAILED. 0 passed; 6 failed    (every *_toy_cpodd row)
restored; (git diff --stat HEAD -- vibegraph-lib/src empty)
```

## Bit identity on real-coefficient rows

A scratch test (`ld2_bit_dump`, since removed) wrote, for every committed standalone
table, the full set's flows and every diagram's amplitude (single-flow tables) at
every banked point and helicity as `f64::to_bits` hex. Main (source reverted to
`c095a1cf`) against the fix (`/home/user/wt/logs/ld2-ab.sh`, `ld2-fixrun.sh`):

```
$ grep -v toy_cpodd main.txt | sha256sum ; grep -v toy_cpodd fix3.txt | sha256sum
b343dcc7b6e8634a02443529e9863b8b013713ba63bf8fdd91b480f323e78d1b  -
b343dcc7b6e8634a02443529e9863b8b013713ba63bf8fdd91b480f323e78d1b  -
$ diff main.txt fix.txt | grep '^[<>]' | cut -d' ' -f2 | sort | uniq -c
    216 dux_to_wmz_toy_cpodd   216 udx_to_wpz_toy_cpodd   216 uux_to_zz_toy_cpodd   216 zu_to_wpd_toy_cpodd
```

89 tables, 9471 (table, point, helicity) lines, identical. They include every row
L-D1 named as touched: the SMEFTsim W-dipole rows (`bg_to_twm_ctw`, `gb_to_twm_cbw`,
`bxg_to_txwp_ctw`, `vlep_to_tbx_ctwim`, `vlep_to_tbx_cbwim`, `ttx_to_wpwm_ctw`), the
neutral dipole rows (`bbx_to_ttx_tbdipole`, `tbx_to_vlep_tbdipole`, `bbx_to_epem_cbb`,
`bbx_to_epemg_cbb`, `ttx_to_epem_ctb`, `*_ctg`), the CKM-using rows, the MSSM rows
(`ttx_to_hphm_mssm`, the neutralino rows) and every Z/γ current row. Note that the
SMEFTsim dipole rows carry nonzero CP-odd coefficients (`restrict_vg_tbdipole`), so
"bit-identical" there is a statement that the shipped rule leaves Gamma5-written
CP-odd terms alone, not that their factor is 1.

The bound values do change on four SMEFTsim charged contacts with real `ctWIm`-type
couplings (`V_587`, `V_593`, `V_627`, `V_633`, FFVV2/FFVV4, value negated) and on the
`t b W H` contacts; no committed table reaches them. On a real-coefficient card the
bound value equals the vertex's own (unit test above).

## Artifact staleness

Measured, not assumed:

- **What artifacts embed.** `DiagramArtifact` and `IntegrateArtifact` embed the
  restricted parsed model (the UFO's own vertex → coupling map, unchanged here) and
  the diagram sets (interaction indices); `bound_coupling` runs at compile time on
  the reading build. No serialised structure carries a `BoundCoupling` or a bound
  coupling id (`grep` over the `Serialize` types: the only `CouplingId`s are the
  UFO's own).
- **Enumeration output is unchanged.** `vibegraph enumerate` of the toy card with the
  main and fix binaries: `cmp enum_main/diagrams.bin.zst enum_fix2/diagrams.bin.zst`
  → identical (and `diagrams.json`). The pinned known-answer test
  `enumeration_output_matches_the_pinned_version` passes in the hermetic run.
- **A stale integrate artifact is refused.** L-D1's main-built card-6 grid
  (`/home/user/wt/sprintl-dim6top-mg/sigma/gp_chain/grid.bin.zst`), `generate` with
  each binary:

  ```
  vg-main: wrote …/art/c6_main.lhe  rc=0
  vg-fix2: ERROR |M|² of set 0 (g u > e+ ve b b~ d mu+ mu-) at amplitude probe point 0 is
    4.5913068289960884e-26 in this build and 5.551157078415146e-26 in the artifact, a relative
    deviation of 1.729e-1 against a tolerance of 1e-10: this build computes a different matrix
    element from the one the cross section and grids were integrated with (a compiler, coupling
    or parameter change since); integrate again with this build   rc=1
  ```

  A toy `ut dt~ > wt+ zt` grid crosses between builds in both directions and gives
  σ = 10.679 ± 0.044 nb on each: with no widths in that process the CP-odd
  interference vanishes from the helicity-summed |M|² at every point, so the old
  artifact is right and its probe agrees.
- **Decision:** no `ENUMERATION_VERSION` bump. What enumeration computes did not move,
  and the one artifact whose content the fix invalidates (an integrate artifact on
  an affected process) is already refused by name.

## σ-level confirmations

Card 6, `g u > t b~ d mu+ mu- $$ w+ w- / h FCNC=0 DIM6=1, (t > w+ b …, w+ > e+ ve)`,
L-D1's command with the fix binary at `44459993` (`/home/user/wt/logs/ld2-sigma.sh`;
the later commits change only `FFVS` and `t~ t g g` bindings this `/ h` subprocess
does not reach):

```
$ vg-fix integrate chain.dat --ufo-dir …/sprintl-dim6top-mg/ufo --param-card param_card_card6.dat \
    --run-card run_card_gp.dat --target-rel 0.02 --no-network
 INFO VEGAS:    834609 evals over 6 iters, seed 20260719 (χ²/dof = 462.691)
σ = 1.511 ± 0.014 fb
```

Against MadGraph 3.7.1's 1.497 ± 0.008 fb: 1.009 (pull +0.87 on the combined error).
Main read 2.497 ± 0.032 (1.668). One seed; the iteration χ²/dof is L-D1's run's
character too.

Card 7, `b b~ > t t~ mu+ mu- / h …` with decays, L-D1's command and cards, fix binary
at `c54ac821` (`/home/user/wt/logs/ld2-fix2.sh`):

| case | vg fix | vg main (L-D1) | MG 3.7.1 | fix/MG |
|---|---|---|---|---|
| card 7 | 5.900 ± 0.046 ab (χ²/dof 9.1e6) | 265.4 ab | 5.68 ± 0.035 ab | 1.039 |
| ctXI = 0 | 4.027 ± 0.028 ab (χ²/dof 1.4e7) | 4.027 ± 0.028 ab | 3.812 ± 0.020 ab | 1.056 |

The ctXI = 0 reading is bit-identical to main's, as it must be, and carries the
same ~5% excess and the same χ²/dof, so the residual is not this defect and the
reading has not converged (Found 2). The fixed-momenta evidence for this channel:
the core `b b~ > t t~ mu+ mu- / h QCD<=2 QED<=4 DIM6<=1` (117 graphs, 2 flows, 3 points,
zero widths, card 7's MadGraph param card), per flow per helicity:

```
fix:  bbx_to_ttxmm_c7: 384 flow entries, G = (3.4e-16, -1.000000000000001), worst flow 5.097e-15, worst |M|² 2.682e-15
main: bbx_to_ttxmm_c7: 384 flow entries, G = (-0.268, 1.094), worst flow 3.458e0, worst |M|² 1.331e1
```

## L-D1's card-6 subprocess, per diagram

L-D1's probe (`harness/ld1_probe.rs`, copied into `vibegraph-lib/tests/` for the run
and deleted) on its MadGraph 3.7.1 tables, main and the fix at `44459993`, fitted
with its `ld1_fit.py` / `ld1_classsum.py` (`…-mg/ab/fit_{main,fix}_*.txt`):

```
fix gu_w:  bijection True; JAMP fit G = +4.3e-14 − 1.000000000000000i, worst 7.20e-15
  contact FFVV (V_382 V_402 V_349 V_369): n=4, min overlap 1.000000000, worst resid 7.1e-14, worst |r-(-i)| 0.0e+00
  self-conjugate dipole term #1 (V_189 V_49 V_51): n=19, min overlap 1.000000000, worst resid 7.5e-14, worst |r-(-i)| 0.0e+00
  every other diagram: n=52, min overlap 1.000000000, worst resid 6.4e-14
  per point eval_m2 / sum_h MG m2: 1.000000000000 (all 10)
fix gd_w (charge conjugate): bijection True; JAMP worst 1.85e-14; classes as above, worst resid 4.4e-14; eval_m2 ratio 1.000000000000 (all 10)
main gu_w: bijection False; JAMP fit G = −0.0716 − 0.9443i, worst 1.49e-01
main gd_w: bijection False; JAMP fit G = −0.0270 − 0.9830i, worst 1.63e-01
gu_noI_w (ctXI = 0): main and fix both JAMP worst 7.90e-15; the two dumps' 778252 floats are equal
```

## dim6top beyond L-D1's subprocess, and HC

Scratch tables banked with the repository's generator under MadGraph 3.7.1, every
width zero (`/home/user/wt/sprintl-cp-odd-couplings-mg/harness/ld2_gen_ext.py`,
driver `/home/user/wt/logs/ld2-extmg.sh`), compared per diagram × per helicity by
`probe_graph_ratios` (overlap and complex ratio over every entry), main and fix
(`…-mg/ab/ext_{main,fix,fix3,main3}.log`):

| table | vertices it exercises | main | fix |
|---|---|---|---|
| `ttx_to_hg_d6` (dim6top, card 6) | V_110 FFS 1+γ⁵ (ctpI), V_188 `t~ t g H` FFVS4, V_189 | V_188 diagram overlap 0.227; V_189#1 diagrams 0.209, 0.227 | all 7 at overlap 1, ratio +i |
| `ttx_to_ag_d6` | V_49, V_189 dipoles | 4 of 6 at overlap 0.59–0.76 | all 6 at 1, ratio −i |
| `gg_to_ttx_d6` (2 flows) | V_189, V_199 `t~ t g g` | worst flow 1.603e-1 | worst flow 9.888e-16 |
| `tbx_to_wph_d6` | V_45 `b~ t W- H` FFVS2 | contact ratio (−0.7239, +0.6899) | all at 1, ratio +i |
| `bbx_to_ttxmm_c7` (card 7) | V_349/V_369/V_382/V_402 contacts, dipoles | worst flow 3.458 | worst flow 5.1e-15 |
| `ttx_to_x0a_hc` (HC, card 4 cpmix) | V_3010 FFS1 + FFS5 | 4 of 4 at 1, ratio +i | the same, bit-identical |
| `ttx_to_x0g_hc` | V_3010 | 2 of 2 at 1, ratio +i | the same |

`ttx_to_hg_d6` is the V_110 check the planner asked for: its V_110 diagrams agree
on main and with the fix, so an FFS 1+γ⁵ pair with a real γ⁵ coupling binds
correctly as written; the failing diagrams there are V_188 and V_189#1, fixed.

**HC.** Card 4's point with cosa = 0.7071068 (`hc/param_card_card4_cpmix.dat`, kHtt =
kAtt = 1, ymt = −172.5), UFO `hc/HC_NLO_X0_UFO` converted to Python 3 by MadGraph's
`auto_convert_model`, `restrict_default.dat` on both sides (what a bare `import model`
applies). Both match, and they match for a reason: the X0 Yukawas are chains with no
gamma, which the rooting never exchanges, so the vertex is evaluated as written and
its own couplings are right; the rule's self-conjugate branch gives them `+1`. So card
4 is not touched by this defect class, on main or at v0.2.2. The toy rows
`uux_to_zh`/`hu_to_zu` commit the same shape (Identity `−i y` beside Gamma5 real `y_a`).
I ran `t t~ > x0 a` and `t t~ > x0 g` (single-flow, so every diagram pairs with a
graph) rather than `g g > x0 t t~`; the vertex and its couplings are the same.
Hashes of the HC inputs: `…-mg/hc_sha256.txt`.

**HC reweight.** `t t~ > x0 g` at fixed 350 + 350 GeV beams, 20 events at card 4's
base point (cosa = 1), `change helicity True`, `launch --rwgt_name=cpmix / set
FRBLOCK 2 0.7071068`, with each binary (`/home/user/wt/logs/ld2-rw.sh`). The reweighter's
own audit: `20 events weighted in their own helicity; … largest relative deviation
0.0e0`. The two files' events and weights are byte-identical (only the header's
artifact path differs). A scratch test (`ld2_rw_check`, since removed) evaluated, at
each event's momenta and helicities, `|M_h|²(cosa = 1/√2) / |M_h|²(cosa = 1)` with this
crate's evaluator (shown above to match MadGraph per diagram at cosa = 1/√2) and
compared with `wgt / XWGTUP`:

```
main: worst relative deviation 4.334e-9
fix2: worst relative deviation 4.334e-9
hel [-1, -1, 0, -1]: reweight ratio 1.403081469529e0  direct ratio 1.403081463449e0  rel 4.33e-9
```

4.3e-9 is the LHEF's eight printed digits.

**SM_alp_UFO** is not in the submodule (`research/refs/mg5amcnlo/models` has no `alp`,
`HC` or `dim6top` directory) and was not staged; I did not run it. By construction
its single-structure FFS vertices bind as on main: a self-conjugate Yukawa has no
gamma, and `conjugate_couplings` returns its own coupling with sign `+1`.

## Licences

- **dim6top_LO_UFO.** The tarball (`dim6top_LO_UFO.tar.gz`, sha256
  `04ceebc7…`) has no licence file and no licence or copyright line in any file
  (`grep -ril 'licen\|copyright'`: none). The FeynRules model page
  (`feynrules.irmp.ucl.ac.be/wiki/dim6top`, fetched over http, saved as
  `…-mg/dl-lic/fr_dim6top.html`, sha256 `b3ed1100…`) states no licence; it asks users to
  "refer to arXiv:1802.07237 … when using this model" and names Durieux and Zhang as
  contacts. The paper is CC BY 4.0 (arXiv abstract page), which covers the note, not
  the model files. No licence, so not vendored.
- **HC_NLO_X0_UFO.** No licence text in the directory (its README lists authors and
  a changelog); the FeynRules HiggsCharacterisation page states none either. Not
  vendored.
- Hence the authored toy model, which copies six Lorentz structure strings from
  dim6top (`FFV9`, `FFV11`, `FFVV1`, `FFVV3`, `FFVS2/4/5/6`). Those strings are the
  Dirac algebra of standard operators as FeynRules writes them; whether copying them
  needs the authors' permission is a judgement I flag rather than make.

## Gates

Driver `/home/user/wt/logs/ld2-gates.sh` at `773ffdce` (the final source; the
commits after it are the merge and kb files), log `/home/user/wt/logs/ld2-gates.log`,
per-gate logs in `/home/user/wt/sprintl-cp-odd-couplings-mg/gates/`. Every cargo
command under `flock /home/user/wt/heavy.lock`, own `CARGO_TARGET_DIR`.

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | rc=0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | rc=0 |
| clippy, extended | `cargo clippy --workspace --all-targets --features extended-validation -- -D warnings` | rc=0 |
| hermetic | `cargo test --workspace` with `research/refs/mg5amcnlo` emptied, `validation/madgraph/output` and the two `validation/pdf` sets moved to `…-mg/aside/` (restored after: `ls` of all three printed) | rc=0; 79 test binaries, 2388 passed, 0 failed, 28 ignored; the 8 `*_toy_cpodd` rows, the 4 new unit tests and `enumeration_output_matches_the_pinned_version` among them |
| `amplitude_oracle` | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test amplitude_oracle` | 51 passed, 0 failed |
| `coupling_oracle` | same, `--test coupling_oracle` | 50 passed, 0 failed |
| `validate_helas` | same, `--test validate_helas` | 1 passed |
| `standalone_jamps` | same, `--test standalone_jamps` | 102 passed, 0 failed, 1 ignored |
| `reweight_mg_oracle` | same, `--test reweight_mg_oracle` | 1 passed |
| `smeftsim` | same, `--test smeftsim` | 14 passed |
| `cli_reweight_proton` | `cargo test -p vibegraph --profile release-debug --features extended-validation --test cli_reweight_proton -- --test-threads 1` | 3 passed |
| lib, extended | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --lib` | 1520 passed, 0 failed, 17 ignored |
| kb-lint | `pixi run kb-lint` (after the merge and the item deletion) | 1468 concepts, 525 backlog items; 0 errors, 10 warnings (all on backlog files this PR does not touch beyond `tzq-eft-cpv-sigma-above-madgraph`'s length warning, which #213 introduced) |

`pixi run validate` was not run (CI's banked job runs it).

## What the tests cannot detect

- **The toy rows** are per graph × per helicity at three RAMBO points against
  MadGraph: blind to a phase shared by every diagram of a row, and to anything at a
  kinematic corner the points miss. They pin MadGraph's binding, which is the
  project's convention; they say nothing about whether MadGraph's sign of a CP-odd
  observable is physical (L-D1's Found 5 stands).
- **The self-conjugate sign rule** is pinned on projector-written and Gamma5-written
  two-gamma chains, symmetric chains and Yukawas. A self-conjugate vertex with a
  chain the measured classes do not cover (an even chain mixing `Gamma5` with
  projectors, a `Sigma` chain beside a gamma, a four-gamma chain) is bound by the
  rule's formula without a row behind it.
- **Charged vertices**: the partner exchange is pinned for FFV (SMEFTsim), FFVV and
  FFVS (toy, dim6top scratch) with projector-written chirality. A charged
  `Gamma5`-written two-gamma chain is unmeasured; by the self-conjugate finding its
  rooting differs from the projector form, so the Dirac-conjugate match may be wrong
  for it (Found 1).
- **Bit identity** covers the committed standalone tables only. Rows reaching
  SMEFTsim's charged contacts (`V_587` …) or `t b W H` vertices with CP-odd
  coefficients changed value and have no MadGraph row.
- **σ**: one seed each, against single MadGraph runs; card 7's reading is not
  converged.
- **The reweight check** shares the evaluator with the integrand, so it tests the
  reweight plumbing against the per-diagram evidence, not independently of it.

## Found

1. **The rooting treats a projector-written and a `Gamma5`-written two-gamma chain
   differently.** Evidence: dim6top's `[γ^μ,p̸](P_L−P_R)` and SMEFTsim's
   `[γ^μ,p̸]γ⁵` need opposite coupling signs on self-conjugate vertices against
   MadGraph (table in "The rule"); `chiral_correction` exchanges gamma-chained
   projectors whatever the chain length, while `Gamma5` nodes are never conjugated.
   The binding now compensates by sign. What would close it: make the two writings
   root identically (one normal form for the chain's chirality before rooting), then
   reduce the self-conjugate branch to `+1` and check the charged exchange still
   holds, against the toy and SMEFTsim rows; plus a charged `Gamma5`-written
   two-gamma row, which nothing measures today.
2. **Card 7's `b b~` channel reads ~5% above MadGraph with an iteration χ²/dof of
   1e7, with or without ctXI.** Evidence: 4.027 ± 0.028 ab against 3.812 ± 0.020 at
   ctXI = 0 (main and fix identical), 5.900 ± 0.046 against 5.68 ± 0.035 with them;
   per-flow agreement at fixed momenta is 5.1e-15. What would close it: a converged
   σ (budget ladder, ≥5 seeds) on that channel to say whether the 5% is sampling or
   integrand; the χ²/dof says the VEGAS iterations disagree far beyond their errors.
3. **SMEFTsim charged contacts with CP-odd coefficients have no MadGraph row.**
   `V_587`, `V_593`, `V_627`, `V_633` (t b W γ/Z, FFVV2/FFVV4) and `V_290`/`V_329`
   (t b W H) bind differently now wherever `ctWIm`/`cbWIm` ≠ 0. The toy rows cover
   the same structures written the same way; a SMEFTsim row (e.g. `b g > t W- a`
   under `restrict_vg_ctWIm`) would pin it in that model.
4. **L-D1's banking harness misses MadEvent's width zeroing only where
   `zero_widths` is off**: my first HC/dim6top scratch tables kept standalone's
   spacelike top widths and showed ~1e-3 per-diagram phase residuals; with
   `zero_widths` they vanish. No committed row is affected (the toy model has no
   widths). Worth a line in the generator's docs that a massive t-channel fermion
   row needs `zero_widths`.

## Corrections to the brief

- **The self-conjugate half of the rule is wrong as stated.** "A self-conjugate
  vertex is its own partner … the FFV9 term is negated" is true for FFV9 because of
  how dim6top writes it, not because of conjugation: the same rule applied by the
  Dirac-conjugation sign negates SMEFTsim's Gamma5-written dipoles and its
  `[γ^μ,γ^ν]` contacts and breaks five enforced rows. The shipped rule keeps the
  self-conjugate coupling and takes the sign of the rooting's projector exchange.
  The item's `closes_when` sentence ("a self-conjugate vertex is its own partner")
  is met in that sense only.
- **"With real coefficients the correction factor is exactly 1"** holds for the
  charged exchange (the hermiticity relation makes `−conj(own)` the bound value) but
  not for the self-conjugate one as the brief proposed it: it would have negated
  SMEFTsim's real CP-even `t~ t g g` coupling.
- **Scope.** Fixing the classes the brief names left the same defect on dim6top's
  `t~ t g H` and `t b W H` contacts (FFVS), reached by any subprocess without `/ h`
  (card 7's `/ h` and card 6's exclude them). Both are now covered; the brief had
  them out of scope ("FFVS stay excluded").
- **dim6top could not be vendored**, so the "use card 6's point on `g u > t b~ d mu+
  mu-`" row is replaced by toy rows with dim6top's structures; card 6's subprocess is
  covered by the σ and by L-D1's per-diagram dumps rerun with the fix ("L-D1's card-6 subprocess").
- The brief's discipline line "Run gates with `--skip-deps`" and "do not run
  `pixi run validate`" were followed; the extended-validation targets were run
  directly with cargo.

## Long-run requests

None required. Card 7's converged σ (Found 2) would be one if the manager wants it:
branch `sprintl-cp-odd-couplings` at the final commit, card 7's `bb/sub.dat`,
`param_card_card7_validator{,_noI}.dat`, `run_card_c7.dat`, lhaid 247000, 5 seeds at
two budgets; expected ≈ 1.00 of MadGraph's 5.68 ab (and 3.812 ab at ctXI = 0) if the
5% is sampling.

## Files (scratch, none committed)

Under `/home/user/wt/sprintl-cp-odd-couplings-mg/`: `standalone/` (MadGraph work
dirs of the toy rows), `standalone_ext/` and `tables/` (HC, dim6top and card-7
scratch tables), `models/`, `models_ext/` (staged toy, HC, dim6top copies),
`harness/ld2_gen_ext.py`, `ab/` (bit dumps, probe and mutation logs), `art/`
(artifact runs), `rw/` (reweight runs), `sigma/`, `c7/`, `dl-lic/` (licence pages),
`bin/` (`vg-main`, `vg-fix`, `vg-fix2` binaries). Drivers and logs:
`/home/user/wt/logs/ld2-*`.
