# Vendored UFO models

UFO model directories the validation layer reads that are not the interned
Standard Model. Each is committed **byte for byte** as its upstream shipped
it — the loader must handle the real file, so nothing here is pre-processed —
together with its licence and a `SHA256SUMS` manifest so drift is detectable:

```bash
(cd validation/ufo/<model> && sha256sum -c SHA256SUMS)
```

Vendoring rather than a submodule (user decision, note 35 §7 D1): the one
directory a gate reads is under a megabyte, while its upstream repository is
~100 MB of FeynRules sources and notebooks that CI's banked job would have to
check out on every run.

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

## `vibegraph_toy_UFO` (planned, note 35 §6)

A hand-written minimal model, ours (MIT OR Apache-2.0 like the rest of the
repository), carrying the Lorentz and colour structures SMEFTsim never emits:
a literal `Sigma`, `d(a,b,c)`, baryonic colour `Epsilon`, sextet `K6`.
