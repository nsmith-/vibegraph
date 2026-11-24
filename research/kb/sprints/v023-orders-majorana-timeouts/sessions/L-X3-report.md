---
type: Session Report
title: "L-X3 report: per-member XWGTUP within one multiplicity part"
description: "The per-member unit weights reported against v0.2.2 do not reproduce with v0.2.2 or main; no per-member factor enters the written weight; the per-member σ shares of 30000-event samples agree with sixteen single-member integrations within statistics."
status: draft
tags: [events, unweighting, lhef, normalisation, mlm, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
Branch `sprintl-xwgtup` off main cd35c4e6 (brief commit 54c31fe3). A
diagnosis session: no source change. Brief: [L-X3](L-X3.md).

## Summary

- **The observation does not reproduce.** With the v0.2.2 release binary
  (sha256 `7ccc24af…`) and with main (`54c31fe3`), every event of a part that
  is not an overweight carries the same `XWGTUP`, whatever its flavour member.
  Three files were checked: 30000 events per binary on the brief's card, and
  1000 events at `-j 8` with v0.2.2 on the user's card shape (`xqcut = 10`,
  lhaid 325300).
- **No per-member factor enters the written weight.** The weight is
  `max(1, w/w_max_j)` from the accept/reject pass, times one scale per part.
  The member draw and MLM's per-member reweighting set *which* member an
  accepted point is labelled with, in proportion to that member's term of the
  point's value. They never touch the weight.
- **Bias check: no bias resolved.** Each member's share of its part's ΣXWGTUP
  was compared with its σ from an integration of that member alone, sixteen
  members in all. On main, Σpull² is 5.78 over 3 dof for 0j and 9.23 over 11
  dof for 1j. On v0.2.2 it is 3.77 over 3 and 15.35 over 11. The largest
  tension is `c g` in 1j: −1.91 on main and −2.27 on v0.2.2. That is
  marginal, and a long-run request is below.
- **Verdict: neither docs-only nor bias.** The kb pages are right on the
  point at issue: one factor per part, and no weight that depends on the
  member. The reported per-member weights are not something either binary
  writes. An optional clarifying sentence for the two pages is below. No fix
  is proposed.

## Reproduction

Inputs, in the scratchpad (`$S` = `/tmp/claude-0/-home-user-vibegraph/87596584-b164-5456-b1bc-28aaced93371/scratchpad/lx1`):
- `proc_card.dat` = `import model sm` / `generate p p > e+ e-` / `add process p p > e+ e- j`, with no `@N` tags, as the user wrote it;
- `run_card.dat` = `validation/madgraph/pp_to_llj_mlm_run_card.dat`, unchanged: 13 TeV, lhaid 247000, `ickkw = 1`, `xqcut = 20`, `use_syst = True`, `systematics_program = none`;
- `run_card_user.dat` = the same card with `lhaid = 325300` and `xqcut = 10`, the user's two values;
- `LHAPDF_DATA_PATH=/home/user/wt/sprintl-xwgtup/validation/pdf` (the fetched NNPDF23 set). 325300 was read from the `~/.vibegraph/pdf` cache.

The commands (each under `flock /home/user/wt/heavy.lock`, with `--no-network --no-tui`):

```text
vibegraph-v022 integrate proc_card.dat --run-card run_card.dat --out $S/v022 --seed 419216344 -j 4
vibegraph-main integrate proc_card.dat --run-card run_card.dat --out $S/main --seed 419216344 -j 4
vibegraph-v022 generate $S/v022/grid.bin.zst --seed 419216344 --nevents 30000 -j 4 -o $S/v022/events.lhe
vibegraph-main generate $S/main/grid.bin.zst --seed 419216344 --nevents 30000 -j 4 -o $S/main/events.lhe
vibegraph-v022 generate $S/v022/grid.bin.zst --seed 419216344 --nevents 1000 -j 8 -o $S/v022/events-1k-j8.lhe
vibegraph-v022 integrate proc_card.dat --run-card run_card_user.dat --out $S/v022u --seed 419216344 -j 4
vibegraph-v022 generate $S/v022u/grid.bin.zst --seed 419216344 --nevents 1000 -j 8 -o $S/v022u/events.lhe
python3 -I $S/tab.py <events.lhe>
```

The two integrations agree to every printed digit:

