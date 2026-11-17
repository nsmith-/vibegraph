#!/usr/bin/env bash
# Move the toolchain pins to the given releases, together:
#   rust-toolchain.toml  channel
#   pixi.toml            requires-pixi
#   .github/workflows/*  setup-pixi's pixi-version
# then re-lock pixi.lock with that pixi (which must be the one on PATH) and
# install the Rust release.
#
# usage: scripts/bump-toolchains.sh <rust, e.g. 1.101.0> <pixi, e.g. 0.85.0>
#
# The scheduled toolchain-bump workflow runs it with the current stable
# releases; run it by hand for an off-schedule bump. Lint fixes are not
# automated: run `cargo fmt --all --check` and both clippy gates afterwards
# (research/kb/tooling/toolchain-pins.md).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

[[ $# -eq 2 ]] || { echo "usage: $0 <rust-version> <pixi-version>" >&2; exit 2; }
rust=$1 pixi=${2#v}
semver='^[0-9]+\.[0-9]+\.[0-9]+$'
[[ $rust =~ $semver ]] || { echo "not an exact Rust release: $rust" >&2; exit 2; }
[[ $pixi =~ $semver ]] || { echo "not an exact pixi release: $pixi" >&2; exit 2; }

have=$(pixi --version | awk '{print $2}')
[[ $have == "$pixi" ]] || {
  echo "pixi on PATH is $have, not $pixi: install it (pixi self-update --version $pixi) and rerun" >&2
  exit 1
}

sed -i.bak -E "s/^channel = \"[^\"]*\"/channel = \"$rust\"/" rust-toolchain.toml
sed -i.bak -E "s/^requires-pixi = \"[^\"]*\"/requires-pixi = \"==$pixi\"/" pixi.toml
for wf in .github/workflows/*.yml; do
  sed -i.bak -E "s/(pixi-version: )v[0-9.]+/\1v$pixi/" "$wf"
done
rm -f rust-toolchain.toml.bak pixi.toml.bak .github/workflows/*.yml.bak

grep -q "^channel = \"$rust\"$" rust-toolchain.toml || { echo "channel not set" >&2; exit 1; }
grep -q "^requires-pixi = \"==$pixi\"$" pixi.toml || { echo "requires-pixi not set" >&2; exit 1; }

pixi lock
rustup toolchain install
