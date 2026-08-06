//! The merge graph: which sets of external legs a process's integration
//! channels allow the clustering to combine, and what flows through the line
//! they combine into.
//!
//! MadGraph's clustering is graph-guided. A pair of lines may merge only if some
//! surviving channel contains a propagator whose subtree is exactly that pair's
//! set of external legs, and the propagator's PDG code — not the pair's — is
//! what the scale walk then asks `isqcd`/`isjet` about. `cluster.f`'s `filmap`
//! builds that lookup once per process directory from the channel forests
//! (`configs.inc`), and this module is its counterpart: [`ChannelSet`] is the
//! forests, [`MergeTable`] is the lookup.
//!
//! Three properties of the lookup are load-bearing and easy to get wrong.
//!
//! * **Both a leg set and its complement are registered.** A propagator can be
//!   found from either side of the tree, so a t-channel line between beam 1 and
//!   leg 3 is reachable as `{1,3}` and as its complement. `cluster.f:262`.
//! * **The table is a function of the integration channel**, not of the process:
//!   `cluster.f:360` drops every channel whose QCD coupling order differs from
//!   the one being integrated, so a mixed QED/QCD process has one table per
//!   coupling order.
//! * **Only the last merge of a channel that reaches the beams** takes its PDG
//!   from beam 2's own line (`cluster.f:271-272`); every other line takes it from
//!   the channel's s-channel or t-channel propagator code.

use std::collections::{BTreeMap, BTreeSet, HashMap};

/// One internal line of a channel's forest, as `configs.inc` writes it.
#[derive(Clone, Debug, PartialEq)]
pub struct ForestLine {
    /// The line's own index, negative, as the daughters of other lines name it.
    pub index: i32,
    /// `iforest(1:2, k, config)`: positive is an external leg number, negative
    /// another line of the same forest.
    pub daughters: [i32; 2],
    /// `tprid`: the magnitude of the PDG code on a spacelike line, `0` on a
    /// timelike one.
    pub tprid: i64,
    /// `sprop(iproc, k, config)`: the signed PDG code on a timelike line, per
    /// subprocess of the group, `0` on a spacelike one.
    pub sprop: Vec<i64>,
    pub mass: f64,
    pub width: f64,
}

/// One integration channel's forest, plus the coupling order that decides which
/// other channels share a merge table with it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigForest {
    /// `nqcd(config)`: the channel's QCD coupling order.
    pub nqcd: i64,
    pub lines: Vec<ForestLine>,
}

impl ConfigForest {
    fn line(&self, index: i32) -> Option<&ForestLine> {
        self.lines.iter().find(|l| l.index == index)
    }

    /// The set of external legs below `index`, as a bit per leg.
    ///
    /// Returns `None` for a forest whose daughters do not resolve, which is a
    /// malformed table rather than a physical statement.
    pub fn mask(&self, index: i32) -> Option<u32> {
        let mut mask = 0u32;
        let line = self.line(index)?;
        for daughter in line.daughters {
            if daughter > 0 {
                mask |= 1 << (daughter - 1);
            } else {
                mask |= self.mask(daughter)?;
            }
        }
        Some(mask)
    }

    /// The line no other line has as a daughter — the one the bottom-up walk
    /// merges last.
    fn root(&self) -> Option<&ForestLine> {
        self.lines.iter().find(|line| {
            !self
                .lines
                .iter()
                .any(|other| other.daughters.contains(&line.index))
        })
    }
}

/// The merge lookup for one subprocess of one process directory.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MergeTable {
    /// `id_cl`: leg set → the channels that contain a line with that leg set,
    /// ascending. `findmt` intersects these lists, which requires the order.
    pub id_cl: BTreeMap<u32, Vec<usize>>,
    /// `ipdgcl`: (leg set, channel) → the PDG code on the line.
    pub ipdgcl: BTreeMap<(u32, usize), i64>,
    /// `resmap`: (leg set, channel) pairs whose line carries a width, so an
    /// on-shell Breit-Wigner can constrain the clustering to them.
    pub resmap: BTreeSet<(u32, usize)>,
}

