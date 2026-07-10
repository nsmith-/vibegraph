mod common;

use vibegraph::diagrams::{parse_proc_card, AliasTable, ParsingOptions};

fn total_diagrams(sets: &[vibegraph::diagrams::DiagramSet]) -> usize {
    sets.iter().map(|s| s.diagrams.len()).sum()
}

#[test]
fn test_parse_proc_card_string() {
    let card = "generate e+ e- > mu+ mu-\nadd process e+ e- > ta+ ta-\n";
    let opts = ParsingOptions::default();
    let parsed = parse_proc_card(card, &opts).unwrap();
    assert_eq!(parsed.processes.len(), 2);
}

#[test]
fn test_alias_table_built_from_defines() {
    let card = "define myp = u d\ngenerate myp > e+ e-\n";
    let opts = ParsingOptions::default();
    let parsed = parse_proc_card(card, &opts).unwrap();
    let table = AliasTable::from_defines(&parsed.defines);
    assert_eq!(table.expand_name("myp"), vec!["u", "d"]);
}

/// e+e- → μ+μ-: two s-channel diagrams (γ and Z).
#[test]
fn test_generate_ee_to_mumu() {
    let sets = common::generate("e+ e- > mu+ mu-");
    assert_eq!(sets.len(), 1);
    assert_eq!(
        sets[0].diagrams.len(),
        2,
        "expected γ and Z exchange diagrams"
    );
}

/// uu~ → gg: three pure-QCD diagrams (s-channel 3g vertex, t- and u-channel quark).
#[test]
fn test_generate_uux_to_gg() {
    let sets = common::generate("u u~ > g g");
    assert_eq!(total_diagrams(&sets), 3);
}

/// gg → uu~: crossing of uu~ → gg, also 3 diagrams.
#[test]
fn test_generate_gg_to_uux() {
    let sets = common::generate("g g > u u~");
    assert_eq!(total_diagrams(&sets), 3);
}

/// gg → gg: four pure-QCD diagrams (s-, t-, u-channel gluon + 4-gluon contact).
#[test]
fn test_generate_gg_to_gg() {
    let sets = common::generate("g g > g g");
    assert_eq!(total_diagrams(&sets), 4);
}

/// uu~ → dd~: automatic WEIGHTED ordering selects only the QCD (gluon) s-channel diagram.
#[test]
fn test_generate_uux_to_ddx_weighted_lo() {
    let sets = common::generate("u u~ > d d~");
    assert_eq!(
        total_diagrams(&sets),
        1,
        "only s-channel gluon at minimum WEIGHTED order"
    );
    let prop = sets[0].diagrams[0]
        .props
        .first()
        .expect("s-channel propagator");
    let prop_name = common::sm_model().particle(prop.particle).name.clone();
    assert_eq!(prop_name, "g", "single diagram should be s-channel gluon");
}

/// uu~ → dd~ QED<=2: explicit constraint admits gluon, photon, Z, and W+.
#[test]
fn test_generate_uux_to_ddx_explicit_qed() {
    let sets = common::generate("u u~ > d d~ QED<=2");
    assert_eq!(
        total_diagrams(&sets),
        4,
        "gluon + photon + Z (s-channel) + W+ (t-channel CKM)"
    );
}

/// Required s-channel filtering not yet implemented.
#[test]
#[ignore = "required_s_channel filtering not yet implemented (selector.rs TODO)"]
fn test_generate_ee_to_mumu_required_z() {
    let sets = common::generate("e+ e- > Z > mu+ mu-");
    assert_eq!(
        sets[0].diagrams.len(),
        1,
        "only Z exchange when Z is required s-channel"
    );
}

/// Forbidden mediators: e+e- → μ+μ- with both γ and Z forbidden gives zero diagrams.
#[test]
fn test_no_diagrams_when_both_mediators_forbidden() {
    let sets = common::generate("e+ e- > mu+ mu- / a / Z");
    assert_eq!(total_diagrams(&sets), 0);
}

/// Forbidden propagator in uu~ → gg leaves only the s-channel gluon diagram.
#[test]
fn test_forbidden_u_propagator_in_uux_to_gg() {
    let sets = common::generate("u u~ > g g / u");
    assert_eq!(
        total_diagrams(&sets),
        1,
        "only s-channel gluon without quark propagators"
    );
}
