---
type: Session Report
title: "H-V2 report: the mirrored beam ordering follows MadEvent's IMIRROR = 2"
description: "FlavorGroup::mirror_into kept the beams and rotated only the outgoing legs, so a group's mirror term |M(Rq)|² peaked where no channel of the group sampled; on VBF_HH (52 parts after #95's mass split) σ read 0.840 ± 0.012 of MadGraph at 1e4 and 0.99 ± 0.08 at χ²/dof 21 at 1e5. The mirror term is now MadEvent's: the sampled point's |M(q)|², the densities with the beams exchanged (each parton at its own x and its own μF), and the event written rotated by π about x with the beams exchanged. VBF_HH reads 1.0111 ± 0.0080 of MadGraph over 10 seeds at 1e5 (χ²/dof 0.67, 10/10 runs accepted) and 1.002 ± 0.051 over 10 at 1e4, about twice as fast per point. No-mirror cards are bit for bit unchanged. A per-event oracle against a local MadGraph run with unequal per-beam factorisation scales passes on every event and fails on every exchanged event of the old code, whose σ there was 1.6% low."
status: draft
tags: [proton, beams, mirror, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-07}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: mg-dsigproc, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc#L812", title: "MadGraph super_auto_dsig_group_v4.inc DSIGPROC, IMIRROR = 2 (rotate P1, IB swap, XBK swap, CM_RAP sign), update_scale_coupling(pp)"}
  - {id: mg-pdflines, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L1960", title: "MadGraph export_v4.py grouped PDF lines: PDG2PDF(LPP(IB(i)), pdg, IB(i), XBK(IB(i)), QSCALE) with QSCALE = sqrt(Q2FACT(i))"}
  - {id: mg-autodsig, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/auto_dsig_v4.inc#L150", title: "MadGraph auto_dsig_v4.inc: REWGT(PP), SMATRIX(P1), UNWGT(PP) on the rotated momenta"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L560", title: "MadGraph unwgt.f write_leshouche: flip when p(3,1) < 0, pdfrwt beam numbers"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1450", title: "MadGraph reweight.f: s_xpdf(1,j) = xbk(ib(j)), s_qpdf(1,j) = sqrt(q2fact(j)); update_scale_coupling"}
---
Session H-V2 of sprint H. Worktree `/home/user/wt-mirror`, branch
`sprinth-mirror-rotation` from `origin/main` 9d1e9e6f, verified before any
action (`git rev-parse --show-toplevel` → `/home/user/wt-mirror`,
`git branch --show-current` → `sprinth-mirror-rotation`). Scratch:
`/home/user/H-V2-scratch` (logs prefixed `H-V2-`). Commits: see §9.

## 1. MadGraph's mirror, read from the source

`DSIGPROC` (`super_auto_dsig_group_v4.inc:782–860`), for `IMIRROR = 2`:

```
P1(2,i) = -P1(2,i); P1(3,i) = -P1(3,i)     (all legs, beams included: rotation by π about x)
IB(1) = 2; IB(2) = 1
XBK(1) <-> XBK(2)
CM_RAP = -CM_RAP
call update_scale_coupling(pp, wgt)        (pp = the UNROTATED sampled momenta)
DSIGn(P1, WGT, IMODE)                      (P1 = rotated)
```

Inside `DSIGn` (`auto_dsig_v4.inc`, PDF lines from `export_v4.py:1960–2010`):
leg `i`'s density is `PDG2PDF(LPP(IB(i)), pdg_i, IB(i), XBK(IB(i)), QSCALE)`
with `QSCALE = sqrt(Q2FACT(i))` (indexed by **leg**, not by `IB`). With `XBK`
already exchanged, `XBK(IB(1)) = x₁(sampled)` and `XBK(IB(2)) = x₂(sampled)`.
`SMATRIX(P1)`, `REWGT(PP)` and `UNWGT(PP)` all take the rotated momenta;
`write_leshouche` flips legs 1↔2 when `p(3,1) < 0`, so the first written
incoming line is the `+z` parton. The `<pdfrwt>` chains are written leg by
leg with `beam="2"` first for a flipped event (`unwgt.f:790–830`), with
`s_xpdf(1,j) = xbk(ib(j))`, `s_qpdf(1,j) = sqrt(q2fact(j))`
(`reweight.f:1450–1482`).

So, for a subprocess `a b` sampled at `(x₁, x₂, q)` (partonic CM, `a` along `+z`):

| quantity | direct (`IMIRROR = 1`) | exchanged (`IMIRROR = 2`) |
|---|---|---|
| `|M|²` | `|M_ab(q)|²` | `|M_ab(R q_all)|² = |M_ab(q)|²` (rotation invariance) |
| density of `a` | beam 1 PDF at `x₁`, `Q2FACT(1)` | beam 2 PDF at `x₁`, `Q2FACT(1)` |
| density of `b` | beam 2 PDF at `x₂`, `Q2FACT(2)` | beam 1 PDF at `x₂`, `Q2FACT(2)` |
| scales | `update_scale_coupling` on the sampled `pp` | the same call, same `pp` |
| written event | `q` boosted by `cm_rap` | `R q` with beams swapped: beam 1 = `b` at `x₂`, beam 2 = `a` at `x₁` |
| per-beam μF in the record | beam 1 `Q2FACT(1)`, beam 2 `Q2FACT(2)` | beam 1 `Q2FACT(2)`, beam 2 `Q2FACT(1)` |

At proton–proton beams the two density products are the same number, so
MadEvent's mirror term equals its direct term at every point (per mirrored
member), and the exchanged events are the direct ones rotated. The brief's
summary was right on every count; the one precision it lacked is that the
factorisation scales stay with the **partons** (leg-indexed `Q2FACT`), so on
the written exchanged event they appear exchanged across the beams.

## 2. What changed (`vibegraph-lib/src/proton.rs`, `generate/source.rs`)

* `FlavorGroup::mirror_into` is gone. `exchange_into(momenta, out)` builds the
  exchanged event in physical leg order: every leg rotated, beams swapped.
* `member_luminosity_rows`: mirror = direct for a mirrored member (`a` at
  `(x₁, μF₁)`, `b` at `(x₂, μF₂)`; only the beams differ, both protons).
* `group_term` / `per_group_sum`: one `|M(q)|²` per group per point,
  `(L_direct·Θ_direct + L_mirror·Θ_mirror)·|M(q)|²`.
* Cuts: each ordering on its own event. `cut_point` now cuts the exchanged
  event (`exchanged_lab_into`: `R q`, beams `x₂P₁`, `x₁P₂`, boosted at
  `½ln(x₂E₁/(x₁E₂))`) for any filter some mirrored group reads; at equal
  energies that is the sampled lab frame rotated. The test-side
  `emulate_madevent_mirrored_cut` now gives the mirror term the direct term's
  cut (MadEvent cuts the sampled configuration; still a registered deviation
  at `E₁ ≠ E₂` only).
* Scales: both orderings take the sampled point's prescription. Closed forms
  are unchanged; under clustering one configuration draw and one clustering
  serve both orderings (`AMP2` and every clustering measure are rotation
  invariant; MadEvent clusters the unrotated `pp`). `exchanged_scales` reports
  the mirror slot's `μF` in physical beam order (exchanged). Matched `rewgt`
  factors of a mirrored member equal its direct ones (`mirrored_rewgts`); the
  record `x` is the sampled `[x₁, x₂]` in clustering order in both orderings.
* `select_event`: helicity, colour flow, configuration and `rewgt` are read at
  the sampled point; `event_legs` maps the representative's leg 0 to beam 2 as
  before. `record_momenta(Exchanged)` writes the rotated event (massless: the
  exchanged cut frame; massive: the anchored `write_leshouche` boost on the
  rotated partonic event with the physical fractions `[x₂, x₁]`).
* Reweighter: the member's own amplitude at the sampled point in both orderings.
* `ProtonIntegrand::ordering_contributions(event)`: per group, `[direct,
  mirrored]` term of a point — the exact-symmetry diagnostic's accessor.
* `validation/cms_gridpacks/beam_balance.py`: the file-level diagnostic (§4).

Deviation from the brief: none in scope. One judgement call: at `E₁ ≠ E₂` the
production cut reads each ordering's own written event (as this crate already
did for the old construction); MadEvent's cut-the-sampled-configuration stays
the registered deviation, emulated for the unequal-beam gate.