impl MergeTable {
    /// The channels that allow `mask`, filtered the way `findmt` filters them on
    /// its first call of a clustering: to the integration channel alone when
    /// `chcluster` is set, and to channels carrying every tagged resonance.
    pub fn seed(
        &self,
        mask: u32,
        chcluster: bool,
        this_config: usize,
        tagged: &[u32],
    ) -> Option<Vec<usize>> {
        let graphs: Vec<usize> = self
            .id_cl
            .get(&mask)?
            .iter()
            .copied()
            .filter(|&graph| !chcluster || graph == this_config)
            .filter(|&graph| {
                tagged
                    .iter()
                    .all(|&res| self.resmap.contains(&(res, graph)))
            })
            .collect();
        (!graphs.is_empty()).then_some(graphs)
    }

    /// How many channels [`seed`](Self::seed) would return. The clustering's
    /// first pass over the external pairs reads only that count and whether it
    /// is zero, so it asks here and builds no list.
    pub fn seed_count(
        &self,
        mask: u32,
        chcluster: bool,
        this_config: usize,
        tagged: &[u32],
    ) -> usize {
        self.id_cl.get(&mask).map_or(0, |graphs| {
            graphs
                .iter()
                .copied()
                .filter(|&graph| !chcluster || graph == this_config)
                .filter(|&graph| {
                    tagged
                        .iter()
                        .all(|&res| self.resmap.contains(&(res, graph)))
                })
                .count()
        })
    }

    /// The intersection `findmt` takes on every later call: the running list
    /// against the channels allowing `mask`. Both lists are ascending, so this
    /// is a merge join.
    pub fn narrow(&self, mask: u32, running: &[usize]) -> Option<Vec<usize>> {
        let allowed = self.id_cl.get(&mask)?;
        let mut graphs: Vec<usize> = Vec::new();
        let mut next = 0;
        for &graph in running {
            while next < allowed.len() && allowed[next] < graph {
                next += 1;
            }
            if next < allowed.len() && allowed[next] == graph {
                graphs.push(graph);
                next += 1;
            }
        }
        (!graphs.is_empty()).then_some(graphs)
    }
}

/// Every integration channel of one process directory, with the external
/// flavours of each subprocess in the group.
#[derive(Clone, Debug)]
pub struct ChannelSet {
    pub n_external: usize,
    pub n_incoming: usize,
    /// `configs[c - 1]` is channel `c`, numbered as `mapconfig` numbers them.
    pub configs: Vec<ConfigForest>,
    /// `external_pdg[iproc][leg - 1]`: `leshouche.inc`'s `idup`.
    pub external_pdg: Vec<Vec<i64>>,
    /// `confsub[iproc][config - 1]`: whether that subprocess contributes to that
    /// channel. A subprocess that does not gets no entries for it.
    pub contributes: Vec<Vec<bool>>,
}

impl ChannelSet {
    pub fn n_proc(&self) -> usize {
        self.external_pdg.len()
    }

    /// The full leg set, `2^nexternal - 1`.
    pub fn full_mask(&self) -> u32 {
        (1u32 << self.n_external) - 1
    }

