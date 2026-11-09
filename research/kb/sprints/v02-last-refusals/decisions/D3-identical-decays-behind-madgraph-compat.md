---
type: Design Decision
title: "Sprint I: identical particles across decays move behind --madgraph-compat, which defaults to off; the default cut becomes pairing-weighted"
description: "D3: by default vibegraph keeps every pairing coherently and applies cut_decays = F through a pairing-weighted cut, Σ_p w_p Θ_p |M|²; --madgraph-compat (off by default, every site) gives MadGraph's single pairing and config-1 exemption. Documentation only in v0.2."
decided: 2026-11-08
decided_by: human:nsmith-
status: stable
tags: [cuts, decay-chains, identical-particles, madgraph-compat, ir-safety, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
verified: [{by: "human:nsmith-", at: 2026-11-08}]
sources:
  - {id: d2, resource: "D2-cut-decays-keep-the-stricter-cut.md", title: "D2"}
  - {id: ic1, resource: "../sessions/I-C1-report.md", title: "I-C1 report"}
---
The user's decision, relayed by the vibegraph manager on 2026-11-08. It is
documentation only: no code changes in v0.2, and D2's stricter cut ships until
the weighted cut is implemented.

1. **Default (`--madgraph-compat` off).** Every pairing is kept coherently,
   interference included, as decided on 2026-09-26. `cut_decays = F` is applied
   through a **pairing-weighted cut**:

   ```
   Σ_p w_p Θ_p |M|²,   w_p = |M_p|² / Σ_q |M_q|²
   ```

   Here Θ_p is pairing p's own cut, which exempts p's decay products and cuts
   p's core legs. Once implemented, this replaces D2's stricter rule as the
   default.
2. **`--madgraph-compat` on.** MadGraph's single pairing, its
   `identical_decay_chain_factor`, and its configuration-1 exemption
   (`check_decay`, `setcuts.f:961`). This also evaluates fewer diagrams.
3. **The flag defaults to off, for every site.** vibegraph's own behaviour is
   the default, and the banked MadGraph gates run with `--madgraph-compat` on.

**Why the weighted cut is IR-safe.** The argument is derived, not yet measured.
- By Cauchy–Schwarz, |M|² ≤ N Σ_q |M_q|², so
  w_p Θ_p |M|² ≤ N Θ_p |M_p|². The weighted σ is finite whenever each
  pairing's own |M_p|² is integrable where Θ_p is open.
- **Where it holds:** two-body decays of a massive resonance. In I-C1's limit,
  pairing p's weight goes as ε², which cancels pairing q's 1/ε², and Θ_q cuts
  the leg in q's own term.
- **Where it fails:** multi-body decays with massless products. There the
  guard is #113's limit check run over each pairing's own diagrams, with that
  pairing's exemptions. Under the weighted rule, H-C3's Found 2 (rebuild the
  guard over the exempting pairing's diagrams) is therefore the right guard. It
  is unsound only for rules that exempt on the full |M|²: the window rule (D2)
  and MadGraph's exemption without dropping the other pairings.
- **Acceptance oracle:** I-C1's probe on the weighted integrand,
  w_p Θ_p |M|² ε² → 0 in its three configurations.
- **Caveat:** w_p is built from diagram subsets that are not gauge invariant off
  the pole. It moves only the interference share (about 0.2%), and it is a
  fixed choice, stated but not blocking.
