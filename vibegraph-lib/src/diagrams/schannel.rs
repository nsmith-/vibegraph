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

use super::diagram::{Diagram, Prop};

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