    /// `filmap`: the merge table of each subprocess, for the channel being
    /// integrated.
    ///
    /// `this_config` enters only by naming the QCD coupling order that selects
    /// which channels contribute, so channels of one order share a table set —
    /// which is what [`MergeTablesByOrder`] hoists out of the per-event path.
    /// Every other channel-dependent step is a property of the forest being
    /// walked, including the last line taking beam 2's flavour where that
    /// forest names no propagator there.
    pub fn merge_tables(&self, this_config: usize) -> Vec<MergeTable> {
        let mut tables = vec![MergeTable::default(); self.n_proc()];
        let order = self.configs[this_config - 1].nqcd;
        for (index, forest) in self.configs.iter().enumerate() {
            let graph = index + 1;
            if forest.nqcd != order {
                continue;
            }
            // `iproc` names a subprocess, not a position in `tables` alone: the
            // body reads `self.external_pdg` at the same index, so replacing the
            // index with an iterator over `tables` would keep the index anyway.
            #[allow(clippy::needless_range_loop)]
            for iproc in 0..self.n_proc() {
                for leg in 1..=self.n_external {
                    tables[iproc]
                        .ipdgcl
                        .insert((1 << (leg - 1), graph), self.external_pdg[iproc][leg - 1]);
                }
            }
            let root = forest.root().map(|line| line.index);
            let last_level = forest.lines.len() == self.n_external - 2;
            for line in &forest.lines {
                let Some(mask) = forest.mask(line.index) else {
                    continue;
                };
                let complement = self.full_mask() - mask;
                // `iproc` names a subprocess: the body selects that subprocess's
                // row of `self.contributes` and its entry in `line.sprop` by the
                // same index, so an iterator over `tables` would not remove it.
                #[allow(clippy::needless_range_loop)]
                for iproc in 0..self.n_proc() {
                    if !self.contributes[iproc][index] {
                        continue;
                    }
                    let sprop = line.sprop.get(iproc).copied().unwrap_or(0);
                    let pdg = if sprop != 0 {
                        sprop
                    } else if line.tprid != 0 {
                        line.tprid
                    } else if last_level && root == Some(line.index) {
                        self.external_pdg[iproc][1]
                    } else {
                        0
                    };
                    for id in [mask, complement] {
                        // A single external leg keeps its own flavour. The
                        // complement of a line that reaches every leg but one is
                        // that one leg, and the reference's live table shows the
                        // leg's flavour there rather than the line's code.
                        if !id.is_power_of_two() {
                            tables[iproc].ipdgcl.insert((id, graph), pdg);
                        }
                        let entry = tables[iproc].id_cl.entry(id).or_default();
                        if !entry.contains(&graph) {
                            entry.push(graph);
                        }
                        if line.width > 0.0 {
                            tables[iproc].resmap.insert((id, graph));
                        }
                    }
                }
            }
        }
        tables
    }
}

/// The merge tables of every coupling order a channel set carries, built once.
///
/// [`ChannelSet::merge_tables`] reads the integration channel only to pick the
/// coupling order that decides which channels enter the lookup, so every channel
/// of one order shares one set of tables. The clustering asks for them on each
/// event, which is what makes building them there worth hoisting.
#[derive(Clone, Debug)]
pub struct MergeTablesByOrder {
    /// One table set per distinct coupling order, in first-appearance order.
    tables: Vec<Vec<MergeTable>>,
    /// `order_of[config - 1]`: the entry of `tables` that channel reads.
    order_of: Vec<usize>,
}

impl MergeTablesByOrder {
    pub fn build(set: &ChannelSet) -> Self {
        let mut orders: Vec<i64> = Vec::new();
        let mut first_config: Vec<usize> = Vec::new();
        let mut order_of = Vec::with_capacity(set.configs.len());
        for (index, forest) in set.configs.iter().enumerate() {
            let slot = match orders.iter().position(|&o| o == forest.nqcd) {
                Some(slot) => slot,
                None => {
                    orders.push(forest.nqcd);
                    first_config.push(index + 1);
                    orders.len() - 1
                }
            };
            order_of.push(slot);
        }
        let tables = first_config
            .iter()
            .map(|&config| set.merge_tables(config))
            .collect();
        MergeTablesByOrder { tables, order_of }
    }

    /// The tables `this_config` clusters against, numbered as `mapconfig`
    /// numbers channels.
    pub fn of(&self, this_config: usize) -> &[MergeTable] {
        &self.tables[self.order_of[this_config - 1]]
    }
}

/// PDG code → colour representation, the question `isqcd`, `isjet` and
/// `is_octet` are all asked through.
///
/// MadGraph generates this from the model as a flat table and reports colour `0`
/// for a code the model does not carry, which makes an unassigned line neither
/// coloured nor an octet; the same convention is kept here so an unassigned line
/// behaves the same way.
#[derive(Clone, Debug, Default)]
pub struct ColorTable {
    colors: HashMap<i64, i32>,
    maxjetflavor: i64,
}

impl ColorTable {
    pub fn new(colors: impl IntoIterator<Item = (i64, i32)>, maxjetflavor: i64) -> Self {
        ColorTable {
            colors: colors.into_iter().collect(),
            maxjetflavor,
        }
    }

