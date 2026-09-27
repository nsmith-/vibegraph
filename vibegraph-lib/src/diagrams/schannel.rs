//! Diagram selection by s-channel propagators: MadGraph's required (`> A >`)
//! and forbidden (`$$ A`) s-channels.
//!
//! Both are pure diagram filters (`diagram_generation.py:715` and `:742`): no
//! propagator is treated differently in the amplitude or the phase space, a
//! diagram is only kept or dropped. They are read off a converted [`Diagram`],
//! whose propagators carry the external momenta they are the sum of.
//!
//! **Orientation.** MadGraph compares the restriction lists with
//! `Vertex.get_s_channel_id` (`base_objects.py:2435`): the propagator's
//! particle *as it flows towards the final state*, so `u d~ > w- > e+ ve` has no
//! diagram (the `W` there is a `W+`) and `$$ t` leaves an s-channel `t~` alone.
//! The direction is read off the propagator's momentum: a line is s-channel
//! when it is not spacelike, and its momentum then carries positive energy from
//! the side holding the initial state to the side holding only final-state
//! legs. [`Prop::momentum`] is in the basis where incoming momenta flow in and
//! outgoing ones flow out, and is fixed only up to the null combination
//! Σ p_in − Σ p_out; the energy is evaluated at a point where every external
//! energy is positive and that combination vanishes, which makes the sign
//! independent of the representative.
//!
//! With one initial particle every propagator is s-channel
//! (`get_s_channel_id` with `ninitial == 1`), oriented away from the decaying
//! particle.

use crate::ufo::UFOModel;

use super::diagram::{Diagram, LegIdx, Prop, PropIdx};

/// Required and forbidden s-channels of one process, as PDG codes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SChannelFilter {
    /// Alternatives (or), each a list of ids that must all be s-channels (and).
    pub required: Vec<Vec<i64>>,
    /// Ids no s-channel may carry.
    pub forbidden: Vec<i64>,
}

impl SChannelFilter {
    /// Whether the filter keeps every diagram.
    pub fn is_empty(&self) -> bool {
        self.required.iter().all(Vec::is_empty) && self.forbidden.is_empty()
    }

    /// Whether `diagram` passes: some alternative of the required list has every
    /// one of its ids among the diagram's s-channels, and no s-channel is
    /// forbidden. A required id is a membership test, not a count.
    pub fn keeps(&self, diagram: &Diagram, model: &UFOModel) -> bool {
        if self.is_empty() {
            return true;
        }
        let ids = s_channel_ids(diagram, model);
        let required = self.required.iter().all(Vec::is_empty)
            || self
                .required
                .iter()
                .any(|and| and.iter().all(|id| ids.contains(id)));
        required && !ids.iter().any(|id| self.forbidden.contains(id))
    }
}

/// The oriented ids of a diagram's s-channel propagators, sorted.
pub fn s_channel_ids(diagram: &Diagram, model: &UFOModel) -> Vec<i64> {
    let mut ids: Vec<i64> = diagram
        .props
        .iter()
        .filter_map(|prop| oriented_s_channel_id(prop, diagram, model))
        .collect();
    ids.sort_unstable();
    ids
}

/// The PDG code of `prop`'s particle as it flows towards the final state, or
/// `None` for a spacelike (t-channel) line.
pub fn oriented_s_channel_id(prop: &Prop, diagram: &Diagram, model: &UFOModel) -> Option<i64> {
    let n_in = diagram.n_in;
    if prop.is_spacelike(n_in) {
        return None;
    }
    let particle = model.particle(prop.particle);
    let forward = towards_final_state(prop, n_in, diagram.n_ext());
    Some(if forward || particle.name == particle.antiname {
        particle.pdg_code
    } else {
        -particle.pdg_code
    })
}

/// Whether `prop`'s momentum, `endpoints[0] → endpoints[1]`, runs towards the
/// final state: its energy at E_in = n_out for every incoming leg and
/// E_out = n_in for every outgoing one, which conserves energy, is positive.
fn towards_final_state(prop: &Prop, n_in: usize, n_ext: usize) -> bool {
    let n_out = n_ext - n_in;
    let energy: i64 = prop
        .momentum
        .iter()
        .enumerate()
        .map(|(i, &c)| i64::from(c) * if i < n_in { n_out as i64 } else { n_in as i64 })
        .sum();
    debug_assert!(energy != 0, "a timelike line carries energy");
    energy > 0
}

/// A decay-chain resonance a diagram must hold, for selecting the diagrams of an
/// undecayed final state that a decay chain describes: an s-channel line carrying
/// `particle` (as it flows towards the final state) whose final-state side is exactly
/// `daughters` plus the final-state sides of its `decays`, each of those a resonance of
/// the same kind inside it.
///
/// It generalises [`SChannelFilter`]'s membership test to what the line leads to, and
/// is what a stitched decay chain is compared against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resonance {
    /// PDG code, oriented towards the final state.
    pub particle: i64,
    /// The final-state particles the resonance decays to directly, by name.
    pub daughters: Vec<String>,
    /// Resonances among its products, each with its own products.
    pub decays: Vec<Resonance>,
}

