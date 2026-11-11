---
type: Session Report
title: "J-F1 report: the relative Fermi sign read the order the legs are listed in"
description: "e+ e- > e- e+ against e+ e- > e+ e-, u~ u > u u~, the t t~ b b~ W class and four identical leptons are one defect: Diagram::sign listed each fermion pair lower index first. Dumps, the arithmetic check, the MadGraph verdict, the reach census, the fix and its gates."
status: draft
tags: [fermion-flow, sign-convention, relabelling, madgraph-oracle, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
sources:
  - {id: code-pairing, resource: "../../../../../vibegraph-lib/src/diagrams/diagram.rs", title: "Diagram::fermion_pairing_sign, Diagram::from_view"}
  - {id: code-tests, resource: "../../../../../vibegraph-lib/src/helas/eval/run.rs", title: "relative_fermi_signs_do_not_read_the_leg_order, probe_final_order_pair"}
  - {id: code-standalone, resource: "../../../../../vibegraph-lib/tests/standalone_jamps.rs", title: "ee_to_emep, uxu_to_uux, uux_to_ttxbbx_qed2"}
---

Session J-F1 of sprint J, on `sprintj-fermion-order` (branched from `main` at
`9a1349c`). It began as a diagnosis and became the fix session when the user put
both fermion-sign defects in scope. One cause explains both.

## The defect in one paragraph

`Diagram::sign` was `view.sign() × fermion_line_sign`. feyngraph's `view.sign()`
is the parity of the external fermions listed line by line, **each pair lower
leg index first**. The order inside a pair is a choice of labels, not of
physics. Relabel the legs, and the pairs whose lower-index end changes flip that
diagram's parity, while diagrams that pair the same legs differently may not
flip. Their relative sign then depends on the order the card lists the legs.
`fermion_line_sign` cannot repair this, because it reads only the class of each
line (initial–initial, mixed, crossed) and its propagator count, and both are
label-free. The per-line constants it carries were fitted on the banked rows'
orderings, so it is right only where a card lists its pairs the way those rows
do.

## 1. Dumps: one diagram class flips

`probe_final_order_pair` (an ignored diagnostic in `helas/eval/run.rs`)
compiles a line A and the same line B with its outgoing legs reordered,
evaluates both at one physical point (seed-7 RAMBO, √s = 500 GeV), pairs the
diagrams by helicity-vector overlap and prints each pair's complex ratio. On
the unfixed code:

```
VG_PAIR='e+ e- > e+ e-|e+ e- > e- e+|1,0' VG_DUMP=1 cargo test -p vibegraph-lib --lib \
    helas::eval::run::tests::probe_final_order_pair -- --ignored --nocapture

  A d0: sign=-1 pairing=+1 line=-1 props=[a] lines (0e+,1e-;P=0) (2e+,3e-;P=0)
  A d1: sign=-1 pairing=+1 line=-1 props=[Z] lines (0e+,1e-;P=0) (2e+,3e-;P=0)
  A d2: sign=-1 pairing=-1 line=+1 props=[a] lines (0e+,2e+;P=0) (1e-,3e-;P=0)
  A d3: sign=-1 pairing=-1 line=+1 props=[Z] lines (0e+,2e+;P=0) (1e-,3e-;P=0)
  B d0: sign=-1 pairing=+1 line=-1 props=[a] lines (0e+,1e-;P=0) (2e-,3e+;P=0)
  B d1: sign=-1 pairing=+1 line=-1 props=[Z] lines (0e+,1e-;P=0) (2e-,3e+;P=0)
  B d2: sign=+1 pairing=+1 line=+1 props=[a] lines (0e+,3e+;P=0) (1e-,2e-;P=0)
  B d3: sign=+1 pairing=+1 line=+1 props=[Z] lines (0e+,3e+;P=0) (1e-,2e-;P=0)
  B d0 <-> A d0: overlap 1.000000000000  ratio B/A = +1.000000000000 +0.000000000000i  max resid 0.00e0
  B d1 <-> A d1: overlap 1.000000000000  ratio B/A = +1.000000000000 +0.000000000000i  max resid 0.00e0
  B d2 <-> A d2: overlap 1.000000000000  ratio B/A = -1.000000000000 +0.000000000000i  max resid 0.00e0
  B d3 <-> A d3: overlap 1.000000000000  ratio B/A = -1.000000000000 +0.000000000000i  max resid 0.00e0
  hel B [-1, 1, 1, -1] | A [-1, 1, -1, 1]
     A: -3.131423e-2+5.300422e-2i  -9.241604e-3+1.567581e-2i  +4.636252e-2-7.847579e-2i  +1.262650e-2-2.137233e-2i
     B: -3.131423e-2+5.300422e-2i  -9.241604e-3+1.567581e-2i  -4.636252e-2+7.847579e-2i  -1.262650e-2+2.137233e-2i
eval_m2 A 8.458485220690e-2  B 1.683477662989e-1  rel 9.903e-1
```

Every diagram's magnitude agrees to the last bit. The s-channel pair (photon
and Z) has ratio +1, the t-channel pair −1. The only factor that differs is the
pairing parity of the t channel: A lists its lines as `(0,2)(1,3)` (odd) and B
as `(0,3)(1,2)` (even). The line signs are equal.

