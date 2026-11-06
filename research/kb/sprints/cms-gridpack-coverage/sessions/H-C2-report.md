---
type: Session Report
title: "H-C2 report: MadEvent's cut_decays exemption cannot be reproduced on an every-pairing |M|²"
description: "Stopped before code, as the brief allows. With every pairing kept in |M|², exempting MadGraph's pairing's decay legs leaves uncut a leg that is a core quark in another kept pairing. That quark carries the t-channel collinear singularity, so σ diverges: on g g > z u u~, z > u u~ the integrand at a point that passes the exempted cuts and the window union grows as 1/pT² of the exempt quark over three decades (value·pT² = 5.15e-18 constant to 4%), a logarithmic divergence of σ. MadEvent is finite only because its |M|² is the single pairing. An honest reproduction needs per-pairing |M|²: compat site (d), or a pairing-weighted cut. The user's call."
status: draft
tags: [cuts, decay-chains, identical-particles, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-06}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-06}
sources:
  - {id: setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L961", title: "MadGraph setcuts.f check_decay and the do_cuts readers"}
  - {id: hk1, resource: "H-K1-report.md", title: "H-K1 report, §5b and Found 1"}
  - {id: hd1, resource: "H-D1-report.md", title: "H-D1 report, §3: per-pattern cuts on a coherent sum are IR-unsafe"}
---
Session H-C2 of sprint H: the user's decision D5 (reproduce MadEvent's
`cut_decays = F` exemption on cards with identical particles across a decay,
as compat site (g)). Worktree `/home/user/wt-cd`, branch `sprinth-cut-decays`
off `sprinth-group-cuts` `1648ec79`:

```
$ cd /home/user/wt-cd && git rev-parse --show-toplevel && git branch --show-current && git log --oneline -1
/home/user/wt-cd
sprinth-cut-decays
1648ec79 kb: proton-integrand lists the refusals derive_flavor_groups raises today
```

**Outcome: stopped before writing production code.** The brief says: "If an
honest reproduction would need the single-pairing |M|² too, stop and report:
that would be site (d)". It does. With every pairing kept in |M|²,
exempting MadGraph's decay legs gives a σ that diverges logarithmically
(§2, §3). No code is changed on this branch. This report is the only commit.

`$S` below is the scratch directory
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad`,
and `$K` is `$S/H-K1`.

## 1. What MadEvent does, read on the pinned source

**`check_decay`** (`setcuts.f:961–1004`) runs only at `cut_decays = F`
(`setcuts.f:192–193`). It walks configuration 1's s-channel propagators from
the root down (`i = -(nexternal-3) … -1`; a parent has the more negative index)
and marks `from_decay` on both daughters of every propagator with
`tprid(i,1) = 0` and `gForceBW(i,1) = 1`, and on the daughters of every
propagator already marked. So `from_decay` is the set of all descendants of
configuration 1's forced lines.

**Configuration 1 of `g g > z u u~, z > u u~`** (`$K/H-K1-ggzuu`, P directory
`P1_gg_zqq_z_qq`):

```
$ head -1 leshouche.inc
      DATA (IDUP(I,1,1),I=1,6)/21,21,2,-2,2,-2/
$ sed -n 1,12p configs.inc
C     Diagram 1
      DATA (IFOREST(I,-1,1),I=1,2)/4,3/
      DATA (SPROP(I,-1,1),I=1,1)/23/
      DATA TPRID(-1,1)/0/
      DATA (IFOREST(I,-2,1),I=1,2)/5,-1/
      …
$ grep IDEN matrix1_orig.f
      DATA IDEN/256/
