---
type: Session Brief
title: "G-F1: find a set on LHAPDF_DATA_PATH with no home directory"
description: "pdf_tiers needs the cache root only when the cache or download tier is reached; a CLI test with HOME and VIBEGRAPH_HOME unset pins it."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: [pdf-tiers-need-a-home-directory]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker` (draft PR #87). Start
at the commit that adds this brief. Read the item
(`python3 scripts/kb.py backlog --item pdf-tiers-need-a-home-directory`), the
[sprint](../sprint.md), and `vibegraph-cli/src/assets.rs` (`pdf_tiers`,
`cache_root`). Also read the open hygiene item `cache-root-resolved-two-ways`:
do not widen scope into it, but do not make it worse, and say in the report how
your change relates to it.

**Environment.**
- Linux container, 4 cores, about 25 GB free disk. Pixi is not installed;
  run the cargo commands that pixi tasks wrap.
- Build with
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.
- A sibling performance session builds in `/home/user/wt/M` and times
  things there. Run long builds in the background, and do not run
  CPU-heavy loops longer than you need.
- Reference data is hard-linked into the worktree; never write into it.

**Task.**
- The lookup reads `$LHAPDF_DATA_PATH` (and `$LHAPATH`) and any other tier
  that does not need the cache before it resolves the cache root.
- The cache root is required only when the cache or download tier is
  reached. A refusal then names that tier, and says the set was not found on
  the data path.
- Recheck the refusal's advice text so it no longer recommends what already
  failed.
- Pin it with CLI tests:
  - with `HOME` and `VIBEGRAPH_HOME` removed from the environment and the set
    on `LHAPDF_DATA_PATH`, the lookup succeeds (through `generate` or the
    narrowest CLI path that reaches `pdf_tiers`; say which);
  - with the set absent, the refusal is the cache-tier one.

  Show the first test fails before the fix (stash the fix, run, record the
  output).
- Gates: `cargo test -p vibegraph-cli`, the default `cargo test`, fmt, clippy.

**Report.** `research/kb/sprints/grid-worker-fixes/sessions/G-F1-report.md`
(type Session Report), with commands and outputs and a Found section. Commit
as described in your agent definition. Do not push.
