#!/usr/bin/env bash
# Assemble the banked-reference bundle from the local MadGraph work area.
#
# The bundle is what lets the banked layer run on a machine that has never run
# MadGraph: it carries the frozen outputs the gates read — the unweighted event
# files and their banners and logs, the run and parameter cards, each
# subprocess's `leshouche.inc` and every `matrixN_orig.f` (one per diagram-group
# case, `#1`, `#2`, ...), the combined `results.dat`, and the fixed-grid
# amplitude tables — and nothing that a build produces
# (objects, libraries, executables, generated Fortran beyond the two files the
# gates parse).
#
# It unpacks *into* `validation/madgraph/output/`, so a fetched checkout and a
# machine that generated the runs itself present the gates with identical paths.
#
# The archive is byte-reproducible from a given work area: the member list is
# sorted in the C locale, every member is staged with the same timestamp, mode
# and (blank) ownership, event files are carried decompressed (so no gzip
# encoder's output is in the archive), and zstd runs single-threaded at a fixed
# level. Two runs over the same work area therefore hash the same, which is what
# makes the SHA-256 in `validation/manifest.toml` a pin rather than a snapshot —
# and a work area that was itself unpacked from a bundle re-assembles to that
# same bundle, which a gzipped-member archive could not promise.
#
# Usage:
#   bash validation/madgraph/assemble_bundle.sh          # assemble + report
#   bash validation/madgraph/assemble_bundle.sh --check  # verify the pin only
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/fetch_common.sh"

WORK_AREA="$VG_VALIDATION_DIR/madgraph/output"
BUNDLE_DIR="$WORK_AREA/bundle"
ARCHIVE_NAME="$(vg_manifest_value refdata archive)"
PINNED_SHA="$(vg_manifest_value refdata sha256)"
[ -n "$ARCHIVE_NAME" ] || vg_die "no [refdata] archive name in $VG_MANIFEST"
ARCHIVE="$BUNDLE_DIR/$ARCHIVE_NAME"

CHECK_ONLY=0
[ "${1:-}" = "--check" ] && CHECK_ONLY=1

if [ "$CHECK_ONLY" = 1 ]; then
  [ -f "$ARCHIVE" ] || vg_die "no bundle at $ARCHIVE to check"
  actual="$(vg_sha256 "$ARCHIVE")"
  if [ "$actual" = "$PINNED_SHA" ]; then
    vg_say "✓ $ARCHIVE_NAME matches the manifest pin ($actual)"
    exit 0
  fi
  vg_die "$ARCHIVE_NAME hashes $actual, manifest pins $PINNED_SHA"
fi

[ -d "$WORK_AREA" ] || vg_die "no MadGraph work area at $WORK_AREA"

STAGE="$(mktemp -d "${TMPDIR:-/tmp}/vg-refdata-XXXXXX")"
trap 'rm -rf "$STAGE"' EXIT

# Per process directory, the files the banked gates read. Everything else in a
# process directory is build product: a fetching checkout never compiles it.
vg_say ">>> selecting banked files under $WORK_AREA"
(
  cd "$WORK_AREA"
  for proc in */; do
    proc="${proc%/}"
    [ -d "$proc/SubProcesses" ] || continue
    find "$proc/Cards" "$proc/Events" -type f 2>/dev/null || true
    find "$proc/SubProcesses" -maxdepth 1 -type f -name results.dat 2>/dev/null || true
    find "$proc/SubProcesses" -maxdepth 2 -type f \
      \( -name leshouche.inc -o -name 'matrix[0-9]*_orig.f' \) 2>/dev/null || true
    # Per-channel integration logs: where MadGraph prints the alpha_s source rule
    # and its own alpha_s at the scales the run evaluated.
    find "$proc/SubProcesses" -maxdepth 3 -type f -name 'run_*_log.txt' 2>/dev/null || true
  done
  # The fixed-grid amplitude tables sit beside the process directories.
  find . -maxdepth 1 -type f -name '*_amplitude.csv' 2>/dev/null | sed 's|^\./||'
) | LC_ALL=C sort > "$STAGE/members.txt"

count="$(wc -l < "$STAGE/members.txt" | tr -d ' ')"
[ "$count" -gt 0 ] || vg_die "the work area holds no banked files"
vg_say "    $count files"

vg_say ">>> staging with normalised metadata"
mkdir -p "$STAGE/root"
tar -cf - -C "$WORK_AREA" -T "$STAGE/members.txt" | tar -xf - -C "$STAGE/root"

# The event files travel as plain Les Houches text. MadGraph writes them gzipped
# and every gate reads them gzipped — `vg_ensure_refdata` gzips them back as it
# unpacks — but an already-gzipped member is incompressible, so archiving them as
# they sit spends most of the archive on bytes zstd cannot touch. Decompressed
# first, the same events cost about a third less. What a gate reads does not
# change, and neither does what the byte-for-byte round-trip gate compares
# against: gzip is lossless, so those are these bytes, one encoding layer nearer.
vg_say ">>> decompressing the event files"
find "$STAGE/root" -type f -name '*.lhe.gz' -exec gzip -d {} +
sed 's/\.lhe\.gz$/.lhe/' "$STAGE/members.txt" | LC_ALL=C sort > "$STAGE/archive.txt"

find "$STAGE/root" -type d -exec chmod 755 {} +
find "$STAGE/root" -type f -exec chmod 644 {} +
find "$STAGE/root" -exec touch -t 197001010000 {} +

vg_say ">>> writing $ARCHIVE"
mkdir -p "$BUNDLE_DIR"
COPYFILE_DISABLE=1 tar -cf "$STAGE/bundle.tar" \
  --format ustar --uid 0 --gid 0 --uname '' --gname '' --no-mac-metadata \
  -C "$STAGE/root" -T "$STAGE/archive.txt"
zstd -19 -q -f --single-thread --no-progress -o "$ARCHIVE" "$STAGE/bundle.tar"

sha="$(vg_sha256 "$ARCHIVE")"
size="$(wc -c < "$ARCHIVE" | tr -d ' ')"
printf '%s  %s\n' "$sha" "$ARCHIVE_NAME" > "$BUNDLE_DIR/SHA256SUMS"

vg_say "✓ $ARCHIVE_NAME  $size bytes"
vg_say "  sha256 = $sha"
if [ "$sha" != "$PINNED_SHA" ]; then
  vg_say ""
  vg_say "!!! the manifest pins $PINNED_SHA"
  vg_say "    The work area's contents changed. Update [refdata].sha256 in"
  vg_say "    $VG_MANIFEST and publish the new archive before the fetch path works."
fi
