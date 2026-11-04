---
type: Session Report
title: "G-F1 report: a PDF set on the data path is found with no home directory"
description: "Tiers.cache_root is optional; the cache and download tiers refuse with a named NoCacheRoot error, and CLI tests run as a user with no home directory pin both outcomes."
status: draft
tags: [grid-worker-fixes, pdf, lhapdf, cache, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker`. Closes
`pdf-tiers-need-a-home-directory`.

## Change

- `Tiers.cache_root` is `Option<PathBuf>`; `Tiers::from_env` takes the option and
  `cache_pdf_dir()` returns an option. `pdf_tiers()` no longer refuses: it passes
  `cache_root()` through.
- `locate_set` reads the data path and CVMFS first, then needs the root. With none it
  returns the new `LookupError::NoCacheRoot`: "PDF set X was not found on
  $LHAPDF_DATA_PATH or the CVMFS LHAPDF directory, and the cache and download tiers
  need a home directory, which cannot be located; set $VIBEGRAPH_HOME to a writable
  directory, or put the PDF set's directory on $LHAPDF_DATA_PATH". `fetch_index`
  refuses the same way, and `local_indices` skips the cache.
- The advice names the data path only after saying the set is not on it.

## Tests

- `vibegraph-lib` `no_cache_root_serves_local_tiers_and_refuses_at_the_cache`:
  data path and CVMFS serve with no root, the fetch factory is never asked;
  an absent set and the index fetch refuse with `NoCacheRoot`.
- `vibegraph-cli/tests/cli_pdf_cache.rs`: `a_set_on_the_data_path_is_found_without_a_home_directory`
  and `an_absent_set_without_a_home_directory_is_refused_at_the_cache_tier`, both
  through `vibegraph integrate` (proton Drell-Yan, the narrowest CLI path that
  reaches `pdf_tiers`).

## Brief correction

Removing `HOME` and `VIBEGRAPH_HOME` does not make `cache_root()` `None` on Unix:
`dirs::home_dir()` falls back to the user database. A first version of the tests
that only removed the variables passed before the fix (12 passed). The CLI tests
therefore run the binary as uid 4242424, which has no passwd entry, and skip
(printing why) when the test process may not change user, i.e. when it is not
privileged. The lib test covers the unprivileged case. The brief's
`cargo test -p vibegraph-cli` names no package; the CLI package is `vibegraph`.

## Evidence

Fails before the fix (all source changes stashed, only the new test file restored):

```
$ cargo test -p vibegraph --test cli_pdf_cache home
test an_absent_set_without_a_home_directory_is_refused_at_the_cache_tier ... FAILED
test a_set_on_the_data_path_is_found_without_a_home_directory ... FAILED
 ERROR cannot locate a home directory for the asset cache; set $VIBEGRAPH_HOME, or put the PDF set's directory on $LHAPDF_DATA_PATH
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

After the fix:

```
$ cargo test -p vibegraph --test cli_pdf_cache
test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

Gates: `cargo fmt --all --check` rc 0; `cargo clippy --workspace --all-targets -- -D warnings`
rc 0; `cargo test -p vibegraph` rc 0; `cargo test` rc 0 (see commit).

## Relation to `cache-root-resolved-two-ways`

Not widened. `cache_root()` and `locate_from_env` are untouched, so the two
resolution paths are as they were; the change only makes the PDF lookup consume
the result lazily. `--ufo-dir`'s `no_home` refusal is unchanged.

## Found

- `$HOME` unset is not "no home directory" on Unix (`dirs` consults the user
  database), so the refusal text "cannot locate a home directory" is reachable
  only for users without a passwd entry (some container uids). Nothing to fix; the
  hygiene item should not assume the variable decides it.