    pub fn color(&self, pdg: i64) -> i32 {
        self.colors.get(&pdg).copied().unwrap_or(0)
    }

    /// `isqcd`: carries colour.
    pub fn is_qcd(&self, pdg: i64) -> bool {
        self.color(pdg).abs() > 1
    }

    /// `is_octet`: an adjoint line.
    pub fn is_octet(&self, pdg: i64) -> bool {
        self.color(pdg).abs() == 8
    }

    /// `isjet`: a gluon, or a quark light enough for the run card's
    /// `maxjetflavor`. Note that this asks nothing about colour, so a code the
    /// table does not carry is a jet if its magnitude is small enough.
    pub fn is_jet(&self, pdg: i64) -> bool {
        let magnitude = pdg.abs();
        magnitude <= self.maxjetflavor || magnitude == 21
    }

    pub fn maxjetflavor(&self) -> i64 {
        self.maxjetflavor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(index: i32, d1: i32, d2: i32, tprid: i64, sprop: i64, width: f64) -> ForestLine {
        ForestLine {
            index,
            daughters: [d1, d2],
            tprid,
            sprop: vec![sprop],
            mass: 0.0,
            width,
        }
    }

    /// `u ū → u ū` at `QCD = 2`: one timelike gluon channel and one spacelike
    /// one, which is the smallest table with both kinds of line in it.
    fn uux_to_uux() -> ChannelSet {
        ChannelSet {
            n_external: 4,
            n_incoming: 2,
            configs: vec![
                ConfigForest {
                    nqcd: 2,
                    lines: vec![line(-1, 4, 3, 0, 21, 0.0)],
                },
                ConfigForest {
                    nqcd: 2,
                    lines: vec![line(-1, 1, 3, 21, 0, 0.0), line(-2, -1, 4, 2, 0, 0.0)],
                },
            ],
            external_pdg: vec![vec![2, -2, 2, -2]],
            contributes: vec![vec![true, true]],
        }
    }

    #[test]
    fn a_leg_set_and_its_complement_are_both_registered() {
        let tables = uux_to_uux().merge_tables(1);
        let id_cl = &tables[0].id_cl;
        // {3,4} from the timelike channel, and {1,2} reached from the other side.
        assert_eq!(id_cl[&0b1100], vec![1]);
        assert_eq!(id_cl[&0b0011], vec![1]);
        // {1,3} from the spacelike channel, and {2,4}.
        assert_eq!(id_cl[&0b0101], vec![2]);
        assert_eq!(id_cl[&0b1010], vec![2]);
        // The spacelike channel's last line closes on beam 2, so the whole event
        // minus beam 2 is a leg set and beam 2 alone is its complement.
        assert_eq!(id_cl[&0b1101], vec![2]);
        assert_eq!(id_cl[&0b0010], vec![2]);
        assert_eq!(id_cl.len(), 6);
    }

    /// The propagator's code, not the pair's: the leg set `{3,4}` is a quark and
    /// an antiquark, and the line joining them is a gluon.
    #[test]
    fn a_line_carries_its_propagator_code() {
        let tables = uux_to_uux().merge_tables(1);
        assert_eq!(tables[0].ipdgcl[&(0b1100, 1)], 21);
        assert_eq!(tables[0].ipdgcl[&(0b0011, 1)], 21);
        assert_eq!(tables[0].ipdgcl[&(0b0101, 2)], 21);
    }

    /// A single external leg keeps its own flavour, even where it is the
    /// complement of a channel's outermost spacelike line and so is registered
    /// again with that line's code. Beam 2 here is the `ū`, not the `u` the
    /// spacelike channel's last line carries.
    #[test]
    fn a_single_leg_keeps_its_flavour() {
        let tables = uux_to_uux().merge_tables(1);
        assert_eq!(tables[0].ipdgcl[&(0b0010, 2)], -2);
        assert_eq!(tables[0].ipdgcl[&(0b0001, 2)], 2);
        // The leg set is still registered; only the code on it is the leg's.
        assert_eq!(tables[0].id_cl[&0b0010], vec![2]);
    }

    /// A channel of a different coupling order is not in the table at all, which
    /// is what makes the merge graph a property of the channel being integrated.
    #[test]
    fn the_coupling_order_of_the_integrated_channel_selects_the_table() {
        let mut set = uux_to_uux();
        set.configs.push(ConfigForest {
            nqcd: 0,
            lines: vec![line(-1, 4, 3, 0, 23, 2.44)],
        });
        set.contributes[0].push(true);
        let qcd = set.merge_tables(1);
        assert_eq!(qcd[0].id_cl[&0b1100], vec![1]);
        let qed = set.merge_tables(3);
        assert_eq!(qed[0].id_cl[&0b1100], vec![3]);
        assert!(!qed[0].id_cl.contains_key(&0b0101));
        // A line with a width is what an on-shell resonance can be required to
        // pass through.
        assert!(qed[0].resmap.contains(&(0b1100, 3)));
        assert!(qcd[0].resmap.is_empty());
    }

    /// Hoisting the tables out of the per-event path keys them on the coupling
    /// order and nothing else, so a set carrying two orders must still hand each
    /// channel the table `merge_tables` would have built for it — and the two
    /// orders' tables must differ, or the keying would be untested.
    #[test]
    fn hoisted_tables_are_keyed_on_the_coupling_order() {
        let mut set = uux_to_uux();
        set.configs.push(ConfigForest {
            nqcd: 0,
            lines: vec![line(-1, 4, 3, 0, 23, 2.44)],
        });
        set.contributes[0].push(true);
        let hoisted = MergeTablesByOrder::build(&set);
        for config in 1..=set.configs.len() {
            assert_eq!(
                hoisted.of(config),
                set.merge_tables(config).as_slice(),
                "channel {config}"
            );
        }
        assert_ne!(hoisted.of(1), hoisted.of(3));
    }
}

#[cfg(test)]
mod single_leg_entries {
    use super::*;
    use crate::coupling::cluster::configs::derive_channels;
    use crate::coupling::cluster::kt::{Channel, ClusterSettings};
    use crate::coupling::cluster::setclscales::{setclscales, JetMemo, ScaleSettings};
    use crate::diagrams::{generate_from_proc_card, parse_proc_card, ParsingOptions};
    use crate::ufo::particles::ParticleId;
    use crate::ufo::sm::{sm_model, SMRestrict};
    use crate::ufo::EvaluatedModel;

