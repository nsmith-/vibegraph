---
type: Session Report
title: "J-D1 report: SMEFTsim dipoles on a quark line read against its arrow"
description: "A two-gamma dipole chain closed at a vector sink on a reversed quark line was evaluated in reverse order, which flips the commutator; fixed by permuting the chain's gamma indices, pinned by five MadGraph standalone rows and a rooting-invariance test, with mutation evidence. A separate CP-odd W-dipole defect is characterised and filed."
status: draft
tags: [evaluator, smeft, dipole, rooting, madgraph-parity, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-11}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-11}
---
Worktree `/home/user/wt/sprintj-d1`, branch `sprintj-dipoles`, based on `main`
at `9a1349c`. MadGraph is the pinned 3.7.1 submodule run through
`validation/madgraph/gen_standalone_jamps.py` from the `madgraph` pixi
environment, with SMEFTsim staged in the session scratchpad (`VG_MODELS_DIR`).
Diagnosis tables and per-coefficient cards (one Wilson coefficient on, derived
from `restrict_vg_ctdipole.dat`, i.e. `restrict_massless` with every SMEFT and
SMEFTcpv coefficient zeroed) live in the scratchpad; the rows that gate are
committed. The host has 4 cores shared with siblings, load 10-15.

Commits: `f7b2c96` (failing-first rows, listed in `KNOWN_DISAGREEMENT`), then
the fix commit (see the end).

## 1. Parameter provenance

Both sides read the param card the table carries (`EvaluatedModel::from_model_card`
on MadGraph's own `param_card.dat` for that restrict card); masses and widths
are therefore MadGraph's. Every non-dipole diagram of every probe matches its
MadGraph `AMP()` at modulus `1 ± 4e-15` with a common constant, which is what
makes the remaining residual attributable.

## 2. The offending term, per diagram and per helicity

Probe: a scratch integration test (not committed) that compiles each diagram on
its own through the production evaluator, evaluates it at every banked point and
helicity, pairs it with MadGraph's `c_k·AMP_k` by normalised overlap over all
entries, and fits one complex ratio per pair.

`b b~ > e+ e- NP<=1`, cbBRe = 2e-4 only, 3 points x 16 helicities, before the fix:

| diagram | vertices | MadGraph graph | overlap | ratio | residual / scale |
|---|---|---|---|---|---|
| 0 | `V_188#1` (b b a gauge), `V_203` | AMP(1) | 1.000000000000000 | +1.000000i | 3.5e-16 |
| 1 | `V_188#3` (b b a dipole, FFV2+FFV9), `V_203` | AMP(2) | 1.000000000000000 | **−1.000000i** | 5.4e-16 |
| 2 | `V_233#1` (b b Z gauge), `V_248` | AMP(3) | 1.000000000000000 | +1.000000i | 3.9e-15 |
| 3 | `V_234#4` (b b Z dipole), `V_248` | AMP(4) | 1.000000000000000 | **−1.000000i** | 3.1e-15 |

Full amplitude: `|G|−1 = −3.7e-6`, worst flow 1.67e-4. Every magnitude is exact
and every phase is right except a sign on the two dipole graphs. Same picture on
`t t~ > e+ e- NP<=1` under ctBRe alone (dipole graphs −i, gauge graphs +i,
worst flow 3.5e-3). The mirrors `e+ e- > b b~` and `e+ e- > t t~` agree on
every graph at +i (worst flow 2.1e-15, 1.6e-14).

## 3. Arithmetic check on the dumps

Negating only the dipole diagrams on the dumped values and recontracting, before
any production change:

| process | negated | G | worst flow |
|---|---|---|---|
| `b b~ > e+ e-` cbB | 1, 3 | +1.0000000i, `|G|−1 = −5.6e-16` | 1.6e-15 |
| `t t~ > e+ e-` ctB | 1, 3 | +1.0000000i, `|G|−1 = −2.3e-15` | 1.4e-14 |
| `b b~ > e+ e- g` cbB | 1, 3, 5, 7 | −1.0000000i, `|G|−1 = −3.7e-15` | 1.2e-15 |
| `t b~ > ve e+` ctWRe | 1 | −1.0000000i | 1.7e-13 |
| `t b~ > ve e+` cbWRe | 1 | −1.0000000i | 1.7e-14 |

## 4. Root cause

A fermion chain closed into a bilinear (the amplitude contraction, or a vector
current rooted at a boson leg, through `GammaVout`) is evaluated by applying its
operators to the spinor at each end, from that end inwards, following the bound
adjoint (`kernel::off_shell_fermion_current`, `resolve_bra_ket`). When the
chain's row slot is bound to a ket — an initial-state pair, or a line from an
initial to a final state, which this crate reads against the vertex's arrow —
the bra is at the column slot, so the kernels form `ψ̄_bra (O_n ⋯ O_1) ψ_ket`
for a chain written `O_1 ⋯ O_n` row to column: the reverse product. The existing
machinery compensates exactly what one gamma needs: the chirality conjugation of
a gamma-chained projector (`chiral_correction`) and one `Cγ^TC⁻¹ = −γ` sign
(runtime `reversed` flag, cancelled by the live parity, and the graph-level
`vertex_root_signs`). For a two-gamma chain the reverse product is a different
matrix: SMEFTsim's dipoles are commutators (`FFV9 = [γ^μ, p̸]`, `FFV2 = [γ^μ, p̸]γ⁵`,
`FFV10/FFV4 = [γ^μ, p̸]P_{R/L}`), and reversed they are their negatives, while
the gauge coupling on the same line is not. Hence the per-graph sign.

