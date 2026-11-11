---
type: Session Report
title: "J-E1 report: the SMEFTsim four-quark compile panic"
description: "Two causes behind the panic on SMEFTsim u u~ > t t~ b b~ NP<=1, each fixed with a failing-first test and a MadGraph standalone oracle; the target process disagrees with MadGraph because of two defects found here and filed: a W between the t and b lines beside a third line, and a dipole on an initial-state quark line."
status: draft
tags: [evaluator, smeft, color, rooting, four-fermion, madgraph-parity, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-11}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
---
Worktree `/home/user/wt/sprintj-e1`, branch `sprintj-panics`, based on `main`
at `9a1349c`. MadGraph is the pinned 3.7.1 submodule run through
`gen_standalone_jamps.py` from the `madgraph` pixi environment, with the
SMEFTsim model staged in the session scratchpad (`VG_MODELS_DIR`). The host
has 4 cores, shared with two sibling sessions.

| commit | what |
|---|---|
| `e678102` | test: the crossed contact's colour chains (fails before `f0c0c00`) |
| `f0c0c00` | fix(color): a vertex's colour structures come from its own fermion-flow group |
| `511ff0a` | test: a four-fermion current re-rooted across the other line (fails before `79301b2`) |
| `79301b2` | fix(eval): relabel a re-rooted Lorentz term against the caller's inputs |
| `c839ba9` | test: two six-quark processes against MadGraph standalone |

## The panic has two causes

Reproduced on the branch's base, release build:

```
$ vibegraph integrate card.dat --run-card run_card.dat --ufo-dir validation/ufo \
      --neval 1000 --fixed-budget --niter 1
 INFO enumerate: 2517 diagrams for u u~ > t t~ b b~
 INFO compile: 6 colour flows, CF matrix 6×6 process=u u~ > t t~ b b~
thread 'main' panicked at vibegraph-lib/src/helas/eval/analysis.rs:603:29:
index out of bounds: the len is 0 but the index is 0
```

### Cause 1: a colour chain with no coupling (lowering, from colourisation)

The empty `Op::Add` comes from `lower::lower_vertex`. It is
`sum_or_single` over the terms of a `VertexInfo` that has none. A temporary
`eprintln!` in `VertexInfo::from_ufo` (not committed) printed every empty
occurrence in the 2517-diagram compile. There were 96 of them, all on one
vertex:

```
24 × V_753#1 [t~ b b~ t] color_idx=0 flow_group=0 group_lorentz=[0, 1] keys=[(0,2),(0,3),(1,0),(1,1)]
24 × V_753#1 [t~ b b~ t] color_idx=1 flow_group=1 group_lorentz=[2, 3] ...
24 × V_753#2 [t~ b b~ t] color_idx=2 flow_group=0 group_lorentz=[0, 1] keys=[(2,2),(2,3),(3,0),(3,1)]
24 × V_753#2 [t~ b b~ t] color_idx=3 flow_group=1 group_lorentz=[2, 3] ...
```

`V_753` is `O_QtQb` (`cQtQb1Re`, `cQtQb8Re`). Its Lorentz structures 0 and 1
(`FFFF1`, `FFFF10`) pair its spinors (2,3)(4,1), and 2 and 3 (`FFFF12`,
`FFFF2`) pair them (2,1)(4,3). The UFO couples colour structures 1 and 3
only through the first pair, and 0 and 2 only through the second. Enumeration
splits the vertex into one occurrence per spinor pairing, and the evaluator
sums only that pairing's couplings (`VertexInfo::from_ufo`). But
`colorize::used_color_indices` took every colour index any coupling key
named, so the colour basis also built chains pairing an occurrence with a
colour structure that only the other pairing couples. That `(diagram,
chain)` amplitude has no term, and its value is the empty sum, identically
zero.

So it is the benign case: a chain whose terms never existed, not a flow or
configuration that lost a contribution. The evidence is the key lists above.
Every dropped `(colour, group)` pair has no coupling key at all, and every
chain that has one is untouched. No numerical "before" exists to dump against,
because the process does not compile without the fix. The empty `JAMP` probe
in `lower_flows` never fired: no colour flow lost all its terms, and
`b b~ > t t~ NP<=1` keeps its two flows.

The fix reads only the coupling keys whose Lorentz structure belongs to the
occurrence's flow group (`used_color_indices`, which `check_slot_reps` also
uses). `VertexInfo::from_ufo` refuses an occurrence with no coupling by name,
so an empty sum can never reach lowering again.

### Cause 2: legs relabelled across a non-adjacent partner (rooting)

With cause 1 fixed, the same card panicked one stage later:

