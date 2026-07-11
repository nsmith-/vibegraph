//! Operator-fusion microbenchmark: the FFV `[g_L, g_R]` vertex evaluated as the
//! interpreter's generic kernel chain vs a single fused kernel.
//!
//! Two vertex rootings are measured, each in three strategies:
//! - `generic_res` — the kernel chain with every intermediate materialized in a
//!   result buffer and children read back by index, mimicking the forward scan's
//!   per-node slot traffic (9 nodes; the real lowered vertex is ~14 because the
//!   per-structure `Coeff` factors and the two-level coupling sum are separate
//!   nodes here folded into `g_L`/`g_R`, so this chain *understates* the gap).
//! - `generic_direct` — the same kernel calls nested without materialization:
//!   isolates kernel arithmetic + call overhead from slot traffic.
//! - `fused` — one kernel computing `g_L·(chiral-left term) + g_R·(chiral-right
//!   term)` directly. Besides collapsing the `Mul`/`Add` scaffolding, it skips the
//!   arithmetic the generic chain wastes on structurally-zero chiral halves
//!   (`GammaVout` computes both currents even when its input was just projected).
//!
//! Run: `cargo bench -p vibegraph-lib --features bench-internals --bench kernel_fusion`

use criterion::{criterion_group, criterion_main, Criterion};
use num_traits::Zero;

use vibegraph::helas::eval::bench_internals::{
    gamma_iout, gamma_vout, mul_apply, proj_m, proj_p, rand_bra, rand_c, rand_ket, rand_vector,
    seeded_rng, slots_approx_eq, WaveformSlot,
};
use vibegraph::helas::repr::lorentz::{Bispinor, LorentzVector, SpinorRepr, VectorRepr};
use vibegraph::helas::repr::C;
use vibegraph::helas::wavefn::ScalarWf;
use vibegraph::helas::{InDiracWf, VectorWf};

type Slot = WaveformSlot<f64>;

/// One FFV vertex input set: bra/ket fermions, a vector, and the per-chirality
/// effective couplings (`Σ coupling·coeff` per chiral structure, as constant
/// folding would produce them at bind time).
struct FfvInput {
    bra: Slot,
    ket: Slot,
    v: Slot,
    gl: C<f64>,
    gr: C<f64>,
}

fn gen_inputs(n: usize, seed: u64) -> Vec<FfvInput> {
    let mut rng = seeded_rng(seed);
    (0..n)
        .map(|_| FfvInput {
            bra: rand_bra(&mut rng),
            ket: rand_ket(&mut rng),
            v: rand_vector(&mut rng),
            gl: rand_c(&mut rng),
            gr: rand_c(&mut rng),
        })
        .collect()
}

fn coupling_slot(g: C<f64>) -> Slot {
    WaveformSlot::Scalar(ScalarWf {
        value: g,
        momentum: LorentzVector::zero(),
    })
}

// ─────────────────────────── FFV → off-shell vector current ───────────────────────────
//
// The `Add(Mul(g_L, GammaVout(bra, ProjM(ket))), Mul(g_R, GammaVout(bra, ProjP(ket))))`
// subgraph (e.g. the Z current in ee→μμ).

fn vout_generic_res(inp: &FfvInput, res: &mut Vec<Slot>) -> Slot {
    res.clear();
    res.push(coupling_slot(inp.gl)); // 0: Coupling g_L
    res.push(coupling_slot(inp.gr)); // 1: Coupling g_R
    res.push(proj_m(&[inp.ket])); // 2
    res.push(proj_p(&[inp.ket])); // 3
    res.push(gamma_vout(&[inp.bra, res[2]])); // 4
    res.push(gamma_vout(&[inp.bra, res[3]])); // 5
    res.push(mul_apply([res[0], res[4]])); // 6
    res.push(mul_apply([res[1], res[5]])); // 7
    res.push(res[6] + res[7]); // 8: Add
    res[8]
}

fn vout_generic_direct(inp: &FfvInput) -> Slot {
    let tl = mul_apply([
        coupling_slot(inp.gl),
        gamma_vout(&[inp.bra, proj_m(&[inp.ket])]),
    ]);
    let tr = mul_apply([
        coupling_slot(inp.gr),
        gamma_vout(&[inp.bra, proj_p(&[inp.ket])]),
    ]);
    tl + tr
}

