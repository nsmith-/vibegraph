---
type: Overview
title: The vendored SMEFTsim topU3l MwScheme UFO
description: "What SMEFTsim_topU3l_MwScheme_UFO contains and what its restrict cards turn on; vendored byte-for-byte (v3.0.2, db7d4a80) with LICENSE, SHA256SUMS and provenance, not a 101 MB submodule."
status: draft
tags: [smeftsim, ufo, non-sm-ufo, vendoring, census]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n35-census, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L111-L143", title: "Note 35 §1.2, SMEFTsim static census"}
  - {id: n35-conv, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L175-L214", title: "Note 35 §1.4, conventions read from the pinned MadGraph source"}
  - {id: n35-f1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L475-L548", title: "Note 35 §3 F1, four-fermion vertices (pairing census corrected)"}
  - {id: n35-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/35-ufo-lorentz-sprint-plan.md#L1191-L1218", title: "Note 35 §7, decisions (user, 2026-09-05)"}
  - {id: ufo-readme, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/ufo/README.md", title: "validation/ufo/README.md, provenance of the vendored and authored UFOs"}
  - {id: smeftsim-upstream, resource: "https://github.com/SMEFTsim/SMEFTsim/tree/db7d4a80bdcff424eee27dde71f1eb09ac894039/UFO_models/SMEFTsim_topU3l_MwScheme_UFO", title: "SMEFTsim upstream at v3.0.2"}
  - {id: brivio, resource: "https://arxiv.org/abs/2012.11343", title: "I. Brivio, SMEFTsim 3.0 — a practical guide, JHEP 04 (2021) 073"}
---

`validation/ufo/SMEFTsim_topU3l_MwScheme_UFO/` is the one published non-SM model the
validation layer reads: SMEFTsim 3.0 [^brivio] with the `topU3l` flavour assumption and
`{m_W, m_Z, G_F}` electroweak inputs. It is the reference case for general UFO Lorentz
structures: four-fermion contacts, γ-chains with `Gamma5`, `Epsilon`, five- and
six-point vertices and custom propagators.

## Vendored, not a submodule

The directory is a **byte-for-byte copy** of upstream's
`UFO_models/SMEFTsim_topU3l_MwScheme_UFO` at tag `v3.0.2`, commit
`db7d4a80bdcff424eee27dde71f1eb09ac894039` (2021-01-24) [^smeftsim-upstream], with the
upstream MIT `LICENSE` and a `SHA256SUMS` manifest. Provenance (upstream path, tag,
copy date, reference) is recorded in `validation/ufo/README.md` [^ufo-readme];
`version.info` inside the directory says `tag: v3.0.2`.

The decision (user, 2026-09-05) replaced a depth-1 submodule: the submodule checked out
about 101 MB of FeynRules sources and notebooks for a UFO under a megabyte, and CI's
`banked` job checks submodules out on every run. No workflow needs a submodule step for
it. [^n35-decisions]

- **Drift is detectable.** `vendored_copy_matches_its_manifest`
  (`vibegraph-lib/tests/smeftsim.rs`) checks the manifest both ways: every listed file
  has its digest and every file in the directory is listed. The test file requires
  `extended-validation`, so the default hermetic `cargo test` does not run it;
  `(cd validation/ufo/SMEFTsim_topU3l_MwScheme_UFO && sha256sum -c SHA256SUMS)` is the
  manual check.
- **Nothing authored goes inside.** Per-class restrict cards written for the validation
  rows live under `validation/madgraph/cards/smeft/restrict_vg_*.dat`, because a file in
  the vendored directory would break its manifest.
- **Licence.** Not compiled into any binary, so it is outside `THIRD-PARTY-NOTICES`;
  the licence file travels with the copy.
- The other SMEFTsim variants (`alphaScheme`, `U35`, `MFV`, `general`, `top`) are not
  vendored; fetch them from upstream at the same tag if a gate needs one.

## Census

Counted from the files [^n35-census], with the four-fermion pairing counts as corrected by
MadGraph's own fermion-flow walk [^n35-f1]:

| Item | Count |
|---|---|
| particles | 21 (spins 1/2/3 only, no ghosts; colours 1/3/8) |
| Lorentz structures | 260 |
| vertices | 904 (1985 interactions once split by coupling-order tuple; see [coupling orders](coupling-orders.md)) |
| couplings | 1278 |
| parameters | 315 |

**Operators in the Lorentz structures:**

| Operator | Uses | Note |
|---|---|---|
| `P` | 3189 | 123 of them as `P(-1,a)**2` |
| `Metric` | 2885 | |
| `Epsilon` | 846 | the CP-odd (`…til`) operators |
| `Gamma` | 137 | including γ-chains sharing a summed spinor index |
| `ProjM` / `ProjP` | 40 / 40 | |
| `Gamma5` | 13 | inside the CP-odd dipole chains |
| `Identity` | 3 | |
| `Sigma`, `C` | 0 | FeynRules expands σ^μν into γγ chains before writing a UFO |

Dipoles appear as momentum-slashed γ-chains,
`P(-1,3)*Gamma(-1,2,-3)*Gamma(3,-3,-2)*ProjM(-2,1)`. How each operator is evaluated
is in [gamma chains, Gamma5 and Epsilon](../amplitudes/gamma-chains-gamma5-and-epsilon.md);
the literal `Sigma` that no published model emits is exercised by the
[toy UFO models](../validation/toy-ufo-models.md) instead.

**Leg-spin patterns:** 62 six-vector and 57 five-vector structures (the `cG`/`cW`
field-strength cubes), 21 four-fermion, 20 `VVVVS`, 15 `VVVV`, 10 `FFV`, 7 `FFVS`,
5 `FFVVS`, 4 `FFVV`, and the rest three-point. feyngraph accepts the five- and six-leg
vertices, and enumeration stays in milliseconds.

**Four-fermion structures** come in three shapes:

- scalar ⊗ scalar, `ProjM(2,3)*ProjM(4,1)`;
- vector ⊗ vector, `Gamma(-1,2,-2)*Gamma(-1,4,-3)*Proj*Proj`, tree-shaped;
- tensor ⊗ tensor, `Gamma(-2,-4,-3)*Gamma(-2,2,-6)*Gamma(-1,-6,-5)*Gamma(-1,4,-4)*…`,
  whose index graph is a 4-cycle that no rooting of a tree can evaluate.

Of the 21 structures, **15 pair the legs `(1,2)(3,4)` and 6 pair them `(1,4)(2,3)`**
(`FFFF13` and `FFFF16` write their chains crossed, which reading index labels misses).
After splitting, **80 of the 1985 interactions** carry structures of both pairings, every
one of them same-flavour (`X̄ X X̄ X`). The consequences are in
[four-fermion vertices](../amplitudes/four-fermion-vertices.md).

**Colour strings:** `1`, `Identity`, `T`, `T(-1,·,·)*T(-1,·,·)`, `f`, `f*f`, `f*f*f`
chains. No `d`, no sextets, no baryonic `Epsilon`.

**Coupling orders** (`coupling_orders.py`): `QCD` hierarchy 1, `QED` 2; `NP`, `NPshifts`,
`NPprop`, `NPcpv` and `SMHLOOP` hierarchy 99; one order per operator (`NPcG`, `NPctW`, …)
at hierarchy 1. Every `expansion_order` is 99 except `NPprop`, which is 0. Because `NP`
weighs 99, a SMEFT process has to ask for `NP<=1` explicitly; the automatic `WEIGHTED`
search returns the SM limit. Why `NPprop = 0` caps nothing is in
[coupling orders](coupling-orders.md). [^n35-conv]

**Custom propagators.** `propagators.py` is present and bound to four auxiliary fields,
`Z1`, `W1±`, `t1`, `H1` (PDG 9000005–9000008). All 125 of their vertices carry `NPprop`.
The loader parses the forms; a diagram in which one of these fields propagates is refused
([UFO parsing](ufo-parsing.md)).

**Derived parameters.** The MW-scheme derivation evaluates without NaN over all 315
parameters, e.g. `ee` 0.30825, `sth` 0.47208, `vevhat` 246.22, `yt` 0.99228 in the SM
limit. [^n35-census]

## The restrict cards

| Card | Sets | Use |
|---|---|---|
| `restrict_SMlimit_massless.dat` | every Wilson coefficient zero | the SM limit: 62 interactions |
| `restrict_massless.dat` | every **real** Wilson coefficient at a distinct fixed value (`cG` 0.2, `cW` 0.3, `cH` 0.4, …), `LambdaSMEFT = 1000`; imaginary parts zero | every structure class on at once: 913 interactions |

Both cards keep `MB = 4.18`, `MT = 172.76` and `MH = 125.09` and zero the τ mass and
Yukawa, so SMEFTsim cards stay four-flavour: the `p`/`j` labels do not gain `b b~` (see
[proc-card grammar](../process/proc-card-grammar.md)).

What `restrict_massless` leaves off matters for coverage: its CP-odd block is zero and
its lepton Yukawas are zero, so neither the `Epsilon` structures nor the cyclic
tensor-tensor operator reach a row that imports it. The authored `restrict_vg_*` cards
switch individual classes on for the rows that cover them. A restrict card's non-zero
values are the model's defaults ([restrict-card semantics](restriction-semantics.md)).

## Out of reach by construction

- **Squared-order constraints** (`NP^2==1` for interference only) are not yet
  supported, so every SMEFTsim row compares the full |M|² at `NP<=1`. They are in scope
  as a [backlog item](../backlog/feature/squared-order-constraints-refused.md).
- **Majorana fermions in four-fermion vertices** are a MadGraph `InvalidModel`; SMEFTsim
  has none.

The rows gated on this model, and their tolerances, are in
[non-SM rows](../validation/non-sm-rows.md).

[^n35-census]: Note 35 §1.2–§1.3: the static census and the measured loader probe (2026-09-05).
[^n35-conv]: Note 35 §1.4: `restrict_massless` values and the `expansion_order` rule.
[^n35-f1]: Note 35 §3, four-fermion vertices: pairing census 15/6, 80 of 1985 interactions mix pairings.
[^n35-decisions]: Note 35 §7, decisions of 2026-09-05: vendor rather than submodule; SMEFT rows compare the full |M|² at `NP<=1`.
[^ufo-readme]: `validation/ufo/README.md`.
[^smeftsim-upstream]: The upstream directory at the vendored commit.
[^brivio]: The SMEFTsim 3.0 reference cited in the provenance README.