```
thread 'main' panicked at vibegraph-lib/src/helas/eval/layout.rs:767:22:
internal invariant: a fermion bilinear needs one bra and one ket operand, got (FermionOut, FermionOut)
```

I compiled each diagram as its own `DiagramSet` (a temporary ignored lib test,
not committed). In `u u~ > t t~ b b~ NP=1 QED<=2`, 20 of 658 diagrams panic.
In `u u~ > b b~ g NP<=1`, 20 of 189 panic, all with one contact vertex:
`V_787#{3,4,5,6,9,10,11,12}` and `V_788#{3,6}` (`u~ b b~ u`). Their
vector-vector structures are `FFFF3`, `FFFF14`, `FFFF15` and `FFFF16`, all
pairing (2,3)(4,1). `d d~ > b b~ g`, `u u~ > t t~ g` and `u u~ > c c~ g`
compile. The dump of diagram 4 shows the rooted current:

```
vertex V_787#6 group 0 rays [Leg(1) u~ in, Leg(3) b out, Leg(2) b~ out, Prop u*]
OffShellCurrent[bra](GC·ProjM(GammaOout(GammaVout(Leg(0),ProjP(Leg(1))),Leg(2))))
```

The output is slot 4 (the `u*` propagator). Its partner, slot 1 (the
incoming `u~`), is a bra, so `correct_spin_index_for_flow` re-roots the term
at slot 1. The rooted tree then reads slot 4 as the continuing input.
`compact_legs` compacted over slot 1, but the caller binds its inputs with
slot 4 removed (`[slot 1, slot 2, slot 3]`), so the tree's `Leg(0..2)` landed
one input over: the closed vector paired the `u~` with the `b`, two bras. The
two compactions agree only when the partner is adjacent to the output. That
holds for every FFV vertex and every (1,2)(3,4) pairing, but not for (2,3)(4,1).

