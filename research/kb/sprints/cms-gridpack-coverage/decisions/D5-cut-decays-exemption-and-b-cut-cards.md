---
type: Design Decision
title: "Sprint H: register the cut_decays exemption as a deviation; integrate the b-cut cards MadGraph refuses"
description: "D5 (revised): MadEvent's cut_decays exemption with identical particles across a decay is a registered deviation, since reproducing it needs the single-pairing |M|² of compat site (d). D6: keep integrating the b-cut cards MadGraph refuses."
decided: 2026-11-06
decided_by: human:nsmith-
status: stable
tags: [cuts, decay-chains, flavour-groups, madgraph-parity, sprint-h]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-06}
verified: [{by: "human:nsmith-", at: 2026-11-06}]
sources:
  - {id: hk1, resource: "../sessions/H-K1-report.md", title: "H-K1 report (on branch sprinth-group-cuts)"}
---
The user's answers after H-K1.

- **D5. `cut_decays = F` with identical particles across a decay.**
  - The rule being reproduced: MadEvent exempts the decay products of the first
    configuration from the cuts (`check_decay`, `setcuts.f:193`).
  - What vibegraph does today: it keeps every pairing (the registered deviation of
    [identical particles across decays](../../../process/identical-particles-across-decays.md))
    and cuts all four quarks.
  - The size: −6.8% on `g g > z j j, z > j j`, 5.348 pb against 7.211 pb on the
    same-flavour subprocess.
  - First decided (2026-11-06): reproduce MadEvent's exemption as a
    `--madgraph-compat` site (g).
  - **Revised the same day, after H-C2.** On vibegraph's every-pairing |M|²,
    exempting MadGraph's configuration-1 legs leaves a massless core quark
    uncut in the other pairings, so σ diverges logarithmically (the integrand
    goes as 1/pT², measured; [H-C2](../sessions/H-C2-report.md), on branch
    `sprinth-cut-decays`). Matching MadEvent needs MadGraph's single-pairing
    |M|² as well, which is compat site (d).
  - **Decided:** keep vibegraph's IR-safe rule (cut every leg that is not a
    decay product in every pairing) and register it as a deviation with its
    measured size. σ rows on such cards are informational. Sites (d) and (g)
    are filed as one item in
    [madgraph-compat-sites-unconditional](../../../backlog/feature/madgraph-compat-sites-unconditional.md).
    A guard refuses any cut rule that would exempt a massless coloured core
    leg.
- **D6. Cards whose b cuts differ from the jet cuts at `maxjetflavor 4`.**
  - Why MadGraph refuses them: one grouped P directory shares one cut setup, so
    `setcuts.f` stops ("Grouping of subprocesses not consistent with setcuts.f…
    turn grouping of subprocesses off", `setcuts.f:740–810`).
  - What vibegraph does: it gives each cut class its own group (H-K1). That is
    what MadGraph computes with `group_subprocesses False`.
  - Decided: keep integrating these cards, and document MadGraph's refusal.