    struct Derived {
        set: ChannelSet,
        colors: ColorTable,
        beam2: i64,
    }

    fn derived(spec: &str) -> Derived {
        let model = sm_model(SMRestrict::Default);
        let evaluated = EvaluatedModel::from_model(model.clone());
        let parsed = parse_proc_card(&format!("generate {spec}"), &ParsingOptions::default())
            .expect("proc card parses");
        let sets = generate_from_proc_card(&parsed, model.as_ref()).expect("enumerate");
        let set = sets
            .iter()
            .find(|s| !s.diagrams.is_empty())
            .expect("a non-empty subprocess");
        let externals: Vec<ParticleId> = set
            .particles_in
            .iter()
            .chain(set.particles_out.iter())
            .map(|name| model.particle_id(name).expect("external in model"))
            .collect();
        Derived {
            beam2: model.particle(externals[1]).pdg_code,
            set: derive_channels(
                &set.diagrams,
                &externals,
                set.particles_in.len(),
                model.as_ref(),
                &evaluated,
            )
            .expect("channel forests")
            .set,
            colors: ColorTable::new(
                model
                    .particles
                    .values()
                    .map(|p| (p.pdg_code, p.color))
                    .collect::<Vec<(i64, i32)>>(),
                4,
            ),
        }
    }