```

The Z is legs 3, 4 (final-state positions 0, 1); the core u ū is legs 5, 6.
`IDEN = 256` is 2·2·8·8 (gluon helicities and colours): MadGraph's matrix
element carries no identical-particle factor, because its one pairing has no
identical particles within a block.

**Every reader of `from_decay`**, directly or through `do_cuts`:

| reader | line | effect of `from_decay` |
|---|---|---|
| `do_cuts` | `setcuts.f:203` | false for a decay product |
| class pT / E / η cuts | `:269` | read `do_cuts` |
| per-PDG cuts | `:322` | skip a decay product (reads `cut_decays` and `from_decay`, not `do_cuts`) |
| ΔR cuts | `:344` | read `do_cuts` of both legs |
| pair-mass cuts (`mmjj`, `mmll`, …) | `:392` | read `do_cuts` of both legs |
| per-PDG pair-mass cuts | `:441–448` | skip a pair with a decay product |
| `smin` (τ floor) terms | `:538–683` | read `do_cuts` |
| merging-cut membership | `:245`, `cuts.f:553–676` | `ktdurham` / `ptlund` jets exclude decay products; both refused in vibegraph (`cuts.rs` `UNIMPLEMENTED_CUTS`) |
| `setxqcuts` | `:943`, `:951` | `xqcut` phase-space floors read `do_cuts` |

`ptll` (`:479`) and `mmnl` (`:411–429`) do not read `do_cuts`. vibegraph's
`Cuts::compile_with` already routes its one `uncut_products` mask through all
of these readers (`LegInfo::do_cuts` and `uncut_decay`, `cuts.rs:283–293`,
`:697–735`). So the per-leg plumbing would be a one-mask change. What blocks
the change is the mask's meaning, not its readers.

**Which vibegraph legs are MadGraph's.** The stitching places a decay's
products at the decayed leg's position (`chain.rs` `stitch_set`), and the
identity permutation is MadGraph's pairing. On this subprocess the scratch
probe (§3) prints:

```
member [21, 21] > [2, -2, 2, -2]
pattern ["0011"]   pattern ["0110"]   pattern ["1001"]   pattern ["1100"]
diagram 0 forced: ["0011"]
```

There are four pairings. MadGraph's is `0011`: final-state positions 0, 1, its
legs 3, 4. The forced-line patterns alone cannot name it, because `1100` is
just as contiguous. A reproduction would have to carry the identity
permutation's pattern out of `stitch_set`, for instance as a per-diagram mark
in `Provenance`, which survives `with_final_order`.

## 2. What σ equals with MadGraph's legs exempt and every pairing kept

Name the pairings by the Z's legs: a = {0,1} (MadGraph's), a' = {2,3},
c = {0,3}, c' = {2,1}. Let Θ_a be the cut indicator with legs 0, 1 exempt
(cuts on 2, 3), and W the window union. vibegraph's |M|² is
|M_a + M_a' + M_c + M_c'|², with the identical-particle factor 1/4. Then

σ_vg = ¼ ∫ (|M_a|² + |M_a'|² + |M_c|² + |M_c'|² + interference) Θ_a W,

while MadEvent computes σ_MG = ∫ |M_a|² Θ_a W_a.

- The |M_a|² term is σ_MG/4, up to the window union against W_a.
- In the |M_a'|² term, legs 0, 1 are the core u ū of `g g > Z u ū`, and
  Θ_a leaves them uncut. W passes through a' (m₂₃ ≈ M_Z). The core quarks
  carry the initial-state collinear singularity g → u ū:
  |M_a'|² ∝ 1/pT² as either goes along a beam. σ diverges logarithmically
  (∫ d²pT / pT²).
- In the |M_c|² and |M_c'|² terms, one of their core quarks (leg 0, or leg 1)
  is exempt, so they diverge the same way.

So the brief's expected residual, the +0.2% interference, does not hold. The
residual is a divergent term. In a VEGAS run it would not show as a divergence:
the channels cap each spacelike transfer at the `ptj`² floor
(`Cuts::spacelike_floor`), which is a support bound only while every leg is
cut. The run would report a finite, confidently wrong σ set by the channel
floors.

Moving the window does not rescue it. With MadGraph's own window W_a in place
of the union, the |M_a'|² term keeps a region where legs 0 and 1 are both
collinear, one per beam, with m₀₁ ≈ M_Z. That region is double-collinear
singular.

**Every rule that leaves a leg uncut has this problem when the leg is a massless
coloured core particle in some kept pairing.** The pairing-symmetric rule that
stays IR-safe is today's rule: exempt only the legs that are decay products in
every pairing (`ForcedResonances::decay_products`). That cuts all four quarks
here.

## 3. Measurement: the integrand at the exempt quark's collinear limit

A scratch test in `proton.rs` (not committed; the patch is
`$S/H-C2-probe.patch`) builds `g g > z u u~, z > u u~` under `sm-no_b_mass`
with every cut off and `cut_decays = T`, so `shape` is |M|² × PDF × window
union. It evaluates `shape` at fixed x₁ = x₂, √ŝ = 400 GeV. Leg 0 (u) has
E = 60 GeV and pT = ε along +z. Leg 1 (ū) is at polar angle 2.0 rad, with its
energy set so that legs 2 + 3 are an on-shell Z. Legs 2, 3 are that Z's
products, perpendicular in its rest frame. At every ε, legs 2, 3 pass the run
card's `ptj 20`, `etaj 5`, `drjj 0.4`, which MadGraph's exemption leaves as the
only cuts. Pairing a' is in its window (m₂₃ = 91.188). MadGraph's pairing a is
not (m₀₁ ≈ 167 GeV), so MadEvent gives this point zero.

```
$ CARGO_TARGET_DIR=/home/user/wt-cd-target CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
    cargo test -p vibegraph-lib --lib scratch_collinear_probe -- --nocapture      ($S/H-C2-probe.log)
