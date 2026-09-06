#!/usr/bin/env bash
# Assemble the documentation site under target/site/:
#
#   target/site/            the mdBook (docs/), served at the site root
#   target/site/api/        rustdoc for vibegraph-lib, with the KaTeX header
#                           from doc-include/ so LaTeX in doc comments renders
#
# and refresh docs/src/cli/reference.md from the built binary first, so the
# published CLI reference is always the binary's own help text. `docs.yml` runs
# exactly this script; locally, `pixi run docs` does too. Requires `mdbook` on
# PATH (https://rust-lang.github.io/mdBook/guide/installation.html).
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
site="$repo/target/site"

cd "$repo"
command -v mdbook >/dev/null || { echo "mdbook not found on PATH" >&2; exit 1; }

cargo build -q -p vibegraph
scripts/gen-cli-docs.sh target/debug/vibegraph

RUSTDOCFLAGS="--html-in-header $repo/doc-include/mathjax-header.html" \
  cargo doc -q --no-deps -p vibegraph-lib

rm -rf "$site"
mdbook build docs --dest-dir "$site"
rm -rf "$site/api"
cp -R target/doc "$site/api"
# Pages serves nothing under a directory named `.lock`-style hidden files;
# cargo's lock file has no business in the tree anyway.
rm -f "$site/api/.lock"

# Every link the book makes into the API tree must resolve to a file the
# rustdoc build produced, or the page is wrong: rustdoc paths change when an
# item moves, and nothing else here would notice.
missing=0
while IFS= read -r ref; do
  target="${ref#*api/}"
  target="${target%%#*}"
  if [[ ! -e "$site/api/$target" ]]; then
    echo "broken API link: $ref" >&2
    missing=1
  fi
done < <(grep -rhoE '\]\((\.\./)*api/[^)#]+(#[^)]*)?\)' docs/src | sed -E 's/^\]\((.*)\)$/\1/' | sort -u)
[[ $missing -eq 0 ]] || exit 1

echo "site assembled in $site (open $site/index.html)"