```text
INFO 2 outgoing legs (@1; masses 0, 0 GeV): σ = 665.13 ± 0.51 pb over 1 channels
INFO 3 outgoing legs (@2; masses 0, 0, 0 GeV): σ = 269.26 ± 0.74 pb over 6 channels
σ = 934.40 ± 0.90 pb
```

The two 30000-event samples differ, because main's blocked multicore pass
draws a different sample from the same seed.

`tab.py` reads every event's `IDPRUP`, `XWGTUP`, its incoming partons
(status −1) and its outgoing particles (status 1). It groups the events by
part (2 or 3 outgoing) and by member, the ordered incoming flavours → the
outgoing flavours. For each member it gives:
- the count;
- the modal `XWGTUP` (to 6 s.f.) and how many events sit at it;
- the number of distinct values;
- the minimum and maximum;
- the member's Σw/N.

### v0.2.2, 30000 events, the brief's card

```text
events 30000  sum XWGTUP/N = 934.398
== part with 2 outgoing: 21406 events, sum/N = 665.135 pb, min XWGTUP 923.853, distinct (6 s.f.) 371
   weight above the part's minimum: 371 events, 2.6100% of the part's sum
   member (in > out)          n  mode XWGTUP #distinct    min      max  n@mode
   d d~ > e+ e-            4405    923.853        68   923.85   3888.4   4338
   d~ d > e+ e-            4331    923.853        75   923.85   2779.9   4257
   u~ u > e+ e-            4004    923.853        80   923.85   3894.7   3925
   u u~ > e+ e-            3934    923.853        81   923.85   7268.9   3854
   s~ s > e+ e-            1491    923.853        28   923.85   3045.5   1464
   s s~ > e+ e-            1398    923.853        16   923.85   2373.7   1383
   c c~ > e+ e-             939    923.853        16   923.85   2399.2    924
   c~ c > e+ e-             904    923.853        15   923.85   2287.1    890
== part with 3 outgoing: 8594 events, sum/N = 269.263 pb, min XWGTUP 923.572, distinct (6 s.f.) 201
   weight above the part's minimum: 200 events, 4.0287% of the part's sum
   every one of the 24 members has mode 923.572 (g u: 720/744 there, d d~ > e+ e- g: 514/522,
   s s~ > e+ e- g: 113/119, c c~ > e+ e- g: 72/72, ...)
```

### main 54c31fe3, 30000 events, the brief's card

```text
events 30000  sum XWGTUP/N = 934.398
== part with 2 outgoing: 21383 events, sum/N = 665.135 pb, min XWGTUP 924.98, distinct (6 s.f.) 374
   weight above the part's minimum: 373 events, 2.6071% of the part's sum
   d~ d 4353 @924.98 (4275 at mode) · d d~ 4304 (4240) · u~ u 3978 (3901) · u u~ 3935 (3852)
   s~ s 1528 (1507) · s s~ 1408 (1392) · c c~ 975 (956) · c~ c 902 (887)
== part with 3 outgoing: 8617 events, sum/N = 269.263 pb, min XWGTUP 917.735, distinct (6 s.f.) 194
   weight above the part's minimum: 193 events, 4.2944% of the part's sum
   every one of the 24 members has mode 917.735
```

### v0.2.2, the user's card shape (`xqcut = 10`, lhaid 325300), 1000 events, `-j 8`

```text
σ = 1.4465 ± 0.0014 nb   (@1 845.70 ± 0.69 pb, @2 600.8 ± 1.3 pb)
events 1000  sum XWGTUP/N = 1446.51
== part with 2 outgoing: 607 events, min XWGTUP 1387.92, distinct (6 s.f.) 10
   all 8 members: mode 1387.92 (d d~ 125/125, s s~ 58/58, u u~ 92/94, c c~ 25/25, ...)
== part with 3 outgoing: 393 events, min XWGTUP 1520.64, distinct (6 s.f.) 8
   all 24 members: mode 1520.64 (d g 18/18, d~ g 19/20, d d~ > e+ e- g 27/27, ...)
```

The brief's card at `-j 8` and 1000 events (v0.2.2) gives the same picture:
one value per part, 930.058 for 0j, with 10 overweights.

**The reproduction table.** Rows are the members the brief names. Columns are
the modal `XWGTUP` per member, with events at the mode over the member's
events.

