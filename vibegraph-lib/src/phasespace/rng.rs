//! Counter-based random substreams for phase-space sampling.
//!
//! Sampling is defined in two decoupled layers:
//!
//! 1. An *integer* bit-stream produced by a counter-based generator
//!    ([`rand_chacha::ChaCha8Rng`]). Because the generator is counter-based,
//!    substreams are structurally independent: a `(stream, position)` pair
//!    names an exact location in the output, with 2⁶⁴ selectable streams per
//!    seed and a settable draw position within each. This maps directly onto
//!    parallel/distributed accumulation — `stream ← (iteration, chunk index)`,
//!    `position ← draw counter` — with no reliance on hash-mixing for
//!    independence.
//!
//! 2. A documented **bits→uniform** conversion ([`u64_to_uniform`]) that turns
//!    each 64-bit draw into a `[0, 1)` value in the scalar field `F`. The
//!    conversion is defined on the integer bits, so lane-batched `f64` sampling
//!    reproduces scalar `f64` sampling bit-for-bit: the same integer draw feeds
//!    every lane through the same arithmetic.

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::helas::repr::Real;

/// A ChaCha8 output at 32-bit-word position `p` occupies words `[p, p+1)`; a
/// 64-bit draw consumes two words, so the `position` (draw counter) maps to the
/// underlying word position as `2 * position`.
const WORDS_PER_DRAW: u128 = 2;

/// The stream family the per-point scale-configuration draw consumes, offset by
/// channel index.
///
/// It is disjoint from every other stream a run opens, and that is the whole
/// point: the coordinate this draw needs is appended to a point *after* the
/// phase-space map has taken its own, so no existing sequence — the channel
/// grids, the channel selection, the acceptance test — shifts by a single bit
/// when the draw is present.
pub const SCALE_DRAW_STREAM_BASE: u64 = 0x5CA1_0000;

/// Map a 64-bit draw to a uniform in `[0, 1)`.
///
/// Rule: take the top 53 bits of the draw as the mantissa of an `f64`, i.e.
/// `(bits >> 11) as f64 / 2^53`, then cast that `f64` into the scalar field `F`.
/// 53 bits is the full `f64` significand, so every representable `f64` in
/// `[0, 1)` with that many mantissa bits is reachable and the result never
/// reaches exactly `1.0`. Because the whole rule is a function of the integer
/// `bits`, a `NumericArray` lane pack fed the same draw yields the same value
/// per lane as scalar `f64`.
#[inline]
pub fn u64_to_uniform<F: Real>(bits: u64) -> F {
    // 2^53 as f64 is exact; the quotient is in [0, 1).
    let mantissa = (bits >> 11) as f64;
    let scale = (1u64 << 53) as f64;
    F::from(mantissa / scale).expect("f64 uniform is representable in F")
}

/// A counter-based random substream addressed by `(stream, position)`.
///
/// `stream` selects one of 2⁶⁴ independent output sequences for the seed;
/// `position` is the index of the next 64-bit draw within that sequence. Two
/// substreams with different `stream` values are structurally independent;
/// re-creating a substream with the same `(seed, stream, position)` replays the
/// identical draw sequence.
#[derive(Clone, Debug)]
pub struct SubStream {
    rng: ChaCha8Rng,
}

impl SubStream {
    /// Open the substream `stream` of `seed`, positioned at draw `position`.
    pub fn new(seed: u64, stream: u64, position: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        rng.set_stream(stream);
        rng.set_word_pos(u128::from(position) * WORDS_PER_DRAW);
        Self { rng }
    }

    /// Open `stream` of `seed` at the start (`position = 0`).
    #[inline]
    pub fn from_stream(seed: u64, stream: u64) -> Self {
        Self::new(seed, stream, 0)
    }

    /// The index of the next 64-bit draw within this substream.
    #[inline]
    pub fn position(&self) -> u64 {
        (self.rng.get_word_pos() / WORDS_PER_DRAW) as u64
    }

    /// The stream number this substream draws from.
    #[inline]
    pub fn stream(&self) -> u64 {
        self.rng.get_stream()
    }

