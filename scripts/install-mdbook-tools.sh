#!/usr/bin/env bash
# Install the three binaries the documentation site needs, at pinned
# versions, into a directory of your choosing (default ~/.local/bin):
#
#   mdbook          the book renderer            (prebuilt release)
#   mdbook-mermaid  ```mermaid fences → diagrams (prebuilt release)
#   mdbook-katex    $…$ math → HTML at build time (cargo install; no
#                   prebuilt Linux asset is published for it)
#
# Usage: scripts/install-mdbook-tools.sh [bin-dir]
#
# Idempotent: a binary already present at the pinned version is left alone,
# which is what lets a cached ~/.cargo/bin in CI skip the compile.
set -euo pipefail

bindir="${1:-$HOME/.local/bin}"
mkdir -p "$bindir"

MDBOOK_VERSION=v0.4.52
MERMAID_VERSION=v0.15.0
KATEX_VERSION=0.9.3

have() { command -v "$1" >/dev/null 2>&1 && "$1" --version 2>/dev/null | grep -q "$2"; }

if ! have "$bindir/mdbook" "${MDBOOK_VERSION#v}"; then
  curl -fsSL "https://github.com/rust-lang/mdBook/releases/download/${MDBOOK_VERSION}/mdbook-${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz -C "$bindir"
fi
if ! have "$bindir/mdbook-mermaid" "${MERMAID_VERSION#v}"; then
  curl -fsSL "https://github.com/badboy/mdbook-mermaid/releases/download/${MERMAID_VERSION}/mdbook-mermaid-${MERMAID_VERSION}-x86_64-unknown-linux-gnu.tar.gz" \
    | tar -xz -C "$bindir"
fi
if ! have "$bindir/mdbook-katex" "$KATEX_VERSION"; then
  cargo install --locked --version "$KATEX_VERSION" --root "$(dirname "$bindir")" mdbook-katex
fi

echo "mdbook tools in $bindir:"
"$bindir/mdbook" --version
"$bindir/mdbook-mermaid" --version
"$bindir/mdbook-katex" --version
