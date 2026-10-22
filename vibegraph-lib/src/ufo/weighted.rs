//! What MadGraph's automatic lowest-WEIGHTED search reads from a model: the
//! largest WEIGHTED of any interaction, which caps the search, and the particle
//! hierarchy classes, which set where it starts. The quantities follow MadGraph
//! 3.x's `base_objects.py`; each cites its lines where it is computed.

use std::collections::{BTreeMap, BTreeSet};

/// The WEIGHTED of one coupling-order tuple: each order's power times its
/// hierarchy, summed (`Interaction.get_WEIGHTED_order`, `base_objects.py:893-899`,
/// before its division by the vertex size).
///
/// An order the hierarchy does not list weighs nothing, and an interaction
/// carrying one joins no particle class (see [`WeightedSearchModel::new`]): the
/// diagram filter the search bounds ignores such an order too, where MadGraph
/// raises `KeyError` (`base_objects.py:897`, `:1448`).
pub(crate) fn order_weight(
    orders: &BTreeMap<String, usize>,
    hierarchy: &BTreeMap<String, u32>,
) -> u32 {
    orders
        .iter()
        .map(|(order, &n)| hierarchy.get(order).copied().unwrap_or(0) * n as u32)
        .sum()
}

/// One interaction as the search sees it: the PDG codes of its particles, as
/// listed, and its single coupling-order tuple (the loader splits a vertex
/// whose couplings differ in their orders into one interaction per tuple).
pub(crate) struct Interaction<'a> {
    pub(crate) pdg_codes: Vec<i64>,
    pub(crate) orders: &'a BTreeMap<String, usize>,
}

/// The model-level inputs of the automatic WEIGHTED search, computed once from
/// the restricted interaction set.
#[derive(Clone)]
pub(crate) struct WeightedSearchModel {
    /// The largest WEIGHTED of any interaction per equivalent three-point
    /// vertex, `WEIGHTED / max(n_particles - 2, 1)` (`Interaction.get_WEIGHTED_order`,
    /// `Model.get_max_WEIGHTED`, `base_objects.py:893-899`, `:1481-1486`),
    /// truncated to an integer as `find_optimal_process_orders` truncates it
    /// (`diagram_generation.py:1999`). A four-point vertex carrying QED=2 under a
    /// QED hierarchy of 2 counts 2, not 4.
    pub(crate) max_interaction_weighted: u32,
    /// Rising hierarchy values, each with the particles first reached by the
    /// interactions all of whose orders sit at or below it
    /// (`Model.get_particles_hierarchy`, `base_objects.py:1432-1479`).
    pub(crate) particle_classes: Vec<(u32, BTreeSet<i64>)>,
}