The same probe on the other orderings (`jf1-battery.log`, scratch) gives a
per-diagram split, with every magnitude exact, for `e- e+ > e+ e-`,
`u u~ > u~ u`, `u~ u > u u~`, `u u~ > u~ u g` (5/5),
`u u~ > e+ e- e+ e- QCD=0` against `e+ e- e- e+` (24/24) and
`u u~ > e+ e- u~ u QCD=0` (24/24). It gives a uniform ratio, and |M|² equal to
1e-16, for `u d~ > d~ u`, `u d > d u`, `u u~ > t t~ c c~ QED<=2` (only one
pairing exists), `e+ e- mu- mu+` and `e+ ve e- ve~` against `e+ e- ve ve~`.
`u u~ > t t~ b b~ QED<=2` against `t t~ b~ b` disagrees in |M|² by 4.2e-2.

## 2. The arithmetic check

For Bhabha, flipping the s-channel pair (B d0, d1) turns B into exactly −A,
diagram by diagram and helicity by helicity, with zero residual (the dump
above). So |M|²_B(x) equals |M|²_A at the relabelled point exactly:
8.458485220690e-2 on both sides. Without the flip, B gives 1.683477662989e-1.

The candidate rule, written first as a free-standing function in the diagnostic
commit (`flow_ordered_sign`), was compared with `Diagram::sign` over whole
enumerated sets. It agrees with the current sign up to one factor per set on
every ordering whose |M|² was invariant, and differs on exactly the diagrams
the dumps flip:

```
  A: 4 diagrams, candidate/sign over diagrams {1}          (e+ e- > e+ e-)
  B: 4 diagrams, candidate/sign over diagrams {-1, 1}      (e+ e- > e- e+)
  A: 39 diagrams, candidate/sign over diagrams {-1, 1}     (u u~ > t t~ b b~ QED<=2)
  B: 39 diagrams, candidate/sign over diagrams {1}         (u u~ > t t~ b~ b QED<=2)
  A: 31 diagrams, candidate/sign over diagrams {-1}        (u u~ > t t~ c c~ QED<=2)
```

The fixed-seed σ check is in §7.

## 3. The MadGraph verdict

MadGraph standalone (pinned 3.7.1, `sm`) generated `e+ e- > e+ e-` and
`e+ e- > e- e+` at the same RAMBO point X. Its two `matrix.f` files give the s
and t channels opposite JAMP coefficients in **both** orderings
(`[-1,-1,1,1]` and `[1,1,-1,-1]`). Our side was evaluated at X with MadGraph's
param card (`VG_CARD`, `VG_PA`):

| | MadGraph | vibegraph (unfixed) |
|---|---|---|
| `e+ e- > e+ e-` at X | 0.159442445 | 0.159442066 |
| `e+ e- > e- e+` at X | 0.171584230 | **0.450295660** |
| `e+ e- > e+ e-` at X with the outgoing legs swapped | — | 0.171583815 |

