---
type: Session Report
title: "V3 report: the u b~ W vertex carried the conjugate interaction's coupling"
description: "The 42.3° is 2·arg(V_ub) on top of the expected +i: diagram enumeration binds a vertex by its particles read all-incoming, the UFO lists them all-outgoing, so the bound [b~, u, W-] carried GC_43 ∝ CKM1x3 where MadGraph's [u~, b, W+] carries GC_102 ∝ conj(CKM1x3). That evaluates the model at η → −η, which |M|² sees in u c~ > b b~ d d~ (3.7e-2). A charged vector current now carries its Hermitian conjugate's coupling; ubx_to_epve_ckm_no_b_mass gates with G = +1i."
status: draft
tags: [amplitudes, ckm, coupling, crossing, convention, sm, sprint-d, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-22}]
sources:
  - {id: mg-v95, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/vertices.py#L576-L580", title: "sm vertices.py V_95: [u~, b, W+], FFV2, GC_102"}
  - {id: mg-v125, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/vertices.py#L756-L760", title: "sm vertices.py V_125: [b~, u, W-], FFV2, GC_43"}
  - {id: mg-gc43, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/couplings.py#L180-L182", title: "sm couplings.py GC_43 = CKM1x3 ee i/(sw sqrt2)"}
  - {id: mg-gc102, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/couplings.py#L416-L418", title: "sm couplings.py GC_102 = ee i conj(CKM1x3)/(sw sqrt2)"}
  - {id: mg-ckm, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/parameters.py#L229-L281", title: "sm parameters.py CKM1x1..CKM3x3, leading-order Wolfenstein"}
  - {id: mg-ffv2, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/sm/lorentz.py#L47-L49", title: "sm lorentz.py FFV2 = Gamma(3,2,-1)*ProjM(-1,1)"}
  - {id: mg-outgoing, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L553-L554", title: "diagram_generation.py: the resulting vertices have all particles outgoing"}
  - {id: mg-flip, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L654-L660", title: "diagram_generation.py: flip part-antipart for incoming particles so they are all outgoing"}
  - {id: aloha-spinor, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/aloha/create_aloha.py#L337-L346", title: "create_aloha.py: odd/even fermion slot parity"}
---
Brief: diagnose the `ubx_to_epve_ckm_no_b_mass` amplitude constant
`G = 0.6733031674208174 − 0.7393665158371072 i` on PR #62. Built on `866c884`.
Classification: **(a), a real convention bug.** The fix landed, the row's
`amplitudes` cell is `gate`.

## Provenance

The coupling table banks MadGraph's Python `coupling_dict` and the Fortran
couplings the process uses. The Fortran column is `GC_102` and `GC_108`:

| coupling | MadGraph value | expression |
|---|---|---|
| `GC_102` | `−1.4554670805025875e-3 + 5.634066118074532e-4 i` | `ee·i·conj(CKM1x3)/(sw√2)` |
| `GC_43` | `+1.4554670805025875e-3 + 5.634066118074532e-4 i` | `CKM1x3·ee·i/(sw√2)` |
| `GC_108` | `0.4619054347431334 i` | `ee·i·conj(CKM3x3)/(sw√2)`, `CKM3x3 = 1`: the lepton vertex's value |

`CKM1x3 = AWS·lamWS³·(rhoWS − i·etaWS)` at the card's `0.2253, 0.808, 0.132,
0.341` is `1.21974e-3 − 3.15101e-3 i`, `|V_ub| = 3.37885e-3`,
`arg V_ub = −68.8387°`. `GC_102/GC_108 = conj(V_ub)` to the last digit, and
`GC_43 = −conj(GC_102)`: the two are Hermitian partners.

The coupling gate (`coupling_oracle`) compares couplings by name and passed
before and after: this crate's evaluated `CKM1x3`, `GC_43` and `GC_102` are
MadGraph's. Parameter provenance is clean; the disagreement is which of the two
the vertex carries.

## The arithmetic of 42.3°

`G = vg/mg` (`fit_constant`). With `vg ∝ GC_43` and `mg ∝ GC_102`, and the `+i`
every other row carries,

`G = i · GC_43/GC_102 = i · V_ub/conj(V_ub) = i · e^{2i·arg V_ub}`.

`i·V_ub/conj(V_ub) = 0.6733031674208145 − 0.7393665158371039 i` against the
fitted `0.6733031674208174 − 0.7393665158371072 i` (difference 3e-15).
`arg G = −47.677°`, so `G = +i·e^{−137.677° i} = −i·e^{+42.323° i}`, and
`2·arg V_ub = −137.677°`. The 42.3° "off −i" is `180° + 2·arg V_ub`: it is a
conjugated `V_ub` read against `+i`, not a rotation against `−i`. The other
candidates in the brief do not fit: a different parametrisation or a phase on the
W-fermion vertex would not leave `GC_102/GC_43` as the exact factor, and
`bbx_to_tapta` (no W) is the `+i` control.

## MadGraph's side, at b7687064

- `models/sm/vertices.py` L576–580, `V_95 = [u~, b, W+]`, `FFV2`, `GC_102`;
  L756–760, `V_125 = [b~, u, W-]`, `FFV2`, `GC_43`. Every W–quark pair is the
  same: `[u~_i, d_j, W+]` carries `conj(CKM_ij)`, `[d~_j, u_i, W-]` carries
  `CKM_ij`.
- `couplings.py` L180–182 (`GC_43`), L416–418 (`GC_102`); `parameters.py`
  L229–281, the leading-order Wolfenstein matrix, complex only in `CKM1x3` and
  `CKM3x1`.
- `madgraph/core/diagram_generation.py` L553–554: "the resulting vertices have
  all particles outgoing"; L654–660: incoming legs are flipped particle ↔
  antiparticle "so they are all outgoing". MadGraph reads an interaction's
  particle list all-outgoing. For incoming `u`, `b~` it looks up `{u~, b, W+}`:
  `V_95`, `GC_102`.
- `lorentz.py` L47–49, `FFV2 = Gamma(3,2,-1)*ProjM(-1,1)`: the row index is leg
  2, the column leg 1, so the operator is `ψ̄_{leg 2} γ^μ P_L ψ_{leg 1}`; ALOHA
  keeps that parity (`create_aloha.py` L337–346, odd fermion slot flowing out of
  the column, even into the row). Read all-outgoing, `V_95` is
  `b̄ γ^μ P_L u W⁻` with `conj(V_ub)`: the Hermitian-conjugate term of the PDG
  `(g/√2) ū_i γ^μ P_L V_ij d_j W⁺_μ`, whose `V_ub = Aλ³(ρ − iη)` is exactly
  `CKM1x3`. So the UFO, read as MadGraph reads it, is the PDG Lagrangian.
- The restriction card is not MadGraph's (CMS genproductions); it is vendored at
  `vibegraph-lib/src/ufo/sm_assets/restrict_ckm_no_b_mass.dat`, and its
  Wolfenstein block is the four values above.

## This crate's side

`Diagram::from_view` binds each vertex to the feyngraph interaction, whose
particles are read all-incoming (`diagrams/diagram.rs` module doc). For incoming
`u`, `b~` that is `{u, b~, W-}`, i.e. `V_125`, `GC_43`. The Lorentz side of the
swap was already handled: `root_lorentz.rs`'s crossing and reversal corrections
evaluate the bound structure on the conjugate interaction's slots, which is why
moduli, per-diagram values and `|M|²` all agreed. The coupling was the one thing
taken from the bound interaction rather than its conjugate. Read as a Lagrangian,
that is `ū V* d W⁺ + h.c.`: the model at `η → −η`.

## Classification and proof

**(a).** Conjugating `V` is the CP conjugate of the weak phase, and `|M|²` sees it
whenever two diagram classes with different CKM pairings interfere with a
kinematic phase between them.

Probe (scratch integration test, not committed): `|M|²` at five RAMBO points,
√s = 500, `sm-ckm_no_b_mass`, at the card's `η = +0.341` and at `η = −0.341`:

| process | diagrams | worst rel. `|M|²(+η) − |M|²(−η)|` |
|---|---|---|
| `u b~ > e+ ve` | 1 | 2.8e-16 |
| `d b~ > e- ve~ mu+ vm` | 3 | 4.8e-15 |
| `d s~ > e- ve~ mu+ vm` | 3 | 3.0e-13 |
| `u c~ > e+ ve e- ve~` | 3 | 7.3e-12 |
| `u c~ > b b~ d d~ QCD=0` | 32 | **5.3e-4** |
| `u c~ > b b~ d d~` | 8 | **3.7e-2** |

The first four cannot see it: one CKM product, or a flavour sum whose terms share
one Lorentz structure (a chiral `P_L … P_L` line kills the internal mass, so the
top term is the light one times a real factor). `u c~ > b b~ d d~` has two
pairings, `(u b̄)(c̄ b)` `∝ conj(V_ub)V_cb` and `(u d̄)(c̄ d)` `∝ conj(V_ud)V_cd`,
whose quartet has `Im ∝ A²λ⁶η`, with different topologies. Since the old binding
evaluated the model at `−η`, its `|M|²` for this process was MadGraph's at
`−η`, off by up to 3.7e-2.

After the fix, the same probe gives `|M|²_new(+η) = |M|²_old(−η)` and
`|M|²_new(−η) = |M|²_old(+η)` **bit for bit (0.0 relative) on every point of all
six processes**, including the 32- and 8-diagram ones, whose W–quark vertices sit
in annihilation, t-channel and internal positions. The fix is exactly the CKM
conjugation, in every orientation.

What this does not prove: no MadGraph reference exists for a CP-sensitive
process. That `new = MadGraph` beyond `u b~ > e+ ve` rests on the per-vertex
argument (both codes make one global crossing choice) plus that one row (see
Found).

## The fix

`UFOModel::bound_coupling(vertex, key)` (`ufo/mod.rs`) is the coupling a diagram
vertex carries. For a charged fermion–fermion–vector vertex whose every Lorentz
structure is a vector current (one `γ^μ`, with only `P_L`, `P_R`, `γ⁵` or the
identity beside it) it is the coupling of the Hermitian-conjugate interaction
(`conjugate_vector_current`). That interaction is the unique vertex with the
antiparticles, the same Lorentz structures, colour, coupling keys and per-key
orders. Such a structure is its own conjugate, so the term correspondence is the
identity. Every other vertex keeps its own coupling. `VertexInfo::from_ufo`
(`helas/eval/diagram_eval.rs`) takes its coupling ids from it, and so does the
reweight support in `reweight/poly.rs`, which then reads the coupling that is
evaluated.

Scope is deliberate. For a chiral scalar or a tensor (dipole) structure, the
conjugate's term with the same name is the opposite chirality's (the MadGraph SM
`[c~, b, G+]` / `[b~, c, G-]` pair lists `FFS1, FFS3` on both sides, crosswise).
A name-keyed exchange would be wrong there, so those keep their own coupling (see
Found).

## Test and mutation evidence

`ufo::tests::ckm_vertices_bind_the_lagrangian_orientation`: for all 18 W–quark
vertices of `sm-ckm_no_b_mass`, the bound coupling equals
`i·ee/(sw√2)·V_ij` for all-incoming `{u~_i, d_j, W+}` and its conjugate for
`{d~_j, u_i, W-}`. `V` is built from the card's Wolfenstein values, and the test
asserts that 4 of the 18 carry a phase.

Mutation: `bound_coupling` returning the bound vertex's own coupling.

```
thread 'ufo::tests::ckm_vertices_bind_the_lagrangian_orientation' panicked at vibegraph-lib/src/ufo/mod.rs:1046:13:
V_91 ["t~", "d", "W+"] carries -0.0014554670805025905+0.0037048252958247754i where the Lagrangian gives 0.0014554670805025907+0.0037048252958247754i (relative deviation 7.313e-1)
[ubx_to_epve_ckm_no_b_mass] the vibegraph↔MadGraph amplitude constant is not ±i: G = Complex { re: 0.6733031674208174, im: -0.7393665158371072 }. ...
test result: FAILED. 50 passed; 1 failed
```

With the fix:

```
[ubx_to_epve_ckm_no_b_mass] 'u b~ > e+ ve' NGRAPHS=1 NCOLOR=1: |M|² max_rel grid 8.13e-14 / event 9.83e-15, per-diagram 3.31e-16, per-flow 3.31e-16, JAMP2 8.61e-15 (G = +1i, |G|-1 = 4.4e-15, 15 helicity combinations pruned); 1 configurations: amplitude 3.31e-16, |k|-1 4.4e-15, |Im(k/G)| 0.0e0, AMP2 8.61e-15, pruning moves AMP2 by 0.00e0
test ubx_to_epve_ckm_no_b_mass ... ok
test result: ok. 51 passed; 0 failed
```

Bookkeeping: the manifest cell is `gate` with the measured note, and
`CONFIG_AMP_SIGNS` banks `("ubx_to_epve_ckm_no_b_mass", "-")`, as the gate asks
of every per-diagram row. `MG_VALIDATED_PROCESSES` is not extended: its sweeps
compile at the default restriction, where `V_ub = 0` prunes the vertex.

## Found

1. **Every other banked amplitude row is blind to coupling conjugation.**
   Across `validation/madgraph/couplings/*.json`, `ubx_to_epve_ckm_no_b_mass` is
   the only table whose Fortran couplings include a genuinely complex value. The
   others are `i·real` or real, and those are invariant under the exchange. To
   close: bank a MadGraph row for a CP-sensitive process (`u c~ > b b~ d d~
   QCD=0` at `sm-ckm_no_b_mass`, 32 diagrams, 5e-4 effect), which pins the
   orientation in t-channel and internal positions where `u b~ > e+ ve` has only
   annihilation.
2. **Charged non-vector-current vertices still carry the bound interaction's
   coupling**: chiral scalars (`FFS`, a charged Higgs or Goldstone) and tensor
   or derivative structures (SMEFTsim's W–t–b dipoles). The same all-incoming
   reading conjugates their Lagrangian coefficients, and the conjugate's
   same-name term is the wrong chirality, so the vector-current exchange does not
   carry over. With a complex coefficient (a CP-odd Wilson coefficient, a complex
   charged-Higgs Yukawa) `|M|²` would be the CP conjugate. To close: map each
   Lorentz structure to its Hermitian image and take the conjugate interaction's
   coupling on the image term, with a MadGraph row at a complex coefficient as the
   oracle.
3. **Self-conjugate vertices with a complex chiral coefficient are unpinned.**
   The all-incoming reading binds the same interaction with its fermion slots
   swapped, and the chiral correction puts `P_χ̄` on the swapped slots. Whether
   the coefficients follow (`c_L ↔ c_R` against `−conj`) is untested, because no
   banked row has a complex coefficient on such a vertex (a CP-odd `h t t~` or
   `t t~ γ` dipole). To close: one MadGraph row with an imaginary dipole or
   CP-odd Yukawa.
4. **A vertex with no unique conjugate silently keeps its own coupling.**
   SMEFTsim splits each charged current into several vertices with the same
   particles (`[u~, d, W+]` is `V_331`–`V_334`), which is why
   `conjugate_vector_current` also matches orders and structures. A Python
   replica of the predicate run on the UFO sources finds a unique conjugate for
   all 36 SMEFTsim and 24 SM charged vector-current vertices. That replica
   compares raw colour strings, not this crate's parsed ones. A model where the
   match fails would fall back to the old convention without saying so. To
   close: make the fallback loud, as an error or a census test over the vendored
   models.
5. **`MG_VALIDATED_PROCESSES`' doc says it is "every concrete subprocess that
   gate covers"**; it holds 19 against the gate's 51 rows, and it cannot hold rows
   at other restrictions, since it compiles on the default model. To close:
   reword it, or key it by restriction.
6. **`tests/sm_ckm_no_b_mass.rs`'s module doc** says no coupling table is banked
   for the CKM sector against MadGraph. `couplings/ubx_to_epve_ckm_no_b_mass.json`
   now banks the whole `coupling_dict` at this card. To close: update the
   "What composition cannot see" list.
