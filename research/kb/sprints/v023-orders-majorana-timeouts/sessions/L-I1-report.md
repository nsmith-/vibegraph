---
type: Session Report
title: "L-I1 report: the VLQ 612 interference sign is the released chirality bug, fixed on main"
description: "Per diagram, per helicity and per flow, g u > h t b~ d / bp x y VLQ^2==2 on main matches MadGraph 3.7.1 standalone to 1e-15 in both classes (G2/G0 = 1), with and without widths and near the T' pole. The campaign binary 9f196713 gets the five VLQ=2 diagrams wrong: it lacks a5bf8fca's projector exchange on the non-self-conjugate T' t H vertex, and disabling that exchange on main reproduces it bit for bit. On main the reduced 612 σ is negative, −937 ± 27 ab; helicity pruning changes nothing."
status: draft
tags: [vlq, squared-orders, interference, chirality, oracle, madgraph-parity, cms-gridpacks, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Worktree `/home/user/wt/sprintl-interference-sign`, branch `sprintl-interference-sign`,
head `e6ba95ea` (main `3f4dd815` plus the brief). No source change is committed: the
only commit is this report. Every scratch artifact is under
`/home/user/wt/sprintl-interference-sign-mg/` (listed at the end). The item
`squared-order-interference-sign-opposite-to-madgraph` has no file on main (it was
filed on kb PR #207), so there is nothing to delete here. Whether it closes is the
planner's decision: see "What closes the item".

## Summary

- **The relative-sign hypothesis is false.** On main, G2/G0 = 1.000000000000 with
  every residual at 1e-15, on 4 generic points and 6 points near the T' pole, with
  widths zero and with widths.
- **The defect is in the binary the campaign ran.** vg-validator ran bin-pr1
  `9f196713`, which does not contain `a5bf8fca` (the charged-scalar chirality fix,
  PR #169). On `9f196713` the 14 VLQ=0 diagrams match MadGraph to 1e-15 and all 5
  VLQ=2 diagrams are wrong. They are wrong in chirality, not by a sign: G2 = −0.31i,
  with residual 0.39 of the largest entry. The 5 diagrams share one vertex, `V_24`
  (`t~ tp H`, `FFS2` = bare `ProjM(2,1)`, coupling `GC_19`). It is non-self-conjugate,
  so it is the class `conjugate_standalone_projectors` exchanges.
- **Attribution is exact.** On main with that exchange disabled (a scratch patch),
  every per-diagram value and every `eval_m2` equals `9f196713`'s bit for bit.
- **σ on main is negative.** The reduced-budget 612 σ is −937 ± 27 ab at seed 1.
  `9f196713`'s iterations at the same seed and cards are positive (+1.3e-12 to
  +2.6e-12 in internal units). That run was then refused for its effective sample size.
- **Helicity pruning is exact.** The pruned and unpruned runs agree in every printed
  digit of every iteration.

## Provenance

| item | sha256 |
|---|---|
| UFO manifest as found (`vlq_manifest_local.txt`, before any MadGraph import) | `83bdef2cf364f229bf8ed5978bb8ba53f0c0650d06417d86763d1b6f171ce548` |
| validator's manifest (`vlq_manifest_validator.txt`) | `2184d0099639cf49d550457b19e4c4af81f373b8add0c3b680cfa54d6855e9f8` |
| `restrict_3rdL_noCKM.dat` (same as the validator's `0459cb74…`) | `0459cb74199afbf3d97a7fb1d54fef1c96561c4456827cb2900e98458db45504` |
| `param_card_612.dat` (transcribed; no QNUMBERS) | `0032ecaff96d5129da25247f4be7b552ef0b33706ab9d5631187e7ec950d8d31` |
| card both sides evaluated, widths (`param_card_edited.dat`) | `0168ffd119a13693b3c7a3c450a816be72cb6ad55fe61a0b2e05d77eb86b9446` |
| the same, every width zero | `70a5dc17c62b4fa937f170a76e4e829f8d0f79d0c8c4f34c2e2daceede8f99da` |
| MadGraph | 3.7.1, a copy of the submodule at `b7687064b9a013317ca164aa1395bc9c0e39ae1e` (`mg371/`) |

The amplitude files, as hashed after the run (`li1_bank.py`'s `sha256` lines), are
byte for byte the local manifest's. The manager's comparison found them equal to the
validator's.

| file | sha256 |
|---|---|
| `couplings.py` | `22615b0a…40d` |
| `coupling_orders.py` | `711bf997…488` |
| `lorentz.py` | `dea3535e…bc4b` |
| `parameters.py` | `6368cbc9…c99` |
| `particles.py` | `21733e48…201` |
| `vertices.py` | `6fc97e6c…3b2` |
| `propagators.py` | `b383d12a…3f3` |
| `decays.py` | `3ef36151…c11` |
| `function_library.py` | `596e28fe…510c` |
| `CT_*.py` | `e9385a8d…`, `c7e8c904…`, `5ece599a…` |

**Why three files differ: MadGraph rewrites the UFO when it imports it.** 3.7.1's
import log (`/home/user/wt/logs/li1-gen_split.log`, lines 48–49) says:

```
fail to load model but auto_convert_model is on True. Trying to convert the model
convert model /home/user/wt/sprintl-interference-sign-mg/models/VLQ_v4_4FNS_UFO
```

It rewrote three files in place (timestamps 02:15; the rest are 02:11):

| file | before import | after import | validator |
|---|---|---|---|
| `__init__.py` | `7c018f93…` | `6cf59aa8…` | `6cf59aa8…` |
| `object_library.py` | `f1afd9b5…` | `edb49c54…` | `edb49c54…` |
| `write_param_card.py` | `2a127d3a…` | `38ef361…` | `b1fefa31…` |

After the import, `__init__.py` and `object_library.py` equal the validator's. So the
validator's staging is a MadGraph-converted copy. `write_param_card.py` still differs,
probably because a different MadGraph version converted it. All three files are
infrastructure. This crate's model loader read the copy after conversion.

**The 612 card is consistent with the full card.** Both sides evaluated the restricted
model's own `Cards/param_card.dat` with two edits: MTP 800 and WTP 160. The banking
script asserts that all 58 entries of `param_card_612.dat` equal that card:

```
card 612: all 58 entries equal on the edited card .../param_card_edited.dat
entries only on the edited card (restricted-model defaults): [(('sminputs', 2), 1.16637e-05),
  (('yukawa', 5), 4.7), (('yukawa', 6), 172.0), (('yukawa', 15), 1.777)]
```

The transcribed card lacks Gf and the `YUKAWA` block, as well as `QNUMBERS`. The model
defaults fill them, and they are numerically what the card implies: ymb = mb, ymt = mt,
and MW = 79.82436 follows from Gf.

## Banking (MadGraph 3.7.1 standalone)

**`gen_standalone_jamps.bank()` cannot carry the split.** Under a squared-order
selection, standalone `MATRIX` is a subroutine `MATRIX(P,NHEL,IC,RES(NSQAMPSO))` and
JAMP is `JAMP(NCOLOR,NAMPSO)`. `bank()`'s declaration regex and its `MATRIX(...)`
driver call both assume the unsplit form. The scratch driver `harness/li1_bank.py`
does the following:

- reuses `bank()`'s point generator, card editor and NHEL reader;
- patches `MATRIX` to publish `AMP(NGRAPHS)` and `JAMP(NCOLOR,NAMPSO)` through a COMMON
  block;
- dumps `RES(1..3)`, the squared orders VLQ² = 0, 2 and 4;
- parses the per-class JAMP coefficients.

One output carries both classes, so separate VLQ=0 and VLQ=2 outputs are not needed.

**The width convention comes from MadEvent's own output.** `output madevent` of the
same process (generation only, nothing launched) writes the per-line widths. The
driver copies its `ZERO`s onto the standalone calls, line by line. Eight propagators
are zeroed, the same set `PropInfo::t_channel` selects. Two of them are the spacelike
T' lines: diagram 17's T' after the gluon, and diagram 5's T' (t + g, then h).

```
cd /home/user/wt/sprintl-interference-sign-mg/sa
bash .../validation/madgraph/mg5_pinned.sh gen_split.mg5      # generate g u > h t b~ d / bp x y VLQ^2==2; output standalone
  INFO: Process has 19 diagrams
bash .../validation/madgraph/mg5_pinned.sh gen_me.mg5         # same process, output madevent
  INFO: Some T-channel width have been set to zero [new since 2.8.0]
python harness/li1_bank.py                                     # widths/zero-width/resonant tables
  t-channel widths zeroed as MadEvent writes them:
    CALL FFV1_1(W(1,4),W(1,1),GC_9,MDL_MT,MDL_WT,W(1,9))     CALL FFV2_3(W(1,2),W(1,6),GC_27,MDL_MW,MDL_WW,W(1,12))
    CALL FFS3_1(W(1,9),W(1,3),GC_19,MDL_MTP,MDL_WTP,W(1,13)) CALL FFS4_1(W(1,9),W(1,3),GC_87,MDL_MT,MDL_WT,W(1,13))
    CALL FFV2_3(W(1,5),W(1,9),GC_27,MDL_MW,MDL_WW,W(1,13))   CALL FFV2_3(W(1,9),W(1,4),GC_27,MDL_MW,MDL_WW,W(1,13))
    CALL FFV1_1(W(1,8),W(1,1),GC_9,MDL_MTP,MDL_WTP,W(1,6))   CALL FFV1_1(W(1,7),W(1,1),GC_9,MDL_MT,MDL_WT,W(1,6))
  wrote .../li1_gu_htbxd_zw.json: 4 pts x 32 hels x 2 flows x 2 classes; Σ RES(VLQ^2=0,2,4) = [3.1388e-09, 3.2398e-10, 4.2243e-11]
  wrote .../li1_gu_htbxd_w.json:  ...                                     = [3.1392e-09, 2.8354e-10, 3.7864e-11]
  wrote .../li1_gu_htbxd_w_sa.json (standalone's own widths)              = [3.1395e-09, 2.8277e-10, 3.7787e-11]
LI1_RESONANT=1 python harness/li1_bank.py                      # 6 RAMBO points at 1200 GeV with |m(t h) - 800| < 80
  resonant points: m(t h) = 791.8, 794.1, 799.2, 849.5, 827.0, 721.5
  res_w:    Σ RES = [6.7727e-09, -3.9793e-12, 1.3642e-08]
  res_zw:   Σ RES = [6.7775e-09,  1.6110e-07, 1.3908e-05]
  res_w_sa: Σ RES = [6.7651e-09,  1.4744e-10, 1.3538e-08]
```

The generic points are 4 RAMBO points at √ŝ = 1500 GeV, seed 6121, with all 32 NHEL
entries. `MATRIX` puts the VLQ=0 class in `JAMP(·,1)` (14 graphs) and the VLQ=2 class
in `JAMP(·,2)` (graphs 1, 5, 11, 13 and 17). Every coefficient is −1, NCOLOR = 2 and
the CF matrix is diagonal.

The `res_w` and `res_w_sa` rows have the same card and points. Only the width on the
spacelike T', top and W lines differs between them, and Σ 2Re(M0·M2*) changes sign,
−4.0e-12 against +1.5e-10. So near the pole the interference is a cancellation that a
width convention can flip.

## The oracle

This crate's side is `harness/li1_probe.rs`. It was a test-side scratch, copied into
`vibegraph-lib/tests/` only to run it, and it is not committed. It dumps four things:

- every diagram compiled alone without the selection, through `run_flows` or
  `eval_amplitude`, with its flow mapped by relabel-invariant colour-flow tags;
- the class sets (VLQ=0 only and VLQ=2 only, no selection) through `run_flows`;
- the unsplit set;
- the production split evaluator, both through `eval_m2` and restricted to each
  helicity through the polarisation field.

`run_flows` panics on a split evaluator (`run_forward_flows_typed on a non-Flows
amplitude`), and its per-(class, flow) JAMPs are `pub(crate)`. The class sets stand in
for them, and the per-helicity split `eval_m2` checks the production assembly.
`harness/li1_fit.py` does the comparison:

- it pairs diagrams to graphs by normalised overlap over every (point, helicity, flow)
  entry;
- it fits one complex constant per diagram;
- it fits G0 and G2 on the class JAMPs;
- it compares `RES` to the contracted class JAMPs.

```
harness/run_probe.sh zw w res_w res_zw         # = flock heavy.lock cargo test --release -p vibegraph-lib --test li1_probe
python harness/li1_fit.py tables/li1_gu_htbxd_<t>.json tables/vg_<t>.json
```

The census is machine-checked: 19 diagrams on each side, 5 of them VLQ=2. The
diagrams carry `{"QCD": 1, "QED": 1, "VLQ": 2}` or `{"QCD": 1, "QED": 3}`, and each
pairs with a MadGraph graph of the same class.

### Fit on main (`e6ba95ea`)

| table | G0 | G2 | G2/G0 | worst class entry (G0 / G2) | worst per-helicity (G0 / G2) | worst per-diagram |
|---|---|---|---|---|---|---|
| widths zero, 4 generic points | `2e-15 − 1.000000000000002i` | `2e-15 − 1.000000000000000i` | `0.9999999999999987 + 3.7e-16i` | 8.5e-15 / 1.9e-15 | 2.6e-13 / 2.6e-13 | 6.7e-14 |
| widths (MadEvent convention), 4 generic | `1e-15 − 1.000000000000002i` | `2e-15 − 1.000000000000000i` | `0.9999999999999987 + 6.7e-16i` | 8.9e-15 / 1.9e-15 | 2.7e-13 / 3.3e-13 | 6.7e-14 |
| widths zero, 6 near the pole | `1e-15 − 1.000000000000002i` | `−0 − 1.000000000000089i` | `1.0000000000000868 − 6.1e-16i` | 2.9e-15 / 6.2e-15 | 2.1e-14 / 9.9e-14 | 8.0e-16 |
| widths, 6 near the pole | `1e-15 − 1.000000000000002i` | `−0 − 1.000000000000002i` | `0.9999999999999998 − 7.2e-16i` | 2.3e-15 / 6.6e-16 | 3.1e-14 / 1.3e-14 | 8.5e-16 |

How the columns are normalised:

- **Worst class entry:** the largest `|vg − G·mg|` against the largest entry of that
  class.
- **Worst per-helicity:** the same, against that helicity's own largest entry. The
  2.6e-13 sits on helicities that are tiny but nonzero.
- **Worst per-diagram:** the residual of each diagram's own fitted constant, against
  that diagram's largest value.

Every diagram's ratio is `−i` with ||r| − 1| ≤ 1e-14 and a best overlap of 1 to 12
digits. The runner-up overlap is at most 0.99996 (the mirror diagram with the gluon on
the other quark line), so the pairing is unique.

The production split evaluator, restricted to each helicity, against MadGraph's
`RES(VLQ²=2)`: fit factor 1.000000000000 on every table, worst deviation 1.2e-15
(zero widths), 9.4e-16 (widths), 6.3e-15 (widths, near the pole), 8.8e-15 (zero widths,
near the pole). The sign agrees on every helicity. `eval_m2` matches Σ_h `RES(2)` to
10 digits at every point. That includes the sign-changing points: −2.116e-11, +2.428e-11,
+2.926e-10 and −1.220e-11 (generic, widths); +2.86e-12 up to −1.155e-09 (near the pole).

The `w_sa` table keeps standalone's widths on the spacelike lines, which neither this
crate nor MadEvent does. Against it G2/G0 = 1.0010 − 0.0034i, worst G2 entry 2.7e-3,
and RES(2) differs by 0.25%. So the t-channel width convention is the only way a
width could have mattered.

### Precondition: |M0|² and |M2|²

On main, as Σ over points and helicities against MadGraph's `RES`:

| table | |M0|² ratio | |M2|² ratio | 2Re(M0·M2*) ratio | worst per-helicity |
|---|---|---|---|---|
| zw | 1.000000000000 | 1.000000000000 | 1.000000000000 | 4.0e-15 |
| w | 1.000000000000 | 1.000000000000 | 1.000000000000 | 3.8e-15 |
| res_w | 1.000000000000 | 1.000000000000 | 0.999999999994 (Σ = −3.98e-12, a cancellation) | 1.2e-14 |
| res_zw | 1.000000000000 | 1.000000000000 | 1.000000000000 | 1.8e-13 |

### Fit on the campaign binary (`9f196713`, `git archive` into `vg_9f196713/`)

The same probe was built at `9f196713`, with the diagram summary replaced, since that
commit has no `fermion_flow_summary`.

| table | G0 | G2 | G2/G0 | worst G2 entry | |M0|² | |M2|² | 2Re(M0·M2*) |
|---|---|---|---|---|---|---|
| zw | `2e-15 − 1.000000000000002i` | `0.0029 − 0.3129i` | `0.3129 + 0.0029i` | 0.39 | 1.000000000000 | 0.2686 | 0.938 (per-hel fit 0.358) |
| w | `1e-15 − 1.000000000000002i` | `0.0011 − 0.3125i` | `0.3125 + 0.0011i` | 0.39 | 1.000000000000 | 0.2699 | 0.900 (per-hel fit 0.354) |
| res_w | `1e-15 − 1.000000000000002i` | `−0.0071 − 0.5543i` | `0.5543 − 0.0071i` | 1.29 | 1.000000000000 | 1.831 | **−572** (Σ +2.28e-09 against −3.98e-12) |

How the 5 VLQ=2 diagrams (2, 5, 10, 14, 17) look at `9f196713`:

- none overlaps its MadGraph graph better than 0.51–0.98;
- their best "match" is a VLQ=0 graph, at 2–4% of its size;
- all 14 VLQ=0 diagrams are exact.

**A pure class sign does not close it.** Flipping the 5 diagrams
(`--flip=2,5,10,14,17` on the `w` dump) gives G2/G0 = −0.3125. The residual stays at
0.39, and the per-helicity sign agreement drops from 1.000 to 0.750.

### Carriers

The 5 VLQ=2 diagrams carry the defect. They share one vertex: `V_24 [t~, tp, H]`,
Lorentz `FFS2` = `ProjM(2,1)`, coupling `GC_19` = `i·KTLh3`. Its hermitian partner is
`V_15 [tp~, t, H]`, `FFS3` = `ProjP(2,1)`, the same `GC_19`. The `KTRh` halves vanish
under the restriction.

MadGraph's call for the T' → t h vertex is `FFS3_1(...,GC_19,...)`, a ProjP: the Dirac
conjugate of the ProjM that this crate's all-incoming match reads off `V_24`. That is
the case the exchange rule describes.

The other vertices in these diagrams are not touched by the exchange:

- `V_41` (`tp~ b W+`, `FFV2` = γ^μ P_L) is a chained structure;
- `V_25` (T' g) is a vector vertex;
- `V_16` and `V_18` (b and t Yukawa, `FFS4`) are self-conjugate.

**The exchange is the whole difference.** A scratch patch on main
(`harness/li1_noswap.patch`, reverted) disables the exchange:

```
- let swap = !self_conjugate_set(model, id);
+ let swap = std::env::var_os("LI1_NO_SWAP").is_none() && !self_conjugate_set(model, id);
```

```
LI1_NO_SWAP=1 LI1_SUFFIX=_noswap harness/run_probe.sh zw w res_w
zw:    main-without-exchange vs 9f196713, per diagram max rel diff: max 0.00e+00 (exact zeros: 19/19)
       main vs 9f196713 changed diagrams (rel diff > 1e-12): [2, 5, 10, 14, 17], their VLQ orders [2, 2, 2, 2, 2]
       eval_m2 main-without-exchange [-7.080513704809305e-12, 7.242355570187967e-12, 3.1168346828533886e-10, -7.931109270939716e-12]
       eval_m2 9f196713             [-7.080513704809305e-12, 7.242355570187967e-12, 3.1168346828533886e-10, -7.931109270939716e-12]
w, res_w: the same, 19/19 diagrams bit-identical, eval_m2 bit-identical at all 4 and 6 points
```

So on this subprocess everything between `9f196713` and main that touches the
amplitude is `a5bf8fca`'s exchange at `helas/eval/diagram_eval.rs:422`
(`conjugate_standalone_projectors`, `:125`). It reaches exactly the VLQ=2 class and
nothing else.

## Pruning check

One subprocess at fixed momenta (probe, `LI1_PRUNE=1`): 24 of 32 combinations are
dropped. The 8 that remain are exactly MadGraph's nonzero ones (u⁻ and d⁻ at the W,
g, t and b̄ free). `eval_m2` is bit-identical at all 10 points
(`rel +0.00e+00` at each).

The full 612 card at reduced budget, on main. The scratch `LI1_NO_PRUNE` switch is in
`hadronic::compile_class` (`harness/li1_noprune.patch`, reverted):

```
S=/home/user/wt/sprintl-interference-sign-mg/sigma; LHAPDF_DATA_PATH=$M/pdf
vibegraph integrate proc_612.dat --run-card run_card_126family.dat --param-card param_card_612_full.dat \
  --no-network --seed 1 --fixed-budget --neval 60000 --niter 6 --out out_<label> --force --no-tui
```

| run | iterations 3–6 (σ, internal units) | σ | support |
|---|---|---|---|
| pruned (production) | −2.283096e-12 ± 1.571119e-13, −2.374958e-12 ± 1.045753e-13, −2.422655e-12 ± 8.157569e-14, −2.407305e-12 ± 6.949648e-14 | **−937 ± 27 ab** | 273580 of 305530 points weighted, ESS 1191, χ²/dof 1.641 |
| unpruned (`LI1_NO_PRUNE: kept all 32 helicity combinations` ×12) | identical in every printed digit | **−937 ± 27 ab** | identical |
| `9f196713`, same cards and seed | +1.632936e-12 ± 4.4e-13, +1.430408e-12 ± 2.4e-13, +1.324158e-12 ± 1.6e-13, +2.615477e-12 ± 1.3e-12 (χ²/dof 25.2) | refused: "carried by an effective 3.8 points" | ESS 4 |

The unpruned run's ns/eval is 30–38k against 23–27k, so the switch was reached.

The run card is not 612's own. It is card 126's: the same VLQ family and generator
campaign, from `/home/user/wt/cards-sprintL/tprime126.md`, with its hidden parameters.
612's run card was not available here. This does not matter for the pruned/unpruned
comparison. It does mean −937 ab cannot be compared number to number with MadGraph's
−1101 ± 32 ab. The run is also one seed at a reduced budget. Only the sign is a
finding.

PDF: lhaid 325500 is `NNPDF31_nnlo_as_0118_nf_4_mc_hessian` (SetIndex 325500,
copied from `sprintl-vbs-mg/pdf`), not the `_pdfas` set `tprime126.md` names.

## Candidate fix

**There is nothing to fix on main.** The fix is `a5bf8fca`: a vertex whose particles
are not their own conjugate set has its bare chiral projectors exchanged, at
`helas/eval/diagram_eval.rs:422` through `conjugate_standalone_projectors`
(`diagram_eval.rs:125`). It is already on main.

**The arithmetic check is done on the dumps.**

1. Main's per-diagram values close to G2/G0 = 1 at 1e-15. The interference matches
   `RES(2)` per helicity, and in sign at every helicity.
2. Removing the exchange alone reproduces the campaign binary's values exactly, so
   the exchange is the complete difference.
3. A pure −1 on the VLQ=2 class, the brief's candidate, does not close the
   `9f196713` fit.

**The site is none of L-Z1's three signed-|M|² sites.**

- Pruning near `run.rs:719`: σ bit-identical with and without.
- The split `JAMP2` `.abs()` near `run.rs:462`: picks the colour flow only, and
  enters no σ.
- The signed helicity draw near `compile.rs:346`: events only.

### What closes the item

The sign on 612 should already be right on main and on any release that contains
`a5bf8fca`. That claim rests on one subprocess per diagram and one reduced-budget σ
with a sibling card's run card. Closing it at the σ level needs the long-run request
below.

## What each oracle cannot see

- **Per-diagram × helicity × flow against standalone (this oracle):**
  - a phase common to all 19 diagrams;
  - kinematic regions the 10 points do not visit;
  - the other 11 subprocesses of the card, which are 6 flavour groups on this side
    (`g d > H t~ b u`, `u d~ > H t b~ g`, the charge conjugates, the s/c copies);
  - anything after the matrix element: PDFs, cuts, channel weights, the sampler.
- **|M0|², |M2|² and the per-order σ partition:** a relative phase or chirality error
  confined to one class. At `9f196713` the vibegraph-internal partition ==0 + ==2 + ==4
  = VLQ<=2 held while |M2|² was wrong by 0.27× to 1.8×. Internal consistency says
  nothing about agreement with MadGraph.
- **Per-helicity `eval_m2` of the split evaluator (polarised compile):** the
  per-(class, flow) JAMPs individually. They are compared through the class sets,
  which share the compile path except for the split assembly.
- **Pruned vs unpruned σ at one seed:** pruning's effect away from the partonic CM
  frame. Integration evaluates in that frame, the kinematic contract.
- **The one reduced σ:** the size of σ_int, which needs 612's own run card, a full
  budget and seeds. It is a cancellation with ESS 1191 of 2.7e5.

## Requests for the long-run host

- **R1.** vibegraph on card 612, and on card 614, at main (`3f4dd815` or later, which
  contains `a5bf8fca`). Use the cards' own proc, run and param cards, campaign budget,
  seeds 1–5.
  - Expected: σ negative, near MadGraph's −1.101e-3 ± 3.2e-5 pb on 612.
  - Card 614 should move the same way.
  - How close is limited by MadGraph 2.9.18's `sde_strategy = 2` channel weight, which
    does not peak on a resonance ([madgraph-defects](../../../validation/madgraph-defects.md)).
    This card's interference is a cancellation around a 160 GeV-wide T' pole.
  - The same five seeds of bin-pr1 `9f196713` gave +4.779e-4 pb.
- **R2 (optional, if R1 differs from the gridpack by more than the seed spread).**
  MadGraph 3.7.1 on card 612, `hard_survey = 2`, 5 seeds. The 2.9.18 reference may be
  biased by the channel weight above.

## Gates

No production source, test or table is committed. The diff is this one Markdown file.
The scratch patches were applied only to build scratch binaries and probes, and each
was reverted with `git checkout`. `git diff --stat` after each revert was empty.

```
pixi run kb-lint
1451 concepts, 512 backlog items; 0 errors, 7 warnings   (all 7 on backlog items this commit does not touch)
```

Hermetic layer and `cargo test`: not run. The commit changes no Rust, no test target
and no table, so no cargo gate can be affected, and the probe file was removed from
`vibegraph-lib/tests/` before the commit. Release builds of the probe and of the
`vibegraph` binary on this head compiled cleanly, with one warning (below).

## Files (all under `/home/user/wt/sprintl-interference-sign-mg/`, none committed)

Harness:

- `harness/li1_bank.py`: the split-aware MadGraph banking driver;
- `harness/li1_probe.rs`: this crate's dump;
- `harness/li1_fit.py`: the fit;
- `harness/run_probe.sh`: the probe runner;
- `harness/li1_noprune.patch` and `harness/li1_noswap.patch`: the two scratch patches,
  both reverted.

MadGraph outputs:

- `sa/split/`: standalone;
- `sa/me_split/`: madevent, generation only;
- `mg371/`: MadGraph 3.7.1.

Tables:

- MadGraph: `tables/li1_gu_htbxd_{zw,w,w_sa,res_zw,res_w,res_w_sa}.json`;
- this crate on main: `tables/vg_*.json`;
- the campaign binary: `tables/vg9f_*.json`;
- main without the exchange: `tables/vg_*_noswap.json`;
- pruned: `tables/vg_*_pruned.json`.

σ runs:

- cards: `sigma/{proc_612.dat,run_card_126family.dat,param_card_612_full.dat}`;
- binaries: `sigma/vibegraph_{pruned,noprune,9f196713}`;
- artifacts: `sigma/out_{pruned,noprune}/grid.bin.zst`.

Other:

- `vg_9f196713/`: the `git archive` of the campaign commit;
- `pdf/`: the 325500 set.

Logs: `/home/user/wt/logs/li1-*.log`.

## Found

- **The standalone harness cannot bank a squared-order split.** `gen_standalone_jamps.bank()`
  assumes `JAMP(NCOLOR)` and a `MATRIX` function. Under a split, MadGraph writes
  `JAMP(NCOLOR,NAMPSO)` and `SUBROUTINE MATRIX(P,NHEL,IC,RES)`. `standalone_jamps.rs`
  compares only unsplit flows, and `run_flows` panics on a split evaluator. No committed
  gate therefore sees a per-class amplitude or a signed interference against MadGraph.
  To close it:
  - a split row type in `gen_standalone_jamps.py` (`li1_bank.py` is a working
    prototype: RES per squared order, JAMP per class, per-class coefficients);
  - a matching comparison in `standalone_jamps.rs`;
  - a committed row such as this `g u > h t b~ d / bp x y VLQ^2==2`, which would pin
    `V_24`'s exchange on a model other than MSSM and g2HDM. That needs the VLQ UFO
    vendored under `validation/ufo/`.
- **The campaign's squared-order σ rows ran a binary without the chirality fix.** Every
  vg-validator row from bin-pr1 `9f196713` predates `a5bf8fca`. So does any other
  pre-#169 binary. Those rows carry the bug on any vertex with a bare chiral projector
  and particles that are not their own conjugate set. The rows that matter are the
  VLQ cards (612, 614, the 126 family), the g2HDM cards (already found, L-G1) and
  anything with charged scalars, neutralinos or T'/B' Yukawas. To close it: re-run
  those rows on a post-#169 binary, R1 first.
- **MadGraph rewrites a py2 UFO in place on import** (`auto_convert_model`). It changed
  `__init__.py`, `object_library.py` and `write_param_card.py` of the scratch copy here.
  That is why a staged gridpack UFO and a pristine one hash differently on exactly these
  files. To close it: the provenance recipe hashes the amplitude files only, or hashes
  before the first MadGraph import, and the provenance concept says so.
- **The polarisation compile as a per-helicity probe.** The probe compiled a set
  polarised to one NHEL vector to get the production split evaluator's per-helicity
  signed |M|². That object has no public accessor (`eval_hel_m2` is `pub(crate)`). A
  committed split row could use the same route, or a public per-helicity read-out.
- **`resolve_mom` is dead code in release builds.** It is used only inside
  `run.rs:1679`'s debug check, so `cargo build --release` warns `method resolve_mom is
  never used` (`helas/eval/analysis.rs:208`). CI's clippy runs the dev profile and does
  not see it. To close it: `#[cfg(debug_assertions)]` on the method, or a gated call
  site.
- **`tprime126.md` names the wrong PDF set for lhaid 325500.** It says
  `NNPDF31_nnlo_as_0118_mc_hessian_pdfas`, which is 325300. 325500 is
  `NNPDF31_nnlo_as_0118_nf_4_mc_hessian`, from `pdfsets.index` and the set's own
  `SetIndex`.

## Corrections to the brief

- **"|M0|² and |M2|² partition consistently, so only 2Re(M0·M2*) disagrees" does not
  follow.** The partition check was vibegraph against itself. At the campaign binary
  |M2|² is wrong against MadGraph by 0.27× at generic points and 1.83× near the pole.
  The defect is a chirality on one class, which a σ partition cannot see. It is not a
  relative sign: the relative-sign hypothesis is falsified on main and on `9f196713`
  alike.
- **The comparison file is `vibegraph-lib/tests/standalone_jamps.rs`**, not
  `tests/standalone_jamps.rs`. Its `measure` and `check` cannot read a split table; the
  scratch probe replaces them.
- **The binary matters as much as the card.** The campaign's +4.779e-4 pb comes from a
  binary without `a5bf8fca`, and the brief's "what is known" does not say so. Main
  differs from it on exactly this vertex class.

## Follow-up A: t-channel zero-width sets, every subprocess of card 612

Head `6f660873`, with origin/main merged. This is diagnosis only: no source is changed.
The question was whether this crate's production zero-width rule (`PropInfo::t_channel`)
picks the same propagators as MadEvent's on all of 612's subprocesses. L-I1 checked only
`g u > h t b~ d`. The answer is yes: the sets are identical on all 12 flavour processes,
with 0 mismatches.

### Counting the subprocesses

There are 12 flavour processes, so the planner's "12" is right. On each side they group
differently.

**MadEvent:**
- 6 amplitudes, in 6 `matrix*.f` files across 2 P directories;
- 12 flavour processes, every one with `MIRRORPROCS = .TRUE.`;
- so 24 beam-ordered channels;
- 228 diagrams (12 × 19): "12 processes with 228 diagrams".

**This crate:**
- 12 sets, all `mirrored true`, each with 19 diagrams;
- `integrate` compiles 6 flavour groups (`flavour_groups 6, compiled_diagrams 114,
  subprocesses 12`).

### MadEvent side

`output madevent` of the full card, generation only, through `mg5_pinned.sh` with
MadGraph 3.7.1:

```
sa/gen_me612.mg5: import model .../VLQ_v4_4FNS_UFO-3rdL_noCKM; the 4 defines; generate p p > h tt bb j / bp x y VLQ^2==2; output madevent sa/me612
  Total: 12 processes with 228 diagrams
  INFO: Some T-channel width have been set to zero [new since 2.8.0]
  P1_gq_htbxq/matrix1: g u > h t b~ d, g c > h t b~ s     matrix2: g d > h t~ b u, g s > h t~ b c
              matrix3: g u~ > h t~ b d~, g c~ > h t~ b s~ matrix4: g d~ > h t b~ u~, g s~ > h t b~ c~
  P1_qq_htbxg/matrix1: u d~ > h t b~ g, c s~ > h t b~ g   matrix2: d u~ > h t~ b g, s c~ > h t~ b g
```

The comparison script is `harness/li1_widthdiff.py`. For each `matrix*_orig.f`, it walks
the `MATRIX` helas calls in order and rebuilds each `AMP(k)`'s propagators from the
current wavefunction slots. For each propagator it records:

- the particle, through its mass argument (`MDL_MT`, `MDL_MTP`, `MDL_MW`, ...);
- the set of external legs it carries, taken on the side away from the last leg;
- the width token: `ZERO` or `FK_MDL_W*`.

`FK_MDL_WTP` is `SIGN(MAX(|WTP|, |MTP|·small_width_treatment), WTP)`, which equals WTP at
these widths.

### This crate's side

The scratch test `harness/li1_widths.rs` was copied into `vibegraph-lib/tests/`, run in
release under the lock, and removed afterwards. It does three things:

- It parses `sigma/proc_612.dat` with `parse_proc_card` and enumerates the sets with
  `generate_from_proc_card`. That is the enumeration `integrate` runs.
- For each propagator of each diagram it records the particle, the baked `momentum`
  and `Prop::is_spacelike(2)`. That predicate is the one production reads:
  `root_diagram.rs:511` sets `PropInfo::t_channel` from it, and `lower.rs:340` bakes
  `Coeff(0)` for the width on those lines.
- **It checks the compiled code directly.** It compiles each diagram alone through the
  production compile and evaluates it at the 4 generic points over every helicity. This
  is done under the 612 card, and again under three cards, each with one width doubled
  (WT, WTP or WW). A diagram's amplitude can move under a doubled width only if it has
  a timelike line of that particle that keeps its width. The script checks that the
  observed moves agree with the prediction from `is_spacelike`.

```
LI1_CARD=sigma/proc_612.dat LI1_PARAM=sigma/param_card_612_full.dat LI1_POINT=tables/li1_gu_htbxd_w.json \
LI1_VARIANTS=wt=..x2wt.dat,wtp=..x2wtp.dat,ww=..x2ww.dat cargo test --release -p vibegraph-lib --test li1_widths
  12 sets; set ["g", "u"] > ["H", "t", "b~", "d"]: 19 diagrams, mirrored true; ... (12 lines)
python harness/li1_widthdiff.py sa/me612 tables/vg_widths_612.json
```

### The diff

Diagrams are paired machine-wise by their full propagator signature, the multiset of
(particle, leg set). The pairing is a bijection on every process, with 0 signature ties.
The script asserts that the signature sets are equal and that each multiplicity matches.

The counts are propagator occurrences per diagram, summed over the 19 diagrams, and only
for lines whose model width is non-zero (t, T′, W; the b width is 0).

| P dir / matrix | flavour process | graphs / diagrams | zeroed (MadEvent = this crate) | kept width | result | width-doubling observation |
|---|---|---|---|---|---|---|
| P1_gq_htbxq/matrix1 | g u > h t b~ d | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix1 | g c > h t b~ s | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix2 | g d > h t~ b u | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix2 | g s > h t~ b c | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix3 | g u~ > h t~ b d~ | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix3 | g c~ > h t~ b s~ | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix4 | g d~ > h t b~ u~ | 19 / 19 | 21 | 18 | identical | consistent |
| P1_gq_htbxq/matrix4 | g s~ > h t b~ c~ | 19 / 19 | 21 | 18 | identical | consistent |
| P1_qq_htbxg/matrix1 | u d~ > h t b~ g | 19 / 19 | 0 | 39 | matched, both sets empty | consistent |
| P1_qq_htbxg/matrix1 | c s~ > h t b~ g | 19 / 19 | 0 | 39 | matched, both sets empty | consistent |
| P1_qq_htbxg/matrix2 | d u~ > h t~ b g | 19 / 19 | 0 | 39 | matched, both sets empty | consistent |
| P1_qq_htbxg/matrix2 | s c~ > h t~ b g | 19 / 19 | 0 | 39 | matched, both sets empty | consistent |

```
TOTAL mismatches: 0
```

**The spacelike T′ lines, all zero-width on both sides.** In each g q channel, two T′
lines carry the legs {g, h, t}: the T′ after the gluon, and the T′ from (t + g) before
the h. The q q̄′ channels have no spacelike T′ line.

| flavour processes | first T′ line | second T′ line |
|---|---|---|
| g u > h t b~ d, g c, g d~, g s~ | AMP(5) = diagram 5 | AMP(17) = diagram 17 |
| g d > h t~ b u, g s, g u~, g c~ | AMP(6) = diagram 4 | AMP(18) = diagram 16 |

In every case MadEvent writes ZERO and this crate's line is spacelike.

**The width-doubling observation is not vacuous.** Doubling a width moves 64 (WT),
52 (WTP) and 140 (WW) of the 228 compiled diagrams. The smallest move is 5.2e-4, 1.1e-1
and 7.4e-5 of the diagram's size respectively, and every unmoved diagram moves by exactly
0.0. Every observed move agrees with the `is_spacelike` prediction.

Step 4 of the task, quantifying the effect of a difference, has no case: no subprocess
differs.

### What this check cannot see

- **Width values on timelike lines.** Those are the card's, and the L-I1 oracle checks
  them numerically on `g u > h t b~ d` only.
- **A wrong width on a line of a particle other than t, T′ or W.** Z and H have no
  internal lines in this process.
- **The amplitude values of the 11 subprocesses other than `g u > h t b~ d`.** The
  width-doubling check sees only whether a width enters, not whether the amplitude is
  right. The 4 doubled-width cards share the generic points, so a dependence that
  vanished at all 4 points would be missed; the moves seen are 7e-5 or larger.
- **How each flavour group's representative is applied to its members.** `integrate`
  compiles a representative per flavour group. Here all 12 sets were enumerated and
  checked individually, and every member of a group agrees.
- **Anything about the σ.** The cause of 612's 0.81 and 614's 0.52 is not here. The
  real-mass t-channel convention agrees on every line. Follow-up B shows the complex-mass
  scheme moving the interference by large factors near the pole.

## Follow-up B: what the complex-mass scheme does on `g u > h t b~ d`, MadGraph against MadGraph

The 612 and 614 gridpacks were generated with `set complex_mass_scheme`. Every vibegraph
σ, and every L-I1 table above, is real-mass. This section compares MadGraph with itself,
CMS on against CMS off. This crate has no CMS, and nothing here compares it to a CMS
table.

### Banking

`set complex_mass_scheme True` was put before `import model`, and the process was output
both ways:

```
sa/gen_cms_standalone.mg5: output standalone sa/cms_standalone_split      "Activate complex mass scheme." 19 diagrams
sa/gen_cms_madevent.mg5:   output madevent   sa/cms_madevent_split        no "T-channel width have been set to zero" line
```

**Correction to the request: under CMS, MadEvent zeroes no t-channel width.** Its calls
pass `DCMPLX(CMASS_MDL_MX)` on every massive line, spacelike T′, t and W included. Two
examples are `FFV1_1(W(1,8),W(1,1),GC_9(IVEC),DCMPLX(CMASS_MDL_MTP),W(1,6))` and
`FFV2_3(W(1,2),W(1,6),GC_27,DCMPLX(CMASS_MDL_MW),W(1,12))`. The zeroing in
`helas_call_writers.py:288` substitutes only `fk_` width tokens, and CMS calls carry
none. So "the same madevent-style zeroing" does not exist under CMS. Two CMS tables
were banked:

- **`cms_*`: the gridpack's own convention.** This is MadEvent's CMS calls, which are
  standalone's calls exactly (`li1_bank.py` with `LI1_ME=cms_madevent_split` makes no
  substitution).
- **`cmsreal_*`: a hybrid MadGraph never writes.** It is CMS, with the real mass
  substituted on exactly the 8 lines real-mass MadEvent zeroes. It is a control that
  separates the complex couplings and timelike complex masses from the complex mass on
  spacelike lines.

```
LI1_SA=sa/cms_standalone_split LI1_ME=sa/cms_madevent_split LI1_TAG=cms_ [LI1_RESONANT=1] python harness/li1_bank.py
LI1_CMS_REAL_SPACELIKE=sa/me_split/.../matrix1_orig.f LI1_SA=sa/cms_hybrid_split ... LI1_TAG=cmsreal_ [LI1_RESONANT=1] python harness/li1_bank.py
```

The points and card values are the same as in the real-mass tables. The script asserts
that the momenta are identical. The card text differs, because the CMS model reads MW
as an input, with no Gf and no YUKAWA block, but the values are the same.

The control holds. With every width zero, CMS on and off agree to 6 printed digits on all
three squared orders, and Σ|M0|² differs by 5.5e-8 relative.

### 1. Σ over points and helicities, CMS on against off (`/home/user/wt/logs/li1-cms-compare.log`)

| table | RES(VLQ²=0) on/off | RES(VLQ²=2) on/off | RES(VLQ²=4) on/off |
|---|---|---|---|
| w, MadEvent's CMS (`cms_w`) | 0.999927 | **1.048243** | 1.017487 |
| w, hybrid (`cmsreal_w`) | 0.996612 | 1.052085 | 1.018748 |
| res_w, MadEvent's CMS (`cms_res_w`) | 0.994663 | **−273.3** (+1.0875e-09 against −3.979e-12) | 1.011830 |
| res_w, hybrid (`cmsreal_res_w`) | 1.052455 | −481.6 (+1.917e-09 against −3.979e-12) | 1.018006 |
| zw (control) | 1.000000 | 1.000000 | 1.000000 |
| res_zw (control) | 1.000000 | 1.000000 | 1.000000 |

At the generic points, CMS moves the interference by +4.8% and |M0|² by −0.007%. Near
the pole, |M0|² moves by −0.5% and |M2|² by +1.2%. The summed interference changes sign
and grows by a factor of 270, because the real-mass sum is itself a near-total
cancellation.

### 2. Per point, RES(VLQ²=2), CMS on against off (MadEvent's CMS)

| point (m(t h), GeV) | on | off | on/off | |M0|² on/off | |M2|² on/off |
|---|---|---|---|---|---|
| 0 (791.8) | +2.098e-11 | +2.855e-12 | **+7.35** | 0.9973 | 1.0209 |
| 1 (794.1) | +1.884e-10 | +1.924e-10 | +0.979 | 1.0086 | 1.0214 |
| 2 (799.2) | +1.033e-09 | +8.120e-10 | **+1.272** | 1.0083 | 1.0284 |
| 3 (849.5) | −3.236e-10 | −1.155e-09 | **+0.280** | 0.9887 | 1.0096 |
| 4 (827.0) | −1.149e-11 | −1.170e-11 | +0.982 | 0.9960 | 1.0298 |
| 5 (721.5) | +1.801e-10 | +1.553e-10 | +1.160 | 0.9992 | 1.0211 |

For the 4 generic points the on/off ratio of RES(2) is 0.985, 1.032, 1.042 and 0.970.
In the hybrid, point 3 changes sign (−0.418): the complex mass on the spacelike lines
alone moves it by O(1). So yes: near the pole, CMS moves the interference by tens of
percent to factors of several, point by point, while |M0|² moves by at most 1.1%.

### 3. Which MadGraph parameters and couplings change under CMS

`Source/MODEL` of each output was linked with a 5-line driver that calls `SETPARA` and
then `PRINTOUT`. The output is `tables/li1_params_{split,cms_standalone_split}.txt`, and
the diff is `/home/user/wt/logs/li1-cms-params.log`. Every entry below changes by more
than 1e-6 relative.

**Complex masses, used on every massive propagator under CMS:**

| mass | CMS value |
|---|---|
| `CMASS_MDL_MTP` | 803.951 − 79.607i |
| `CMASS_MDL_MT` | 172.002 − 0.754i |
| `CMASS_MDL_MW` | 79.831 − 1.042i |
| `CMASS_MDL_MZ` | 91.196 − 1.247i |
| `CMASS_MDL_MH` | 125 − 0.002i |
| `CMASS_MDL_MX`, `CMASS_MDL_MY`, `CMASS_MDL_MBP` | 1000 − 2.5i |

Squared masses:

| parameter | real-mass | CMS |
|---|---|---|
| `MDL_MTP__EXP__2` | 640000 | 640000 − 128000i (20% imaginary) |
| `MDL_MT__EXP__2` | 29584 | 29584 − 259i |
| `MDL_MW__EXP__2` | 6371.9 | 6371.9 − 166i |

**Derived parameters that become complex**, with the relative change:

| parameter | relative change |
|---|---|
| `MDL_SW` | 2.0e-3 |
| `MDL_SW2` | 4.1e-3 |
| `MDL_CW` | 6.2e-4 |
| `MDL_GW` | 2.0e-3 |
| `MDL_G1` | 6.2e-4 |
| `MDL_VEV` | 1.5e-2 (246.249 − 3.717i) |
| `MDL_LAM` | 3.0e-2 |
| `MDL_YT` | 1.1e-2 |
| `MDL_YB` | 1.5e-2 |
| `MDL_YTAU` | 1.5e-2 |
| `MDL_YMT` | 4.4e-3 |
| `MDL_MUH` | 1.6e-5 |

`GF` itself becomes a derived complex quantity.

**Couplings** that change in this process, with the relative change:

| coupling | relative change | what it is |
|---|---|---|
| `GC_27` | 2.0e-3 | W f f′ |
| `GC_55` | 2.0e-3 | T′ b W, ∝ KTLw/sw |
| `GC_84` | 1.1e-2 | WWH |
| `GC_86` | 1.5e-2 | b Yukawa |
| `GC_87` | 1.1e-2 | t Yukawa |

`GC_19` = i·KTLh3 (T′ t H) and `GC_9` (gluon) are unchanged. So the large effect near the
pole comes mainly from the T′ complex mass, 20% imaginary in MTP², sitting on every T′
line. That includes the two spacelike T′ lines per g q diagram set, where the real-mass
scheme has zero width. The couplings move only at the 0.2–1.5% level.

### What this cannot settle

- **Fixed points are not σ.** The interference near the pole is a cancellation, and its
  integral depends on how these per-point changes weigh over the T′ line shape and the
  rest of phase space. The σ-level test is **R2**: MadGraph σ on 612 and 614 with
  `set complex_mass_scheme` on against off, which the validator runs. If CMS-off
  MadGraph lands near vibegraph's 0.81 (612) and 0.52 (614) of the CMS-on σ, the
  residual is the scheme, not a vibegraph defect.
- **One subprocess.** These tables cover only `g u > h t b~ d`. They are not a
  comparison of vibegraph with CMS.

### Correction to the L-I1 provenance

The transcribed `param_card_612.dat` lists MW as an input and has no Gf and no YUKAWA
block. That is exactly the layout of the card the CMS model writes (see the card diff
in `cms_w` above). So those entries were not lost in transport: the card is the CMS
gridpack's own. Only the QNUMBERS blocks were dropped. The values are equal on both
sides, as stated above.

### Files (scratch, not committed)

- `harness/li1_widths.rs`, `harness/li1_widthdiff.py`;
- `harness/li1_bank.py`, extended with the `LI1_SA`, `LI1_ME`, `LI1_TAG` and
  `LI1_CMS_REAL_SPACELIKE` switches;
- `sa/me612/`, `sa/cms_standalone_split/`, `sa/cms_madevent_split/`, `sa/cms_hybrid_split/`;
- `tables/li1_gu_htbxd_{cms,cmsreal}_{zw,w,w_sa,res_zw,res_w,res_w_sa}.json`;
- `tables/vg_widths_612.json`, `tables/li1_params_*.txt`;
- `sigma/param_card_612_x2{wt,wtp,ww}.dat`;
- logs `li1-gen_me612.log`, `li1-widths.log`, `li1-widthdiff.log`, `li1-bank-cms*.log`,
  `li1-cms-compare.log`, `li1-cms-params.log`.