The 2.4e-6 residual on the correct rows comes from the width of the spacelike
Z, which standalone output keeps and this crate zeroes, as the methodology
predicts. MadGraph is invariant under the relabelling, and our A ordering agrees
with it. Per diagram on B at X, helicity `[-1,1,1,-1]`, our s-channel graphs
equal `+i ×` MadGraph's `c_k·AMP_k` and our t-channel graphs `−i ×`: the
relative sign is ours, and it is wrong.

The committed standalone rows reproduce this per flow and helicity
(`cargo test -p vibegraph-lib --test standalone_jamps`). Before and after the
fix:

| row | unfixed: worst flow (|M|²) | fixed: worst flow (|M|²), G |
|---|---|---|
| `ee_to_emep` (`e+ e- > e- e+`) | 2.158e-1 (4.595e-1) FAIL | 2.431e-15 (5.096e-15), −i |
| `uxu_to_uux` (`u~ u > u u~`) | 4.851e-1 (4.132e-1) FAIL | 2.851e-16 (5.856e-16), −i |
| `uux_to_ttxbbx_qed2` (`u u~ > t t~ b b~ QED<=2`) | 2.005e-1 (2.916e-1) FAIL | 2.202e-15 (2.352e-15), −i |

On the unfixed code the fit found |G| = 0.77, 1.02 and 1.25. A modulus that is
not 1 is the signature of a relative-sign error, not of a phase convention.

## 4. One cause or two

One cause. J-E1 measured `u u~ > t t~ b b~ QED<=2` against MadGraph: only the 4
W diagrams of `QED=2 QCD=2 / z a h` take the opposite overall G. Those are the
diagrams that pair (t b~)(b t~). Listed `t t~ b b~`, the pair `(b, t~)` = legs
`(4, 3)` has its lower-index end on the antiparticle t~, while every pair of the
(t t~)(b b~) pairing lists the particle first. The pair order therefore flips
the W class alone, as with Bhabha's t channel. The census (§5) finds exactly
those 4 of 39 diagrams. With the fix the row agrees at 2.2e-15. The candidate
also explains J-E1's controls: `t t~ c c~` has only one pairing, and the
`b b~` process with `/ w+ w-` loses the W class.

## 5. Reach

`probe_label_dependent_sign_census` (ignored diagnostic, diagnostic commit)
enumerated each card line and listed the sets whose `Diagram::sign` differed
from the label-free rule by more than one factor, that is, the sets whose
relative signs read the label order. On the unfixed code:

| card line | sets | label-dependent (diagrams flipped) |
|---|---|---|
| `e+ e- > e+ e-`, `e- e+ > e- e+`, `u u~ > u u~` | 1 each | 0 |
| `e+ e- > e- e+`, `e- e+ > e+ e-` | 1 | 1 (2/4) |
| `u~ u > u u~`, `u u~ > u~ u` | 1 | 1 (1/2) |
| `p p > j j` | 65 | 0 |
| `p p > j j j` | 97 | 0 |
| `p p > b b~`, `p p > t t~ j`, `p p > t t~ b b~` (QCD only) | 5, 13, 5 | 0 |
| `p p > e+ e- j j` | 64 | 0 |
| `p p > l+ l- j` | 24 | 0 |
| `p p > l+ vl l- vl~` | 32 | 0 |
| **`p p > l+ l- l+ l-`** | 12 | **8**: every `q q~ > e+ e- e+ e-` and `mu+ mu- mu+ mu-` (24/48) |
| **`p p > e+ e- e+ e-`** | 4 | **4** (24/48) |
| `p p > e+ e- e- e+`, `p p > e- e+ e+ e-` | 4 | 0 |
| **`p p > t t~ b b~ QED<=2`** | 5 | **5**: `g g` (108/132), each `q q~` (4/39) |
| **`p p > b b~ b b~`** | 5 | **5**: `g g` (36/72), each `q q~` (7/14) |
| **`p p > z z, z > e+ e-`** | 4 | **4** (2/4) |
| **`p p > z z, z > l+ l-`** | 12 | **8** (2/4) |
| `p p > w+ w-, w+ > e+ ve, w- > e- ve~` | 4 | 0 |
| **`z > u u~ d d~`** | 1 | **1** (12/17) |
| **`e+ e- > e+ e- e+ e-`** | 1 | **1** (96/144) |
| `e+ e- > e+ e- mu+ mu-` | 1 | 0 |
| every `MG_VALIDATED_PROCESSES` line, and the SM standalone rows' processes tried (`e+ e- > t t~ h`, `ta+ ta- > t t~ h h`, `b b~ > h h`, `w+ w- > u u~ g`, `w- w+ > e- e+`, `u u~ > t t~ g`, `g g > t t~ g`, `e+ e- > w+ w- z`) | 1 each | 0 |

