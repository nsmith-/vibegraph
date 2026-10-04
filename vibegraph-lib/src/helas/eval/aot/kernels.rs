//! Out-of-line entry points over the evaluator's kernels, one set per rendered form.
//!
//! Every entry point is `#[inline(never)]`: a rendered program calls it, and keeps one
//! copy of each kernel's code however long the program is. The kernel itself inlines
//! into its entry point. Each entry point performs exactly the arithmetic of the
//! matching arm of the forward pass in `run.rs`.

/// By-value entry points: each returns its result, as the kernels do. A rendered
/// program binds the result to a local.
#[allow(
    dead_code,
    reason = "an entry point per kernel; the rendered rows call a subset"
)]
pub(super) mod by_value {
    use crate::helas::eval::kernel;
    use crate::helas::repr::lorentz::{
        Bispinor, Bra, ComplexVector, Ket, LorentzVector, Multivector,
    };
    use crate::helas::repr::numbers::Chirality;
    use crate::helas::repr::{Real, C};

    macro_rules! outline {
        ($(fn $f:ident($($a:ident: $t:ty),*) -> $r:ty;)*) => {$(
            #[inline(never)]
            pub(in super::super) fn $f<F: Real>($($a: $t),*) -> $r {
                kernel::$f($($a),*)
            }
        )*};
    }

    outline! {
        fn propagate_scalar_bare(value: C<F>, q: &LorentzVector<F>, mass: F, width: F) -> C<F>;
        fn propagate_vector_bare(eps: &ComplexVector<F>, q: &LorentzVector<F>, mass: F, width: F) -> ComplexVector<F>;
        fn propagate_fin_bare(spinor: &Bispinor<F, Ket>, q: &LorentzVector<F>, mass: F, width: F) -> Bispinor<F, Ket>;
        fn propagate_fout_bare(spinor: &Bispinor<F, Bra>, q: &LorentzVector<F>, mass: F, width: F) -> Bispinor<F, Bra>;
        fn pmom_bare(q: &LorentzVector<F>) -> ComplexVector<F>;
        fn gamma_vout_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>, reversed: bool) -> ComplexVector<F>;
        fn ffv_vout_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>, gl: C<F>, gr: C<F>, reversed: bool) -> ComplexVector<F>;
        fn off_shell_fin_bare(eps: &ComplexVector<F>, fi: &Bispinor<F, Ket>) -> Bispinor<F, Ket>;
        fn off_shell_fout_bare(eps: &ComplexVector<F>, fo: &Bispinor<F, Bra>) -> Bispinor<F, Bra>;
        fn ffv_fin_bare(eps: &ComplexVector<F>, fi: &Bispinor<F, Ket>, gl: C<F>, gr: C<F>) -> Bispinor<F, Ket>;
        fn ffv_fout_bare(eps: &ComplexVector<F>, fo: &Bispinor<F, Bra>, gl: C<F>, gr: C<F>) -> Bispinor<F, Bra>;
        fn proj_fin_bare(fi: &Bispinor<F, Ket>, chirality: Chirality) -> Bispinor<F, Ket>;
        fn proj_fout_bare(fo: &Bispinor<F, Bra>, chirality: Chirality) -> Bispinor<F, Bra>;
        fn gamma5_fin_bare(fi: &Bispinor<F, Ket>) -> Bispinor<F, Ket>;
        fn gamma5_fout_bare(fo: &Bispinor<F, Bra>) -> Bispinor<F, Bra>;
        fn scalar_bilinear_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>, chirality: Chirality) -> C<F>;
        fn pseudoscalar_bilinear_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>) -> C<F>;
        fn metric_bare(v1: &ComplexVector<F>, v2: &ComplexVector<F>) -> C<F>;
        fn metric_vout_bare(vin: &ComplexVector<F>) -> ComplexVector<F>;
        fn epsilon_vout_bare(a: &ComplexVector<F>, b: &ComplexVector<F>, c: &ComplexVector<F>) -> ComplexVector<F>;
        fn epsilon_amp_bare(a: &ComplexVector<F>, b: &ComplexVector<F>, c: &ComplexVector<F>, d: &ComplexVector<F>) -> C<F>;
        fn fierz_out_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>, reversed_order: bool) -> Multivector<F>;
        fn multivector_fin_bare(m: &Multivector<F>, fi: &Bispinor<F, Ket>) -> Bispinor<F, Ket>;
        fn multivector_fout_bare(m: &Multivector<F>, fo: &Bispinor<F, Bra>) -> Bispinor<F, Bra>;
        fn fierz_pair_bare(m: &Multivector<F>, fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>) -> C<F>;
        fn sigma_vout_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>, v: &ComplexVector<F>, negate: bool) -> ComplexVector<F>;
        fn sigma_mv_bare(a: &ComplexVector<F>, b: &ComplexVector<F>) -> Multivector<F>;
        fn sigma_out_bare(fo: &Bispinor<F, Bra>, fi: &Bispinor<F, Ket>, reversed_order: bool) -> Multivector<F>;
    }
}