eps     10 value 4.946134e-20 value*eps^2 4.946134e-18  pT(2) 90.04 pT(3) 90.04 pT(1) 145.28 m23 91.1880 m01 155.352
eps      3 value 5.638949e-19 value*eps^2 5.075054e-18  pT(2) 88.34 pT(3) 88.34 pT(1) 148.34 m23 91.1880 m01 163.775
eps      1 value 5.126317e-18 value*eps^2 5.126317e-18  pT(2) 87.86 pT(3) 87.86 pT(1) 149.20 m23 91.1880 m01 166.087
eps    0.3 value 5.717963e-17 value*eps^2 5.146166e-18  pT(2) 87.69 pT(3) 87.69 pT(1) 149.51 m23 91.1880 m01 166.887
eps    0.1 value 5.152039e-16 value*eps^2 5.152039e-18  pT(2) 87.64 pT(3) 87.64 pT(1) 149.59 m23 91.1880 m01 167.115
eps   0.03 value 5.726797e-15 value*eps^2 5.154117e-18  pT(2) 87.62 pT(3) 87.62 pT(1) 149.62 m23 91.1880 m01 167.194
eps   0.01 value 5.154713e-14 value*eps^2 5.154713e-18  pT(2) 87.62 pT(3) 87.62 pT(1) 149.63 m23 91.1880 m01 167.217
test proton::tests::scratch_collinear_probe ... ok
```

value·ε² is constant to 4% from ε = 10 to 0.01 GeV, and to 0.6% below
ε = 1. The integrand is ∝ 1/pT² there, and the phase-space measure is
∝ pT dpT, so σ diverges as log(pT_min). Under today's rule leg 0 fails
`ptj`, and the point is zero.

**Corroboration.** H-D1 §3 measured the same mechanism on the distinct-flavour
coherent sum. With each pattern cutting only its own core, the estimate is
"large and noisy (1.39 ± 0.05, 1.84 ± 0.22) and would grow without a
cutoff". The identical-flavour case is the same physics: a quark that is a
decay product in one kept pairing and a core quark in another.

**Blind spot.** This oracle is one ray toward one collinear limit. It shows
that the integrand is unbounded on the exempted region, which is enough for
divergence. It does not size the σ a VEGAS run would report.

## 4. The options, for the user

1. **Site (d) on: MadGraph's single pairing.** The evaluator keeps only the
   diagrams whose forced lines lead to MadGraph's blocks; the census already
   selects them (`decay_chain_census.rs` `unpermuted`). It drops the 1/4
   identical-particle factor for the factor MadGraph applies (here 1, from
   `IDEN = 256`) and uses MadGraph's per-pairing window. Then exempting that
   pairing's decay legs reproduces MadEvent's σ, region and |M|². Cost: a
   second evaluator path, and the registered "keep every pairing" deviation
   becomes the compat flag's off mode.
2. **A pairing-weighted cut.** Keep the coherent |M|², and replace Θ by
   Σ_p w_p Θ_p W_p with w_p = |M_p|² / Σ_q |M_q|². Here Θ_p exempts pairing p's
   decay legs. This is IR-safe: where pairing p' is collinear-singular,
   w_p' → 1 and Θ_p' cuts its core. It reduces to MadGraph up to the
   interference. Cost: per-pairing partial |M|² on every point (the same
   evaluator split as option 1) and a new cut semantics.
3. **Keep today's rule** (cut every leg that is not a decay product in every
   pairing). It is IR-safe and pairing-symmetric, and it is the natural off mode
   of site (g). Against MadGraph it is −26% on `g g > z u u~, z > u u~` and
   −6.8% on the `g g > z j j, z > j j` card (H-K1 §5b), to be registered as a
   deviation.

Option 1 is the only one that reproduces MadEvent's number at the precision the
other σ rows use. Options 1 and 2 both need per-pairing |M|², which is site
(d)'s machinery.

## 5. What was not done, and why

- No production code, test, `madgraph-defects.md` row or
  `identical-particles-across-decays.md` edit. The brief makes the catalogue
  entry's handling "reproduced; compat site (g)", and that is not yet true.
  Cataloguing it waits for the user's choice in §4.
- No σ sweeps. The three oracle rows (same flavour, distinct flavour bit for
  bit, the full card at `cut_decays = F`) measure an implementation, and there
  is none. H-K1's single-seed numbers stand as the current state:
  `g g > z u u~, z > u u~` at `cut_decays F` is 7.211 ± 0.028 pb (MadGraph s41,
  `$K/mgkeep/H-K1-mg-ggzuu-cdF-s41/results.dat`: `0.72110E+01 0.28166E-01`)
  against 5.348 ± 0.015 pb (vibegraph). At `cut_decays T` MadGraph reads
  5.283 ± 0.019 pb (`0.52831E+01 0.19443E-01`).
- No gates run. The tree is unchanged from `1648ec79` apart from this file:
  `git diff --stat` was empty after reverting the scratch test.

## Found

- **The brief's premise is wrong for every-pairing |M|².** It expected
  "exempting MadGraph's pairing's decay legs reproduces MadEvent's cut region …
  residual … about +0.2%". The residual is a log-divergent term (§2, §3).
  Closing D5 needs the user to choose among §4's options. Options 1 and 2 are
  site (d)'s per-pairing |M|² first.
- **Site (g)'s text in `madgraph-compat-sites-unconditional.md` assumes it can
  sit on top of the every-pairing |M|².** It says "On, reproduce the first
  configuration's exemption per group; off, a pairing-symmetric rule". With
  site (d) off, "on" is IR-unsafe. Site (g) on has to imply site (d) on, or
  use option 2. The item needs that constraint written in.
- **MadGraph's pairing is not recoverable from the forced-line patterns.** On
  `u u~ u u~` both `0011` and `1100` are contiguous. Any implementation of
  option 1 or 2 must carry the identity permutation's pattern out of
  `chain.rs` `stitch_set`, for example a per-diagram mark in `Provenance`. The
  census test's `unpermuted` helper gets it from MadGraph's banked blocks, not
  from vibegraph.
- **`Cuts::spacelike_floor` is a support bound only while every leg is cut.**
  Any rule that exempts a massless coloured leg which is a core particle in
  some kept diagram makes the channels cap a divergent region instead of
  sampling it. The result is a finite, wrong σ with no warning. Whatever lands
  for D5 should refuse, or assert at compile time, an exempt leg that is a
  massless coloured non-decay leg in any diagram of the set.
