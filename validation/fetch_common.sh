#!/usr/bin/env bash
# Shared acquisition for the external inputs the validation layers declare:
# the pinned `mg5amcnlo` submodule, LHAPDF sets, and the banked-reference
# bundle. Source it, do not execute it:
#
#     . "$(dirname "$0")/../fetch_common.sh"    # from validation/<dir>/
#
# Everything acquired from the network goes through one function, `vg_download`,
# which asks before it reaches out and refuses when it may not ask. That mirrors
# the CLI's asset seam (`vibegraph-cli/src/assets.rs`): a single place that can
# download, so a caller whose "is it already here?" predicate is wrong still
# cannot fetch silently.
#
# Consent, in the order it is decided:
#
#   VIBEGRAPH_NO_NETWORK=1        refuse, whatever else is set
#   VIBEGRAPH_FETCH_CONSENT=1     granted without asking (what CI sets)
#   an interactive terminal       ask, default no
#   otherwise                     refuse, naming the variable that would allow it
#
# Content-addressed inputs (the bundle) are verified against a SHA-256 pinned in
# `validation/manifest.toml` no matter where the bytes came from, so pointing
# VIBEGRAPH_REFDATA_SOURCE at a local file exercises the same verification path
# as a release download.

VG_VALIDATION_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VG_REPO_ROOT="$(cd "$VG_VALIDATION_DIR/.." && pwd)"
VG_MANIFEST="$VG_VALIDATION_DIR/manifest.toml"

vg_say() { printf '%s\n' "$*" >&2; }
vg_die() { printf '!!! %s\n' "$*" >&2; exit 1; }

# vg_manifest_value TABLE KEY — the string value of `KEY = "..."` inside the
# `[TABLE]` table of the manifest. Only the flat string keys the shell layer
# needs; anything structured is read by the Rust and Python consumers.
vg_manifest_value() {
  awk -v table="[$1]" -v key="$2" '
    /^\[/ { in_table = ($0 == table); next }
    in_table && $1 == key {
      sub(/^[^=]*=[[:space:]]*/, "")
      gsub(/^"|"[[:space:]]*$/, "")
      print
      exit
    }
  ' "$VG_MANIFEST"
}

vg_sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    sha256sum "$1" | awk '{print $1}'
  fi
}

# vg_verify_sha FILE EXPECTED — nonzero on a mismatch. Deleting the offending
# file is the caller's call: a failed download should not be left behind for a
# later run to mistake for a good copy, but a file the user pointed us at is
# theirs.
vg_verify_sha() {
  local file="$1" expected="$2" actual
  actual="$(vg_sha256 "$file")"
  if [ "$actual" != "$expected" ]; then
    vg_say "!!! checksum mismatch for $file"
    vg_say "    expected $expected"
    vg_say "    got      $actual"
    return 1
  fi
}

# vg_consent WHAT URL — 0 if this run may download, 1 if it may not.
vg_consent() {
  local what="$1" url="$2"
  if [ "${VIBEGRAPH_NO_NETWORK:-}" = "1" ]; then
    vg_say "⊘ \$VIBEGRAPH_NO_NETWORK is set; not downloading $what"
    return 1
  fi
  if [ "${VIBEGRAPH_FETCH_CONSENT:-}" = "1" ]; then
    return 0
  fi
  if [ -t 0 ] && [ -t 2 ]; then
    local reply
    printf 'Download %s from %s? [y/N] ' "$what" "$url" >&2
    read -r reply
    case "$reply" in [yY]*) return 0 ;; *) return 1 ;; esac
  fi
  vg_say "⊘ not downloading $what: no terminal to ask at."
  vg_say "  Set VIBEGRAPH_FETCH_CONSENT=1 to allow it, or place the data yourself."
  return 1
}

# vg_download URL DEST WHAT — the only function here that talks to the network.
vg_download() {
  local url="$1" dest="$2" what="$3"
  vg_consent "$what" "$url" || return 1
  vg_say ">>> downloading $what"
  vg_say "    $url"
  curl -sSL --fail -o "$dest.part" "$url"
  mv "$dest.part" "$dest"
}

# ── the three declared inputs ────────────────────────────────────────────────

