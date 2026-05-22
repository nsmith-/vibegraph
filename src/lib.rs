/// Helicity amplitude routines (HELAS-compatible).
///
/// This module will provide Rust implementations of the HELAS wavefunction and
/// vertex subroutines, validated against the original Fortran77 HELAS routines
/// via the harness in `validation/helas/`.
pub mod helas {
    /// Compute |M|² summed over all 16 helicity combinations for
    /// e⁺ e⁻ → μ⁺ μ⁻ via a single virtual photon exchange (QED, tree level,
    /// massless fermion approximation).
    ///
    /// # Arguments
    /// * `sqrt_s`    — CM energy in GeV
    /// * `cos_theta` — cosine of the μ⁻ scattering angle in the CM frame
    ///
    /// # Returns
    /// Σ_{helicities} |M|²  (not averaged over initial states)
    ///
    /// # Note
    /// This is a stub returning 0.0 until the HELAS implementation is written.
    /// The integration test in `tests/helas_validation.rs` is marked `#[ignore]`
    /// until this function is implemented correctly.
    pub fn compute_m2_ee_mumu(_sqrt_s: f64, _cos_theta: f64) -> f64 {
        0.0
    }
}