An off-shell fermion current through the same vertex is re-rooted onto the slot
whose adjoint matches (`correct_spin_index_for_flow`) and applies the operators
in their written order, so it is right. The defect was therefore also a rooting
dependence: the same diagram gave opposite signs when rooted at the dipole
vertex and across its internal b line (section 6, mutation of the rooting test).

It is not the momentum: `P` reads stored momenta (out of the consuming vertex,
as ALOHA) consistently for internal and external vectors. The control that
separates the two: in `b b~ > e+ e- g`, the O_bG dipole on the same reversed b
line agrees with MadGraph before the fix (worst flow 1.9e-15), the O_bB dipole
does not (1.45e-4). Canonical rooting (fewest external legs) makes every O_bG
vertex there an off-shell b current, and every O_bB vertex a vector sink.

**Correction to the brief.** J-E1's hypothesis (the dipole as the amplitude root
with the vector an internal input; a momentum sign) names a correlate. The
condition is "the chain is closed at a vector sink — amplitude root *or* vector
current — on a line read against its arrow", independent of where the vector
comes from; the momentum convention is not involved.

### Why the J-E1 pattern follows

On `b b~ > t t~ NP<=1`: the b pair annihilates (reversed); the t pair is crossed
(both final), whose row slot is never bound to a ket; the t-channel W joins
initial b to final t (reversed, mixed line), and the W vertices have no internal
fermion leg, so they are always sinks.

