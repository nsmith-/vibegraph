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
# One process, `p p > l+ l- j`, carries the whole run: it is hadronic, so it
# resolves a PDF set by name, refuses to fetch it unattended, downloads it
# against the compiled-in checksum pin when told to, and then serves a second
# command from the cache; and it is a process `generate` supports at proton
# beams, so the same cards go on to produce a Les Houches file the binary reads
# back.
#
# Drell–Yan (`p p > e+ e-`) is deliberately not the process here, though the
# binary would run it: its final state carries no colour, so an acceptance run on
# it would exercise neither the colour-flow selection nor the `ICOLUP` lines a
# downstream shower reads.

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
    -h|--help) sed -n '2,32p' "$0"; exit 0 ;;
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

  # Every tarball carries the third-party notice because the interned MG5 SM
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
  # Every run below happens inside the work directory, so a relative `--binary`
  # would stop resolving the moment this script changed directory.
  binary="$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")"
  step "using the binary at $binary"
fi

step "version"
"$binary" --version || fail "--version failed"

# ------------------------------------------------------------------ the cards

# Written here rather than read from the repo: a user with a downloaded binary
# has no checkout, and these are the same cards the README quick start shows.
cards="$work/cards"
mkdir -p "$cards"

cat > "$cards/proc_card.dat" <<'EOF'
import model sm
generate p p > l+ l- j QCD=2 QED=2
EOF

# The scales are fixed rather than dynamical: a 2 -> 3 dynamical scale needs kT
# clustering, which this generator refuses rather than approximates.
cat > "$cards/run_card.dat" <<'EOF'
  1       = lpp1
  1       = lpp2
  6500.0  = ebeam1
  6500.0  = ebeam2
  lhapdf  = pdlabel
  247000  = lhaid
  True    = fixed_ren_scale
  True    = fixed_fac_scale1
  True    = fixed_fac_scale2
  91.188  = scale
  91.188  = dsqrt_q2fact1
  91.188  = dsqrt_q2fact2
  50.0    = mmll
  20.0    = ptj
  10.0    = ptl
  5.0     = etaj
  2.5     = etal
  0.4     = drll
  0.4     = drjl
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

# ------------------------------------------------------------- the four steps

# Budget: small on purpose. This job answers "does the shipped artifact work at
# all", not "is the cross section right" — that is the repository's own gated
# comparison against MadGraph, which runs a far larger budget against a banked
# reference. Here the wall clock should stay dominated by the 27 MB download.
NEVAL=20000
NITER=4
NEVENTS=2000

step "an unattended run refuses to download the PDF set"
# This script has no terminal, so the default policy must refuse — the property
# that keeps a CI job from silently pulling 27 MB. It has to hold on the shipped
# binary, not just in the test suite.
if "$binary" integrate "$cards/proc_card.dat" \
     --run-card "$cards/run_card.dat" --out "$work/refused" \
     --fixed-budget --neval 2000 --niter 2 >"$work/refusal.out" 2>&1; then
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
"$binary" --yes integrate "$cards/proc_card.dat" \
  --run-card "$cards/run_card.dat" --out "$work/llj" \
  --fixed-budget --neval "$NEVAL" --niter "$NITER" 2>&1 | tee "$work/integrate.out" \
  || fail "the hadronic integration failed"
[ -f "$work/llj/grid.bin.zst" ] || fail "no grid artifact was written"
[ -f "$VIBEGRAPH_HOME/pdf/NNPDF23_lo_as_0130_qed/NNPDF23_lo_as_0130_qed_0000.dat" ] \
  || fail "the PDF set is not in the cache where resolution looks for it"
grep -q "^σ = " "$work/integrate.out" || fail "the run printed no cross section"

step "generating events, with the PDF set served from the cache"
# `--no-network`, and generation needs the same PDF set the integration used: if
# the cache did not take, nothing is allowed to fetch it and this fails. That is
# the cache-hit check and the event generation in one command.
"$binary" --no-network generate "$work/llj/grid.bin.zst" "$cards/proc_card.dat" \
  --run-card "$cards/run_card.dat" --nevents "$NEVENTS" --seed 1 \
  -o "$work/events.lhe" 2>&1 | tee "$work/gen.out" \
  || fail "event generation failed (a cached PDF set is part of what this needs)"
[ -s "$work/events.lhe" ] || fail "the event file is empty"

step "reading the events back"
"$binary" check-events "$work/events.lhe" --min-events "$NEVENTS" \
  || fail "the emitted event file did not survive being read back"

printf '\nACCEPTANCE PASSED\n'
printf '  binary:  %s\n' "$("$binary" --version)"
printf '  events:  %s\n' "$work/events.lhe"
