//! Lines through Majorana fermions and fermion-number-violating vertices, beyond
//! what the MadGraph tables in `standalone_jamps.rs` compare.
//!
//! The per-diagram agreement with MadGraph is `standalone_jamps`'s. This file holds
//! the refusals that have no MadGraph counterpart to compare against.

use std::path::PathBuf;
use std::sync::Arc;

use vibegraph::diagrams::{
    generate_from_proc_card, parse_proc_card, DiagramError, DiagramSet, ParsingOptions,
};
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

/// The committed toy model `name`, loaded from a scratch copy with each `(file, tail)`
/// appended to its file.
fn toy_with(name: &str, appends: &[(&str, &str)]) -> Arc<UFOModel> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../validation/ufo")
        .join(name);
    let scratch = tempfile::tempdir().expect("create a scratch directory");
    for entry in std::fs::read_dir(&src).expect("read the toy model directory") {
        let path = entry.expect("read a toy model entry").path();
        if path.is_file() {
            std::fs::copy(&path, scratch.path().join(path.file_name().unwrap())).unwrap();
        }
    }
    for (file, tail) in appends {
        let path = scratch.path().join(file);
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("{text}\n{tail}\n")).unwrap();
    }
    UFOModel::load(scratch.path(), None).expect("the edited toy model loads")
}

fn generate(process: &str, model: &UFOModel) -> Result<Vec<DiagramSet>, DiagramError> {
    let card = parse_proc_card(&format!("generate {process}"), &ParsingOptions::default())
        .expect("the process card parses");
    generate_from_proc_card(&card, model)
}

/// A four-fermion contact structure, the two pairs `(1, 2)` and `(3, 4)`.
const FFFF: (&str, &str) = (
    "lorentz.py",
    "FFFF1 = Lorentz(name='FFFF1', spins=[2, 2, 2, 2], \
     structure='Identity(2,1)*Identity(4,3)')",
);

/// A Majorana fermion on a vertex of four fermions is refused by name, with the
/// vertex: a four-fermion structure has no single charge-conjugated form, and
/// MadGraph's ALOHA refuses it too. The same model without the contact vertex
/// generates the process.
#[test]
fn a_majorana_fermion_on_a_four_fermion_vertex_is_refused() {
    let model = toy_with(
        "vibegraph_toy_majorana_UFO",
        &[
            FFFF,
            (
                "vertices.py",
                "V_ntntltlt = Vertex(name='V_ntntltlt', \
                 particles=[P.nt, P.nt, P.lt__tilde__, P.lt], color=['1'], \
                 lorentz=[L.FFFF1], couplings={(0, 0): C.GC_LV})",
            ),
        ],
    );
    let message = generate("lt~ lt > nt nt", &model)
        .expect_err("a Majorana fermion on a four-fermion vertex is refused")
        .to_string();
    assert!(
        message.contains(
            "particle 'nt' is a Majorana fermion (its own antiparticle) on the \
             four-fermion vertex 'V_ntntltlt'"
        ),
        "{message}"
    );
}

/// A four-fermion vertex whose pairing joins two particles on one line is refused
/// by name, with the pair; the vertex keeps total fermion number, so only the
/// pairing is at fault.
#[test]
fn a_four_fermion_vertex_pairing_two_particles_is_refused() {
    let model = toy_with(
        "vibegraph_toy_fnv_UFO",
        &[
            FFFF,
            (
                "vertices.py",
                "V_atbtatbt = Vertex(name='V_atbtatbt', \
                 particles=[P.at, P.bt, P.at__tilde__, P.bt__tilde__], color=['1'], \
                 lorentz=[L.FFFF1], couplings={(0, 0): C.GC_AV})",
            ),
        ],
    );
    let message = generate("at bt > at bt", &model)
        .expect_err("a fermion-number-violating four-fermion pairing is refused")
        .to_string();
    assert!(
        message.contains(
            "vertex 'V_atbtatbt' joins 'at' and 'bt' on one fermion line as two \
             incoming particles"
        ),
        "{message}"
    );
}

/// A decay chain whose stitched diagram holds a Majorana fermion is refused: each
/// line is oriented per core and per decay, and a line crossing the stitched
/// propagator is not oriented as one. The core process alone generates.
#[test]
fn a_decay_chain_through_a_majorana_fermion_is_refused() {
    let model = toy_majorana();
    generate("lt~ lt > sc sc~", &model).expect("the core process generates");
    let err = generate("lt~ lt > sc sc~, sc > lt nt", &model)
        .expect_err("a decay chain with a Majorana fermion is refused");
    assert!(
        matches!(&err, DiagramError::DecayChain { reason, .. }
            if reason.contains("a decay-chain diagram carries a Majorana fermion")),
        "{err}"
    );
}
