//! Compiled helicity programs rendered to Rust and compiled ahead of time.
//!
//! A compiled, helicity-pruned evaluator's helicity-expanded [`Program`] — the stream
//! the forward pass in `run.rs` interprets — is written out as Rust source over the same
//! kernels, in production execution order, in one of two forms:
//!
//! - [`Form::Mg`] ([`mg`]): MadGraph's form. Every value lives in a per-class slot array
//!   with the interpreter's slot assignment; every instruction is one call to an
//!   out-of-line entry point that takes its operands by reference and writes its result
//!   in place.
//! - [`Form::ByValue`] ([`byvalue`]): every value a fresh local, every kernel behind an
//!   out-of-line entry point that returns its result by value.
//!
//! Both perform the interpreter's arithmetic in the interpreter's order, so |M|² is
//! bit-identical to it. [`AotAmplitude::eval_m2`] forms |M|² from the root amplitudes
//! with the read-out `BoundAmplitude::eval_m2` performs.
//!
//! The rendered sources live beside this file (`generated/<form>/<row>.rs`) and are
//! compiled in through `include!`. [`AotAmplitude::new`] re-renders the live program and
//! refuses a source that differs from it, so a stale file is caught at construction
//! rather than read as a timing.
//!
//! [`Program`]: super::layout::Program

mod byvalue;
mod kernels;
mod mg;

use super::layout::RootKind;
use super::run::{assert_partonic_cm_beams_along_z, BoundAmplitude};
use crate::helas::repr::lorentz::LorentzVector;
use crate::helas::repr::{Real, C};

/// One rendered process: its bench-row name, the process string it was compiled from,
/// and its rendered sources, which [`AotAmplitude::new`] checks against the live
/// program. A row whose by-value source is not compiled in has none.
#[derive(Clone, Copy, Debug)]
pub struct AotRow {
    pub name: &'static str,
    pub process: &'static str,
    mg: &'static str,
    by_value: Option<&'static str>,
}

impl AotRow {
    /// Whether `form`'s rendering of this row is compiled in.
    pub fn has(&self, form: Form) -> bool {
        self.source(form).is_some()
    }

    fn source(&self, form: Form) -> Option<&'static str> {
        match form {
            Form::Mg => Some(self.mg),
            Form::ByValue => self.by_value,
        }
    }
}

/// Instructions per function a source was rendered with (`0`: one function), read
/// back from the header both renderers write.
fn header_chunk(source: &str) -> usize {
    source
        .lines()
        .next()
        .unwrap_or_default()
        .split_once("chunk: ")
        .and_then(|(_, rest)| rest.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(usize::MAX)
}

/// Rendered rows.
pub const ROWS: &[AotRow] = &[
    AotRow {
        name: "ee_to_mumu",
        process: "e+ e- > mu+ mu-",
        mg: include_str!("generated/mg/ee_to_mumu.rs"),
        by_value: Some(include_str!("generated/byvalue/ee_to_mumu.rs")),
    },
    AotRow {
        name: "gg_to_gg",
        process: "g g > g g",
        mg: include_str!("generated/mg/gg_to_gg.rs"),
        by_value: Some(include_str!("generated/byvalue/gg_to_gg.rs")),
    },
    AotRow {
        name: "ee_to_mumu_tata_qcd0",
        process: "e+ e- > mu+ mu- ta+ ta- QCD=0",
        mg: include_str!("generated/mg/ee_to_mumu_tata_qcd0.rs"),
        by_value: Some(include_str!("generated/byvalue/ee_to_mumu_tata_qcd0.rs")),
    },
    #[cfg(feature = "aot-mg-study-large")]
    AotRow {
        name: "uux_to_ccx_emmm_qcd0",
        process: "u u~ > c c~ e+ e- mu+ mu- QCD=0",
        mg: include_str!("generated/mg/uux_to_ccx_emmm_qcd0.rs"),
        by_value: None,
    },
];

/// The rendered form of a program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// Values in per-class slot arrays; each instruction an out-of-line call writing its
    /// result in place.
    Mg,
    /// Values in locals; each kernel an out-of-line call returning its result.
    ByValue,
}

/// The source `form` renders `eval`'s program to, as `chunk` instructions per function.
fn render(
    form: Form,
    name: &str,
    eval: &super::compile::AmplitudeEvaluator,
    chunk: usize,
) -> String {
    match form {
        Form::Mg => mg::render(name, eval, chunk),
        Form::ByValue => byvalue::render(name, eval, chunk),
    }
}