| part | member (both orderings) | user, v0.2.2 | v0.2.2, 30k | main, 30k | v0.2.2, user card, 1k |
|---|---|---|---|---|---|
| 0j | d d~ | 1366.11 (180) | 923.853 (8595/8736) | 924.98 (8515/8657) | 1387.92 (222/225) |
| 0j | s s~ | 1701.34 (74) | 923.853 (2847/2889) | 924.98 (2899/2936) | 1387.92 (117/118) |
| 0j | u u~ | 1505.17 (142) | 923.853 (7779/7938) | 924.98 (7753/7913) | 1387.92 (198/202) |
| 0j | c c~ | 1225.35 (36) | 923.853 (1814/1843) | 924.98 (1843/1877) | 1387.92 (61/62) |
| 1j | d g, d~ g | 1455.43 | 923.572 | 917.735 | 1520.64 |
| 1j | u g, u~ g | 1233.90 | 923.572 | 917.735 | 1520.64 |
| 1j | d d~ > e+ e- g | 1771.95 | 923.572 | 917.735 | 1520.64 |

The two units within one file differ only through the per-part
normalisation, `σ_k N / Σ_{i∈k} w_i`. On v0.2.2 at 30000 events the sample
over-estimated both parts by about the same amount (+0.259% and +0.290%), so
the two units land near each other: 923.853 and 923.572.

The user's grouping is also not vibegraph's. `d g` and `d~ g` are two
different flavour groups here (`g d > e+ e- d` and `g d~ > e+ e- d~`), and
`d g` shares its group, and so its matrix element, with `s g`. A factor keyed
on vibegraph's groups would pair `d g` with `s g`, not with `d~ g`. Nothing in
either binary is keyed on the user's pairing.

## Where the weight comes from

Line numbers are main's. The v0.2.2 line follows in brackets where the code
is identical there.

1. **The accept/reject pass gives a weight of 1, or `r > 1` for an
   overweight** (`vibegraph-lib/src/unweight.rs:860-871` [v0.2.2 `:786`]):

   ```rust
   if accept >= r {
       return None;
   }
   let weight = r.max(1.0);
   stats.accepted += 1;
   stats.event_weight_sum += weight;
   Some(AcceptedPoint {
       channel: j,
       u: point.to_vec(),
       weight,
   })
   ```

   `r = w / channel.w_max` is per channel. The channel is a merged
   `(group, diagram)` map, so `w_max_j` sets the acceptance, not the kept
   weight.

2. **The proton source passes that weight through unchanged**
   (`vibegraph-lib/src/generate/source.rs:705-711`):

   ```rust
   Some(WeightedEvent {
       record,
       weight: point.weight,
       part: k,
       reweights,
       lo_record,
   })
   ```

3. **The emitter scales every event of part `k` by one factor**
   (`vibegraph-lib/src/lhef/emit.rs:745-749`, `:707-712`, `:642`
   [v0.2.2 `:746`]):

   ```rust
   t.scale = if t.weight > 0.0 {
       part.sigma_pb * events_per_weight / t.weight
   } else {
       0.0
   };
   ...
   fn written_weight(norm: EventNorm, scale: f64, w: f64) -> f64 {
       if norm_rule(norm).flatten { 1.0 } else { scale * w }
   }
   ...
   let xwgtup = written_weight(norm, tally[event.part].scale, event.weight);
   ```

   `finished_record` (`emit.rs:463-503`) sets `record.weight = xwgtup`. It
   multiplies only the systematics and reweighting `<rwgt>` entries by it,
   which leaves `XWGTUP` itself alone.

4. **The member is drawn, not weighted** (`vibegraph-lib/src/proton.rs:4251`,
   `:4260-4279` [v0.2.2 `:4216`]):

   ```rust
   let group = select_index(&terms, u[0])?;
   ...
   let weights: Vec<f64> = g
       .members()
       .iter()
       .enumerate()
       .flat_map(|(i, member)| {
           let s = member.symmetry_factor();
           let [direct, _] = g.member_luminosity_rows(i, &direct_rows[0], &direct_rows[1]);
           let [_, mirrored] =
               g.member_luminosity_rows(i, &mirrored_rows[0], &mirrored_rows[1]);
           let r = [direct_rewgt, mirrored_rewgt].map(|f| f.map_or(1.0, |f| f[i]));
           match (direct_rewgt, mirrored_rewgt) {
               (None, None) => [s * direct * m2[group][0], s * mirrored * m2[group][1]],
               _ => [
                   s * direct * r[0] * m2[group][0],
                   s * mirrored * r[1] * m2[group][1],
               ],
           }
       })
       .collect();
   let picked = select_index(&weights, u[1])?;
   ```

   This is the brief's first candidate, member selection within a flavour
   group. The draw is `∝ S_i · L_i^o(x₁, x₂, μ_F) · r_i · |M_g|²`.

