//! Initial-state kinematics: the two beam momenta, the invariant they collide
//! at, and the boost between the laboratory and the partonic centre of mass.
//!
//! Everything the initial state contributes to a fixed-energy run is a function
//! of the beam energies and the two pole masses, so it is derived once here and
//! read by both the integrand and the per-diagram channel maps. A map whose
//! beams disagreed with the integrand's would sample a different process than
//! the one being evaluated, and the disagreement is invisible in `|M|²`.

use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::Real;

/// Källén function, evaluated as `(a−b−c)² − 4bc` rather than as the expanded
/// `a²+b²+c²−2(ab+bc+ca)`.
///
/// The expanded form adds and subtracts terms of order `a²` to reach a result that
/// can be many orders smaller — a soft emission leaves `λ(ŝ, 0, ŝ_rest)` at `10` out
/// of terms of order `10¹⁰`, so the answer carries only a few correct digits, and a
/// one-ulp change in `ŝ_rest` moves `√λ` by `1e-6`. That is not merely inaccurate:
/// the sampler's walk and the density evaluate it at inputs that differ in the last
/// ulp (one drew the invariant, the other rebuilt it from momenta), so an
/// ill-conditioned `λ` makes the two describe measurably different maps. The
/// grouped form cancels once, at `a−b−c`, and holds the same configuration to
/// `1e-11`.
///
/// It is not unconditionally stable: at the two-body threshold `(a−b−c)²` and `4bc`
/// approach each other and cancel in turn. That regime is where `λ → 0` and the
/// LIPS factor it feeds vanishes with it, so the error rides a weight going to zero.
pub(crate) fn kallen<F: Real>(a: F, b: F, c: F) -> F {
    let four = F::from(4).expect("4 fits the scalar field");
    let d = a - b - c;
    d * d - four * b * c
}

/// The two incoming beam four-momenta in the CM frame at `sqrt_s` for beam masses
/// `ma`, `mb`: beam `0` along `+z`, beam `1` along `−z`, both on shell.
pub(crate) fn beam_momenta<F: Real>(sqrt_s: F, ma: F, mb: F) -> [LorentzVector<F>; 2] {
    beam_momenta_m2(sqrt_s, ma * ma, mb * mb)
}

/// [`beam_momenta`] taking invariants rather than masses, so the incoming line can
/// be *spacelike* (`ma2 < 0`) — which is what an interior rung of a peripheral
/// chain scatters. A negative `ma2` gives `e_a < |k|`, the sign that the line is off
/// shell in the spacelike direction; nothing downstream assumes otherwise.
pub(crate) fn beam_momenta_m2<F: Real>(sqrt_s: F, ma2: F, mb2: F) -> [LorentzVector<F>; 2] {
    let two = F::one() + F::one();
    let s = sqrt_s * sqrt_s;
    let e_a = (s + ma2 - mb2) / (two * sqrt_s);
    let e_b = (s + mb2 - ma2) / (two * sqrt_s);
    let k = kallen(s, ma2, mb2).max(F::zero()).sqrt() / (two * sqrt_s);
    [
        LorentzVector::new(e_a, F::zero(), F::zero(), k),
        LorentzVector::new(e_b, F::zero(), F::zero(), -k),
    ]
}

/// The two beams on shell in the laboratory frame: energy `e[i]` from the run
/// card, mass `m[i]` from the model, momentum along `+z` for beam `0` and `−z`
/// for beam `1`.
pub(crate) fn lab_beam_momenta<F: Real>(e: [F; 2], m: [F; 2]) -> [LorentzVector<F>; 2] {
    let k = |i: usize| (e[i] * e[i] - m[i] * m[i]).max(F::zero()).sqrt();
    [
        LorentzVector::new(e[0], F::zero(), F::zero(), k(0)),
        LorentzVector::new(e[1], F::zero(), F::zero(), -k(1)),
    ]
}

/// The partonic invariant `ŝ = (p_a + p_b)²` of two beams collided head-on at
/// laboratory energies `e` with pole masses `m`:
/// `ŝ = m_a² + m_b² + 2(E_a E_b + |p_a||p_b|)`, which collapses to `(E_a + E_b)²`
/// only when both masses vanish.
pub(crate) fn partonic_s<F: Real>(e: [F; 2], m: [F; 2]) -> F {
    let [pa, pb] = lab_beam_momenta(e, m);
    let two = F::one() + F::one();
    m[0] * m[0] + m[1] * m[1] + two * (pa.e() * pb.e() - pa.pz() * pb.pz())
}