## 3. Oracle 1 — exact symmetry, test and diagnostic

**Test** `proton::tests::each_ordering_carries_the_term_of_the_event_it_writes`
(new): `p p > l+ l- j`, beams 6500 × 4000, flavour-asymmetric probe PDF; for 300
accepted points, each group's `[direct, mirrored]` term from the integrand
against an oracle built from the PDF directly, the **independently compiled**
`b a > …` subprocess at the exchanged event, and the group filter on the
written lab record. Known-wrong informational comparison: the beams-kept
reading `f_b(x₁)f_a(x₂)|M(p₁,p₂,Rq)|²`.

```
$ cargo test -p vibegraph-lib --lib proton:: -- --nocapture     (H-V2-proton-tests2.log)
ordering terms: worst 1.56e-14 against the written events' own terms; 1434 fully mirrored
group terms symmetric to 0.00e0; 366 cut on one ordering only; the beams-kept reading
rejected on 1625 of 1626 mirrored terms
test result: ok. 66 passed; 0 failed; 3 ignored
```

On the old construction this test cannot pass: its exchanged term is the
beams-kept reading, rejected on 1625/1626 terms.

**Diagnostic** `beam_balance.py`: per unordered initial state, share of events
(and weight) with each parton on beam 1, binomial pull; exit 1 above 5σ. On
VBF_HH (2000 events per seed, `--min-events 40`):

