#!/usr/bin/env bash
# Assemble the documentation site under target/site/:
#
#   target/site/            the mdBook (docs/), served at the site root
#   target/site/api/        rustdoc for vibegraph-lib, with the KaTeX header
#                           from doc-include/ so LaTeX in doc comments renders
#
# and refresh docs/src/cli/reference.md from the built binary first, so the
# published CLI reference is always the binary's own help text. The backlog
# page, docs/src/backlog.md, is rendered from the research notes' backlog
# items (`pixi run backlog`) and is gitignored: committed, it would change with
# every item and conflict between branches. `docs.yml` runs
# exactly this script; locally, `pixi run docs` does too. Requires `pixi` on
# PATH, and `mdbook`, `mdbook-katex` and `mdbook-mermaid`, which
# `scripts/install-mdbook-tools.sh` provides at the pinned versions.
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
site="$repo/target/site"

cd "$repo"
for tool in mdbook mdbook-katex mdbook-mermaid; do
  command -v "$tool" >/dev/null || { echo "$tool not found on PATH (run scripts/install-mdbook-tools.sh)" >&2; exit 1; }
done
command -v pixi >/dev/null || { echo "pixi not found on PATH (it renders the backlog page)" >&2; exit 1; }

cargo build -q -p vibegraph
scripts/gen-cli-docs.sh target/debug/vibegraph
claims=()
[[ -n "${GITHUB_TOKEN:-}" ]] && claims=(--claims)
pixi run backlog ${claims[@]+"${claims[@]}"} \
  --link-base "https://github.com/nsmith-/vibegraph/blob/main/research/kb/" \
  -o docs/src/backlog.md

RUSTDOCFLAGS="--html-in-header $repo/doc-include/mathjax-header.html" \
  cargo doc -q --no-deps -p vibegraph-lib

rm -rf "$site"
# mdbook-katex leaves a formula it cannot parse as source and only warns, so
# the warning is promoted to a failure here: a broken formula is a broken page.
log="$(mktemp)"
mdbook build docs --dest-dir "$site" 2>&1 | tee "$log"
if grep -q 'Rendering failed' "$log"; then
  echo "a formula failed to render (see the mdbook_katex warnings above)" >&2
  exit 1
fi
rm -f "$log"
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
