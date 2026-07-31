#!/usr/bin/env bash
# Acquire the banked-reference bundle — the frozen MadGraph outputs the banked
# validation layer reads — and unpack it into the work-area layout.
#
# Usage:
#   pixi run fetch-refdata
#   VIBEGRAPH_REFDATA_SOURCE=/path/to/vibegraph-refdata-1.tar.zst bash validation/fetch_refdata.sh
#
# A work area that already holds process directories is left untouched: on a
# machine that generates the references, this is a no-op. Which archive, from
# where, and its SHA-256 are pinned in ../validation/manifest.toml; the fetch and
# the verification live in fetch_common.sh.
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/fetch_common.sh"

vg_ensure_refdata ||
  vg_die "the banked MadGraph reference runs are a declared input of the banked layer and were not acquired"