```
$ python3 validation/cms_gridpacks/beam_balance.py <events.lhe>   (H-V2-vbfhh-balance.txt)
new n1e4 s1..s5: worst |pull| 1.28, 2.49, 2.32, 1.32, 1.43   (exit 0 each)
old n1e4 s1..s4: worst |pull| 6.81, 8.73, 6.33, 6.87         (exit 1 each)
pooled old (4 seeds):  -2 2  543 events  n-share 0.2339 w-share 0.1839  pull -12.40
                        2 3  463          0.7797         0.5868         +12.04
                       -1 1  381          0.2205         0.1555         -10.91
pooled new (10 seeds): worst |pull| 1.80 over 16 pairs; e.g. 1 2  9406  0.4934 0.4989 -1.28
```

(Old seed 5 at 1e4 was refused by the effective-sample-size guard,
"carried by an effective 3.0 points".) Per *group* mirror/direct after the fix
is identically 1 for a fully mirrored group (the test's "symmetric to 0.00e0";
`ordering_contributions` returns equal terms), so it is not a statistical
measurement any more; the event-level balance above is the before/after
measurement. The old per-part 0.005–0.28 is the CERN host's, not re-measured
here.

## 4. Oracle 2 — VBF_HH σ (masses in), against MadGraph 1.911 ± 0.0065 fb

Card from `sprinth-diagnosis-cards` 1795adea (read via `git archive`), model
`SM_HEL_UFO_noLightYukawa` staged as vgrun does, **with the gridpack's
`param_card.dat`** (the customizecards set CV = C2V = C3 = 1; the model's own
card has 0.1, which gives ~0.15 fb), `--niter 3 --fixed-budget -j 4
--parallel-diagrams`. Old seed 1 reproduces outcome.txt's 1.6314 fb to all
digits. `combine.py`: unweighted mean, sd, error of the mean, χ²/dof of seeds.

