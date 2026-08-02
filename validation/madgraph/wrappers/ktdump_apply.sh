#!/usr/bin/env bash
# Install the kT-clustering instrumentation into one generated MadGraph process
# directory. Nothing here touches the pinned submodule: MadGraph copies its
# clustering sources into every directory it generates, so the patches apply to
# those copies, the way the amplitude probes patch the generated matrix1_orig.f.
#
# The patched sources are byte-identical to Template/LO's before the patch runs
# — the check below is what makes that a fact rather than an assumption, so a
# submodule bump that moves any of these files fails here instead of silently
# producing a dump of something else.
#
# Usage: bash validation/madgraph/wrappers/ktdump_apply.sh <process directory>
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TEMPLATE="$ROOT/research/refs/mg5amcnlo/Template/LO"
PDIR="${1:?usage: ktdump_apply.sh <process directory>}"

[ -d "$PDIR/SubProcesses" ] || { echo "!!! $PDIR is not a process directory" >&2; exit 1; }

# file:relative-source-path — where the patched copy lives in the process
# directory and where its pristine original lives in the template.
declare -a TARGETS=(
  "SubProcesses/cluster.f:SubProcesses/cluster.f"
  "SubProcesses/reweight.f:SubProcesses/reweight.f"
  "SubProcesses/initcluster.f:SubProcesses/initcluster.f"
  "SubProcesses/unwgt.f:SubProcesses/unwgt.f"
  "Source/kin_functions.f:Source/kin_functions.f"
)

for spec in "${TARGETS[@]}"; do
  rel="${spec%%:*}"
  src="${spec#*:}"
  target="$PDIR/$rel"
  pristine="$TEMPLATE/$src"
  patch_file="$HERE/ktdump_$(basename "$rel").patch"

  [ -f "$target" ] || { echo "!!! missing $target" >&2; exit 1; }
  [ -f "$patch_file" ] || { echo "!!! missing patch $patch_file" >&2; exit 1; }
  cmp -s "$target" "$pristine" || {
    echo "!!! $rel in the process directory is not the pinned template's copy" >&2
    exit 1
  }
  patch --silent "$target" < "$patch_file" || {
    echo "!!! failed to apply $patch_file to $target" >&2
    exit 1
  }
done

# The dump writer, and the buffer it appends to. Only SubProcesses compiles the
# writer; Source needs the buffer's declaration alone, because the one thing
# patched there records a branch index into it and calls nothing.
cp "$HERE/ktdump.f" "$PDIR/SubProcesses/ktdump.f"
cp "$HERE/ktdump.inc" "$PDIR/SubProcesses/ktdump.inc"
cp "$HERE/ktdump.inc" "$PDIR/Source/ktdump.inc"

# SubProcesses/makefile names its objects rather than globbing them, so the new
# one is registered by hand. Every subprocess directory symlinks that makefile.
mk="$PDIR/SubProcesses/makefile"
grep -q 'ktdump.o' "$mk" || perl -pi -e \
  's/^(PROCESS= )/$1ktdump.o /' "$mk"
grep -q 'ktdump.o' "$mk" || { echo "!!! could not register ktdump.o in $mk" >&2; exit 1; }

# Each subprocess directory reaches its shared sources through symlinks that
# MadGraph laid down when it generated the directory; the new ones need theirs.
for sub in "$PDIR"/SubProcesses/P*/; do
  [ -d "$sub" ] || continue
  ln -sf ../ktdump.f "$sub/ktdump.f"
  ln -sf ../ktdump.inc "$sub/ktdump.inc"
done

echo "    instrumented $PDIR"
