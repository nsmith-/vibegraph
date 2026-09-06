# UFO models

UFO model directories the validation layer reads that are not the interned
Standard Model. Two kinds live here.

**Vendored** directories are committed **byte for byte** as their upstream
shipped them — the loader must handle the real file, so nothing here is
pre-processed — together with the upstream licence and a `SHA256SUMS` manifest
so drift is detectable:

```bash
(cd validation/ufo/<model> && sha256sum -c SHA256SUMS)
```

Vendoring rather than a submodule (user decision, note 35 §7 D1): the one
directory a gate reads is under a megabyte, while its upstream repository is
~100 MB of FeynRules sources and notebooks that CI's banked job would have to
check out on every run.

**Authored** directories are this repository's own, under its own licence
(`MIT OR Apache-2.0`, the terms of `LICENSE-MIT` and `LICENSE-APACHE` at the
repository root). They carry no `SHA256SUMS`: there is no upstream to drift
from, and git is what records their history. Each one exists to put a Lorentz
or colour structure no published model in reach isolates into a single vertex
with a coupling we choose, so a gate that fails names the structure rather than
a corner of a 900-vertex model.

## `SMEFTsim_topU3l_MwScheme_UFO`

| | |
|---|---|
| Upstream | https://github.com/SMEFTsim/SMEFTsim, path `UFO_models/SMEFTsim_topU3l_MwScheme_UFO/` |
| Version | tag `v3.0.2`, commit `db7d4a80bdcff424eee27dde71f1eb09ac894039` (2021-01-24); `version.info` inside the directory says the same |
| Copied | 2026-09-05, every file of the upstream directory unchanged, plus the repository's `LICENSE` (MIT, © 2020 SMEFTsim) |
| Reference | I. Brivio, *SMEFTsim 3.0 — a practical guide*, JHEP 04 (2021) 073, arXiv:2012.11343; validated upstream per CERN-LPCC-2019-02 |
| Role | The non-SM test case of the `ufo-lorentz` sprint (note 35): `topU3l` flavour assumption, `{m_W, m_Z, G_F}` inputs; 21 particles, 260 Lorentz structures, 904 vertices. `restrict_SMlimit_massless.dat` is the SM-limit card (every Wilson coefficient zero); `restrict_massless.dat` sets every real coefficient to a fixed non-zero value |

Not compiled into any binary, so it is outside `THIRD-PARTY-NOTICES`; the
licence file travels with the copy as MIT requires.

The other nine SMEFTsim variants (`alphaScheme`, `U35`, `MFV`, `general`,
`top`) are not vendored; fetch them from upstream at the same tag if a gate
ever needs one.

## `vibegraph_toy_UFO`

| | |
|---|---|
| Provenance | Authored in this repository, 2026-09-06 |
| Licence | `MIT OR Apache-2.0`, the repository's own |
| Role | The Lorentz structures SMEFTsim never emits, one per vertex. A literal `Sigma` in an FFV dipole and in a tensor-tensor four-fermion contact; the bare `Identity` and `Gamma5` scalar bilinears; and the symmetric colour structure constant `d(1,2,3)`, which this crate's colour algebra can represent and no Standard-Model process reaches |

FeynRules expands `sigma^{mu nu}` into `gamma^mu gamma^nu` chains before it
writes a UFO, so a literal `Sigma` appears in ALOHA's object library and in no
model file anywhere — which is why it is here rather than in a published model.
The four-fermion contact is written twice, once with `Sigma` and once as the
gamma-gamma expansion of the same operator, under two coupling orders, so
MadGraph splits them into two diagrams and the two spellings can be compared
against each other per helicity inside one process.

Five fields: a Dirac lepton `lt`, a colour-triplet Dirac quark `qt`, a singlet
scalar `st`, a singlet vector `vt` and a real colour-octet scalar `o8`. Masses
and widths are free external parameters chosen to keep every banked row at
sqrt(s) = 500 GeV above threshold and off every pole. Restrict cards, inside the
model directory because the model is ours: `restrict_dipole`, `restrict_tensor`,
`restrict_yukawa`, `restrict_dcolor` — one structure class each — and
`restrict_all` with everything switched on.

## `vibegraph_toy_color_UFO`

| | |
|---|---|
| Provenance | Authored in this repository, 2026-09-06 |
| Licence | `MIT OR Apache-2.0`, the repository's own |
| Role | The colour atoms this crate's colour grammar refuses outright: the baryonic `Epsilon`/`EpsilonBar`, and the sextet Clebsch coefficients `K6`/`K6Bar` |

Separate from `vibegraph_toy_UFO` on purpose. The colour grammar
(`vibegraph-lib/src/ufo/color.rs`) parses `Identity`, `T`, `f` and `d` and
nothing else, so a model containing an `Epsilon` or `K6` colour string is
refused at *load*: putting these atoms in the same model as the Lorentz
structures would make every toy row red for a reason that has nothing to do
with the row.

Every field is a scalar — two distinct colour triplets `p3` and `r3`, a triplet
diquark `d3`, a sextet diquark `d6` and a singlet `st`. Scalars rather than
quarks: two same-representation fermions reach a diquark only through a
fermion-number-violating vertex, which needs charge-conjugation machinery this
crate does not have, so a fermionic model would measure that absence instead of
the colour atoms. Two distinct triplets rather than one repeated field because
`Epsilon(1,2,3)` is antisymmetric in its first two indices and would annihilate
the vertex. Restrict cards inside the model directory: `restrict_eps`,
`restrict_k6`, `restrict_all`.

Its two colour deltas are written as an explicit `T(2,1)` rather than as
`Identity(1,2)`, because a model with no `T(a,i,j)` vertex gives MadGraph
nothing to infer the 3/3bar labelling from and its fallback labels them the
other way round. That reversal is a uniform transpose of the colour basis, which
nothing can see — until an `Epsilon` pins the absolute assignment, and then
MadGraph's own colour matrix stops reducing. `vertices.py` records the exact
code path.

Both diquark vertices are listed twice, once conjugated. MadGraph keys its
vertex lookup on the sorted PDG tuple of an interaction's particles and flips a
process's initial-state legs to their antiparticles first, so an interaction
whose particle multiset is not self-conjugate is reachable from one side only
unless the h.c. term is in the model too. Every Standard-Model vertex happens to
be self-conjugate as a multiset, which is why nothing in this repository had met
the rule before.
