//! Skip accounting for the banked validation layer.
//!
//! Tests are sorted into dependency layers by *where they are registered*: the
//! default `cargo test` suite is the hermetic layer and runs complete on a bare
//! clone, while everything needing the `mg5amcnlo` submodule, a fetched PDF set
//! or a frozen MadGraph run is registered behind `required-features =
//! ["extended-validation"]`. A test that decides at runtime whether it has
//! anything to do defeats that: it reports "ok" having asserted nothing, and the
//! absence of an input becomes indistinguishable from agreement with it.
//!
//! The banked layer still has a few gates that iterate over reference runs a
//! given work area need not all hold. Those call [`skip`], which refuses any
//! reason not named in [`EXPECTED_SKIPS`] — so a *new* missing input fails the
//! suite, and every tolerated one is a line in this file that has to be deleted
//! when the input becomes guaranteed.
//!
//! `validation/manifest.toml` describes the layers and the per-process coverage
//! this table is the exception list for.

/// One tolerated runtime skip: the test that may take it, the input whose
/// absence permits it, and why that input is not yet guaranteed.
pub struct ExpectedSkip {
    /// The test function's own name, or the test binary's name when one missing
    /// input silences the whole binary.
    pub test: &'static str,
    /// Stable name of the missing input, from the vocabulary used below.
    pub input: &'static str,
    /// What would make the input guaranteed, so the entry can be deleted.
    pub because: &'static str,
}

/// Prefix every recorded skip prints, so a run's output can be searched for
/// skips mechanically rather than by reading it.
pub const SKIP_MARKER: &str = "VALIDATION-SKIP";

/// The complete set of runtime skips the banked layer tolerates.
pub const EXPECTED_SKIPS: &[ExpectedSkip] = &[
    ExpectedSkip {
        test: "sigma_gate_matches_madgraph",
        input: "madgraph output tree",
        because:
            "the MadGraph work area is produced locally; the fetched reference bundle replaces it",
    },
    ExpectedSkip {
        test: "sigma_gate_matches_madgraph",
        input: "banked run card",
        because: "a banked process whose run card the local work area lacks",
    },
    ExpectedSkip {
        test: "unweighted_sample_reproduces_the_integration_it_came_from",
        input: "madgraph output tree",
        because:
            "the MadGraph work area is produced locally; the fetched reference bundle replaces it",
    },
    ExpectedSkip {
        test: "unweighted_sample_reproduces_the_integration_it_came_from",
        input: "banked madgraph run",
        because: "a banked process the local work area lacks",
    },
    ExpectedSkip {
        test: "banked_files_round_trip_byte_for_byte",
        input: "madgraph output tree",
        because:
            "the MadGraph work area is produced locally; the fetched reference bundle replaces it",
    },
    ExpectedSkip {
        test: "the_round_trip_is_sensitive_to_every_convention_sensitive_field",
        input: "madgraph output tree",
        because:
            "the MadGraph work area is produced locally; the fetched reference bundle replaces it",
    },
    ExpectedSkip {
        test: "the_round_trip_is_sensitive_to_every_convention_sensitive_field",
        input: "banked madgraph run",
        because: "the gluon-initiated run this mutation set needs may be absent locally",
    },
    ExpectedSkip {
        test: "generated_events_serialise_into_a_coherent_file",
        input: "madgraph output tree",
        because:
            "the MadGraph work area is produced locally; the fetched reference bundle replaces it",
    },
    ExpectedSkip {
        test: "generated_events_serialise_into_a_coherent_file",
        input: "banked madgraph run",
        because: "a banked process the local work area lacks",
    },
    ExpectedSkip {
        test: "integrate_default_cuts_reproduces_h7_sigma",
        input: "banked hadronic sigma reference",
        because: "the two dy13 runs are produced locally by the oracle layer",
    },
    ExpectedSkip {
        test: "integrate_mmll_window_reproduces_h7_sigma",
        input: "banked hadronic sigma reference",
        because: "the two dy13 runs are produced locally by the oracle layer",
    },
    ExpectedSkip {
        test: "generated_proton_events_are_coherent_and_madgraph_labelled",
        input: "banked madgraph run or fetched pdf set",
        because: "the llj reference run is produced locally and the PDF set is fetched on consent",
    },
    ExpectedSkip {
        test: "a_different_pdf_set_is_refused",
        input: "banked madgraph run or fetched pdf set",
        because: "the llj reference run is produced locally and the PDF set is fetched on consent",
    },
    ExpectedSkip {
        test: "a_dynamical_scale_card_is_still_refused",
        input: "banked madgraph run or fetched pdf set",
        because: "the llj reference run is produced locally and the PDF set is fetched on consent",
    },
    ExpectedSkip {
        test: "root_override_hook_is_transparent",
        input: "banked amplitude reference csv",
        because:
            "the fixed-grid amplitude tables are regenerated by the oracle layer, not committed",
    },
    ExpectedSkip {
        test: "all_rootings_preserve_amplitude",
        input: "banked amplitude reference csv",
        because:
            "the fixed-grid amplitude tables are regenerated by the oracle layer, not committed",
    },
];

/// Record a tolerated runtime skip, or fail the test if it is not one.
///
/// `test` is the test function's own name and `input` the missing input, both
/// matched against [`EXPECTED_SKIPS`]; `detail` is free text for the log.
pub fn skip(test: &str, input: &str, detail: impl std::fmt::Display) {
    assert!(
        EXPECTED_SKIPS
            .iter()
            .any(|e| e.test == test && e.input == input),
        "`{test}` tried to skip for a missing `{input}` ({detail}), which is not \
         in vibegraph::validation::EXPECTED_SKIPS. Either the input is a declared \
         dependency of the banked layer and its absence is a failure, or the skip \
         belongs in that table with a note saying what would make it unnecessary."
    );
    eprintln!("{SKIP_MARKER} {test}: no {input} ({detail})");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_skips_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for e in EXPECTED_SKIPS {
            assert!(
                seen.insert((e.test, e.input)),
                "duplicate expected skip: {} / {}",
                e.test,
                e.input
            );
            assert!(!e.because.is_empty(), "{}: empty rationale", e.test);
        }
    }

    #[test]
    #[should_panic(expected = "EXPECTED_SKIPS")]
    fn an_unlisted_skip_fails() {
        skip("no_such_test", "no such input", "probe");
    }
}