/// The `z` velocity of the two-beam system in the laboratory, `p³/p⁰` of the beam
/// sum, so a boost by it carries a partonic-CM momentum into the laboratory. Its
/// rapidity `artanh β = ½ ln((p⁰+p³)/(p⁰−p³))` is the shift between a rapidity
/// measured in the two frames, and it vanishes for beams of equal energy and mass.
pub(crate) fn lab_beta_z<F: Real>(e: [F; 2], m: [F; 2]) -> F {
    let [pa, pb] = lab_beam_momenta(e, m);
    (pa.pz() + pb.pz()) / (pa.e() + pb.e())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The banked `p3 r3` run: 250 + 250 GeV beams of mass 60 and 70, whose
    /// MadGraph event record carries `E = 248.69635 / 251.29639` and
    /// `pz = ±241.35011` — so even `√ŝ` is 499.99275 and not 500.
    #[test]
    fn the_massive_beam_invariant_matches_madgraphs_banked_record() {
        let e = [250.0_f64, 250.0];
        let m = [60.0, 70.0];
        let s = partonic_s(e, m);
        let sqrt_s = s.sqrt();
        assert!(
            (sqrt_s - 499.99275).abs() < 5e-6,
            "sqrt(s-hat) = {sqrt_s}, banked 499.99275"
        );
        let [a, b] = beam_momenta(sqrt_s, m[0], m[1]);
        assert!((a.e() - 248.69635).abs() < 5e-6, "E_a = {}", a.e());
        assert!((b.e() - 251.29639).abs() < 5e-6, "E_b = {}", b.e());
        assert!((a.pz() - 241.35011).abs() < 5e-6, "|p*| = {}", a.pz());
        assert!((b.pz() + 241.35011).abs() < 5e-6, "|p*| = {}", -b.pz());
        // Both on their own mass shells, and their sum at rest at the invariant.
        assert!((a.m2() - 3600.0).abs() < 1e-6);
        assert!((b.m2() - 4900.0).abs() < 1e-6);
        let total = a + b;
        assert!((total.e() - sqrt_s).abs() < 1e-9);
        assert!(total.pz().abs() < 1e-9);
    }

    /// The banked `qt qt~` run: equal masses, so the beams split `√ŝ` evenly and
    /// the lab and centre-of-mass frames coincide, but `|p*|` is still not `√ŝ/2`.
    #[test]
    fn the_equal_mass_beams_match_madgraphs_banked_record() {
        let e = [250.0_f64, 250.0];
        let m = [50.0, 50.0];
        let sqrt_s = partonic_s(e, m).sqrt();
        assert!((sqrt_s - 500.0).abs() < 1e-9, "sqrt(s-hat) = {sqrt_s}");
        let [a, b] = beam_momenta(sqrt_s, m[0], m[1]);
        assert!((a.pz() - 244.94897).abs() < 5e-6, "|p*| = {}", a.pz());
        assert!((a.e() - 250.0).abs() < 1e-9);
        assert!((b.e() - 250.0).abs() < 1e-9);
        assert_eq!(lab_beta_z(e, m), 0.0);
    }

    /// Massless beams of equal energy reproduce the light-cone construction
    /// exactly, which is what lets every such row keep its bit-for-bit result.
    #[test]
    fn massless_beams_of_equal_energy_reduce_to_the_light_cone() {
        let e = [250.0_f64, 250.0];
        let m = [0.0, 0.0];
        let s = partonic_s(e, m);
        assert_eq!(s, 500.0 * 500.0);
        let [a, b] = beam_momenta(s.sqrt(), 0.0, 0.0);
        assert_eq!(a, LorentzVector::new(250.0, 0.0, 0.0, 250.0));
        assert_eq!(b, LorentzVector::new(250.0, 0.0, 0.0, -250.0));
        assert_eq!(lab_beta_z(e, m), 0.0);
    }

    /// Massless beams of *unequal* energy collide at `4 E_a E_b`, not at
    /// `(E_a + E_b)²`: the sum of two head-on light-cone momenta carries
    /// `p³ = E_a − E_b`, and the difference is the boost between the laboratory
    /// and the partonic centre of mass.
    #[test]
    fn unequal_massless_energies_do_not_collide_at_the_energy_sum() {
        let e = [140.0_f64, 360.0];
        let m = [0.0, 0.0];
        let s = partonic_s(e, m);
        assert_eq!(s, 4.0 * 140.0 * 360.0);
        assert_ne!(s, 500.0 * 500.0);
        assert_eq!(lab_beta_z(e, m), (140.0 - 360.0) / 500.0);
    }

    /// The Møller flux `4√((p_a·p_b)² − m_a²m_b²)` is `2√λ(ŝ, m_a², m_b²)`, and
    /// `2ŝ` when both masses vanish.
    #[test]
    fn the_moller_flux_is_two_root_lambda() {
        let sqrt_s = 499.99275_f64;
        let (ma2, mb2) = (3600.0, 4900.0);
        let [a, b] = beam_momenta_m2(sqrt_s, ma2, mb2);
        let dot = a.e() * b.e() - a.pz() * b.pz();
        let moller = 4.0 * (dot * dot - ma2 * mb2).sqrt();
        let lambda = kallen(sqrt_s * sqrt_s, ma2, mb2);
        assert!((moller / (2.0 * lambda.sqrt()) - 1.0).abs() < 1e-12);
        // and 4 |p*| sqrt(s-hat), the third form of the same number.
        assert!((moller / (4.0 * a.pz() * sqrt_s) - 1.0).abs() < 1e-12);
        let massless = kallen(sqrt_s * sqrt_s, 0.0, 0.0);
        assert_eq!(2.0 * massless.sqrt(), 2.0 * sqrt_s * sqrt_s);
    }

    /// The laboratory rapidity of the beam system, against the closed form the
    /// cut filter's frame shift is built from. A hand-built point, so the check
    /// would fail if the shift were dropped or applied with the wrong sign.
    #[test]
    fn the_lab_boost_carries_the_beam_system_rapidity() {
        let e = [250.0_f64, 250.0];
        let m = [60.0, 70.0];
        let beta = lab_beta_z(e, m);
        let p0 = 500.0_f64;
        let p3 = (250.0f64 * 250.0 - 3600.0).sqrt() - (250.0f64 * 250.0 - 4900.0).sqrt();
        let y_cm = 0.5 * ((p0 + p3) / (p0 - p3)).ln();
        assert!((beta - p3 / p0).abs() < 1e-15);
        assert!((beta.atanh() - y_cm).abs() < 1e-15);
        assert!(
            (y_cm - 5.386e-3).abs() < 1e-6,
            "beam-system rapidity {y_cm}"
        );
    }
}
