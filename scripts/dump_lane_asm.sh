#!/usr/bin/env bash
# Usage: [RUSTFLAGS=...] dump_lane_asm.sh [symbol-regex]
#
# Confirms whether the SIMD lane-batched eval path (F = NumericArray<f64, N>)
# actually compiles to packed vector instructions. Builds the eval_strategies
# bench (honoring RUSTFLAGS from the environment, e.g.
#     RUSTFLAGS="-C target-cpu=native" scripts/dump_lane_asm.sh
# ), disassembles the bench binary, extracts the hot-path eval functions
# (Lorentz kernels, external-wavefunction builders, the interpreter loop, and
# the lane entry points), and prints a per-function instruction census:
#
#   x86-64:  packed-double mnemonics (`...pd`: vmulpd, vsqrtpd, ...) and the
#            xmm/ymm/zmm register classes touched. Packed `pd` ops on zmm are
#            genuine 8-lane AVX-512; `...sd` on xmm is scalar math (xmm alone
#            is not a SIMD signal — all x86-64 float math lives in xmm).
#   aarch64: `.2d` vector-arrangement instructions (2-lane f64 NEON).
#
# Reading the table: legacy Rust mangling omits generic arguments from symbol
# names, so each kernel appears once per monomorphization, distinguished only
# by hash suffix — typically four instances (f64 scalar + lanes N=2,4,8, per
# benches/eval_strategies.rs). The lane instances are the ones with high
# packed-instruction counts and the widest register class; the scalar sibling
# doubles as a baseline in the same table. (On aarch64 even the f64 instance
# shows some `.2d` — LLVM SLP-packs Complex<f64> re/im pairs — so compare
# densities, not zero-vs-nonzero.)
#
# Output: summary table on stdout (sorted by vector-instruction count); full
# disassembly of every matched function in target/lane-asm/lane_functions.asm;
# complete binary disassembly in target/lane-asm/full.asm; the whole census in
# target/lane-asm/census.tsv.
#
# An optional symbol-regex (awk ERE) replaces the default hot-path selection,
# e.g. `scripts/dump_lane_asm.sh 'ffv_vout|vxxxxx'`.
set -euo pipefail

PATTERN="${1:-helas::eval::kernel::|helas::eval::run::(eval_m2|apply|fill_arenas)|helas::wavefn::|repr::lorentz::weyl_ixxxxx|NumericArray}"
OUT_DIR="target/lane-asm"

BUILD_OUTPUT=$(cargo bench -p vibegraph-lib --bench eval_strategies --no-run 2>&1)
echo "$BUILD_OUTPUT"

EXECUTABLE=$(echo "$BUILD_OUTPUT" | grep 'Executable' | tail -1 | sed -n 's/.*Executable[^(]*(//;s/).*//p')
if [[ -z "$EXECUTABLE" ]]; then
    echo "error: could not parse bench executable path from cargo output" >&2
    exit 1
fi
echo "bench binary: $EXECUTABLE"

OBJDUMP=$(command -v llvm-objdump || command -v objdump || true)
if [[ -z "$OBJDUMP" ]]; then
    echo "error: neither llvm-objdump nor objdump found in PATH" >&2
    exit 1
fi

mkdir -p "$OUT_DIR"
: > "$OUT_DIR/lane_functions.asm"
"$OBJDUMP" -d --demangle "$EXECUTABLE" > "$OUT_DIR/full.asm"
echo "full disassembly: $OUT_DIR/full.asm ($(wc -l < "$OUT_DIR/full.asm" | tr -d ' ') lines)"

# Split on `addr <symbol>:` headers (GNU objdump and llvm-objdump, ELF and
# Mach-O), keep functions matching the pattern, and census each one's
# instructions.
awk -v pattern="$PATTERN" -v fnfile="$OUT_DIR/lane_functions.asm" '
function flush() {
    if (!collecting || instrs == 0) return
    printf "%s\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n", \
        name, instrs, packed_pd, scalar_sd, xmm, ymm, zmm, neon2d
}
/^[0-9a-f]+ <.*>:[ \t]*$/ {
    flush()
    name = $0
    sub(/^[0-9a-f]+ </, "", name); sub(/>:[ \t]*$/, "", name)
    collecting = (name ~ pattern)
    instrs = packed_pd = scalar_sd = xmm = ymm = zmm = neon2d = 0
    if (collecting) print "\n" $0 >> fnfile
    next
}
collecting {
    print >> fnfile
    if ($0 !~ /^[ \t]*[0-9a-f]+:/) next
    instrs++
    if ($0 ~ /[ \t]v?[a-z][a-z0-9]*pd[ \t]/)  packed_pd++
    if ($0 ~ /[ \t]v?[a-z][a-z0-9]*sd[ \t]/)  scalar_sd++
    if ($0 ~ /xmm[0-9]/)  xmm++
    if ($0 ~ /ymm[0-9]/)  ymm++
    if ($0 ~ /zmm[0-9]/)  zmm++
    if ($0 ~ /\.2d/)      neon2d++
}
END { flush() }
' "$OUT_DIR/full.asm" > "$OUT_DIR/census.tsv"

if [[ ! -s "$OUT_DIR/census.tsv" ]]; then
    echo "no functions matched pattern '$PATTERN'" >&2
    exit 1
fi

echo
echo "eval hot-path functions (matched disasm: $OUT_DIR/lane_functions.asm; full census: $OUT_DIR/census.tsv)"
echo "top 40 by vector-instruction count; packed_pd on zmm = 8-lane AVX-512, on ymm = 4-lane AVX, .2d = 2-lane NEON"
{
    printf "vector\tinstrs\tpacked_pd\tscalar_sd\txmm\tymm\tzmm\tneon.2d\tfunction\n"
    awk -F'\t' '{ v = $3 + $8; printf "%d\t%s\n", v, $0 }' "$OUT_DIR/census.tsv" \
        | sort -t$'\t' -k1,1nr -k3,3nr | head -40 \
        | awk -F'\t' '{ printf "%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%.110s\n", $1,$3,$4,$5,$6,$7,$8,$9,$2 }'
} | column -t -s$'\t'

echo
awk -F'\t' '
{ fns++; instrs += $2; pd += $3; sd += $4; ymm += $6; zmm += $7; n2d += $8 }
END {
    printf "TOTAL: %d functions, %d instructions, %d packed-pd, %d scalar-sd, %d neon-.2d lines\n", \
        fns, instrs, pd, sd, n2d
    if (zmm > 0)      print "verdict: zmm registers present -> 512-bit (8x f64) vectors are being emitted"
    else if (ymm > 0) print "verdict: ymm but no zmm -> 256-bit (4x f64) max; on AVX-512 silicon this is LLVM prefer-256-bit tuning (try -C target-cpu=znver4/znver5 or x86-64-v4)"
    else if (n2d > 0) print "verdict: NEON .2d arrangements present -> 128-bit (2x f64) vectors"
    else if (pd > 0)  print "verdict: packed-pd on xmm only -> 128-bit (2x f64) SSE vectors"
    else              print "verdict: no packed f64 instructions -> the lane element loops are compiling to scalar code"
}' "$OUT_DIR/census.tsv"
