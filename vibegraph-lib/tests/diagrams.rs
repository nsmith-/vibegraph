mod common;

use vibegraph::diagrams::{
    generate_from_proc_card, parse_proc_card, DiagramError, ParsingOptions, Unsupported,
};

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
    let card = "define myp = u d\ngenerate myp myp > e+ e-\n";
    let opts = ParsingOptions::default();
    let parsed = parse_proc_card(card, &opts).unwrap();
    let table = &parsed.processes[0].aliases;
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

/// Required s-channels keep only the diagrams with that s-channel propagator.
#[test]
fn test_generate_ee_to_mumu_required_z() {
    let sets = common::generate("e+ e- > Z > mu+ mu-");
    assert_eq!(
        sets[0].diagrams.len(),
        1,
        "only Z exchange when Z is required s-channel"
    );
}

/// Forbidden mediators: e+e- → μ+μ- with both γ and Z forbidden has no
/// diagram, which is an error, as MadGraph's `NoDiagramException` is.
#[test]
fn test_no_diagrams_when_both_mediators_forbidden() {
    let card =
        parse_proc_card("generate e+ e- > mu+ mu- / a Z", &ParsingOptions::default()).unwrap();
    let result = generate_from_proc_card(&card, common::sm_model().as_ref());
    assert!(matches!(result, Err(DiagramError::NoDiagrams { .. })));
}

