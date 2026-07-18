use num_complex::ComplexFloat;

use crate::helas::repr::{
    lorentz::{Bispinor, ComplexVector, DiracAdjoint, SpinorRepr, VectorRepr},
    numbers::Chirality,
    r, ri, Real, C,
};
use crate::helas::wavefn::{InDiracWf, OutDiracWf, ScalarWf, VectorWf};

// ──────────────────────────────────────────────────────────────────────────────
// j3xxxx — off-shell W³ (γ + Z combined) current
// ──────────────────────────────────────────────────────────────────────────────

/// Off-shell W³ boson current from two external fermion legs.
///
/// Corresponds to the HELAS routine `j3xxxx`.  Uses Weinberg-angle mixing
/// extracted from the coupling ratio gzf[1]/gaf[1] — this matches the exact
/// Fortran logic where the photon and Z propagators are combined in the W³
/// eigenstate basis.
///
/// # Arguments
/// * `fo`     – flowing-OUT fermion wavefunction
/// * `fi`     – flowing-IN  fermion wavefunction
/// * `gaf`    – photon couplings  `[g_L^γ, g_R^γ]`
/// * `gzf`    – Z couplings       `[g_L^Z, g_R^Z]`
/// * `zmass`  – Z mass
/// * `zwidth` – Z total decay width (Breit-Wigner)
pub fn j3xxxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    gaf: [F; 2],
    gzf: [F; 2],
    zmass: F,
    zwidth: F,
) -> VectorWf<F> {
    // Off-shell momentum: fo.p − fi.p
    let jmom = fo.momentum - fi.momentum;
    // Propagator momentum (inflow convention)
    let q = -jmom;
    let q2 = q.m2();

    let zm2 = zmass * zmass;
    let zmw = zmass * zwidth;

    // ── Weinberg angle from coupling ratio  gzf[1] / gaf[1] ─────────────
    let ratio = gzf[1] / gaf[1];
    let cw = F::one() / (F::one() + ratio * ratio).sqrt();
    let sw = ((F::one() - cw) * (F::one() + cw)).sqrt();

    let ga3l = C::from(gaf[0] * sw); // photon left coupling
    let gz3l = C::from(gzf[0] * cw); // Z left coupling
    let gn = C::from(gaf[1] * sw); // combined right coupling

    // ── Propagators ───────────────────────────────────────────────────────
    let da = F::one() / q2; // real photon: 1/q²
    let dz = C::new(F::one(), F::zero())                 // Z: 1/(q²−mZ²+imZΓZ)
        / C::new(q2 - zm2, zmw);
    let ddif = C::new(-zm2, zmw) * r(da) * dz; // ≈ da for mZ → ∞

    // ── Bilinear currents via GammaL / GammaR intertwiners ────────────────
    let cl = fo.vector_bilinear(fi, Chirality::Left);
    let cr = fo.vector_bilinear(fi, Chirality::Right);

    // Longitudinal-mode projections divided by complex mZ²
    let cm2 = C::new(zm2, -zmw);
    let csl = cl.dot_lorentz(&q) / cm2;
    let csr = cr.dot_lorentz(&q) / cm2;

    // ── Output polarisation vector ────────────────────────────────────────
    // eps[μ] = gz3l·dz·(cl[μ] − q[μ]·csl)
    //        + ga3l·da·cl[μ]
    //        + gn·(cr[μ]·ddif + q[μ]·csr·dz)
    let qc = ComplexVector::from(q);
    let eps = (cl - qc * csl) * (gz3l * dz) + cl * (ga3l * da) + (cr * ddif + qc * (csr * dz)) * gn;

    VectorWf {
        eps,
        momentum: jmom,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// jioxxx — off-shell single-boson current
// ──────────────────────────────────────────────────────────────────────────────

/// Off-shell single-boson current from a fermion pair, with independent
/// left/right couplings.
///
/// Mirrors the HELAS `jioxxx` routine.  Uses Feynman gauge for massless bosons
/// and unitary gauge (with the Fabio fixed-width prescription) for massive ones.
///
/// Unlike [`j3xxxx`], this routine handles a single boson species (photon *or*
/// Z) at a time.  To compute the full SM `e⁺e⁻ → μ⁺μ⁻` matrix element, call
/// this once for the photon and once for the Z, then sum the resulting
/// amplitudes before squaring.
///
/// # Arguments
/// * `fo`     – flowing-OUT fermion wavefunction (e.g. e⁺ in the electron current)
/// * `fi`     – flowing-IN  fermion wavefunction (e.g. e⁻)
/// * `gc`     – couplings `[g_L, g_R]` (left/right-handed, real)
/// * `vmass`  – boson mass (0 for photon)
/// * `vwidth` – boson total width (0 for stable)
pub fn jioxxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    gc: [F; 2],
    vmass: F,
    vwidth: F,
) -> VectorWf<F> {
    // Off-shell momentum: jmom = fo.p − fi.p  (outflow convention)
    let q = fo.momentum - fi.momentum;
    let q2 = q.m2();

    // Bilinear currents via GammaL / GammaR intertwiners
    let cl = fo.vector_bilinear(fi, Chirality::Left);
    let cr = fo.vector_bilinear(fi, Chirality::Right);
    let blin = cl * gc[0] + cr * gc[1]; // linear combination of left and right currents

    let eps = if vmass == F::zero() {
        // Massless: Feynman gauge — propagator is 1/q²
        blin / q2
    } else {
        // Massive: unitary gauge with Fabio fixed-width complex denominator
        let vm2 = vmass * vmass;
        let vmw = vmass * vwidth;
        let denom = C::new(q2 - vm2, vmw);
        // Longitudinal mode subtraction: divide by m²−imΓ (Fabio prescription)
        let cs = blin.dot_lorentz(&q) / C::new(vm2, -vmw);
        (blin - ComplexVector::from(q) * cs) / denom
    };

    VectorWf { eps, momentum: q }
}

// ──────────────────────────────────────────────────────────────────────────────
// iovxxx — amplitude: fermion–fermion–vector contraction
// ──────────────────────────────────────────────────────────────────────────────

/// Compute the amplitude ⟨fo | γ^μ (gL P_L + gR P_R) | fi⟩ · ε_μ.
///
/// Corresponds to HELAS `iovxxx`.
///
/// # Arguments
/// * `fo`  – flowing-OUT fermion wavefunction
/// * `fi`  – flowing-IN  fermion wavefunction
/// * `v`   – vector (gauge boson) wavefunction
/// * `gc`  – gauge couplings  `[g_L, g_R]`
///
/// # Returns
/// The complex-valued Lorentz-invariant amplitude.
pub fn iovxxx<F: Real>(fo: &OutDiracWf<F>, fi: &InDiracWf<F>, v: &VectorWf<F>, gc: [F; 2]) -> C<F> {
    let cl = fo.vector_bilinear(fi, Chirality::Left);
    let cr = fo.vector_bilinear(fi, Chirality::Right);

    // M = gc[0] * (C_L · V) + gc[1] * (C_R · V)
    C::from(gc[0]) * cl.dot(&v.eps.lower()) + C::from(gc[1]) * cr.dot(&v.eps.lower())
}

// ──────────────────────────────────────────────────────────────────────────────
// ffv2_3 / ffv4_3 / ffv2_4_3 — ALOHA off-shell vector currents (chiral basis)
// ──────────────────────────────────────────────────────────────────────────────
//
// These transcribe the ALOHA-generated routines `FFV2_3.f` / `FFV4_3.f` (and the
// `FFV2_4_3` wrapper) component-for-component. They are the modern equivalent of
// [`jioxxx`], but expressed directly in ALOHA's chiral-projector convention:
//
//   FFV2  =  Gamma(3,2,-1)·ProjM(-1,1)                  (pure left, P_L)
//   FFV4  =  Gamma(3,2,-1)·(ProjM + 2·ProjP)(-1,1)      (left + 2·right)
//
// vibegraph stores the Weyl-basis bispinor index-for-index with the HELAS/ALOHA
// 6-component array: our `spinor[k]` is ALOHA `F(3+k)` (components 0,1 left-chiral,
// 2,3 right-chiral). The off-shell momentum is `P3 = f2.p − f1.p`, matching
// [`jioxxx`]'s `q`. ALOHA folds an explicit `−i` per Lorentz structure into the
// current (the `(−CI)` / `(−2·CI)` prefactors below), which vibegraph's evaluator
// instead carries in the UFO coupling — so an ALOHA current equals the vibegraph /
// `jioxxx` current times that `−i`.

/// The four Weyl-basis spinor components, ordered as the HELAS/ALOHA `F(3..6)` array.
fn spinor_components<F: Real, Adj: DiracAdjoint>(s: &Bispinor<F, Adj>) -> [C<F>; 4] {
    [
        s.component(0),
        s.component(1),
        s.component(2),
        s.component(3),
    ]
}

/// ALOHA `FFV2_3`: off-shell vector current with the pure-left (P_L) structure.
///
/// `f1` is the flow-IN (ket) fermion `F1`, `f2` the flow-OUT (bra) fermion `F2`.
pub fn ffv2_3<F: Real>(
    f1: &InDiracWf<F>,
    f2: &OutDiracWf<F>,
    coup: C<F>,
    m3: F,
    w3: F,
) -> VectorWf<F> {
    let a = spinor_components(&f1.spinor);
    let b = spinor_components(&f2.spinor);
    let q = f2.momentum - f1.momentum;
    let (p0, p1, p2, p3) = (q.e(), q.px(), q.py(), q.pz());

    let ci: C<F> = C::i();
    let om3 = if m3 == F::zero() {
        F::zero()
    } else {
        F::one() / (m3 * m3)
    };
    let denom = coup / C::new(q.m2() - m3 * m3, m3 * w3);

    // TMP1 = J_L · P3  (left current contracted with the off-shell momentum).
    let tmp1 = a[0] * (b[2] * r(p0 + p3) + b[3] * (r(p1) + ri(p2)))
        + a[1] * (b[2] * (r(p1) - ri(p2)) + b[3] * r(p0 - p3));

    let eps = ComplexVector::new([
        denom * (-ci) * (a[0] * b[2] + a[1] * b[3] - tmp1 * r(p0 * om3)),
        denom * (-ci) * (-a[0] * b[3] - a[1] * b[2] - tmp1 * r(p1 * om3)),
        denom * (-ci) * (-ci * (a[0] * b[3]) + ci * (a[1] * b[2]) - tmp1 * r(p2 * om3)),
        denom * (-ci) * (-a[0] * b[2] + a[1] * b[3] - tmp1 * r(p3 * om3)),
    ]);

    VectorWf { eps, momentum: q }
}

/// ALOHA `FFV4_3`: off-shell vector current with the (P_L + 2·P_R) structure.
///
/// `f1` is the flow-IN (ket) fermion `F1`, `f2` the flow-OUT (bra) fermion `F2`.
pub fn ffv4_3<F: Real>(
    f1: &InDiracWf<F>,
    f2: &OutDiracWf<F>,
    coup: C<F>,
    m3: F,
    w3: F,
) -> VectorWf<F> {
    let a = spinor_components(&f1.spinor);
    let b = spinor_components(&f2.spinor);
    let q = f2.momentum - f1.momentum;
    let (p0, p1, p2, p3) = (q.e(), q.px(), q.py(), q.pz());

    let ci: C<F> = C::i();
    let two = r(F::one() + F::one());
    let half = r(F::one() / (F::one() + F::one()));
    let om3 = if m3 == F::zero() {
        F::zero()
    } else {
        F::one() / (m3 * m3)
    };
    let denom = coup / C::new(q.m2() - m3 * m3, m3 * w3);
    let hf = F::one() / (F::one() + F::one()); // ½ as a real, for the OM3 momentum terms

    // TMP1 = J_L · P3,  TMP3 = J_R · P3  (left/right currents · momentum).
    let tmp1 = a[0] * (b[2] * r(p0 + p3) + b[3] * (r(p1) + ri(p2)))
        + a[1] * (b[2] * (r(p1) - ri(p2)) + b[3] * r(p0 - p3));
    let tmp3 = a[2] * (b[0] * r(p0 - p3) - b[1] * (r(p1) + ri(p2)))
        + a[3] * (b[0] * (-r(p1) + ri(p2)) + b[1] * r(p0 + p3));
    let s = tmp1 + tmp3 * two; // TMP1 + 2·TMP3

    let eps = ComplexVector::new([
        denom
            * (-(ci * two))
            * (s * r(-hf * p0 * om3)
                + half * (a[0] * b[2] + a[1] * b[3])
                + a[2] * b[0]
                + a[3] * b[1]),
        denom
            * (-(ci * two))
            * (s * r(-hf * p1 * om3) - half * (a[0] * b[3] + a[1] * b[2])
                + a[2] * b[1]
                + a[3] * b[0]),
        denom
            * (ci * two)
            * (s * r(hf * p2 * om3) + ci * half * (a[0] * b[3])
                - ci * half * (a[1] * b[2])
                - ci * (a[2] * b[1])
                + ci * (a[3] * b[0])),
        denom
            * (ci * two)
            * (s * r(hf * p3 * om3) + half * (a[0] * b[2]) - half * (a[1] * b[3]) - a[2] * b[0]
                + a[3] * b[1]),
    ]);

    VectorWf { eps, momentum: q }
}

/// ALOHA `FFV2_4_3`: the SM Z off-shell vector current, combining the FFV2 (left)
/// and FFV4 (left + 2·right) Lorentz structures with independent couplings.
///
/// Equivalent to `ffv2_3(.., coup1, ..) + ffv4_3(.., coup2, ..)` — exactly the body
/// of the generated `FFV2_4_3.f` wrapper.
pub fn ffv2_4_3<F: Real>(
    f1: &InDiracWf<F>,
    f2: &OutDiracWf<F>,
    coup1: C<F>,
    coup2: C<F>,
    m3: F,
    w3: F,
) -> VectorWf<F> {
    let v2 = ffv2_3(f1, f2, coup1, m3, w3);
    let v4 = ffv4_3(f1, f2, coup2, m3, w3);
    VectorWf {
        eps: v2.eps + v4.eps,
        momentum: v2.momentum,
    }
}

/// ALOHA `FFV2_2`: off-shell *fermion* current with the pure-left (P_L) vertex
/// `Gamma(3,2,-1)·ProjM(-1,1)`, propagator folded in.
///
/// This is the chiral analogue of [`fvixxx`] (FFV1_2 is the vector version): a
/// flow-IN fermion `f1` absorbs the vector `v`, continuing the line. It computes
/// `(q̸+m)·γ̸_V·P_L·ψ / (q²−m²+imΓ)` with `q = f1.p − v.p`, transcribed
/// component-for-component from the generated `FFV2_2.f`. The explicit `±i` per
/// component is part of ALOHA's gamma algebra; the overall current relates to the
/// vibegraph evaluator's by the same global `−i` UFO-coupling factor as the other
/// ALOHA references.
pub fn ffv2_2<F: Real>(
    f1: &InDiracWf<F>,
    v: &VectorWf<F>,
    coup: C<F>,
    m2: F,
    w2: F,
) -> InDiracWf<F> {
    let a = spinor_components(&f1.spinor);
    let w = [
        v.eps.component(0),
        v.eps.component(1),
        v.eps.component(2),
        v.eps.component(3),
    ];
    let q = f1.momentum - v.momentum;
    let (p0, p1, p2, p3) = (q.e(), q.px(), q.py(), q.pz());
    let ci = C::i();
    let denom = coup / C::new(q.m2() - m2 * m2, m2 * w2);

    // F1(3..6) = a[0..4];  V3(3..6) = w[0..4];  P2 = q.
    let f2_3 = denom
        * ci
        * (a[0]
            * (r(p0) * (w[0] + w[3]) - r(p1) * (w[1] + ci * w[2]) + r(p2) * (ci * w[1] - w[2])
                - r(p3) * (w[0] + w[3]))
            + a[1]
                * (r(p0) * (w[1] - ci * w[2])
                    + r(p1) * (-w[0] + w[3])
                    + r(p2) * (ci * w[0] - ci * w[3])
                    + r(p3) * (-w[1] + ci * w[2])));
    let f2_4 = denom
        * ci
        * (a[0]
            * (r(p0) * (w[1] + ci * w[2]) - r(p1) * (w[0] + w[3]) - r(p2) * (ci * (w[0] + w[3]))
                + r(p3) * (w[1] + ci * w[2]))
            + a[1]
                * (r(p0) * (w[0] - w[3]) + r(p1) * (-w[1] + ci * w[2])
                    - r(p2) * (ci * w[1] + w[2])
                    + r(p3) * (w[0] - w[3])));
    let f2_5 = denom * (-ci) * r(m2) * (-a[0] * (w[0] + w[3]) + a[1] * (-w[1] + ci * w[2]));
    let f2_6 = denom * ci * r(m2) * (a[0] * (w[1] + ci * w[2]) + a[1] * (w[0] - w[3]));

    InDiracWf::from_spinor(Bispinor::from_components([f2_3, f2_4, f2_5, f2_6]), q)
}

/// ALOHA `FFV4_2`: off-shell *fermion* current with the `(P_L + 2·P_R)` vertex
/// `Gamma(3,2,-1)·(ProjM + 2·ProjP)(-1,1)`, propagator folded in.
///
/// The FFV4 (left + 2·right) analogue of [`ffv2_2`]; together with it this covers the
/// SM Z fermion absorption (`FFV2_4_2 = ffv2_2(coup1) + ffv4_2(coup2)`). Transcribed
/// component-for-component from `FFV4_2.f`; relates to the vibegraph evaluator by the
/// same global `−i` factor.
pub fn ffv4_2<F: Real>(
    f1: &InDiracWf<F>,
    v: &VectorWf<F>,
    coup: C<F>,
    m2: F,
    w2: F,
) -> InDiracWf<F> {
    let a = spinor_components(&f1.spinor);
    let w = [
        v.eps.component(0),
        v.eps.component(1),
        v.eps.component(2),
        v.eps.component(3),
    ];
    let q = f1.momentum - v.momentum;
    let (p0, p1, p2, p3) = (q.e(), q.px(), q.py(), q.pz());
    let ci = C::i();
    let two = r(F::one() + F::one());
    let half = r(F::one() / (F::one() + F::one()));
    let m = r(m2);
    let denom = coup / C::new(q.m2() - m2 * m2, m2 * w2);

    // Left-output (F2(3,4)): left-input momentum terms (≡ FFV2_2) + 2·right-input mass.
    let f2_3 = denom
        * ci
        * (a[0]
            * (r(p0) * (w[0] + w[3]) - r(p1) * (w[1] + ci * w[2]) + r(p2) * (ci * w[1] - w[2])
                - r(p3) * (w[0] + w[3]))
            + a[1]
                * (r(p0) * (w[1] - ci * w[2])
                    + r(p1) * (-w[0] + w[3])
                    + r(p2) * (ci * w[0] - ci * w[3])
                    + r(p3) * (-w[1] + ci * w[2]))
            + m * two * (a[2] * (w[0] - w[3]) + a[3] * (-w[1] + ci * w[2])));
    let f2_4 = denom
        * ci
        * (a[0]
            * (r(p0) * (w[1] + ci * w[2]) - r(p1) * (w[0] + w[3]) - r(p2) * (ci * (w[0] + w[3]))
                + r(p3) * (w[1] + ci * w[2]))
            + a[1]
                * (r(p0) * (w[0] - w[3]) + r(p1) * (-w[1] + ci * w[2])
                    - r(p2) * (ci * w[1] + w[2])
                    + r(p3) * (w[0] - w[3]))
            + m * two * (-a[2] * (w[1] + ci * w[2]) + a[3] * (w[0] + w[3])));
    // Right-output (F2(5,6)): 2·right-input momentum terms + ½·left-input mass.
    let f2_5 = denom
        * (-(two * ci))
        * (a[2]
            * (r(p0) * (-w[0] + w[3])
                + r(p1) * (w[1] + ci * w[2])
                + r(p2) * (-ci * w[1] + w[2])
                + r(p3) * (-w[0] + w[3]))
            + a[3]
                * (r(p0) * (w[1] - ci * w[2]) - r(p1) * (w[0] + w[3])
                    + r(p2) * (ci * (w[0] + w[3]))
                    + r(p3) * (w[1] - ci * w[2]))
            + m * (-half * a[0] * (w[0] + w[3]) + half * a[1] * (-w[1] + ci * w[2])));
    let f2_6 = denom
        * (-(two * ci))
        * (a[2]
            * (r(p0) * (w[1] + ci * w[2])
                + r(p1) * (-w[0] + w[3])
                + r(p2) * (-ci * w[0] + ci * w[3])
                - r(p3) * (w[1] + ci * w[2]))
            + a[3]
                * (-r(p0) * (w[0] + w[3])
                    + r(p1) * (w[1] - ci * w[2])
                    + r(p2) * (ci * w[1] + w[2])
                    + r(p3) * (w[0] + w[3]))
            + m * (-half * a[0] * (w[1] + ci * w[2]) + half * a[1] * (-w[0] + w[3])));

    InDiracWf::from_spinor(Bispinor::from_components([f2_3, f2_4, f2_5, f2_6]), q)
}

// ──────────────────────────────────────────────────────────────────────────────
// Phase 2: Off-shell vertex routines
// ──────────────────────────────────────────────────────────────────────────────

/// Off-shell fermion from incoming fermion + vector boson.
///
/// Corresponds to HELAS `fvixxx` (FFV1_2 in ALOHA).
/// Computes `Ψ = i (q̸ + m) (i g_L P_L ψ + i g_R P_R ψ) / (q² − m² + imΓ)` where `q = fi.p + V.p`.
///
/// # Arguments
/// * `fi`    – flowing-IN  fermion wavefunction
/// * `v`     – vector (gauge boson) wavefunction
/// * `gc`     – couplings `[g_L, g_R]` (left/right-handed, real)
/// * `mass`  – off-shell fermion mass (usually 0 for light quarks)
/// * `width` – fermion width (usually 0 for stable)
///
/// # Returns
/// Off-shell fermion current as `InDiracWf<F>` (the codebase stores every
/// off-shell fermion in the flow-IN representation so it can be paired with a
/// flow-OUT leg at the next vertex).
pub fn fvixxx<F: Real>(
    fi: &InDiracWf<F>,
    v: &VectorWf<F>,
    gc: [F; 2],
    mass: F,
    width: F,
) -> InDiracWf<F> {
    // Accumulated momentum for the flow-IN case: q = fi.p − V.p
    // (Fortran `fvixxx`: fvi(5)=fi(5)−vc(5)). Opposite vector sign from the
    // flow-OUT `fvoxxx` — the asymmetry that conserves momentum along a line.
    let q = fi.momentum - v.momentum;
    let q2 = q.m2();

    // Vertex factor ε̸ ψ, then the (q̸ + m)/(q² − m² + imΓ) propagator, scaled by g.
    let psi = fi.spinor.project_left().slash(&v.eps) * gc[0]
        + fi.spinor.project_right().slash(&v.eps) * gc[1];
    let num = psi.slash(&q.into()) + psi * mass;
    // i^2 = -1 from the coupling and overall i factor
    let scale = -C::new(q2 - mass * mass, mass * width).recip();

    InDiracWf::from_spinor(num * scale, q)
}

/// Off-shell fermion from outgoing fermion + vector boson.
///
/// Corresponds to HELAS `fvoxxx` (FFV1_1 in ALOHA).
/// Similar to `fvixxx` but with outgoing fermion input.
///
/// # Arguments
/// * `fo`    – flowing-OUT fermion wavefunction (acts as input here)
/// * `v`     – vector (gauge boson) wavefunction
/// * `gc`     – couplings `[g_L, g_R]` (left/right-handed, real)
/// * `mass`  – off-shell fermion mass
/// * `width` – fermion width
///
/// # Returns
/// Off-shell fermion current as `OutDiracWf<F>`. `ε̸` and the propagator are
/// flow-preserving, so a flow-OUT (bra) input yields a flow-OUT current.
pub fn fvoxxx<F: Real>(
    fo: &OutDiracWf<F>,
    v: &VectorWf<F>,
    g: [F; 2],
    mass: F,
    width: F,
) -> OutDiracWf<F> {
    // Accumulated momentum: q = fo.p + V.p (outflow convention)
    let q = fo.momentum + v.momentum;
    let q2 = q.m2();

    // Bra vertex factor ψ̄ε̸, then the bra propagator ψ̄(q̸ + m)/(q² − m² + imΓ),
    // scaled by g. Note the right projection for flow-out is the correct
    // left coupling
    let psi = fo.spinor.project_right().slash(&v.eps) * g[0]
        + fo.spinor.project_left().slash(&v.eps) * g[1];
    let num = psi.slash(&q.into()) + psi * mass;
    let scale = -C::new(q2 - mass * mass, mass * width).recip();

    OutDiracWf::from_spinor(num * scale, q)
}

/// Off-shell vector from two vector bosons.
///
/// Corresponds to HELAS `jvvxxx` (VVV1P0_1 in ALOHA for massless case).
/// Computes the three-gauge-boson coupling.
/// `V_1^μ = g · P(q²) · [v2·v3 · (q_out^μ - q_in^μ) + v2^μ·(q_in·v3) - v3^μ·(q_in·v2)]`
/// where `q = v2.p + v3.p`.
///
/// # Arguments
/// * `v2`    – first input vector
/// * `v3`    – second input vector
/// * `g`     – coupling constant
/// * `mass`  – off-shell vector mass (0 for massless)
/// * `width` – off-shell vector width
///
/// # Returns
/// Off-shell vector boson as `VectorWf<F>`.
pub fn jvvxxx<F: Real>(
    v2: &VectorWf<F>,
    v3: &VectorWf<F>,
    g: C<F>,
    mass: F,
    width: F,
) -> VectorWf<F> {
    // Accumulated momentum: q = v2.p + v3.p (outflow for incoming vectors)
    let q = v2.momentum + v3.momentum;
    let q2 = q.m2();

    let m2 = mass * mass;
    let mw = mass * width;

    // Complex propagator denominator: q² − m² + imΓ
    let denom = C::new(q2 - m2, mw);

    // Minkowski contractions (cf. VVV1P0_1.f)
    let tmp1 = v2.eps.dot_lorentz(&q); // v2·q
    let tmp2 = v2.eps.dot_lorentz(&v3.momentum); // v2·p3
    let tmp3 = v3.eps.dot_lorentz(&q); // v3·q
    let tmp4 = v3.eps.dot_lorentz(&v2.momentum); // v3·p2
    let tmp5 = v2.eps.dot(&v3.eps.lower()); // v2·v3

    let scale = g / denom;

    // V1^μ = scale · [v2·v3·(p3^μ − p2^μ) + v2^μ·(v3·q − v3·p2) + v3^μ·(v2·q − v2·p3)]
    let q_diff = ComplexVector::from(v3.momentum - v2.momentum);
    let eps = (q_diff * tmp5 + v2.eps * (tmp3 - tmp4) + v3.eps * (tmp1 - tmp2)) * scale;

    VectorWf { eps, momentum: q }
}

/// Off-shell scalar from two fermions.
///
/// Corresponds to HELAS `jsixxx`.
/// Computes the scalar current from a fermion pair.
///
/// # Arguments
/// * ` flowing-OUT fermionfo`    
/// * ` flowing-IN  fermionfi`    
/// * ` coupling constant (real or complex)g`     
/// * ` off-shell scalar massmass`  
/// * ` off-shell scalar widthwidth`
///
/// # Returns
/// Off-shell scalar current as `ScalarWf<F>`.
pub fn jsixxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    g: C<F>,
    mass: F,
    width: F,
) -> ScalarWf<F> {
    // Accumulated momentum: q = fi.p + fo.p
    let q = fi.momentum + fo.momentum;
    let q2 = q.m2();

    // Scalar current: sum of left and right bilinears (identity structure)
    let scalar_value = Bispinor::scalar_bilinear(&fo.spinor, &fi.spinor, Chirality::Both);

    // Scalar propagator 1/(q² − m² + imΓ), scaled by the coupling.
    let denom = C::new(q2 - mass * mass, mass * width);

    ScalarWf {
        value: g * scalar_value / denom,
        momentum: q,
    }
}

