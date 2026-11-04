---
type: Session Report
title: "H-G0 report: why crossing pairs refuse a shared colour basis"
description: "The charge-conjugate pairs have one basis up to a permutation, and the raw-order CF check refuses them before flow_permutation, which finds it, can run. The Singlet-model refusal is 184 zero-coupling b-Yukawa diagrams in the member."
status: draft
tags: [cms, flavour-groups, colour, crossing, sprint-h, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: campaign, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6605096410", title: "Campaign main-23d3a76-200-pc on #71"}
  - {id: mg-group, resource: "../../../references/codebases/madgraph5-amcnlo.md", title: "MadGraph at the pinned commit b7687064 (research/refs/mg5amcnlo)"}
---
Session H-G0, diagnosis only. No source change is committed. The probe is a
`#[cfg(test)]` child module of `proton.rs`, and it is on the local branch
`sprinth-g0-probe`, not on `sprinth-g0`.

## Verdict per pair

| pair | refusal | class | one group possible? |
|---|---|---|---|
| `u u > g g u u` / `u~ u~ > g g u~ u~` | `ColorStructureDiffers` | **(a)**: the same basis, permuted by charge conjugation | yes, with π = `[7,3,5,1,8,10,9,11,6,2,4,0]` |
| `u b > b g u` / `u~ b~ > b~ g u~` | `ColorStructureDiffers` | **(a)**, same mechanism | yes, with π = `[0,2,3,1]` |
| `d d > H H d d` / `b b > H H b b`, MB = ymb = 0 from the card | `ColorFlowPairing`, "flow 1 of the representative has no counterpart" | **neither (a), (b) nor (c)**: identical basis, keys, tags and CF; different *diagram sets* (8 against 192) | not by the structural pairing; see below |

The (b) case in the brief, "3 ↔ 3̄ swapped, CF conjugated or transposed", is
the mechanism behind (a), not a separate class. Charge conjugation maps a key
`T(a₁…aₙ, i, j)` to `T(aₙ…a₁, j, i)`. That gives another key of the conjugate's
basis, so it acts on the sorted basis as a permutation. CF is real and
symmetric, so conjugation and transposition leave it unchanged. What changes is
only where each flow sits in the sorted order.

## Commands

The probe is `vibegraph-lib/src/proton_g0_probe.rs`, included from `proton.rs`
as `#[cfg(test)] #[path = "proton_g0_probe.rs"] mod g0_probe;`. It calls the
production `compile_class`, `flow_permutation`, `probe_momenta` and
`derive_flavor_groups`.

```
cd /home/user/wt-g0
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -j 2 -p vibegraph-lib --lib g0_probe -- --nocapture --test-threads 1
# test result: ok. 4 passed; 0 failed; ... finished in 0.75s
```

For each pair the probe prints:

- per member: n_diagrams, n_flows, leg reps, and per flow the basis key, the
  `ColorFlowTags`, the tags canonicalised by first appearance of each line, the
  number of contributions, and the diagrams it touches; then the CF matrix;
- `|M|²` worst relative difference on the grouping probe points;
- raw `n_flows` and `cf_matrix` equality, and `flow_permutation(rep, member)`;
- an independent pairing: canonical line topology, direct and with both
  `ICOLUP` slots swapped on every leg (the full-conjugation rule of
  `amplitudes/colour-flow-lines-and-conjugation`). Under that pairing it checks
  `CF_rep[i,j] = CF_mem[p(i),p(j)]`, fingerprint equality, helicity-summed
  `JAMP2` per flow, and per-helicity `|JAMP|` with the member's helicity taken
  as `h` and as `−h`;
- the number of permutations `p` with `CF_rep = P·CF_mem·Pᵀ`, the
  automorphisms of `CF_rep`, and duplicate fingerprints within each basis;
- the `derive_flavor_groups` outcome.

The MadGraph side is `python3 /home/user/vibegraph/research/refs/mg5amcnlo/bin/mg5_aMC <card>`.
The submodule is at `b7687064`, the pin. The run directories are under the
session scratchpad as `H-G0-*`:

| run | card |
|---|---|
| `H-G0-qcd2` | `import model sm-no_b_mass; define q = u u~; generate q q > g g q q; output` |
| `H-G0-qcd3` | `import model sm-no_b_mass; define q = u b u~ b~; generate q q > q g q; output` |
| `H-G0-vbf2` | `import model sm; define q = d b; generate q q > h h q q QCD=0; output` |
| `H-G0-vbf3` | `import model sm-no_b_mass; define q = d b; generate q q > h h q q QCD=0; output` |
| `H-G0-qcd`, `H-G0-vbf` | the same pairs as separate `add process` lines. MadGraph never groups across process ids, so these land in separate P-dirs and say nothing about grouping. |

## Dumps

### `u u > g g u u` (rep) / `u~ u~ > g g u~ u~` (member), `sm-no_b_mass`

Both have 72 diagrams and 12 flows. The rep's keys, in order, are MadGraph
`matrix1`'s keys in `P1_qq_ggqq`; the member's are `matrix3`'s:

```
rep flow  0  T(3,4,5,1) T(6,2)      member flow  0  T(1,5) T(3,4,2,6)
rep flow  1  T(3,4,5,2) T(6,1)      member flow  1  T(1,5) T(4,3,2,6)
rep flow  2  T(3,4,6,1) T(5,2)      member flow  2  T(1,6) T(3,4,2,5)
rep flow  3  T(3,4,6,2) T(5,1)      member flow  3  T(1,6) T(4,3,2,5)
rep flow  4  T(3,5,1) T(4,6,2)      member flow  4  T(2,5) T(3,4,1,6)
rep flow  5  T(3,5,2) T(4,6,1)      member flow  5  T(2,5) T(4,3,1,6)
rep flow  6  T(3,6,1) T(4,5,2)      member flow  6  T(2,6) T(3,4,1,5)
rep flow  7  T(3,6,2) T(4,5,1)      member flow  7  T(2,6) T(4,3,1,5)
rep flow  8  T(4,3,5,1) T(6,2)      member flow  8  T(3,1,5) T(4,2,6)
rep flow  9  T(4,3,5,2) T(6,1)      member flow  9  T(3,1,6) T(4,2,5)
rep flow 10  T(4,3,6,1) T(5,2)      member flow 10  T(3,2,5) T(4,1,6)
rep flow 11  T(4,3,6,2) T(5,1)      member flow 11  T(3,2,6) T(4,1,5)
```

First rows of CF (exact, Nc = 3):

```
rep    row 0: 16 16/3 16/3 2 0 16/3 -2/3 0 -2 -2/3 -2/3 2
member row 0: 16 -2 16/3 -2/3 16/3 -2/3 2 2 0 -2/3 16/3 0
```

MadGraph's `DATA CF` packs the upper triangle with off-diagonal entries
doubled, so its `matrix1` row 1, `48,32,32,12,0,32,-4,0,-12,-4,-4,12 / 3`, is
the same row.

```
|M|² worst rel over 20 probe points: 4.073e-16
n_flows equal: true
raw cf_matrix equal: false
flow_permutation(a,b): Ok([7, 3, 5, 1, 8, 10, 9, 11, 6, 2, 4, 0])
line-topology matching, direct: [[], [], [], [], [], [], [], [], [], [], [], []]
line-topology matching, charge-conjugated (c<->a on every leg): [[7], [3], [5], [1], [8], [10], [9], [11], [6], [2], [4], [0]]
  -> permutation [7, 3, 5, 1, 8, 10, 9, 11, 6, 2, 4, 0]; CF_a[i,j] == CF_b[p(i),p(j)]: true
  -> fingerprint_a[i] == fingerprint_b[p(i)]: [true × 12]
  -> JAMP2_a[f] vs JAMP2_b[p(f)] worst rel: 3.973e-16
  -> |JAMP_a(h,f)| vs |JAMP_b(h,p(f))| worst rel: 4.796e-14; vs |JAMP_b(-h,p(f))|: 0.000e0  (24 vs 24 helicity combos)
permutations p with CF_a == P CF_b P^T: 16, first Some([0, 2, 4, 6, 11, 10, 9, 8, 1, 3, 5, 7])
automorphisms of CF_a: 16
duplicate fingerprints within a: []   within b: []
derive_flavor_groups: Err: ... share a matrix element but not a colour basis ...
```

By hand, rep flow 0 `T(3,4,5,1)T(6,2)` conjugates to `T(4,3,1,5)T(2,6)`, which
is member flow 7, so π(0) = 7. Rep flow 11 and member flow 0 touch the same
diagram set, `{1,22,24,31,33,34,44,45,46,48,51,52,54,69,71}`, so π(11) = 0.

### `u b > b g u` (rep) / `u~ b~ > b~ g u~` (member), `sm-no_b_mass`

Both have 5 diagrams and 4 flows.

```
rep    0 T(3,1)T(4,5,2)   1 T(3,2)T(4,5,1)   2 T(4,3,1)T(5,2)   3 T(4,3,2)T(5,1)
member 0 T(1,3)T(4,2,5)   1 T(1,5)T(4,2,3)   2 T(2,3)T(4,1,5)   3 T(2,5)T(4,1,3)
CF rep    = [[12,4,0,4],[4,12,4,0],[0,4,12,4],[4,0,4,12]]
CF member = [[12,4,4,0],[4,12,0,4],[4,0,12,4],[0,4,4,12]]
|M|² worst rel: 3.528e-16; raw cf_matrix equal: false
flow_permutation(a,b): Ok([0, 2, 3, 1])
line-topology matching, charge-conjugated: [[0], [2], [3], [1]]; CF under it: true; fingerprints under it: all true
JAMP2 under it: 2.463e-16; |JAMP(h)| vs |JAMP(-h)|: 0.000e0 (8 combos)
permutations with CF_a == P CF_b P^T: 8; automorphisms of CF_a: 8
derive_flavor_groups: Err: ... share a matrix element but not a colour basis ...
```

### `d d > H H d d` / `b b > H H b b`, QCD = 0

The Singlet UFO is not available in this container. The probe uses the SM,
which has the same colour structure on the quark lines, in the configuration
the campaign ran: the model's default restriction, so MB and ymb are live
parameters and the b-Yukawa vertices exist, with a param card that sets
`MASS 5 = YUKAWA 5 = 0`, and `maxjetflavor = 5`.

```
member 0 d d > H H d d:   n_diagrams 8    n_flows 2
  flow 0 T(5,1)T(6,2)  4 contributions, diagrams {0,2,3,4}
  flow 1 T(5,2)T(6,1)  4 contributions, diagrams {1,5,6,7}
member 1 b b > H H b b:   n_diagrams 192  n_flows 2
  flow 0 T(5,1)T(6,2)  96 contributions;  flow 1 T(5,2)T(6,1)  96 contributions
CF both = [[9,3],[3,9]]
|M|² worst rel: 0.000e0; raw cf_matrix equal: true
flow_permutation(a,b): Err("flow 1 of the representative has no counterpart in the member's basis")
line-topology matching, direct: [[0], [1]] -> identity; CF: true; fingerprints: [false, false]
JAMP2 under identity: 0.000e0; |JAMP(h)| under identity: 0.000e0 (6 combos)
duplicate fingerprints within a: []   within b: []
derive_flavor_groups: Err: ... their colour flows cannot be paired up: flow 1 of the
  representative has no counterpart in the member's basis ...
```

This is the campaign's reason string, reproduced exactly. Two controls:

- **With `maxjetflavor = 4`** (the `RunCard` default), the same pair is refused
  one step earlier, by `CutIndicatorDiffers`, because a b is not a jet.
- **Under the `no_b_mass` restriction** the Yukawa vertices are gone. Both
  members then have 8 diagrams, `flow_permutation` returns `Ok([0, 1])`, and
  `derive_flavor_groups` returns **Ok, 1 group** at `maxjetflavor = 5`.

MadGraph's census agrees with ours: `NGRAPHS=192` for `b b > h h b b` at
`import model sm` (`H-G0-vbf2/P1_bb_hhbb`), and 8 for `d d > h h d d`.

## Classification and root cause

### The QCD crossing pairs: class (a), refused by a check order, not by the physics

The two subprocesses carry the same colour basis. The member's sorted list is
the conjugate of the rep's, so the same flows sit at different indices.
`derive_flavor_groups` (`vibegraph-lib/src/proton.rs:866–875`) compares
`cf_matrix()` element by element in each member's own index order, and it does
so **before** `flow_permutation` (`:892`) is called. `flow_permutation` exists
because the orders differ, and on both pairs it returns the correct π: three
oracles below agree on it.

The two-flow conjugate pairs the code already groups (`g u > g u` /
`g u~ > g u~`, flow index reversed) pass the raw check only by accident: any
2×2 CF `[[a,b],[b,a]]` is invariant under the swap. The 4- and 12-flow bases
have no such luck.

So, on these two pairs, nothing about the bases differs. The refusal is the
raw-order comparison.

### The Singlet case: identical bases, different diagram sets

`flow_permutation` returns `no counterpart` for flow 1. It does **not** report
duplicate fingerprints: neither basis has any. The keys, tags, CF, and per-flow
and per-helicity JAMPs agree under the identity. The fingerprint
(`compile.rs:802`, `(diagram index, chain, Nc power, |q|)` per contribution)
differs because `b b > H H b b` carries 184 extra diagrams. These are H
radiated off the b lines, and their `ymb`-proportional couplings are exactly
zero at this card. They add contributions to both flows and shift the diagram
indices. Under the `no_b_mass` restriction the same pair groups cleanly.

This is a restriction-versus-param-card issue: the card zeroes a parameter
that the restriction left live, and zero-coupling diagrams are not pruned
after the card is applied. It is not a colour issue. To the brief's question,
the two subprocesses have the same topology and the same colour; the member
also has extra topologies whose value is zero.

## What MadGraph does (pinned `b7687064`)

MadGraph uses two levels, and only the second shares a colour basis.

1. **SubProcessGroup (P-dir).** `group_subprocs.py:454–512`,
   `find_process_classes`, `criteria="madevent"` (`:483–489`): the initial
   state is classed by `is_fermion()` only (the `is_part` term is commented
   out), so q and q̄ share a class. Every leg is classed by mass *name*, spin
   and |colour|. `u u > g g u u` and `u~ u~ > g g u~ u~` are therefore in one
   P-dir. With MB a live parameter (`'MB'` ≠ `'ZERO'`), d and b are not.
2. **Matrix element.** Inside a group, `HelasMultiProcess.generate_matrix_elements`
   (`helas_objects.py:5817`, non-decay branch `:5918–5961`) merges two
   amplitudes into one matrix element only when their `IdentifyMETag`s are
   equal. The tag's external-leg link (`helas_objects.py:178–180`) contains
   `is_part`, `mass` (the parameter name) and `color`. So a particle and its
   antiparticle are **never one matrix element**, and a merged process is
   reordered to the first one's leg order (`reorder_process`, `:5952`).
3. **Colour.** Each matrix element writes its own colour basis, JAMPs and CF
   (`matrix<N>.f`), and its own `ICOLUP(…, iflow, numproc)` table
   (`export_v4.py:947–990`). Within a merged matrix element only the first
   process's table is written (`:967`, "Only one output, for the first
   subproc!"), since its processes share legs and basis by construction. An
   event's colour comes from the selected matrix element's own table
   (`template_files/addmothers.f:134`, `icolup(1,i,icol,numproc)`).
   **MadGraph never maps colour flows between two matrix elements.**

Measured with `output`:

| run | P-dir | matrix elements |
|---|---|---|
| `H-G0-qcd2` | `P1_qq_ggqq` | `matrix1` `u u > g g u u`; `matrix2` `u u~ > g g u u~`; `matrix3` `u~ u~ > g g u~ u~`, three separate bases of 12 flows |
| `H-G0-qcd3` | `P1_qq_qgq` | `matrix4` `u b > u g b` and `matrix7` `u~ b~ > u~ g b~`, separate. `b b > b g b` is combined into `matrix1` with `u u > u g u` (log: "Combined process b b > b g b … with process u u > u g u") |
| `H-G0-vbf2` (`sm`) | `P1_qq_hhqq` and `P1_bb_hhbb` | two P-dirs: `d d` with 8 graphs, `b b` with 192 |
| `H-G0-vbf3` (`sm-no_b_mass`) | `P1_qq_hhqq` | `matrix1` holds both `d d > h h d d` and `b b > h h b b`, 8 graphs ("Combined process b b > h h b b … with process d d > h h d d") |

So the backlog item's sentence "MadGraph groups such pairs and maps the colour
flow between them" is half right. MadGraph puts them in one P-dir as **separate**
matrix elements, each with its own basis, and maps nothing. Our flavour group,
one compiled evaluator whose members reuse its JAMPs, corresponds to MadGraph's
*matrix element*, not to its P-dir. Holding a conjugate pair in one flavour
group is therefore a step beyond MadGraph, not a port of it. It is legitimate
only because |M|² and every per-flow JAMP² agree under π. The Singlet card's
own generation-time restriction is unknown. MadGraph would put `b b` in its own
P-dir (MB live, as in `vbf2`) or merge it into the same matrix element (MB
restricted to zero, as in `vbf3`). Neither case involves mapping colour.

## Can each pair stay in one group? A criterion for the `colour-split-groups` review

**The QCD conjugate pairs: yes.** Replace the raw-order check at
`proton.rs:866` with this order:

1. `n_flows` equal;
2. π = `flow_permutation(rep, member)`, which must be `Ok`;
3. **`CF_rep[i·n + j] == CF_mem[π(i)·n + π(j)]` for all i, j**, as exact
   rationals.

The pair is refused, or split, only if step 2 or 3 fails. On both pairs step 3
holds with the π that step 2 returns, as measured above.

What the review must check:

- **π comes from the fingerprints, never from CF.** CF has 16 automorphisms on
  `u u > g g u u` and 8 on `u b > b g u`. A "find any P with P·CF·Pᵀ = CF′"
  search therefore returns a wrong π for 15 of 16 (7 of 8) candidates while
  passing every |M|² and Gram-matrix gate. The probe's first such P on the
  12-flow pair is `[0,2,4,6,11,10,9,8,1,3,5,7]`, not π. Step 3 is a
  *confirmation* of π and is blind to any error inside CF's automorphism group.
- **A test that fails before and passes after:** grouping `u u > g g u u` with
  `u~ u~ > g g u~ u~`, asserting the member's `flow_permutation ==
  [7,3,5,1,8,10,9,11,6,2,4,0]`. Assert also that the member's reindexed table
  equals `ColorFlowTags::conjugated` of the rep's table as line connectivity.
  That second oracle is independent of the fingerprints, and it holds only for
  full conjugates. The `u b` pair, `[0,2,3,1]`, is the small case.
- **Mutation:** restoring the raw `cf_matrix() !=` comparison must make that
  test fail. Replacing π with any other element of the CF-automorphism coset
  must fail the `conjugated` assertion but pass the CF check, and showing that
  is the blind-spot demonstration.

If the branch instead **splits** the pair into a new group, the result is
correct but redundant. MadGraph also splits them, as separate matrix elements.
The two groups have identical |M|² traces, which is the configuration that
`DegenerateGroups` refuses (`proton.rs:832–846`). A split made after that check
must not re-enter it, and nothing downstream may assume that two groups'
traces differ. The review should check both.

**The Singlet pair: not by a structural pairing.** The fingerprints genuinely
differ (8 against 192 diagrams), and per-flow JAMP equality (exactly 0.0 here)
holds only at this parameter point. Two correct options:

- (i) **Prune** diagrams whose couplings are exactly zero at the evaluated
  card before compiling. This is what MadGraph's restriction does at import.
  After it, the pair groups as under `no_b_mass`: `flow_permutation`
  `Ok([0,1])`, one group. This is outside item 1's scope.
- (ii) **Start a new group**, which is what `colour-split-groups` does and
  what MadGraph does when MB is live. The new group's |M|² equals the first
  group's to 0.0, so it is the degenerate-groups configuration above, in its
  purest form.

The criterion "same keys at the same tags, so pair them" is **not**
sufficient. It says nothing about which contributions a flow carries, and here
it would be numerically right only because ymb = 0.

### What these comparisons cannot see

- The CF checks are blind to any permutation inside CF's automorphism group (16
  and 8 elements) and to a uniform transpose. CF is symmetric, so a transpose
  is invisible by construction.
- Per-flow JAMP² and per-helicity |JAMP| are blind to a phase per flow and per
  helicity, which is harmless for selecting a flow by JAMP². They are also
  blind to two flows with equal JAMP² by symmetry. Neither occurs in these
  bases, since none has duplicate fingerprints, but in general they are not a
  pairing oracle on their own.
- The line-topology pairing applies only to full conjugates (every leg
  conjugated) and to same-rep pairs. The crossing class (`u c > u c` /
  `u c~ > u c~`) has no slot relation, so it rests on fingerprints alone.
- The Singlet case was reproduced on the SM, not the Singlet UFO. It shows the
  mechanism and reproduces the exact reason string. That the Singlet gridpacks
  fail for the same 184-diagram reason is an inference: the SM-like quark
  sector and the card's MB = ymb = 0 make it likely, but no Singlet run was
  made.
- Three examples cover 19 refused gridpacks. The other gridpacks of the 15 + 4
  were not enumerated pair by pair.

## Found

1. **The raw-order CF check refuses every conjugate pair with more than two
   flows.** Evidence: `proton.rs:866–875` runs before `flow_permutation`; with
   π applied, CF is equal on both QCD pairs. To close: compare CF under π
   (criterion above), with the fails-before test and the automorphism-coset
   mutation. It may supersede, for these pairs, the split that
   `colour-split-groups` makes.
2. **Zero-coupling diagrams survive a param card that zeroes a live
   parameter.** `b b > H H b b` compiles 192 diagrams, 184 of them identically
   zero at MB = ymb = 0. Effects: the `ColorFlowPairing` refusal on the four
   Singlet gridpacks, 24× the diagram work for that subprocess, and channels
   for zero diagrams. To close: prune couplings that are exactly zero at the
   evaluated card before compiling, as MadGraph's `RestrictModel` does at
   import, and gate it on the `no_b_mass` agreement shown here. MadGraph itself
   keeps them when the zero comes from the run-time card, so this would be a
   departure, and it needs a decision.
3. **Splitting into a new group re-creates the degenerate-groups
   configuration.** A group split off for colour has a trace equal to its
   origin's, which `DegenerateGroups` exists to refuse. To close: the
   `colour-split-groups` review checks that no consumer (channel merging, map
   identity, partition reporting) assumes distinct traces, or the branch
   documents why equal-trace groups are fine.
4. **The backlog item's MadGraph claim is wrong.** "MadGraph groups such pairs
   and maps the colour flow between them": MadGraph puts them in one P-dir as
   separate matrix elements and maps nothing (`helas_objects.py:178–180`,
   `export_v4.py:967`, `addmothers.f:134`). The item also cites
   `proton.rs:573/:865`, which are now `:579/:871`. To close: correct the item
   text at close-out.
5. **MadGraph's `DATA CF` stores off-diagonal entries doubled** (`48,32,…/3`
   against our `16, 16/3`). Any CF comparison against `matrix<N>.f` text must
   halve the off-diagonals. This is already true of the banked colour oracles,
   so it is noted for the review, not a defect.
