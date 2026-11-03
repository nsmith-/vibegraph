# grid-worker-fixes log

## 2026-11-03
* **Creation**: Sprint folder created.
* Plan posted as draft PR #87. G-F1 dispatched on worktree `/home/user/wt/G` (feature-dev, Sonnet).
* G-F1 checked (10a3209). The manager re-ran `cargo test -p vibegraph --test cli_pdf_cache home` as root: 2 passed.
  - The brief was wrong in two places. With `HOME` unset, `dirs::home_dir` falls back to the passwd database. And the CLI package is `vibegraph`, not `vibegraph-cli`.
  - The CLI tests therefore run the binary as a uid with no passwd entry, and they skip when the test may not change user. That covers CI, so on CI only the lib unit test exercises the fix. This is for G-HY1 to weigh.
  - The item's `closes_when` holds, so its file is deleted here. G-F2 dispatched.
* G-F2 checked (e9041eb). The manager re-ran the four new unit tests: they pass.
  - Root cause: `member_line_pdg` assumed that a member differs from its group's representative only by conjugation. A QCD-only group mixes u and d, so a quark line that an outgoing leg carries had no rule that fit.
  - Corrections to the brief:
    - the row's own full-`j` process is refused too, not only the quark-only jet;
    - integrate reports no per-subprocess σ, so G-F2 integrated each subprocess on its own. That gave χ² 3.90 for 7 dof on one seed, against this side's own integrand.
  - The item's `closes_when` holds (events written; the test fails before the fix), so its file is deleted here.
  - G-V1 dispatched to gate the ttx row's own events against MadEvent's (G-F2's Found 1).