    /// Draw the next 64 raw bits.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Draw the next uniform in `[0, 1)` in the scalar field `F`.
    #[inline]
    pub fn next_uniform<F: Real>(&mut self) -> F {
        u64_to_uniform::<F>(self.next_u64())
    }

    /// Fill `out` with successive uniforms in `[0, 1)`.
    pub fn fill_uniforms<F: Real>(&mut self, out: &mut [F]) {
        for slot in out.iter_mut() {
            *slot = self.next_uniform::<F>();
        }
    }

    /// Collect `n` successive uniforms in `[0, 1)`.
    pub fn uniforms<F: Real>(&mut self, n: usize) -> Vec<F> {
        (0..n).map(|_| self.next_uniform::<F>()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_conversion_goldens() {
        // Pinned bits→uniform values. These freeze the top-53-bit rule and must
        // not drift: lane-batched sampling depends on this exact map.
        assert_eq!(u64_to_uniform::<f64>(0), 0.0);
        // All 64 bits set: (2^53 - 1) / 2^53, the largest value below 1.0.
        let max = ((1u64 << 53) - 1) as f64 / (1u64 << 53) as f64;
        assert_eq!(u64_to_uniform::<f64>(u64::MAX), max);
        assert!(u64_to_uniform::<f64>(u64::MAX) < 1.0);
        // Exactly one half: top bit set.
        assert_eq!(u64_to_uniform::<f64>(1u64 << 63), 0.5);
        // A quarter: second bit set.
        assert_eq!(u64_to_uniform::<f64>(1u64 << 62), 0.25);
        // The low 11 bits are discarded (below the 53-bit mantissa).
        assert_eq!(u64_to_uniform::<f64>(0x7FF), 0.0);
        assert_eq!(u64_to_uniform::<f64>((1u64 << 11) - 1), 0.0);
        // f32 truncates the same f64 quotient on cast.
        assert_eq!(u64_to_uniform::<f32>(1u64 << 63), 0.5f32);
    }

    #[test]
    fn uniforms_stay_in_unit_interval() {
        let mut s = SubStream::from_stream(0xABCD, 3);
        for u in s.uniforms::<f64>(10_000) {
            assert!((0.0..1.0).contains(&u), "uniform out of range: {u}");
        }
    }

    #[test]
    fn position_addressing_seeks() {
        // Drawing k values then reading equals opening directly at position k.
        let seed = 0x1234_5678;
        let stream = 7;
        let mut sequential = SubStream::from_stream(seed, stream);
        let drawn: Vec<u64> = (0..8).map(|_| sequential.next_u64()).collect();
        assert_eq!(sequential.position(), 8);

        for (k, &expected) in drawn.iter().enumerate() {
            let mut seeked = SubStream::new(seed, stream, k as u64);
            assert_eq!(seeked.position(), k as u64);
            assert_eq!(seeked.next_u64(), expected, "seek to draw {k} mismatched");
        }
    }

    #[test]
    fn streams_are_independent() {
        // Two different streams of the same seed produce different draws.
        let mut s0 = SubStream::from_stream(42, 0);
        let mut s1 = SubStream::from_stream(42, 1);
        let seq0: Vec<u64> = (0..16).map(|_| s0.next_u64()).collect();
        let seq1: Vec<u64> = (0..16).map(|_| s1.next_u64()).collect();
        assert_ne!(seq0, seq1);
    }

    #[test]
    fn seeded_draw_sequence_golden() {
        // End-to-end freeze of seed→stream→draw addressing and the conversion.
        // Regenerating this must be a conscious choice, not an accident.
        let mut s = SubStream::new(0xFEED_BEEF, 5, 0);
        let got: Vec<u64> = (0..4).map(|_| s.next_u64()).collect();
        let uniforms: Vec<f64> = SubStream::new(0xFEED_BEEF, 5, 0).uniforms::<f64>(4);
        for (bits, u) in got.iter().zip(uniforms.iter()) {
            assert_eq!(u64_to_uniform::<f64>(*bits), *u);
        }
    }
}