    /// Why keeping a single external leg's own flavour costs nothing.
    ///
    /// `filgrp` registers each line under its leg set *and* its complement
    /// (`cluster.f:262`), writing the same PDG to both, and MadGraph's live
    /// `ipdgcl` gives a single-leg mask the subprocess flavour instead. Reading
    /// the Fortran, those two look like different tables; they are not, and this
    /// is the reason.
    ///
    /// A line's complement is a single external leg exactly when the line has
    /// `nexternal - 1` legs below it, and only one line ever does: the vertex
    /// that closes a channel on the beams, whose complement is beam 2 alone. An
    /// s-channel-only channel does not even write that vertex, so it has no
    /// single-leg complement at all. And `configs.inc` gives the closing line
    /// `tprid = abs(leg 2's own id)` (`export_v4.py:2262`, where the vertex's
    /// last leg *is* beam 2) — so the code the complement rule would write is the
    /// leg's own code, up to sign.
    ///
    /// Everything the clustering asks a line's code — `isqcd`, `isjet`,
    /// `is_octet` — is a question about `abs(pdg)`, so the two readings are the
    /// same table wherever they are read. [`a_single_leg_keeps_its_flavour`]
    /// pins the signed form, which is what `ipartupdate`'s flavour propagation
    /// then mutates and what the instrumented dump's per-event `LINE` records
    /// compare against.
    #[test]
    fn only_the_closing_line_can_write_a_single_leg_entry() {
        for spec in [
            "u u~ > u u~",
            "g g > g g",
            "b b~ > b b~",
            "u d~ > u d~",
            "e+ e- > mu+ mu-",
        ] {
            let d = derived(spec);
            let n = d.set.n_external;
            for (index, forest) in d.set.configs.iter().enumerate() {
                for line in &forest.lines {
                    let mask = forest.mask(line.index).expect("a line that resolves");
                    let complement = d.set.full_mask() - mask;
                    if !complement.is_power_of_two() {
                        continue;
                    }
                    // The only single-leg complement is beam 2's, and it belongs
                    // to the line that closes the channel on the beams.
                    assert_eq!(complement, 1 << 1, "{spec}: channel {}", index + 1);
                    assert_eq!(mask.count_ones() as usize, n - 1);
                    assert_eq!(line.tprid, d.beam2.abs(), "{spec}: the closing line's code");
                    assert_eq!(line.sprop[0], 0);
                }
            }
        }
    }

    /// The consequence, measured rather than argued: overwriting beam 2's entry
    /// with the closing line's code moves no scale.
    ///
    /// `b b̄ → b b̄` at `maxjetflavor = 4` is where the two readings would be
    /// furthest apart if they could be — the exchanged gluon is a jet and the
    /// beams are not — and the scale is identical to the bit. If it were not,
    /// the choice this module makes would need an oracle it does not have.
    #[test]
    fn overwriting_a_single_leg_entry_moves_no_scale() {
        let d = derived("b b~ > b b~");
        assert!(!d.colors.is_jet(-5), "the comparison needs a non-jet beam");
        assert!(d.colors.is_jet(21));
        let spacelike = d
            .set
            .configs
            .iter()
            .position(|c| c.lines.iter().any(|l| l.tprid == 21))
            .expect("a t-channel gluon channel")
            + 1;
        let behaviour = d.set.merge_tables(spacelike);
        let mut reading = behaviour.clone();
        let beam2 = 1u32 << 1;
        for graph in reading[0].id_cl[&beam2].clone() {
            reading[0].ipdgcl.insert((beam2, graph), 21);
        }
        assert_ne!(
            behaviour[0].ipdgcl[&(beam2, spacelike)],
            reading[0].ipdgcl[&(beam2, spacelike)]
        );

        let p = [
            [3000.0, 0.0, 0.0, 3000.0],
            [2000.0, 0.0, 0.0, -2000.0],
            [2400.0, 900.0, 0.0, 2225.398840_f64],
            [2600.0, -900.0, 0.0, -1225.398840_f64],
        ];
        let scale_of = |table: &MergeTable| {
            let channel = Channel {
                set: &d.set,
                table,
                colors: &d.colors,
                this_config: spacelike,
                iproc: 1,
            };
            setclscales(
                &channel,
                &ClusterSettings::default(),
                &ScaleSettings::default(),
                &p,
                &mut JetMemo::default(),
                false,
                &[],
                (0.0, [0.0; 2]),
                false,
            )
            .map(|s| (s.mu_r, s.q2fact, s.jcode, s.mur_branch, s.muf_branch))
        };
        assert_eq!(
            scale_of(&behaviour[0]).expect("a scale"),
            scale_of(&reading[0]).expect("a scale")
        );
    }
}
