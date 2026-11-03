# grid-worker-fixes log

## 2026-11-03
* **Creation**: Sprint folder created.
* Plan posted as draft PR #87. G-F1 dispatched on worktree `/home/user/wt/G` (feature-dev, Sonnet).
* G-F1 checked (10a3209). The manager re-ran `cargo test -p vibegraph --test cli_pdf_cache home` as root: 2 passed.
  - The brief was wrong in two places. With `HOME` unset, `dirs::home_dir` falls back to the passwd database. And the CLI package is `vibegraph`, not `vibegraph-cli`.
  - The CLI tests therefore run the binary as a uid with no passwd entry, and they skip when the test may not change user. That covers CI, so on CI only the lib unit test exercises the fix. This is for G-HY1 to weigh.
  - The item's `closes_when` holds, so its file is deleted here. G-F2 dispatched.
