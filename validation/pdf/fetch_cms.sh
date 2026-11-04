#!/usr/bin/env bash
# Fetch the LHAPDF members CMS's `runcmsgrid.sh` systematics request reads (43
# sets, 880 members) into an LHAPDF tree, for the gates that need them:
# `cms_arguments_reproduce_madgraphs_weights` and
# `the_pdfas_member_reads_its_own_coupling_and_clamp` (`validate_systematics`)
# and the interpolator's table oracle (`pdf::table_oracle`).
#
# The list is the `pdfsets.json` MadGraph's run wrote beside its events
# (`Events/run_01_cms/`, in the banked reference bundle): per set, the members
# the request reads (`members_in_list`, the first N), and the SHA-256 of the
# set's archive and of its `.info`. Each archive is checked against that pin
# before anything is unpacked from it, and the unpacked `.info` against its own.
# Only the `.info` and the listed members are kept; `pdfsets.index` is fetched
# beside them, since the gates resolve LHAPDF ids through it.
#
# Resumable: a set whose directory carries `.done` is complete and skipped; an
# archive already downloaded is re-verified and reused; an interrupted download
# leaves only a `.part` file, which the next run replaces.
#
# Usage:
#   bash fetch_cms.sh [DEST]          # DEST defaults to $SYSTEMATICS_CMS_PDFS,
#                                     # then validation/pdf/cms
#   CMS_PDFSETS_JSON=<run_01_cms>/pdfsets.json bash fetch_cms.sh
#   pixi run fetch-cms-pdfs
#
# Downloads go through ../fetch_common.sh and its consent rules; consent is
# asked once for the whole list.
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/fetch_common.sh"

DEST="${1:-${SYSTEMATICS_CMS_PDFS:-$VG_VALIDATION_DIR/pdf/cms}}"
LIST="${CMS_PDFSETS_JSON:-$VG_VALIDATION_DIR/madgraph/output/pp_to_ll/Events/run_01_cms/pdfsets.json}"
BASE="https://lhapdfsets.web.cern.ch/current"

[ -f "$LIST" ] || vg_die "no $LIST: the banked reference bundle carries it (pixi run fetch-refdata)"
mkdir -p "$DEST"

# One line per set: name, members in the list, archive SHA-256, .info SHA-256.
# Read loops rather than `mapfile` and associative arrays: macOS ships bash 3.2.
SETS=()
while IFS= read -r line; do
  SETS+=("$line")
done < <(python3 -I -c '
import json, sys
for name, s in sorted(json.load(open(sys.argv[1])).items()):
    print(name, s["members_in_list"], s["archive_sha256"], s["info_sha256"])
' "$LIST")
[ "${#SETS[@]}" -gt 0 ] || vg_die "$LIST lists no set"

n_pending=0
for line in "${SETS[@]}"; do
  read -r name _ <<<"$line"
  [ -f "$DEST/$name/.done" ] || n_pending=$((n_pending + 1))
done
if [ "$n_pending" -eq 0 ] && [ -f "$DEST/pdfsets.index" ]; then
  vg_say "⊘ all ${#SETS[@]} sets of $LIST already in $DEST"
  exit 0
fi

vg_consent "$n_pending of the ${#SETS[@]} CMS PDF sets into $DEST" "$BASE" ||
  vg_die "the CMS PDF sets were not acquired"
export VIBEGRAPH_FETCH_CONSENT=1

if [ ! -f "$DEST/pdfsets.index" ]; then
  vg_download "$BASE/pdfsets.index" "$DEST/pdfsets.index" "the LHAPDF set index" ||
    vg_die "pdfsets.index was not acquired"
fi

for line in "${SETS[@]}"; do
  read -r name n_members archive_sha info_sha <<<"$line"
  [ -f "$DEST/$name/.done" ] && continue
  tarball="$DEST/$name.tar.gz"
  if [ -f "$tarball" ] && ! vg_verify_sha "$tarball" "$archive_sha" 2>/dev/null; then
    rm -f "$tarball"
  fi
  if [ ! -f "$tarball" ]; then
    vg_download "$BASE/$name.tar.gz" "$tarball" "the $name PDF set" ||
      vg_die "$name was not acquired"
    if ! vg_verify_sha "$tarball" "$archive_sha"; then
      rm -f "$tarball"
      vg_die "$name.tar.gz does not match the archive SHA-256 in $LIST"
    fi
  fi

  # The archive's own spelling of each wanted path (some carry a leading ./).
  listing="$(tar tzf "$tarball")"
  entries=()
  for ((i = -1; i < n_members; i++)); do
    if [ "$i" -lt 0 ]; then
      want="$name/$name.info"
    else
      want="$(printf '%s/%s_%04d.dat' "$name" "$name" "$i")"
    fi
    entry="$(printf '%s\n' "$listing" | grep -Fx -e "$want" -e "./$want" | head -n 1 || true)"
    [ -n "$entry" ] || vg_die "$name.tar.gz holds no $want"
    entries+=("$entry")
  done

  tar xzf "$tarball" -C "$DEST" "${entries[@]}"
  vg_verify_sha "$DEST/$name/$name.info" "$info_sha" ||
    vg_die "$name.info does not match the .info SHA-256 in $LIST"
  rm -f "$tarball"
  : >"$DEST/$name/.done"
  vg_say "✓ $name: $n_members of its members"
done
vg_say "✓ the ${#SETS[@]} sets of $LIST are in $DEST"
