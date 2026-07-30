#!/usr/bin/env bash
# Usage: acceptance.sh [--binary PATH] [--tag TAG] [--repo OWNER/REPO] [--keep]
#
# The end-to-end acceptance run: take a released vibegraph binary, hand it
# nothing but process and run cards, and require that it comes back with a Les
# Houches event file it can read again.
#
# Nothing here needs a checkout, a Rust toolchain, a Python installation or a
# pre-fetched PDF set. Everything it consumes it either downloads (the binary
# from a GitHub release, the PDF set from CERN through the binary's own pinned
# fetch) or writes itself (the cards, below). That is the point: it reproduces
# what a user with a fresh machine actually does. `curl`, `tar` and a POSIX
# shell are the whole dependency list; `sha256sum`/`shasum` is used when
# present to verify the release asset.
#
# It downloads ~27 MB of PDF grids on every run, deliberately. The download is
# the path under test — a cached PDF set would skip exactly the code this exists
# to exercise — so `$VIBEGRAPH_HOME` points at a scratch directory rather than
# the caller's real cache.
#
# Two legs, because the pipeline is not yet one:
#
#   hadronic  `p p > e+ e-` — `integrate` only. This is the leg that resolves a
#             PDF set by name, prompts for it, downloads it against the
#             compiled-in checksum pin, and serves the second run from cache.
#             Event generation for proton beams is not wired yet.
#   events    `e+ e- > mu+ mu-` — `integrate`, `generate`, `check-events`. This
#             is the leg that produces and re-reads a file, on a process that
#             needs no PDF set at all.
#
# The two collapse into one when proton-beam event generation lands: a single
# `p p > e+ e- j` run doing both halves. Until then, "cards in, `.lhe` out" and
# "a PDF set arrives over the network" are demonstrated side by side rather than
# in one command.

set -euo pipefail

repo="nsmith-/vibegraph"
tag=""
binary=""
keep=0

while [ $# -gt 0 ]; do
  case "$1" in
    --binary) binary="$2"; shift 2 ;;
    --tag)    tag="$2";    shift 2 ;;
    --repo)   repo="$2";   shift 2 ;;
    --keep)   keep=1;      shift ;;
    -h|--help) sed -n '2,35p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

work="$(mktemp -d)"
cleanup() {
  if [ "$keep" -eq 1 ]; then
    echo "work directory kept at $work"
  else
    rm -rf "$work"
  fi
}
trap cleanup EXIT

step() { printf '\n=== %s\n' "$*"; }
fail() { printf 'ACCEPTANCE FAILED: %s\n' "$*" >&2; exit 1; }

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d' ' -f1
  else
    echo ""
  fi
}

# ---------------------------------------------------------------- the binary

# The release asset names are the Rust target triples release.yml builds.
release_target() {
  case "$(uname -s)/$(uname -m)" in
    Darwin/arm64)  echo "aarch64-apple-darwin" ;;
    Darwin/x86_64) echo "x86_64-apple-darwin" ;;
    Linux/x86_64)  echo "x86_64-unknown-linux-musl" ;;
    *) fail "no release binary is published for $(uname -s)/$(uname -m)" ;;
  esac
}

download_release() {
  local target asset base
  target="$(release_target)"
  asset="vibegraph-${target}.tar.gz"
  if [ -n "$tag" ]; then
    base="https://github.com/${repo}/releases/download/${tag}"
  else
    # GitHub redirects this to whatever the newest release is, which is what a
    # user following the README gets.
    base="https://github.com/${repo}/releases/latest/download"
  fi

  step "downloading $asset from ${tag:-the latest release}"
  curl -fsSL --retry 3 -o "$work/$asset" "$base/$asset" \
    || fail "cannot download $base/$asset (is there a published release?)"

  # The release publishes a SHA256SUMS alongside its assets; check it when both
  # it and a hashing tool are available, and say so plainly when they are not.
  if curl -fsSL --retry 3 -o "$work/SHA256SUMS" "$base/SHA256SUMS" 2>/dev/null; then
    local expected actual
    expected="$(grep -E "[ *]${asset}\$" "$work/SHA256SUMS" | cut -d' ' -f1 || true)"
    actual="$(sha256_of "$work/$asset")"
    if [ -n "$expected" ] && [ -n "$actual" ]; then
      [ "$expected" = "$actual" ] || fail "release asset checksum mismatch: expected $expected, got $actual"
      echo "release asset checksum verified: $actual"
    else
      echo "note: could not verify the release asset checksum (no entry or no hashing tool)"
    fi
  else
    echo "note: the release publishes no SHA256SUMS; asset not verified"
  fi

  tar xzf "$work/$asset" -C "$work"
  binary="$work/vibegraph-${target}/vibegraph"
  [ -x "$binary" ] || fail "the tarball contains no executable at vibegraph-${target}/vibegraph"

  # U1 puts the third-party notice in every tarball because the interned MG5 SM
  # model is redistributed inside the binary; a tarball without it is a release
  # that should not have shipped.
  [ -f "$work/vibegraph-${target}/THIRD-PARTY-NOTICES" ] \
    || fail "the tarball is missing THIRD-PARTY-NOTICES"
  echo "THIRD-PARTY-NOTICES present in the tarball"
}

if [ -z "$binary" ]; then
  download_release
else
  [ -x "$binary" ] || fail "$binary is not executable"
  step "using the binary at $binary"