/// Fused FFV vector current: `g_L·J_L + g_R·J_R` in one kernel.
fn vout_fused(inp: &FfvInput) -> Slot {
    let (WaveformSlot::FermionOut(fo), WaveformSlot::FermionIn(fi)) = (&inp.bra, &inp.ket) else {
        panic!("ffv_vout: expected (bra, ket)");
    };
    let jl = fo.spinor.left_current(&fi.spinor);
    let jr = fo.spinor.right_current(&fi.spinor);
    WaveformSlot::Vector(VectorWf {
        eps: jl * inp.gl + jr * inp.gr,
        momentum: fo.momentum - fi.momentum,
    })
}

// ─────────────────────────── FFV → continuing fermion (ket) current ───────────────────────────
//
// The `Add(Mul(g_L, GammaIout(v, ProjM(ket))), Mul(g_R, GammaIout(v, ProjP(ket))))`
// subgraph (a fermion line absorbing a vector).

fn iout_generic_res(inp: &FfvInput, res: &mut Vec<Slot>) -> Slot {
    res.clear();
    res.push(coupling_slot(inp.gl)); // 0
    res.push(coupling_slot(inp.gr)); // 1
    res.push(proj_m(&[inp.ket])); // 2
    res.push(proj_p(&[inp.ket])); // 3
    res.push(gamma_iout(&[inp.v, res[2]])); // 4
    res.push(gamma_iout(&[inp.v, res[3]])); // 5
    res.push(mul_apply([res[0], res[4]])); // 6
    res.push(mul_apply([res[1], res[5]])); // 7
    res.push(res[6] + res[7]); // 8
    res[8]
}

fn iout_generic_direct(inp: &FfvInput) -> Slot {
    let tl = mul_apply([
        coupling_slot(inp.gl),
        gamma_iout(&[inp.v, proj_m(&[inp.ket])]),
    ]);
    let tr = mul_apply([
        coupling_slot(inp.gr),
        gamma_iout(&[inp.v, proj_p(&[inp.ket])]),
    ]);
    tl + tr
}

/// Fused FFV ket current: `ε̸(g_L ψ_L ⊕ g_R ψ_R)` — the slash is linear, so the
/// per-chirality weights combine before a single slash.
fn iout_fused(inp: &FfvInput) -> Slot {
    let (WaveformSlot::Vector(v), WaveformSlot::FermionIn(fi)) = (&inp.v, &inp.ket) else {
        panic!("ffv_iout: expected (vector, ket)");
    };
    let s = &fi.spinor;
    let weighted = Bispinor::from_components([
        s.component(0) * inp.gl,
        s.component(1) * inp.gl,
        s.component(2) * inp.gr,
        s.component(3) * inp.gr,
    ]);
    WaveformSlot::FermionIn(InDiracWf::from_spinor(
        weighted.slash(&v.eps),
        fi.momentum - v.momentum,
    ))
}

fn bench_ffv(c: &mut Criterion) {
    const N: usize = 512;
    let inputs = gen_inputs(N, 0xF05ED);

    // The fused kernels must agree with the generic chains before their timings
    // mean anything (the adoption-grade oracle lives in the kernel tests).
    let mut res = Vec::with_capacity(9);
    for inp in &inputs {
        slots_approx_eq(&vout_fused(inp), &vout_generic_res(inp, &mut res), 1e-13)
            .expect("vout fused == generic");
        slots_approx_eq(&iout_fused(inp), &iout_generic_res(inp, &mut res), 1e-13)
            .expect("iout fused == generic");
    }

    let mut group = c.benchmark_group("ffv_vout");
    group.bench_function("generic_res", |b| {
        let mut res = Vec::with_capacity(9);
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match vout_generic_res(inp, &mut res) {
                    WaveformSlot::Vector(v) => v.eps.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("generic_direct", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match vout_generic_direct(inp) {
                    WaveformSlot::Vector(v) => v.eps.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("fused", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match vout_fused(inp) {
                    WaveformSlot::Vector(v) => v.eps.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.finish();

    let mut group = c.benchmark_group("ffv_iout");
    group.bench_function("generic_res", |b| {
        let mut res = Vec::with_capacity(9);
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match iout_generic_res(inp, &mut res) {
                    WaveformSlot::FermionIn(f) => f.spinor.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("generic_direct", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match iout_generic_direct(inp) {
                    WaveformSlot::FermionIn(f) => f.spinor.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.bench_function("fused", |b| {
        b.iter(|| {
            inputs
                .iter()
                .map(|inp| match iout_fused(inp) {
                    WaveformSlot::FermionIn(f) => f.spinor.component(0).re,
                    _ => unreachable!(),
                })
                .sum::<f64>()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_ffv);
criterion_main!(benches);