After the fix every row of the table reads 0.

Where a set is affected:

- **Explicit lines.** A set is affected when two of its pairings differ in how
  many pairs list the "wrong" end first under lower-index order. For a
  same-flavour 2→2 that means the beam order and the final order disagree in
  particle-first-ness (`e+ e- > e- e+`, `u u~ > u~ u`). For identical
  final-state pairs it means the standard listing itself
  (`e+ e- e+ e-`, `b b~ b b~`, and `t t~ b b~` once a W joins them).
- **Multi-particle labels.** `p` and `j` expand particle before antiparticle
  (`g u c d s u~ c~ d~ s~`), and enumeration keeps the first ordering it emits
  for each subprocess. Its representative of a same-flavour pair is therefore
  `u u~ > u u~`, which is consistent. The `u~ u` ordering is folded into the
  `mirrored` flag and evaluated as the representative at the exchanged event,
  so the mirrored term is immune whenever the representative is. A user-defined
  label that lists an antiquark first could emit an affected representative.
  The identical-pair final states (`l+ l- l+ l-`, `b b~ b b~`) are affected
  whatever the labels, because the defect sits inside the final state.
- **`p p~` beams** change only which beam is which, with the same
  representative rule. They are affected exactly where `p p` is.
- **Decay chains** relabel the glued graph and rebuild its sign from the graph
  (`chain::relabelled`), so a chain gives what the explicit line in its final
  order gives. ZZ → 4e and ZZ → 4μ are affected (2 of 4 diagrams).
- **`with_final_order` / `content_ordered`** also rebuild the sign from the
  relabelled graph. They inherited the defect for the new order. They could
  turn a consistent order into an affected one only by moving a fermion pair
  across mass or cut classes (`t` against `b~`).
- **Banked rows.** No banked amplitude row reaches the defect: every
  `MG_VALIDATED_PROCESSES` line and every SM standalone row's process is 0 in
  the census. `ee_to_ee` is listed `e+ e- > e+ e-` and `uux_to_uux`
  `u u~ > u u~`, the two consistent orderings. They passed because they sit on
  the labelling the line-sign constants were fitted to, not because the rule
  was right. No banked gate passed by luck in a weaker sense: none of them
  contained an affected set.

## 6. The fix

`Diagram::fermion_pairing_sign` now puts each pair's ends in a label-free
order before counting inversions:

- on a line with both ends on one side (both incoming, or both outgoing), the
  end that is physically the antiparticle comes first;
- on a mixed line, the incoming end comes first.

`Diagram::from_view` uses it in place of `view.sign()`. `fermion_line_sign` is
unchanged. The new rule reproduces the old one, up to one factor per set, on
every ordering the old one handled correctly: `e+ e- > e+ e-`, where both rules
list the same pairs, and `u u~ > u u~`, where the initial and the final pair
both swap, so their factors cancel. Every other sign in the evaluator is
untouched. Size: one function body (about 10 lines) and its documentation, plus
one line in `from_view`.

Gated by:

- `helas::eval::run::tests::relative_fermi_signs_do_not_read_the_leg_order`:
  |M|² and every helicity's colour-summed |M|² at 1e-12 for Bhabha (final swap
  and beam swap), `u u~ > u u~` (both swaps), `u u~ > e+ e- e+ e- QED<=4`
  against `e+ e- e- e+`, and `u u~ > t t~ b b~ QED<=2` against `t t~ b~ b`,
  at two RAMBO points.
- `proton::tests::antiquark_first_line_with_its_flavour_in_the_final_state`:
  J-W1's reproducer from `4b5d3bb` (setup and controls kept), now asserting at
  1e-12 and enforced. Unfixed, it fails with
  `4.709463437305e3 against 4.398818122664e3, rel 6.596e-2`, J-W1's numbers.
