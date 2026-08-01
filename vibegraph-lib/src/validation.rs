//! The banked validation layer's declared inputs.
//!
//! Tests are sorted into dependency layers by *where they are registered*: the
//! default `cargo test` suite is the hermetic layer and runs complete on a bare
//! clone, while everything needing the `mg5amcnlo` submodule, a fetched PDF set
//! or a frozen MadGraph run is registered behind `required-features =
//! ["extended-validation"]`. A test that decides at runtime whether it has
//! anything to do defeats that: it reports "ok" having asserted nothing, and the
//! absence of an input becomes indistinguishable from agreement with it.
//!
//! Every input the banked layer reads is acquired before any of it runs, by
//! `pixi run validate`'s own dependency tasks — the submodule checkout, the two
//! PDF sets, and the reference bundle, which unpacks the frozen MadGraph runs of
//! every row the gates iterate over. Each of those tasks fails when it cannot
//! acquire what it names. A banked gate that finds an input missing is therefore
//! looking at an incomplete environment, and [`require`] says so rather than
//! passing.
//!
//! There is no tolerated-skip list. There was one while the reference runs could
//! only be produced locally, machine by machine; the bundle pins every run every
//! gate reads, so a missing one became a failure instead of an exception.
//!
//! `validation/manifest.toml` describes the layers and the per-process coverage.

pub mod samples;

/// Fail a banked gate whose declared input is absent, naming the input and what
/// acquires it.
///
/// `test` is the test function's own name, `input` the missing input, `detail`
/// free text locating it (a path, a run name).
pub fn require(test: &str, input: &str, detail: impl std::fmt::Display) -> ! {
    panic!(
        "`{test}` needs {input} ({detail}). The banked layer declares that as an \
         input, so this is an incomplete environment and not a reason to skip: run \
         `pixi run validate`, whose dependency tasks acquire the reference bundle \
         and the PDF sets, or `pixi run fetch-refdata` / `pixi run fetch-pdf` on \
         their own."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "needs the reference bundle (some/path)")]
    fn a_missing_input_names_itself_and_fails() {
        require("a_gate", "the reference bundle", "some/path");
    }
}
