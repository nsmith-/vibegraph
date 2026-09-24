// The threaded instruction dispatcher's handlers chain through `become`.
#![cfg_attr(feature = "threaded-dispatch", feature(explicit_tail_calls))]
#![cfg_attr(feature = "threaded-dispatch", allow(incomplete_features))]

pub mod artifact;
pub mod budget;
pub mod cache;
pub mod config;
pub mod coupling;
pub mod cuts;
pub mod diagrams;
pub mod hadronic;
pub mod helas;
pub mod lhef;
pub mod pdf;
pub mod phasespace;
pub mod progress;
pub mod proton;
pub mod runcard;
pub mod select;
pub mod stats;
pub mod ufo;
pub mod unweight;
// Test support for the banked validation layer, shared by the integration tests
// of both crates; absent from a default build.
#[cfg(feature = "extended-validation")]
pub mod validation;
pub mod vegas;
