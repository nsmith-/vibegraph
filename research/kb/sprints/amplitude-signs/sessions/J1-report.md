---
type: Session Report
title: "J1 report: jioxxx chirality and longitudinal-term oracle"
description: "test_eval_jioxxx gains a massive-fermion (q·J ≠ 0) regime with the real SM Z vertices, every sink structure in both line orientations, and an ALOHA cross-check. jioxxx is aligned with ALOHA's real-m² longitudinal denominator. 13 mutations recorded."
status: draft
tags: [helas, chirality, oracle, propagator, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
sources:
  - {id: cc76f58, resource: "https://github.com/nsmith-/vibegraph/commit/cc76f58", title: "pin jioxxx chirality and longitudinal term on massive fermions"}
---
The dev agent's report. Mutation driver and logs are in the session
scratchpad (`J1/mutate.py`, `J1/mutate2.py`, `J1-mutations.log`,
`J1-mutations-massive.log`, `J1-gate.log`).

## What was done

One commit, `cc76f58`, touching `vibegraph-lib/src/helas/eval/run.rs` and
`vibegraph-lib/src/helas/vertex.rs`. No production code changed.

- **`test_eval_jioxxx` runs over a case table.** Each case is checked against
  the photon and against the Z.
  - The massless regime (e⁺e⁻ → μ⁺μ⁻, √s = 1) uses FFV1, FFV2 and FFV4.
  - The massive regime (t t̄ → τ⁺τ⁻, √s = 500 GeV) uses the same three
    structures and the real SM Z vertices: tt̄Z is FFV2·GC_51 ⊕ FFV5·GC_58,
    τ⁺τ⁻Z is FFV2·GC_50 ⊕ FFV4·GC_59, and both have g_L ≠ g_R.
- **Every structure's amplitude sink is now checked.** The old test checked only
  FFV1. The sink is checked in both line orientations.
- **Massive single-structure FFV2/FFV4 currents are cross-checked against ALOHA.**
  The references are the transcriptions `ffv2_3`/`ffv4_3`, which share no
  bilinear code with `jioxxx` or the kernel.
- **A guard asserts which regime each case is in.** It checks
  `max |q·ε| / (|ε|√s)`:
  - below 1e-13 when the current is conserved (massless fermions, or g_L = g_R);
  - above 1e-2 otherwise.

  The measured values were 0 for every conserved case, and between 0.13 and 1.0
  for the non-conserved ones (FFV2/Z 0.999, FFV4/Z 0.995, SM Z/Z 0.9999).
- **`jioxxx` is aligned with ALOHA's real-m² longitudinal denominator.** The doc
  comment records how this differs from HELAS.

### The "different decomposition" of the chiral sinks

The old comment said the FFV2/FFV4 sinks "use a different HELAS gc
decomposition". I diagnosed this with per-helicity complex dumps. The
hand-built sink puts the bra in the UFO slot whose spinor index the projector
acts on. The evaluator then reads the line reversed, through
C γ^{μT} C⁻¹ = −γ^μ, which exchanges P_L and P_R.

The dump for FFV2/a, e⁺e⁻→μ⁺μ⁻:

| helicity | got (production) | `−i·iovxxx(gc)` | `−i·iovxxx([gR, gL])` | ket-first sink |
|---|---|---|---|---|
| ↓↑↓↑ | 0 | −1.8019e-1 i | 0 | +1.8019e-1 i |
| ↓↑↑↓ | +9.4836e-3 i | 0 | +9.4836e-3 i | 0 |

The test now pins both orientations at 1e-12:
- with the ket in the projector slot (`reversed = false`), the amplitude is
  `+i·iovxxx(gc)`;
- reversed, it is `−i·iovxxx([g_R, g_L])`.

The swap holds in the massive regime too. The relative global sign between the
two orientations is a property of the hand-built diagram. I pin the value I
measured, but I have no derivation of it, and |M|² cannot see it.

## Gates

At `cc76f58`'s tree. The last command ran after a comment-only edit.

```
$ cargo fmt --all --check
FMT-OK
$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY-EXIT=0
$ cargo test --workspace
TEST-EXIT=0     (38 suites: 1395 passed, 0 failed, 16 ignored)
$ cargo test -p vibegraph-lib --lib test_eval_jioxxx     (after the comment edit)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 993 filtered out
```

All cargo invocations ran with
`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_{DEV,TEST}_DEBUG=0`.

**Observed deviations.** These are the largest over all cases and helicities, as
a fraction of the per-case scale. The tolerance is 1e-12 for all three.

| comparison | max deviation | headroom |
|---|---|---|
| current vs `jioxxx` | 3.2e-17 | about 3×10⁴ |
| `−i·jioxxx` vs ALOHA `FFVk_3` | 2.7e-16 | about 3.7×10³ |
| amplitude vs `iovxxx∘jioxxx` | 2.1e-16 | about 4.9×10³ |

No banked `pixi run validate` run was needed: the session changed no
production kernel and no coverage list.

## Falsifier mutations

Each mutation was applied, followed by
`cargo test -p vibegraph-lib --lib test_eval_jioxxx`, and then reverted.

- **M0, the control (no mutation):** passes.
- **M-series:** the full new test.
- **Q-series:** the new test with the massless cases removed, to show that the
  massive regime has teeth of its own.
- **P-series:** the test as it stood at the base commit.

| id | mutation | result (first failing assertion) |
|---|---|---|
| M1 | `jioxxx`: `cl·gL − cr·gR` | `current vs jioxxx (FFV1/a, e+e-→mu+mu-, hel ↑↓): \|diff\|=8.710e-1, scale=4.355e-1` |
| Q1 | same, massive cases only | `q·ε = 6.778e-1 (FFV1/a, t~t→ta+ta-)`: the conservation guard fires, because the flip makes FFV1 axial |
| M2 | `iovxxx`: `gL·cl − gR·cr` | `amplitude … (FFV1/a, e+e-→mu+mu-, reversed sink false, hel ↓↑↑↓): got=-9.483552e-3i want=+9.483552e-3i` |
| Q2 | same, massive cases only | `amplitude … (FFV1/a, t~t→ta+ta-, reversed sink false, hel ↓↓↓↓): got=4.198238e-4i want=0` |
| M3 | kernel `ffv_vout_bare`: `jl·gl − jr·gr` | `current vs jioxxx (FFV4/a, e+e-→mu+mu-, hel ↑↓): \|diff\|=1.742e0, scale=8.710e-1` |
| Q3 | same, massive cases only | `current vs jioxxx (FFV4/a, t~t→ta+ta-, hel ↓↓): \|diff\|=1.205e-3, scale=1.621e-3` |
| M4 | kernel `ffv_vout_bare` reversed branch: `−(jr·gl − jl·gr)` | `amplitude … (FFV4/a, e+e-→mu+mu-, reversed sink true, hel ↓↑↓↑): got=+3.603750e-1i want=-3.603750e-1i` |
| Q4 | same, massive cases only | `amplitude … (FFV4/a, t~t→ta+ta-, reversed sink true, hel ↓↓↓↓): got=6.647210e-4i want=-1.061221e-3i` |
| M5 | kernel `chiral_weighted`: `−gr` | **passes**. The s-channel test does not reach this kernel, but the `helas::` lib suite catches the mutation (S5) |
| S5 | M5 against `cargo test -p vibegraph-lib --lib helas::` | 4 failures: `ffv_fermion_out_matches_generic_chiral_pair`, `renumbering_preserves_signs_and_amplitudes`, `test_ward_identity_full_amplitude_eemumuaa`, `…_eemumutata_a` |
| M6 | kernel `propagate_vector_bare`: `/ (m² − imΓ)` | `current vs jioxxx (FFV2/Z, t~t→ta+ta-, hel ↓↓): \|diff\|=1.774e-4, scale=6.411e-3` |
| M7 | `jioxxx`: `/ (m² − imΓ)` (the HELAS form) | `current vs jioxxx (FFV2/Z, t~t→ta+ta-, hel ↓↓): \|diff\|=1.774e-4, scale=6.409e-3` |
| M8 | ALOHA `ffv2_3`: OM3 = 0 | `−i·jioxxx vs ALOHA FFVk_3 (FFV2/Z, t~t→ta+ta-, hel ↓↓): \|diff\|=6.627e-3, scale=6.411e-3` |
| P1 | base-commit test, M1 | **fails**: `current vs jioxxx (FFV1/a, hel ↑↓): \|diff\|=8.710e-1` |
| P2 | base-commit test, M5 | passes |
| P3 | base-commit test, M6 | **passes**: this is the longitudinal blind spot the backlog item names |

The denominator difference is 2.8% of the current scale at FFV2/Z. That is
about 3×10¹⁰ times the tolerance.

## Resolution of the HELAS-vs-ALOHA denominator

MadGraph 3.7.1's `HELAS/jioxxx.F:114-117` uses Fabio's fixed width,
`cm2 = dcmplx(vm2, -vmass*vwidth)`, so `cs = (q·J)/cm2`. ALOHA instead
declares `OM{n} = 1/M{n}**2` with a real `M` (`aloha/aloha_writers.py:624`).
`M` is complex only when `aloha.complex_mass` is set.

MadGraph 5 evaluates the SM through ALOHA, and the production kernel matches it
(`kernel.rs` `propagate_vector_bare`, `/ vm2`). The port is an oracle for the
production kernel, so I **aligned the port with ALOHA**. The `jioxxx` doc
comment states the deviation from HELAS, and when the two agree: whenever
`q·J = 0`, otherwise they differ at O(Γ/m) of the longitudinal term.

The test therefore needs no tolerance for the difference: it stays at 1e-12.
The unaligned port fails the new test, as M7 shows.

I did not change the production kernel.

## Blind spots

- **Per-diagram global signs.** Each sink orientation's global factor (+i or −i)
  is pinned as measured, but it is a hand-built-diagram convention: |M|² cannot
  see it, and I have no derivation of it. Relative signs between diagrams are
  pinned only by the MadGraph-gated processes.
- **Shared bilinears.** `jioxxx`, `iovxxx` and the kernel share
  `left_current`/`right_current`, so a bug common to both bilinears is invisible
  to the jioxxx comparisons. The ALOHA cross-check covers only the
  single-structure FFV2/FFV4 currents, not FFV1, FFV5 or the amplitude sink.
- **Fermion-continuing chiral kernels.** The test never reaches
  `ffv_fin`/`ffv_fout` (`chiral_weighted`, M5). Other `helas::` tests cover them
  (S5).
- **No MadGraph oracle.** The test is hermetic, so a convention shared by the
  port and the kernel is not checked against MadGraph. That includes the real
  m² and the [g_R, g_L] swap for a reversed line. Only the ALOHA transcriptions
  pin the denominator independently, and they are themselves hand
  transcriptions.
- **The amplitude's longitudinal term is small.** It is
  `(q·J_t)(q·J_τ)/m_Z²`, and m_τ is small. The current-level check carries
  that sensitivity: M6 and M7 fail at the current.

## Found

1. **The brief's premise is partly wrong.** The old `test_eval_jioxxx` already
   failed a flip of `jioxxx`'s L/R relative sign (P1). In the massless regime,
   a flip is a per-helicity global sign, and the per-helicity complex current
   comparison sees it. The "passes" in F-B Found 2 refers to the e⁺e⁻→μ⁺μ⁻
   |M|² tests in `helas/mod.rs`, not to this test. The real gaps were the
   chiral amplitude sinks, the kernel's fused reversed branch (M4), and the
   longitudinal denominator (P3). All three are now closed.
2. **The "different HELAS gc decomposition" comment was a misdiagnosis.** It
   was a reversed line orientation in the hand-built sink, explained above.
   Nothing to file, but the sprint log should carry the correction.
3. **`research/kb/history/notes/10-lorentz-runtime-eval-plan.md:216` describes
   the denominator as "Fabio fixed-width".** The note is archived and unchanged
   by design, so this only matters if a live concept repeats the claim. I did
   not check the live concepts.
4. **The bilinears have no independent oracle at the amplitude sink.** To close
   it, extend the ALOHA cross-check to the sink, or to FFV1/FFV5, with
   component transcriptions of `FFV1_0`/`FFV2_0`/`FFV5_0`. Alternatively, bank
   an MG per-helicity table that reaches a massive Z current, such as
   `t t~ > ta+ ta-` through the Z. That would pin the shared-bilinear class and
   the reversed-line [g_R, g_L] convention against MadGraph.