```
neval 1e5, new, seeds 1–10:  1.9638 1.8824 2.0307 1.9300 1.8854 1.9005 1.9143 1.8881 1.9549 1.9730 fb
  mean 1.9323, sd 0.0481, err 0.0152 → σ/MG 1.0111 ± 0.0080, pull +1.29, χ²/dof 0.67, 10/10 accepted
neval 1e5, old, seeds 1,2,3,5: 2.0396 1.6076 1.6727 2.2513 (seed 4 refused, effective 19.0 points)
  mean 1.8928 → σ/MG 0.9905 ± 0.0799, χ²/dof 21.14     (CERN host: 0.990 ± 0.080, χ²/dof 21.1)
neval 1e4, new, seeds 1–10:  2.0809 2.1383 1.8829 1.9263 2.6398 1.7121 1.7048 1.7424 1.6722 1.6492 fb
  mean 1.9149, sd 0.3071, err 0.0971 → σ/MG 1.0020 ± 0.0508, pull +0.04, χ²/dof 2.54, 10/10 accepted
neval 1e4, old, seeds 1–4:   1.6314 1.5398 1.6106 1.6409 (seed 5 refused)
  mean 1.6057 → σ/MG 0.8402 ± 0.0120, χ²/dof 0.17       (CERN host: 0.840 ± 0.012)
```

At 1e5 the mean is +1.1% (1.4σ of its own error), inside the known 1–2%
gridpack-survey offset, with a χ²/dof below one and no refused seed. At 1e4
the mean is right but the seed spread (16%) still exceeds the quoted errors
(χ²/dof 2.5): the budget is thin for 52 parts, a residual under-sampling at
the low rung, not a bias. Wall per integration: new 13–16 s (1e4), 44–47 s
(1e5); old 22–25 s, 79–82 s (one `|M|²` per point instead of two).

## 5. Oracle 3 — per-event conventions against a local MadGraph run