/// Out-parameter entry points, in MadGraph's calling form: every operand by reference,
/// the result written through `out` into its slot. Nothing is returned by value.
#[allow(
    dead_code,
    reason = "an entry point per instruction kind; the rendered rows call a subset"
)]
pub(super) mod out_param {
    use std::ops::{Add, Mul};

    use num_traits::Zero;

    use crate::helas::eval::kernel;
    use crate::helas::eval::run::build_external_core;
    use crate::helas::eval::waveform_slot::WaveformSlot;
    use crate::helas::repr::lorentz::{
        Bispinor, Bra, ComplexVector, Ket, LorentzVector, Multivector,
    };
    use crate::helas::repr::numbers::{Charge, Chirality};
    use crate::helas::repr::{Real, C};

    /// `out = Σ terms`, folded left from the first term, as the forward pass's `Add*`
    /// arms sum their operand list.
    #[inline(never)]
    pub(in super::super) fn add<T: Copy + Add<Output = T>>(out: &mut T, terms: &[&T]) {
        let mut acc = *terms[0];
        for t in &terms[1..] {
            acc = acc + **t;
        }
        *out = acc;
    }

    /// `out = a · b`: the forward pass's `Mul*` and `Scale*` arms.
    #[inline(never)]
    pub(in super::super) fn mul<T: Copy + Mul<S, Output = T>, S: Copy>(out: &mut T, a: &T, b: &S) {
        *out = *a * *b;
    }

    /// An external leg's wavefunction, as `build_external_slot` builds it for a leg
    /// whose helicity is baked into the program.
    #[inline(always)]
    fn external<F: Real>(
        p: &LorentzVector<F>,
        hel: i32,
        spin: i32,
        charge: Charge,
        incoming: bool,
        mass: &F,
    ) -> WaveformSlot<F> {
        build_external_core(*p, hel, spin, charge, incoming, *mass)
    }

    #[inline(never)]
    pub(in super::super) fn ext_scalar<F: Real>(
        out: &mut C<F>,
        p: &LorentzVector<F>,
        hel: i32,
        spin: i32,
        charge: Charge,
        incoming: bool,
        mass: &F,
    ) {
        let WaveformSlot::Scalar(s) = external(p, hel, spin, charge, incoming, mass) else {
            panic!("external scalar leg produced a non-scalar slot");
        };
        *out = s.value;
    }

    #[inline(never)]
    pub(in super::super) fn ext_vector<F: Real>(
        out: &mut ComplexVector<F>,
        p: &LorentzVector<F>,
        hel: i32,
        spin: i32,
        charge: Charge,
        incoming: bool,
        mass: &F,
    ) {
        let WaveformSlot::Vector(v) = external(p, hel, spin, charge, incoming, mass) else {
            panic!("external vector leg produced a non-vector slot");
        };
        *out = v.eps;
    }

    #[inline(never)]
    pub(in super::super) fn ext_fin<F: Real>(
        out: &mut Bispinor<F, Ket>,
        p: &LorentzVector<F>,
        hel: i32,
        spin: i32,
        charge: Charge,
        incoming: bool,
        mass: &F,
    ) {
        let WaveformSlot::FermionIn(f) = external(p, hel, spin, charge, incoming, mass) else {
            panic!("external ket leg produced a non-fermion-in slot");
        };
        *out = f.spinor;
    }

    #[inline(never)]
    pub(in super::super) fn ext_fout<F: Real>(
        out: &mut Bispinor<F, Bra>,
        p: &LorentzVector<F>,
        hel: i32,
        spin: i32,
        charge: Charge,
        incoming: bool,
        mass: &F,
    ) {
        let WaveformSlot::FermionOut(f) = external(p, hel, spin, charge, incoming, mass) else {
            panic!("external bra leg produced a non-fermion-out slot");
        };
        *out = f.spinor;
    }

    /// `PMomOut`: the momentum `-(0 ± p₁ ± p₂ …)` summed in operand order, then `P`.
    #[inline(never)]
    pub(in super::super) fn pmom_out<F: Real>(
        out: &mut ComplexVector<F>,
        mm: &[LorentzVector<F>],
        terms: &[(u32, i8)],
    ) {
        let mut acc = LorentzVector::zero();
        for &(mid, sign) in terms {
            let p = mm[mid as usize];
            acc = if sign < 0 { acc - p } else { acc + p };
        }
        let neg = -acc;
        *out = kernel::pmom_bare(&neg);
    }

    /// One out-parameter entry point per kernel: the kernel's arguments by reference
    /// (flags by value), the result written through `out`.
    macro_rules! out_param {
        ($(fn $f:ident = $k:ident($($a:ident: $t:ty => $e:expr),*) -> $r:ty;)*) => {$(
            #[inline(never)]
            pub(in super::super) fn $f<F: Real>(out: &mut $r, $($a: $t),*) {
                *out = kernel::$k($($e),*);
            }
        )*};
    }

