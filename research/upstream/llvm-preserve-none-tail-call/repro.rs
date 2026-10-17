// x86-64 `preserve_none` + indirect `become` with all 12 argument registers in use.
// Expected output: 66 (sum of a..l over the final call). Correct with 11 args.
#![feature(rust_preserve_none_cc, explicit_tail_calls)]
#![allow(incomplete_features)]

type H = extern "rust-preserve-none" fn(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64) -> u64;
static TABLE: [H; 2] = [step, done];

#[inline(never)]
extern "rust-preserve-none" fn step(n: u64, a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64, h: u64, i: u64, j: u64, k: u64) -> u64 {
    let t = std::hint::black_box(&TABLE)[(n == 1) as usize];
    become t(n - 1, a, b, c, d, e, f, g, h, i, j, k)
}

#[inline(never)]
extern "rust-preserve-none" fn done(n: u64, a: u64, b: u64, c: u64, d: u64, e: u64, f: u64, g: u64, h: u64, i: u64, j: u64, k: u64) -> u64 {
    n + a + b + c + d + e + f + g + h + i + j + k
}

fn main() {
    let r = step(std::hint::black_box(4), 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
    println!("{r} (expected 66)");
    assert_eq!(r, 66);
}