- The three standalone rows in §3, per flow × helicity against MadGraph.

Mutation evidence (fix reverted, the tests unchanged): see §7.

One banked pin moves, and the move is a sign shared by every diagram of the
process. `CONFIG_AMP_SIGNS` in `tests/amplitude_oracle.rs` records, per row, the
sign of each configuration amplitude against its diagram compiled alone. The
configuration amplitudes carry no Fermi sign and the diagram compiled alone
does, so the pin reads `Diagram::sign` itself, including the process's overall
sign. No label-free pair order can keep every listing's overall sign: `e+ e-`
and `mu+ mu-` list the antiparticle first, `u u~` and `t t~` the particle
first. Seventeen rows therefore flip **every** entry, and no row flips some
entries but not others (the commit's edit script asserted that each new string
is the exact complement of the old):
`bbx_to_h_identity`, `bbx_to_tapta_ckm_no_b_mass`, `ddx_to_epemg`, `ee_to_tlt`,
`ee_to_ttx`, `ee_to_ttx_dipole`, `ee_to_ttx_smeft`, `ee_to_ttx_smlimit`,
`ll_to_qqx_toy_dipole`, `ll_to_qqx_toy_tensor`, `ll_to_qqx_toy_yukawa`,
`pp_to_ll_qcd0`, `tata_to_ttx_tensor4f`, `ubx_to_epve_ckm_no_b_mass`,
`uux_to_epemg`, `uux_to_mumu`, `uux_to_ztg`. The oracle's `violation!` returns
at the first failed check, and the pin is checked after every MadGraph
comparison of the row: per diagram, per flow, JAMP2, and each configuration
amplitude against `AMP()` up to `±G`. All of those passed for the seventeen rows
before the pin was read. Nothing reads the pin downstream, since `AMP2` is a
modulus.

Why exactly these seventeen: the new rule lists an initial–initial or crossed
pair antiparticle first, where the old one took the lower index. The overall
sign therefore flips when a listing puts an odd number of such pairs particle
first. Every flipped row has `q q~` beams or `t t~`/`qt qt~` outgoing (in
`ubx_to_epve`, `u b~` in and `e+ ve` out). Every unflipped row lists
antiparticle first on both sides (`e+ e- > mu+ mu-`, `ta+ ta-`, bosons), has no
fermions (`g g > h`), or has a mixed quark line beside `e+ e-` (`g u`, `g u~`).
An overall sign reaches nothing observable. It multiplies every diagram, and so
every JAMP, of one set. |M|², per-helicity |M|², JAMP2 and the CF contraction,
and AMP2 (configuration amplitudes carry no Fermi sign) are quadratic or moduli.
No two sets are summed coherently: decay chains are glued into one set, flavour
groups and mirrored terms re-evaluate one |M|², and the event record carries no
amplitude phase.

## 7. Gate results

**Commits.** `ab2be842` adds the failing tests (ignored), `e1a55721` adds the
fix and enforces them. Both are pushed to `origin/sprintj-fermion-order`.

**Fixed-seed σ** (`vibegraph integrate <card> --run-card mgr-ee/run_card.dat
--neval 20000 --niter 5 --seed 3`, debug-profile build of `e1a55721`):