The fix splits `build_at_leg` into `build_rerooted` (the tree over the
vertex's own slots, plus the slot it was rooted at) and `compact_for_caller`.
`compact_for_caller` swaps the output and partner labels, then compacts over
the output. For an adjacent partner this is the identity on the compacted
tree, which a test pins (below). With both fixes, all 2517 diagrams of the
target compile one at a time, and the whole process compiles.

## Oracles

All MadGraph tables are `output standalone` (`gen_standalone_jamps.py`) at 3
RAMBO points, √s = 1 TeV, every helicity. The gate is
`tests/standalone_jamps.rs` at `REL_TOL = 1e-12`. It fits one complex
constant G over every (point, helicity, flow) entry and requires |G| = 1, the
per-flow residual, and the per-helicity colour-summed |M|². Both sides bind
the param card banked in the table, and both read the restrict card the table
names.

The rows use a new restrict card, `vg_nodipole`: `restrict_massless` with
every dipole operator zero (Found 1). Rows reaching the loop-induced
Higgs-gluon vertices use `SMHLOOP=0`, because MadGraph writes those couplings
from seven-digit literals. `u u~ > t t~ c c~ NP<=1` missed by 1.1e-11 with
them and agreed at 7.5e-16 without.

```
$ cargo test -p vibegraph-lib --test standalone_jamps -- 4f_nodipole --nocapture --test-threads 1
bbx_to_ttx_4f_nodipole: 96 flow entries, G = -1.0000000000000004i, worst flow 1.642e-15, worst |M|² 9.720e-16
uux_to_bbxg_4f_nodipole: 384 flow entries, G = 1.0i, worst flow 1.727e-15, worst |M|² 1.659e-15
uux_to_ttxbbx_4f_nodipole: 1152 flow entries, G = -0.0194+0.9269i, worst flow 6.298e-1, worst |M|² 4.710e-1
uux_to_ttxbbx_4f_nodipole: informational, disagrees as listed: ...
uux_to_ttxccx_4f_nodipole: 1152 flow entries, G = 1.0000000000000073i, worst flow 6.058e-15, worst |M|² 5.147e-15
test result: ok. 4 passed; 0 failed
```

| row | process | MG diagrams | reaches | result |
|---|---|---|---|---|
| `bbx_to_ttx_4f_nodipole` | `b b~ > t t~ NP<=1` | 59 | cause 1 (`V_753`, both pairings) | 1.6e-15, gated |
| `uux_to_bbxg_4f_nodipole` | `u u~ > b b~ g NP<=1 SMHLOOP=0` | 155 | cause 2 (`V_787`/`V_788` re-rooted) | 1.7e-15, gated |
| `uux_to_ttxccx_4f_nodipole` | `u u~ > t t~ c c~ NP<=1 SMHLOOP=0` | 1325 | six quarks, all contacts, no t-b charged current | 6.1e-15, gated |
| `uux_to_ttxbbx_4f_nodipole` | `u u~ > t t~ b b~ NP<=1 SMHLOOP=0` | (2113 with SMHLOOP) | the target | 0.63, `KNOWN_DISAGREEMENT` (Found 2) |

Each fix's isolation was measured with a card that switches on the one
operator involved. With `cQtQb1Re` and `cQtQb8Re` only (on top of
`restrict_SMlimit_massless`), `b b~ > t t~ NP<=1` (7 MG diagrams: `V_753`
plus SM) agrees at 9.8e-16. Under `vg_nodipole` it agrees at 1.3e-15, and
`u u~ > b b~ g` at 1.3e-15 (zero widths, seed 5199).

`uux_to_bbxg_4f_nodipole` moved from seed 5109 to 5112. At 5109, point 0 had
the gluon nearly collinear with the incoming quark (`t(u,g)` = −788 GeV² at
√s = 1 TeV). One entry agreed only to 1e-11 of itself, which is 6.3e-13 of
the largest amplitude: 1.6× below `REL_TOL`, with `SMHLOOP=0` already
applied. I read that as rounding amplified by the near-collinear
cancellation. That is a reading, not a measurement of where the digits went.

**Coverage bookkeeping.** These are standalone rows, which (like every other
row of `gen_standalone_jamps.py`) have no `validation/manifest.toml` row and
no entry in `MG_VALIDATED_PROCESSES`. The SMEFTsim op census
(`smeftsim::gated_rows_op_census`) reads the manifest's gated rows, so it does
not see them.

### What each test cannot see

- The standalone gate cannot see a sign shared by every diagram (G is fitted),
  nor how a flow splits into diagrams. Per diagram it sees nothing directly. It
  is per flow, and a multi-flow table banks no `AMP()`. Two compensating
  defects in diagrams that reach the same flows with the same weights at every
  banked point would pass. It also cannot see points outside the 3 banked
  ones.
- `crossed_four_quark_contact_compiles` sees compilation and the flow count,
  not values.
- `uncoupled_color_and_flow_group_is_refused` pins the refusal on one vertex.
  It cannot see whether the colour walk ever reaches the refusal; the compile
  test covers that.
- `four_fermion_current_rerooted_across_a_line_reads_its_partner` pins the
  input positions of one structure at one rooting. It cannot see the
  current's value; `uux_to_bbxg_4f_nodipole` does.
- `rerooting_onto_an_adjacent_partner_keeps_the_tree` compares the relabelled
  tree with the old compaction over every fermion-output rooting of every
  tree-rooted term of the SMEFTsim, toy and toy-Majorana UFOs, with each line
  bound along and against its arrow and the output bound as its partner, as
  `root_diagram` binds it (218 adjacent re-rootings identical, 24 across
  another leg). It cannot see the tensor path, which still compacts over the
  output and never re-roots. It also cannot see rootings whose binding pattern
  `root_diagram` never produces.

### Unchanged banked layer

`validate.sh` results are recorded under "Gates" below.

## Gates

Run on the merged head `da1b96e` (this branch merged with `origin/sprintj-sde`),
with `CARGO_TARGET_DIR=/home/user/cargo-target-je1 CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

```
$ cargo fmt --all --check                                   # rc=0
$ cargo build --workspace                                   # Finished, rc=0
$ cargo clippy --workspace --all-targets -- -D warnings     # Finished, rc=0
$ cargo test --workspace --no-fail-fast                     # rc=0
  58 test binaries "ok"; 2076 passed, 0 failed, 26 ignored
$ python3 scripts/kb.py lint
  1260 concepts, 434 backlog items; 0 errors, 5 warnings    # the 5 warnings predate the branch
```

The first workspace run failed `smeftsim::vendored_copy_matches_its_manifest`
on a `__pycache__` that an aborted attempt of mine to import the UFO with the
system Python had left in `validation/ufo/SMEFTsim_topU3l_MwScheme_UFO/`.
Removing it, the rerun above is clean.

## Found

1. **A dipole operator on an initial-state quark line disagrees with
   MadGraph.** Restrict cards with one coefficient on, all else from
   `restrict_massless` zeroed, standalone per flow × helicity:

   | coefficient | process | worst flow |
   |---|---|---|
   | `ctWRe` | `b b~ > t t~ NP<=1` | 2.1e-3 |
   | `cbGRe` | `b b~ > t t~ NP<=1` | 2.0e-3 |
   | `cbWRe` | `b b~ > t t~ NP<=1` | 8.7e-3 |
   | `cbBRe` | `b b~ > t t~ NP<=1` | 2.5e-5 |
   | `ctGRe`, `ctBRe` | `b b~ > t t~ NP<=1` | ≤ 1.6e-15 (pass) |
   | `cbBRe` | `b b~ > e+ e- NP<=1` | 2.9e-4 |
   | `ctBRe` | `t t~ > e+ e- NP<=1` | 4.6e-3 |
   | `cbBRe` | `e+ e- > b b~ NP<=1` | 1.9e-15 (pass) |
   | `cbGRe` | `b b~ > e+ e- NP<=1` | 1.6e-15 (pass: no gluon dipole diagram) |

   The minimal reproducers are `b b~ > e+ e- NP<=1` under `cbBRe` alone
   (2.9e-4) and `t t~ > e+ e- NP<=1` under `ctBRe` alone (4.6e-3). The mirror
   `e+ e- > b b~` agrees. The other groups of `restrict_massless`
   coefficients each agree on `b b~ > t t~` at ≤ 1.6e-15: bosonic, Yukawa,
   Higgs currents, all four-quark, and leptonic. Hypothesis, unverified: the
   dipole structures (`FFV2`, `FFV9`, `FFV10`, `P(-1,3)`-weighted gamma
   chains) disagree when the vertex holding the leg-0 quark is the amplitude
   root and the boson enters as an internal current. That would mean the
   momentum convention of `P` on an internal vector input of an amplitude
   root. A dipole rooted as a current agrees. Why no existing gate sees it:
   every dipole row (`ee_to_ttx_dipole` under `vg_ctdipole`, `ee_to_ttx_smeft`
   under `massless`, `ll_to_qqx_toy_dipole`) has the lepton at leg 0 and the
   dipole on the final-state line, so the dipole is always rooted as a
   current. No standalone row switches a dipole on. To close it: per diagram
   on `b b~ > e+ e- NP<=1` under `cbBRe` alone, then a fix with that row
   gated.

2. **A W between the t and b lines beside a third fermion line takes the
   opposite sign to MadGraph's, in the Standard Model.** MadGraph's own `sm`
   (massive t and b), standalone, zero widths:

   | process | MG diagrams | worst flow | G |
   |---|---|---|---|
   | `u u~ > t t~ b b~ QED<=2` | 39 | 4.95e-1 (fail) | |
   | `u u~ > t t~ b b~ QED<=2 / z a h` | 11 | 4.96e-1 (fail) | |
   | `u u~ > t t~ b b~ QED=2 QCD=2 / z a h` (the W diagrams) | 4 | 1.8e-15 | −i |
   | `u u~ > t t~ b b~ QED<=2 / w+ w-` | 35 | 8.7e-16 | +i |
   | `u u~ > t t~ c c~ QED<=2` | 31 | 9.1e-16 | +i |

   The same holds under SMEFTsim's `restrict_SMlimit_massless`:
   `QED<=2` (41 diagrams) fails at 0.495, while the QCD-only process passes.
   The two-line `b b~ > t t~` with its t-channel W agrees. So the
   W-exchange class carries an overall −1 against the rest when a third
   fermion line is present. This is a silent wrong answer on any card that
   admits a W between a t line and a b line beside another quark line, such
   as `p p > t t~ b b~ QED<=2`. Default-order (QCD-only) `t t~ b b~` is
   unaffected. I dumped the class-level G split, not individual diagram
   pairs. To close it: a per-diagram dump of the 11-diagram
   `/ z a h` process against MadGraph's `AMP()`, then the GF(2) fermion-sign
   method of `validation/bit-exact-amplitude-debugging`. The target row
   `uux_to_ttxbbx_4f_nodipole` is listed in `KNOWN_DISAGREEMENT` and fails
   the moment it agrees, so it flips with the fix.

3. **The backlog item's `closes_when` is met with an attributed exception.**
   The target compiles, and the cause is named (two causes). Per-diagram
   agreement with MadGraph holds on the no-dipole card for every row the
   fixes reach. The target's own disagreement is Found 2, and the dipole
   disagreement on the full card is Found 1. Both are named and filed by the
   manager. The comparison is per flow and per helicity, the finest level the
   standalone gate exposes for a multi-flow process.

4. **The brief's premise.** The item located the empty sum in
   "`lower::lower_flows` / `lower::optimize`, or rooting". It came from
   colourisation, through `lower_vertex`, and a second, independent rooting
   defect sat behind it. The item read the trigger as "most likely the
   four-quark contact operators". That holds for both causes, but through two
   different contacts.