`p p > e+ e-`, sm, `pdlabel = lhapdf`, `lhaid = 247000`, `hard_survey = 2`,
5000 events, `iseed 21`, fixed scales with **unequal per-beam factorisation
scales** `dsqrt_q2fact1 = 50`, `dsqrt_q2fact2 = 200` (so the parton-vs-beam
assignment of μF is visible on every event), `mmll 60`, `use_syst T`
(`/home/user/H-V2-scratch/mg/dy_fixed.mg5`; MadGraph σ = 665.2 ± 1.635 pb).
vibegraph old and new on MadGraph's own `Cards/run_card.dat`.
`oracle3.py` checks per event: `<pdfrwt beam=b>` names beam `b`'s parton at
`x = E/E_beam` (2e-6, MadGraph's written precision), beam 1 reads μF 50 on
direct events and 200 on exchanged ones (and beam 2 the other), the first
`<pdfrwt>` line is the matrix element's first leg; and A_FB of the electron
relative to the quark direction per ordering (a non-rotated exchanged event
would flip its sign).

```
$ python3 oracle3.py dy_fixed/.../unweighted_events.lhe.gz vg/new/events.lhe vg/base/events.lhe
MadGraph   direct  2503 failures 0  A_FB +0.0923 ± 0.0199 | exchanged 2497 failures 0     A_FB +0.0885 ± 0.0199
new        direct  2536 failures 0  A_FB +0.1088 ± 0.0197 | exchanged 2464 failures 0     A_FB +0.1161 ± 0.0200
old        direct  2496 failures 0  A_FB +0.1506 ± 0.0198 | exchanged 2504 failures 2504  A_FB +0.0839 ± 0.0199
```

The same card's σ, 5 seeds each (`neval 2e4, niter 5`): new 665.07 668.06
664.44 661.55 664.69 → 664.76 ± 1.04 pb (−0.07% vs 665.2 ± 1.6); old 656.49
656.04 654.06 652.55 652.74 → 654.38 ± 0.78 pb (−1.6%). The old code read
the density of the parton on beam 1 at beam 1's `dsqrt_q2fact`, MadGraph reads
it at the parton's own: a σ-level difference the old code had at any card
with `μF₁ ≠ μF₂`.

Clustered scales on exchanged events are pinned in the unit tests
(`clustered_scales_are_read_in_the_partonic_centre_of_mass`: the gluon's
`p_T` lands on beam 2 for the exchanged ordering;
`matched_terms_carry_each_member_s_own_reweighting` recomputes every member's
`rewgt` from the sampled point) and against MadEvent's own banked events by the
existing `madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`
and `validate_mlm_records` (they read MadEvent's exchanged events rotated back
to the sampled point, the frame the integrand now uses).

## 6. Gates

All with `CARGO_TARGET_DIR=/home/user/H-V2-scratch/target CARGO_INCREMENTAL=0`
(and the untracked `.cargo/config.toml` `[profile.release-debug] debug = 0`
for `validate.sh`, deleted afterwards).

```
$ cargo fmt --all --check                                          → exit 0
$ cargo clippy --workspace --all-targets -- -D warnings            → Finished (after 17917fc5; the
    first run flagged two `Some(x).filter(|_| p)` → `then_some`, fixed in 17917fc5)
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation,\
    vibegraph-lib/extended-validation -- -D warnings               → Finished (at 7f34122c)
$ cargo test --workspace                       (H-V2-test-workspace.log, at 63f28903)
    55 test-result lines: 1970 passed, 0 failed, 25 ignored; EXIT 0
$ bash validation/validate.sh                  (H-V2-validate.log, at 17917fc5)
    85 test-result lines: 2423 passed, 1 failed, 110 ignored; clippy clean;
    report: "77 rows x 4 categories: the measured cells are the declared cells"; EXIT 101
    failed: cli_generate_proton::a_matched_sample_carries_the_shower_record,
            "event 1470: the Z's mass"
$ cargo test -p vibegraph --profile release-debug --features …extended-validation \
    --test cli_generate_proton                 (H-V2-cli-proton.log, at 7f34122c)
    test result: ok. 11 passed; 0 failed; 2 ignored
```

The one failure is a test tolerance below the record's printed precision, not
a physics change: reproducing the sample through the CLI
(`/home/user/H-V2-scratch/zmass`), event 1470 is a forward `d g → Z j` with the Z
at `E = 1255.7 GeV`, `E/m = 13.9`; its daughters printed at eleven significant
digits rebuild `m = 90.36554123` (exact decimal arithmetic on the printed
digits) against the record's `90.365540159`, rel `1.19e-8`, while the Z's own
printed momentum differs from the daughters' sum by `5e-8` GeV in `E` — the last
printed digit. The rebuilt mass is good only to `~1e-10 (E/m)²` (each printed
component carries `5e-11` of itself, amplified by the `E² − p²` cancellation);
7f34122c sets the check to `1e-9 + 1e-10 (E/m)²`, which all 2862 resonances of
that sample pass, and which still sees any frame or leg error (a finite
fraction). The old sample passed the flat `1e-8` by drawing no such event.
The full `validate.sh` was not rerun after 7f34122c (≈95 min on this host);
only the failing target was.

**Banked σ rows.** Every banked σ row passes. Against the last full
`validate.sh` on this host (H-N1's, at `20aca9fe` + its branch, the nearest
available baseline; 9d1e9e6f itself was not re-run), 37 of the 50 `GATE`
lines are identical to the printed digit (every `g g`, lepton-collider and
single-ordering row among them) and 13 moved, all rows with a mirrored
ordering, all within statistics:

```
row                  before (H-N1)             after (this branch)        MG
bb_fixed             2148212.8 ± 1310.4 (+0.0017)  2148125.1 ± 1363.7 (+0.0016)  2144600.0 ± 1291.2
beams_6500_4000 (emul) 717.575 ± 0.309 (+0.0005)     717.537 ± 0.409 (+0.0004)     717.250 ± 0.145
dy13 default          933.921 ± 0.434 (+0.0004)     933.763 ± 0.448 (+0.0002)     933.578 ± 0.184
jj                   6.797732e8 ± 3.603e5 (−0.0005)  6.796501e8 ± 3.821e5 (−0.0006)  6.800860e8 ± 5.020e5
llj_dyn               415.653 ± 0.244 (+0.0023)     415.547 ± 0.284 (+0.0020)     414.716 ± 0.895
llj_fixed             424.269 ± 0.248 (+0.0028)     424.032 ± 0.252 (+0.0023)     423.066 ± 0.898
mmll_60_120           644.457 ± 0.277 (+0.0005)     644.480 ± 0.284 (+0.0005)     644.138 ± 0.118
pp_to_bb             3.762756e8 (+0.0006)           3.762787e8 (+0.0006)           3.760360e8
pp_to_bb_iht         4.844779e3 (−0.0001)           4.844713e3 (−0.0001)           4.845030e3
pp_to_bb_qcd2        3.763923e8 (+0.0010)           3.763962e8 (+0.0010)           3.760240e8
pp_to_jj_ht          2.134238e5 ± 2.005e2 (−0.0004) 2.130868e5 ± 1.940e2 (−0.0020, pull −1.64)  2.135160e5 ± 1.758e2
pp_to_ll_mz95        746.88953 ± 0.461 (+0.00004)   747.45334 ± 0.335 (+0.00079, pull +1.60)   746.86000 ± 0.158
pp_to_ll_scalefact2  1957.591 ± 1.270 (−0.0002)     1958.589 ± 1.310 (+0.0003)     1958.040 ± 1.971
pp_to_llj             505.3132 ± 0.3575 (+0.0020)   505.6177 ± 0.3611 (+0.0026)   504.3220 ± 0.8758
```

(`H-V2-gates.txt`, `H-V2-gates-HN1.txt` in the scratch: `diff` of the sorted
`GATE` lines.) The largest moves are `pp_to_jj_ht` (−0.16%, 1.2σ of the
combined seed errors) and `pp_to_ll_mz95` (+0.075%, 1.0σ). The unequal-beam
row's own (unemulated) σ: 754.956 ± 0.352 before, 755.112 ± 0.337 after.

## 7. Bit for bit on no-mirror cards

`bfb/cards.sh`: `g g > t t~` (pp_to_llj run card), `g g > g g` (pp_to_jj run
card), `u u~ > e+ e-` (single ordering), `--neval 10000 --niter 4
--fixed-budget --seed 7`, 2000 events `--seed 11`. Before = 9d1e9e6f, after =
63f28903.

```
              σ (before = after)   grid SHA-256 (before = after)       events body SHA-256 (before = after)
gg_ttx  447.3 ± 1.3 pb   120a8ace0f6b5a45…6721ed   fa74798e980f4c07…a58f63
gg_gg   437.0 ± 2.6 µb   b3192c2f4f747d89…0fae     151257de54964d17…59503d
uux_ee  177.4 ± 1.8 pb   fb5144058b4a3e09…6fd26    6260eaa4322a6fe0…cda1e1
```

Grid files identical in full. Whole event files differ in exactly one header
line, the artifact path (`…/bfb/before/…` vs `…/bfb/after/…`), so the hash
compared is of everything after `</init>`; each file holds 2000 `<event>`
blocks (not the empty-input e3b0c442…).

## 8. #96's mirror tests

All pass in `cargo test --workspace`: `mirror_luminosity_follows_the_orderings_the_card_generated`
(its single-ordering and two-line assertions unchanged; its `p p > e+ e-`
expectation is now `[f1[a]f2[b], f1[a]f2[b]]`),
`a_subprocess_has_a_mirror_only_where_its_line_generated_both_orderings`,
`two_lines_in_exchanged_orderings_are_refused_at_fixed_energy`,
`a_subprocess_reached_from_two_lines_is_refused`, and
`a_polarized_group_is_evaluated_in_the_partonic_centre_of_mass` (its oracle
now builds the exchanged event by hand: 1.911e-12 from the CM assembly, 6.69
from the lab one).

Tests rewritten to the new convention (each with an independent oracle that
the old construction fails): `the_mirrored_beam_ordering_needs_the_reflected_matrix_element`
(mirror identity worst 5.97e-13), `a_point_reproduces_an_independently_assembled_integrand`
(5.5e-14 over 400 points, mirror up to 0.500 of a group), `an_exchanged_ordering_relabels_the_beams_of_every_per_leg_field`
(per-helicity 5.07e-13, per-flow 1.49e-13), `exchanging_unequal_beam_energies_reflects_the_integrand`
(now: the integrand at a point equals itself with the beam energies exchanged
at the same point), `the_emulated_madevent_mirrored_cut_reads_the_sampled_configuration`,
`a_members_share_of_the_draw_is_its_share_of_the_parton_luminosity`,
`drell_yan_classes_and_their_two_orderings_are_reproduced`,
`an_identical_parton_initial_state_carries_a_single_ordering`,
`clustered_scales_are_read_in_the_partonic_centre_of_mass`,
`a_massive_parton_record_is_anchored_where_madevent_anchors_it`, and in
`validate_hadronic.rs` `pointwise_integrand_oracle` (its MadGraph-side oracle
holds the unordered pair `q(x₁)q̄(x₂) + q̄(x₁)q(x₂)`, now matched as the direct
luminosity at `(x₁,x₂)` plus at `(x₂,x₁)`, with mirror = direct asserted).

## 9. Commits

On `sprinth-mirror-rotation`, pushed to `origin`, author Nicholas Smith
<nick.smith@cern.ch>, each ending `Assisted-by: claude-code:claude-opus-5-5`:

* 63f28903 proton: evaluate the mirrored beam ordering as MadEvent's IMIRROR = 2
* 17917fc5 proton: take the ordering scales with bool::then_some
* 7f34122c cli tests: hold the record's resonance mass to its printed precision
* (this report) docs: H-V2 session report

Files: `vibegraph-lib/src/proton.rs`, `vibegraph-lib/src/generate/source.rs`,
`vibegraph-lib/tests/validate_hadronic.rs`, `vibegraph-cli/tests/cli_generate_proton.rs`,
`validation/cms_gridpacks/beam_balance.py`, this report.

## 10. Found

1. **Thin budget on many-part cards at the low rung.** VBF_HH at `neval 1e4`
   now has the right mean (1.002 ± 0.051) but a seed spread of 16% against
   quoted errors of ~10% (χ²/dof 2.54; seeds 6–10 all at 0.86–0.91). At
   `1e5` it is clean (χ²/dof 0.67). The 52-part split of #95 spends 512 points
   per channel minimum on parts that carry 0.1% of σ. Closing: the CERN host's
   directions (a)/(b) (merge or share grids across negligible-mass parts,
   allocate per merged channel), re-measured with 20 seeds per rung.
2. **The old code's per-beam μF convention moved σ.** Not new work (fixed
   here), but worth a line in the sprint record: at `fixed_fac_scale` with
   `dsqrt_q2fact1 ≠ dsqrt_q2fact2` the old construction read each density at
   its *beam's* scale where MadGraph reads it at its *parton's*; σ was 1.6%
   low on `p p > e+ e-` at 50/200 (5 seeds, −6σ). Any banked or gridpack card
   with unequal fixed per-beam scales was affected; none of the banked rows is.
