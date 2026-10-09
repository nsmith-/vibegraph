---
type: Feasibility Study
title: Deriving diagram phases from U(1) charge flow
description: "Research-only test of whether charge flow fixes every pinned phase convention: not actionable (n−k=0); the φ/H split is invisible to every oracle; phase = charge flow × i-counting/Lorentz."
status: draft
tags: [phase-conventions, fermion-flow, sign-convention, research, negative-result]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n29-chainf, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L227-L270", title: "Note 29 chain F: the hypothesis and its sidecar terms"}
  - {id: n29-design, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L353-L682", title: "Note 29 F.1–F.6: inventory, pre-registered bar, hostile cases, method, vacuity modes, brief errors"}
  - {id: n29-findings, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L693-L1152", title: "Note 29 F.7–F.13: P0, the rule, hostile cases, verdict, negative result, errors"}
  - {id: n29-notdecided, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L1208-L1224", title: "Note 29 F.15: what the study provably did not decide"}
---

# Deriving diagram phases from U(1) charge flow

**Question.** The amplitude's phase conventions — every rooting sign lifted into
`fermi_sign`, the colour `3`/`3̄` transpose, the VVVV phase, the fitted constant
`G = ±i` of the per-diagram oracle — were each found and fixed locally. Does one
principle produce them all? The hypothesis: following the diagram's U(1) charge
flow (fermion-number arrows treated as a flow structure, the way SU(3) is treated
as colour flow) determines the diagram's phase.[^n29-chainf]

**Answer (2026-08-03, against the code of that date).** No. The phase
*factorizes* as (charge flow) × (i-counting / Lorentz structure). Charge flow
reproduces the fermionic arms and the colour transpose; three arms have no
fermion arrow to read. Verdict against the pre-registered bar: **interesting but
not actionable**, `n − k = 0`.[^n29-findings]

The study was research only: no production code, no refactor, derivations on
paper against the banked per-diagram × per-helicity × per-flow dumps.

## The bar, fixed before any derivation

A *charge-flow rule* assigns a phase to each (diagram, rooting) using only the
fermion arrows, the per-leg crossed bits, the vertex/propagator incidence of each
fermion line and the rooting's output leg — no per-Lorentz-structure case, no
per-process table, no constant fitted to a MadGraph dump. It is written in full
before its first comparison; an amended rule is a new rule. With `k` binary
choices the author picked and `n` pinned binary conventions it reproduces *and a
dump checks*, only `n − k` is evidence.[^n29-design]

- *Very promising*: reproduces every pinned convention with `n − k ≥ 5`.
- *Interesting but not actionable*: reproduces at least the fermionic arms and
  the colour transpose but needs non-charge-flow input for the rest.
- *Refuted*: a witness pair — two configurations identical in every charge-flow
  input with different pinned signs — against the fermionic arms.

A pre-registered falsifier P0 (the strong reading predicts `fermi_sign ≡ +1` on a
process with no fermion line) came first, so a factorized result could not later
be reported as the strong claim. The design is the model for
[validation/pre-registered-verdicts](../validation/pre-registered-verdicts.md).

## The rule

For a line `ℓ` with `V(ℓ)` vertices, `P(ℓ) = V(ℓ) − 1` propagators and crossing
bit `x(ℓ)`:

> `Φ(d,r) = ε(d) · Π_ℓ [ (−1)^{x(ℓ)} · Π_{v∈ℓ} ω(v,ℓ) ] · κ(d,r)`

`ε` is the Wick parity of the line pairing; `ω = −1` at every vertex of a line with
an initial-state end (read against its arrow, `C γ^{μT} C⁻¹ = −γ^μ`) and `+1` on a
crossed line; `κ` removes the one `ω` the runtime already applies at each line's
sink. It collapses to `(−1)^P` per uncrossed line and `−1` per crossed line — the
current [fermion-line-sign](fermion-line-sign.md) exactly.

Two structural findings came before any dump:

- The per-propagator `−1` is not a repackaged propagator `−i`: the propagator's
  `−i` is applied in the kernel and is present separately.
- "One −1 per propagator" and "one −1 per vertex read backwards, less the sink" are
  extensionally identical on every tree line (`P = V − 1`). No tree-level dump can
  ever choose between them.

## Results (as of 2026-08-03)

| convention | rule covers it | checked by a dump |
|---|---|---|
| Wick parity | yes | yes: Bhabha's MadGraph `c = −diagram.sign`; `u d > e+ e- u d` NC/CC split |
| fermion-line sign, both arms | yes | yes: `g g > t t~` uniform over 0/1/1 propagators; `u d` 11 vs 24 split |
| reversed-bilinear parity | yes | no — `revC·revL ≡ +1` at the reference rooting, unobservable in principle at the `fermi_sign` level |
| crossed-pair −1 | yes | no — no varying instance |
| scalar-sink bilinear −1 | no arrow | no — no varying instance (then) |
| pure-metric / contact −1 | no arrow | yes (then: `g g > g g` contact vs exchange) |
| colourless VVV source | no arrow | yes (`e+ e- > W+ W-`, pattern `−,−,+`) |
| standalone projector on crossed line | yes | yes (`e+ e- > ta+ ta- H`, 4:1) |
| colour `3`/`3̄` transpose | yes | value only — under the SM UFO's uniform antifermion-first FFV slot order, "swap unconditionally" and "index by the arrow-out leg" are the same function |

`n = 4`, `k = 4` (global sign; crossed-line sign; placement of `κ`; colour
indexing direction): `n − k = 0`, and `4` under the most generous admissible
count. Which arms the current code carries, and their present pin status, is in
[convention-sign-inventory](convention-sign-inventory.md); the study's arm table
and line numbers describe the code of its date.

**Witness pairs.** W1, against the strong reading: `g g > g g`'s contact diagram
and an exchange diagram had empty, identical charge-flow inputs and different
contact/VVV signs (their product agreed). The current code no longer gives gluon
vertices any net sign (gluon VVV and gluon contacts are exempt in
`yang_mills_vvv_sign` and `vector_contact_sign`), so W1 no longer holds as
written; the arrow-free arms that remain — colourless VVV sources, the colourless
contact at the anchor, pure-metric VVS, scalar sinks — still carry the
factorization. That last statement is read from the code, not re-measured. W2,
against the `G`-sign clause: `e+ e- > mu+ mu-` and `u u~ > mu+ mu-` have identical
fermion-line structure and `G = −i` vs `+i`; MadGraph's coefficients compensate
(`G·c₀ = +i` for both). The sign of `G` tracks MadGraph's own colour-coefficient
sign, so no rule of this form can win that clause.

**By-product.** `|G| = 1, Re G = 0` follows from i-counting (see
[global-phase-i-counting](global-phase-i-counting.md)), and every
per-configuration phase measured exactly `k/G ∈ {±1}` (113 configurations, worst
residual 1.19e-13): the fitted quantities are bits, mostly MadGraph's `c_j`. Not
yet asserted: [config-amp-phase-and-sign-unpinned](../backlog/validation/config-amp-phase-and-sign-unpinned.md).

## What the study cannot decide

These limits are intrinsic to the reference data, not to the effort spent.[^n29-notdecided]

- **The `φ`/`H` split.** Oracles pin `A_d = φ_d·H_d`; any reassignment of a sign
  between the convention lift and the honest evaluation is invisible. In Bhabha
  and `u u~ > u u~` the Wick and line signs cancel and the relative sign sits in
  `H`. This is why banking more processes cannot raise `n`.
- Any phase common to all diagrams and flows; whose convention the `G` sign is.
- Majorana lines (the SM has none); the two 2→6 tables that bank flows only; loop
  level.
- `g g > g g`'s trace-reversal-degenerate flow pairs (`J₁=J₆, J₂=J₄, J₃=J₅`) —
  that class belongs to `color_flow_tags_oracle`, not to amplitude dumps.

## Corrections to carry

- The rooting sweep the brief cited as 165 re-rootings was 133 (`165264` is an
  unrelated node count).
- The VVVV phase was not an imaginary `−i`: commit `b62ac17` replaced it with a
  real `−1` and deleted `Op::MetricNegI`. A `−i` on the propagator-free contact
  diagram is an uncancelled 90° rotation, which i-counting forbids.
- `S` was graded as nine members `{1,2,3,4a,4b,4c,4d,5,7}`; the design's table
  had twelve rows.
- The `u d > e+ e- u d` 24/11 split the design expected is the
  neutral-/charged-current (Wick) split; the line sign's own partition of the same
  35 diagrams is 11 vs 24, a different partition of the same shape. A first
  by-hand count of it was wrong — machine-check census claims.
- P0 was sited on `channel_counts`, a code-internal quantity, so it could refute
  only a bookkeeping claim. Site a falsifier on a dump-visible quantity.

Oracle mechanics: [validation/amplitude-oracle](../validation/amplitude-oracle.md).

[^n29-chainf]: Note 29, chain F sidecar.
[^n29-design]: Note 29 §F.1–F.6.
[^n29-findings]: Note 29 §F.7–F.13.
[^n29-notdecided]: Note 29 §F.15.
