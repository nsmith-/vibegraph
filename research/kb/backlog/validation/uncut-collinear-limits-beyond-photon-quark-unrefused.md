---
type: Backlog Item
title: "Uncut collinear limits other than photon–quark are integrated silently"
description: "#109 refuses a photon on a massless quark with no draj; massless jets at ptj = 0 and drjj = 0 with no xqcut, a photon off a massless lepton at dral = 0, and partons above maxjetflavor are not."
area: validation
state: open
priority: high
closes_when: "Every massless splitting with a collinear or soft pole (q → q g, g → g g, ℓ → ℓ γ, and coloured partons outside the jet classes) that no pair cut, pT cut or xqcut regulates is refused with a typed error naming the legs, with a test per class, and every banked card is still accepted."
blocked_by: []
opened: 2026-11-08
tags: [cuts, ir-safety, guards, collinear, silent-sigma]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: hg2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-G2-report.md", title: "H-G2 report, Found 3"}
  - {id: hb1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-B1-report.md", title: "H-B1 report, Found 1: IR-divergent cards integrate silently"}
---
Sprint H's issue B was a σ "disagreement" on four cards whose integral is
divergent. Each side reported how far its own sampler reached into an uncut
`q → qγ` pole. #109 refuses that case. `CutError::UncutPhotonSplitting`
(`cuts.rs`, `photon_parton_vertices`) covers a photon and a massless coloured
parton at one vertex with no ΔR minimum between them.

The same failure (a finite, sampler-dependent σ) remains for the limits H-G2
listed and did not run:
- massless jets at `ptj = 0` and `drjj = 0` with no `xqcut` (`q → q g`
  final-state, `g → g g`);
- a photon off a massless charged lepton at `dral = 0` (`e+ e- > mu+ mu- a` in
  the default SM has massless muons);
- a coloured parton above `maxjetflavor` that no class pair cut reaches.

H-G2's proposed fix: extend `photon_parton_vertices` to every massless-pair
splitting, and refuse where no pair cut or `xqcut` regulates it. Priority high:
the failure is a silent wrong σ.

[ir-unsafe-exemption-guard-sees-coloured-legs-only](ir-unsafe-exemption-guard-sees-coloured-legs-only.md)
is the `cut_decays`-exemption side of the same question.
