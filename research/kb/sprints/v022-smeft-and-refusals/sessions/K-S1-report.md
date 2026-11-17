---
type: Session Report
title: "K-S1 report: the coloured-scalar seagull colour structure"
description: "The squark-squark-gluon-gluon contact's colour T(1,-1,3)*T(2,4,-1) now compiles as one generator product crossed between its two slots; g g > t1 t1~ and u u~ > t1 t1~ match MadGraph standalone per flow, the contact's per-structure flow assignment and CF match matrix.f, and the 0-jet SMS-StopStop_mStop-600 sigma agrees with MadEvent to 0.01%."
status: draft
tags: [color, ufo, bsm, mssm, cms-gridpacks, madgraph-parity, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-17}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-seagull`, branch `sprintk-seagull`, based on
`main` at `703f57f`. MadGraph is the pinned 3.7.1 submodule, run with the main
checkout's `madgraph` pixi environment. MadGraph work areas are under
`/home/user/wt/sprintk-seagull-mg/`. The host has 4 cores, shared with the
sprint manager's verification run; MadEvent ran on 2.

| commit | what |
|---|---|
| `9be3577` | wip(color): read generator chains between a 3 and a 3̄ slot (checkpoint) |
| `951a2f6` | fix(color): cross a vertex's generator chain as one generator; MSSM standalone tables and the per-structure test |
| (final) | this report, the backlog item deleted |

## The refusal, reproduced

On the base source, dev build:

```
$ cat gg_t1t1x_proc_card.dat
import model MSSM_SLHA2
generate g g > t1 t1~
$ vibegraph integrate gg_t1t1x_proc_card.dat --ufo-dir research/refs/mg5amcnlo/models \
      --neval 1000 --fixed-budget --niter 1 --no-network --no-tui
 INFO enumerate: 6 diagrams for g g > su3 su3~
ERROR cannot compile g g > su3 su3~ for the amplitude probe: amplitude compilation failed:
  unsupported color structure 'T(-1,3) in 'T(1,-1,3)*T(2,4,-1)' is not a 3/3̄ slot pair'
