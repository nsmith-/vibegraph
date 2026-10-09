---
type: Validation Methodology
title: The finite-field evaluator as an exact oracle
description: "Running the production evaluator over Z_p with an f64 shadow gives an exact oracle for zeros, per-pair identities and gauge cancellations."
status: draft
tags: [finite-field, oracle, evaluator, reconstruction, exact-arithmetic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n41-9, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L452-L462", title: "Note 41 (completeness) §9 — the per-pair trace form by reconstruction"}
  - {id: n41-91, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L463-L518", title: "Note 41 (completeness) §9.1 — the box"}
  - {id: n41-10, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L613-L621", title: "Note 41 (completeness) §10 — the full |M|²"}
  - {id: n41-101, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-completeness-trace-msq-feasibility.md#L622-L674", title: "Note 41 (completeness) §10.1 — method and oracles"}
  - {id: code-ff, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/finite_field_msq.rs", title: "vibegraph-lib/tests/finite_field_msq.rs"}
---

# The finite-field evaluator as an exact oracle

The evaluator is generic over its scalar field, so it runs **unchanged** over a
test-only scalar `Fz`: an element of `Z_p`, with `i` adjoined through
`num_complex`, carried together with an `f64` shadow of the same computation.
Everything lives in `vibegraph-lib/tests/finite_field_msq.rs`, a hermetic
integration test (dev-dependencies `num-modular`, `num-prime`, `num-bigint`).
It was built to measure how large a trace-form `|M|²` would be (see
[the trace-form feasibility study](../performance/trace-form-msq-feasibility.md)),
but the box itself is a validation instrument: arithmetic is exact, so a zero is
a zero, an identity holds or it does not, and a gauge cancellation is seen
exactly rather than at rounding level.

## How the box works

- **The field.** For `p ≡ 3 (mod 4)`, `Z_p[i]` is the field `F_{p²}`, whose
  Frobenius is complex conjugation, so `A A*` means what it means over `C`.
- **The shadow decides branches.** Wavefunction routines branch on `min`, `max`
  and `== 0`; the `f64` shadow decides every comparison, so the field computation
  follows the path the real one takes.
- **Square roots.** External wavefunctions take roots, but the helicity sum sees
  only each leg's completeness relation (`Σ_h u ū = p̸ + m`, …), which is
  polynomial in the momentum. Any root choice keeping those relations gives the
  reduction mod `p` of the exact rational answer. The root taken is
  `x^((p+1)/4)`, the root that is itself a square, which makes it multiplicative;
  with `p ≡ 7 (mod 8)` the separately taken roots in `weyl_ixxxxx` stay
  consistent. That is a convention claim, pinned by
  `fz_wavefunctions_satisfy_completeness`: `Σ u ū = p̸ + m`, `Σ v v̄ = p̸ − m`,
  the massive vector sum and the massless one with HELAS's `n = (p⁰, −p⃗)`, mod
  `p`. (Its first failure, on the massive antiparticle, was the test's own fault —
  the same identity in `f64` showed it.)
- **Roots that do not exist.** About half of random draws have no root in `Z_p`
  per root taken. Rather than reject points, the test builds legs whose roots
  are rational squares by construction (`square_root_leg`: rational half-angle
  directions, `E = 2w²` for a massless leg, `E ± |p⃗| = a², m²/a²` for a massive
  one). `square_root_legs_never_miss_a_root` pins that no root goes missing and
  completeness survives, over two primes. Legs along the z axis are excluded: the
  root of `|p⃗|²` can come out `−|p⃗|`, making `|p⃗| + p_z = 0` on the branch the
  shadow takes. Rejection sampling instead left one usable node in ~1000 on
  `ee_to_mumu_tata_qcd0`[^n41-101].
- **Denominators** are read off the diagram set, `Π (q² − M² + iMΓ)` with no
  width on spacelike lines, as the evaluator lowers them. A wrong denominator
  makes the numerator non-polynomial and the fit fails, so the fit checks them.
- **External massless vectors** break per-pair Lorentz invariance (the HELAS
  polarisation sum has a frame vector `n` whose gauge terms cancel only in the sum
  over diagrams). Per-pair fits with such legs are sampled in the partonic CM
  frame, with one more known denominator `(p·P)²` per vector leg. The full
  `|M|²` is gauge invariant and needs no frame restriction[^n41-91].

## What it can serve as an oracle for

| test | runs | what it pins |
|---|---|---|
| `fz_wavefunctions_satisfy_completeness` | default | the root convention keeps each leg's completeness relation |
| `square_root_legs_never_miss_a_root` | default | the constructed legs always have roots, massless and massive, spinors and vectors |
| `per_pair_numerators_lift_to_the_f64_evaluator` | default | per-pair numerators `N_ij = D_i D_j* Σ_hel A_i A_j*` fitted mod `p`, lifted to `Q(i)` by Chinese remaindering and rational reconstruction (8 primes on `ee_to_mumu`, 12 on `ee_to_mumua`, confirmed by one more), then evaluated in `f64` at fresh physical points against the evaluator's own per-pair sums: worst 7.5e-15 and 1.2e-15 of the largest term |
| `full_msq_matches_the_textbook_closed_form` | default | against physics rather than the evaluator: `u ū → e⁺e⁻ g` must satisfy `\|M\|² · s₃₄ s₁₅ s₂₅ / (s₁₃² + s₁₄² + s₂₃² + s₂₄²) = const` (the crossing of `e⁺e⁻ → q q̄ g`), exactly mod `p`, at 20 points — through `eval_m2`'s colour and helicity sums |
| `measure_trace_form`, `measure_full_msq` | `#[ignore]` | the reconstruction census itself (`cargo test --release -p vibegraph-lib --test finite_field_msq measure_trace_form -- --ignored --nocapture`) |

The full-`|M|²` reconstruction reads pole exponents as multiplicities in a
univariate Thiele reconstruction along random rational curves (≈1 400 nodes for
2 → 4), checks the numerator degree by the scaling `p → μ² p` (massless legs), and
fits the numerator densely, separating parity-even and -odd parts by mirroring
`p⃗ → −p⃗`. Curve artefacts must be avoided: invariants share energy factors, and
closing momentum conservation by a lightlike split leaks the closing pair's
invariant into the others, so each propagator is tested only on the part of its
polynomial shared with no other invariant, on a curve that does not close on
it[^n41-101]. Without that, the photon's `1/s²` in `ee_to_mumu` read as exponent 0.

The method is Peraro's functional reconstruction over finite fields; see
[the paper](https://arxiv.org/abs/1608.01902) and
[FiniteFlow](../references/papers/finiteflow.md).

## Blind spots

- **The box is the evaluator.** It pins the field arithmetic, the roots and the
  fit, not the amplitudes; the amplitude gates hold those against MadGraph. Only
  `full_msq_matches_the_textbook_closed_form` reaches past the evaluator.
- **A coefficient divisible by `p` reads as zero.** Every count must agree across
  two primes; on every measured row a second prime reproduced every count.
- **The lift is not run on the 2 → 4 rows.**
- **Frame-dependent counts.** Per-pair counts with external massless vectors are
  those of an axial-type gauge and overstate what a Feynman-gauge trace program
  would get.

[^n41-91]: Note 41 (completeness-trace) §9.1.
[^n41-101]: Note 41 (completeness-trace) §10.1.
