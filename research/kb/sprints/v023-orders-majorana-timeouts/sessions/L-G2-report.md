---
type: Session Report
title: "L-G2 report: the generate resonance-flavour refusal"
description: "generate refused a flavour group whose members carry an s-channel line of different flavours, vanishing and live groups alike; the measurement, the fix that reads each member's line off its own diagrams, and the gates."
status: draft
tags: [generate, refusal, flavour-groups, resonance, g2hdm, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Branch `sprintl-generate-resonance`, off main 7b36722c with `origin/main`
5b1f9f71 merged in first (multicore generate touched `generate.rs`). "Base"
below is that merge, 39bfd845: current main plus the brief, no code change.
The backlog item `generate-refuses-vanishing-group-resonance-flavour` is closed
and its file deleted in the final commit.

## What changed

- **Each member's line comes from its own diagrams**
  (`vibegraph-lib/src/lhef/resonance.rs`: `MemberLines`, `MemberCode`,
  `LineCode`). A flavour group draws events in its representative's
  configurations. For every member, every configuration and every line, the
  member's code *and colour* are those of its own diagram that has the
  configuration's lines (same leg sets, mass, width, colour up to
  conjugation). Failing such a diagram, any member line on the same legs with
  that mass, width and colour is used. Where several codes qualify, the old
  leg rule (`member_line_pdg`) breaks the tie, and only toward one of them.
  This is what MadGraph does: `sprop(numproc, …)` is the matrix element's own
  diagram's code (below).