5. **MLM's per-member reweighting** is the brief's second candidate. It
   enters the integrand's value itself, as a member-weighted luminosity
   (`proton.rs:331-351`, `term_luminosity`, called at `:3569`):

   ```rust
   g.members()
       .iter()
       .zip(factors)
       .enumerate()
       .map(|(i, (member, factor))| {
           member.symmetry_factor()
               * g.member_luminosity_rows(i, &rows[0], &rows[1])[slot]
               * factor
       })
       .sum()
   ```

   `ordering_terms` (`:4116-4185`) rebuilds the same sum for the selection,
   at the same scales and the same factors.

The integrand's value at a point is `Σ_g avg_g Σ_i S_i L_i r_i |M_g|²`. The
group and the member are drawn with probability equal to their term's share
of that value. So the accepted sample populates member `i` at rate `σ_i`, and
**no compensating factor is needed. None is applied.** A per-member factor
is not intended, and nothing compensates for one. MLM's `r_i` is part of the
member's cross section, carried by the draw, not by the weight.

## Bias check

**Per-member σ.** The integration reports σ per channel and per (group,
configuration) (`vibegraph inspect`), never per member. The members of a
group share their channels. So each member was integrated on its own, with
main, on the same run card and seed. The card held the member and its beam
mirror (`generate d d~ > e+ e-` / `add process d~ d > e+ e-`; `generate d g >
e+ e- d` / `add process g d > e+ e- d`; …). That is the same flavour group of
two sets the mixed run builds, with one member. The driver is
`$S/members/run_members.sh`. Its `$S/members/STATUS` records:

```text
dd rc=0 σ = 268.16 ± 0.26 pb      dg rc=0 σ = 42.170 ± 0.039 pb     ddg rc=0 σ = 32.279 ± 0.032 pb
ss rc=0 σ = 89.503 ± 0.089 pb     ug rc=0 σ = 45.566 ± 0.044 pb     uug rc=0 σ = 32.128 ± 0.030 pb
uu rc=0 σ = 250.44 ± 0.25 pb      dxg rc=0 σ = 25.737 ± 0.025 pb    ssg rc=0 σ = 7.9739 ± 0.0077 pb
cc rc=0 σ = 56.757 ± 0.056 pb     uxg rc=0 σ = 19.434 ± 0.018 pb    ccg rc=0 σ = 4.8196 ± 0.0047 pb
                                   sg rc=0 σ = 16.807 ± 0.016 pb     cg rc=0 σ = 12.322 ± 0.011 pb
                                   sxg rc=0 σ = 16.818 ± 0.016 pb    cxg rc=0 σ = 12.326 ± 0.012 pb
```