/// A second `/` is not a second restriction list: MadGraph's expression takes
/// the last one, and the first is left among the legs, where it is no particle.
#[test]
fn a_second_slash_is_an_error() {
    assert!(parse_proc_card(
        "generate e+ e- > mu+ mu- / a / Z",
        &ParsingOptions::default()
    )
    .is_err());
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

/// A squared-order constraint is refused rather than dropped.
///
/// `NP^2==1` is how MadGraph is asked for the dimension-six *interference* alone,
/// and it is the first thing a SMEFT user types. It bounds the order of a term in
/// |M|², which is a statement about pairs of diagrams and not about any one of
/// them, so this generator cannot honour it — and answering the amplitude-level
/// question instead would hand back a cross section that is not the one asked for,
/// with nothing on screen to say so.
///
/// The unconstrained control is what makes this a test of the refusal rather than
/// of the process string: the same legs without the `^2` enumerate normally.
#[test]
fn a_squared_order_constraint_is_a_hard_error() {
    let opts = ParsingOptions::default();
    let model = common::sm_model();

    let text = match parse_proc_card("generate e+ e- > mu+ mu- QED^2==2", &opts) {
        Err(e) => e.to_string(),
        Ok(card) => panic!(
            "a squared-order constraint was accepted: {}",
            card.processes[0]
        ),
    };
    assert!(text.contains("QED^2==2"), "{text}");
    assert!(text.contains("QED<=n"), "{text}");

    let control = parse_proc_card("generate e+ e- > mu+ mu- QED<=2", &opts).unwrap();
    assert!(
        total_diagrams(
            &vibegraph::diagrams::generate_from_proc_card(&control, model.as_ref())
                .expect("the same legs without the `^2` enumerate")
        ) > 0
    );
}

fn enumerate(card: &str) -> Result<Vec<vibegraph::diagrams::DiagramSet>, String> {
    let model = common::sm_model();
    let card = parse_proc_card(card, &ParsingOptions::default()).map_err(|e| e.to_string())?;
    vibegraph::diagrams::generate_from_proc_card(&card, model.as_ref()).map_err(|e| e.to_string())
}

/// Two process lines that reach the same subprocess would both be summed into
/// the cross section. MadGraph lets this through whenever the two lines' process
/// numbers differ (and `p p > e+ e-` with `u u~ > e+ e-` even under the same
/// number); here it is refused, `--no_warning=duplicate` or not.
#[test]
fn a_subprocess_reached_from_two_lines_is_refused() {
    for card in [
        "generate p p > e+ e-\nadd process u u~ > e+ e-\n",
        "generate e+ e- > mu+ mu- @1\nadd process e+ e- > mu+ mu- @2\n",
        "generate e+ e- > mu+ mu-\nadd process e+ e- > mu- mu+ --no_warning=duplicate\n",
        "generate p p > e+ e-\nadd process u~ u > e+ e-\n",
        "generate u u~ > e+ e-\nadd process u~ u > e+ e-\nadd process u u~ > e+ e-\n",
    ] {
        let Err(err) = enumerate(card) else {
            panic!("{card} enumerated");
        };
        assert!(err.contains("two process lines"), "{card}: {err}");
    }
    let sets = enumerate("generate p p > e+ e-\nadd process p p > mu+ mu-\n").unwrap();
    assert_eq!(
        sets.iter().filter(|s| !s.diagrams.is_empty()).count(),
        8,
        "disjoint lines enumerate side by side"
    );
}

/// The populated sets of a card as `(incoming, mirrored)`, in enumeration order.
fn orderings(card: &str) -> Vec<(Vec<String>, bool)> {
    enumerate(card)
        .unwrap_or_else(|e| panic!("{card}: {e}"))
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .map(|s| (s.particles_in, s.mirrored))
        .collect()
}

fn names(legs: &[&str]) -> Vec<String> {
    legs.iter().map(|l| (*l).to_owned()).collect()
}

/// A subprocess stands for its exchanged beam ordering only when its own
/// process line generated that ordering too, as MadGraph's
/// `has_mirror_process` is set: a line written in one ordering has no mirror,
/// a multiparticle line that reaches both has one, and two lines that write the
/// two orderings are two subprocesses without one, one per line.
#[test]
fn a_subprocess_has_a_mirror_only_where_its_line_generated_both_orderings() {
    for card in [
        "generate u u~ > e+ e-",
        "generate u~ u > e+ e-",
        "generate u g > u z",
        "generate u u > u u",
        "generate u u~ > z, z > e+ e-",
    ] {
        let sets = orderings(card);
        assert_eq!(sets.len(), 1, "{card}");
        assert!(
            !sets[0].1,
            "{card}: one ordering written, a mirror recorded"
        );
    }

    for card in [
        "generate p p > e+ e-",
        "define q = u u~\ngenerate q q > e+ e-",
        "generate p p > z, z > e+ e-",
    ] {
        let sets = orderings(card);
        let uu = sets
            .iter()
            .filter(|(i, _)| {
                let mut i = i.clone();
                i.sort();
                i == names(&["u", "u~"])
            })
            .collect::<Vec<_>>();
        assert_eq!(uu.len(), 1, "{card}: one u u~ subprocess, {sets:?}");
        assert!(
            uu[0].1,
            "{card}: both orderings generated, no mirror recorded"
        );
        for (incoming, mirrored) in &sets {
            assert_eq!(
                *mirrored,
                incoming[0] != incoming[1],
                "{card}: {incoming:?} — a multiparticle line generates both orderings"
            );
        }
    }

    // `u u` and `g g` have no exchanged ordering; `u g` has, and its line
    // generates it.
    let sets = orderings("define q = u g\ngenerate q q > q q");
    assert_eq!(sets.len(), 3, "{sets:?}");
    assert!(sets.iter().all(|(i, m)| *m == (i[0] != i[1])), "{sets:?}");

    for (card, first, second) in [
        (
            "generate u u~ > e+ e-\nadd process u~ u > e+ e-",
            ["u", "u~"],
            ["u~", "u"],
        ),
        (
            "generate u g > u z\nadd process g u > u z",
            ["u", "g"],
            ["g", "u"],
        ),
    ] {
        assert_eq!(
            orderings(card),
            vec![(names(&first), false), (names(&second), false)],
            "{card}"
        );
    }
}

/// At fixed-energy beams the two orderings of one initial state are the same
/// collision, so two lines writing them are refused there, while the
/// enumeration, which proton beams share, keeps both.
#[test]
fn two_lines_in_exchanged_orderings_are_refused_at_fixed_energy() {
    use vibegraph::multiplicity::refuse_exchanged_beam_orderings;
    let card = "generate e+ e- > mu+ mu-\nadd process e- e+ > mu- mu+\n";
    let sets = enumerate(card).unwrap();
    let err = refuse_exchanged_beam_orderings(&sets).expect_err("refused");
    assert!(err.contains("two beam orderings"), "{err}");
    for card in [
        "generate e+ e- > mu+ mu-\nadd process e+ e- > ta+ ta-\n",
        "generate u u~ > e+ e-\nadd process d d~ > e+ e-\n",
    ] {
        refuse_exchanged_beam_orderings(&enumerate(card).unwrap())
            .unwrap_or_else(|e| panic!("{card}: {e}"));
    }
}

/// An integer leg is a PDG code: `21` is a gluon, not two particles named `1`.
#[test]
fn pdg_code_legs_enumerate_as_their_particles() {
    let by_code = enumerate("generate 11 -11 > 13 -13").unwrap();
    let by_name = enumerate("generate e- e+ > mu- mu+").unwrap();
    assert_eq!(by_code[0].particles_in, by_name[0].particles_in);
    assert_eq!(by_code[0].diagrams.len(), 2);
    assert_eq!(
        total_diagrams(&enumerate("generate 21 21 > 21 21").unwrap()),
        4
    );
}

/// `generate` discards the processes before it, as MadGraph's does.
#[test]
fn a_second_generate_starts_over() {
    let sets = enumerate("generate e+ e- > mu+ mu-\ngenerate e+ e- > ta+ ta-\n").unwrap();
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].particles_out, ["ta+", "ta-"]);
}