impl WeightedSearchModel {
    pub(crate) fn new(interactions: &[Interaction<'_>], hierarchy: &BTreeMap<String, u32>) -> Self {
        let max_interaction_weighted = interactions
            .iter()
            .map(|i| {
                let legs = i.pdg_codes.len().saturating_sub(2).max(1) as u32;
                order_weight(i.orders, hierarchy) / legs
            })
            .max()
            .unwrap_or(0);

        let values: BTreeSet<u32> = interactions
            .iter()
            .flat_map(|i| i.orders.keys())
            .filter_map(|order| hierarchy.get(order).copied())
            .collect();
        let mut assigned = vec![false; interactions.len()];
        let mut reached: BTreeSet<i64> = BTreeSet::new();
        let mut particle_classes = Vec::with_capacity(values.len());
        for value in values {
            let within = |order: &String| hierarchy.get(order).is_some_and(|&h| h <= value);
            let mut class = BTreeSet::new();
            for (i, interaction) in interactions.iter().enumerate() {
                if assigned[i] || !interaction.orders.keys().all(within) {
                    continue;
                }
                assigned[i] = true;
                class.extend(
                    interaction
                        .pdg_codes
                        .iter()
                        .filter(|code| !reached.contains(code)),
                );
            }
            reached.extend(&class);
            particle_classes.push((value, class));
        }

        WeightedSearchModel {
            max_interaction_weighted,
            particle_classes,
        }
    }

    /// The bound the search tries first (`ProcessDefinition.get_minimum_WEIGHTED`,
    /// `base_objects.py:3910-3966`).
    ///
    /// Each leg takes the value of the first class holding any of its PDG codes
    /// (a leg in no class takes none); the two smallest values are dropped, as
    /// a tree has `n_ext - 2` three-point vertices, and the rest summed. Each
    /// required s-channel alternative weighs twice the class value of each of
    /// its particles; the lightest alternative, with the `2k` largest leg values
    /// it replaces left out, can raise the start above the legs' sum.
    pub(crate) fn minimum_weighted(
        &self,
        legs: &[Vec<i64>],
        required_s_channels: &[Vec<i64>],
    ) -> u32 {
        let class_of = |id: &i64| {
            self.particle_classes
                .iter()
                .find(|(_, p)| p.contains(id))
                .map(|c| c.0)
        };
        let mut values: Vec<u32> = Vec::with_capacity(legs.len());
        let mut remaining: Vec<&Vec<i64>> = legs.iter().collect();
        for (value, particles) in &self.particle_classes {
            remaining.retain(|ids| {
                let hit = ids.iter().any(|id| particles.contains(id));
                if hit {
                    values.push(*value);
                }
                !hit
            });
        }
        values.sort_unstable();
        let rest: Vec<u32> = values.into_iter().skip(2).collect();
        let legs_sum: u32 = rest.iter().sum();

        let propagator = required_s_channels
            .iter()
            .map(|ids| {
                ids.iter()
                    .filter_map(class_of)
                    .fold((0u32, 0usize), |(w, n), value| (w + 2 * value, n + 1))
            })
            .min_by_key(|&(w, _)| w);
        match propagator {
            None => legs_sum,
            Some((w, n)) => {
                let kept = rest.len().saturating_sub(2 * n);
                legs_sum.max(w + rest[..kept].iter().sum::<u32>())
            }
        }
    }

    /// The bound the search stops at: `(n_ext - 2)` times
    /// [`max_interaction_weighted`](Self::max_interaction_weighted)
    /// (`diagram_generation.py:1998-1999`).
    pub(crate) fn maximum_weighted(&self, n_ext: usize) -> u32 {
        n_ext.saturating_sub(2) as u32 * self.max_interaction_weighted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::UFOModel;

    fn orders(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
        pairs.iter().map(|&(o, n)| (o.to_owned(), n)).collect()
    }

    fn hierarchy(pairs: &[(&str, u32)]) -> BTreeMap<String, u32> {
        pairs.iter().map(|&(o, n)| (o.to_owned(), n)).collect()
    }

    /// Both quartic vertices of the SM carry twice a cubic one's orders, so per
    /// equivalent three-point vertex the SM's largest WEIGHTED is a cubic QED
    /// vertex's, 2: the cap is unchanged from (n_ext - 2) times the largest
    /// hierarchy. Without the division by n_particles - 2 the WWWW and WWZZ
    /// vertices (QED=2, WEIGHTED 4) would double it.
    #[test]
    fn the_sm_cap_is_a_cubic_qed_vertex() {
        let model = sm_model(SMRestrict::Default);
        assert_eq!(model.weighted_search.max_interaction_weighted, 2);
        let quartic_qed = model.vertices.values().any(|v| {
            v.particles.len() == 4
                && v.couplings
                    .values()
                    .any(|&c| model.coupling_def(c).orders == orders(&[("QED", 2)]))
        });
        assert!(quartic_qed, "the SM has a QED=2 four-point vertex");
    }

    /// The authored toy's `yw^4` vertex weighs 7 over four legs, 3.5 per
    /// equivalent cubic vertex, and its `fw~ fw xw` vertex 3; MadGraph truncates
    /// the larger to 3. Rounding would give 4, and no division 7.
    #[test]
    fn the_toy_cap_truncates_the_largest_interaction_weighted() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/vibegraph_toy_weighted_UFO");
        let model = UFOModel::load(&dir, None).unwrap();
        let search = &model.weighted_search;
        assert_eq!(search.max_interaction_weighted, 3);
        assert_eq!(search.maximum_weighted(4), 6);
        // Every vertex carries QED, so the NP class (value 1) reaches nothing.
        let everything: BTreeSet<i64> = [9000111, -9000111, 9000122, 9000135, 9000136]
            .into_iter()
            .collect();
        assert_eq!(
            search.particle_classes,
            vec![(1, BTreeSet::new()), (2, everything)]
        );
    }

    /// Powers multiply the hierarchy; an order the hierarchy omits adds nothing,
    /// and an interaction carrying one is in no class (its other orders still
    /// count towards the cap).
    #[test]
    fn an_unlisted_order_weighs_nothing_and_joins_no_class() {
        let h = hierarchy(&[("QCD", 1), ("QED", 2)]);
        assert_eq!(order_weight(&orders(&[("QCD", 3), ("QED", 2)]), &h), 7);
        assert_eq!(order_weight(&orders(&[("QED", 1), ("LOOP", 5)]), &h), 2);
        let loop_vertex = orders(&[("QED", 1), ("LOOP", 1)]);
        let interactions = [Interaction {
            pdg_codes: vec![25, 21, 21],
            orders: &loop_vertex,
        }];
        let search = WeightedSearchModel::new(&interactions, &h);
        assert_eq!(search.max_interaction_weighted, 2);
        assert_eq!(search.particle_classes, vec![(2, BTreeSet::new())]);
    }

    fn sm_like() -> WeightedSearchModel {
        let h = hierarchy(&[("QCD", 1), ("QED", 2)]);
        let qcd = orders(&[("QCD", 1)]);
        let qed = orders(&[("QED", 1)]);
        let interactions = [
            Interaction {
                pdg_codes: vec![-1, 1, 21],
                orders: &qcd,
            },
            Interaction {
                pdg_codes: vec![-1, 1, 22],
                orders: &qed,
            },
            Interaction {
                pdg_codes: vec![-11, 11, 23],
                orders: &qed,
            },
        ];
        WeightedSearchModel::new(&interactions, &h)
    }

    #[test]
    fn a_particle_takes_the_lowest_class_it_interacts_in() {
        let search = sm_like();
        assert_eq!(
            search.particle_classes,
            vec![
                (1, [-1, 1, 21].into_iter().collect()),
                (2, [-11, 11, 22, 23].into_iter().collect()),
            ]
        );
    }

    /// `d d~ > e+ e-`: leg values 1, 1, 2, 2, the two smallest dropped. `g g > g g`
    /// sums to 2, but requiring an s-channel Z (twice its class value, 4) in
    /// place of the two largest leg values starts it at 4; offered the
    /// alternative of an s-channel gluon (2), the lighter alternative wins.
    #[test]
    fn the_start_sums_the_largest_legs_and_the_lightest_s_channel() {
        let search = sm_like();
        let d = vec![1];
        let dx = vec![-1];
        let g = vec![21];
        let e = vec![11];
        let ex = vec![-11];
        assert_eq!(
            search.minimum_weighted(&[d.clone(), dx.clone(), e.clone(), ex.clone()], &[]),
            4
        );
        let gggg = [g.clone(), g.clone(), g.clone(), g.clone()];
        assert_eq!(search.minimum_weighted(&gggg, &[]), 2);
        assert_eq!(search.minimum_weighted(&gggg, &[vec![23]]), 4);
        assert_eq!(search.minimum_weighted(&gggg, &[vec![23], vec![21]]), 2);
        // A multiparticle leg takes the lowest class any of its members is in.
        assert_eq!(search.minimum_weighted(&[vec![21, 11], ex, e, dx], &[]), 4);
    }
}