| card | before (manager's measurement) | after |
|---|---|---|
| `e+ e- > e+ e-` | 155.96 ± 0.16 pb | 155.96 ± 0.16 pb |
| `e- e+ > e- e+` | 155.96 ± 0.16 pb | 155.96 ± 0.16 pb |
| `e+ e- > e- e+` | **168.92 ± 0.16 pb** | 155.96 ± 0.16 pb |

With `--fixed-budget` all three read 156.20 ± 0.31 pb. The per-iteration
estimates of `e+ e- > e+ e-` and `e- e+ > e- e+` are identical to the printed
seven digits. Those of `e+ e- > e- e+` differ from them by at most 1.7e-6
relative (last iteration: 4.005314e-7 against 4.005321e-7), at a statistical
error of 1.0e-3. Both lines draw the same points with the outgoing species
exchanged, so the integrand is evaluated at the mirrored configuration, which
differs only through the Z's small parity-violating part.

**Workspace** (`cargo test --workspace --no-fail-fast`, `CARGO_TARGET_DIR` own,
after the pin update): every binary ok. The lib binary passed 1334, with 15
ignored. `amplitude_oracle` passed 51, `standalone_jamps` 57 (54 existing rows
plus 3 new), and every other binary ok. Before the pin update the only failures
were the 17 `CONFIG_AMP_SIGNS` rows above. `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` are clean.
`pixi run kb-lint`: 0 errors.

**Standalone rows after the fix.** Every pre-existing row reads the same values
as before. Examples: `ee_to_ee_4f` worst flow 4.866e-14 before and after,
`wmwp_to_emep` 1.409e-15 before and after, `wpwm_to_wpwmz_cw` 1.931e-14. The
three new rows are in §3.

**Mutation.** `jf1-mut.sh`, scratch: `diagram.rs` is set back to `ab2be842`'s,
the tests are left as committed, and the file is restored on exit (verified
restored):

```
test proton::tests::antiquark_first_line_with_its_flavour_in_the_final_state ... FAILED
  u u~ > u u~ against u~ u > u u~ at the exchanged event: 4.709463437305e3 against 4.398818122664e3, rel 6.596e-2
test helas::eval::run::tests::relative_fermi_signs_do_not_read_the_leg_order ... FAILED
  e+ e- > e+ e-: 6.621195661713e-2, e+ e- > e- e+: 9.329936714448e-2, rel 4.091e-1
ee_to_emep: G = -1.2481i, worst flow 2.158e-1, worst |M|² 4.595e-1        FAILED
uxu_to_uux: G = -0.7660i, worst flow 4.851e-1, worst |M|² 4.132e-1        FAILED
uux_to_ttxbbx_qed2: G = 0.0048+1.0225i, worst flow 2.005e-1, worst |M|² 2.916e-1   FAILED
```

**`pixi run --skip-deps validate`**: VALIDATE-PLACEHOLDER

## What would falsify this

- A banked amplitude row moving under the fix. None moved: §7.
- A relabelling pair whose per-diagram ratios split after the fix. The census
  reads 0 on every line above, and the relabelling test holds at 1e-12.
- MadGraph disagreeing with the fixed sign on an ordering not tried here. The
  three new standalone rows cover a final-state swap, a beam swap with
  colour, and two pairings joined by a W.

## Found

- **Stored diagram sets carry the old sign.** `Diagram::sign` is serialised
  into the diagram artifact (`enumerate`, format 2) and into the integrate
  artifact's `sets` (format 22). A `generate` run reads those sets instead of
  enumerating again, and its amplitude probe recomputes |M|² *from the stored
  sets*, so it reproduces the old value and passes. An artifact written before
  this fix for an affected set (four identical leptons, `b b~ b b~`,
  `t t~ b b~` with a W, a same-flavour 2→2 listed against its beams) therefore
  still generates with the wrong interference, silently. To close: on reading
  either artifact, rebuild each diagram's sign from the graph
  (`fermion_pairing_sign × fermion_line_sign`, the model is embedded or
  loaded) and refuse a set whose stored sign differs, or bump both format
  versions.
- **The fermion-line-sign concept describes the old pairing.**
  `research/kb/amplitudes/fermion-line-sign.md` says the pairing sign is
  "feyngraph's `view.sign()`", and the crossed-line `−1` derivation there
  assumes the lower-index-first listing. Its "How it is checked" section should
  add the relabelling test and the three standalone rows. The same text appears
  in `convention-sign-inventory.md` if that page repeats the factor list. This
  is close-out work.
- **No banked row lists a pair against its labels.** The reach table shows
  that the whole banked layer sits on two orderings. A standalone row for
  `u u~ > e+ e- e+ e-` (four identical leptons, the common ZZ final state) and
  for `g g > b b~ b b~` would put the most-used affected processes under
  MadGraph directly, not only under the relabelling test.
- **`compile_single_diagram`'s doc claimed label invariance** ("however the
  legs are labelled") that nothing tested. It is now true up to one factor per
  set, and pinned by the relabelling test.
