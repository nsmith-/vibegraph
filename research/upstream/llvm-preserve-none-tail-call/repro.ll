; preserve_none on x86-64 passes 12 integer args (r12 r13 r14 r15 rdi rsi rdx rcx r8 r9 r11 rax).
; With all 12 in use, the indirect musttail target is loaded into rax, clobbering arg 12 (%k).
target triple = "x86_64-unknown-linux-gnu"

define preserve_nonecc i64 @f(ptr %fp, i64 %a, i64 %b, i64 %c, i64 %d, i64 %e, i64 %f, i64 %g, i64 %h, i64 %i, i64 %j, i64 %k) {
  %t = load ptr, ptr %fp
  %r = musttail call preserve_nonecc i64 %t(ptr %fp, i64 %a, i64 %b, i64 %c, i64 %d, i64 %e, i64 %f, i64 %g, i64 %h, i64 %i, i64 %j, i64 %k)
  ret i64 %r
}
