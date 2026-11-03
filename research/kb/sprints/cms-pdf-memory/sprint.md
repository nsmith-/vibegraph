---
type: Sprint
title: "CMS's 880-member systematics request under 2 GB"
description: "generate with CMS's runcmsgrid.sh systematics request (43 sets, 880 members) fits a 2 GB CMS production slot, measured, with the CMS-arguments weights gate still exact."
status: draft
active: true
tags: [systematics, pdf, memory, cms, lhapdf, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
## Goal and exit criteria

Part of sprint F (CMS production readiness). Done when the item's
`closes_when` holds: `generate` on `pp_to_ll` at lhaid 325300 with CMS's
`runcmsgrid.sh` request (880 members, 926 weights) peaks below 2 GB resident,
measured, and `cms_arguments_reproduce_madgraphs_weights`
(`validate_systematics`) is still exact. Every clause is pinned by a test or a
recorded measurement.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `pdf-members-exceed-cms-memory-slot`

## What is known before the sprint (manager, 2026-11-03)

- Measured in sprint E's S-V1: 3.76 GB peak at `-j 1` and `-j 4`, against
  0.44 GB for the card's own 147-weight request. About 4.2 MB per loaded
  member, against about 0.56 MB of raw knots.
- A member is a `LogBicubic` (`vibegraph-lib/src/pdf/interp.rs`). Per band it
  keeps `coeffs`, four `f64` per x interval × Q² knot × flavour, and `xf`, the
  raw knots, which only the two-knot bilinear fallback reads. That is about
  five `f64` per knot, about 2.8 MB per member; the rest of the 4.2 MB is not
  yet attributed (parse transients, allocator behaviour, other copies).
- Projections, unmeasured: dropping `xf` alone leaves about 2.4 GB; `f32`
  coefficients about 1.9 GB (no margin, and lossy); the raw knots alone, with
  the x-direction cubic built on use as LHAPDF does, about 1 GB.

## Design choices for the user

Asked before any code session is dispatched; answers recorded under
`decisions/` and in the log.

- **D1. Which technique.** Recommended: lossless first, storing each member
  as its knots and building the cubic on use, in the same arithmetic as
  today's precomputation, so every interpolated value is bit-identical. Set
  at a time (buffered per-event records, one pass per set) is a design change
  of `generate` and is built only on the user's word. `f32` storage only if
  shown not to move a printed weight, or on the user's word.
- **D2. CPU budget.** Building the cubic on use costs arithmetic on every
  evaluation. On `pp_to_ll` the PDF weights already dominate the 880-member
  run (54.6 s against 6.1 s), so the slowdown shows there in full.
- **D3. Target.** Below 2 GB closes the item; the headroom to aim for.
- **D4. Regression guard.** CI has no CMS sets. A CI test on bytes per loaded
  member catches a regression without them; the RSS figure itself is a
  recorded measurement.

## How each claim is measured

- **Where the memory goes** (M-P0): peak and steady RSS over the load and the
  event loop; heap attributed per structure (bytes per member by field,
  transient parse buffers, allocator arenas at `-j 1` and `-j 4`).
- **Exactness**: the new interpolator against the current one, bit for bit,
  on every flavour of every member of real sets (multi-band and two-knot
  bands included) at dense random and knot-edge points. That is finer than
  the weights gate, which sees eight printed digits. Then the CMS-arguments
  gate on all four `run_01_cms` runs.
- **Memory**: peak RSS of `generate` on the item's command, `-j 1` and
  `-j 4`, five repeats, child `getrusage`.
- **Time**: wall and user time on the same command, and the per-member
  evaluation cost, against the current code on the same machine.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| M-P0 | performance-dev (Opus), measurement only | – | report | – |
| M-P1 | performance-dev (Opus) | M-P0, D1–D4 | bit-identical interpolation, CMS gate, RSS, time | – |
| M-V1 | validation-dev (Opus) | M-P1 | independent re-measurement, CMS gate, grid oracles, controls | the item |
| M-HY1 | hygiene-dev (Sonnet) | M-V1 | hermetic, fmt, clippy | – |
