---
type: Backlog Item
title: "Heap corruption detected once in validate_hadronic (macOS libmalloc abort)"
description: "validate_hadronic aborted once on the M3: libmalloc found a corrupted free-list entry while a matrix-element scratch arena grew. About 34 reruns on macOS and Linux were clean, and the corrupting write is not located."
area: validation
state: open
priority: medium
closes_when: "The corrupting write is located (for example under AddressSanitizer on the hadronic suite) and fixed or attributed outside this crate; or an ASan run of the suite is clean and a soak puts a rate on the abort, recorded here."
blocked_by: []
opened: 2026-10-26
tags: [flaky, banked, macos, abort, heap-corruption, rayon]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: pr60-m3, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6373556349", title: "nsmith-/vibegraph#60, the M3's refdata-11 comment"}
  - {id: pr60-ips, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6428160160", title: "nsmith-/vibegraph#60, the M3's crash report"}
---
During the refdata-11 export on the M3, the first full `validate` run on a clean
`git archive` tree of `47ac21a` died with SIGABRT in `validate_hadronic`. The
rerun of the same tree was clean
([#60](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6373556349)).

**The crash report** (macOS 15.8.1, process up 30 s, 31 threads;
[#60](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6428160160)):
- `abort()` came from libmalloc (`nanov2_guard_corruption_detected`), not from
  Rust, so no panic line was printed.
- Faulting stack: `nanov2_realloc` ← `Vec::resize` ←
  `ScratchSpace::ensure_sizes` (`helas/eval/run.rs:101`, growing the
  `Bispinor` arena) ← `fill_arenas` ← `BoundAmplitude::eval_m2` ←
  `BoundSubprocess::eval_m2`.
- The realloc is where the damage was noticed, not necessarily where it was
  done. The other threads were in rayon or ordinary integrand code.

**Ruled out so far.**
- The binary has no `unsafe` of this crate's own. The `get_unchecked` pair in
  `helas/eval/run.rs` sits behind `unchecked-study`, which no build enables.
  No global allocator is installed.
- What remains is a dependency's `unsafe` (rayon, rand, zstd-sys, the PDF
  reader) or the OS allocator.

**Reproduction.** None of these runs aborted:
- 22 Linux runs under the banked flags;
- 12 runs of the exact crashing binary on the M3, 6 of them under
  `MallocScribble`/`MallocGuardEdges`.

That is one abort in about 70 runs.

**The same signature has been seen before.**
[malloc-abort-under-proton-sample-load](malloc-abort-under-proton-sample-load.md)
(2026-09-07) was another one-off macOS libmalloc abort. It came under rayon, in
unrelated code (`kt::Clustering`), and was also never reproduced. Two
corruptions in different safe-Rust paths point at something shared (a
dependency, the allocator, or the host) rather than at either call site.

**Next.** Run the hadronic and proton-sample suites under AddressSanitizer
(nightly, `-Zsanitizer=address`). ASan reports the corrupting write itself,
not a later victim. Separately,
[validate-sh-keeps-no-crash-evidence](../hygiene/validate-sh-keeps-no-crash-evidence.md)
makes a recurrence leave its evidence.