    out_param! {
        fn propagate_scalar = propagate_scalar_bare(value: &C<F> => *value, q: &LorentzVector<F> => q, mass: &F => *mass, width: &F => *width) -> C<F>;
        fn propagate_vector = propagate_vector_bare(eps: &ComplexVector<F> => eps, q: &LorentzVector<F> => q, mass: &F => *mass, width: &F => *width) -> ComplexVector<F>;
        fn propagate_fin = propagate_fin_bare(spinor: &Bispinor<F, Ket> => spinor, q: &LorentzVector<F> => q, mass: &F => *mass, width: &F => *width) -> Bispinor<F, Ket>;
        fn propagate_fout = propagate_fout_bare(spinor: &Bispinor<F, Bra> => spinor, q: &LorentzVector<F> => q, mass: &F => *mass, width: &F => *width) -> Bispinor<F, Bra>;
        fn pmom = pmom_bare(q: &LorentzVector<F> => q) -> ComplexVector<F>;
        fn gamma_vout = gamma_vout_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi, reversed: bool => reversed) -> ComplexVector<F>;
        fn ffv_vout = ffv_vout_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi, gl: &C<F> => *gl, gr: &C<F> => *gr, reversed: bool => reversed) -> ComplexVector<F>;
        fn off_shell_fin = off_shell_fin_bare(eps: &ComplexVector<F> => eps, fi: &Bispinor<F, Ket> => fi) -> Bispinor<F, Ket>;
        fn off_shell_fout = off_shell_fout_bare(eps: &ComplexVector<F> => eps, fo: &Bispinor<F, Bra> => fo) -> Bispinor<F, Bra>;
        fn ffv_fin = ffv_fin_bare(eps: &ComplexVector<F> => eps, fi: &Bispinor<F, Ket> => fi, gl: &C<F> => *gl, gr: &C<F> => *gr) -> Bispinor<F, Ket>;
        fn ffv_fout = ffv_fout_bare(eps: &ComplexVector<F> => eps, fo: &Bispinor<F, Bra> => fo, gl: &C<F> => *gl, gr: &C<F> => *gr) -> Bispinor<F, Bra>;
        fn proj_fin = proj_fin_bare(fi: &Bispinor<F, Ket> => fi, chirality: Chirality => chirality) -> Bispinor<F, Ket>;
        fn proj_fout = proj_fout_bare(fo: &Bispinor<F, Bra> => fo, chirality: Chirality => chirality) -> Bispinor<F, Bra>;
        fn gamma5_fin = gamma5_fin_bare(fi: &Bispinor<F, Ket> => fi) -> Bispinor<F, Ket>;
        fn gamma5_fout = gamma5_fout_bare(fo: &Bispinor<F, Bra> => fo) -> Bispinor<F, Bra>;
        fn scalar_bilinear = scalar_bilinear_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi, chirality: Chirality => chirality) -> C<F>;
        fn pseudoscalar_bilinear = pseudoscalar_bilinear_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi) -> C<F>;
        fn metric = metric_bare(a: &ComplexVector<F> => a, b: &ComplexVector<F> => b) -> C<F>;
        fn metric_vout = metric_vout_bare(v: &ComplexVector<F> => v) -> ComplexVector<F>;
        fn epsilon_vout = epsilon_vout_bare(a: &ComplexVector<F> => a, b: &ComplexVector<F> => b, c: &ComplexVector<F> => c) -> ComplexVector<F>;
        fn epsilon_amp = epsilon_amp_bare(a: &ComplexVector<F> => a, b: &ComplexVector<F> => b, c: &ComplexVector<F> => c, d: &ComplexVector<F> => d) -> C<F>;
        fn fierz_out = fierz_out_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi, reversed_order: bool => reversed_order) -> Multivector<F>;
        fn multivector_fin = multivector_fin_bare(m: &Multivector<F> => m, fi: &Bispinor<F, Ket> => fi) -> Bispinor<F, Ket>;
        fn multivector_fout = multivector_fout_bare(m: &Multivector<F> => m, fo: &Bispinor<F, Bra> => fo) -> Bispinor<F, Bra>;
        fn fierz_pair = fierz_pair_bare(m: &Multivector<F> => m, fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi) -> C<F>;
        fn sigma_vout = sigma_vout_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi, v: &ComplexVector<F> => v, negate: bool => negate) -> ComplexVector<F>;
        fn sigma_mv = sigma_mv_bare(a: &ComplexVector<F> => a, b: &ComplexVector<F> => b) -> Multivector<F>;
        fn sigma_out = sigma_out_bare(fo: &Bispinor<F, Bra> => fo, fi: &Bispinor<F, Ket> => fi, reversed_order: bool => reversed_order) -> Multivector<F>;
    }
}