- ctGRe, ctBRe: on the crossed top line only → never reversed → agree.
- cbGRe, cbWRe (neutral part), cbBRe: on the annihilating b pair, a sink → fail.
- ctWRe, cbWRe (W part): FFV10/FFV4 on the mixed lines, a sink → fail.
- cbBRe is small (2.5e-5 in J-E1's table) only because its coupling is.
- `e+ e- > b b~` and the banked `ee_to_ttx_dipole`: dipole on a crossed line → agree.

Measured before / after the fix, per-coefficient cards (scratch tables, worst flow):

| `b b~ > t t~ NP<=1` | before | after |
|---|---|---|
| ctGRe | 2.1e-15 | 2.1e-15 |
| ctWRe | 1.7e-3 | 2.1e-15 |
| ctBRe | 2.1e-15 | 2.1e-15 |
| cbGRe | 1.9e-3 | 2.1e-15 |
| cbWRe | 7.1e-3 | 2.1e-15 |
| cbBRe | 2.3e-5 | 2.1e-15 |
| ctGIm, cbGIm alone | — | 2.1e-15 |

## 5. The fix

`root_lorentz::reorder_reversed_sink_chains`, called in `build_at_leg` on the
tree path: for each fermion pair of the term that does not hold the rooted
output, whose row slot is bound to a ket, and whose chain carries two or more
gammas, permute the gammas' Lorentz indices so the gamma at position `k` from
the row carries the index of the one at `n+1−k`. The reverse-order reading then
equals the structure as written, term by term (exact, not only for antisymmetric
structures). The row operator stays a gamma, so the reversed-bilinear parity and
the runtime reversal it cancels are unchanged; a projector or `γ⁵` beside an even
number of gammas commutes back to where it stood. A chain with a literal `Sigma`
beside another carrier is refused (no index permutation reverses it). One-gamma
chains — every Standard-Model structure — are untouched.

## 6. Oracles, commands and outputs

Committed rows (`validation/madgraph/gen_standalone_jamps.py`, tables under
`validation/madgraph/standalone/`, cards `restrict_vg_cbB`, `restrict_vg_ctB`,
`restrict_vg_tbdipole`):

```
cargo test -p vibegraph-lib --test standalone_jamps -- dipole cbb ctb --nocapture
```

| row | before (f7b2c96, listed) | after |
|---|---|---|
| `bbx_to_epem_cbb` | flow 1.563e-4, graph 1.775e-4 | flow 1.708e-15, graph 5.432e-15 |
| `ttx_to_epem_ctb` | flow 3.211e-3, graph 4.136e-3 | flow 6.337e-15, graph 9.404e-15 |
| `bbx_to_epemg_cbb` | flow 2.454e-4, graph 1.794e-4 | flow 1.628e-15, graph 2.385e-15 |
| `bbx_to_ttx_tbdipole` | flow 1.398e-2 | flow 2.315e-15 |
| `tbx_to_vlep_tbdipole` | flow 4.701e-3 | flow 3.526e-14 |

Rooting invariance (oracle-free):

```
cargo test -p vibegraph-lib --features extended-validation --lib chain_is_rooting_invariant
test helas::eval::rooting_soundness::reversed_dipole_chain_is_rooting_invariant ... ok
test helas::eval::rooting_soundness::momentum_slashed_chain_is_rooting_invariant ... ok
```

Mutation (the fix reverted with `git apply -R`, tests kept): all five rows FAILED
(`test result: FAILED. 0 passed; 5 failed`), and

```
test helas::eval::rooting_soundness::reversed_dipole_chain_is_rooting_invariant ... FAILED
diagram 1 ["V_278#1", "V_188#3", "V_203#1"] rooted at vertex 0, helicities [1, -1, 1, -1, -1]:
Complex { re: -1.98452e-10, im: -3.38885e-10 } against the production rooting's
Complex { re: 1.98452e-10, im: 3.38885e-10 }
```

The existing `momentum_slashed_chain_is_rooting_invariant` passes under the
mutation: its dipole is on a crossed line and a two-vertex diagram never roots it
at a fermion leg.

Workspace gates and the banked layer: see the fix commit's section below.

## 7. What each new test cannot detect

- The five standalone rows: a global phase (fitted G); the split of a flow into
  diagrams where no `AMP()` is banked (`bbx_to_ttx_tbdipole`,
  `tbx_to_vlep_tbdipole`); a defect at helicities or kinematics the 3 banked
  points do not visit; anything only reachable with a symmetric (non-commutator)
  two-gamma structure, since every dipole structure here is antisymmetric and the
  sign-only and permutation repairs agree on them. `tbx_to_vlep_tbdipole` banks
  no `AMP()`: with a massless b the O_tW and O_bW graphs are proportional to the
  gauge graph at every helicity (overlap 1 at real ratios 1.25, −1.067, 0.8), so a
  pairing is ill-posed there.
- `reversed_dipole_chain_is_rooting_invariant`: anything rooting-invariant —
  a sign common to every rooting of a diagram, i.e. the MadGraph comparison's job.
  It covers only O_bB on `b b~ > e+ e- g` at one point.
- No row covers a three-gamma chain on a reversed line, nor the two-gluon
  contacts (FFVV1-4) of the quark-gluon dipoles on a reversed line; both now take
  the permutation and are unmeasured.

## 8. Second defect: the W dipoles' CP-odd parts

Found while widening `vg_tbdipole` to both CP parities; fixed in this branch at the
manager's direction, with its own failing-first commit.

**Size.** Each coefficient alone, the dipole graph is a pure sign flip at every
banked helicity (overlap 1.000000000000000, ratio −G against the gauge graph's G,
residual ≤ 1.8e-14). Worst flow: ctWIm = 0.0025 on `t b~ > ve e+`, `ve e+ > t b~`,
`b b~ > t t~`: 2.2e-3, 2.4e-3, 2.2e-3; cbWIm = 0.0075: 7.2e-3, 7.8e-3, 6.7e-3.
ctWRe and cbWRe agree on the same processes once the first fix is in
(1.7e-13, 1.7e-14). The crossed `ve e+ > t b~` is untouched by the first fix, so
the defect was on `main`.

**Is the UFO Hermitian?** For a vertex with coupling `G` on structure `L`, the
Hermitian partner (antiparticles in the same slots) carries `γ⁰L†γ⁰` with
`(−1)^{n_P+1}·conj(G)` — the `i` of the Feynman rule and the `−i` of each
derivative. On the SM W vertex, FFV3 = `γ^μP_L` maps to itself and GC_202 sits on
both members, `G' = −conj(G)` at `n_P = 0`. On the dipole chains,
`γ⁰[γ^μ, p̸]P_R†γ⁰ = −[γ^μ, p̸]P_L`, so FFV10 maps to −FFV4 and hermiticity
requires `partner.FFV4 = −conj(G.FFV10)`, `partner.FFV10 = −conj(G.FFV4)`. Both
SMEFTsim pairs satisfy it exactly: `V_324` [t~ b W+] FFV10 = GC_368 = i·ctWRe·v,
FFV4 = GC_343 = i·cbWRe·v against `V_285` [b~ t W-] FFV4 = GC_368,
FFV10 = GC_343; `V_323` FFV10 = GC_365 = −ctWIm·v, FFV4 = GC_342 = +cbWIm·v
against `V_284` FFV4 = GC_366 = +ctWIm·v, FFV10 = GC_341 = −cbWIm·v. The other
sign convention fails both pairs. So the choice between the two members cannot
change physics if each is evaluated as its own Feynman rule, and the defect is
this crate's.

**Root cause.** Enumeration binds a vertex by its particles read all-incoming,
MadGraph by the UFO's listing (all-outgoing): the two name Hermitian-conjugate
members (the matrix.f calls confirm it: `t b~ > ve e+` evaluates
`FFV10_3(…, GC_365)` of `V_323`, this crate binds `V_284`). The kernels evaluate
the bound structure on the partner's spinor slots, with momenta read incoming,
so a bound term stands for the partner's term on `γ⁰L†γ⁰` with each momentum
reversed. `UFOModel::bound_coupling` took the partner's coupling only for pure
vector currents (`conjugate_vector_current`, the CKM fix), where that conjugate
is the same structure and key; every other charged vertex kept its own coupling.
For the dipoles the conjugate is the other chirality (`[γ^μ, p̸]P_R` →
`[γ^μ, p̸]P_L`: the commutator's −1 against the momentum's), whose partner
coupling is `−conj` of the bound one: equal for the imaginary CP-even couplings,
opposite for the real CP-odd ones. MadGraph's `−GC_365` in `ve e+ > t b~` is its
restriction merging GC_366 = −GC_365, not a flow flip.

**Fix.** `UFOModel::conjugate_couplings` generalises the exchange term by term.
`chain_normal_form` writes a two-fermion structure as its chain of `Gamma`/`Sigma`
factors from the row slot, each Lorentz index named by what it meets, with the
chiral factors commuted to the column end and expanded on `1` and `γ⁵`; its
conjugate form reads the chain from the column, maps `P_L ↔ P_R`, `γ⁵ → −γ⁵`,
exchanges the fermion slots of momenta and multiplies by `(−1)^{n_P}`. Each bound
term takes the coupling of the unique partner term with that normal form, same
colour index and coupling orders. A vector current maps to its own key, so the CKM
behaviour is unchanged (`ckm_vertices_bind_the_lagrangian_orientation` passes); a
self-conjugate vertex, a structure outside the normal form, or no unique partner
keeps its own coupling.

**Oracles.**

```
cargo test -p vibegraph-lib --lib ufo::tests
test result: ok. 15 passed; 0 failed
```

`w_dipole_terms_bind_their_hermitian_partners` checks, on every split occurrence
of `V_284`, `V_323`, `V_285`, `V_324` under `vg_tbdipole`, that each term binds the
partner's coupling on the mapped structure (FFV3 → FFV3, FFV10 ↔ FFV4) and that
the UFO values satisfy `bound = −conj(partner)` to 1e-15 (the `n_P = 0` and
`n_P = 1` rule on the key map); `a_self_conjugate_dipole_vertex_has_no_partner`
checks the b photon and Z dipole vertices keep their own.

```
cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture
test result: ok. 61 passed; 0 failed
```

| row | before (d9e019f, listed) | after |
|---|---|---|
| `vlep_to_tbx_ctwim` | flow 2.408e-3, graph 2.595e-3 | flow 1.526e-14, graph 1.526e-14 |
| `vlep_to_tbx_cbwim` | flow 8.912e-3, graph 8.913e-3 | flow 4.204e-15, graph 4.204e-15 |
| `bbx_to_ttx_tbdipole` (all dipoles, both parities) | flow 1.142e-2 | flow 2.314e-15 |
| `tbx_to_vlep_tbdipole` (same card) | flow 4.404e-3 | flow 3.526e-14 |

Mutation (the `ufo/mod.rs` change reverted with `git apply -R`): those four rows
`FAILED` at exactly the "before" column (`test result: FAILED. 0 passed;
4 failed`); the other 57 standalone rows are unchanged by the fix (every one of
them passed before and after).

**What it cannot detect.** A charged vertex whose structure falls outside the
chain normal form (`Metric`, `Epsilon`, four fermions) keeps its own coupling,
silently, as before; no row reaches a charged vertex with a complex coupling on a
scalar or tensor structure. The unit test's hermiticity check is on the UFO, not
on the evaluator.

## Found

1. **The rooting-soundness sweep never roots a reversed multi-gamma chain both
   ways.** The defect fixed here was a rooting dependence that
   `all_rootings_preserve_amplitude` could not see (no MG-validated process puts a
   two-gamma chain on a reversed line). A sweep over SMEFTsim dipole and FFVV
   processes on initial-state lines would close the class.
2. **No gate reaches the quark-gluon dipoles' two-gluon contacts (FFVV1-4) on a
   reversed line** (e.g. `t t~ > g g NP<=1` under ctGRe/ctGIm). The fix changes
   their evaluation there; it is unmeasured.
3. **The backlog item `conjugate-vector-current-fallback-silent` names a function
   that no longer exists.** `conjugate_vector_current` is now
   `UFOModel::conjugate_couplings`, which matches more structures but keeps the
   same silent fallback: with no unique partner, `bound_coupling` binds the
   vertex's own coupling without a word. The item's premise stands; its text needs
   the new name.
4. **Hermitian-partner coverage beyond FFV chains is unmeasured.** A charged
   vertex with a complex coupling on a structure outside the chain normal form
   (a four-fermion contact with `Metric`, an `Epsilon` structure, a charged
   scalar with a CP phase) would still bind its own coupling; no model in reach
   has one with a non-zero imaginary part on the cards in use.

The knowledge bundle's open item "the fermion dipole" in
`amplitudes/convention-sign-inventory` is resolved there in this branch (at the
manager's direction): the dipole needs no `(−1)^{n_P}` in `convention_sign`; its
two failures were the chain order on reversed lines and the Hermitian-partner
coupling on charged vertices.
