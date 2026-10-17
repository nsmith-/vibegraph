#!/usr/bin/env bash
# Runs four hand-written bodies of repro.ll's @f under a preserve_none caller and
# callee compiled by llc. Expected: llc's own output (rax) prints garbage; the
# r10, rbx and memory-indirect variants print 66.
# Usage: LLC=/path/to/llc ./check-valid-code.sh   (needs an x86-64 Linux host and gcc)
set -euo pipefail
LLC=${LLC:-llc}
dir=$(mktemp -d)
cd "$dir"
cat > driver.ll <<'EOF'
target triple = "x86_64-unknown-linux-gnu"
declare preserve_nonecc i64 @f(ptr, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64)
define preserve_nonecc i64 @callee(ptr %fp, i64 %a, i64 %b, i64 %c, i64 %d, i64 %e, i64 %f, i64 %g, i64 %h, i64 %i, i64 %j, i64 %k) {
  %s1 = add i64 %a, %b
  %s2 = add i64 %s1, %c
  %s3 = add i64 %s2, %d
  %s4 = add i64 %s3, %e
  %s5 = add i64 %s4, %f
  %s6 = add i64 %s5, %g
  %s7 = add i64 %s6, %h
  %s8 = add i64 %s7, %i
  %s9 = add i64 %s8, %j
  %s10 = add i64 %s9, %k
  ret i64 %s10
}
@slot = global ptr @callee
define i64 @run() {
  %r = call preserve_nonecc i64 @f(ptr @slot, i64 1, i64 2, i64 3, i64 4, i64 5, i64 6, i64 7, i64 8, i64 9, i64 10, i64 11)
  ret i64 %r
}
EOF
cat > main.c <<'EOF'
#include <stdio.h>
long run(void);
int main(void) { long r = run(); printf("%ld (expected 66)\n", r); return r != 66; }
EOF
"$LLC" -O2 -relocation-model=pic -filetype=obj driver.ll -o driver.o
for v in rax r10 rbx mem; do
  case $v in
    mem) body='jmpq *(%r12)' ;;
    *) body="movq (%r12), %$v; jmpq *%$v" ;;
  esac
  printf '.text\n.globl f\n.type f,@function\nf:\n  %s\n.section .note.GNU-stack,"",@progbits\n' "$body" > f_$v.s
  gcc -o t_$v main.c driver.o f_$v.s
  printf '%-30s -> %s\n' "$body" "$(./t_$v || true)"
done
