---
type: Design Decision
title: Keep every pairing of identical particles across decays
description: "Unlike MadGraph's single pairing with identical_decay_chain_factor, all pairings and their interference are kept; expected σ offset about +0.2%."
status: stable
tags: [decay-chains, identical-particles, madgraph-deviation, sigma]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}, {by: "human:nsmith-", at: 2026-10-09}]
decided: 2026-09-26
decided_by: human:nsmith-
sources:
  - {id: n38-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1144-L1201", title: "Note 38 §5, decision of 2026-09-26 and the MadGraph-only study"}
  - {id: n38-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L599-L688", title: "Note 38 §4 D2, MadGraph census of identical decays"}
  - {id: n38-d3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L689-L801", title: "Note 38 §4 D3, σ against MadEvent and the window union"}
  - {id: pr18-review, resource: "https://github.com/nsmith-/vibegraph/pull/18#discussion_r4233116250", title: "User review on PR #18: MadGraph's single pairing is an approximation"}
  - {id: mg-identical-factor, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/helas_objects.py#L4581", title: "MadGraph helas_objects.py identical_decay_chain_factor"}
measured:
  - {pr: 12, landed_in: 1539abc, command: "cli_decay_chain.rs, ten seeds at --target-rel 2e-3, against MadEvent 3.7.1 (decay_chain_sigma_reference.json)"}
---

**Decision (user, 2026-09-26).** When a decay-chain card produces identical final-state
particles from different decays (`e+ e- > z z, z > e+ e-`), vibegraph keeps **every
pairing** of those particles with the resonances, with the interference between pairings
and the final state's own identical-particle factor. MadGraph does not. MadGraph's
single pairing is an **approximation** that changes a weight: the narrow-width treatment
of decay chains with the interference between pairings dropped. Under the
[defect policy](../validation/madgraph-defect-policy.md) it is a **registered deviation**
(outcome 3, listed in the [defect register](../validation/madgraph-defects.md)): this
crate keeps the more complete calculation, so the σ of such a card is not expected to
equal MadGraph's. [^n38-decisions] [^pr18-review] The parity goal it departs from is in
[release scope](../pipeline/release-scope.md).

## What each side computes

- **MadGraph** keeps each decay's products on the legs it put them on, one pairing, and
  divides by `identical_decay_chain_factor` (`helas_objects.py:4581`), which is 2 for
  `z z, z > e+ e-` [^mg-identical-factor]. The interference between pairings is dropped.
- **Here**, the stitched diagrams are closed under every permutation of identical
  final-state particles between blocks ([decay chains](decay-chains.md)), exactly as the
  diagrams of the undecayed final state are. For `z z, z > e+ e-` that is four diagrams
  where MadGraph has two, and the usual 2!·2! = 4 for the identical final state
  ([identical-particle factor](../phase-space/identical-particle-factor.md)). A stitched
  set is then exactly the full final state's diagrams in which each chain resonance is an
  s-channel line with its stated products.

The forced Breit–Wigner windows follow: a point passes when every forced line of *some*
pairing is inside its window. The union is symmetric under the permutation and independent
of the sampler, which a per-configuration reading like MadEvent's would not be.
[^n38-d3] Event records pick the pairing by restricting the configuration draw to those
whose forced lines are inside their windows ([resonance records](../events/resonance-records.md)).

## Expected size of the difference

A MadGraph-only study (`p p > z z, z > e+ e-` at 13 TeV, fixed μ = M_Z, `bwcutoff` 15)
showed that MadGraph's normalisation of its single pairing is right (the factor 2 is
exact) and that the whole difference is the dropped interference and the off-window
tail [^n38-decisions]:

- the identical chain is exactly half the e-μ chain: A/C = 0.499990 ± 0.000010;
- against the Z-only four-electron process restricted to "either pairing on its window",
  MadGraph's chain is 0.22% low; per event, +0.23% of that is the dropped pairing
  interference (⟨2 Re M_a M_b*⟩/|M|² = −0.00227 ± 0.00012) and −0.45% the other
  assignment's off-window tail;
- the double ratio (A/B)/(C/D) = 0.9975 ± 0.0015 matches the per-event prediction
  0.9978 ± 0.0001;
- the interference is O(1) only where both pairings sit within about Γ of the pole (about
  1e-3 of events), and about 0.08·Γ/M overall.

So the permuted treatment should read about **+0.2%** above MadGraph on such cards. The
measurement on `e+ e- > z z, z > e+ e-` (500 GeV) is **+2.6e-3 ± 0.8e-3**: MadEvent
4.7194e-4 ± 3.3e-7 pb, here 4.7318e-4 ± 2.0e-7 pb (ten seeds here). [^n38-d3]

## How it is gated

- σ is **gated** only on cards without identical particles across decays. Cards with that
  overlap are compared with MadGraph **informationally**, the expected difference being the
  pairing interference.
- The structural census matches MadGraph's per-matrix-element diagram counts by counting
  only the stitched diagrams whose forced lines lead to MadGraph's blocks
  ([decay chains](decay-chains.md), oracle 3). [^n38-d2]
- On MadEvent's side, `NSYM = 1` and one pairing appear in its event records for identical
  decays (the two Z over legs (3,4) and (5,6)). Naming a parent Z per electron is
  bookkeeping that nothing physical could observe, not a different sample.
- With the planned `--madgraph-compat` flag on, MadGraph's single pairing and factor 2
  would be reproduced
  ([madgraph-compat-sites-unconditional](../backlog/feature/madgraph-compat-sites-unconditional.md)).

The study's side findings bear on any comparison at this precision: MadEvent's quoted σ
errors are unreliable at 0.1% (χ²/dof across seeds of 3–14, even for full-process runs),
and a few `dummy_cuts` window violations appeared in MadEvent events (27 in 500k,
unexplained; see the
[backlog item](../backlog/validation/madevent-dummy-cuts-window-leakage.md)).

[^n38-decisions]: Note 38 §5, 2026-09-26 (user): the decision and the MadGraph-only study (scripts kept outside the repository).
[^n38-d2]: Note 38 §4, decay-chain enumeration by stitching: MadGraph does not permute between decays; the census comparison.
[^n38-d3]: Note 38 §4, decay-chain phase space and σ: the window union over pairings and the informational σ row.
[^pr18-review]: User review on PR #18, 2026-10-09: the single pairing is at least an approximation, handled as a registered deviation.
[^mg-identical-factor]: `madgraph/core/helas_objects.py` `identical_decay_chain_factor`, L4581.