**Closure against the mixed run's own per-group σ** (`vibegraph-v022 inspect
$S/v022/grid.bin.zst`, the sum over a group's configurations):

| group | Σ configurations (mixed run) | Σ members (own integrations) | difference |
|---|---|---|---|
| 0j d d~ (d, s) | 357.76 ± 0.26 | 357.66 ± 0.28 | −0.3σ |
| 0j u u~ (u, c) | 307.37 ± 0.25 | 307.20 ± 0.26 | −0.5σ |
| 1j g u (u, c) | 58.12 ± 0.16 | 57.89 ± 0.05 | −1.4σ |
| 1j g d (d, s) | 59.06 ± 0.15 | 58.98 ± 0.04 | −0.5σ |
| 1j g u~ (u~, c~) | 31.77 ± 0.08 | 31.76 ± 0.02 | −0.1σ |
| 1j g d~ (d~, s~) | 42.63 ± 0.10 | 42.56 ± 0.03 | −0.7σ |
| 1j u u~ g (u, c) | 37.15 ± 0.12 | 36.95 ± 0.03 | −1.6σ |
| 1j d d~ g (d, s) | 40.53 ± 0.12 | 40.25 ± 0.03 | −2.1σ |
| 0j part | 665.13 ± 0.51 | 664.86 ± 0.38 | −0.4σ |
| 1j part | 269.26 ± 0.74 | 268.38 ± 0.10 | −1.2σ |

The per-configuration errors from VEGAS are the integration's own. They are
correlated across a group's configurations and taken here as independent. So
the two q q̄ g rows (0.5–0.7% low) are compatible with that.
They are not a demonstration. They do not enter the share test below, which
normalises the members among themselves.

**The test.** `python3 -I $S/bias.py <events.lhe> $S/members.tsv 665.1349
0.5143 269.2632 0.7423`. Both orderings are pooled into one member.
- Sample share `f_i = Σ_{e∈i} XWGTUP_e / Σ_{e∈part} XWGTUP_e`, with the
  weighted binomial error `√(Σ_e w_e²(1_{e∈i} − f_i)²)/Σ_e w_e`.
- Integrated share `g_i = σ_i / Σ_j σ_j` over the part's member integrations,
  with the error propagated from the sixteen independent errors.
- Pull `(f − g)/√(δf² + δg²)`.

The integrations are 0.1% precise, so the sample's error dominates every
pull.

```text
=== v0.2.2 (30000 events)
part member      n  sample share   ± stat  int share    ± int   pull
   2     cc   1843       0.08616  0.00193    0.08537  0.00009  +0.41
   2     dd   8736       0.40735  0.00337    0.40333  0.00029  +1.19
   2     ss   2889       0.13475  0.00234    0.13462  0.00014  +0.05
   2     uu   7938       0.37174  0.00333    0.37668  0.00028  -1.48
     part 0j: Σpull² = 3.77 over 4 members (3 dof)
   3    ccg    153       0.01779  0.00144    0.01796  0.00002  -0.12
   3     cg    354       0.04101  0.00216    0.04591  0.00004  -2.27
   3    cxg    369       0.04283  0.00221    0.04593  0.00005  -1.40
   3    ddg   1039       0.12053  0.00356    0.12027  0.00011  +0.07
   3     dg   1417       0.16454  0.00406    0.15713  0.00013  +1.82
   3    dxg    819       0.09434  0.00315    0.09590  0.00009  -0.49
   3     sg    559       0.06534  0.00274    0.06262  0.00006  +0.99
   3    ssg    237       0.02737  0.00176    0.02971  0.00003  -1.33
   3    sxg    567       0.06631  0.00277    0.06266  0.00006  +1.32
   3     ug   1442       0.16876  0.00413    0.16978  0.00014  -0.25
   3    uug   1029       0.11966  0.00354    0.11971  0.00010  -0.01
   3    uxg    609       0.07153  0.00295    0.07241  0.00007  -0.30
     part 1j: Σpull² = 15.35 over 12 members (11 dof)
=== main 54c31fe3 (30000 events)
   2     cc   1877       0.08787  0.00195    0.08537  0.00009  +1.29
   2     dd   8657       0.40432  0.00337    0.40333  0.00029  +0.29
   2     ss   2936       0.13695  0.00236    0.13462  0.00014  +0.99
   2     uu   7913       0.37085  0.00332    0.37668  0.00028  -1.75
     part 0j: Σpull² = 5.78 over 4 members (3 dof)
   3    ccg    158       0.01817  0.00145    0.01796  0.00002  +0.15
   3     cg    356       0.04169  0.00221    0.04591  0.00004  -1.91
   3    cxg    380       0.04426  0.00230    0.04593  0.00005  -0.72
   3    ddg   1051       0.12119  0.00356    0.12027  0.00011  +0.26
   3     dg   1394       0.16069  0.00402    0.15713  0.00013  +0.88
   3    dxg    815       0.09366  0.00316    0.09590  0.00009  -0.71
   3     sg    535       0.06302  0.00275    0.06262  0.00006  +0.14
   3    ssg    235       0.02687  0.00173    0.02971  0.00003  -1.64
   3    sxg    561       0.06467  0.00269    0.06266  0.00006  +0.75
   3     ug   1481       0.17114  0.00412    0.16978  0.00014  +0.33
   3    uug   1032       0.12175  0.00381    0.11971  0.00010  +0.53
   3    uxg    619       0.07290  0.00298    0.07241  0.00007  +0.16
     part 1j: Σpull² = 9.23 over 12 members (11 dof)