enum Rendered<F: Real> {
    Mg(Box<dyn mg::MgRun<F>>),
    ByValue {
        kernel: byvalue::AotKernel<F>,
        roots: Vec<C<F>>,
    },
}

/// A bound amplitude paired with one rendering of its program.
pub struct AotAmplitude<'a, F: Real> {
    amp: &'a BoundAmplitude<'a, F>,
    program: Rendered<F>,
    moms: Vec<LorentzVector<F>>,
}

impl<'a, F: Real> AotAmplitude<'a, F> {
    /// Pair `amp` with row `name` rendered in `form`. Panics if no such rendering is
    /// compiled in, or if its source is not what the live program renders to.
    pub fn new(name: &str, amp: &'a BoundAmplitude<'a, F>, form: Form) -> Self {
        let row = ROWS
            .iter()
            .find(|r| r.name == name)
            .unwrap_or_else(|| panic!("no rendered program for `{name}`"));
        let source = row
            .source(form)
            .unwrap_or_else(|| panic!("`{name}` has no {form:?} rendering compiled in"));
        let eval = amp.evaluator();
        let live = render(form, name, eval, header_chunk(source));
        assert!(
            live == source,
            "{form:?} source for `{name}` is stale: the live program renders differently"
        );
        let RootKind::Hels { locs, .. } = &eval.folded_hel().program().root else {
            panic!("rendered program without a helicity-expanded root");
        };
        let program = match form {
            Form::Mg => Rendered::Mg(mg::bind(name).expect("row listed in ROWS has a binder")),
            Form::ByValue => Rendered::ByValue {
                kernel: byvalue::kernel_for(name).expect("row listed in ROWS has a kernel"),
                roots: vec![C::new(F::zero(), F::zero()); locs.len()],
            },
        };
        AotAmplitude {
            amp,
            program,
            moms: Vec::with_capacity(eval.folded_hel().analysis().mom_table().len()),
        }
    }

    /// The colour- and helicity-summed |M|², as `BoundAmplitude::eval_m2` forms it.
    pub fn eval_m2(&mut self, momenta: &[LorentzVector<F>]) -> F {
        let eval = self.amp.evaluator();
        if momenta.len() != eval.n_ext() {
            return F::zero();
        }
        if eval.is_pruned() {
            assert_partonic_cm_beams_along_z(momenta, eval.n_in());
        }
        let folded = eval.folded_hel();
        let table = folded.analysis().mom_table();
        self.moms.clear();
        for id in 0..table.len() as u32 {
            self.moms.push(table.resolve(id, momenta));
        }
        let (consts_c, consts_f) = self.amp.pools();
        let RootKind::Hels { n_flows, locs } = &folded.program().root else {
            panic!("rendered program without a helicity-expanded root");
        };
        let n = *n_flows as usize;
        let cf = self.amp.cf();
        match &mut self.program {
            Rendered::Mg(p) => {
                p.run(momenta, consts_c, consts_f, &self.moms);
                let s = p.scalars();
                read_out(locs.len(), n, cf, |k| s[locs[k] as usize])
            }
            Rendered::ByValue { kernel, roots } => {
                kernel(momenta, consts_c, consts_f, &self.moms, roots);
                read_out(roots.len(), n, cf, |k| roots[k])
            }
        }
    }
}

/// `BoundAmplitude::eval_m2`'s read-out over `n_roots` root amplitudes in
/// [`RootKind::Hels`] order, `root(k)` the `k`-th: the helicity sum times `CF(1,1)` for
/// one flow, the per-combination CF contraction otherwise.
fn read_out<F: Real>(n_roots: usize, n: usize, cf: &[F], root: impl Fn(usize) -> C<F>) -> F {
    if n == 1 {
        let mut hel_sum = F::zero();
        for k in 0..n_roots {
            hel_sum = hel_sum + root(k).norm_sqr();
        }
        return hel_sum * cf[0];
    }
    let mut total = F::zero();
    for base in (0..n_roots - n_roots % n).step_by(n) {
        for i in 0..n {
            let mut ztemp = C::new(F::zero(), F::zero());
            for j in 0..n {
                ztemp = ztemp + root(base + j).scale(cf[j * n + i]);
            }
            total = total + (ztemp * root(base + i).conj()).re;
        }
    }
    total
}

#[cfg(test)]
mod tests;
