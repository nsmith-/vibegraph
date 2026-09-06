# Contributing

Send a patch. That is the whole process.

No CLA, no issue template, no design document required first, no commit-message
police beyond keeping the subject line short and in the imperative. Open an
issue if you want to talk something through before writing it — useful for
anything large, entirely optional for anything else. A PR that arrives with no
preamble is welcome too.

## What CI checks

Three things, all runnable locally in about the time it takes to read this:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

That last one is the hermetic suite: it runs on a bare clone with no submodules,
no downloads and no Python. It never skips, so a green run means green.

## If you touched the physics

There is a second, much heavier layer that compares this generator against
MadGraph5_aMC@NLO — diagram by diagram, amplitude by amplitude, cross section by
cross section. It needs a checked-out UFO submodule, fetched PDF sets and a
bundle of frozen MadGraph runs, so it is not something you can casually run on a
fresh clone:

```bash
pixi run validate
```

`.agents/skills/extended-validation/SKILL.md` maps which change needs which
gate. If you can run the relevant one, paste what it printed. If you cannot, say
so and let CI do it — that is a completely normal way to contribute here. The
only thing that causes trouble is a changed numerical result with no gate output
behind it, because there is no way for a reviewer to tell a fix from a
regression by reading the diff.

`AGENTS.md` has the conventions — units, metric signature, momentum layout,
comment style — and is worth a skim before a first patch. It is written for AI
harnesses but it is just the project's house rules, and it reads fine as prose.

## If you touched the CLI or want to write docs

The documentation site under `docs/` is an mdBook, published from `main` by
`docs.yml`; `scripts/build-docs.sh` builds the same site locally, API reference
included, given `mdbook` on `PATH`. The command reference chapter is generated
from `vibegraph --help`, so after changing a flag or its help text run
`scripts/gen-cli-docs.sh` and commit the result — `cargo test` fails while the
committed chapter is stale.

## Using AI to write it

Encouraged, with four small asks. See [`AI_POLICY.md`](AI_POLICY.md) — the short
version is: name the model in an `Assisted-by:` trailer, never let it sign as a
co-author, understand what you are sending, and back physics claims with a gate.

## Licence

Dual MIT / Apache-2.0, at your option. Contributions are taken under the same
terms; sending a patch is how you agree to that, and there is nothing to sign.