/// An order the model does not define constrains nothing in MadGraph (which
/// accepts `EW` on a model whose order is `QED`, then bounds the name as
/// written); here it is refused.
#[test]
fn an_order_the_model_does_not_define_is_refused() {
    let Err(err) = enumerate("generate e+ e- > mu+ mu- EW=2") else {
        panic!("EW=2 enumerated");
    };
    assert!(err.contains("not an order of this model"), "{err}");
}

/// 1→n decays enumerate with one incoming leg: the diagram census against
/// MadGraph's own generation at the pinned version (`import model sm`, one
/// `generate` per row), subprocess by subprocess.
#[test]
fn decay_processes_match_madgraphs_diagram_census() {
    for (process, subprocesses, diagrams) in [
        ("t > w+ b", 1, 1),
        ("t > b e+ ve", 1, 1),
        ("h > e+ e- mu+ mu-", 1, 1),
        ("z > e+ e-", 1, 1),
        ("t > b e+ ve a", 1, 4),
        ("t > w+ b g", 1, 2),
        ("z > e+ e- a", 1, 2),
        ("w+ > j j", 2, 2),
        ("h > b b~ g", 1, 2),
        ("z > e+ e- mu+ mu-", 1, 8),
        ("h > w+ e- ve~", 1, 1),
    ] {
        let sets = common::generate(process);
        let populated: Vec<_> = sets.iter().filter(|s| !s.diagrams.is_empty()).collect();
        assert_eq!(populated.len(), subprocesses, "{process}: subprocesses");
        assert_eq!(total_diagrams(&sets), diagrams, "{process}: diagrams");
        for set in populated {
            assert_eq!(set.particles_in.len(), 1, "{process}");
            assert!(set.diagrams.iter().all(|d| d.n_in == 1), "{process}");
        }
    }
}

/// A card cannot mix decays with scattering processes: MadGraph refuses the
/// second line (`madgraph_interface.py`), and so does the check.
#[test]
fn a_decay_cannot_share_a_card_with_a_scattering_process() {
    let err = parse_proc_card(
        "generate t > w+ b\nadd process e+ e- > mu+ mu-\n",
        &ParsingOptions::default(),
    )
    .unwrap_err();
    let vibegraph::diagrams::DiagramError::Unsupported(all) = err else {
        panic!("{err}");
    };
    assert!(matches!(
        all.0[..],
        [Unsupported::MixedInitialStates { .. }]
    ));
}

/// `/ w+` forbids the W propagator in either orientation: MadGraph compares
/// |PDG code| (`diagram_generation.py`, the `abs(...) in forbidden_particles`
/// test), and `e+ e- > e+ ve e- ve~` keeps 20 of its 56 diagrams under `/ w+`,
/// `/ w-` and `/ w+ w-` alike (MadGraph's own generation at the pinned
/// version). Forbidding only the named orientation keeps 26 or 44 — a
/// different diagram set for each spelling of the same restriction.
#[test]
fn a_forbidden_particle_is_forbidden_in_either_orientation() {
    let count = |card: &str| total_diagrams(&enumerate(card).unwrap());
    assert_eq!(count("generate e+ e- > e+ ve e- ve~"), 56);
    for forbid in ["w+", "w-", "w+ w-", "24", "-24"] {
        assert_eq!(
            count(&format!("generate e+ e- > e+ ve e- ve~ / {forbid}")),
            20,
            "/ {forbid}"
        );
    }
}
