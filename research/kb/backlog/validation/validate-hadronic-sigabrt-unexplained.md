---
type: Backlog Item
title: "Heap corruption detected once in validate_hadronic (macOS libmalloc abort)"
description: "validate_hadronic aborted once on the M3: libmalloc found a corrupted free list while a scratch arena grew. About 34 reruns were clean; the corrupting write is not located."
area: validation
state: open
priority: medium
closes_when: "The corrupting write is located and fixed or attributed outside this crate (the allocator, a C dependency or the host); or a macOS soak with and without the nano allocator puts a rate on the abort, recorded here."
blocked_by: []
opened: 2026-10-26
tags: [flaky, banked, macos, abort, heap-corruption, rayon]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: pr60-m3, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6373556349", title: "nsmith-/vibegraph#60, the M3's refdata-11 comment"}
  - {id: pr60-ips, resource: "https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6428160160", title: "nsmith-/vibegraph#60, the M3's crash report"}
---
One full `validate` run of the refdata-11 export (`47ac21a`) on the M3 died with
SIGABRT in `validate_hadronic`; the rerun was clean
([#60](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6373556349)).

**The crash report** (macOS 15.8.1;
[#60](https://github.com/nsmith-/vibegraph/pull/60#issuecomment-6428160160)):
libmalloc's `nanov2_guard_corruption_detected` called `abort()`, so no Rust panic
line was printed. Faulting stack: `nanov2_realloc` ← `Vec::resize` ←
`ScratchSpace::ensure_sizes` (`helas/eval/run.rs:101`) ← `fill_arenas` ←
`BoundAmplitude::eval_m2`. That is where the damage was noticed, not
necessarily where it was done.

**Ruled out so far.** The binary has no `unsafe` of this crate's own (the
`get_unchecked` pair in `helas/eval/run.rs` sits behind `unchecked-study`, which
no build enables) and no global allocator. A dependency's `unsafe` (rayon, rand,
zstd-sys, the PDF reader) or the OS allocator remain.

**Reproduction.** None of 22 Linux runs under the banked flags, or 12 runs of
the exact crashing binary on the M3 (6 under `MallocScribble`/`MallocGuardEdges`),
aborted: one abort in about 70 runs.

**The same signature has been seen before.**
[malloc-abort-under-proton-sample-load](malloc-abort-under-proton-sample-load.md)
(2026-09-07) was another one-off macOS libmalloc abort. It came under rayon, in
unrelated code (`kt::Clustering`), and was also never reproduced. Two
corruptions in different safe-Rust paths point at something shared (a
dependency, the allocator, or the host) rather than at either call site.

**AddressSanitizer on Linux, clean (2026-10-27).** Nightly 1.101 with
`-Zbuild-std`, so std, rayon and rand are instrumented, and the spawned CLI child
is instrumented too. A heap-buffer-overflow control was caught. Runs: the lib
unit tests, 3 × `validate_hadronic` and 2 × `validate_samples_proton` (which
includes `pp_to_llj_dyn`), about 4.6 h in all, with no ASan report. A corruption
on most runs is ruled out, a rare one is not.

**Next.** The suspects left are the macOS nano allocator and C code not built
with ASan (`zstd-sys`). Two steps: soak on the M3 with `MallocNanoZone=0`
against the default, which also puts a rate on the abort; and rebuild with
clang ASan for `zstd-sys`. Separately,
[validate-sh-keeps-no-crash-evidence](../hygiene/validate-sh-keeps-no-crash-evidence.md)
makes a recurrence leave its evidence.
