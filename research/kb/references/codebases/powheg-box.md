---
type: Codebase Survey
title: POWHEG-BOX-V2
description: "POWHEG-BOX-V2 at e26982d7: an NLO+PS framework around user-supplied matrix elements; the B-tilde integrand, FKS-style subtraction, MINT integration, the user interface and LHEF output."
resource: "https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2/-/tree/e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa"
status: draft
tags: [powheg, nlo, subtraction, external-code, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n03-powheg, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L10-L34", title: "Note 03, surveyed revisions and purpose"}
  - {id: n03-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L165-L368", title: "Note 03 Part 2, POWHEG-BOX-V2 survey"}
  - {id: n03-compare, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/03-sherpa-powheg.md#L369-L383", title: "Note 03 Part 3, comparison table and primary references"}
  - {id: pw-btilde, resource: "https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2/-/blob/e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa/btilde.f#L1-L101", title: "btilde.f"}
  - {id: pw-sigborn, resource: "https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2/-/blob/e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa/sigborn.f#L222-L275", title: "sigborn.f, setborn0"}
  - {id: pw-sigreal, resource: "https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2/-/blob/e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa/sigreal.f#L1-L110", title: "sigreal.f, btildereal"}
  - {id: pw-mint, resource: "https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2/-/blob/e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa/integrator.f#L12", title: "integrator.f, mint"}
  - {id: pw-lhef, resource: "https://gitlab.com/POWHEG-BOX/V2/POWHEG-BOX-V2/-/blob/e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa/lhefwrite.f#L3-L124", title: "lhefwrite.f"}
  - {id: pw-user, resource: "https://gitlab.com/POWHEG-BOX/V2/User-Processes/hvq", title: "POWHEG-BOX V2 user process hvq (GitLab group User-Processes)"}
  - {id: nason, resource: "https://arxiv.org/abs/hep-ph/0409146", title: "P. Nason, A new method for combining NLO QCD with shower Monte Carlo algorithms (2004)"}
  - {id: powheg-box, resource: "https://arxiv.org/abs/1002.2581", title: "Alioli, Nason, Oleari, Re, A general framework for implementing NLO calculations in shower Monte Carlo programs: the POWHEG BOX (2010)"}
---

POWHEG-BOX-V2 is not a matrix-element generator. It is a Fortran 77/90
framework that turns a user's Born, virtual and real matrix elements into
NLO+PS events: it supplies the subtraction, the integration, the Sudakov
(POWHEG) emission and the LHEF output. The method is Nason's[^nason]; the
framework paper is Alioli, Nason, Oleari and Re[^powheg-box]. vibegraph is
leading order, so POWHEG-BOX matters here as the reference for how NLO+PS is
organised ([beyond leading order](../../pipeline/beyond-leading-order.md)).

The submodule `research/refs/powheg-box-v2` is pinned at
`e26982d7ad3d61db9fcbfcdccf4dd281fc12d1aa` (`master`); note 03 surveyed that
commit and the lines below were re-checked there[^n03-p2].

## Framework and user code

The BOX provides `btilde.f` (the NLO integrand), `sigborn.f`, `sigvirtual.f`
and `sigreal.f` (infrastructure around the user's amplitude calls),
`integrator.f` (MINT), `find_regions.f` (singular regions),
`gen_Born_phsp.f` and `gen_real_phsp.f` (phase-space scaffolding) and
`lhefwrite.f` (output).

The user supplies, in a process directory:

| Subroutine | Called from | Supplies |
|---|---|---|
| `setborn(p,bflav,born,bornjk,bmunu)` | `sigborn.f:237` (in `setborn0`) | Born `|M|²`, colour-correlated `bornjk(j,k)`, spin-correlated `bmunu(μ,ν,j)` |
| `setvirtual(p,bflav,virtual)` | `sigvirtual.f:42`, `:74` | the one-loop virtual |
| `setreal(p,rflav,amp2)` | `sigreal.f:1399`, inside the BOX's `sigreal_btl` (`:1011`), which `btildereal` calls at `:52`, `:107` | real-emission `|M|²` for one real flavour structure; the BOX splits it over singular regions (ALR) |
| `born_phsp(xborn)` | `gen_Born_phsp.f` | unit cube to Born momenta |
| `init_processes` | initialisation | the `flst_born` / `flst_real` flavour tables |

`setborn0` wraps the user call: it zeroes any non-finite `born`, `bornjk` or
`bmunu` entry and divides each by the flux `2 * kn_sborn`[^pw-sigborn]:

```fortran
call setborn(p,bflav,born,bornjk,bmunu)
if (.not.pwhg_isfinite(born)) born=0d0
...
born=born/(2*kn_sborn)
```

Process directories are not in this repository; each is its own project under
the GitLab group `POWHEG-BOX/V2/User-Processes` (88 of them, `hvq` for
heavy-quark pairs among them)[^pw-user]. `hvq` holds `nlegborn.h` (which also
sets `nlegreal`, `ndiminteg`, `maxprocborn` and `maxprocreal`), `Born.f`,
`Born_phsp.f`, `virtual.f`, `real.f`, `init_processes.f`, `init_couplings.f`
and a `Makefile`. `bbinit.f` is the BOX's own, not user code.

Data passes between the BOX and user code in common blocks under `include/`:
`pwhg_flst.h` (flavour structures, `flst_nborn`, `flst_born`, `flst_nalr`),
`pwhg_kn.h` (kinematics, `kn_pborn`, `kn_sborn`, `kn_jacborn`), `pwhg_rad.h`,
`pwhg_st.h` (`st_mufact2`, `st_muren2`), `pwhg_br.h` (`br_born`, `br_bornjk`,
`br_bmunu`) and `LesHouches.h` (`nup`, `idup`, `pup`, `istup`, `icolup`).

## Main flow

`pwhg_main.f`: `pwhginit()` (flavour tables via `init_flsttag` at
`pwhg_init.f:16`, flags at 41–114, physics and PDFs via `init_phys` at 150,
then `bbinit` at 203). `bbinit` (`bbinit.f`) runs the integration in stages,
which `#parallelstage` can split across runs: importance-sampling grids
(MINT with `imode = 0`), the upper-bounding envelope for `B̃` (`imode = 1`),
the upper bound for radiation, then events. Per event the main program calls
`pwhgevent()` and `lhefwritev`.

## The B-tilde integrand

`function btilde(xx,www0,ifirst,imode,retval,retval0)` (`btilde.f:1`)
computes the POWHEG `B̃`:

```text
B(Φ_n) + V(Φ_n) − C_integrated(Φ_n) + ∫ [R(Φ_{n+1}) − C(Φ_{n+1})]
```

It calls `btildeborn` (line 63), `btildevirt` (68), `btildecoll` (76, the
integrated collinear remnant) and `btildereal` (81, real minus local
counterterms), and sums per Born flavour structure[^pw-btilde]:

```fortran
tmp=resborn(j)
if (.not.flg_bornonly.and..not.imode.eq.0) then
   tmp = tmp + resvirt(j) + rescoll(j) + resreal(j)
endif
```

`btildeborn` (`sigborn.f:1`) multiplies `br_born(j)` by the two PDFs,
`kn_jacborn` and a MiNLO rescaling factor. The `ifirst` argument drives
**folding**: correlated samples from symmetric regions are combined before they
reach the integrand, reducing variance.

## Subtraction

The local counterterms are FKS-style, not Catani–Seymour dipoles. In
`btildereal` (`sigreal.f:1`), with subtraction on, the real contribution for a
final-state emitter is[^pw-sigreal]:

```fortran
resreal(iuborn)= resreal(iuborn)+rrr-rrrc
#-rrrs+rrrcs+remnant
```

real minus collinear minus soft plus soft-collinear counterterms, plus the
remnant. `collfsr` (`sigcollsoft.f:49`) builds the collinear limit;
`btildecoll` (`sigcollremn.f`) the integrated collinear remnant.
`find_regions` (`find_regions.f:21`) lists every (emitter, radiated) pair of a
real flavour structure, final-state pairs and initial-state emitters, and
`ubornflav` (line 135) maps a region to its underlying Born. Catani–Seymour,
for comparison, is [its own paper](../papers/catani-seymour.md).

## MINT

`mint(fun,ndim,ncalls,nitmax,ifold,imode,…)` (`integrator.f:12`) is a
VEGAS-like adaptive integrator on a 50-interval grid per dimension
(`xgrid(0:50,ndim)`), with folding (`ifold(k) > 1`, a divisor of 50) and, in
`imode = 1`, an upper-bound envelope `ymax` per cell for unweighting.
`mint_upb.f` stores the bounds (`startstoremintupb` at 30, `storemintupb` at
60)[^pw-mint].

## LHEF output

`lhefwrite.f` writes a `<LesHouchesEvents version="3.0">` header with the
`<init>` block (`lhefwritehdr`, line 3), one event per `lhefwritev` (75), and a
trailer that also saves the random-number state (`lhefwritetrailer`, 124). The
particle line is[^pw-lhef]:

```fortran
write(buffer,220) idup(i),istup(i),mothup(1,i),
& mothup(2,i),icolup(1,i),icolup(2,i),(pup(j,i),j=1,5),
& vtimup(i),spinup(i)
220  format(1p,i8,5(1x,i5),5(1x,e16.9),1x,e12.5,1x,e10.3)
```

## What an LO generator takes from it

- The user interface separates matrix elements from infrastructure: Born,
  virtual and real are three callable functions over a shared flavour table.
- MINT's folding reduces variance without the user choosing sampling
  variables.
- `B̃` is the integrand to implement for POWHEG-style NLO+PS (as opposed to
  MC@NLO).
- Colour- and spin-correlated Borns (`bornjk`, `bmunu`) are what NLO
  subtraction needs beyond an LO `|M|²`.

[^n03-p2]: Note 03 Part 2; lines re-checked at `e26982d7`.
[^pw-sigborn]: `sigborn.f:222–275` at `e26982d7`.
[^pw-btilde]: `btilde.f` at `e26982d7`, lines 63–101.
[^pw-sigreal]: `sigreal.f:77–78` at `e26982d7`.
[^pw-mint]: `integrator.f:12` and `mint_upb.f` at `e26982d7`.
[^pw-lhef]: `lhefwrite.f:97–119` at `e26982d7`.
[^nason]: arXiv:hep-ph/0409146.
[^powheg-box]: arXiv:1002.2581.
[^pw-user]: GitLab API listing of group `POWHEG-BOX/V2/User-Processes` and the `hvq` repository tree at its default branch (not pinned), read 2026-10-09; `bbinit.f` at `e26982d7`. Note 03 lists `bbinit.f`, `nlegreal.h`, `maxprocborn.h`, `maxprocreal.h` and `maxalr.h` as user files.
