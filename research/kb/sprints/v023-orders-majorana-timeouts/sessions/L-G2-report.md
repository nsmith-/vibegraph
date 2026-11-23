---
type: Session Report
title: "L-G2 report: the generate resonance-flavour refusal"
description: "generate refused a flavour group whose members carry an s-channel line of different flavours, vanishing and live groups alike; the measurement, the fix that reads each member's line off its own diagrams, and the gates."
status: draft
tags: [generate, refusal, flavour-groups, resonance, g2hdm, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-generate-resonance`, off main 7b36722c with `origin/main`
5b1f9f71 merged in first (multicore generate touched `generate.rs`).

## Step 1: the live-group measurement (before any fix)

**Live groups are refused too.** Card (g2HDM, the UFO copied to scratch from
`/home/user/wt/dl-g2hdm/x/g2HDM_UFO`, never committed):

```
import model g2HDM_UFO
generate g u > h+ d QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
add process g d~ > h+ u~ QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
```

with card 894's param card and one change, `GUR1x1 = 0.5` (`yukawagur 1 1`,
so H± couples to u d̄), and card 894's run card with `lhaid = 247000` (the
fetched set; 325300 is not in `validation/pdf`). The base binary is the
branch after the `origin/main` merge, before any fix (`v0.2.2-22-g39bfd845`).

```
vibegraph -v integrate cards/proc_min2.dat --run-card cards/run_card_247.dat \
    --param-card cards/param_894_rhouu.dat --ufo-dir . --seed 1 --fixed-budget \
    --niter 2 --neval 50000 -j 4 --out runs/min2_rhouu
 INFO phase compile: ... flavour_groups 1, ... subprocesses 2 ...
σ = 21.6 ± 2.1 fb
vibegraph inspect runs/min2_rhouu/grid.bin.zst
set 0 (g u > e+ ve b b~ d, a flavour group of 2 sets):
vibegraph generate runs/min2_rhouu/grid.bin.zst --seed 2 --nevents 200 -j 4 --out runs/min2_rhouu/ev
ERROR the resonance 2 of g u > e+ ve b b~ d has no flavour in the subprocess with outgoing [-11, 12, 5, -5, -2] that shares its matrix element
```

No "vanishes at the card" line: the group is live (σ = 21.6 fb), and the two
members share a matrix element because H± is a scalar and the rest is QCD, so
`g d~ > h+ u~` is the CP image of `g d > h- u`, whose |M|² equals
`g u > h+ d`'s. The massless s-channel quark line is `u` in one member and `d~`
in the other; their legs below carry charge +2/3 and +1/3, so the charge rule
of `member_line_pdg` names neither.

The reproduction itself on the same base binary:

```
vibegraph integrate cards/proc_min3.dat --run-card cards/run_card_247.dat --param-card cards/param_894.dat \
    --ufo-dir . --seed 1 --fixed-budget --niter 1 --neval 20000 -j 4 --out runs/min3
 INFO 1 of 1 flavour groups (2 subprocesses) have a matrix element that vanishes at the card; ...
σ = 2.70 ± 0.24 fb
vibegraph generate runs/min3/grid.bin.zst --seed 2 --nevents 50 -j 4      # rc=1
ERROR the resonance 2 of g u > e+ ve b b~ d has no flavour in the subprocess with outgoing [-11, 12, 5, -5, -2] that shares its matrix element
```

Two Standard-Model cards bracket it (sm, card 894's run card at lhaid 247000,
`--neval 5000 --niter 1`):

| card | groups | generate | what it shows |
|---|---|---|---|
| `generate g q > w+ qq, w+ > e+ ve` with `q = u d~`, `qq = d u~` | 2 (`compile: flavour_groups 2`) | 200 events, rc 0 | `g u > w+ d` and `g d~ > w+ u~` do **not** share a matrix element (the W couples to one chirality, so the CP image is not the same function), so no group carries both lines |
| `g u > w+ d, w+ > e+ ve` + `g c > w+ s, w+ > e+ ve` | 1 (`a flavour group of 2 sets`) | 200 events, rc 0; status-2 lines: 200 × `24` only | the `u` line resolves through the charge rule to `2` for the `c` member as well: wrong, and never written, since a line without width is never on its Breit–Wigner |

**What MadGraph writes for the live card.** MadGraph 3.5.7 `output madevent`
of the same two lines (`/home/user/wt/sprintl-generate-resonance-mg/lg2/mg_min2`)
puts each member in its own `P` directory, and each member's `configs.inc`
carries its own s-channel code on the line above `d`/`u~`:

```
P1_gq_hpq_.../leshouche.inc:  DATA (IDUP(I,1,1),I=1,7)/21,2,-11,12,5,-5,1/
P1_gq_hpq_.../configs.inc:14: DATA (SPROP(I,-4,1),I=1,1)/2/
P2_gq_hpq_.../leshouche.inc:  DATA (IDUP(I,1,1),I=1,7)/21,-1,-11,12,5,-5,-2/
P2_gq_hpq_.../configs.inc:14: DATA (SPROP(I,-4,1),I=1,1)/-1/
```

How MadGraph assigns it, in `research/refs/mg5amcnlo`:

- `madgraph/iolibs/export_v4.py:5527-5537` (`write_configs_file`, grouped
  subprocesses) writes `sprop(i, -n, iconfig)` for every matrix element `i` of
  the group from that matrix element's own diagram for the configuration
  (`v.get('legs')[-1].get('id')`), and `0` for one with no such diagram.
- `madgraph/iolibs/template_files/addmothers.f:196-197` reads
  `jpart(1,i) = sprop(numproc,i,lconfig)`: the drawn matrix element's own
  code. `numproc` is the matrix element, not the flavour row `iproc` inside it
  (`leshouche.inc`'s `IDUP(…, iproc, numproc)`), so flavour rows that share
  one matrix element share its code.
- `madgraph/core/helas_objects.py:206-215` (`IdentifyMETag.vertex_id_from_vertex`)
  keeps the s-channel PDG code in the tag that decides which processes share a
  matrix element, *unless the line has zero width or is forced off shell*:
  "split the processes to ensure that we write the correct resonance in the
  event file". So two flavour rows of one matrix element can differ only on
  lines MadGraph never writes as status 2 outside matching.
- `addmothers.f:252-267` and `:346` write a line as `ISTUP = 2` only on `OnBW`
  (`myamp.f:131-147`, which needs a width) or, under matching, `isbw` of its
  leg set.

So the event record MadGraph would write for a member carries that member's own
diagram's code on every line it writes.