impl Resonance {
    /// The final-state particles the resonance ends in, its decays' included, sorted.
    fn content(&self) -> Vec<String> {
        let mut all = self.daughters.clone();
        all.extend(self.decays.iter().flat_map(Resonance::content));
        all.sort();
        all
    }
}

/// The propagators of `diagram` that the `chain` resonances are, in the order of a
/// depth-first walk of `chain` (a resonance before its decays), or `None` when the
/// diagram does not hold every one of them on distinct, non-overlapping lines.
///
/// `names` are the external particles' names, incoming first. With identical particles
/// a diagram may hold the chain in more than one way; the first assignment found is
/// returned.
pub fn match_resonances(
    diagram: &Diagram,
    model: &UFOModel,
    names: &[String],
    chain: &[Resonance],
) -> Option<Vec<PropIdx>> {
    let sides: Vec<Option<(i64, Vec<LegIdx>)>> = (0..diagram.props.len())
        .map(|p| {
            let prop = &diagram.props[p];
            let id = oriented_s_channel_id(prop, diagram, model)?;
            Some((id, diagram.final_state_side(PropIdx(p))?))
        })
        .collect();
    let mut chosen = Vec::new();
    let all_final: Vec<LegIdx> = (diagram.n_in..diagram.n_ext()).map(LegIdx).collect();
    assign(chain, &all_final, &sides, names, &mut chosen).then_some(chosen)
}

/// Place `resonances`, siblings inside the final-state legs `within`, onto propagators
/// not yet in `chosen`, disjoint from each other; on success `chosen` holds them and
/// their decays in walk order.
fn assign(
    resonances: &[Resonance],
    within: &[LegIdx],
    sides: &[Option<(i64, Vec<LegIdx>)>],
    names: &[String],
    chosen: &mut Vec<PropIdx>,
) -> bool {
    let Some((first, rest)) = resonances.split_first() else {
        return true;
    };
    let content = first.content();
    for (p, side) in sides.iter().enumerate() {
        let Some((id, legs)) = side else { continue };
        if *id != first.particle
            || chosen.contains(&PropIdx(p))
            || !legs.iter().all(|l| within.contains(l))
        {
            continue;
        }
        let mut names_here: Vec<String> = legs.iter().map(|l| names[l.0].clone()).collect();
        names_here.sort();
        if names_here != content {
            continue;
        }
        let mark = chosen.len();
        chosen.push(PropIdx(p));
        // The decays sit inside this line; the siblings outside it.
        let outside: Vec<LegIdx> = within
            .iter()
            .copied()
            .filter(|l| !legs.contains(l))
            .collect();
        if assign(&first.decays, legs, sides, names, chosen)
            && direct_daughters_match(first, legs, &chosen[mark + 1..], sides, names)
            && assign(rest, &outside, sides, names, chosen)
        {
            return true;
        }
        chosen.truncate(mark);
    }
    false
}

/// Whether the legs of `legs` outside the lines of the resonance's decays (the first
/// `decays.len()` top-level entries of `inner`, which lists them in walk order) are its
/// stated direct daughters.
fn direct_daughters_match(
    resonance: &Resonance,
    legs: &[LegIdx],
    inner: &[PropIdx],
    sides: &[Option<(i64, Vec<LegIdx>)>],
    names: &[String],
) -> bool {
    let mut covered: Vec<LegIdx> = Vec::new();
    for p in inner {
        if let Some((_, side)) = &sides[p.0] {
            covered.extend(side);
        }
    }
    let mut direct: Vec<String> = legs
        .iter()
        .filter(|l| !covered.contains(l))
        .map(|l| names[l.0].clone())
        .collect();
    direct.sort();
    let mut stated = resonance.daughters.clone();
    stated.sort();
    direct == stated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::sm::{sm_model, SMRestrict};

    fn ids(process: &str) -> Vec<Vec<i64>> {
        let model = sm_model(SMRestrict::Default);
        let card =
            parse_proc_card(&format!("generate {process}"), &ParsingOptions::default()).unwrap();
        let mut out: Vec<Vec<i64>> = generate_from_proc_card(&card, &model)
            .unwrap()
            .iter()
            .flat_map(|s| s.diagrams.iter().map(|d| s_channel_ids(d, &model)))
            .collect();
        out.sort();
        out
    }

    /// The `W` of `u d~ > e+ ve` is a `W+` flowing to the final state, whichever
    /// way feyngraph happened to orient the line; the crossed `e+ ve > u d~`
    /// (initial and final swapped) sees the same line as a `W+` too.
    #[test]
    fn a_w_is_named_as_it_flows_to_the_final_state() {
        assert_eq!(ids("u d~ > e+ ve"), [vec![24]]);
        assert_eq!(ids("d u~ > e- ve~"), [vec![-24]]);
        assert_eq!(ids("e+ ve > u d~"), [vec![24]]);
    }

    /// Spacelike lines are not s-channels: `e+ e- > e+ e-` has two s-channel
    /// diagrams and two t-channel ones.
    #[test]
    fn spacelike_lines_are_not_s_channels() {
        assert_eq!(ids("e+ e- > e+ e-"), [vec![], vec![], vec![22], vec![23]]);
    }
}