```

`check_slot_reps` required both fundamental indices of every `T` to be
interaction slots of the right representation, so a summed index between two
generators of one vertex could not pass. After the fix the same command
reports `2 colour flows, CF matrix 2×2` and `compiled evaluator: 115 ops`.

## What changed

All in `vibegraph-lib/src/helas/color/colorize.rs`:

- `check_slot_reps` checks the `3`/`3̄` slot of a `T` index only when the index
  is a slot (positive).
- `check_generator_chains` admits a summed fundamental index only as a link of
  an **open generator chain** `T(a,i,-1)·T(b,-1,-2)·…·T(z,-n,j)` from a `3`
  slot `i` to a `3̄` slot `j`: it must occur exactly once as one `T`'s
  antifundamental index, once as another's fundamental index, and nowhere else.
  It refuses, by name: a closed loop of generators (a trace written with `T`s,
  `T(1,-1,-2)*T(2,-2,-1)` or `T(1,-1,-1)`); a summed index repeated in one
  position; a summed index shared with an `Epsilon`, `K6` or other atom, or with
  an adjoint position. Chains of any length are covered.
- `merge_generator_chains`, called first in `convert_expr`, replaces each chain
  with the single `T(a,b,…,z,i,j)` it writes (adjoint indices in matrix-product
  order). The slot crossing (`T(adj,i,j)` read as `T(adj,j,i)`) is then applied
  to the merged generator.

### The convention, and the oracle that caught the first version

The first version (commit `9be3577`) left the chain to the algebra engine and
crossed each link on its own. That reverses the product: `T(1,-1,3)*T(2,4,-1)`,
which is `(T^2 T^1)` from slot 4 to slot 3, came out as the key `T(1,2,3,4)`.
MadGraph puts that structure on `T(2,1,3,4)`. The per-structure test failed
against it:

```
$ cargo test -p vibegraph-lib --features extended-validation --test standalone_jamps mssm
gg_to_t1t1x_mssm: 24 flow entries, G = Complex { re: 4.944441865318248e-35, im: 0.9999999999999993 }, worst flow 3.440e-16, worst |M|² 6.055e-16
test gg_to_t1t1x_mssm ... ok
thread 'mssm_contact_structures_reach_madgraphs_flows' panicked at vibegraph-lib/tests/standalone_jamps.rs:1376:9:
assertion `left == right` failed: flow 0 (MadGraph's JAMP(1)) takes structure 1
  left: [0]
 right: [1]
test mssm_contact_structures_reach_madgraphs_flows ... FAILED
```

The per-flow JAMP comparison passed at 3.4e-16 while the assignment was
wrong, which is the blind spot predicted below. The slot crossing exchanges
which of a `T`'s two slots MadGraph reads as the fundamental end. A summed
index inside one vertex is not a slot, so MadGraph does not cross it: it reads
the whole chain as one generator product between its two slots taken the
other way round, with the adjoint order kept. A summed index between two
vertices is a propagator slot at both ends, so a chain across vertices is
crossed link by link, as before; the t- and u-channel stop diagrams of the
same table, whose flows differ in value, pin that part.

## Oracles

### Per-flow complex JAMPs against MadGraph standalone

Two new rows in `validation/madgraph/gen_standalone_jamps.py`, generated with
the pinned MadGraph. A new `bundled` Row field names a model that ships with
the pinned MadGraph (`research/refs/mg5amcnlo/models/`): MadGraph imports it as
`MSSM_SLHA2-default`, and the table's `ufo` paths point at the submodule's
model directory and `restrict_default.dat`.

```
$ PATH=/home/user/vibegraph/.pixi/envs/madgraph/bin:$PATH \
    python validation/madgraph/gen_standalone_jamps.py gg_to_t1t1x_mssm \
    --work /home/user/wt/sprintk-seagull-mg/standalone --amps
wrote validation/madgraph/standalone/gg_to_t1t1x_mssm.json: 3 points x 4 helicities x 2 flows
wrote validation/madgraph/standalone/uux_to_t1t1x_mssm.json: 3 points x 4 helicities x 2 flows
```

Both rows zero every width (the t-channel stop is spacelike). The Rust tests
read the MSSM from the submodule, so they are compiled under the
`extended-validation` feature (`validation/manifest.toml`'s
`standalone-jamps` entry records it):

```
$ cargo test -p vibegraph-lib --features extended-validation --test standalone_jamps mssm -- --nocapture
gg_to_t1t1x_mssm: 24 flow entries, G = Complex { re: 4.944441865318248e-35, im: 0.9999999999999993 }, worst flow 3.440e-16, worst |M|² 6.055e-16
test gg_to_t1t1x_mssm ... ok
uux_to_t1t1x_mssm: 24 flow entries, G = Complex { re: 0.0, im: -0.9999999999999991 }, worst flow 7.403e-16, worst |M|² 1.597e-15
test uux_to_t1t1x_mssm ... ok
test mssm_contact_structures_reach_madgraphs_flows ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 75 filtered out; finished in 0.27s
```

`u u~ > t1 t1~` is the control without the contact. At MadGraph's default
coupling orders it is one graph (the s-channel gluon; `NGRAPHS=1`,
`JAMP(1) = -1/6 AMP(1)`, `JAMP(2) = 1/2 AMP(1)`), so it checks the
squark-gluon vertex's colour and normalisation and little else.

**What it cannot see.** The fitted global phase `G`. The split of a flow into
diagrams. Above all, **which flow each of the contact's two colour structures
reaches**: in MSSM_SLHA2 both structures carry the same Lorentz structure
(`VVSS1`) and coupling (`GC_90`), so the contact is
`A·(T^aT^b + T^bT^a)` and its two structures are equal at every point. The
measured passes of the per-link version above show this directly.

### The contact's per-structure flow assignment and the CF matrix against `matrix.f`

`mssm_contact_structures_reach_madgraphs_flows` (in `standalone_jamps.rs`)
compares against MadGraph's standalone `matrix.f` for `g g > t1 t1~`:

```
CALL VVSS1_0(W(1,1),W(1,2),W(1,4),W(1,3),GC_90,AMP(1))
CALL VVSS1_0(W(1,1),W(1,2),W(1,4),W(1,3),GC_90,AMP(2))
JAMP(1) = AMP(2)+((0,-1))*AMP(3)+AMP(4)+AMP(5)
JAMP(2) = AMP(1)+((0, 1))*AMP(3)+AMP(6)+AMP(7)
DATA DENOM/3/
DATA (CF(I),I=  1,  2) /16,-4/      C 1 T(1,2,3,4)
DATA (CF(I),I=  3,  3) /16/         C 1 T(2,1,3,4)
```

MadGraph writes one `AMP()` per colour structure of the contact, in sorted
coupling-key order (`helas_objects.py`: `keys = sorted(inter.get('couplings').keys())`),
so `AMP(1)` is structure 0 and enters `JAMP(2)`, `AMP(2)` structure 1 into
`JAMP(1)`. The test asserts: 6 diagrams, one contact; NCOLOR 2;
`CF = [[16, -2], [-2, 16]]/3` (the packed form doubles the off-diagonal);
structure 1 alone from the contact on flow 0 and structure 0 alone on flow 1,
each with coefficient 1. It fails on the per-link crossing (above) and passes
on the merged one. The unit test `a_generator_chain_crosses_as_one_generator`
pins the same key at the algebra level, and that the chain and the single
generator it writes (`T(2,1,4,3)`) cross to the same key.

**What it cannot see.** It is one process with one contact vertex; a chain of
three or more generators is checked only by the unit tests
(`generator_chains_between_slots_are_read`), not against MadGraph, since no
model in reach writes one. CF is a Gram matrix and is blind to a uniform
transpose of the basis; the per-structure assertion is what sees it here.

### Diagram census and the stop-mixing gluon couplings

Both sides draw the same graphs: MadGraph's seven `AMP()` are this side's six
diagrams with the contact counted once per colour structure. Two of the five
propagator graphs reach the heavy stop `su6` through the gluon vertices
`V_231` (`g su3~ su6`, `GC_57`) and `V_234` (`g su3 su6~`, `GC_63`):
`GC_57 = -i G (Ru6x3·Ru3x3* + Ru6x6·Ru3x6*)`, the off-diagonal element of
`Ru Ru†` over the stop block, which vanishes by unitarity. MadGraph keeps them
too (`AMP(5)`, `AMP(7)`, with `GC_57`). `restrict_default` cannot drop them:
its `USQMIX` entries (0.5536450, 0.8327528) are each non-zero, and only the
combination vanishes. With the card's values the two products are the same
two doubles multiplied in either order, so the sum is exactly zero in IEEE
arithmetic on both sides. That is derived from the card and the coupling
expression, not measured as a coupling value; the per-flow match at 3.4e-16
says the two diagrams contribute alike on both sides.

### σ: the 0-jet piece of SMS-StopStop_mStop-600 against MadEvent

**Substitution, accepted by the sprint manager.** The production cards are in
`cms-sw/genproductions` at commit `a7fbf78`, under
`bin/MadGraph5_aMCatNLO/cards/production/13p6TeV/SUSY_SMS/SMS-StopStop/templatecards/`
(read from a blobless git clone; the `raw.githubusercontent.com` paths the
brief guessed return 404). The gridpack is MLM-matched `p p > t1 t1~` plus up
to two jets (`ickkw = 1`, `xqcut = 30`). This comparison integrates its 0-jet
piece only:

- proc card: `import model MSSM_SLHA2`, the card's `p` (five flavours),
  `generate p p > t1 t1~`;
- param card: MadGraph's generated card for the process with the card's
  `customizecards` masses applied, mStop = 600 GeV, every other sparticle at
  1e5 GeV, `MB = 0` (`1000039` is not a MASS entry of MSSM_SLHA2 and is
  skipped, as MadGraph skips it); both sides read the same file;
- run card: the CMS template with `ickkw = 0`, `xqcut = 0`, `lhaid = 247000`
  (NNPDF23_lo_as_0130_qed, fetched here, in place of 325300),
  `use_syst = F`, `gridpack = .false.`, `nevents = 10000`, and the
  `$DEFAULT_PDF_MEMBERS = reweight_PDF` template line removed (vibegraph
  refuses `reweight_PDF` as an unknown key).

Drivers: `ks1_mg_sigma.sh` (one fresh copy of the generated process directory
per seed, `iseed` set, `nb_core = 2`, each run's banner checked for its seed and
its event file for 10000 events) and `ks1_vg_sigma.sh` (`vibegraph integrate`,
default budget, `--seed N -j 2`), in `/home/user/wt/sprintk-seagull-mg/sigma/`.

```
MadEvent (pb):  1 0.15224 ±0.00016   2 0.15167 ±0.00018   3 0.15227 ±0.00016
                4 0.15217 ±0.00018   5 0.15214 ±0.00021
vibegraph (fb), every seed ±0.091:
  151.927 152.137 152.203 151.937 151.955 152.226 151.951 152.091 152.138 152.066
  152.217 152.081 152.223 152.125 152.100 151.966 152.227 151.952 152.164 152.001

MG: n=5  mean=152.0980 fb  sd=0.2449  sem=0.1095  mean quoted err=0.1787  chi2/dof=1.91
VG: n=20 mean=152.0844 fb  sd=0.1088  sem=0.0243  mean quoted err=0.0910  chi2/dof=1.43
VG-MG = -0.0136 fb (-0.009%), pull -0.12 sigma (sem-combined 0.1122)
```

Unweighted means, spread from the sample standard deviation, χ²/dof of each
side's seeds about its own mean with their quoted errors. MadEvent's seed 2 is
the low one (151.67 fb, 2.4σ of its quoted error below the others) and carries
most of its χ²/dof; vibegraph's 20 seeds scatter 1.2× their quoted error.
Each vibegraph seed met its 0.1% target in 6 iterations, 720k evaluations,
about 7 s.

**What it cannot see.** σ is a sum over helicities and flows and is blind to
the contact's flow assignment, as the JAMPs are. It covers the 0-jet piece
only: the matched one- and two-jet pieces, where the contact sits beside an
emission, are not integrated here. The PDF set and `ickkw` differ from the
gridpack's.

## Gates

Run on the head of `951a2f6` plus this report, with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2`.
`validation/validate.sh` was not run locally; CI's banked job is that gate.

```
$ cargo fmt --all --check                                    # clean
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 1m 12s   # rc=0
$ cargo test --workspace
58 test binaries "test result: ok", summed: passed 2117, failed 0, ignored 27   # rc=0
$ cargo test -p vibegraph-lib --features extended-validation --test standalone_jamps mssm
test result: ok. 3 passed; 0 failed
$ cargo test -p vibegraph-lib --features extended-validation --test color_cf_oracle
test result: ok. 112 passed; 0 failed; 0 ignored
$ pixi run kb-lint
1297 concepts, 456 backlog items; 0 errors, 5 warnings      # the warnings are on other items
```

`color_cf_oracle` reads every committed MadEvent run's CF matrix and per-graph
JAMP coefficients; no committed model writes a summed fundamental index inside
a vertex, so the merge leaves every existing row unchanged there.

## Found

- **The `T` crossing's documentation describes conjugation, and the code does
  slot exchange.** `convert_expr`'s doc says reading a `T` straight through
  would complex-conjugate the colour string, and that the transpose undoes it.
  For one generator the two are the same. For a product they differ, and the
  measurement here (the contact against `matrix.f`) says MadGraph's reading is
  the slot exchange with the adjoint order kept, which is also what the code
  does for a single `T` with several adjoint indices. This session's interim
  hand-back's worry that a multi-adjoint `T(a,b,i,j)` atom is crossed wrongly is
  therefore not supported: the merged chain and the single atom cross alike
  (unit-tested), and the chain's crossing is the one MadGraph confirms. No
  vendored model writes a multi-adjoint `T` atom (grep over `validation/ufo` and
  the submodule's `models/`), so no MadGraph oracle reaches that atom directly.
  What would close it: reword the doc in terms of slot exchange, and, if a model
  with a multi-adjoint `T` appears, a standalone row on it.
- **A contact whose colour structures carry different couplings is untested
  against MadGraph.** Every vendored model writes the squark-gluon contact with
  one coupling on both structures, so only the colour-level test sees the
  assignment. A toy UFO vertex with two different couplings on
  `T(1,-1,3)*T(2,4,-1)` and `T(1,4,-1)*T(2,-1,3)` would put it in the per-flow
  gate.
- **The matched SMS-StopStop card is not integrated.** The 0-jet piece agrees;
  the full card (`ickkw = 1`, `xqcut = 30`, one and two extra jets) needs an MLM
  run against MadEvent, which is the item's `closes_when` gridpack in its
  production form.