- **A line the member does not carry** (no member line on those legs with that
  mass and width: the pole is not in the member's matrix element) holds
  `LineCode::NONE`, MadGraph's `sprop = 0`. An event that would list it is
  written without intermediates and counted in a new warning
  (`resonances: N of M events list no intermediate record because their
  subprocess names no code for a resonance of its configuration`).
- **A widthful line carried under several codes, none of which the legs
  name,** still refuses the run by name ("has no single flavour …"). A
  widthless one in that state holds `LineCode::NONE`.
- **A group that vanishes at the card resolves nothing**
  (`generate.rs::group_resonances`; `FlavorGroup::vanishes` is now
  `pub(crate)`). `ProtonIntegrand` never gives such a group weight
  (`proton.rs:3526`), so it is never drawn.
- **The record reads a member's code with `get`**
  (`generate/source.rs::member_code`). A missing entry, or `LineCode::NONE`,
  writes the event without intermediates and counts it. Nothing indexes past a
  member's table, and a vanishing group's empty tables are never read.
- **The member's colour travels with its code.** Before, a member whose line
  was the representative's antiparticle got the representative's colour, so
  `event_with_intermediates` refused the colour and the event lost every
  intermediate. Card 894 showed it: 103 of 300 events, every one an `h-`
  event, before this change in the same branch, and 0 after.

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

## Step 2: the fix, and where it departs from the brief

The brief proposed, for a live group, "take the line's flavour from the drawn
subprocess's own legs, as MadGraph's per-member event record does". MadGraph
does not read it off legs. It writes each matrix element's own diagram's code
(`sprop`, cited above). Legs alone are not enough, as three cases show:

- In the live g2HDM card the legs name no flavour (charges +2/3 against +1/3).
- In `g u > w+ d` + `g c > w+ s` they name the *wrong* one: the charge rule
  gives `u` for the `c` member. That line is never written, so the error never
  showed.
- With two degenerate widthful states (MSSM `ul`/`cl` shape) the legs would
  write the representative's state into the other member's record. That one
  *is* written.

So the fix reads the member's own diagrams first and keeps the leg rule only
as a tie-break.

A second departure came from card 894 (below). After the first fix commit
(9c38da04), the card was still refused, by a live group
`u b~ > e+ ve b b~ d d~` (h+) with member `u~ b > e- ve~ b~ b d d~` (h-).

- The member's legs are aligned with `d` under `d`, not under the `d~` its CP
  image would need. Its diagrams therefore put the lines the representative
  has on `{e+ ve b b~ d}` on `{e- ve~ b~ b d~}` instead. A temporary debug dump
  of that config, not committed, showed this: the member has `-2`/`-4`/`-6`
  on slot mask 47 and nothing on 31.
- Those representative lines are a `u` and a top. They sit in diagrams through
  the `H± t d` and `H± u d` vertices, which vanish at the card (V_td = 0, no
  ρ_u or ρ_d). Their pole is in neither matrix element, and the two members
  agree numerically only because those diagrams are zero.
- Refusing the run on such a line (my first version did, for widthful lines)
  refused card 894 again.

The rule became the one above: a line no member line sits on holds MadGraph's
`sprop = 0`. MadGraph writes that for a matrix element without the
configuration's line (`export_v4.py:5530-5535`), and `addmothers.f:195-197`
never writes such a line as an `s`-channel resonance. The named refusal stays
for the case that would write a wrong code: a widthful line the member carries
under several codes.

On the brief's "anything still unresolvable stays a named refusal": a line
whose pole is absent from the member is not unresolvable but absent. MadGraph's
answer for it is 0. An event that would still list it is written without
intermediates under a counted warning, not with a guessed code. On card 894
and every row below that warning never fired (0 events).

## Tests

| test | what it pins | what it cannot see |
|---|---|---|
| `vibegraph-cli/tests/cli_generate_vanishing_group.rs` (hermetic: interned `sm-ckm`, a synthetic PDF grid written by the test under the default set's name) | the reproduction's shape, `g q > w+ qq` with `q = u s~` at `lamWS = 0` beside `g u > w+ d`: integrate logs one vanishing group of two subprocesses, generate writes 50 events, every one `g u` with one status-2 `24` | the code of a line the record never writes (all lines that differ between members here are massless); the densities are a stand-in, so nothing about rates |
| `generate::tests::a_live_groups_quark_line_carries_each_members_own_flavour` | `g u > w+ d` + `g d~ > w+ u~` form one live group; the `s`-channel quark line is `(2, 3)` in one member and `(-1, -3)` in the other (code, colour), exactly MadGraph's `SPROP` in each `configs.inc` (below); the leg rule alone names nothing for the second | whether the line is ever written (it is massless, so it is not) |
| `generate::tests::a_group_that_vanishes_at_the_card_resolves_no_resonance_codes` | `restrict_ckm`, `lamWS = 0`: the vanishing group holds no codes; the live group holds codes for every config | the event record (the CLI test reads that) |
| `generate::tests::a_quark_line_of_a_mixed_flavour_group_carries_each_members_quark` (existing, now also asserting colour) | `p p > t t~ j` groups: every quark line carries the member's outgoing quark and its colour sign | lines carried by an incoming leg |
| `lhef::resonance::tests::a_member_line_takes_its_own_diagrams_code` | hand-built lines: a degenerate widthful pair takes the member's code where the leg rule keeps the representative's; a member without the line gets `Absent`; disagreeing diagrams give `Ambiguous` unless the legs name one of them | anything about real diagrams (the lines are hand-built) |
| `generate::source::tests::a_member_code_is_read_only_where_the_member_has_one` | `LineCode::NONE` and out-of-range entries read as no code, never a panic | the event-level fallback (no hermetic card reaches it) |

MadGraph's record for the live case, MadGraph 3.5.7 `output madevent` of
`generate g u > w+ d` / `add process g d~ > w+ u~`
(`/home/user/wt/sprintl-generate-resonance-mg/lg2/mg_wlive`):

```
P1_gq_wpq/leshouche.inc: DATA (IDUP(I,1,1),I=1,4)/21,2,24,1/
P1_gq_wpq/configs.inc:5: DATA (SPROP(I,-1,1),I=1,1)/2/
P2_gq_wpq/leshouche.inc: DATA (IDUP(I,1,1),I=1,4)/21,-1,24,-2/
P2_gq_wpq/configs.inc:5: DATA (SPROP(I,-1,1),I=1,1)/-1/
```

**Mutations** (each built and run, then reverted with `git checkout`; the tree
was clean after, `git diff --stat` empty):

| mutation | cli test | unit tests |
|---|---|---|
| M1: `group_resonances` back to the base rule (leg rule only, no vanishing skip) | **FAILED**: `generate refused a card whose only unresolvable line is in a group that vanishes: ERROR the resonance 2 of g u > e+ ve s has no flavour in the subprocess with outgoing [-11, 12, -2] …` | not run |
| M2: vanishing skip removed, own-diagram resolution kept | passes (the vanishing members' own diagrams carry the line) | `a_group_that_vanishes_at_the_card_resolves_no_resonance_codes` **FAILED** |
| M3: skip kept, leg rule only | not run | `a_live_groups_quark_line_carries_each_members_own_flavour` **FAILED**: `the resonance 2 of g u > W+ d has no flavour in the subprocess with outgoing [24, -2] …` |

M1 to M3 ran on 9c38da04. The second commit changed neither the CLI test nor
the skip. The CLI test alone does not catch M2: in this card's vanishing group
the members share a topology, so the own-diagram path resolves it either way.
The unit test pins the skip.

## Gates

Run on 2573faaa by `/home/user/wt/sprintl-generate-resonance-mg/lg2/gates.sh`,
each cargo step under the heavy lock, with
`CARGO_TARGET_DIR=/home/user/wt/target-generate-resonance CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`:

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | rc 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | rc 0 |
| clippy, extended-validation | `cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings` | rc 0 |
| `validate-lhef` + `validate-unweighting` | `cargo test -p vibegraph-lib --features extended-validation --test validate_unweighting --test validate_lhef -- --nocapture` | rc 0. `validate_lhef`: 3 passed (`banked_files_round_trip_byte_for_byte`, `generated_events_serialise_into_a_coherent_file`, `the_round_trip_is_sensitive_to_every_convention_sensitive_field`), 111 s. `validate_unweighting`: 1 passed (`unweighted_sample_reproduces_the_integration_it_came_from`), 18 s |
| `validate-generate-proton` | `cargo test -p vibegraph --features extended-validation --test cli_generate_proton -- --nocapture --test-threads 1` | rc 0: 12 passed, 2 ignored (oracle-layer and seed-headroom rows, ignored by design). It includes `a_quark_jet_top_pair_is_generated_across_a_mixed_flavour_group` (outgoing quarks labelled {-4…4}), `a_matched_sample_carries_the_shower_record` (2843 of 3000 list the Z) and `ttx_matched_sample_record_fractions_against_madevent` (@1 0.5292 against MadEvent 0.5306, pull −0.20) |
| hermetic suite | `research/refs/mg5amcnlo`, `validation/madgraph/output` and `validation/pdf` moved to `/home/user/wt/sprintl-generate-resonance-mg/hidden` (`ls` of all three: rc 2, absent), then `cargo test --workspace`, then moved back by the script's `EXIT` trap | rc 0: 79 test targets, 2347 passed, 0 failed, 28 ignored. `cli_generate_vanishing_group` ran and passed with no fetched PDF. After the restore `research/refs/mg5amcnlo/models`, `validation/madgraph/output` and `validation/pdf/NNPDF23_lo_as_0130_qed` were back |

The unit and CLI tests above also ran per change. Final run: `cargo test -p
vibegraph-lib --lib -- lhef::resonance generate::` gave 25 passed;
`cargo test -p vibegraph --test cli_generate_vanishing_group` gave 1 passed.

## Bit-for-bit: banked rows with no vanishing or mixed-flavour line

Each row was integrated once with the fix binary at a small fixed budget
(`--seed 11 --fixed-budget`; 20000 × 2 for the proton rows, 5000 × 1 for
`ttx_nested`). The same artifact was then generated with the base binary (39bfd845)
and with the final code (`vibegraph-fix4`, built from 2573faaa), `--seed 7
--nevents 500 -j 4`, and the files compared with `cmp`. Driver:
`/home/user/wt/sprintl-generate-resonance-mg/lg2/{bit/run.sh,round4.sh}`.

| row (cards from the bank) | path exercised | status-2 lines in 500 events | base sha256[:16] | fix sha256[:16] | cmp |
|---|---|---|---|---|---|
| `pp_ttx_lep`: `p p > t t~, t > b e+ ve, t~ > b~ mu- vm~`, `decay_chain_pp13_run_card.dat` | decay chain, groups `u u~`/`c c~`/`d d~`/… | 1990 | be3f59c2a1457820 | be3f59c2a1457820 | IDENTICAL |
| `pp_to_llj_mlm`: `p p > e+ e- j`, its banked `run_card.dat` (ickkw 1, xqcut 20) | matched record, mixed-flavour groups with quark lines | 473 | f83495be4d204451 | f83495be4d204451 | IDENTICAL |
| `pp_to_ttx_0j1j_mlm`: `p p > t t~ @0` + `p p > t t~ j @1`, its banked run card | matched, two multiplicities | 0 | 82d12c25f3dfa445 | 82d12c25f3dfa445 | IDENTICAL |
| `ttx_nested`: `e+ e- > t t~, (t > w+ b, w+ > e+ ve), t~ > w- b~`, `decay_chain_ee500_run_card.dat` | fixed beams | 1500 | 29ee83fa961dc52f | 29ee83fa961dc52f | IDENTICAL |

The same four were also identical on both intermediate builds (9c38da04,
and the uncommitted widthful-only refusal).
`pp_to_ttx_0j1j_mlm` writes no intermediate (its tops are external), so it
tests only that nothing else moved.

## Card 894 on the head

The UFO, cards and command are as in L-G1's addendum. lhaid is 247000 (325300
is not fetched), and the budget is reduced to one iteration at `--neval 6000`
(the per-channel 512-point floor makes that 1.1 M points per iteration). The
integration took 14m23s wall time, under 20 min, so it ran here.

```
vibegraph integrate cards/proc_card.dat --run-card cards/run_card_247.dat --param-card cards/param_894.dat \
    --ufo-dir . --seed 1 --fixed-budget --niter 1 --neval 6000 -j 4 --out runs/c894       # 14m23s
σ = 64.6 ± 5.8 fb
vibegraph generate runs/c894/grid.bin.zst --seed 2 --nevents 300 -j 4 --out runs/c894/events.lhe   # 11m22s, rc 0
 INFO phase generate: 680.761 s (channels 2176, events_written 300) ... outcome="ok"
```

Of the 300 events, 155 are `h+` (status-2 `24 6 37`) and 145 are `h-` (`-24 -6 -37`).
Every event lists its intermediates. The top carries a colour line and the
anti-top an anticolour. One event also lists a `25`. No `resonances:` warning
fired: neither the colour fallback nor the new no-code fallback was used.

The artifact was made by 9c38da04's binary. The resolution code runs only in
`generate`, so the artifact is the same on the head.

Along the way, the same artifact on the binaries in between:

- base: refused (L-G1's message);
- 9c38da04: refused again, by the h+/h- group above;
- a widthful-only refusal: refused, `the resonance 6 of u b~ > e+ ve b b~ d d~ …`;
- before the colour change: 300 events, `103 of 300 events list no
  intermediate record because a resonance's daughters leave colour lines that
  do not fit it`, all of them `h-`.

Remaining warnings on the head run:

- `178 of 2176 channels produced no point in the weight scan`;
- four parts (8, 9, 11, 12, together about 3.7e-5 pb of the 0.0646 pb) drew
  no event in 300.

Both are budget-scale effects of a 300-event, one-iteration run and were not
studied further. The first also fires on the vanishing group's channels of the
minimal card (Found F2).

The reproduction cards on the final binary (`--seed 2 --nevents 200 -j 4`):

| card | generate | status-2 | incoming |
|---|---|---|---|
| `proc_min3` (the brief's five lines, card 894's params) | rc 0, 200 events | 200 × `24 37 6` | 200 × `4 -5` |
| `proc_min2` at `GUR1x1 = 0.5` (the live group) | rc 0, 200 events | 200 × `24 37 6` | 178 × `2`, 22 × `-1` (with 200 × `21`) |
| SM `h_vanish` (the CLI test's card at 13 TeV, real PDF) | rc 0, 200 events | 200 × `24` | 200 × `2 21` |

In the live card the `d~` member's events carry no status-2 quark. Neither
MadGraph nor vibegraph writes a massless line. Their `W`, top and `H+` codes
are the same in both members.

## Release note

> `vibegraph generate` no longer refuses a run whose flavour group joins
> subprocesses with different `s`-channel flavours (g2HDM's `g u > h+ d` with
> `g d~ > h+ u~`, `u b~ > h+ …` with `u~ b > h- …`; card 894 generates). Each
> subprocess's intermediates now carry its own code and colour, as MadGraph
> writes them. Before, an `h-` event drawn in an `h+` group lost its
> intermediates. A group whose matrix element vanishes at the card is no
> longer consulted.

## Corrections to the brief

- **The mechanism MadGraph uses is the member's diagram, not its legs.** See
  step 2. The brief's "as MadGraph's per-member event record does" is right
  about the outcome and wrong about the source. `sprop` is per matrix
  element (`numproc`), not per flavour row (`iproc`).
- **The live case exists, and it is not only the massless quark.** The g2HDM
  live group at `GUR1x1 = 0.5` is refused exactly as the vanishing one is. On
  card 894 a second live shape appeared: h+ grouped with its CP image h-,
  member legs aligned `d` under `d`. It needed the "absent line" rule and a
  colour fix the brief did not anticipate.
- **No new toy UFO was needed.** The interned SM reproduces the group
  structure: `restrict_ckm` with `lamWS = 0` makes `g u > w+ s` and
  `g s~ > w+ u~` vanish and share a group. The proton-beam CLI test needs a
  PDF set, so it writes a synthetic grid. The hermetic layer has no fetched
  set.
- **Paths and lines.** `member_line_pdg` is at `lhef/resonance.rs:356` and
  `group_resonances` at `generate.rs:1543` on the merged base (the brief's
  ~L1514/L1539/L1542 are before the multicore merge). The brief's
  `CARGO_TARGET_DIR` (`<worktree>/target`) differs from the assignment's
  (`/home/user/wt/target-generate-resonance`). I used the assignment's.
- **Gate commands.** The pixi tasks run `--profile release-debug`, and
  `pixi run -e madgraph` needs the 2.5 GB environment the discipline forbids
  installing here. The same `cargo test` targets ran directly in the
  workspace's dev profile, which is optimised with debug assertions on. The
  commands are below.

## Long-run requests

None. The longest job, card 894's integration at reduced budget, took 14m23s
here.

## Found

- **F1: a group member's legs can be aligned against its own diagrams.** On
  card 894 the live group joins `u b~ > e+ ve b b~ d d~` (h+) with
  `u~ b > e- ve~ b~ b d d~` (h-).
  - *Evidence:* the member's outgoing legs sit `d` under `d` and `d~` under
    `d~`, where its CP image would need `d~` under `d`. Its diagrams therefore
    put lines on leg sets the representative's do not have: slot mask 47 holds
    `-2`/`-4`/`-6` in the member, and the representative has `2`/`6` on mask
    31 (temporary dump of config 8, not committed). The two matrix elements
    agree only because the misaligned diagrams vanish at the card. Resonance
    codes are now safe, because such a line is "absent" and never written.
  - *Not measured:* anything else that reads a member's flavours onto the
    representative's configuration, chiefly the matched clustering and
    `<mgrwt>` record of the h- events. `proton.rs:3518-3525` already warns that
    reading a member's flavours onto the representative's clustering can build
    a vertex none of its diagrams has, but says so only for vanishing groups.
  - *To close:* compare card 894's h- matched records (clustering, `<scales>`,
    `<mgrwt>`) against MadEvent's for the same subprocess. If they differ,
    align members by the leg pairing that maps their diagrams onto the
    representative's.
- **F2: the weight-scan warning counts a vanishing group's channels as lost
  cross section.**
  - *Evidence:* `proc_min3` generate prints `WARN 2 of 3 channels produced no
    point in the weight scan and will never be drawn from ([1, 2]); their share
    of the banked cross section is missing from the sample`. Channels 1 and 2
    belong to the group that vanishes at the card, whose share is zero.
  - *To close:* leave channels of vanishing groups out of that count, or say
    their share is zero.
- **F3: no end-to-end card for a written, widthful line whose flavour differs
  between members.**
  - *Evidence:* the MSSM shape (`e+ e- > ul ul~, ul > d x1+` with the same for
    `cl`, degenerate in `restrict_default`) is refused at enumeration: "a
    decay-chain diagram carries a Majorana fermion or a
    fermion-number-violating vertex …". Only a hand-built unit test pins that
    the member writes its own code there.
  - *To close:* a small authored UFO with two degenerate widthful coloured
    scalars decaying to SM quarks and leptons, and a comparison of its event
    records against MadEvent's.
