---
type: Sprint
title: "Scale, PDF and αs variation weights at generate"
description: "generate writes MadGraph systematics-module weights in-process, in MadGraph's exact format, from a typed request recorded at integrate; gated per event against MadGraph's systematics output on banked events."
status: draft
active: true
tags: [systematics, lhef, pdf, scales, alpha-s, cms, sprint-e]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
## Goal and exit criteria

Done when the item's `closes_when` holds, every clause pinned by a test or a
recorded measurement.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `systematics-variation-weights-not-written`

Part of sprint E (param card, systematics, peripheral spine).

## What the banked data already holds (manager, 2026-10-27)

- Twelve unmatched proton runs in refdata-11 carry `<mgrwt>` on every event and
  MadGraph's default systematics output, 145 weights: 45 scale combinations
  (μR × μF × dynamical scale −1, 1–4, varied together) and the 247000 error
  set. Among them are `pp_to_ll`, `pp_to_jj`, `pp_to_bb`, `pp_to_llj` and
  `pp_to_llj_dyn`. They give a per-event gate on MadGraph's events with no
  new banking.
- The MLM runs (`pp_to_ll_0j2j_mlm`, `pp_to_llj_mlm`, `pp_to_ttx_0j1j_mlm`)
  carry `<mgrwt>` with emission scales, but no weights.
- MadGraph's weight is `event weight × W(variation) / W(nominal)`, with `W`
  rebuilt from the `<mgrwt>` fields (`systematics.py`, `get_lo_wgt`). The
  dynamical-scale variations use `lhe_parser.py`'s Python definitions over the
  written event's status-1 particles, not `setscales.f`.

## Decisions (user, 2026-10-27 and 2026-10-28)

- **Q1.** Honour `--mur`, `--muf`, `--alps`, `--dyn` (−1, 1–4), `--together`,
  `--start_id` and `--pdf` (`errorset`, `central`, an id, a name, `name@N`,
  `id@0`), for unmatched and MLM events. Read both MadGraph entry points:
  `systematics_arguments` when the card sets it, else the `sys_*` fields.
  Refuse `--weight_format`, `--weight_info_format`, `--remove_wgts`,
  `--keep_wgts`, `--only_beam`, `--ion_scaling`, `--start_event`,
  `--stop_event`, `--lhapdf_config` and `systematics_program = syscalc`.
  `sys_matchscale` and `sys_scalecorrelation` are ignored, as MadGraph ignores
  them under the `systematics` program. The request is recorded at integrate
  (`RunCard::split`, `GenerateSettings`), and a refused option fails at
  integrate.
- **Q2.** `generate --systematics '<MadGraph arguments>'` overrides the
  recorded request (CMS passes its arguments on `runcmsgrid.sh`'s command line).
- **Q3.** Follow MadGraph's default: `use_syst = T` on a default proton card
  writes the default weights.
- **Q4.** Match MadGraph's output exactly: group names and `combine`
  attributes, tags, info text, ids and the per-event `<wgt>` layout.
- **Q5.** In-process at generate.
- **Q6.** Write `<mgrwt>` under `use_syst`, as MadGraph does.
- **Q7.** "αs variation" means PDF members and sets with varied αs, each
  evaluated with its own αs. `--alps` is the MLM emission scale.
- **Q8.** Typed fields: the request is parsed into a typed value at integrate
  and stored in `GenerateSettings`, with the verbatim strings kept for
  `<MGRunCard>`. This changes the artifact's shape: version 15, floor raised.
- **Q9.** Bank against the most common CMS PDF list.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| S-D1 | feature-dev (Opus), design only | – | kb-lint; user approval | – |
| S-F1 | feature-dev (Opus) | S-D1 approved | unmatched per-event gate on the 12 banked runs | – |
| S-F2 | feature-dev (Opus) | S-F1, M3 banking | MLM per-event gate | – |
| S-V1 | validation-dev (Opus) | S-F2 | CMS-arguments gate, controls | the item |
| S-HY1 | hygiene-dev (Sonnet) | S-V1 | hermetic, fmt, clippy | – |