3. **MadEvent's massive-parton anchor on exchanged events (unverified).**
   `write_leshouche` chooses its boost anchor by `pmass(1)` (`unwgt.f:642`),
   which is leg 1's mass in amplitude order even after the `p(3,1) < 0` flip,
   so for `IMIRROR = 2` it tests the beam-2 parton's mass while anchoring beam
   1. vibegraph anchors on the physical beam-1 parton's mass (as before this
   session). Evidence: code reading only. Closing: one `g b > …` massive-b
   mirrored MadEvent event, compare the written beam energies.
4. **`validate.sh` after 7f34122c not re-run in full**; the manager may want a
   full gate run on the branch head before merge (≈95 min here).
5. **Narrative in a test doc.** `validate_hadronic.rs`'s
   `madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`
   says the unrotated reading is the one "the integrand used to take"; it now
   describes history. Hygiene: reword to the control's own terms.

Brief corrections: none that change the work. Two precisions: (i) MadGraph's
factorisation scales are leg-indexed (`Q2FACT(i)` with `QSCALE` for leg `i`),
so on a written exchanged event they appear exchanged across the beams; "the
per-beam μF as MadGraph assigns them" means exactly that. (ii) "mirror/direct
… pp symmetry gives exactly 1" holds for a group all of whose members are
mirrored; a group with an `a == b` or single-ordering member has a mirror term
equal to its mirrored members' share of the direct term. Also, VBF_HH needs the
gridpack's `param_card.dat` (`--param-card`, as vgrun's option): the model's own
card has CV = C2V = C3 = 0.1 and gives ~0.15 fb.
