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

/// The particles a stitched set's forced lines decay into, per diagram: the names on the
/// final-state side of each forced propagator, sorted.
fn forced_products(set: &vibegraph::diagrams::DiagramSet) -> Vec<Vec<Vec<String>>> {
    use vibegraph::diagrams::diagram::{OnShell, PropIdx};
    set.diagrams
        .iter()
        .map(|d| {
            let mut lines: Vec<Vec<String>> = (0..d.props.len())
                .filter(|&p| d.props[p].onshell == OnShell::Forced)
                .map(|p| {
                    let mut names: Vec<String> = d
                        .final_state_side(PropIdx(p))
                        .expect("a forced line is an s-channel")
                        .iter()
                        .map(|leg| set.particles_out[leg.0 - d.n_in].clone())
                        .collect();
                    names.sort();
                    names
                })
                .collect();
            lines.sort();
            lines
        })
        .collect()
}

/// One subprocess reached by two decay assignments of one line is two subprocesses, as
/// MadGraph's two combined matrix elements are (`g g > z j j, z > j j`: six such
/// contents, each 8 + 8 diagrams in MadGraph). Each keeps its own assignment's forced
/// line: Z → uū on the cc̄ core and Z → cc̄ on the uū core.
#[test]
fn a_subprocess_from_two_decay_assignments_is_two_subprocesses() {
    use std::collections::BTreeMap;
    let sets = enumerate("generate g g > z j j, z > j j").unwrap();
    let sets: Vec<_> = sets
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .collect();
    // Four cores (g g > z q q~) times four decays (z > q q~).
    assert_eq!(sets.len(), 16);
    let mut by_content: BTreeMap<Vec<String>, Vec<&vibegraph::diagrams::DiagramSet>> =
        BTreeMap::new();
    for set in &sets {
        let mut content = set.particles_out.clone();
        content.sort();
        by_content.entry(content).or_default().push(set);
    }
    let shared: Vec<_> = by_content.iter().filter(|(_, v)| v.len() > 1).collect();
    assert_eq!(shared.len(), 6, "u c, u d, u s, c d, c s, d s");
    for (content, pair) in shared {
        assert_eq!(pair.len(), 2, "{content:?}");
        let [a, b] = [pair[0], pair[1]];
        assert_eq!(a.diagrams.len(), 8, "{}", a.label());
        assert_eq!(b.diagrams.len(), 8, "{}", b.label());
        // The Z's products lead each final state, so the two orders differ.
        assert_ne!(a.particles_out, b.particles_out);
        let (fa, fb) = (forced_products(a), forced_products(b));
        assert!(fa.iter().all(|lines| lines.len() == 1 && *lines == fa[0]));
        assert!(fb.iter().all(|lines| lines.len() == 1 && *lines == fb[0]));
        let leading = |names: &[String]| {
            let mut pair = names[..2].to_vec();
            pair.sort();
            pair
        };
        assert_eq!(fa[0][0], leading(&a.particles_out));
        assert_eq!(fb[0][0], leading(&b.particles_out));
        assert_ne!(fa[0], fb[0], "{content:?}: both sets decay the same pair");
    }
    let identical = by_content.values().filter(|v| v.len() == 1).count();
    assert_eq!(identical, 4, "g g > q q~ q q~, one assignment each");
}

/// The same two assignments written as two lines are two subprocesses too: MadGraph
/// sums them, as one matrix element with two PDF terms when their process numbers agree
/// (`@1` on both lines) and as two matrix elements otherwise. Each keeps its own forced
/// line. So do two lines whose resonances differ (`g g > w+ d u~, w+ > u d~` and
/// `g g > w- u d~, w- > d u~` reach `g g > u d~ d u~` through a W+ and through a W-).
#[test]
fn two_assignments_on_two_lines_are_two_subprocesses() {
    for (card, content, decays) in [
        (
            "generate e+ e- > z u u~, z > c c~ @1\nadd process e+ e- > z c c~, z > u u~ @1\n",
            ["c", "c~", "u", "u~"],
            [["c", "c~"], ["u", "u~"]],
        ),
        (
            "generate g g > w+ d u~, w+ > u d~\nadd process g g > w- u d~, w- > d u~\n",
            ["d", "d~", "u", "u~"],
            [["d~", "u"], ["d", "u~"]],
        ),
    ] {
        let sets: Vec<_> = enumerate(card)
            .unwrap_or_else(|e| panic!("{card}: {e}"))
            .into_iter()
            .filter(|s| !s.diagrams.is_empty())
            .collect();
        assert_eq!(sets.len(), 2, "{card}");
        for (set, decay) in sets.iter().zip(decays) {
            let mut names = set.particles_out.clone();
            names.sort();
            assert_eq!(names, content, "{card}: {}", set.label());
            let lines = forced_products(set);
            assert!(!lines.is_empty(), "{card}: {}", set.label());
            for forced in lines {
                assert_eq!(forced, [decay], "{card}: {}", set.label());
            }
        }
    }
}

/// Two lines with different decay assignments of one content keep their own beam
/// orderings: on `p p` lines each set stands for both orderings of a mixed initial
/// state (its mirror) and for one of `g g`, as a single line's sets do.
#[test]
fn two_assignments_on_two_lines_keep_their_mirrors() {
    use std::collections::BTreeMap;
    let card = "generate p p > z u u~, z > c c~\nadd process p p > z c c~, z > u u~\n";
    let sets: Vec<_> = enumerate(card)
        .unwrap_or_else(|e| panic!("{card}: {e}"))
        .into_iter()
        .filter(|s| !s.diagrams.is_empty())
        .collect();
    let mut by_content: BTreeMap<(Vec<String>, Vec<String>), Vec<_>> = BTreeMap::new();
    for set in &sets {
        let mut initial = set.particles_in.clone();
        initial.sort();
        let mut fin = set.particles_out.clone();
        fin.sort();
        by_content.entry((initial, fin)).or_default().push(set);
        assert_eq!(
            set.mirrored,
            set.particles_in[0] != set.particles_in[1],
            "{}: a p p line generates both orderings of a mixed initial state",
            set.label()
        );
    }
    assert!(!by_content.is_empty());
    for (content, pair) in &by_content {
        assert_eq!(pair.len(), 2, "{content:?}: one set per line");
        assert_ne!(forced_products(pair[0])[0], forced_products(pair[1])[0]);
    }
}

/// Two lines with the same decay assignment, or a decay-chain line and a line that is
/// not one, would count the same diagrams twice: they stay refused, as any subprocess
/// two lines reach.
#[test]
fn one_assignment_on_two_lines_is_refused() {
    for card in [
        "generate e+ e- > z u u~, z > c c~ @1\nadd process e+ e- > z u u~, z > c c~ @2\n",
        "generate e+ e- > z u u~, z > c c~\nadd process e+ e- > u u~ c c~\n",
        "generate e+ e- > u u~ c c~\nadd process e+ e- > z c c~, z > u u~\n",
    ] {
        let Err(err) = enumerate(card) else {
            panic!("{card} enumerated");
        };
        assert!(
            err.contains("e+ e- > ") && err.contains("two process lines"),
            "{card}: {err}"
        );
    }
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
