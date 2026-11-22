//! Lines through Majorana fermions and fermion-number-violating vertices, beyond
//! what the MadGraph tables in `standalone_jamps.rs` compare.
//!
//! The per-diagram agreement with MadGraph is `standalone_jamps`'s. This file holds
//! the refusal that has no MadGraph counterpart to compare against.

use std::path::PathBuf;

use vibegraph::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
use vibegraph::ufo::UFOModel;

fn toy_majorana() -> UFOModel {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/ufo/vibegraph_toy_majorana_UFO");
    (*UFOModel::load(&dir, None).expect("the toy Majorana model loads")).clone()
}

/// A vertex joining two identical Majorana fermions through a bare vector current
/// (`γ^μ`, odd under charge conjugation) is refused by name, with its structure: read
/// from either end it gives opposite values, so it has none. The committed toy model
/// carries the axial current there, its own conjugate, and generates; the same model
/// with the vector current put on that vertex does not.
#[test]
fn a_majorana_pair_vertex_odd_under_conjugation_is_refused() {
    let generate = |model: &UFOModel| {
        let card = parse_proc_card("generate lt~ lt > nt nt", &ParsingOptions::default()).unwrap();
        generate_from_proc_card(&card, model)
    };
    let axial = toy_majorana();
    let sets = generate(&axial).expect("the axial nt nt vt vertex generates");
    assert_eq!(sets[0].diagrams.len(), 3);

    let mut vector = toy_majorana();
    let ffv1 = vector
        .lorentz
        .get_index_of("FFV1")
        .expect("FFV1 is in the model");
    vector.vertices["V_ntntvt"].lorentz = vec![ffv1.into()];
    let err = generate(&vector).expect_err("a vector nt nt vt vertex is refused");
    let message = err.to_string();
    for needle in [
        "vertex 'V_ntntvt' joins two 'nt' Majorana fermions",
        "'FFV1'",
        "not its own charge conjugate",
    ] {
        assert!(message.contains(needle), "{needle:?} not in: {message}");
    }
}