# The pinned model source. Offline once the object cache exists, so it needs no
# consent step of its own.
vg_ensure_submodule() {
  if [ -f "$VG_REPO_ROOT/research/refs/mg5amcnlo/models/sm/particles.py" ]; then
    return 0
  fi
  vg_say ">>> checking out the mg5amcnlo submodule"
  git -C "$VG_REPO_ROOT" submodule update --init research/refs/mg5amcnlo
}

# vg_ensure_pdf_set [NAME] — an LHAPDF6 set unpacked under validation/pdf/.
vg_ensure_pdf_set() {
  local set_name="${1:-NNPDF23_lo_as_0130_qed}"
  local set_dir="$VG_VALIDATION_DIR/pdf/$set_name"
  if [ -d "$set_dir" ]; then
    vg_say "⊘ $set_name already at $set_dir"
    return 0
  fi
  local url="https://lhapdfsets.web.cern.ch/current/${set_name}.tar.gz"
  local tarball="$VG_VALIDATION_DIR/pdf/${set_name}.tar.gz"
  vg_download "$url" "$tarball" "the $set_name PDF set" || return 1
  tar xzf "$tarball" -C "$VG_VALIDATION_DIR/pdf"
  rm -f "$tarball"
  vg_say "✓ $set_name ready at $set_dir"
}

# Where the bundle unpacks, and the stamp that records which one is unpacked.
vg_refdata_dir() { printf '%s\n' "$VG_VALIDATION_DIR/madgraph/output"; }
vg_refdata_stamp() { printf '%s\n' "$(vg_refdata_dir)/.refdata-stamp"; }

# vg_ensure_refdata — the banked MadGraph reference runs, unpacked into the
# MadGraph work-area layout so a fetched checkout and a machine that generated
# the runs itself present the gates with the same paths.
#
# A work area that already holds process directories is left alone: it is the
# oracle layer's cache, it is a superset of the bundle, and re-unpacking over it
# would replace locally generated files with banked ones of the same name.
#
# VIBEGRAPH_REFDATA_SOURCE overrides where the archive comes from — a local path
# or a file:// URL — for testing the unpack path against a bundle that is not
# published yet. The pinned checksum is still enforced.
vg_ensure_refdata() {
  local dir stamp archive url sha
  dir="$(vg_refdata_dir)"
  stamp="$(vg_refdata_stamp)"
  archive="$(vg_manifest_value refdata archive)"
  url="$(vg_manifest_value refdata url)"
  sha="$(vg_manifest_value refdata sha256)"
  [ -n "$archive" ] && [ -n "$sha" ] || vg_die "no [refdata] pin in $VG_MANIFEST"

  if [ -f "$stamp" ] && [ "$(cat "$stamp")" = "$sha" ]; then
    vg_say "⊘ banked reference data already unpacked at $dir"
    return 0
  fi
  if [ -d "$dir" ] && [ -n "$(find "$dir" -maxdepth 2 -name SubProcesses -print -quit)" ]; then
    vg_say "⊘ $dir already holds MadGraph process directories — leaving it as it is"
    return 0
  fi

  local tarball="${TMPDIR:-/tmp}/$archive" downloaded=0
  local source="${VIBEGRAPH_REFDATA_SOURCE:-}"
  if [ -n "$source" ]; then
    case "$source" in
      file://*) source="${source#file://}" ;;
    esac
    [ -f "$source" ] || vg_die "VIBEGRAPH_REFDATA_SOURCE=$source is not a file"
    vg_say ">>> using the local bundle at $source"
    tarball="$source"
  else
    [ -n "$url" ] || vg_die "no [refdata] url in $VG_MANIFEST and no VIBEGRAPH_REFDATA_SOURCE"
    vg_download "$url" "$tarball" "the banked MadGraph reference bundle ($archive)" || return 1
    downloaded=1
  fi

  if ! vg_verify_sha "$tarball" "$sha"; then
    [ "$downloaded" = 1 ] && rm -f "$tarball"
    vg_die "$archive does not match the pin in $VG_MANIFEST"
  fi

  vg_say ">>> unpacking $archive into $dir"
  mkdir -p "$dir"
  zstd -dc "$tarball" | tar -xf - -C "$dir"
  printf '%s\n' "$sha" > "$stamp"
  vg_say "✓ banked reference data ready at $dir"
}