```

How to read it:
- **Size resolved.** At 30000 events a member's share is known to 0.8%
  relative (0j d d~) up to 8% relative (1j c c~ g). Every share agrees at
  that precision.
- **The χ².** The χ² per part is consistent with statistics (p ≈ 0.12 for
  main's 0j, ≈ 0.17 for v0.2.2's 1j).
- **The one coherent tension.** `c g` is low in both samples. Within the
  `g u` group, `c/(u + c)` reads 0.1955 (v0.2.2) and 0.1959 (main) against
  the integrations' 0.2128, about −1.8σ each on 1796 and 1837 group events.
  The two samples are independent draws from one grid. Combined, the `c g`
  pull is about −2.9σ. Among sixteen members that is p ≈ 0.05 for the
  largest. That is not a bias at this budget, and the long-run request
  below resolves it.
- **No compensation is needed.** No member carries a weight that compensates
  its draw, so a correct member share is what the code predicts. It is not
  a docs-versus-code coincidence.

## Verdict and kb wording

**Neither docs-only nor bias.** The kb's statements are what both binaries
do:
- an accepted event has generator weight 1, except an overweight at `r > 1`
  (`unweighting.md` says both);
- one factor per part (`multi-process-normalisation.md`).

The reported per-member units come from no code path in v0.2.2 or main that
this session found or ran. The pairing they show (`d g` with `d~ g`) is not
vibegraph's grouping. The manager should ask the user for the file's
`<header>`: a vibegraph `Buffer` file declares itself there, with the
`<generator>` element, `sample estimate before normalisation …` and the
`part k: …, normalised to …` lines. Also ask for the exact command and
`--version`, and for how the units were read. Three ways a file shows
per-member weights outside `generate`: post-processing, a merge of several
files, or another generator's output.

No fix is proposed. The kb pages need no correction. If the manager wants the
point made explicit, this is the sentence I would add (not applied).

- `events/multi-process-normalisation.md`, after rule 1:

  > Within a part the scale does not depend on the event's flavour member:
  > members of one flavour group, and of different groups, share it. A
  > member's share of the part is carried by how often it is drawn — the
  > member draw is `∝ S_i L_i r_i |M_g|²`, the member's own term of the
  > point's value, MLM's per-member reweighting `r_i` included — never by
  > its weight.

- `events/unweighting.md`, under "The weight":

  > The discrete labels drawn after acceptance (group, flavour member, beam
  > ordering, helicity, colour flow) are drawn in proportion to their terms
  > of the accepted point's value and carry no weight of their own, so they
  > never change an event's generator weight.

## What each test cannot detect

- **The reproduction** (modal `XWGTUP` per member). It sees any factor that
  depends on the member and enters `XWGTUP`. It cannot see a member that is
  drawn wrongly at uniform weight.
  - It covers this card shape, these two PDF sets, `-j 4` and `-j 8`, and one
  seed.
  - A factor that appears only under another strategy or event norm is
    outside it, and so is one that needs a different card (decays,
    reweighting).
- **The share test against member integrations.** It sees a member drawn at
  the wrong rate, at 1–8% relative per member at 30000 events. It cannot
  see:
  - a shape distortion inside a member that keeps the member's total, such as
    the wrong `(x₁, x₂)` or scale inside the member;
  - a common-mode error in the definition of a member's integrand. The
    single-member integration and the mixed run share the integrand code, the
    MLM reweighting and the scales, so an error in `rewgt` itself is
    invisible here. That oracle is the MadEvent `<mgrwt>` comparison, not
    this.
  - The beam orderings are pooled, so a q q̄ / q̄ q asymmetry is not tested.
    It is consistent by eye: 4405/4331 d d~ / d~ d on v0.2.2, 4304/4353 on
    main.
- **The closure table.** It is a self-consistency check of one code's two
  integrations, and is blind to anything both share.

## Long-run requests

- **The `c g` tension.** Branch `sprintl-xwgtup`, head as committed.
  - Command: `vibegraph generate $S/main/grid.bin.zst --seed <s> --nevents 300000 -j 4`, on
    the grid above, for two seeds, then `bias.py` as above. It is over the
    brief's one-run budget, which is why it was not run.
  - Cost: main's 30000 events took 14.5 s wall (20 s CPU), so about 2.5 min
    CPU per seed.
  - Expected answer: the `g u` group's `c/(u + c)` to ±0.003. It is 0.2128 if
    there is no bias. If two seeds both read 0.196 ± 0.003, that is a
    resolved −8% bias on the charm member of the `g u` group. Its first
    suspect would be the charm entry of `member_luminosity_rows` or of
    `member_rewgt` against the single-member integrand.

## Gates

No source change. The hermetic layer was run, as the brief asks:
`/tmp/claude-0/lp/verify-hidden.sh /home/user/wt/sprintl-xwgtup lx1` (under
the lock):

```text
head 54c31fe3
FMT_OK
CLIPPY_RC=0
CLIPPY_EV_RC=0
TEST_RC=0
lines=79 passed=2368 failed=0
```

`pixi run kb-lint`: `1412 concepts, 480 backlog items; 0 errors, 7 warnings`
(all seven on existing backlog items).

## Found

- **The user's per-member weights are unexplained.** Neither binary writes
  them on the reported card shape. Closing it needs the user's file
  `<header>`, `--version` and their analysis code (see the verdict).
- **The `c g` member share is 2–3σ low** in two independent 30000-event
  samples (−1.91 main, −2.27 v0.2.2). The long-run request above resolves it.
- **The q q̄ g groups' member integrations are 0.5–0.7% below the mixed
  run's per-group σ** (−1.6σ and −2.1σ on per-configuration errors taken as
  independent). Closing it needs the same comparison with the group
  integrated alone, `generate u u~ > e+ e- g` / `add process u~ u > e+ e- g`
  / `add process c c~ …` / `add process c~ c …`, against the mixed run's
  group σ. If the two disagree past their errors, there is a part-dependent
  difference in the integrand (the MLM `xqcut` / clustering on a mixed card
  against a single-multiplicity card).
- **The integration does not report σ per flavour member.** `inspect` gives
  channels and (group, configuration). A per-member σ (the share split
  already carries the group terms; adding the members is the same
  accumulation) would turn this session's sixteen extra integrations into a
  read-off. It would also give `generate` a per-member sample-against-
  integration check as cheap as the per-part one.

## Brief corrections

- The premise that v0.2.2 writes a unit weight per member does not hold on
  this card shape (above).
- The brief expects d d~ and s s~ to share one weight, and they do. But all
  members of a part share it, not only the members of one group. The
  normalisation is per part.
- The brief's example of a shared 1j weight (`d g` with `d~ g`) spans two
  vibegraph flavour groups.
- Deviation: beyond the one 30000-event run per binary, two 1000-event
  v0.2.2 runs (under 10 s CPU each) were made, to match the user's `-j 8`
  and card values. One extra integration was made for the user's card, and
  sixteen single-member integrations to obtain the per-member σ that the
  integration does not report.

## Addendum: the two 300k samples (manager-run)

The long-run request above, run by the manager on the same `$S/main` grid. Each seed was generated with:

```
vibegraph-main --no-network generate main/grid.bin.zst --seed <seed> --nevents 300000 -j 4
```

Both runs exited with rc 0. Each sample was then scored with `bias.py`:

| seed | 0j Σpull² (3 dof) | 1j Σpull² (11 dof) | `c g` share (integrated 0.04591) | pull |
|---|---|---|---|---|
| 101 | 2.00 | 24.43 | 0.04411 ± 0.00071 | −2.54 |
| 202 | 6.63 | 9.20 | 0.04528 ± 0.00072 | −0.88 |

- **The two seeds agree with each other.** They differ by 1.2σ, so the deficit does not migrate between seeds.
- **The 8% charm bias is excluded.** The `c g` share was 0.196 against 0.213 integrated in the 30k samples, which would have been an 8% bias. At 300k it is not there.
- **About 3% remains, unresolved.** All four samples combined give `c g` = 0.0445 ± 0.0005, about −2.8σ or −3% of the member.
  - That is within the reach of the look-elsewhere effect over 16 members.
  - The reference has its own systematic: the 1j closure is −1.2σ.
- **Scale of any effect.** A 3% shift on this member is 0.13% of the 1j part's composition. It moves no σ, because each part's scale is fixed.
- **Not a blocker.** Settling the residual needs ≥5 seeds, against shares read from the mixed run itself, which is Found 4.

**Found 1 resolved (user's rerun).** The user's card used `smeftsim_topu3l_mwscheme_ufo`, in which the light quarks are massive. `split_by_external_masses` therefore splits each multiplicity by incoming flavour mass: the header shows 12 normalisation parts against 2 LHE processes. Buffer then normalises each part to its own integration, so the per-flavour units are as designed. The planner recorded this on `uniform-unit-weight-cms-samples`.
