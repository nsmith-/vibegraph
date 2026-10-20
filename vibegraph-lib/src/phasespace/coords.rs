//! Where a phase-space map's coordinates come from.
//!
//! A map builds its point one coordinate at a time, and the range a coordinate
//! may take can depend on the coordinates drawn before it: an invariant's
//! kinematic window is set by the invariants and energies already fixed. The map
//! therefore *asks* for each coordinate as it reaches it, naming the sub-range
//! `[lo, hi] ⊆ [0, 1]` of that coordinate's own axis the point may occupy, and a
//! [`Coordinates`] source answers. A source that knows nothing about the axis
//! hands back the window's own fraction of a pre-drawn uniform
//! ([`SliceCoords`]); a VEGAS grid hands back a value shaped by its bins.
//!
//! The contract a source keeps is the one the map's density relies on: the value
//! lies in `[lo, hi]`, and whatever density the source draws it with is measured
//! against the uniform density over that window. The map's own analytic density
//! is then uniform-in-window on its axis, which it can evaluate at a point it did
//! not draw without knowing anything about the source — the property a
//! multichannel mixture needs.

use crate::helas::repr::Real;

/// A stream of coordinates, each drawn on request within the window the caller
/// names.
pub trait Coordinates<F: Real> {
    /// The next coordinate, in `[lo, hi]` with `0 ≤ lo ≤ hi ≤ 1` on its own axis.
    ///
    /// Drawn uniformly over the window by a source with no shape of its own;
    /// a shaping source (a VEGAS grid) carries its density relative to that
    /// uniform one in its own weight.
    fn within(&mut self, lo: F, hi: F) -> F;

    /// The next coordinate over the whole unit interval: the window `[0, 1]`.
    fn unit(&mut self) -> F {
        self.within(F::zero(), F::one())
    }
}

/// The coordinates of a pre-drawn point, read in order: the `i`-th request is
/// answered from `u[i]`, placed at that fraction of the requested window.
///
/// Over the window `[0, 1]` the answer is `u[i]` itself, so a map that asks only
/// for whole-interval coordinates reads exactly the slice it would have indexed.
#[derive(Debug)]
pub struct SliceCoords<'u, F: Real> {
    u: &'u [F],
    next: usize,
}

impl<'u, F: Real> SliceCoords<'u, F> {
    /// Read `u` from its first entry.
    pub fn new(u: &'u [F]) -> Self {
        Self { u, next: 0 }
    }

    /// How many coordinates have been read.
    pub fn consumed(&self) -> usize {
        self.next
    }

    fn take(&mut self) -> F {
        let x = self.u[self.next];
        self.next += 1;
        x
    }
}

impl<F: Real> Coordinates<F> for SliceCoords<'_, F> {
    fn within(&mut self, lo: F, hi: F) -> F {
        let x = self.take();
        lo + (hi - lo) * x
    }

    fn unit(&mut self) -> F {
        self.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole-interval window returns the slice entry bit for bit, through
    /// either request — the property every eager caller relies on.
    #[test]
    fn the_unit_window_is_the_slice_itself() {
        let u = [0.0, 1e-300, 0.123_456_789, 0.5, 1.0 - f64::EPSILON];
        let mut by_unit = SliceCoords::new(&u);
        let mut by_window = SliceCoords::new(&u);
        for &x in &u {
            assert_eq!(by_unit.unit().to_bits(), x.to_bits());
            assert_eq!(by_window.within(0.0, 1.0).to_bits(), x.to_bits());
        }
        assert_eq!(by_unit.consumed(), u.len());
    }

    #[test]
    fn a_window_places_the_coordinate_at_its_fraction() {
        let u = [0.0_f64, 0.25, 1.0];
        let mut c = SliceCoords::new(&u);
        assert_eq!(c.within(0.2, 0.6), 0.2);
        assert!((c.within(0.2, 0.6) - 0.3).abs() < 1e-15);
        assert!((c.within(0.2, 0.6) - 0.6).abs() < 1e-15);
    }

    #[test]
    #[should_panic]
    fn reading_past_the_point_panics() {
        let u = [0.5];
        let mut c = SliceCoords::new(&u);
        c.unit();
        c.unit();
    }
}
