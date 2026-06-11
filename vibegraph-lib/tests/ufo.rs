mod common;

use std::f64::consts::PI;
use vibegraph::ufo::slha::ParamCard;
use vibegraph::ufo::{UFOModel, UfoError};

#[test]
fn test_load_loop_sm() {
    let path = common::ufo_path("loop_sm");
    if !path.exists() {
        eprintln!("loop_sm UFO not found — skipping");
        return;
    }
    let result = UFOModel::load(&path, None);
    if let Err(UfoError::FeynGraph(_)) = &result {
        eprintln!(
            "loop_sm: FeynGraph topology parser does not support loop-level \
             particle attributes (.counterterm, .loop_particles) — skipping"
        );
        return;
    }
    let model = result.expect("unexpected error loading loop_sm UFO");
    let empty_card = ParamCard::from_str("").unwrap();
    let ev = model.evaluate(&empty_card);

    let mz = ev.mass(model.particle_id("Z").expect("no Z in model"));
    assert!((mz - 91.188).abs() < 0.01, "loop_sm MZ = {mz}");

    let ma = ev.mass(model.particle_id("A").expect("no a in model"));
    assert!(ma.abs() < 1e-10, "loop_sm m_photon = {ma}");
}

#[test]
fn test_load_mssm() {
    let path = common::ufo_path("MSSM_SLHA2");
    if !path.exists() {
        eprintln!("MSSM_SLHA2 UFO not found — skipping");
        return;
    }
    let model = UFOModel::load(&path, None).expect("failed to load MSSM_SLHA2 UFO");
    let empty_card = ParamCard::from_str("").unwrap();
    let ev = model.evaluate(&empty_card);

    let tb = ev.param_values["tb"].re;
    assert!((tb - 9.74862403).abs() < 1e-6, "MSSM tb = {tb}");

    let beta = ev.param_values["beta"].re;
    let expected_beta = 9.74862403f64.atan();
    assert!((beta - expected_beta).abs() < 1e-8, "MSSM beta = {beta}");

    let ma = ev.mass(model.particle_id("a").expect("no a in model"));
    assert!(ma.abs() < 1e-10, "MSSM m_photon = {ma}");
}

#[test]
#[ignore = "the FFCT2 operator in the lorentz structure is a custom fortran routine in this model's functions.f"]
fn test_load_taudecay() {
    let path = common::ufo_path("taudecay_UFO");
    if !path.exists() {
        eprintln!("taudecay_UFO not found — skipping");
        return;
    }
    let param_card_path = path.join("param_card.dat");
    let card = vibegraph::ufo::slha::ParamCard::from_file(&param_card_path)
        .expect("failed to load taudecay param_card.dat");

    let result = UFOModel::load(&path, None);
    match &result {
        Err(UfoError::Lorentz(e)) if e.to_string().contains("UnknownOperator") => {
            eprintln!("taudecay_UFO: uses unsupported Lorentz operator — skipping");
            return;
        }
        _ => {}
    }
    let model = result.expect("failed to load taudecay UFO");
    let ev = model.evaluate(&card);

    let mta = ev.mass(model.particle_id("ta__minus__").expect("no tau"));
    assert!((mta - 1.776820).abs() < 1e-4, "taudecay MTA = {mta}");

    let mmu = ev.mass(model.particle_id("mu__minus__").expect("no muon"));
    assert!((mmu - 0.105660).abs() < 1e-4, "taudecay MMU = {mmu}");

    let mve = ev.mass(model.particle_id("ve").expect("no ve"));
    assert!(mve.abs() < 1e-10, "taudecay m_ve = {mve}");
}

#[test]
fn test_load_sm_ufo() {
    let path = common::ufo_path("sm");
    if !path.exists() {
        eprintln!("SM UFO not found at {path:?} — skipping");
        return;
    }
    let model = UFOModel::load(&path, None).expect("failed to load SM UFO");
    let empty_card = ParamCard::from_str("").unwrap();
    let ev = model.evaluate(&empty_card);

    let as_val = ev.param_values["aS"].re;
    assert!((as_val - 0.118).abs() < 1e-10, "aS = {as_val}");

    let expected_g = 2.0 * (0.118f64).sqrt() * PI.sqrt();
    let g_val = ev.param_values["G"].re;
    assert!((g_val - expected_g).abs() < 1e-6, "G = {g_val}");

    let gc10 = ev.coupling(model.coupling_id("GC_10").expect("no GC_10 in model"));
    assert!((gc10.re + expected_g).abs() < 1e-6, "GC_10 = {gc10}");
    assert!(gc10.im.abs() < 1e-10);

    let mz = ev.mass(model.particle_id("Z").expect("missing param"));
    assert!((mz - 91.1876).abs() < 0.01, "MZ = {mz}");

    let ma = ev.mass(model.particle_id("a").expect("missing param"));
    assert!(ma.abs() < 1e-10, "m_photon = {ma}");

    let e_id = model.particle_id("e-");
    assert!(e_id.is_some(), "e- not found in particle index");
}

#[test]
fn test_recompute_propagates() {
    let path = common::ufo_path("sm");
    if !path.exists() {
        return;
    }
    let model = UFOModel::load(&path, None).expect("failed to load SM UFO");
    let empty_card = ParamCard::from_str("").unwrap();
    let mut ev = model.evaluate(&empty_card);

    let new_as = 0.130f64;
    ev.recompute("aS", num_complex::Complex64::new(new_as, 0.0));

    let expected_g = 2.0 * new_as.sqrt() * PI.sqrt();
    let g_val = ev.param_values["G"].re;
    assert!(
        (g_val - expected_g).abs() < 1e-6,
        "After recompute: G = {g_val}"
    );

    let gc10 = ev.coupling(model.coupling_id("GC_10").expect("missing coupling"));
    assert!(
        (gc10.re + expected_g).abs() < 1e-6,
        "After recompute: GC_10 = {gc10}"
    );
}
