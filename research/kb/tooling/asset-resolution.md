---
type: Design
title: ~/.vibegraph asset resolution and pinned fetch
description: "PDF sets and UFO models resolve flag → env → ~/.vibegraph → dev fallback; only a pinned PDF set can be fetched, SHA-256-verified before an atomic publish. UFO models are never downloaded."
status: draft
tags: [cli, cache, pdf, ufo, distribution]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-u2-pin, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2445-L2495", title: "Note 24 §U2 outcome: where the pin lives, what landed"}
  - {id: n24-u2-gaps, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2518-L2540", title: "Note 24 §U2 outcome: not verified / known gaps"}
  - {id: n24-u3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2573-L2698", title: "Note 24 §U3 outcome: the cache module, resolution order, Fetch contract"}
  - {id: n24-u4-ufo, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2903-L2969", title: "Note 24 §U4 outcome: the UFO decision, no fetching"}
  - {id: code-cache, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/cache/mod.rs", title: "vibegraph-lib/src/cache/ (mod, resolve, store, pinned)"}
  - {id: code-assets, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/assets.rs", title: "vibegraph-cli/src/assets.rs"}
---
# ~/.vibegraph asset resolution and pinned fetch

A run needs two kinds of external data: a PDF set (hadronic beams only) and,
for any model other than the interned Standard Model, a UFO model directory.
Both are found through one resolution order. Only a PDF set this build has a
compiled-in pin for can be downloaded; UFO models never are.

## Layout and resolution order

The cache root is `$VIBEGRAPH_HOME` if set, otherwise `~/.vibegraph`
(`assets::cache_root`, falling back to `cache::default_cache_root`, which is
`dirs::home_dir().join(".vibegraph")`). Entries live at
`<cache_root>/{pdf,ufo}/<name>/`.

`cache::resolve::locate(kind, name, flag, env, cache_root, dev_fallback)` walks
the same order for both kinds and returns `Located { dir, source, found }`:

| step | PDF set | UFO model | existence checked? |
|---|---|---|---|
| 1. flag | `--pdf-dir <base>` → `<base>/<name>/` | `--ufo-dir <base>` → `<base>/<name>/` | reported, never blocks: an explicit path is returned even if absent |
| 2. env | `$VIBEGRAPH_PDF_DIR/<name>/` | `$VIBEGRAPH_UFO_DIR/<name>/` | same as the flag |
| 3. cache | `<root>/pdf/<name>/` | `<root>/ufo/<name>/` | taken only if the directory exists |
| 4. dev fallback | `validation/pdf/<name>/` relative to the cwd | `./<name>/` (the cwd) | taken only if it exists |
| nothing found | `found: false`, `dir` = the cache path, as the write target for a fetch | same | — |

Flag and env take the step even when the directory is missing, so "explicit
but wrong path" is an error naming that path rather than a silent fall-through.
`locate` reads no environment and no home directory; every input is a
parameter, and seven unit tests in `cache::resolve::tests` each pin one
precedence edge. `locate_from_env` is the thin wrapper that reads the real
variables; it is deliberately untested, since testing it would mean mutating
process environment across parallel tests.

The dev fallback is relative to the working directory, so it only ever
resolves inside a checkout. A user running an installed binary never reaches it.

## PDF sets: what is pinned and when it binds

`vibegraph-lib/src/cache/pinned.rs` holds `PINNED_PDF_SETS`, a compiled-in
table of `{name, url, sha256, archive_bytes}`. It is source, not
configuration, so a user cannot silently run against different data than the
build was validated against.[^n24-u2-pin] It has one entry today:

```rust
pub const PINNED_PDF_SETS: &[PinnedPdfSet] = &[PinnedPdfSet {
    name: DEFAULT_PDF_SET, // "NNPDF23_lo_as_0130_qed"
    url: "https://lhapdfsets.web.cern.ch/current/NNPDF23_lo_as_0130_qed.tar.gz",
    sha256: "60d3c1df1c31e5840f91f4217163ae30a256b9291a5adc894882e86607ef5d63",
    archive_bytes: 27_625_668,
}];
```

`DEFAULT_PDF_SET` also lives here; the CLI holds no second copy of the name.
Why the set is fetched rather than embedded is
[pdf-set-distribution](pdf-set-distribution.md); which set it is and why is
[the pinned PDF set](../scales-pdf/pinned-pdf-set.md).

**The pin binds only on the cache step.** `assets::resolve_pdf_set_dir` returns
a set reached through `--pdf-dir`, `$VIBEGRAPH_PDF_DIR` or the dev fallback as
found, with no checksum check: those name data the caller points at
deliberately, and that is also what keeps sets with no pin usable at all. The
cache step is the only one that decides on the user's behalf what a bare set
name means, so there the pin is authoritative:

- A cache entry counts as current only if its `.vibegraph-checksum` sidecar
  (`store::PIN_FILENAME`) equals the compiled-in SHA-256
  (`pinned::is_current`). An entry with a missing or different sidecar, from an
  interrupted write or a build pinning other data, is refetched, not trusted.
- A set with no pin is used from the cache if present; if absent, the run fails
  telling the user to fetch it and point `--pdf-dir` / `$VIBEGRAPH_PDF_DIR` at it.
- `pdf_set_is_cached(root, name)` answers "would `ensure_pdf_set` fetch?"
  without side effects; it gates whether the consent question is asked.

Only `integrate` and `generate` resolve PDF sets (`integrate.rs`, and
`generate.rs` through `load_pdf_set`), both through this path.

## The fetch seam

The library performs no network I/O. `cache::store::Fetch` is the seam:

```rust
pub trait Fetch {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}
```

Everything downstream of the returned bytes is the store's: `.tar.gz`
extraction (`tar` + `flate2`), unwrapping a single top-level directory (LHAPDF
archives carry one), checksumming, writing the sidecar, and an atomic publish
by `rename` into `<root>/<kind>/<name>/` only after the whole entry is staged.
A failed or interrupted fetch never leaves a half-written entry that `locate`
would treat as cached.[^n24-u3]

`pinned::VerifiedFetch` wraps the caller's `Fetch` and rejects bytes whose
SHA-256 misses the pin *before* the store sees them, so the "a failed fetch
writes nothing" guarantee covers a corrupted or substituted download too. It
records the mismatch, so the caller can tell a tamper signal ("the download
was wrong", `EnsureError::ChecksumMismatch`) from a network signal ("the
download failed").

Checksums differ by kind. A PDF set is pinned by the SHA-256 of the archive
bytes (`ufo::identity::digest_bytes`, i.e. `sha2`). A UFO entry is pinned by
`ufo::identity::model_digest` over the parsed model, recomputed by loading the
extracted directory, so a repackaged but unchanged model pins identically and
an archive that does not parse is never published.

The implementations: `vibegraph-cli/src/fetch.rs` has `HttpFetch` (`ureq`,
body capped at 256 MiB), kept in the CLI crate so `vibegraph-lib` carries no
HTTP or TLS stack; `store::RefusingFetch` and `store::FixedFetch` are public
stubs for refusal paths and tests. `HttpFetch` is constructed in exactly one
place, after consent is granted; every other cache call passes
`RefusingFetch`. That structural invariant is described in
[network-consent](network-consent.md).

## UFO models are never downloaded

FeynRules, where UFO models are published, has no index mapping a model name
to an archive. Its model database is a Trac wiki, one page per model, with
hand-attached files whose names follow no rule. A name-derived URL of the form
`/raw-attachment/wiki/<model>/<model>.tar.gz` was probed and is wrong in two
ways: it returns 404 for `SM`, `HAHM_variableMW_v5`, `EWdim6` and
`DMsimp_s_spin0`, and where it succeeds (`2HDM`) it returns FeynRules
Mathematica sources (`2HDM.fr`, `Lag.fr`, …), not a UFO directory; the UFO on
that page is `2HDM_UFO.tar.gz`, beside a second revision `2HDM_UFO.tar.2.gz`.
[^n24-u4-ufo] So there is no `PINNED_UFO_MODELS` table and no URL builder; a
comment in `store.rs` records the finding where the next person will look.
`store::cache_ufo_model` is kept: fetch → extract → load-and-digest → atomic
publish is sound, and it takes its URL from the caller. Nothing calls it today.

What does exist is resolution (`assets::resolve_ufo_search_path`):

- `import model sm` (or no import) short-circuits to the interned SM before any
  of this, so an SM run never depends on `$HOME` being readable.
- Any other model goes through `locate` with `--ufo-dir` → `$VIBEGRAPH_UFO_DIR`
  → `~/.vibegraph/ufo/` → the working directory.
- A model found nowhere is an error naming the cache path to unpack into
  (`…/ufo/<name>/particles.py`), the flag, the variable, and the fact that no
  download will be attempted.

**Parent, not directory.** `GlobalConfig::ufo_search_path` is the parent a
model name is joined onto, while `locate` returns the model directory itself.
Resolution hands back `located.dir.parent()`; getting this backwards resolves
models to `<dir>/<name>/<name>`. `a_located_model_becomes_its_parent_directory`
pins it on both the flag and the cache step, whose directories are built
differently.

## Caveats

- **The pin's correctness against upstream is checked only by an ignored
  test.** `pinned_checksums_match_the_upstream_archives`
  (`vibegraph-cli/tests/cli_pdf_cache.rs`) downloads the 27.6 MB archive and
  checks it; run it with
  `cargo test -p vibegraph --test cli_pdf_cache -- --ignored` after any pin
  change. The default suite cannot.[^n24-u2-gaps]
- **Upstream repackaging is a live risk.** If CERN re-tars the set, every user
  on a cache miss gets a hard checksum failure. That is the right failure
  direction, but it is a failure. The second detector would be
  [release acceptance](release-acceptance.md) on a timer, which does not run
  yet ([acceptance-yml-weekly-schedule](../backlog/hygiene/acceptance-yml-weekly-schedule.md)).
- **The cache-hit path for the default set is pinned at unit level only.** A
  CLI-level test that populated the cache under the real name would mismatch
  the pin and trigger a 27.6 MB download inside the suite, so the CLI test
  uses an unpinned synthetic set name.
- Tests never touch a real `~/.vibegraph`: library tests pass the cache root
  explicitly, and CLI tests set `$VIBEGRAPH_HOME` to a scratch directory.

[^n24-u2-pin]: Note 24 §U2, "Where the pin lives" and "What landed".
[^n24-u2-gaps]: Note 24 §U2, "Not verified / known gaps", items 1–3.
[^n24-u3]: Note 24 §U3, the `cache` API and the `Fetch` contract.
[^n24-u4-ufo]: Note 24 §U4, "The UFO decision: no fetching, and the URL template removed".