fi

step "version"
"$binary" --version || fail "--version failed"

# ------------------------------------------------------------------ the cards

# Written here rather than read from the repo: a user with a downloaded binary
# has no checkout, and these are the same cards the README quick start shows.
cards="$work/cards"
mkdir -p "$cards"

cat > "$cards/dy_proc_card.dat" <<'EOF'
import model sm
generate p p > e+ e-
EOF

cat > "$cards/dy_run_card.dat" <<'EOF'
  1       = lpp1
  1       = lpp2
  6500.0  = ebeam1
  6500.0  = ebeam2
  lhapdf  = pdlabel
  247000  = lhaid
  True    = fixed_ren_scale
  True    = fixed_fac_scale
  91.1880 = scale
  91.1880 = dsqrt_q2fact1
  91.1880 = dsqrt_q2fact2
  10.0    = ptl
  2.5     = etal
  0.4     = drll
EOF

cat > "$cards/ee_proc_card.dat" <<'EOF'
import model sm
generate e+ e- > mu+ mu-
EOF

cat > "$cards/ee_run_card.dat" <<'EOF'
  0    = lpp1
  0    = lpp2
  45.6 = ebeam1
  45.6 = ebeam2
EOF

# A scratch cache root, so this never reads or writes the caller's real
# ~/.vibegraph and every run starts from "nothing is cached".
export VIBEGRAPH_HOME="$work/home"
unset VIBEGRAPH_PDF_DIR VIBEGRAPH_UFO_DIR VIBEGRAPH_NO_NETWORK
mkdir -p "$VIBEGRAPH_HOME"
echo "cache root: $VIBEGRAPH_HOME"

# Every run happens somewhere with no `validation/pdf` beside it, so the dev
# fallback cannot quietly satisfy a resolution that ought to reach the cache.
cd "$work"

# ------------------------------------------------- hadronic leg: PDF fetching

step "an unattended run refuses to download the PDF set"
# This script has no terminal, so the default policy must refuse — the property
# that keeps a CI job from silently pulling 27 MB. It has to hold on the shipped
# binary, not just in the test suite.
if "$binary" integrate "$cards/dy_proc_card.dat" \
     --run-card "$cards/dy_run_card.dat" --out "$work/dy" \
     --neval 2000 --niter 2 >"$work/refusal.out" 2>&1; then
  fail "the binary downloaded a PDF set without being asked"
fi
grep -q -- "--yes" "$work/refusal.out" \
  || fail "the refusal does not name the flag that would allow the download"
grep -q "lhapdfsets.web.cern.ch" "$work/refusal.out" \
  || fail "the refusal does not name the URL it would have fetched"
[ ! -d "$VIBEGRAPH_HOME/pdf/NNPDF23_lo_as_0130_qed" ] \
  || fail "a refused fetch left a cache entry behind"
echo "refused, naming --yes and the URL"

step "with consent, the PDF set is downloaded, verified and cached"
"$binary" --yes integrate "$cards/dy_proc_card.dat" \
  --run-card "$cards/dy_run_card.dat" --out "$work/dy" \
  --neval 20000 --niter 4 2>&1 | tee "$work/dy.out" \
  || fail "the hadronic integration failed"
[ -f "$work/dy/grid.bin.zst" ] || fail "no grid artifact was written"
[ -f "$VIBEGRAPH_HOME/pdf/NNPDF23_lo_as_0130_qed/NNPDF23_lo_as_0130_qed_0000.dat" ] \
  || fail "the PDF set is not in the cache where resolution looks for it"
grep -q "^σ = " "$work/dy.out" || fail "the run printed no cross section"

step "a second run is served from the cache, with no consent needed"
# No --yes this time: if the cache did not take, the default policy refuses and
# this fails — which is exactly the signal wanted.
"$binary" integrate "$cards/dy_proc_card.dat" \
  --run-card "$cards/dy_run_card.dat" --out "$work/dy2" \
  --neval 2000 --niter 2 >"$work/dy2.out" 2>&1 \
  || fail "the cached PDF set did not serve a second run"
if grep -qi "downloading" "$work/dy2.out"; then
  fail "the second run downloaded again"
fi
echo "served from cache"

# ------------------------------------------------- events leg: cards to .lhe

step "integrating a fixed-energy process"
"$binary" --no-network integrate "$cards/ee_proc_card.dat" \
  --run-card "$cards/ee_run_card.dat" --out "$work/ee" \
  --neval 20000 --niter 4 2>&1 | tee "$work/ee.out" \
  || fail "the fixed-energy integration failed"
[ -f "$work/ee/grid.bin.zst" ] || fail "no grid artifact was written"

step "generating events"
"$binary" --no-network generate "$work/ee/grid.bin.zst" "$cards/ee_proc_card.dat" \
  --run-card "$cards/ee_run_card.dat" --nevents 2000 --seed 1 \
  -o "$work/events.lhe" 2>&1 | tee "$work/gen.out" \
  || fail "event generation failed"
[ -s "$work/events.lhe" ] || fail "the event file is empty"

step "reading the events back"
"$binary" check-events "$work/events.lhe" --min-events 2000 \
  || fail "the emitted event file did not survive being read back"

printf '\nACCEPTANCE PASSED\n'
printf '  binary:  %s\n' "$("$binary" --version)"
printf '  events:  %s\n' "$work/events.lhe"