/// Amplitude: two fermions + scalar.
///
/// Corresponds to HELAS `iosxxx`.
///
/// # Arguments
/// * ` flowing-OUT fermionfo`  
/// * ` flowing-IN  fermionfi`  
/// * ` scalar wavefunctions`   
/// * ` coupling constants `[g_L, g_R]`gc`  
///
/// # Returns
/// Complex amplitude.
pub fn iosxxx<F: Real>(
    fo: &OutDiracWf<F>,
    fi: &InDiracWf<F>,
    s: &ScalarWf<F>,
    gc: [C<F>; 2],
) -> C<F> {
    // Compute left and right chiral bilinears using the new trait methods
    let left_contr = Bispinor::scalar_bilinear(&fo.spinor, &fi.spinor, Chirality::Left);
    let right_contr = Bispinor::scalar_bilinear(&fo.spinor, &fi.spinor, Chirality::Right);

    // Combine with couplings and scalar value
    s.value * (gc[0] * left_contr + gc[1] * right_contr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helas::repr::lorentz::LorentzVector;
    use crate::helas::repr::numbers::{Charge, SpinorHelicity};
    use num_complex::Complex64 as C64;

    #[test]
    fn test_fvixxx_basic() {
        // Create test wavefunctions
        let fi_mom = LorentzVector::new(100.0, 0.0, 0.0, 100.0);
        let fi = InDiracWf::from_momentum(fi_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let v_mom = LorentzVector::new(10.0, 5.0, 0.0, 8.66);
        let v_eps = [
            C64::new(1.0, 0.0),
            C64::new(0.5, 0.2),
            C64::new(0.3, 0.1),
            C64::new(0.4, 0.0),
        ];
        let v = VectorWf {
            eps: ComplexVector::new(v_eps),
            momentum: v_mom,
        };

        // Apply fvixxx
        let g = C64::new(0.0, 1.0);
        let result = fvixxx(&fi, &v, [g.im, g.im], 0.0, 0.0);

        // Basic sanity checks
        assert!(
            result.spinor.bare_norm_sq() > 0.0,
            "Result spinor should be non-zero"
        );

        // Check that output momentum is fi − v (flow-in convention, cf. fvixxx)
        let expected_momentum = fi_mom - v_mom;
        assert!(
            (result.momentum - expected_momentum).bare_norm_sq() < 1e-10,
            "Momentum mismatch: expected {expected_momentum:?}, got {result:?}"
        );
    }

    #[test]
    fn test_fvoxxx_basic() {
        // Create test wavefunctions
        let fo_mom = LorentzVector::new(100.0, 0.0, 0.0, 100.0);
        let fo = OutDiracWf::from_momentum(fo_mom, 0.0, SpinorHelicity::Up, Charge::Particle);

        let v_mom = LorentzVector::new(10.0, 5.0, 0.0, 8.66);
        let v_eps = [
            C64::new(1.0, 0.0),
            C64::new(0.5, 0.2),
            C64::new(0.3, 0.1),
            C64::new(0.4, 0.0),
        ];
        let v = VectorWf {
            eps: ComplexVector::new(v_eps),
            momentum: v_mom,
        };

        // Apply fvoxxx
        let g = C64::new(0.0, 1.0);
        let result = fvoxxx(&fo, &v, [g.im, g.im], 0.0, 0.0);

        // Basic sanity checks
        assert!(
            result.spinor.bare_norm_sq() > 0.0,
            "Result spinor should be non-zero"
        );

        // Check that output momentum is fo + v
        let expected_momentum = fo_mom + v_mom;
        assert!(
            (result.momentum - expected_momentum).bare_norm_sq() < 1e-10,
            "Momentum mismatch: expected {expected_momentum:?}, got {result:?}"
        );
    }

    #[test]
    fn test_jvvxxx_basic() {
        // Create two vector wavefunctions
        let v2_mom = LorentzVector::new(50.0, 30.0, 0.0, 40.0);
        let v2_eps = [
            C64::new(0.8, 0.0),
            C64::new(0.3, 0.1),
            C64::new(0.2, 0.0),
            C64::new(0.4, 0.0),
        ];
        let v2 = VectorWf {
            eps: ComplexVector::new(v2_eps),
            momentum: v2_mom,
        };

        let v3_mom = LorentzVector::new(30.0, -20.0, 0.0, 22.4);
        let v3_eps = [
            C64::new(0.6, 0.0),
            C64::new(-0.2, 0.1),
            C64::new(0.1, 0.0),
            C64::new(0.2, 0.0),
        ];
        let v3 = VectorWf {
            eps: ComplexVector::new(v3_eps),
            momentum: v3_mom,
        };

        // Apply jvvxxx
        let coupling = C64::new(0.1, 0.0);
        let result = jvvxxx(&v2, &v3, coupling, 0.0, 0.0);

        // Basic sanity checks
        assert!(
            result.eps.bare_norm_sq() > 0.0,
            "Result polarization should be non-zero"
        );

        // Check that output momentum is v2 + v3
        let expected_momentum = v2_mom + v3_mom;
        assert!(
            (result.momentum - expected_momentum).bare_norm_sq() < 1e-10,
            "Momentum mismatch: expected {expected_momentum:?}, got {result:?}"
        );
    }

    #[test]
    fn test_jsixxx_basic() {
        // Create fermion wavefunctions
        let fi_mom = LorentzVector::from_pxpypzmass(0.0, 0.0, 20.0, 10.0);
        let fo_mom = LorentzVector::from_pxpypzmass(0.0, 0.0, -20.0, 10.0);
        let fi = InDiracWf::from_momentum(fi_mom, 10.0, SpinorHelicity::Up, Charge::Particle);
        let fo = OutDiracWf::from_momentum(fo_mom, 10.0, SpinorHelicity::Up, Charge::Antiparticle);

        // Apply jsixxx
        let coupling = C64::new(0.1, 0.0);
        let result = jsixxx(&fo, &fi, coupling, 0.0, 0.0);

        // Basic sanity check
        assert!(
            result.value.norm() > 0.0,
            "Result scalar should be non-zero"
        );

        // Check that output momentum is the exchanged momentum (fi - fo)
        let expected_momentum = fi_mom - fo_mom;
        assert!(
            (result.momentum - expected_momentum).bare_norm_sq() < 1e-10,
            "Momentum mismatch: expected {expected_momentum}, got {}",
            result.momentum
        );
    }

    #[test]
    fn test_iosxxx_basic() {
        // Create fermion and scalar wavefunctions
        let fi_mom = LorentzVector::from_pxpypzmass(0.0, 0.0, 20.0, 10.0);
        let fo_mom = LorentzVector::from_pxpypzmass(0.0, 0.0, -20.0, 10.0);
        let fi = InDiracWf::from_momentum(fi_mom, 10.0, SpinorHelicity::Up, Charge::Particle);
        let fo = OutDiracWf::from_momentum(fo_mom, 10.0, SpinorHelicity::Up, Charge::Antiparticle);

        let s_mom = -(fi_mom + fo_mom);
        let s = ScalarWf {
            value: C64::new(0.0, 1.0),
            momentum: s_mom,
        };

        // Apply iosxxx
        let coupling = [C64::new(0.1, 0.0), C64::new(0.1, 0.0)];
        let result = iosxxx(&fo, &fi, &s, coupling);

        // Basic sanity check: the result should be non-zero
        assert!(result.norm() > 0.0, "Result amplitude should be non-zero");
    }
}
