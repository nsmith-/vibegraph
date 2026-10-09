---
type: Design Decision
title: The LHAPDF interpolator is replicated in-house, not taken from scirs2
description: "scirs2's global B-spline missed LHAPDF by up to 98.6% off-knot and pulled ~40 crates, so vibegraph replicates LHAPDF's local log-bicubic; the oracle is LHAPDF itself."
status: draft
tags: [pdf, lhapdf, interpolation, dependencies, decision]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L63-L99", title: "Note 18 §1.2 (Rust spline options and the decision rule)"}
  - {id: n18-h1h2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L560", title: "Note 18 §5 H1/H2 decision records (oracle backend, scirs2 trial)"}
  - {id: n18-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L935-L1039", title: "Note 18 outcome (scirs2 rejection by the numbers)"}
---

# The LHAPDF interpolator is replicated in-house

## Decision

vibegraph evaluates parton densities with its own replica of **LHAPDF 6's
`LogBicubicInterpolator`** (the `lhagrid1` `logcubic` default), in
`vibegraph-lib/src/pdf/interp.rs`, behind its own small trait (`Bicubic2D`; the public
surface is `PdfMember::xfx_q2` / `try_xfx_q2`). It does not use `scirs2-interpolate`, and it
does not link LHAPDF at run time. The algorithm itself is
[scales-pdf/lhapdf-interpolation](lhapdf-interpolation.md).[^n18-h1h2]

## Why

**The reference is LHAPDF's specific local scheme, not "a bicubic spline".** MadGraph
reads its densities through LHAPDF, so matching a MadGraph cross section off-knot means
matching LHAPDF's arithmetic: a *local* cubic Hermite in `(ln x, ln Q²)` with
finite-difference knot derivatives. A *global* not-a-knot B-spline (scipy's
`RectBivariateSpline`, and scirs2's port of it) is a different algorithm. Both read the raw
grid values, so they agree bit for bit on the knots and part between them.[^n18-12]

**The trial measured it.** `scirs2-interpolate` 0.6.1's `RectBivariateSpline` (kx = ky = 3,
s = 0, over the pinned NNPDF23_lo set in `(ln x, ln Q²)`, per flavour) against the LHAPDF
oracle's off-knot interior points:[^n18-h1h2][^n18-outcome]

| | result |
|---|---|
| worst relative error | **9.86e-1** (98.6%, charm, PDG ±4) |
| every flavour | ≥ 3% (down/up ~3.4%, gluon ~9%, strange ~5%) |
| against the acceptance bar of 1e-9 | about eight orders of magnitude past it |
| compile weight | ~40 transitive crates: `scirs2-core`/`-linalg`/`-spatial`, `nalgebra`, `ndarray`, a full BLAS/LAPACK stack (`oxiblas-*`), `simba`, `statrs`, three `rand_distr` versions, `wide` |

A direct LHAPDF-against-scipy comparison on the same set had already diverged by about
120% at some interior points. The in-house replica, which mirrors LHAPDF's per-point
operation order, lands at **1.32e-15** worst on the same off-knot points, in about 250 lines
of code when the decision was taken (`pdf/interp.rs` has since grown to carry the
multi-band walk and the all-flavour path). The trial dependency was reverted, and `Cargo.toml` and `Cargo.lock` carry no
scirs2.

**No LHAPDF FFI.** LHAPDF is a C++ dependency wall, and the `lhagrid1` format is simple
enough not to warrant one. LHAPDF is linked only by the oracle generator
(`validation/pdf/gen_oracle.cpp`, a build-time C++ program in the `madgraph` pixi
environment), never by a crate.

## Revisit condition

[AGENTS.md](../../../AGENTS.md) says never to hand-write a standard primitive: a missing
dependency is a reason to add one. This decision stands because the thing replicated is
not a generic spline. It is LHAPDF's particular local scheme, which is the oracle, and no
generic interpolation crate implements it.

**Re-test it** if a crate appears that claims LHAPDF compatibility (a Rust LHAPDF port, or
bindings): run it against the same LHAPDF oracle (`validate_pdf_grid`'s off-knot, seam,
corner and `x → 1` categories, accept bar 1e-9 relative on interior points) and weigh its
dependency footprint. Adopt it if it meets the bar; the trait seam is there so the backend
can be swapped without touching callers. The oracle and its tolerances are
[validation/lhapdf-oracle](../validation/lhapdf-oracle.md).

[^n18-12]: Note 18 §1.2, the spline options and the decision rule, with its corrected premise.
[^n18-h1h2]: Note 18 §5, H1 (the LHAPDF oracle backend) and H2 (the scirs2 trial and the in-house replica).
[^n18-outcome]: Note 18 outcome, the scirs2 rejection by the numbers.
