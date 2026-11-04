//! Heap bytes a loaded PDF member keeps, against the `x·f` knots it carries.
//!
//! A member is stored as its knots — the `x·f` grid, its x and Q² axes and a
//! few per-band constants — and the interpolation polynomials are built from
//! them on use. One `f64` per knot is the floor; a member that carries a
//! derived table per knot (precomputed polynomial coefficients, a second copy of
//! the grid) costs several times that, and the CMS systematics request loads 880
//! members at once. This binary installs a counting allocator, so every byte a
//! member allocates is seen whatever structure holds it.

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::Path;
use std::sync::atomic::{AtomicIsize, Ordering::Relaxed};

use vibegraph::pdf::grid::SubGrid;
use vibegraph::pdf::{PdfMember, PdfSet};

struct Counting;

/// Bytes currently allocated by the process. This binary holds a single test,
/// so nothing else allocates while one is measured.
static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as isize, Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Relaxed);
        System.dealloc(ptr, layout)
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as isize, Relaxed);
        System.alloc_zeroed(layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE.fetch_add(new_size as isize - layout.size() as isize, Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The allowance over one `f64` per knot: the axes, their logarithms, the Q²
/// stencil constants and the flavour maps are per band rather than per knot,
/// and a quarter of the grid covers them on every real member with room to
/// spare. A second per-knot table would need a full extra `f64` per knot.
const BYTES_PER_KNOT_LIMIT: f64 = 8.0 * 1.25;

fn knots(member: &PdfMember) -> usize {
    member.subgrids().iter().map(|g| g.xf.len()).sum()
}

/// The heap a member holds: everything allocated from before its grid exists
/// until it is built, with nothing freed in between counted against it.
fn held_bytes(build: impl FnOnce() -> PdfMember) -> (PdfMember, f64) {
    let before = LIVE.load(Relaxed);
    let member = build();
    let held = LIVE.load(Relaxed) - before;
    (member, held as f64)
}

/// Geometric knots from `a` to `b`, as PDF grids space them.
fn geomspace(a: f64, b: f64, n: usize) -> Vec<f64> {
    let (la, lb) = (a.ln(), b.ln());
    (0..n)
        .map(|i| (la + (lb - la) * i as f64 / (n - 1) as f64).exp())
        .collect()
}

/// A member shaped like an NNPDF one: 100 x knots, three Q² bands and 13
/// flavours.
fn synthetic_member() -> PdfMember {
    let x = geomspace(1e-9, 1.0, 100);
    let flavors: Vec<i32> = vec![-6, -5, -4, -3, -2, -1, 21, 1, 2, 3, 4, 5, 6];
    let bands = [(1.65, 4.92), (4.92, 1.0e4), (1.0e4, 1.0e10)];
    let subgrids = bands
        .iter()
        .enumerate()
        .map(|(b, &(lo, hi))| {
            let q2 = geomspace(lo, hi, if b == 0 { 4 } else { 20 });
            let mut xf = Vec::with_capacity(x.len() * q2.len() * flavors.len());
            for &xv in &x {
                for &q2v in &q2 {
                    for &fl in &flavors {
                        xf.push(
                            (1.0 - xv).powi(3) * xv.powf(0.1) * (1.0 + 0.01 * q2v.ln())
                                / (1.0 + fl.abs() as f64),
                        );
                    }
                }
            }
            SubGrid {
                x: x.clone(),
                q2,
                flavors: flavors.clone(),
                xf,
            }
        })
        .collect();
    PdfMember::from_subgrids(subgrids)
}

#[test]
fn a_loaded_member_holds_about_one_f64_per_knot() {
    let (member, held) = held_bytes(synthetic_member);
    let n = knots(&member);
    let per_knot = held / n as f64;
    println!("synthetic member: {n} knots, {held} bytes held, {per_knot:.2} bytes/knot");
    assert!(
        per_knot <= BYTES_PER_KNOT_LIMIT,
        "a synthetic member holds {per_knot:.2} bytes per knot, over {BYTES_PER_KNOT_LIMIT}: \
         it carries a derived table beside its knots"
    );
    // The member reads: a held-bytes figure from a member that evaluates
    // nothing would pin an empty structure.
    assert!(member.xfx_q2(21, 0.01, 100.0) > 0.0);

    // A real member, read from its `.dat`, where the set is on disk.
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../validation/pdf/NNPDF23_lo_as_0130_qed");
    if dir.join("NNPDF23_lo_as_0130_qed.info").exists() {
        let set = PdfSet::load(&dir, "NNPDF23_lo_as_0130_qed").expect("the set's .info");
        for id in [0, 1, 100] {
            let (member, held) = held_bytes(|| set.member(id).expect("the member loads"));
            let n = knots(&member);
            let per_knot = held / n as f64;
            println!("NNPDF23_lo_as_0130_qed member {id}: {n} knots, {held} bytes held, {per_knot:.2} bytes/knot");
            assert!(
                per_knot <= BYTES_PER_KNOT_LIMIT,
                "NNPDF23_lo_as_0130_qed member {id} holds {per_knot:.2} bytes per knot, \
                 over {BYTES_PER_KNOT_LIMIT}"
            );
        }
    } else {
        println!("NNPDF23_lo_as_0130_qed not in validation/pdf: the real member is not measured");
    }
}
